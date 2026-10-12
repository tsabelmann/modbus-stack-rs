use crate::function::{DecodeError, Malformed};
use crate::{FunctionCode, FunctionKind, PduData};

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

        if !(Self::MIN_QUANTITY_OF_REGISTERS..=Self::MAX_QUANTITY_OF_REGISTERS).contains(&quantity_of_registers) {
            return Err(Malformed::Quantity.into());
        }

        Ok(Request { starting_address, quantity_of_registers })
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
    use crate::function::test_util::Pdu;

    #[test]
    fn decode_001() {
        // Example from the specification
        let request = Request::decode(&Pdu(&[0x04, 0x00, 0x08, 0x00, 0x01])).unwrap();
        assert_eq!(request.function_code(), FunctionKind::Normal(FunctionCode::ReadInputRegisters));
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
        assert_eq!(result, Err(DecodeError::UnexpectedFunction(FunctionKind::Normal(FunctionCode::ReadHoldingRegisters))));
    }

    #[test]
    fn decode_008() {
        // Exception
        let result = Request::decode(&Pdu(&[0x84, 0x02]));
        assert_eq!(result, Err(DecodeError::UnexpectedFunction(FunctionKind::Exception(FunctionCode::ReadInputRegisters))));
    }
}
