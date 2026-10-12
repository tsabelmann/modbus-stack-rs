use crate::function::{DecodeError, Malformed};
use crate::{ExceptionCode, FunctionCode, FunctionKind, PduData};

// ── Response ──

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Response {
    Normal {
        starting_address: u16,
        quantity_of_registers: u16,
    },
    Exception(ExceptionCode),
}

impl Response {
    pub const MIN_QUANTITY_OF_REGISTERS: u16 = 1;
    pub const MAX_QUANTITY_OF_REGISTERS: u16 = 123;

    /// Number of bytes in the PDU (function code, starting address, quantity of registers).
    pub const PDU_LENGTH: usize = 5;

    pub fn decode(pdu: &dyn PduData) -> Result<Response, DecodeError> {
        let data = pdu.pdu_data();

        match pdu.function_code() {
            FunctionKind::Normal(FunctionCode::WriteMultipleRegisters) => {
                if data.len() != Self::PDU_LENGTH {
                    return Err(Malformed::Length.into());
                }

                let starting_address = u16::from_be_bytes([data[1], data[2]]);
                let quantity_of_registers = u16::from_be_bytes([data[3], data[4]]);

                if !(Self::MIN_QUANTITY_OF_REGISTERS..=Self::MAX_QUANTITY_OF_REGISTERS).contains(&quantity_of_registers) {
                    return Err(Malformed::Quantity.into());
                }

                Ok(Response::Normal { starting_address, quantity_of_registers })
            }
            FunctionKind::Exception(FunctionCode::WriteMultipleRegisters) => {
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
            Response::Normal { .. } => FunctionKind::Normal(FunctionCode::WriteMultipleRegisters),
            Response::Exception(_) => FunctionKind::Exception(FunctionCode::WriteMultipleRegisters),
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
        let response = Response::decode(&Pdu(&[0x10, 0x00, 0x01, 0x00, 0x02])).unwrap();
        assert_eq!(response, Response::Normal { starting_address: 0x0001, quantity_of_registers: 2 });
        assert_eq!(response.function_code(), FunctionKind::Normal(FunctionCode::WriteMultipleRegisters));
    }

    #[test]
    fn decode_002() {
        // Exception
        let response = Response::decode(&Pdu(&[0x90, 0x02])).unwrap();
        assert_eq!(response, Response::Exception(ExceptionCode::IllegalDataAddress));
        assert_eq!(response.function_code(), FunctionKind::Exception(FunctionCode::WriteMultipleRegisters));
    }

    #[test]
    fn decode_003() {
        // Too short / too long
        assert_eq!(Response::decode(&Pdu(&[0x10, 0x00, 0x01, 0x00])), Err(DecodeError::Malformed(Malformed::Length)));
        assert_eq!(Response::decode(&Pdu(&[0x10, 0x00, 0x01, 0x00, 0x02, 0x00])), Err(DecodeError::Malformed(Malformed::Length)));
    }

    #[test]
    fn decode_004() {
        // Exception too short / too long
        assert_eq!(Response::decode(&Pdu(&[0x90])), Err(DecodeError::Malformed(Malformed::Length)));
        assert_eq!(Response::decode(&Pdu(&[0x90, 0x02, 0x00])), Err(DecodeError::Malformed(Malformed::Length)));
    }

    #[test]
    fn decode_005() {
        // Quantity 0, 123 and 124
        assert_eq!(Response::decode(&Pdu(&[0x10, 0x00, 0x01, 0x00, 0x00])), Err(DecodeError::Malformed(Malformed::Quantity)));
        assert_eq!(
            Response::decode(&Pdu(&[0x10, 0x00, 0x01, 0x00, 0x7B])),
            Ok(Response::Normal { starting_address: 0x0001, quantity_of_registers: 123 })
        );
        assert_eq!(Response::decode(&Pdu(&[0x10, 0x00, 0x01, 0x00, 0x7C])), Err(DecodeError::Malformed(Malformed::Quantity)));
    }

    #[test]
    fn decode_006() {
        // Write single register response
        let result = Response::decode(&Pdu(&[0x06, 0x00, 0x01, 0x00, 0x03]));
        assert_eq!(result, Err(DecodeError::UnexpectedFunction(FunctionKind::Normal(FunctionCode::WriteSingleRegister))));
    }

    #[test]
    fn decode_007() {
        // Exception of a different function
        let result = Response::decode(&Pdu(&[0x86, 0x02]));
        assert_eq!(result, Err(DecodeError::UnexpectedFunction(FunctionKind::Exception(FunctionCode::WriteSingleRegister))));
    }
}
