pub mod read_holding_registers;
pub mod read_input_registers;
pub mod write_multiple_registers;
pub mod write_single_register;

use crate::{ExceptionCode, FrameEncoder, FunctionCode, FunctionKind};

// ── DecodeError ──

/// Error returned when decoding a PDU.
///
/// Distinguishes between a PDU that belongs to a different function
/// ([`DecodeError::UnexpectedFunction`]) and a PDU that belongs to the
/// function but violates the specification ([`DecodeError::Malformed`]).
/// Only the latter maps to an [`ExceptionCode`].
#[non_exhaustive]
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum DecodeError {
    /// The PDU belongs to a different function, or is an exception where none is allowed.
    UnexpectedFunction(FunctionKind),
    /// The PDU belongs to the function, but violates the specification.
    Malformed(Malformed),
}

impl From<Malformed> for DecodeError {
    fn from(value: Malformed) -> Self {
        DecodeError::Malformed(value)
    }
}

// ── Malformed ──

/// Specification violation of a PDU.
#[non_exhaustive]
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Malformed {
    /// The PDU is too short or too long.
    Length,
    /// The quantity is outside of the allowed range.
    Quantity,
    /// The byte count does not match the quantity or the length.
    ByteCount,
}

impl Malformed {
    /// Exception code a server responds with.
    pub const fn exception_code(&self) -> ExceptionCode {
        match self {
            Malformed::Length | Malformed::Quantity | Malformed::ByteCount => {
                ExceptionCode::IllegalDataValue
            }
        }
    }
}

impl From<Malformed> for ExceptionCode {
    fn from(value: Malformed) -> Self {
        value.exception_code()
    }
}

// ── EncodeError ──

/// Error returned when encoding a PDU.
#[non_exhaustive]
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum EncodeError {
    /// The frame has not enough space for the PDU.
    BufferTooSmall,
    /// The values violate the specification.
    Malformed(Malformed),
}

impl From<Malformed> for EncodeError {
    fn from(value: Malformed) -> Self {
        EncodeError::Malformed(value)
    }
}

/// Encodes an exception response for the given function.
pub fn encode_exception(
    frame: &mut dyn FrameEncoder,
    function: FunctionCode,
    code: ExceptionCode,
) -> Result<(), EncodeError> {
    let Some(payload) = frame.payload_mut().first_mut() else {
        return Err(EncodeError::BufferTooSmall);
    };
    *payload = u8::from(code);

    frame.finalize(FunctionKind::Exception(function), 1);
    Ok(())
}

// ── Registers ──

/// Iterator over register values.
///
/// Either backed by big endian encoded bytes (decoding) or by register values (encoding).
#[derive(Debug, Clone)]
pub struct Registers<'a> {
    source: Source<'a>,
}

#[derive(Debug, Clone)]
enum Source<'a> {
    /// Big endian encoded register values.
    Bytes(&'a [u8]),
    /// Register values.
    Values(&'a [u16]),
}

impl<'a> Registers<'a> {
    pub(crate) const fn new(data: &'a [u8]) -> Registers<'a> {
        Registers {
            source: Source::Bytes(data),
        }
    }

    /// Writes the register values into `dest` and returns the number of written values.
    #[inline]
    pub fn write_to(&self, dest: &mut [u16]) -> usize {
        let mut count = 0;
        for (dest, value) in dest.iter_mut().zip(self.clone()) {
            *dest = value;
            count += 1;
        }
        count
    }

    /// Splits after `n` register values; `n` is clamped to the number of values.
    #[inline]
    pub(crate) fn split_at(self, n: usize) -> (Registers<'a>, Registers<'a>) {
        let (head, tail) = match self.source {
            Source::Bytes(data) => {
                let (head, tail) = data.split_at(n.saturating_mul(2).min(data.len()));
                (Source::Bytes(head), Source::Bytes(tail))
            }
            Source::Values(values) => {
                let (head, tail) = values.split_at(n.min(values.len()));
                (Source::Values(head), Source::Values(tail))
            }
        };
        (Registers { source: head }, Registers { source: tail })
    }
}

impl<'a> From<&'a [u16]> for Registers<'a> {
    #[inline]
    fn from(values: &'a [u16]) -> Self {
        Registers {
            source: Source::Values(values),
        }
    }
}

impl<'a> PartialEq for Registers<'a> {
    fn eq(&self, other: &Self) -> bool {
        // compares the values, not the source
        self.clone().eq(other.clone())
    }
}

impl<'a> Eq for Registers<'a> {}

impl<'a> Iterator for Registers<'a> {
    type Item = u16;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        match &mut self.source {
            Source::Bytes(data) => {
                let (value, rest) = data.split_first_chunk::<2>()?;
                *data = rest;
                Some(u16::from_be_bytes(*value))
            }
            Source::Values(values) => {
                let (value, rest) = values.split_first()?;
                *values = rest;
                Some(*value)
            }
        }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = match &self.source {
            Source::Bytes(data) => data.len() / 2,
            Source::Values(values) => values.len(),
        };
        (remaining, Some(remaining))
    }
}

impl<'a> ExactSizeIterator for Registers<'a> {}

#[cfg(test)]
pub(crate) mod test_util {
    use super::EncodeError;
    use crate::constants::MODBUS_RTU_FRAME_DATA_LENGTH;
    use crate::rtu::{ModbusRtuFrame, check_rtu_crc};
    use crate::tcp::Frame;
    use crate::{AduData, ExceptionCode, FrameEncoder, FunctionKind, PduData};

    /// PDU backed by a plain byte slice.
    pub(crate) struct Pdu<'a>(pub(crate) &'a [u8]);

    impl<'a> PduData for Pdu<'a> {
        fn pdu_data(&self) -> &[u8] {
            self.0
        }
    }

    /// Frame encoder with a payload capacity of exactly `N` bytes.
    pub(crate) struct SmallFrame<const N: usize> {
        pub(crate) payload: [u8; N],
        pub(crate) finalized: Option<(FunctionKind, usize)>,
    }

    impl<const N: usize> SmallFrame<N> {
        pub(crate) fn new() -> SmallFrame<N> {
            SmallFrame {
                payload: [0; N],
                finalized: None,
            }
        }
    }

    impl<const N: usize> FrameEncoder for SmallFrame<N> {
        fn payload_mut(&mut self) -> &mut [u8] {
            &mut self.payload
        }

        fn finalize(&mut self, code: FunctionKind, payload_len: usize) {
            assert!(payload_len <= N, "payload exceeds frame capacity");
            self.finalized = Some((code, payload_len));
        }
    }

    /// All exception codes, including unknown ones.
    pub(crate) const EXCEPTION_CODES: [ExceptionCode; 12] = [
        ExceptionCode::IllegalFunction,
        ExceptionCode::IllegalDataAddress,
        ExceptionCode::IllegalDataValue,
        ExceptionCode::ServerDeviceFailure,
        ExceptionCode::Acknowledge,
        ExceptionCode::ServerDeviceBusy,
        ExceptionCode::MemoryParityError,
        ExceptionCode::GatewayPathUnavailable,
        ExceptionCode::GatewayTargetFailedToRespond,
        ExceptionCode::Unknown(0x00),
        ExceptionCode::Unknown(0x42),
        ExceptionCode::Unknown(0xFF),
    ];

    /// Addresses and values that expose byte order and sign errors.
    pub(crate) const EDGE_VALUES: [u16; 7] =
        [0x0000, 0x0001, 0x00FF, 0x7FFF, 0x8000, 0xFF00, 0xFFFF];

    /// `N` distinct register values, starting with `0x0000`, `0x00FF`, `0xFF00` and `0xFFFF`.
    pub(crate) fn distinct_values<const N: usize>() -> [u16; N] {
        core::array::from_fn(|i| match i {
            0 => 0x0000,
            1 => 0x00FF,
            2 => 0xFF00,
            3 => 0xFFFF,
            _ => (i as u16) * 0x0101 + 1,
        })
    }

    /// Encodes into a TCP and an RTU frame, checks both frames and calls `check` with each of them.
    pub(crate) fn roundtrip(
        encode: impl Fn(&mut dyn FrameEncoder) -> Result<(), EncodeError>,
        check: impl Fn(&dyn PduData),
    ) {
        // TCP
        let mut tcp = Frame::new(0x1234, 0x11);
        encode(&mut tcp).unwrap();
        assert_eq!(tcp.length() as usize, 1 + tcp.pdu().len());
        assert_eq!(tcp.transaction_id(), 0x1234);
        assert_eq!(tcp.unit_id(), 0x11);
        check(&tcp);

        // RTU
        let mut data = [0u8; MODBUS_RTU_FRAME_DATA_LENGTH];
        let mut rtu = ModbusRtuFrame::new(&mut data, 0x11);
        encode(&mut rtu).unwrap();
        let adu = rtu.adu_data();
        let crc_index = adu.len() - 2;
        assert_eq!(adu[0], 0x11);
        assert!(check_rtu_crc(
            &adu[..crc_index],
            adu[crc_index],
            adu[crc_index + 1]
        ));
        check(&rtu);

        // Same PDU for both transports
        assert_eq!(rtu.pdu_data(), tcp.pdu());
    }
}

#[cfg(test)]
mod function_tests {
    use super::*;

    #[test]
    fn registers_split_at_001() {
        let (head, tail) = Registers::new(&[0x00, 0x01, 0x00, 0x02, 0x00, 0x03]).split_at(1);
        assert_eq!(head.collect::<std::vec::Vec<_>>(), [1]);
        assert_eq!(tail.collect::<std::vec::Vec<_>>(), [2, 3]);

        let (head, tail) = Registers::from(&[1, 2, 3][..]).split_at(2);
        assert_eq!(head.collect::<std::vec::Vec<_>>(), [1, 2]);
        assert_eq!(tail.collect::<std::vec::Vec<_>>(), [3]);
    }

    #[test]
    fn registers_split_at_002() {
        let (head, tail) = Registers::new(&[0x00, 0x01]).split_at(5);
        assert_eq!(head.len(), 1);
        assert_eq!(tail.len(), 0);

        let (head, tail) = Registers::from(&[1, 2][..]).split_at(usize::MAX);
        assert_eq!(head.len(), 2);
        assert_eq!(tail.len(), 0);

        let (head, tail) = Registers::from(&[1, 2][..]).split_at(0);
        assert_eq!(head.len(), 0);
        assert_eq!(tail.len(), 2);
    }

    #[test]
    fn registers_001() {
        let registers = Registers::new(&[0x02, 0x2B, 0x00, 0x00, 0x00, 0x64]);
        assert_eq!(registers.len(), 3);
        assert_eq!(
            registers.collect::<std::vec::Vec<_>>(),
            [0x022B, 0x0000, 0x0064]
        );
    }

    #[test]
    fn registers_002() {
        let mut registers = Registers::new(&[]);
        assert_eq!(registers.len(), 0);
        assert_eq!(registers.next(), None);
    }

    #[test]
    fn registers_003() {
        let registers = Registers::new(&[0x00, 0x01, 0x00, 0x02, 0x00, 0x03]);

        let mut dest = [0u16; 2];
        assert_eq!(registers.write_to(&mut dest), 2);
        assert_eq!(dest, [1, 2]);

        let mut dest = [0u16; 4];
        assert_eq!(registers.write_to(&mut dest), 3);
        assert_eq!(dest, [1, 2, 3, 0]);
    }

    #[test]
    fn registers_004() {
        // Values and bytes with the same content are equal
        let values = [0x022B, 0x0000, 0x0064];
        let registers = Registers::from(&values[..]);
        assert_eq!(registers.len(), 3);
        assert_eq!(
            registers,
            Registers::new(&[0x02, 0x2B, 0x00, 0x00, 0x00, 0x64])
        );
        assert_ne!(registers, Registers::new(&[0x02, 0x2B, 0x00, 0x00]));
        assert_eq!(registers.collect::<std::vec::Vec<_>>(), values);
    }

    #[test]
    fn encode_exception_001() {
        let mut frame = crate::tcp::Frame::new(0x0001, 0x11);
        encode_exception(
            &mut frame,
            FunctionCode::ReadCoils,
            ExceptionCode::IllegalFunction,
        )
        .unwrap();
        assert_eq!(
            frame.as_bytes(),
            &[0x00, 0x01, 0x00, 0x00, 0x00, 0x03, 0x11, 0x81, 0x01]
        );
    }

    #[test]
    fn malformed_001() {
        assert_eq!(
            ExceptionCode::from(Malformed::Length),
            ExceptionCode::IllegalDataValue
        );
        assert_eq!(
            ExceptionCode::from(Malformed::Quantity),
            ExceptionCode::IllegalDataValue
        );
        assert_eq!(
            ExceptionCode::from(Malformed::ByteCount),
            ExceptionCode::IllegalDataValue
        );
    }

    #[test]
    fn encode_exception_002() {
        // Every exception code for an arbitrary function (TCP and RTU)
        for code in test_util::EXCEPTION_CODES {
            test_util::roundtrip(
                |frame| encode_exception(frame, FunctionCode::ReadCoils, code),
                |pdu| {
                    assert_eq!(
                        pdu.function_code(),
                        FunctionKind::Exception(FunctionCode::ReadCoils)
                    );
                    assert_eq!(pdu.pdu_data(), &[0x81, u8::from(code)]);
                },
            );
        }
    }

    #[test]
    fn encode_exception_003() {
        // Capacity of zero bytes
        let mut frame = test_util::SmallFrame::<0>::new();
        let result = encode_exception(
            &mut frame,
            FunctionCode::ReadCoils,
            ExceptionCode::IllegalFunction,
        );
        assert_eq!(result, Err(EncodeError::BufferTooSmall));
        assert_eq!(frame.finalized, None);

        // Capacity of exactly one byte
        let mut frame = test_util::SmallFrame::<1>::new();
        let result = encode_exception(
            &mut frame,
            FunctionCode::ReadCoils,
            ExceptionCode::IllegalFunction,
        );
        assert_eq!(result, Ok(()));
        assert_eq!(
            frame.finalized,
            Some((FunctionKind::Exception(FunctionCode::ReadCoils), 1))
        );
        assert_eq!(frame.payload, [0x01]);
    }

    #[test]
    fn encode_exception_004() {
        // Unknown exception code with a known value: the bytes are stable,
        // decoding returns the named variant
        let response =
            crate::read_holding_registers::Response::new_exception(ExceptionCode::Unknown(0x02));
        test_util::roundtrip(
            |frame| response.encode(frame),
            |pdu| {
                assert_eq!(pdu.pdu_data(), &[0x83, 0x02]);
                assert_eq!(
                    crate::read_holding_registers::Response::decode(pdu),
                    Ok(crate::read_holding_registers::Response::new_exception(
                        ExceptionCode::IllegalDataAddress
                    ))
                );
            },
        );
    }

    #[test]
    fn distinct_values_001() {
        // Test helper produces distinct values
        let values = test_util::distinct_values::<125>();
        for (i, a) in values.iter().enumerate() {
            assert!(!values[i + 1..].contains(a));
        }
    }
}
