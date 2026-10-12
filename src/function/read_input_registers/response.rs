use crate::function::{DecodeError, EncodeError, Malformed, Registers, encode_exception};
use crate::{ExceptionCode, FrameEncoder, FunctionCode, FunctionKind, PduData};

// ── Response ──

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Response<'a> {
    Normal(Registers<'a>),
    Exception(ExceptionCode),
}

impl<'a> Response<'a> {
    pub const MIN_BYTE_COUNT: u8 = 2;
    pub const MAX_BYTE_COUNT: u8 = 250;

    pub const MIN_QUANTITY_OF_REGISTERS: usize = 1;
    pub const MAX_QUANTITY_OF_REGISTERS: usize = 125;

    pub fn new(values: &'a [u16]) -> Response<'a> {
        Response::Normal(Registers::from(values))
    }

    pub const fn new_exception(code: ExceptionCode) -> Response<'a> {
        Response::Exception(code)
    }

    pub fn decode(pdu: &'a dyn PduData) -> Result<Response<'a>, DecodeError> {
        let data = pdu.pdu_data();

        match pdu.function_code() {
            FunctionKind::Normal(FunctionCode::ReadInputRegisters) => {
                let Some(&byte_count) = data.get(1) else {
                    return Err(Malformed::Length.into());
                };

                if !(Self::MIN_BYTE_COUNT..=Self::MAX_BYTE_COUNT).contains(&byte_count)
                    || byte_count % 2 != 0
                {
                    return Err(Malformed::ByteCount.into());
                }

                if data.len() != 2 + byte_count as usize {
                    return Err(Malformed::Length.into());
                }

                Ok(Response::Normal(Registers::new(&data[2..])))
            }
            FunctionKind::Exception(FunctionCode::ReadInputRegisters) => {
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
            Response::Normal(registers) => {
                let quantity = registers.len();
                if !(Self::MIN_QUANTITY_OF_REGISTERS..=Self::MAX_QUANTITY_OF_REGISTERS)
                    .contains(&quantity)
                {
                    return Err(Malformed::Quantity.into());
                }

                let byte_count = 2 * quantity;
                let Some(payload) = frame.payload_mut().get_mut(..1 + byte_count) else {
                    return Err(EncodeError::BufferTooSmall);
                };

                // Byte count
                payload[0] = byte_count as u8;

                // Register values
                for (bytes, value) in payload[1..]
                    .as_chunks_mut::<2>()
                    .0
                    .iter_mut()
                    .zip(registers.clone())
                {
                    bytes.copy_from_slice(&value.to_be_bytes());
                }

                frame.finalize(self.function_code(), 1 + byte_count);
                Ok(())
            }
            Response::Exception(code) => {
                encode_exception(frame, FunctionCode::ReadInputRegisters, *code)
            }
        }
    }

    pub fn function_code(&self) -> FunctionKind {
        match self {
            Response::Normal(_) => FunctionKind::Normal(FunctionCode::ReadInputRegisters),
            Response::Exception(_) => FunctionKind::Exception(FunctionCode::ReadInputRegisters),
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
    use std::vec::Vec;

    #[test]
    fn decode_001() {
        // Example from the specification
        let pdu = Pdu(&[0x04, 0x02, 0x00, 0x0A]);
        let response = Response::decode(&pdu).unwrap();
        assert_eq!(
            response.function_code(),
            FunctionKind::Normal(FunctionCode::ReadInputRegisters)
        );

        let Response::Normal(registers) = response else {
            panic!("expected normal response");
        };
        assert_eq!(registers.len(), 1);
        assert_eq!(registers.collect::<Vec<_>>(), [0x000A]);
    }

    #[test]
    fn decode_002() {
        // Exception
        let response = Response::decode(&Pdu(&[0x84, 0x02])).unwrap();
        assert_eq!(
            response,
            Response::Exception(ExceptionCode::IllegalDataAddress)
        );
        assert_eq!(
            response.function_code(),
            FunctionKind::Exception(FunctionCode::ReadInputRegisters)
        );
    }

    #[test]
    fn decode_003() {
        // Exception too short / too long
        assert_eq!(
            Response::decode(&Pdu(&[0x84])),
            Err(DecodeError::Malformed(Malformed::Length))
        );
        assert_eq!(
            Response::decode(&Pdu(&[0x84, 0x02, 0x00])),
            Err(DecodeError::Malformed(Malformed::Length))
        );
    }

    #[test]
    fn decode_004() {
        // Missing byte count
        assert_eq!(
            Response::decode(&Pdu(&[0x04])),
            Err(DecodeError::Malformed(Malformed::Length))
        );
    }

    #[test]
    fn decode_005() {
        // Data too short / too long
        assert_eq!(
            Response::decode(&Pdu(&[0x04, 0x04, 0x00, 0x01, 0x00])),
            Err(DecodeError::Malformed(Malformed::Length))
        );
        assert_eq!(
            Response::decode(&Pdu(&[0x04, 0x02, 0x00, 0x01, 0x00])),
            Err(DecodeError::Malformed(Malformed::Length))
        );
    }

    #[test]
    fn decode_006() {
        // Byte count 0, odd and above maximum
        assert_eq!(
            Response::decode(&Pdu(&[0x04, 0x00])),
            Err(DecodeError::Malformed(Malformed::ByteCount))
        );
        assert_eq!(
            Response::decode(&Pdu(&[0x04, 0x03, 0x00, 0x01, 0x00])),
            Err(DecodeError::Malformed(Malformed::ByteCount))
        );
        for byte_count in 251..=255 {
            assert_eq!(
                Response::decode(&Pdu(&[0x04, byte_count])),
                Err(DecodeError::Malformed(Malformed::ByteCount))
            );
        }
    }

    #[test]
    fn decode_007() {
        // Maximum byte count
        let mut data = [0u8; 252];
        data[0] = 0x04;
        data[1] = 250;
        let pdu = Pdu(&data);
        let Ok(Response::Normal(registers)) = Response::decode(&pdu) else {
            panic!("expected normal response");
        };
        assert_eq!(registers.len(), 125);
    }

    #[test]
    fn decode_008() {
        // Read holding registers response
        let result = Response::decode(&Pdu(&[0x03, 0x02, 0x00, 0x0A]));
        assert_eq!(
            result,
            Err(DecodeError::UnexpectedFunction(FunctionKind::Normal(
                FunctionCode::ReadHoldingRegisters
            )))
        );
    }

    #[test]
    fn decode_009() {
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
        let values = [0x000A];
        let mut frame = Frame::new(0x0001, 0x11);
        Response::new(&values).encode(&mut frame).unwrap();
        assert_eq!(
            frame.as_bytes(),
            &[
                0x00, 0x01, 0x00, 0x00, 0x00, 0x05, 0x11, 0x04, 0x02, 0x00, 0x0A
            ]
        );
        assert_eq!(Response::decode(&frame), Ok(Response::new(&values)));
    }

    #[test]
    fn encode_002() {
        // Example from the specification (RTU)
        let values = [0x000A];
        let mut data = [0u8; MODBUS_RTU_FRAME_DATA_LENGTH];
        let mut frame = ModbusRtuFrame::new(&mut data, 0x11);
        Response::new(&values).encode(&mut frame).unwrap();

        let adu = frame.adu_data();
        assert_eq!(&adu[..5], &[0x11, 0x04, 0x02, 0x00, 0x0A]);
        assert!(check_rtu_crc(&adu[..5], adu[5], adu[6]));
        assert_eq!(Response::decode(&frame), Ok(Response::new(&values)));
    }

    #[test]
    fn encode_003() {
        // Exception (TCP and RTU)
        let response = Response::new_exception(ExceptionCode::IllegalDataAddress);

        let mut frame = Frame::new(0x0001, 0x11);
        response.encode(&mut frame).unwrap();
        assert_eq!(
            frame.as_bytes(),
            &[0x00, 0x01, 0x00, 0x00, 0x00, 0x03, 0x11, 0x84, 0x02]
        );
        assert_eq!(Response::decode(&frame), Ok(response.clone()));

        let mut data = [0u8; MODBUS_RTU_FRAME_DATA_LENGTH];
        let mut frame = ModbusRtuFrame::new(&mut data, 0x11);
        response.encode(&mut frame).unwrap();
        let adu = frame.adu_data();
        assert_eq!(&adu[..3], &[0x11, 0x84, 0x02]);
        assert!(check_rtu_crc(&adu[..3], adu[3], adu[4]));
        assert_eq!(Response::decode(&frame), Ok(response));
    }

    #[test]
    fn encode_004() {
        // Quantity 0, 125 and 126
        let values = [0u16; 126];
        let mut frame = Frame::new(0x0001, 0x11);
        assert_eq!(
            Response::new(&[]).encode(&mut frame),
            Err(EncodeError::Malformed(Malformed::Quantity))
        );
        assert_eq!(Response::new(&values[..125]).encode(&mut frame), Ok(()));
        assert_eq!(frame.pdu().len(), 2 + 250);
        assert_eq!(
            Response::new(&values).encode(&mut frame),
            Err(EncodeError::Malformed(Malformed::Quantity))
        );
    }
}
