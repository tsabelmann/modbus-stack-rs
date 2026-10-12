use crate::function::{DecodeError, EncodeError, Malformed};
use crate::{FrameEncoder, FunctionCode, FunctionKind, PduData};

// ── Request ──

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct Request {
    starting_address: u16,
    quantity_of_registers: u16,
}

impl Request {
    pub const MIN_QUANTITY_OF_REGISTERS: u16 = 1;
    pub const MAX_QUANTITY_OF_REGISTERS: u16 = 125;

    /// Number of bytes in the PDU (function code, starting address, quantity of registers).
    pub const PDU_LENGTH: usize = 5;

    pub const fn new(starting_address: u16, quantity_of_registers: u16) -> Request {
        Request {
            starting_address,
            quantity_of_registers,
        }
    }

    pub fn decode(pdu: &dyn PduData) -> Result<Request, DecodeError> {
        match pdu.function_code() {
            FunctionKind::Normal(FunctionCode::ReadInputRegisters) => {}
            code => return Err(DecodeError::UnexpectedFunction(code)),
        }

        let data = pdu.pdu_data();
        if data.len() != Self::PDU_LENGTH {
            return Err(Malformed::Length.into());
        }

        let starting_address = u16::from_be_bytes([data[1], data[2]]);
        let quantity_of_registers = u16::from_be_bytes([data[3], data[4]]);

        if !(Self::MIN_QUANTITY_OF_REGISTERS..=Self::MAX_QUANTITY_OF_REGISTERS)
            .contains(&quantity_of_registers)
        {
            return Err(Malformed::Quantity.into());
        }

        Ok(Request {
            starting_address,
            quantity_of_registers,
        })
    }

    pub fn encode(&self, frame: &mut dyn FrameEncoder) -> Result<(), EncodeError> {
        if !(Self::MIN_QUANTITY_OF_REGISTERS..=Self::MAX_QUANTITY_OF_REGISTERS)
            .contains(&self.quantity_of_registers)
        {
            return Err(Malformed::Quantity.into());
        }

        let payload_len = Self::PDU_LENGTH - 1;
        let Some(payload) = frame.payload_mut().get_mut(..payload_len) else {
            return Err(EncodeError::BufferTooSmall);
        };
        payload[..2].copy_from_slice(&self.starting_address.to_be_bytes());
        payload[2..].copy_from_slice(&self.quantity_of_registers.to_be_bytes());

        frame.finalize(self.function_code(), payload_len);
        Ok(())
    }

    pub fn function_code(&self) -> FunctionKind {
        FunctionKind::Normal(FunctionCode::ReadInputRegisters)
    }

    pub fn starting_address(&self) -> u16 {
        self.starting_address
    }

    pub fn quantity_of_registers(&self) -> u16 {
        self.quantity_of_registers
    }
}

#[cfg(test)]
mod request_tests {
    use super::*;
    use crate::AduData;
    use crate::constants::MODBUS_RTU_FRAME_DATA_LENGTH;
    use crate::function::test_util::{self, Pdu, SmallFrame};
    use crate::rtu::{ModbusRtuFrame, check_rtu_crc};
    use crate::tcp::Frame;

    #[test]
    fn decode_001() {
        // Example from the specification
        let request = Request::decode(&Pdu(&[0x04, 0x00, 0x08, 0x00, 0x01])).unwrap();
        assert_eq!(
            request.function_code(),
            FunctionKind::Normal(FunctionCode::ReadInputRegisters)
        );
        assert_eq!(request.starting_address(), 0x0008);
        assert_eq!(request.quantity_of_registers(), 1);
    }

    #[test]
    fn decode_002() {
        // Too short
        let result = Request::decode(&Pdu(&[0x04, 0x00, 0x08, 0x00]));
        assert_eq!(result, Err(DecodeError::Malformed(Malformed::Length)));
    }

    #[test]
    fn decode_003() {
        // Too long
        let result = Request::decode(&Pdu(&[0x04, 0x00, 0x08, 0x00, 0x01, 0x00]));
        assert_eq!(result, Err(DecodeError::Malformed(Malformed::Length)));
    }

    #[test]
    fn decode_004() {
        // Quantity 0
        let result = Request::decode(&Pdu(&[0x04, 0x00, 0x00, 0x00, 0x00]));
        assert_eq!(result, Err(DecodeError::Malformed(Malformed::Quantity)));
    }

    #[test]
    fn decode_005() {
        // Quantity 125
        let request = Request::decode(&Pdu(&[0x04, 0x00, 0x00, 0x00, 0x7D])).unwrap();
        assert_eq!(request.quantity_of_registers(), 125);
    }

    #[test]
    fn decode_006() {
        // Quantity 126
        let result = Request::decode(&Pdu(&[0x04, 0x00, 0x00, 0x00, 0x7E]));
        assert_eq!(result, Err(DecodeError::Malformed(Malformed::Quantity)));
    }

    #[test]
    fn decode_007() {
        // Read holding registers request
        let result = Request::decode(&Pdu(&[0x03, 0x00, 0x08, 0x00, 0x01]));
        assert_eq!(
            result,
            Err(DecodeError::UnexpectedFunction(FunctionKind::Normal(
                FunctionCode::ReadHoldingRegisters
            )))
        );
    }

    #[test]
    fn decode_008() {
        // Exception
        let result = Request::decode(&Pdu(&[0x84, 0x02]));
        assert_eq!(
            result,
            Err(DecodeError::UnexpectedFunction(FunctionKind::Exception(
                FunctionCode::ReadInputRegisters
            )))
        );
    }

    #[test]
    fn encode_001() {
        // Example from the specification (TCP)
        let mut frame = Frame::new(0x0001, 0x11);
        Request::new(0x0008, 1).encode(&mut frame).unwrap();
        assert_eq!(
            frame.as_bytes(),
            &[
                0x00, 0x01, 0x00, 0x00, 0x00, 0x06, 0x11, 0x04, 0x00, 0x08, 0x00, 0x01
            ]
        );
        assert_eq!(Request::decode(&frame), Ok(Request::new(0x0008, 1)));
    }

    #[test]
    fn encode_002() {
        // Example from the specification (RTU)
        let mut data = [0u8; MODBUS_RTU_FRAME_DATA_LENGTH];
        let mut frame = ModbusRtuFrame::new(&mut data, 0x11);
        Request::new(0x0008, 1).encode(&mut frame).unwrap();

        let adu = frame.adu_data();
        assert_eq!(&adu[..6], &[0x11, 0x04, 0x00, 0x08, 0x00, 0x01]);
        assert!(check_rtu_crc(&adu[..6], adu[6], adu[7]));
        assert_eq!(Request::decode(&frame), Ok(Request::new(0x0008, 1)));
    }

    #[test]
    fn encode_003() {
        // Quantity 0, 125 and 126
        let mut frame = Frame::new(0x0001, 0x11);
        assert_eq!(
            Request::new(0, 0).encode(&mut frame),
            Err(EncodeError::Malformed(Malformed::Quantity))
        );
        assert_eq!(Request::new(0, 125).encode(&mut frame), Ok(()));
        assert_eq!(
            Request::new(0, 126).encode(&mut frame),
            Err(EncodeError::Malformed(Malformed::Quantity))
        );
    }

    #[test]
    fn edge_001() {
        // Quantity 1 and 125 with lowest and highest address (TCP and RTU)
        for starting_address in [0x0000, 0xFFFF] {
            for quantity in [1, 125] {
                let request = Request::new(starting_address, quantity);
                test_util::roundtrip(
                    |frame| request.encode(frame),
                    |pdu| assert_eq!(Request::decode(pdu), Ok(request)),
                );
            }
        }
    }

    #[test]
    fn edge_002() {
        // Address range beyond 0xFFFF is passed through (checked by the data model)
        let request = Request::new(0xFFFF, 2);
        test_util::roundtrip(
            |frame| request.encode(frame),
            |pdu| {
                assert_eq!(pdu.pdu_data(), &[0x04, 0xFF, 0xFF, 0x00, 0x02]);
                assert_eq!(Request::decode(pdu), Ok(request));
            },
        );
    }

    #[test]
    fn edge_003() {
        // Byte order of address and quantity
        let mut frame = Frame::new(0x0001, 0x11);
        Request::new(0x1234, 0x0056).encode(&mut frame).unwrap();
        assert_eq!(frame.pdu(), &[0x04, 0x12, 0x34, 0x00, 0x56]);
    }

    #[test]
    fn edge_004() {
        // Capacity one byte too small and exactly fitting
        let request = Request::new(0x006B, 3);

        let mut frame = SmallFrame::<3>::new();
        assert_eq!(request.encode(&mut frame), Err(EncodeError::BufferTooSmall));
        assert_eq!(frame.finalized, None);

        let mut frame = SmallFrame::<4>::new();
        assert_eq!(request.encode(&mut frame), Ok(()));
        assert_eq!(frame.finalized, Some((request.function_code(), 4)));
        assert_eq!(frame.payload, [0x00, 0x6B, 0x00, 0x03]);
    }

    #[test]
    fn edge_005() {
        // Failed encoding leaves the frame unchanged (TCP and RTU)
        let mut frame = Frame::new(0x0001, 0x11);
        Request::new(0x006B, 3).encode(&mut frame).unwrap();
        let before = frame.clone();
        assert_eq!(
            Request::new(0x006B, 0).encode(&mut frame),
            Err(EncodeError::Malformed(Malformed::Quantity))
        );
        assert_eq!(
            Request::new(0x006B, 126).encode(&mut frame),
            Err(EncodeError::Malformed(Malformed::Quantity))
        );
        assert_eq!(frame, before);

        let mut data = [0u8; MODBUS_RTU_FRAME_DATA_LENGTH];
        let mut frame = ModbusRtuFrame::new(&mut data, 0x11);
        Request::new(0x006B, 3).encode(&mut frame).unwrap();
        let before = frame.adu_data().to_vec();
        assert_eq!(
            Request::new(0x006B, 0).encode(&mut frame),
            Err(EncodeError::Malformed(Malformed::Quantity))
        );
        assert_eq!(frame.adu_data(), &before[..]);
    }

    #[test]
    fn edge_006() {
        // Decode and encode again yields the same bytes
        let bytes = [0x04, 0xFF, 0x00, 0x00, 0x7D];
        let request = Request::decode(&Pdu(&bytes)).unwrap();
        let mut frame = Frame::new(0x0001, 0x11);
        request.encode(&mut frame).unwrap();
        assert_eq!(frame.pdu(), &bytes);
    }
}
