pub mod read_holding_registers;
pub mod read_input_registers;
pub mod write_single_register;
pub mod write_multiple_registers;

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
            Malformed::Length | Malformed::Quantity | Malformed::ByteCount => ExceptionCode::IllegalDataValue,
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
pub fn encode_exception(frame: &mut dyn FrameEncoder, function: FunctionCode, code: ExceptionCode) -> Result<(), EncodeError> {
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
        Registers { source: Source::Bytes(data) }
    }

    /// Writes the register values into `dest` and returns the number of written values.
    pub fn write_to(&self, dest: &mut [u16]) -> usize {
        let mut count = 0;
        for (dest, value) in dest.iter_mut().zip(self.clone()) {
            *dest = value;
            count += 1;
        }
        count
    }
}

impl<'a> From<&'a [u16]> for Registers<'a> {
    fn from(values: &'a [u16]) -> Self {
        Registers { source: Source::Values(values) }
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
    use crate::PduData;

    /// PDU backed by a plain byte slice.
    pub(crate) struct Pdu<'a>(pub(crate) &'a [u8]);

    impl<'a> PduData for Pdu<'a> {
        fn pdu_data(&self) -> &[u8] {
            self.0
        }
    }
}

#[cfg(test)]
mod function_tests {
    use super::*;

    #[test]
    fn registers_001() {
        let registers = Registers::new(&[0x02, 0x2B, 0x00, 0x00, 0x00, 0x64]);
        assert_eq!(registers.len(), 3);
        assert_eq!(registers.collect::<std::vec::Vec<_>>(), [0x022B, 0x0000, 0x0064]);
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
        assert_eq!(registers, Registers::new(&[0x02, 0x2B, 0x00, 0x00, 0x00, 0x64]));
        assert_ne!(registers, Registers::new(&[0x02, 0x2B, 0x00, 0x00]));
        assert_eq!(registers.collect::<std::vec::Vec<_>>(), values);
    }

    #[test]
    fn encode_exception_001() {
        let mut frame = crate::tcp::Frame::new(0x0001, 0x11);
        encode_exception(&mut frame, FunctionCode::ReadCoils, ExceptionCode::IllegalFunction).unwrap();
        assert_eq!(frame.as_bytes(), &[0x00, 0x01, 0x00, 0x00, 0x00, 0x03, 0x11, 0x81, 0x01]);
    }

    #[test]
    fn malformed_001() {
        assert_eq!(ExceptionCode::from(Malformed::Length), ExceptionCode::IllegalDataValue);
        assert_eq!(ExceptionCode::from(Malformed::Quantity), ExceptionCode::IllegalDataValue);
        assert_eq!(ExceptionCode::from(Malformed::ByteCount), ExceptionCode::IllegalDataValue);
    }
}
