use crate::function::{DecodeError, EncodeError, Malformed, encode_exception};
use crate::{ExceptionCode, FrameEncoder, FunctionCode, FunctionKind, PduData};

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

    pub const fn new(starting_address: u16, quantity_of_registers: u16) -> Response {
        Response::Normal {
            starting_address,
            quantity_of_registers,
        }
    }

    pub const fn new_exception(code: ExceptionCode) -> Response {
        Response::Exception(code)
    }

    pub fn decode(pdu: &dyn PduData) -> Result<Response, DecodeError> {
        let data = pdu.pdu_data();

        match pdu.function_code() {
            FunctionKind::Normal(FunctionCode::WriteMultipleRegisters) => {
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

                Ok(Response::Normal {
                    starting_address,
                    quantity_of_registers,
                })
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

    pub fn encode(&self, frame: &mut dyn FrameEncoder) -> Result<(), EncodeError> {
        match self {
            Response::Normal {
                starting_address,
                quantity_of_registers,
            } => {
                if !(Self::MIN_QUANTITY_OF_REGISTERS..=Self::MAX_QUANTITY_OF_REGISTERS)
                    .contains(quantity_of_registers)
                {
                    return Err(Malformed::Quantity.into());
                }

                let payload_len = Self::PDU_LENGTH - 1;
                let Some(payload) = frame.payload_mut().get_mut(..payload_len) else {
                    return Err(EncodeError::BufferTooSmall);
                };
                payload[..2].copy_from_slice(&starting_address.to_be_bytes());
                payload[2..].copy_from_slice(&quantity_of_registers.to_be_bytes());

                frame.finalize(self.function_code(), payload_len);
                Ok(())
            }
            Response::Exception(code) => {
                encode_exception(frame, FunctionCode::WriteMultipleRegisters, *code)
            }
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
    use crate::AduData;
    use crate::constants::MODBUS_RTU_FRAME_DATA_LENGTH;
    use crate::function::test_util::{self, EXCEPTION_CODES, Pdu, SmallFrame};
    use crate::rtu::{ModbusRtuFrame, check_rtu_crc};
    use crate::tcp::Frame;

    #[test]
    fn decode_001() {
        // Example from the specification
        let response = Response::decode(&Pdu(&[0x10, 0x00, 0x01, 0x00, 0x02])).unwrap();
        assert_eq!(
            response,
            Response::Normal {
                starting_address: 0x0001,
                quantity_of_registers: 2
            }
        );
        assert_eq!(
            response.function_code(),
            FunctionKind::Normal(FunctionCode::WriteMultipleRegisters)
        );
    }

    #[test]
    fn decode_002() {
        // Exception
        let response = Response::decode(&Pdu(&[0x90, 0x02])).unwrap();
        assert_eq!(
            response,
            Response::Exception(ExceptionCode::IllegalDataAddress)
        );
        assert_eq!(
            response.function_code(),
            FunctionKind::Exception(FunctionCode::WriteMultipleRegisters)
        );
    }

    #[test]
    fn decode_003() {
        // Too short / too long
        assert_eq!(
            Response::decode(&Pdu(&[0x10, 0x00, 0x01, 0x00])),
            Err(DecodeError::Malformed(Malformed::Length))
        );
        assert_eq!(
            Response::decode(&Pdu(&[0x10, 0x00, 0x01, 0x00, 0x02, 0x00])),
            Err(DecodeError::Malformed(Malformed::Length))
        );
    }

    #[test]
    fn decode_004() {
        // Exception too short / too long
        assert_eq!(
            Response::decode(&Pdu(&[0x90])),
            Err(DecodeError::Malformed(Malformed::Length))
        );
        assert_eq!(
            Response::decode(&Pdu(&[0x90, 0x02, 0x00])),
            Err(DecodeError::Malformed(Malformed::Length))
        );
    }

    #[test]
    fn decode_005() {
        // Quantity 0, 123 and 124
        assert_eq!(
            Response::decode(&Pdu(&[0x10, 0x00, 0x01, 0x00, 0x00])),
            Err(DecodeError::Malformed(Malformed::Quantity))
        );
        assert_eq!(
            Response::decode(&Pdu(&[0x10, 0x00, 0x01, 0x00, 0x7B])),
            Ok(Response::Normal {
                starting_address: 0x0001,
                quantity_of_registers: 123
            })
        );
        assert_eq!(
            Response::decode(&Pdu(&[0x10, 0x00, 0x01, 0x00, 0x7C])),
            Err(DecodeError::Malformed(Malformed::Quantity))
        );
    }

    #[test]
    fn decode_006() {
        // Write single register response
        let result = Response::decode(&Pdu(&[0x06, 0x00, 0x01, 0x00, 0x03]));
        assert_eq!(
            result,
            Err(DecodeError::UnexpectedFunction(FunctionKind::Normal(
                FunctionCode::WriteSingleRegister
            )))
        );
    }

    #[test]
    fn decode_007() {
        // Exception of a different function
        let result = Response::decode(&Pdu(&[0x86, 0x02]));
        assert_eq!(
            result,
            Err(DecodeError::UnexpectedFunction(FunctionKind::Exception(
                FunctionCode::WriteSingleRegister
            )))
        );
    }

    #[test]
    fn encode_001() {
        // Example from the specification (TCP)
        let mut frame = Frame::new(0x0001, 0x11);
        Response::new(0x0001, 2).encode(&mut frame).unwrap();
        assert_eq!(
            frame.as_bytes(),
            &[
                0x00, 0x01, 0x00, 0x00, 0x00, 0x06, 0x11, 0x10, 0x00, 0x01, 0x00, 0x02
            ]
        );
        assert_eq!(Response::decode(&frame), Ok(Response::new(0x0001, 2)));
    }

    #[test]
    fn encode_002() {
        // Example from the specification (RTU)
        let mut data = [0u8; MODBUS_RTU_FRAME_DATA_LENGTH];
        let mut frame = ModbusRtuFrame::new(&mut data, 0x11);
        Response::new(0x0001, 2).encode(&mut frame).unwrap();

        let adu = frame.adu_data();
        assert_eq!(&adu[..6], &[0x11, 0x10, 0x00, 0x01, 0x00, 0x02]);
        assert!(check_rtu_crc(&adu[..6], adu[6], adu[7]));
        assert_eq!(Response::decode(&frame), Ok(Response::new(0x0001, 2)));
    }

    #[test]
    fn encode_003() {
        // Exception (TCP and RTU)
        let response = Response::new_exception(ExceptionCode::IllegalDataAddress);

        let mut frame = Frame::new(0x0001, 0x11);
        response.encode(&mut frame).unwrap();
        assert_eq!(
            frame.as_bytes(),
            &[0x00, 0x01, 0x00, 0x00, 0x00, 0x03, 0x11, 0x90, 0x02]
        );
        assert_eq!(Response::decode(&frame), Ok(response));

        let mut data = [0u8; MODBUS_RTU_FRAME_DATA_LENGTH];
        let mut frame = ModbusRtuFrame::new(&mut data, 0x11);
        response.encode(&mut frame).unwrap();
        let adu = frame.adu_data();
        assert_eq!(&adu[..3], &[0x11, 0x90, 0x02]);
        assert!(check_rtu_crc(&adu[..3], adu[3], adu[4]));
        assert_eq!(Response::decode(&frame), Ok(response));
    }

    #[test]
    fn encode_004() {
        // Quantity 0, 123 and 124
        let mut frame = Frame::new(0x0001, 0x11);
        assert_eq!(
            Response::new(0, 0).encode(&mut frame),
            Err(EncodeError::Malformed(Malformed::Quantity))
        );
        assert_eq!(Response::new(0, 123).encode(&mut frame), Ok(()));
        assert_eq!(
            Response::new(0, 124).encode(&mut frame),
            Err(EncodeError::Malformed(Malformed::Quantity))
        );
    }

    #[test]
    fn edge_001() {
        // Quantity 1 and 123 with lowest and highest address (TCP and RTU)
        for starting_address in [0x0000, 0xFFFF] {
            for quantity in [1, 123] {
                let response = Response::new(starting_address, quantity);
                test_util::roundtrip(
                    |frame| response.encode(frame),
                    |pdu| assert_eq!(Response::decode(pdu), Ok(response)),
                );
            }
        }
    }

    #[test]
    fn edge_002() {
        // Byte order of address and quantity
        let mut frame = Frame::new(0x0001, 0x11);
        Response::new(0x1234, 0x0056).encode(&mut frame).unwrap();
        assert_eq!(frame.pdu(), &[0x10, 0x12, 0x34, 0x00, 0x56]);
    }

    #[test]
    fn edge_003() {
        // Capacity one byte too small and exactly fitting
        let response = Response::new(0x0001, 2);

        let mut frame = SmallFrame::<3>::new();
        assert_eq!(
            response.encode(&mut frame),
            Err(EncodeError::BufferTooSmall)
        );
        assert_eq!(frame.finalized, None);

        let mut frame = SmallFrame::<4>::new();
        assert_eq!(response.encode(&mut frame), Ok(()));
        assert_eq!(frame.finalized, Some((response.function_code(), 4)));
        assert_eq!(frame.payload, [0x00, 0x01, 0x00, 0x02]);
    }

    #[test]
    fn edge_004() {
        // Failed encoding leaves the frame unchanged (TCP and RTU)
        let mut frame = Frame::new(0x0001, 0x11);
        Response::new(0x0001, 2).encode(&mut frame).unwrap();
        let before = frame.clone();
        assert_eq!(
            Response::new(0x0001, 0).encode(&mut frame),
            Err(EncodeError::Malformed(Malformed::Quantity))
        );
        assert_eq!(
            Response::new(0x0001, 124).encode(&mut frame),
            Err(EncodeError::Malformed(Malformed::Quantity))
        );
        assert_eq!(frame, before);

        let mut data = [0u8; MODBUS_RTU_FRAME_DATA_LENGTH];
        let mut frame = ModbusRtuFrame::new(&mut data, 0x11);
        Response::new(0x0001, 2).encode(&mut frame).unwrap();
        let before = frame.adu_data().to_vec();
        assert_eq!(
            Response::new(0x0001, 0xFFFF).encode(&mut frame),
            Err(EncodeError::Malformed(Malformed::Quantity))
        );
        assert_eq!(frame.adu_data(), &before[..]);
    }

    #[test]
    fn edge_005() {
        // Every exception code (TCP and RTU)
        for code in EXCEPTION_CODES {
            let response = Response::new_exception(code);
            test_util::roundtrip(
                |frame| response.encode(frame),
                |pdu| {
                    assert_eq!(pdu.pdu_data(), &[0x90, u8::from(code)]);
                    assert_eq!(Response::decode(pdu), Ok(response));
                },
            );
        }
    }

    #[test]
    fn edge_006() {
        // Decode and encode again yields the same bytes
        let bytes = [0x10, 0xFF, 0xFF, 0x00, 0x7B];
        let response = Response::decode(&Pdu(&bytes)).unwrap();
        let mut frame = Frame::new(0x0001, 0x11);
        response.encode(&mut frame).unwrap();
        assert_eq!(frame.pdu(), &bytes);
    }
}
