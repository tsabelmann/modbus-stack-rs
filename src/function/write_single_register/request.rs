use crate::function::{DecodeError, EncodeError, Malformed};
use crate::{FrameEncoder, FunctionCode, FunctionKind, PduData};

// ── Request ──

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct Request {
    register_address: u16,
    register_value: u16,
}

impl Request {
    /// Number of bytes in the PDU (function code, register address, register value).
    pub const PDU_LENGTH: usize = 5;

    pub const fn new(register_address: u16, register_value: u16) -> Request {
        Request {
            register_address,
            register_value,
        }
    }

    pub fn decode(pdu: &dyn PduData) -> Result<Request, DecodeError> {
        match pdu.function_code() {
            FunctionKind::Normal(FunctionCode::WriteSingleRegister) => {}
            code => return Err(DecodeError::UnexpectedFunction(code)),
        }

        let data = pdu.pdu_data();
        if data.len() != Self::PDU_LENGTH {
            return Err(Malformed::Length.into());
        }

        let register_address = u16::from_be_bytes([data[1], data[2]]);
        let register_value = u16::from_be_bytes([data[3], data[4]]);

        Ok(Request {
            register_address,
            register_value,
        })
    }

    pub fn encode(&self, frame: &mut dyn FrameEncoder) -> Result<(), EncodeError> {
        let payload_len = Self::PDU_LENGTH - 1;
        let Some(payload) = frame.payload_mut().get_mut(..payload_len) else {
            return Err(EncodeError::BufferTooSmall);
        };
        payload[..2].copy_from_slice(&self.register_address.to_be_bytes());
        payload[2..].copy_from_slice(&self.register_value.to_be_bytes());

        frame.finalize(self.function_code(), payload_len);
        Ok(())
    }

    pub fn function_code(&self) -> FunctionKind {
        FunctionKind::Normal(FunctionCode::WriteSingleRegister)
    }

    pub fn register_address(&self) -> u16 {
        self.register_address
    }

    pub fn register_value(&self) -> u16 {
        self.register_value
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

    #[test]
    fn decode_001() {
        // Example from the specification
        let request = Request::decode(&Pdu(&[0x06, 0x00, 0x01, 0x00, 0x03])).unwrap();
        assert_eq!(
            request.function_code(),
            FunctionKind::Normal(FunctionCode::WriteSingleRegister)
        );
        assert_eq!(request.register_address(), 0x0001);
        assert_eq!(request.register_value(), 0x0003);
    }

    #[test]
    fn decode_002() {
        // Full value range
        let request = Request::decode(&Pdu(&[0x06, 0xFF, 0xFF, 0xFF, 0xFF])).unwrap();
        assert_eq!(request.register_address(), 0xFFFF);
        assert_eq!(request.register_value(), 0xFFFF);
    }

    #[test]
    fn decode_003() {
        // Too short
        let result = Request::decode(&Pdu(&[0x06, 0x00, 0x01, 0x00]));
        assert_eq!(result, Err(DecodeError::Malformed(Malformed::Length)));
    }

    #[test]
    fn decode_004() {
        // Too long
        let result = Request::decode(&Pdu(&[0x06, 0x00, 0x01, 0x00, 0x03, 0x00]));
        assert_eq!(result, Err(DecodeError::Malformed(Malformed::Length)));
    }

    #[test]
    fn decode_005() {
        // Read holding registers request
        let result = Request::decode(&Pdu(&[0x03, 0x00, 0x01, 0x00, 0x03]));
        assert_eq!(
            result,
            Err(DecodeError::UnexpectedFunction(FunctionKind::Normal(
                FunctionCode::ReadHoldingRegisters
            )))
        );
    }

    #[test]
    fn decode_006() {
        // Exception
        let result = Request::decode(&Pdu(&[0x86, 0x02]));
        assert_eq!(
            result,
            Err(DecodeError::UnexpectedFunction(FunctionKind::Exception(
                FunctionCode::WriteSingleRegister
            )))
        );
    }

    #[test]
    fn decode_007() {
        // Wrong function code takes precedence over length
        let result = Request::decode(&Pdu(&[0x03]));
        assert_eq!(
            result,
            Err(DecodeError::UnexpectedFunction(FunctionKind::Normal(
                FunctionCode::ReadHoldingRegisters
            )))
        );
    }

    #[test]
    fn encode_001() {
        // Example from the specification (TCP)
        let mut frame = Frame::new(0x0001, 0x11);
        Request::new(0x0001, 0x0003).encode(&mut frame).unwrap();
        assert_eq!(
            frame.as_bytes(),
            &[
                0x00, 0x01, 0x00, 0x00, 0x00, 0x06, 0x11, 0x06, 0x00, 0x01, 0x00, 0x03
            ]
        );
        assert_eq!(Request::decode(&frame), Ok(Request::new(0x0001, 0x0003)));
    }

    #[test]
    fn encode_002() {
        // Example from the specification (RTU)
        let mut data = [0u8; MODBUS_RTU_FRAME_DATA_LENGTH];
        let mut frame = ModbusRtuFrame::new(&mut data, 0x11);
        Request::new(0x0001, 0x0003).encode(&mut frame).unwrap();

        let adu = frame.adu_data();
        assert_eq!(&adu[..6], &[0x11, 0x06, 0x00, 0x01, 0x00, 0x03]);
        assert!(check_rtu_crc(&adu[..6], adu[6], adu[7]));
        assert_eq!(Request::decode(&frame), Ok(Request::new(0x0001, 0x0003)));
    }
}
