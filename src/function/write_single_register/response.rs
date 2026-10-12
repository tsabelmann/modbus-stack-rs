use crate::function::{DecodeError, EncodeError, Malformed, encode_exception};
use crate::{ExceptionCode, FrameEncoder, FunctionCode, FunctionKind, PduData};

// ── Response ──

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Response {
    Normal {
        register_address: u16,
        register_value: u16,
    },
    Exception(ExceptionCode),
}

impl Response {
    /// Number of bytes in the PDU (function code, register address, register value).
    pub const PDU_LENGTH: usize = 5;

    pub const fn new(register_address: u16, register_value: u16) -> Response {
        Response::Normal {
            register_address,
            register_value,
        }
    }

    pub const fn new_exception(code: ExceptionCode) -> Response {
        Response::Exception(code)
    }

    pub fn decode(pdu: &dyn PduData) -> Result<Response, DecodeError> {
        let data = pdu.pdu_data();

        match pdu.function_code() {
            FunctionKind::Normal(FunctionCode::WriteSingleRegister) => {
                if data.len() != Self::PDU_LENGTH {
                    return Err(Malformed::Length.into());
                }

                let register_address = u16::from_be_bytes([data[1], data[2]]);
                let register_value = u16::from_be_bytes([data[3], data[4]]);

                Ok(Response::Normal {
                    register_address,
                    register_value,
                })
            }
            FunctionKind::Exception(FunctionCode::WriteSingleRegister) => {
                if data.len() != 2 {
                    return Err(Malformed::Length.into());
                }

                Ok(Response::Exception(ExceptionCode::from(data[1])))
            }
            code => Err(DecodeError::UnexpectedFunction(code)),
        }
    }

    pub fn encode(&self, frame: &mut dyn FrameEncoder) -> Result<(), EncodeError> {
        match self {
            Response::Normal {
                register_address,
                register_value,
            } => {
                let payload_len = Self::PDU_LENGTH - 1;
                let Some(payload) = frame.payload_mut().get_mut(..payload_len) else {
                    return Err(EncodeError::BufferTooSmall);
                };
                payload[..2].copy_from_slice(&register_address.to_be_bytes());
                payload[2..].copy_from_slice(&register_value.to_be_bytes());

                frame.finalize(self.function_code(), payload_len);
                Ok(())
            }
            Response::Exception(code) => {
                encode_exception(frame, FunctionCode::WriteSingleRegister, *code)
            }
        }
    }

    pub fn function_code(&self) -> FunctionKind {
        match self {
            Response::Normal { .. } => FunctionKind::Normal(FunctionCode::WriteSingleRegister),
            Response::Exception(_) => FunctionKind::Exception(FunctionCode::WriteSingleRegister),
        }
    }
}

#[cfg(test)]
mod response_tests {
    use super::*;
    use crate::AduData;
    use crate::constants::MODBUS_RTU_FRAME_DATA_LENGTH;
    use crate::function::test_util::Pdu;
    use crate::rtu::{ModbusRtuFrame, check_rtu_crc};
    use crate::tcp::Frame;

    #[test]
    fn decode_001() {
        // Example from the specification
        let response = Response::decode(&Pdu(&[0x06, 0x00, 0x01, 0x00, 0x03])).unwrap();
        assert_eq!(
            response,
            Response::Normal {
                register_address: 0x0001,
                register_value: 0x0003
            }
        );
        assert_eq!(
            response.function_code(),
            FunctionKind::Normal(FunctionCode::WriteSingleRegister)
        );
    }

    #[test]
    fn decode_002() {
        // Exception
        let response = Response::decode(&Pdu(&[0x86, 0x02])).unwrap();
        assert_eq!(
            response,
            Response::Exception(ExceptionCode::IllegalDataAddress)
        );
        assert_eq!(
            response.function_code(),
            FunctionKind::Exception(FunctionCode::WriteSingleRegister)
        );
    }

    #[test]
    fn decode_003() {
        // Too short / too long
        assert_eq!(
            Response::decode(&Pdu(&[0x06, 0x00, 0x01, 0x00])),
            Err(DecodeError::Malformed(Malformed::Length))
        );
        assert_eq!(
            Response::decode(&Pdu(&[0x06, 0x00, 0x01, 0x00, 0x03, 0x00])),
            Err(DecodeError::Malformed(Malformed::Length))
        );
    }

    #[test]
    fn decode_004() {
        // Exception too short / too long
        assert_eq!(
            Response::decode(&Pdu(&[0x86])),
            Err(DecodeError::Malformed(Malformed::Length))
        );
        assert_eq!(
            Response::decode(&Pdu(&[0x86, 0x02, 0x00])),
            Err(DecodeError::Malformed(Malformed::Length))
        );
    }

    #[test]
    fn decode_005() {
        // Write multiple registers response
        let result = Response::decode(&Pdu(&[0x10, 0x00, 0x01, 0x00, 0x02]));
        assert_eq!(
            result,
            Err(DecodeError::UnexpectedFunction(FunctionKind::Normal(
                FunctionCode::WriteMultipleRegisters
            )))
        );
    }

    #[test]
    fn decode_006() {
        // Exception of a different function
        let result = Response::decode(&Pdu(&[0x83, 0x02]));
        assert_eq!(
            result,
            Err(DecodeError::UnexpectedFunction(FunctionKind::Exception(
                FunctionCode::ReadHoldingRegisters
            )))
        );
    }

    #[test]
    fn encode_001() {
        // Example from the specification (TCP)
        let mut frame = Frame::new(0x0001, 0x11);
        Response::new(0x0001, 0x0003).encode(&mut frame).unwrap();
        assert_eq!(
            frame.as_bytes(),
            &[
                0x00, 0x01, 0x00, 0x00, 0x00, 0x06, 0x11, 0x06, 0x00, 0x01, 0x00, 0x03
            ]
        );
        assert_eq!(Response::decode(&frame), Ok(Response::new(0x0001, 0x0003)));
    }

    #[test]
    fn encode_002() {
        // Example from the specification (RTU)
        let mut data = [0u8; MODBUS_RTU_FRAME_DATA_LENGTH];
        let mut frame = ModbusRtuFrame::new(&mut data, 0x11);
        Response::new(0x0001, 0x0003).encode(&mut frame).unwrap();

        let adu = frame.adu_data();
        assert_eq!(&adu[..6], &[0x11, 0x06, 0x00, 0x01, 0x00, 0x03]);
        assert!(check_rtu_crc(&adu[..6], adu[6], adu[7]));
        assert_eq!(Response::decode(&frame), Ok(Response::new(0x0001, 0x0003)));
    }

    #[test]
    fn encode_003() {
        // Exception (TCP and RTU)
        let response = Response::new_exception(ExceptionCode::IllegalDataAddress);

        let mut frame = Frame::new(0x0001, 0x11);
        response.encode(&mut frame).unwrap();
        assert_eq!(
            frame.as_bytes(),
            &[0x00, 0x01, 0x00, 0x00, 0x00, 0x03, 0x11, 0x86, 0x02]
        );
        assert_eq!(Response::decode(&frame), Ok(response));

        let mut data = [0u8; MODBUS_RTU_FRAME_DATA_LENGTH];
        let mut frame = ModbusRtuFrame::new(&mut data, 0x11);
        response.encode(&mut frame).unwrap();
        let adu = frame.adu_data();
        assert_eq!(&adu[..3], &[0x11, 0x86, 0x02]);
        assert!(check_rtu_crc(&adu[..3], adu[3], adu[4]));
        assert_eq!(Response::decode(&frame), Ok(response));
    }
}
