use crate::function::{DecodeError, Malformed};
use crate::{ExceptionCode, FunctionCode, FunctionKind, PduData};

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

    pub fn decode(pdu: &dyn PduData) -> Result<Response, DecodeError> {
        let data = pdu.pdu_data();

        match pdu.function_code() {
            FunctionKind::Normal(FunctionCode::WriteSingleRegister) => {
                if data.len() != Self::PDU_LENGTH {
                    return Err(Malformed::Length.into());
                }

                let register_address = u16::from_be_bytes([data[1], data[2]]);
                let register_value = u16::from_be_bytes([data[3], data[4]]);

                Ok(Response::Normal { register_address, register_value })
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
    use crate::function::test_util::Pdu;

    #[test]
    fn decode_001() {
        // Example from the specification
        let response = Response::decode(&Pdu(&[0x06, 0x00, 0x01, 0x00, 0x03])).unwrap();
        assert_eq!(response, Response::Normal { register_address: 0x0001, register_value: 0x0003 });
        assert_eq!(response.function_code(), FunctionKind::Normal(FunctionCode::WriteSingleRegister));
    }

    #[test]
    fn decode_002() {
        // Exception
        let response = Response::decode(&Pdu(&[0x86, 0x02])).unwrap();
        assert_eq!(response, Response::Exception(ExceptionCode::IllegalDataAddress));
        assert_eq!(response.function_code(), FunctionKind::Exception(FunctionCode::WriteSingleRegister));
    }

    #[test]
    fn decode_003() {
        // Too short / too long
        assert_eq!(Response::decode(&Pdu(&[0x06, 0x00, 0x01, 0x00])), Err(DecodeError::Malformed(Malformed::Length)));
        assert_eq!(Response::decode(&Pdu(&[0x06, 0x00, 0x01, 0x00, 0x03, 0x00])), Err(DecodeError::Malformed(Malformed::Length)));
    }

    #[test]
    fn decode_004() {
        // Exception too short / too long
        assert_eq!(Response::decode(&Pdu(&[0x86])), Err(DecodeError::Malformed(Malformed::Length)));
        assert_eq!(Response::decode(&Pdu(&[0x86, 0x02, 0x00])), Err(DecodeError::Malformed(Malformed::Length)));
    }

    #[test]
    fn decode_005() {
        // Write multiple registers response
        let result = Response::decode(&Pdu(&[0x10, 0x00, 0x01, 0x00, 0x02]));
        assert_eq!(result, Err(DecodeError::UnexpectedFunction(FunctionKind::Normal(FunctionCode::WriteMultipleRegisters))));
    }

    #[test]
    fn decode_006() {
        // Exception of a different function
        let result = Response::decode(&Pdu(&[0x83, 0x02]));
        assert_eq!(result, Err(DecodeError::UnexpectedFunction(FunctionKind::Exception(FunctionCode::ReadHoldingRegisters))));
    }
}
