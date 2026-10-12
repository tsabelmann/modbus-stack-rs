use crate::function::{DecodeError, Malformed, Registers};
use crate::{ExceptionCode, FunctionCode, FunctionKind, PduData};

// ── Response ──

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Response<'a> {
    Normal(Registers<'a>),
    Exception(ExceptionCode),
}

impl<'a> Response<'a> {
    pub const MIN_BYTE_COUNT: u8 = 2;
    pub const MAX_BYTE_COUNT: u8 = 250;

    pub fn decode(pdu: &'a dyn PduData) -> Result<Response<'a>, DecodeError> {
        let data = pdu.pdu_data();

        match pdu.function_code() {
            FunctionKind::Normal(FunctionCode::ReadHoldingRegisters) => {
                let Some(&byte_count) = data.get(1) else {
                    return Err(Malformed::Length.into());
                };

                if !(Self::MIN_BYTE_COUNT..=Self::MAX_BYTE_COUNT).contains(&byte_count) || byte_count % 2 != 0 {
                    return Err(Malformed::ByteCount.into());
                }

                if data.len() != 2 + byte_count as usize {
                    return Err(Malformed::Length.into());
                }

                Ok(Response::Normal(Registers::new(&data[2..])))
            }
            FunctionKind::Exception(FunctionCode::ReadHoldingRegisters) => {
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
            Response::Normal(_) => FunctionKind::Normal(FunctionCode::ReadHoldingRegisters),
            Response::Exception(_) => FunctionKind::Exception(FunctionCode::ReadHoldingRegisters),
        }
    }
}

#[cfg(test)]
mod response_tests {
    use super::*;
    use crate::function::test_util::Pdu;
    use std::vec::Vec;

    #[test]
    fn decode_001() {
        // Example from the specification
        let pdu = Pdu(&[0x03, 0x06, 0x02, 0x2B, 0x00, 0x00, 0x00, 0x64]);
        let response = Response::decode(&pdu).unwrap();
        assert_eq!(response.function_code(), FunctionKind::Normal(FunctionCode::ReadHoldingRegisters));

        let Response::Normal(registers) = response else {
            panic!("expected normal response");
        };
        assert_eq!(registers.len(), 3);
        assert_eq!(registers.collect::<Vec<_>>(), [0x022B, 0x0000, 0x0064]);
    }

    #[test]
    fn decode_002() {
        // Exception
        let response = Response::decode(&Pdu(&[0x83, 0x02])).unwrap();
        assert_eq!(response, Response::Exception(ExceptionCode::IllegalDataAddress));
        assert_eq!(response.function_code(), FunctionKind::Exception(FunctionCode::ReadHoldingRegisters));
    }

    #[test]
    fn decode_003() {
        // Exception too short / too long
        assert_eq!(Response::decode(&Pdu(&[0x83])), Err(DecodeError::Malformed(Malformed::Length)));
        assert_eq!(Response::decode(&Pdu(&[0x83, 0x02, 0x00])), Err(DecodeError::Malformed(Malformed::Length)));
    }

    #[test]
    fn decode_004() {
        // Missing byte count
        assert_eq!(Response::decode(&Pdu(&[0x03])), Err(DecodeError::Malformed(Malformed::Length)));
    }

    #[test]
    fn decode_005() {
        // Data too short / too long
        assert_eq!(Response::decode(&Pdu(&[0x03, 0x04, 0x00, 0x01, 0x00])), Err(DecodeError::Malformed(Malformed::Length)));
        assert_eq!(Response::decode(&Pdu(&[0x03, 0x02, 0x00, 0x01, 0x00])), Err(DecodeError::Malformed(Malformed::Length)));
    }

    #[test]
    fn decode_006() {
        // Byte count 0, odd and above maximum
        assert_eq!(Response::decode(&Pdu(&[0x03, 0x00])), Err(DecodeError::Malformed(Malformed::ByteCount)));
        assert_eq!(Response::decode(&Pdu(&[0x03, 0x03, 0x00, 0x01, 0x00])), Err(DecodeError::Malformed(Malformed::ByteCount)));
        assert_eq!(Response::decode(&Pdu(&[0x03, 0xFC])), Err(DecodeError::Malformed(Malformed::ByteCount)));
    }

    #[test]
    fn decode_007() {
        // Maximum byte count
        let mut data = [0u8; 252];
        data[0] = 0x03;
        data[1] = 250;
        let pdu = Pdu(&data);
        let Ok(Response::Normal(registers)) = Response::decode(&pdu) else {
            panic!("expected normal response");
        };
        assert_eq!(registers.len(), 125);
    }

    #[test]
    fn decode_008() {
        // Write single register response
        let result = Response::decode(&Pdu(&[0x06, 0x00, 0x01, 0x00, 0x03]));
        assert_eq!(result, Err(DecodeError::UnexpectedFunction(FunctionKind::Normal(FunctionCode::WriteSingleRegister))));
    }

    #[test]
    fn decode_009() {
        // Exception of a different function
        let result = Response::decode(&Pdu(&[0x86, 0x02]));
        assert_eq!(result, Err(DecodeError::UnexpectedFunction(FunctionKind::Exception(FunctionCode::WriteSingleRegister))));
    }
}
