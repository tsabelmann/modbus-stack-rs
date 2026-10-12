use crate::function::{DecodeError, EncodeError, Malformed, Registers};
use crate::{FrameEncoder, FunctionCode, FunctionKind, PduData};

// ── Request ──

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Request<'a> {
    starting_address: u16,
    registers: Registers<'a>,
}

impl<'a> Request<'a> {
    pub const MIN_QUANTITY_OF_REGISTERS: u16 = 1;
    pub const MAX_QUANTITY_OF_REGISTERS: u16 = 123;

    /// Number of bytes before the register values (function code, starting address,
    /// quantity of registers, byte count).
    pub const HEADER_LENGTH: usize = 6;

    pub fn new(starting_address: u16, values: &'a [u16]) -> Request<'a> {
        Request {
            starting_address,
            registers: Registers::from(values),
        }
    }

    pub fn decode(pdu: &'a dyn PduData) -> Result<Request<'a>, DecodeError> {
        match pdu.function_code() {
            FunctionKind::Normal(FunctionCode::WriteMultipleRegisters) => {}
            code => return Err(DecodeError::UnexpectedFunction(code)),
        }

        let data = pdu.pdu_data();
        if data.len() < Self::HEADER_LENGTH {
            return Err(Malformed::Length.into());
        }

        let starting_address = u16::from_be_bytes([data[1], data[2]]);
        let quantity_of_registers = u16::from_be_bytes([data[3], data[4]]);
        let byte_count = data[5] as usize;

        if !(Self::MIN_QUANTITY_OF_REGISTERS..=Self::MAX_QUANTITY_OF_REGISTERS)
            .contains(&quantity_of_registers)
        {
            return Err(Malformed::Quantity.into());
        }

        if byte_count != 2 * quantity_of_registers as usize {
            return Err(Malformed::ByteCount.into());
        }

        if data.len() != Self::HEADER_LENGTH + byte_count {
            return Err(Malformed::Length.into());
        }

        let registers = Registers::new(&data[Self::HEADER_LENGTH..]);
        Ok(Request {
            starting_address,
            registers,
        })
    }

    pub fn encode(&self, frame: &mut dyn FrameEncoder) -> Result<(), EncodeError> {
        let quantity = self.registers.len();
        let range =
            Self::MIN_QUANTITY_OF_REGISTERS as usize..=Self::MAX_QUANTITY_OF_REGISTERS as usize;
        if !range.contains(&quantity) {
            return Err(Malformed::Quantity.into());
        }

        let byte_count = 2 * quantity;
        let payload_len = Self::HEADER_LENGTH - 1 + byte_count;
        let Some(payload) = frame.payload_mut().get_mut(..payload_len) else {
            return Err(EncodeError::BufferTooSmall);
        };

        // Starting address, quantity of registers and byte count
        payload[..2].copy_from_slice(&self.starting_address.to_be_bytes());
        payload[2..4].copy_from_slice(&(quantity as u16).to_be_bytes());
        payload[4] = byte_count as u8;

        // Register values
        for (bytes, value) in payload[5..]
            .as_chunks_mut::<2>()
            .0
            .iter_mut()
            .zip(self.registers())
        {
            bytes.copy_from_slice(&value.to_be_bytes());
        }

        frame.finalize(self.function_code(), payload_len);
        Ok(())
    }

    pub fn function_code(&self) -> FunctionKind {
        FunctionKind::Normal(FunctionCode::WriteMultipleRegisters)
    }

    pub fn starting_address(&self) -> u16 {
        self.starting_address
    }

    pub fn quantity_of_registers(&self) -> u16 {
        self.registers.len() as u16
    }

    pub fn byte_count(&self) -> u8 {
        (2 * self.registers.len()) as u8
    }

    pub fn registers(&self) -> Registers<'a> {
        self.registers.clone()
    }
}

#[cfg(test)]
mod request_tests {
    use super::*;
    use crate::AduData;
    use crate::constants::MODBUS_RTU_FRAME_DATA_LENGTH;
    use crate::function::test_util::Pdu;
    use crate::rtu::{ModbusRtuFrame, check_rtu_crc};
    use crate::tcp::Frame;
    use std::vec::Vec;

    #[test]
    fn decode_001() {
        // Example from the specification
        let pdu = Pdu(&[0x10, 0x00, 0x01, 0x00, 0x02, 0x04, 0x00, 0x0A, 0x01, 0x02]);
        let request = Request::decode(&pdu).unwrap();
        assert_eq!(
            request.function_code(),
            FunctionKind::Normal(FunctionCode::WriteMultipleRegisters)
        );
        assert_eq!(request.starting_address(), 0x0001);
        assert_eq!(request.quantity_of_registers(), 2);
        assert_eq!(request.byte_count(), 4);
        assert_eq!(request.registers().len(), 2);
        assert_eq!(request.registers().collect::<Vec<_>>(), [0x000A, 0x0102]);

        let mut dest = [0u16; 2];
        assert_eq!(request.registers().write_to(&mut dest), 2);
        assert_eq!(dest, [0x000A, 0x0102]);
    }

    #[test]
    fn decode_002() {
        // Header too short
        let result = Request::decode(&Pdu(&[0x10, 0x00, 0x01, 0x00, 0x02]));
        assert_eq!(result, Err(DecodeError::Malformed(Malformed::Length)));
    }

    #[test]
    fn decode_003() {
        // Values too short / too long
        let result = Request::decode(&Pdu(&[
            0x10, 0x00, 0x01, 0x00, 0x02, 0x04, 0x00, 0x0A, 0x01,
        ]));
        assert_eq!(result, Err(DecodeError::Malformed(Malformed::Length)));

        let result = Request::decode(&Pdu(&[
            0x10, 0x00, 0x01, 0x00, 0x02, 0x04, 0x00, 0x0A, 0x01, 0x02, 0x00,
        ]));
        assert_eq!(result, Err(DecodeError::Malformed(Malformed::Length)));
    }

    #[test]
    fn decode_004() {
        // Quantity 0 and 124
        let result = Request::decode(&Pdu(&[0x10, 0x00, 0x01, 0x00, 0x00, 0x00]));
        assert_eq!(result, Err(DecodeError::Malformed(Malformed::Quantity)));

        let result = Request::decode(&Pdu(&[0x10, 0x00, 0x01, 0x00, 0x7C, 0xF8]));
        assert_eq!(result, Err(DecodeError::Malformed(Malformed::Quantity)));
    }

    #[test]
    fn decode_005() {
        // Quantity 123
        let mut data = [0u8; 6 + 246];
        data[..6].copy_from_slice(&[0x10, 0x00, 0x00, 0x00, 0x7B, 0xF6]);
        let pdu = Pdu(&data);
        let request = Request::decode(&pdu).unwrap();
        assert_eq!(request.quantity_of_registers(), 123);
        assert_eq!(request.byte_count(), 246);
        assert_eq!(request.registers().len(), 123);
    }

    #[test]
    fn decode_006() {
        // Byte count does not match quantity
        let result = Request::decode(&Pdu(&[
            0x10, 0x00, 0x01, 0x00, 0x02, 0x03, 0x00, 0x0A, 0x01,
        ]));
        assert_eq!(result, Err(DecodeError::Malformed(Malformed::ByteCount)));
    }

    #[test]
    fn decode_007() {
        // Byte count 250..=255 must not overflow
        for byte_count in 250..=255u8 {
            let data = [0x10, 0x00, 0x01, 0x00, 0x7B, byte_count];
            let pdu = Pdu(&data);
            assert_eq!(
                Request::decode(&pdu),
                Err(DecodeError::Malformed(Malformed::ByteCount))
            );
        }
    }

    #[test]
    fn decode_008() {
        // Write single register request
        let result = Request::decode(&Pdu(&[0x06, 0x00, 0x01, 0x00, 0x03]));
        assert_eq!(
            result,
            Err(DecodeError::UnexpectedFunction(FunctionKind::Normal(
                FunctionCode::WriteSingleRegister
            )))
        );
    }

    #[test]
    fn decode_009() {
        // Exception
        let result = Request::decode(&Pdu(&[0x90, 0x02]));
        assert_eq!(
            result,
            Err(DecodeError::UnexpectedFunction(FunctionKind::Exception(
                FunctionCode::WriteMultipleRegisters
            )))
        );
    }

    #[test]
    fn encode_001() {
        // Example from the specification (TCP)
        let values = [0x000A, 0x0102];
        let mut frame = Frame::new(0x0001, 0x11);
        Request::new(0x0001, &values).encode(&mut frame).unwrap();
        assert_eq!(
            frame.as_bytes(),
            &[
                0x00, 0x01, 0x00, 0x00, 0x00, 0x0B, 0x11, 0x10, 0x00, 0x01, 0x00, 0x02, 0x04, 0x00,
                0x0A, 0x01, 0x02
            ]
        );

        let request = Request::decode(&frame).unwrap();
        assert_eq!(request, Request::new(0x0001, &values));
        assert_eq!(request.quantity_of_registers(), 2);
        assert_eq!(request.byte_count(), 4);
    }

    #[test]
    fn encode_002() {
        // Example from the specification (RTU)
        let values = [0x000A, 0x0102];
        let mut data = [0u8; MODBUS_RTU_FRAME_DATA_LENGTH];
        let mut frame = ModbusRtuFrame::new(&mut data, 0x11);
        Request::new(0x0001, &values).encode(&mut frame).unwrap();

        let adu = frame.adu_data();
        assert_eq!(
            &adu[..11],
            &[
                0x11, 0x10, 0x00, 0x01, 0x00, 0x02, 0x04, 0x00, 0x0A, 0x01, 0x02
            ]
        );
        assert!(check_rtu_crc(&adu[..11], adu[11], adu[12]));
        assert_eq!(Request::decode(&frame), Ok(Request::new(0x0001, &values)));
    }

    #[test]
    fn encode_003() {
        // Quantity 0, 123 and 124
        let values = [0u16; 124];
        let mut frame = Frame::new(0x0001, 0x11);
        assert_eq!(
            Request::new(0, &[]).encode(&mut frame),
            Err(EncodeError::Malformed(Malformed::Quantity))
        );
        assert_eq!(Request::new(0, &values[..123]).encode(&mut frame), Ok(()));
        assert_eq!(frame.pdu().len(), 6 + 246);
        assert_eq!(
            Request::new(0, &values).encode(&mut frame),
            Err(EncodeError::Malformed(Malformed::Quantity))
        );
    }
}
