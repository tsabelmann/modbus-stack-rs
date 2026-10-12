use crate::{constants::MODBUS_RTU_FRAME_DATA_LENGTH, rtu::{crc::check_rtu_crc, ModbusRtuFrame}, FunctionCode, FunctionKind};

#[derive(Debug, PartialEq, Clone)]
enum ModbusRtuFrameDecoderState {
    WaitForSlaveAddress,
    WaitForFunctionCode,
    ReadByteCount {
        bytes_before_byte_count: usize
    },
    CollectPduData {
        index: usize,
        length: usize
    },
    CollectCrcData {
        index: usize
    }
}

#[derive(Debug, PartialEq)]
pub enum ModbusRtuFrameDecoderResult<'a> {
    Success(ModbusRtuFrame<'a>),
    Pending,
    CrcError,
    NotSupported,
    InvalidFunctionCode,
    NotEnoughData
}

#[derive(Debug, PartialEq, Clone, Copy, Default)]
pub enum ModbusRtuFrameDecoderDirection {
    #[default]
    Request,
    Response
}

pub struct ModbusRtuFrameDecoder<'a> {
    data: &'a mut [u8; MODBUS_RTU_FRAME_DATA_LENGTH],
    state: ModbusRtuFrameDecoderState,
    index: usize,
    decoder_direction: ModbusRtuFrameDecoderDirection
}

impl<'a> ModbusRtuFrameDecoder<'a> {
    pub const fn new(data: &'a mut [u8; MODBUS_RTU_FRAME_DATA_LENGTH], decoder_direction: ModbusRtuFrameDecoderDirection) -> ModbusRtuFrameDecoder<'a> {
        ModbusRtuFrameDecoder {
            data,
            state: ModbusRtuFrameDecoderState::WaitForSlaveAddress,
            index: 0,
            decoder_direction
        }
    }

    pub const fn expect_request(&mut self) {
        // Reset state
        self.reset();

        // Switch to request mode
        self.decoder_direction = ModbusRtuFrameDecoderDirection::Request;
    }

    pub const fn expect_response(&mut self) {
        // Reset state
        self.reset();

        // Switch to response mode
        self.decoder_direction = ModbusRtuFrameDecoderDirection::Response;
    }

    pub fn push_data(&mut self, data: u8) -> ModbusRtuFrameDecoderResult<'_> {
        match &mut self.state {
            ModbusRtuFrameDecoderState::WaitForSlaveAddress => {
                // Push data into internal storage
                if let Some(ptr) = self.data.get_mut(self.index) {
                    *ptr = data;
                } else {
                    self.reset();
                    return ModbusRtuFrameDecoderResult::NotEnoughData;
                }

                // Increment index
                self.index += 1;

                // Next state
                self.state = ModbusRtuFrameDecoderState::WaitForFunctionCode;
            },
            ModbusRtuFrameDecoderState::WaitForFunctionCode => {
                // Push data into internal storage
                if let Some(ptr) = self.data.get_mut(self.index) {
                    *ptr = data;
                } else {
                    self.reset();
                    return ModbusRtuFrameDecoderResult::NotEnoughData;
                }

                // Increment index
                self.index += 1;

                // Function Code
                let function_kind = FunctionKind::from(data);
                
                // Decode based on the direction
                match self.decoder_direction {
                    ModbusRtuFrameDecoderDirection::Request => {
                        // Check function kind
                        match function_kind {
                            FunctionKind::Normal(function_code) => {
                                // Next State
                                self.state = match function_code {
                                    FunctionCode::ReadCoils => ModbusRtuFrameDecoderState::CollectPduData { index: 0, length: 4 },
                                    FunctionCode::ReadDiscreteInputs => ModbusRtuFrameDecoderState::CollectPduData { index: 0, length: 4 },
                                    FunctionCode::ReadHoldingRegisters => ModbusRtuFrameDecoderState::CollectPduData { index: 0, length: 4 },
                                    FunctionCode::ReadInputRegisters => ModbusRtuFrameDecoderState::CollectPduData { index: 0, length: 4 },
                                    FunctionCode::WriteSingleCoil => ModbusRtuFrameDecoderState::CollectPduData { index: 0, length: 4 },
                                    FunctionCode::WriteSingleRegister => ModbusRtuFrameDecoderState::CollectPduData { index: 0, length: 4 },
                                    FunctionCode::ReadExceptionStatus => ModbusRtuFrameDecoderState::CollectCrcData { index: 0 },
                                    FunctionCode::Diagnostic => panic!("Not supported"),
                                    FunctionCode::GetComEventCounter => ModbusRtuFrameDecoderState::CollectCrcData { index: 0 },
                                    FunctionCode::GetComEventLog => ModbusRtuFrameDecoderState::CollectCrcData { index: 0 },
                                    FunctionCode::WriteMultipleCoils => ModbusRtuFrameDecoderState::ReadByteCount { bytes_before_byte_count: 4 },
                                    FunctionCode::WriteMultipleRegisters => ModbusRtuFrameDecoderState::ReadByteCount { bytes_before_byte_count: 4 },
                                    FunctionCode::ReportServerId => ModbusRtuFrameDecoderState::CollectCrcData { index: 0 },
                                    FunctionCode::ReadFileRecord => ModbusRtuFrameDecoderState::ReadByteCount { bytes_before_byte_count: 0 },
                                    FunctionCode::WriteFileRecord => panic!("Not supported"),
                                    FunctionCode::MaskWriteRegister => ModbusRtuFrameDecoderState::CollectPduData { index: 0, length: 6 },
                                    FunctionCode::ReadWriteMultipleRegisters => ModbusRtuFrameDecoderState::ReadByteCount { bytes_before_byte_count: 8 },
                                    FunctionCode::ReadFifoQueue => ModbusRtuFrameDecoderState::CollectPduData { index: 0, length: 2 },
                                    FunctionCode::Unknown(_) => {
                                        // Reset
                                        self.index = 0;
                                        ModbusRtuFrameDecoderState::WaitForSlaveAddress
                                    },
                                }
                            },
                            FunctionKind::Exception(_) => {
                                // Reset state 
                                self.reset();

                                // Return
                                return ModbusRtuFrameDecoderResult::InvalidFunctionCode;
                            },
                        }
                    },
                    ModbusRtuFrameDecoderDirection::Response => {
                        match function_kind {
                            FunctionKind::Normal(function_code) => {
                                // Next state
                                self.state = match function_code {
                                    FunctionCode::ReadCoils => ModbusRtuFrameDecoderState::ReadByteCount { bytes_before_byte_count: 0 },
                                    FunctionCode::ReadDiscreteInputs => ModbusRtuFrameDecoderState::ReadByteCount { bytes_before_byte_count: 0 },
                                    FunctionCode::ReadHoldingRegisters => ModbusRtuFrameDecoderState::ReadByteCount { bytes_before_byte_count: 0 },
                                    FunctionCode::ReadInputRegisters => ModbusRtuFrameDecoderState::ReadByteCount { bytes_before_byte_count: 0 },
                                    FunctionCode::WriteSingleCoil => ModbusRtuFrameDecoderState::CollectPduData { index: 0, length: 4 },
                                    FunctionCode::WriteSingleRegister => ModbusRtuFrameDecoderState::CollectPduData { index: 0, length: 4 },
                                    FunctionCode::ReadExceptionStatus => ModbusRtuFrameDecoderState::CollectPduData { index: 0, length: 1 },
                                    FunctionCode::Diagnostic => panic!("Not supported!"),
                                    FunctionCode::GetComEventCounter => ModbusRtuFrameDecoderState::CollectPduData { index: 0, length: 4 },
                                    FunctionCode::GetComEventLog => ModbusRtuFrameDecoderState::ReadByteCount { bytes_before_byte_count: 0 },
                                    FunctionCode::WriteMultipleCoils => ModbusRtuFrameDecoderState::CollectPduData { index: 0, length: 4 },
                                    FunctionCode::WriteMultipleRegisters => ModbusRtuFrameDecoderState::CollectPduData { index: 0, length: 4 },
                                    FunctionCode::ReportServerId => panic!("Not supported!"),
                                    FunctionCode::ReadFileRecord => panic!("Not supported!"),
                                    FunctionCode::WriteFileRecord => panic!("Not supported!"),
                                    FunctionCode::MaskWriteRegister => ModbusRtuFrameDecoderState::CollectPduData { index: 0, length: 6 },
                                    FunctionCode::ReadWriteMultipleRegisters => ModbusRtuFrameDecoderState::ReadByteCount { bytes_before_byte_count: 0 },
                                    FunctionCode::ReadFifoQueue => panic!("Not supported!"),
                                    FunctionCode::Unknown(_) => {
                                        // Reset
                                        self.index = 0;
                                        ModbusRtuFrameDecoderState::WaitForSlaveAddress
                                    },
                                };
                            },
                            FunctionKind::Exception(_) => {
                                // Next State
                                self.state = ModbusRtuFrameDecoderState::CollectPduData { index: 0, length: 1 };
                            },
                        }
                    },
                }
            },
            ModbusRtuFrameDecoderState::ReadByteCount { bytes_before_byte_count } => {
                // Push data into internal storage
                if let Some(ptr) = self.data.get_mut(self.index) {
                    *ptr = data;
                } else {
                    self.reset();
                    return ModbusRtuFrameDecoderResult::NotEnoughData;
                }

                // Increment index
                self.index += 1;

                // Check if the next byte is the byte_count
                if *bytes_before_byte_count == 0 {
                    // byte count 
                    let byte_count = data as usize;

                    // Next state
                    self.state = ModbusRtuFrameDecoderState::CollectPduData { index: 0, length: byte_count };
                } else {
                    // Decrement bytes before byte count
                    *bytes_before_byte_count -= 1;
                }           
            },
            ModbusRtuFrameDecoderState::CollectPduData { index, length } => {
                // Push data into internal storage
                if let Some(ptr) = self.data.get_mut(self.index) {
                    *ptr = data;
                } else {
                    self.reset();
                    return ModbusRtuFrameDecoderResult::NotEnoughData;
                }

                // Increment index
                self.index += 1;

                // Increment index
                *index += 1;
                    
                if *index >= *length  {
                    self.state = ModbusRtuFrameDecoderState::CollectCrcData { index: 0 };
                }
            },
            ModbusRtuFrameDecoderState::CollectCrcData { index } => {   
                // Push data into internal storage
                if let Some(ptr) = self.data.get_mut(self.index) {
                    *ptr = data;
                } else {
                    self.reset();
                    return ModbusRtuFrameDecoderResult::NotEnoughData;
                }

                // Increment index
                self.index += 1;

                // Increment index
                *index += 1;

                if *index >= 2 {
                    // Check CRC
                    if check_rtu_crc(&self.data[..self.index-2], self.data[self.index-2], self.data[self.index-1]) {
                        let frame = unsafe {
                            ModbusRtuFrame::new_unchecked(self.data, self.index)
                        };
                        
                        // Reset state
                        self.state = ModbusRtuFrameDecoderState::WaitForSlaveAddress;
                        self.index = 0;

                        // Success
                        return ModbusRtuFrameDecoderResult::Success(frame);
                    } else {
                        // Reset state
                        self.state = ModbusRtuFrameDecoderState::WaitForSlaveAddress;
                        self.index = 0;

                        // CrcError
                        return ModbusRtuFrameDecoderResult::CrcError;
                    }
                }
            }
        }
        ModbusRtuFrameDecoderResult::Pending
    }

    pub const fn reset(&mut self) {
        self.state = ModbusRtuFrameDecoderState::WaitForSlaveAddress;
        self.index = 0;
    }


}

#[cfg(test)]
mod rtu_frames_extended_tests {
    use super::*;

    // ========== READ INPUT REGISTERS (0x04) ==========
    #[test]
    fn rtu_frame_decoder_read_input_registers_request_001() {
        let mut buffer = [0u8; MODBUS_RTU_FRAME_DATA_LENGTH];
        let mut decoder = ModbusRtuFrameDecoder::new(&mut buffer, ModbusRtuFrameDecoderDirection::Request);

        let _ = decoder.push_data(0x01);
        let _ = decoder.push_data(0x04);
        let _ = decoder.push_data(0x00);
        let _ = decoder.push_data(0x08);
        let _ = decoder.push_data(0x00);
        let _ = decoder.push_data(0x01);
        let _ = decoder.push_data(0xB0);
        assert!(matches!(decoder.push_data(0x08), ModbusRtuFrameDecoderResult::Success(_)));
    }

    #[test]
    fn rtu_frame_decoder_read_input_registers_response_001() {
        let mut buffer = [0u8; MODBUS_RTU_FRAME_DATA_LENGTH];
        let mut decoder = ModbusRtuFrameDecoder::new(&mut buffer, ModbusRtuFrameDecoderDirection::Response);

        let _ = decoder.push_data(0x01);
        let _ = decoder.push_data(0x04);
        let _ = decoder.push_data(0x02);
        let _ = decoder.push_data(0x00);
        let _ = decoder.push_data(0x0A);
        let _ = decoder.push_data(0x39);
        assert!(matches!(decoder.push_data(0x37), ModbusRtuFrameDecoderResult::Success(_)));
    }

    // ========== WRITE SINGLE COIL (0x05) ==========
    #[test]
    fn rtu_frame_decoder_write_single_coil_request_001() {
        let mut buffer = [0u8; MODBUS_RTU_FRAME_DATA_LENGTH];
        let mut decoder = ModbusRtuFrameDecoder::new(&mut buffer, ModbusRtuFrameDecoderDirection::Request);

        let _ = decoder.push_data(0x01);
        let _ = decoder.push_data(0x05);
        let _ = decoder.push_data(0x00);
        let _ = decoder.push_data(0xAC);
        let _ = decoder.push_data(0xFF);
        let _ = decoder.push_data(0x00);
        let _ = decoder.push_data(0x4C);
        assert!(matches!(decoder.push_data(0x1B), ModbusRtuFrameDecoderResult::Success(_)));
    }

    #[test]
    fn rtu_frame_decoder_write_single_coil_response_001() {
        let mut buffer = [0u8; MODBUS_RTU_FRAME_DATA_LENGTH];
        let mut decoder = ModbusRtuFrameDecoder::new(&mut buffer, ModbusRtuFrameDecoderDirection::Response);

        let _ = decoder.push_data(0x01);
        let _ = decoder.push_data(0x05);
        let _ = decoder.push_data(0x00);
        let _ = decoder.push_data(0xAC);
        let _ = decoder.push_data(0xFF);
        let _ = decoder.push_data(0x00);
        let _ = decoder.push_data(0x4C);
        assert!(matches!(decoder.push_data(0x1B), ModbusRtuFrameDecoderResult::Success(_)));
    }

    // ========== WRITE SINGLE REGISTER (0x06) ==========
    #[test]
    fn rtu_frame_decoder_write_single_register_request_001() {
        let mut buffer = [0u8; MODBUS_RTU_FRAME_DATA_LENGTH];
        let mut decoder = ModbusRtuFrameDecoder::new(&mut buffer, ModbusRtuFrameDecoderDirection::Request);

        let _ = decoder.push_data(0x01);
        let _ = decoder.push_data(0x06);
        let _ = decoder.push_data(0x00);
        let _ = decoder.push_data(0x01);
        let _ = decoder.push_data(0x00);
        let _ = decoder.push_data(0x03);
        let _ = decoder.push_data(0x98);
        assert!(matches!(decoder.push_data(0x0B), ModbusRtuFrameDecoderResult::Success(_)));
    }

    #[test]
    fn rtu_frame_decoder_write_single_register_response_001() {
        let mut buffer = [0u8; MODBUS_RTU_FRAME_DATA_LENGTH];
        let mut decoder = ModbusRtuFrameDecoder::new(&mut buffer, ModbusRtuFrameDecoderDirection::Response);

        let _ = decoder.push_data(0x01);
        let _ = decoder.push_data(0x06);
        let _ = decoder.push_data(0x00);
        let _ = decoder.push_data(0x01);
        let _ = decoder.push_data(0x00);
        let _ = decoder.push_data(0x03);
        let _ = decoder.push_data(0x98);
        assert!(matches!(decoder.push_data(0x0B), ModbusRtuFrameDecoderResult::Success(_)));
    }

    // ========== READ EXCEPTION STATUS (0x07) ==========
    #[test]
    fn rtu_frame_decoder_read_exception_status_request_001() {
        let mut buffer = [0u8; MODBUS_RTU_FRAME_DATA_LENGTH];
        let mut decoder = ModbusRtuFrameDecoder::new(&mut buffer, ModbusRtuFrameDecoderDirection::Request);

        let _ = decoder.push_data(0x01);
        let _ = decoder.push_data(0x07);
        let _ = decoder.push_data(0x41);
        assert!(matches!(decoder.push_data(0xE2), ModbusRtuFrameDecoderResult::Success(_)));
    }

    #[test]
    fn rtu_frame_decoder_read_exception_status_response_001() {
        let mut buffer = [0u8; MODBUS_RTU_FRAME_DATA_LENGTH];
        let mut decoder = ModbusRtuFrameDecoder::new(&mut buffer, ModbusRtuFrameDecoderDirection::Response);

        let _ = decoder.push_data(0x01);
        let _ = decoder.push_data(0x07);
        let _ = decoder.push_data(0x6D);
        let _ = decoder.push_data(0xE3);
        assert!(matches!(decoder.push_data(0xDD), ModbusRtuFrameDecoderResult::Success(_)));
    }

    // ========== GET COM EVENT COUNTER (0x0B) ==========
    #[test]
    fn rtu_frame_decoder_get_com_event_counter_request_001() {
        let mut buffer = [0u8; MODBUS_RTU_FRAME_DATA_LENGTH];
        let mut decoder = ModbusRtuFrameDecoder::new(&mut buffer, ModbusRtuFrameDecoderDirection::Request);

        let _ = decoder.push_data(0x01);
        let _ = decoder.push_data(0x0B);
        let _ = decoder.push_data(0x41);
        assert!(matches!(decoder.push_data(0xE7), ModbusRtuFrameDecoderResult::Success(_)));
    }

    #[test]
    fn rtu_frame_decoder_get_com_event_counter_response_001() {
        let mut buffer = [0u8; MODBUS_RTU_FRAME_DATA_LENGTH];
        let mut decoder = ModbusRtuFrameDecoder::new(&mut buffer, ModbusRtuFrameDecoderDirection::Response);

        let _ = decoder.push_data(0x01);
        let _ = decoder.push_data(0x0B);
        let _ = decoder.push_data(0x00);
        let _ = decoder.push_data(0x00);
        let _ = decoder.push_data(0x01);
        let _ = decoder.push_data(0x08);
        let _ = decoder.push_data(0xA4);
        assert!(matches!(decoder.push_data(0x5D), ModbusRtuFrameDecoderResult::Success(_)));
    }

    // ========== GET COM EVENT LOG (0x0C) ==========
    #[test]
    fn rtu_frame_decoder_get_com_event_log_request_001() {
        let mut buffer = [0u8; MODBUS_RTU_FRAME_DATA_LENGTH];
        let mut decoder = ModbusRtuFrameDecoder::new(&mut buffer, ModbusRtuFrameDecoderDirection::Request);

        let _ = decoder.push_data(0x01);
        let _ = decoder.push_data(0x0C);
        let _ = decoder.push_data(0x00);
        assert!(matches!(decoder.push_data(0x25), ModbusRtuFrameDecoderResult::Success(_)));
    }

    #[test]
    fn rtu_frame_decoder_get_com_event_log_response_001() {
        let mut buffer = [0u8; MODBUS_RTU_FRAME_DATA_LENGTH];
        let mut decoder = ModbusRtuFrameDecoder::new(&mut buffer, ModbusRtuFrameDecoderDirection::Response);

        let _ = decoder.push_data(0x01);
        let _ = decoder.push_data(0x0C);
        let _ = decoder.push_data(0x08);
        let _ = decoder.push_data(0x00);
        let _ = decoder.push_data(0x00);
        let _ = decoder.push_data(0x01);
        let _ = decoder.push_data(0x08);
        let _ = decoder.push_data(0x00);
        let _ = decoder.push_data(0x01);
        let _ = decoder.push_data(0x20);
        let _ = decoder.push_data(0x00);
        let _ = decoder.push_data(0x0D);
        assert!(matches!(decoder.push_data(0xF7), ModbusRtuFrameDecoderResult::Success(_)));
    }

    // ========== WRITE MULTIPLE COILS (0x0F) ==========
    #[test]
    fn rtu_frame_decoder_write_multiple_coils_request_001() {
        let mut buffer = [0u8; MODBUS_RTU_FRAME_DATA_LENGTH];
        let mut decoder = ModbusRtuFrameDecoder::new(&mut buffer, ModbusRtuFrameDecoderDirection::Request);

        let _ = decoder.push_data(0x01);
        let _ = decoder.push_data(0x0F);
        let _ = decoder.push_data(0x00);
        let _ = decoder.push_data(0x13);
        let _ = decoder.push_data(0x00);
        let _ = decoder.push_data(0x0A);
        let _ = decoder.push_data(0x02);
        let _ = decoder.push_data(0xCD);
        let _ = decoder.push_data(0x01);
        let _ = decoder.push_data(0x72);
        assert!(matches!(decoder.push_data(0xCB), ModbusRtuFrameDecoderResult::Success(_)));
    }

    #[test]
    fn rtu_frame_decoder_write_multiple_coils_response_001() {
        let mut buffer = [0u8; MODBUS_RTU_FRAME_DATA_LENGTH];
        let mut decoder = ModbusRtuFrameDecoder::new(&mut buffer, ModbusRtuFrameDecoderDirection::Response);

        let _ = decoder.push_data(0x01);
        let _ = decoder.push_data(0x0F);
        let _ = decoder.push_data(0x00);
        let _ = decoder.push_data(0x13);
        let _ = decoder.push_data(0x00);
        let _ = decoder.push_data(0x0A);
        let _ = decoder.push_data(0x24);
        assert!(matches!(decoder.push_data(0x09), ModbusRtuFrameDecoderResult::Success(_)));
    }

    // ========== WRITE MULTIPLE REGISTERS (0x10) ==========
    #[test]
    fn rtu_frame_decoder_write_multiple_registers_request_001() {
        let mut buffer = [0u8; MODBUS_RTU_FRAME_DATA_LENGTH];
        let mut decoder = ModbusRtuFrameDecoder::new(&mut buffer, ModbusRtuFrameDecoderDirection::Request);

        let _ = decoder.push_data(0x01);
        let _ = decoder.push_data(0x10);
        let _ = decoder.push_data(0x00);
        let _ = decoder.push_data(0x01);
        let _ = decoder.push_data(0x00);
        let _ = decoder.push_data(0x02);
        let _ = decoder.push_data(0x04);
        let _ = decoder.push_data(0x00);
        let _ = decoder.push_data(0x0A);
        let _ = decoder.push_data(0x01);
        let _ = decoder.push_data(0x02);
        let _ = decoder.push_data(0x92);
        assert!(matches!(decoder.push_data(0x30), ModbusRtuFrameDecoderResult::Success(_)));
    }

    #[test]
    fn rtu_frame_decoder_write_multiple_registers_response_001() {
        let mut buffer = [0u8; MODBUS_RTU_FRAME_DATA_LENGTH];
        let mut decoder = ModbusRtuFrameDecoder::new(&mut buffer, ModbusRtuFrameDecoderDirection::Response);

        let _ = decoder.push_data(0x01);
        let _ = decoder.push_data(0x10);
        let _ = decoder.push_data(0x00);
        let _ = decoder.push_data(0x01);
        let _ = decoder.push_data(0x00);
        let _ = decoder.push_data(0x02);
        let _ = decoder.push_data(0x10);
        assert!(matches!(decoder.push_data(0x08), ModbusRtuFrameDecoderResult::Success(_)));
    }

    // ========== READ FILE RECORD (0x14) ==========
    #[test]
    fn rtu_frame_decoder_read_file_record_request_001() {
        let mut buffer = [0u8; MODBUS_RTU_FRAME_DATA_LENGTH];
        let mut decoder = ModbusRtuFrameDecoder::new(&mut buffer, ModbusRtuFrameDecoderDirection::Request);

        let _ = decoder.push_data(0x01);
        let _ = decoder.push_data(0x14);
        let _ = decoder.push_data(0x07);
        let _ = decoder.push_data(0x06);
        let _ = decoder.push_data(0x00);
        let _ = decoder.push_data(0x04);
        let _ = decoder.push_data(0x00);
        let _ = decoder.push_data(0x01);
        let _ = decoder.push_data(0x00);
        let _ = decoder.push_data(0x02);
        let _ = decoder.push_data(0xD8);
        assert!(matches!(decoder.push_data(0xE5), ModbusRtuFrameDecoderResult::Success(_)));
    }

    // ========== MASK WRITE REGISTER (0x16) ==========
    #[test]
    fn rtu_frame_decoder_mask_write_register_request_001() {
        let mut buffer = [0u8; MODBUS_RTU_FRAME_DATA_LENGTH];
        let mut decoder = ModbusRtuFrameDecoder::new(&mut buffer, ModbusRtuFrameDecoderDirection::Request);

        let _ = decoder.push_data(0x01);
        let _ = decoder.push_data(0x16);
        let _ = decoder.push_data(0x00);
        let _ = decoder.push_data(0x04);
        let _ = decoder.push_data(0x00);
        let _ = decoder.push_data(0xF2);
        let _ = decoder.push_data(0x00);
        let _ = decoder.push_data(0x25);
        let _ = decoder.push_data(0x67);
        assert!(matches!(decoder.push_data(0xEE), ModbusRtuFrameDecoderResult::Success(_)));
    }

    #[test]
    fn rtu_frame_decoder_mask_write_register_response_001() {
        let mut buffer = [0u8; MODBUS_RTU_FRAME_DATA_LENGTH];
        let mut decoder = ModbusRtuFrameDecoder::new(&mut buffer, ModbusRtuFrameDecoderDirection::Response);

        let _ = decoder.push_data(0x01);
        let _ = decoder.push_data(0x16);
        let _ = decoder.push_data(0x00);
        let _ = decoder.push_data(0x04);
        let _ = decoder.push_data(0x00);
        let _ = decoder.push_data(0xF2);
        let _ = decoder.push_data(0x00);
        let _ = decoder.push_data(0x25);
        let _ = decoder.push_data(0x67);
        assert!(matches!(decoder.push_data(0xEE), ModbusRtuFrameDecoderResult::Success(_)));
    }

    // ========== READ WRITE MULTIPLE REGISTERS (0x17) ==========
    #[test]
    fn rtu_frame_decoder_read_write_multiple_registers_request_001() {
        let mut buffer = [0u8; MODBUS_RTU_FRAME_DATA_LENGTH];
        let mut decoder = ModbusRtuFrameDecoder::new(&mut buffer, ModbusRtuFrameDecoderDirection::Request);

        let _ = decoder.push_data(0x01);
        let _ = decoder.push_data(0x17);
        let _ = decoder.push_data(0x00);
        let _ = decoder.push_data(0x03);
        let _ = decoder.push_data(0x00);
        let _ = decoder.push_data(0x06);
        let _ = decoder.push_data(0x00);
        let _ = decoder.push_data(0x0E);
        let _ = decoder.push_data(0x00);
        let _ = decoder.push_data(0x03);
        let _ = decoder.push_data(0x06);
        let _ = decoder.push_data(0x00);
        let _ = decoder.push_data(0xFF);
        let _ = decoder.push_data(0x00);
        let _ = decoder.push_data(0xFF);
        let _ = decoder.push_data(0x00);
        let _ = decoder.push_data(0xFF);
        let _ = decoder.push_data(0x46);
        assert!(matches!(decoder.push_data(0x91), ModbusRtuFrameDecoderResult::Success(_)));
    }

    #[test]
    fn rtu_frame_decoder_read_write_multiple_registers_response_001() {
        let mut buffer = [0u8; MODBUS_RTU_FRAME_DATA_LENGTH];
        let mut decoder = ModbusRtuFrameDecoder::new(&mut buffer, ModbusRtuFrameDecoderDirection::Response);

        let _ = decoder.push_data(0x01);
        let _ = decoder.push_data(0x17);
        let _ = decoder.push_data(0x0C);
        let _ = decoder.push_data(0x00);
        let _ = decoder.push_data(0xFE);
        let _ = decoder.push_data(0x0A);
        let _ = decoder.push_data(0xCD);
        let _ = decoder.push_data(0x00);
        let _ = decoder.push_data(0x01);
        let _ = decoder.push_data(0x00);
        let _ = decoder.push_data(0x03);
        let _ = decoder.push_data(0x00);
        let _ = decoder.push_data(0x0D);
        let _ = decoder.push_data(0x00);
        let _ = decoder.push_data(0xFF);
        let _ = decoder.push_data(0x1D);
        assert!(matches!(decoder.push_data(0x79), ModbusRtuFrameDecoderResult::Success(_)));
    }

    // ========== READ FIFO QUEUE (0x18) ==========
    #[test]
    fn rtu_frame_decoder_read_fifo_queue_request_001() {
        let mut buffer = [0u8; MODBUS_RTU_FRAME_DATA_LENGTH];
        let mut decoder = ModbusRtuFrameDecoder::new(&mut buffer, ModbusRtuFrameDecoderDirection::Request);

        let _ = decoder.push_data(0x01);
        let _ = decoder.push_data(0x18);
        let _ = decoder.push_data(0x04);
        let _ = decoder.push_data(0xE8);
        let _ = decoder.push_data(0x83);
        assert!(matches!(decoder.push_data(0x51), ModbusRtuFrameDecoderResult::Success(_)));
    }

    // ========== EXCEPTION RESPONSES ==========
    #[test]
    fn rtu_frame_decoder_exception_illegal_function() {
        let mut buffer = [0u8; MODBUS_RTU_FRAME_DATA_LENGTH];
        let mut decoder = ModbusRtuFrameDecoder::new(&mut buffer, ModbusRtuFrameDecoderDirection::Response);

        let _ = decoder.push_data(0x01);
        let _ = decoder.push_data(0x81); // Exception: 0x01 + 0x80
        let _ = decoder.push_data(0x01); // Exception Code: Illegal Function
        let _ = decoder.push_data(0x81);
        assert!(matches!(decoder.push_data(0x90), ModbusRtuFrameDecoderResult::Success(_)));
    }

    #[test]
    fn rtu_frame_decoder_exception_illegal_data_address() {
        let mut buffer = [0u8; MODBUS_RTU_FRAME_DATA_LENGTH];
        let mut decoder = ModbusRtuFrameDecoder::new(&mut buffer, ModbusRtuFrameDecoderDirection::Response);

        let _ = decoder.push_data(0x01);
        let _ = decoder.push_data(0x83); // Exception: 0x03 + 0x80
        let _ = decoder.push_data(0x02); // Exception Code: Illegal Data Address
        let _ = decoder.push_data(0xC0);
        assert!(matches!(decoder.push_data(0xF1), ModbusRtuFrameDecoderResult::Success(_)));
    }

    #[test]
    fn rtu_frame_decoder_exception_illegal_data_value() {
        let mut buffer = [0u8; MODBUS_RTU_FRAME_DATA_LENGTH];
        let mut decoder = ModbusRtuFrameDecoder::new(&mut buffer, ModbusRtuFrameDecoderDirection::Response);

        let _ = decoder.push_data(0x01);
        let _ = decoder.push_data(0x84); // Exception: 0x04 + 0x80
        let _ = decoder.push_data(0x03); // Exception Code: Illegal Data Value
        let _ = decoder.push_data(0x03);
        assert!(matches!(decoder.push_data(0x01), ModbusRtuFrameDecoderResult::Success(_)));
    }

    #[test]
    fn rtu_frame_decoder_exception_slave_device_failure() {
        let mut buffer = [0u8; MODBUS_RTU_FRAME_DATA_LENGTH];
        let mut decoder = ModbusRtuFrameDecoder::new(&mut buffer, ModbusRtuFrameDecoderDirection::Response);

        let _ = decoder.push_data(0x01);
        let _ = decoder.push_data(0x86); // Exception: 0x06 + 0x80
        let _ = decoder.push_data(0x04); // Exception Code: Slave Device Failure
        let _ = decoder.push_data(0x43);
        assert!(matches!(decoder.push_data(0xA3), ModbusRtuFrameDecoderResult::Success(_)));
    }

    #[test]
    fn rtu_frame_decoder_exception_acknowledge() {
        let mut buffer = [0u8; MODBUS_RTU_FRAME_DATA_LENGTH];
        let mut decoder = ModbusRtuFrameDecoder::new(&mut buffer, ModbusRtuFrameDecoderDirection::Response);

        let _ = decoder.push_data(0x01);
        let _ = decoder.push_data(0x90); // Exception: 0x10 + 0x80
        let _ = decoder.push_data(0x05); // Exception Code: Acknowledge
        let _ = decoder.push_data(0x8C);
        assert!(matches!(decoder.push_data(0x03), ModbusRtuFrameDecoderResult::Success(_)));
    }

    #[test]
    fn rtu_frame_decoder_exception_gateway_path_unavailable() {
        let mut buffer = [0u8; MODBUS_RTU_FRAME_DATA_LENGTH];
        let mut decoder = ModbusRtuFrameDecoder::new(&mut buffer, ModbusRtuFrameDecoderDirection::Response);

        let _ = decoder.push_data(0x01);
        let _ = decoder.push_data(0x8F); // Exception: 0x0F + 0x80
        let _ = decoder.push_data(0x0A); // Exception Code: Gateway Path Unavailable
        let _ = decoder.push_data(0xC4);
        assert!(matches!(decoder.push_data(0x37), ModbusRtuFrameDecoderResult::Success(_)));
    }

    // ========== CRC ERROR TESTS ==========
    #[test]
    fn rtu_frame_decoder_crc_error_read_holding_registers() {
        let mut buffer = [0u8; MODBUS_RTU_FRAME_DATA_LENGTH];
        let mut decoder = ModbusRtuFrameDecoder::new(&mut buffer, ModbusRtuFrameDecoderDirection::Request);

        let _ = decoder.push_data(0x01);
        let _ = decoder.push_data(0x03);
        let _ = decoder.push_data(0x00);
        let _ = decoder.push_data(0x00);
        let _ = decoder.push_data(0x00);
        let _ = decoder.push_data(0x02);
        let _ = decoder.push_data(0xFF); // Wrong CRC
        assert!(matches!(decoder.push_data(0xFF), ModbusRtuFrameDecoderResult::CrcError));
    }

    #[test]
    fn rtu_frame_decoder_crc_error_write_single_coil() {
        let mut buffer = [0u8; MODBUS_RTU_FRAME_DATA_LENGTH];
        let mut decoder = ModbusRtuFrameDecoder::new(&mut buffer, ModbusRtuFrameDecoderDirection::Request);

        let _ = decoder.push_data(0x01);
        let _ = decoder.push_data(0x05);
        let _ = decoder.push_data(0x00);
        let _ = decoder.push_data(0xAC);
        let _ = decoder.push_data(0xFF);
        let _ = decoder.push_data(0x00);
        let _ = decoder.push_data(0x00); // Wrong CRC
        assert!(matches!(decoder.push_data(0x00), ModbusRtuFrameDecoderResult::CrcError));
    }

    #[test]
    fn rtu_frame_decoder_crc_error_response() {
        let mut buffer = [0u8; MODBUS_RTU_FRAME_DATA_LENGTH];
        let mut decoder = ModbusRtuFrameDecoder::new(&mut buffer, ModbusRtuFrameDecoderDirection::Response);

        let _ = decoder.push_data(0x01);
        let _ = decoder.push_data(0x03);
        let _ = decoder.push_data(0x04);
        let _ = decoder.push_data(0x00);
        let _ = decoder.push_data(0x06);
        let _ = decoder.push_data(0x00);
        let _ = decoder.push_data(0x05);
        let _ = decoder.push_data(0xAA); // Wrong CRC
        assert!(matches!(decoder.push_data(0xBB), ModbusRtuFrameDecoderResult::CrcError));
    }

    #[test]
    fn rtu_frame_decoder_crc_error_exception_response() {
        let mut buffer = [0u8; MODBUS_RTU_FRAME_DATA_LENGTH];
        let mut decoder = ModbusRtuFrameDecoder::new(&mut buffer, ModbusRtuFrameDecoderDirection::Response);

        let _ = decoder.push_data(0x01);
        let _ = decoder.push_data(0x83); // Exception
        let _ = decoder.push_data(0x02);
        let _ = decoder.push_data(0x12); // Wrong CRC
        assert!(matches!(decoder.push_data(0x34), ModbusRtuFrameDecoderResult::CrcError));
    }

    #[test]
    fn rtu_frame_decoder_crc_error_write_multiple_registers() {
        let mut buffer = [0u8; MODBUS_RTU_FRAME_DATA_LENGTH];
        let mut decoder = ModbusRtuFrameDecoder::new(&mut buffer, ModbusRtuFrameDecoderDirection::Request);

        let _ = decoder.push_data(0x01);
        let _ = decoder.push_data(0x10);
        let _ = decoder.push_data(0x00);
        let _ = decoder.push_data(0x01);
        let _ = decoder.push_data(0x00);
        let _ = decoder.push_data(0x02);
        let _ = decoder.push_data(0x04);
        let _ = decoder.push_data(0x00);
        let _ = decoder.push_data(0x0A);
        let _ = decoder.push_data(0x01);
        let _ = decoder.push_data(0x02);
        let _ = decoder.push_data(0x11); // Wrong CRC
        assert!(matches!(decoder.push_data(0x22), ModbusRtuFrameDecoderResult::CrcError));
    }

    // ========== MODE SWITCHING TESTS ==========
    #[test]
    fn rtu_frame_decoder_switch_from_request_to_response() {
        let mut buffer = [0u8; MODBUS_RTU_FRAME_DATA_LENGTH];
        let mut decoder = ModbusRtuFrameDecoder::new(&mut buffer, ModbusRtuFrameDecoderDirection::Request);

        // First decode a request
        let _ = decoder.push_data(0x01);
        let _ = decoder.push_data(0x03);
        let _ = decoder.push_data(0x00);
        let _ = decoder.push_data(0x00);
        let _ = decoder.push_data(0x00);
        let _ = decoder.push_data(0x02);
        let _ = decoder.push_data(0xC4);
        assert!(matches!(decoder.push_data(0x0B), ModbusRtuFrameDecoderResult::Success(_)));

        // Switch to response mode
        decoder.expect_response();

        // Decode a response
        let _ = decoder.push_data(0x01);
        let _ = decoder.push_data(0x03);
        let _ = decoder.push_data(0x04);
        let _ = decoder.push_data(0x00);
        let _ = decoder.push_data(0x06);
        let _ = decoder.push_data(0x00);
        let _ = decoder.push_data(0x05);
        let _ = decoder.push_data(0xDA);
        assert!(matches!(decoder.push_data(0x31), ModbusRtuFrameDecoderResult::Success(_)));
    }

    #[test]
    fn rtu_frame_decoder_switch_from_response_to_request() {
        let mut buffer = [0u8; MODBUS_RTU_FRAME_DATA_LENGTH];
        let mut decoder = ModbusRtuFrameDecoder::new(&mut buffer, ModbusRtuFrameDecoderDirection::Response);

        // First decode a response
        let _ = decoder.push_data(0x01);
        let _ = decoder.push_data(0x03);
        let _ = decoder.push_data(0x04);
        let _ = decoder.push_data(0x00);
        let _ = decoder.push_data(0x06);
        let _ = decoder.push_data(0x00);
        let _ = decoder.push_data(0x05);
        let _ = decoder.push_data(0xDA);
        assert!(matches!(decoder.push_data(0x31), ModbusRtuFrameDecoderResult::Success(_)));

        // Switch to request mode
        decoder.expect_request();

        // Decode a request
        let _ = decoder.push_data(0x01);
        let _ = decoder.push_data(0x03);
        let _ = decoder.push_data(0x00);
        let _ = decoder.push_data(0x00);
        let _ = decoder.push_data(0x00);
        let _ = decoder.push_data(0x02);
        let _ = decoder.push_data(0xC4);
        assert!(matches!(decoder.push_data(0x0B), ModbusRtuFrameDecoderResult::Success(_)));
    }

    // ========== MULTIPLE FRAMES IN SEQUENCE ==========
    #[test]
    fn rtu_frame_decoder_multiple_requests_in_sequence() {
        let mut buffer = [0u8; MODBUS_RTU_FRAME_DATA_LENGTH];
        let mut decoder = ModbusRtuFrameDecoder::new(&mut buffer, ModbusRtuFrameDecoderDirection::Request);

        // First frame: Read Coils
        let _ = decoder.push_data(0x04);
        let _ = decoder.push_data(0x01);
        let _ = decoder.push_data(0x00);
        let _ = decoder.push_data(0x0A);
        let _ = decoder.push_data(0x00);
        let _ = decoder.push_data(0x0D);
        let _ = decoder.push_data(0xDD);
        assert!(matches!(decoder.push_data(0x98), ModbusRtuFrameDecoderResult::Success(_)));

        // Second frame: Write Single Register
        let _ = decoder.push_data(0x01);
        let _ = decoder.push_data(0x06);
        let _ = decoder.push_data(0x00);
        let _ = decoder.push_data(0x01);
        let _ = decoder.push_data(0x00);
        let _ = decoder.push_data(0x03);
        let _ = decoder.push_data(0x98);
        assert!(matches!(decoder.push_data(0x0B), ModbusRtuFrameDecoderResult::Success(_)));

        // Third frame: Read Holding Registers
        let _ = decoder.push_data(0x01);
        let _ = decoder.push_data(0x03);
        let _ = decoder.push_data(0x00);
        let _ = decoder.push_data(0x00);
        let _ = decoder.push_data(0x00);
        let _ = decoder.push_data(0x02);
        let _ = decoder.push_data(0xC4);
        assert!(matches!(decoder.push_data(0x0B), ModbusRtuFrameDecoderResult::Success(_)));
    }

    #[test]
    fn rtu_frame_decoder_request_response_alternating() {
        let mut buffer = [0u8; MODBUS_RTU_FRAME_DATA_LENGTH];
        let mut decoder = ModbusRtuFrameDecoder::new(&mut buffer, ModbusRtuFrameDecoderDirection::Request);

        // Request
        let _ = decoder.push_data(0x01);
        let _ = decoder.push_data(0x03);
        let _ = decoder.push_data(0x00);
        let _ = decoder.push_data(0x00);
        let _ = decoder.push_data(0x00);
        let _ = decoder.push_data(0x02);
        let _ = decoder.push_data(0xC4);
        assert!(matches!(decoder.push_data(0x0B), ModbusRtuFrameDecoderResult::Success(_)));

        // Switch to response
        decoder.expect_response();

        // Response
        let _ = decoder.push_data(0x01);
        let _ = decoder.push_data(0x03);
        let _ = decoder.push_data(0x04);
        let _ = decoder.push_data(0x00);
        let _ = decoder.push_data(0x06);
        let _ = decoder.push_data(0x00);
        let _ = decoder.push_data(0x05);
        let _ = decoder.push_data(0xDA);
        assert!(matches!(decoder.push_data(0x31), ModbusRtuFrameDecoderResult::Success(_)));

        // Switch back to request
        decoder.expect_request();

        // Another request
        let _ = decoder.push_data(0x01);
        let _ = decoder.push_data(0x06);
        let _ = decoder.push_data(0x00);
        let _ = decoder.push_data(0x01);
        let _ = decoder.push_data(0x00);
        let _ = decoder.push_data(0x03);
        let _ = decoder.push_data(0x98);
        assert!(matches!(decoder.push_data(0x0B), ModbusRtuFrameDecoderResult::Success(_)));
    }

    // ========== EDGE CASES ==========
    #[test]
    fn rtu_frame_decoder_minimum_valid_frame() {
        let mut buffer = [0u8; MODBUS_RTU_FRAME_DATA_LENGTH];
        let mut decoder = ModbusRtuFrameDecoder::new(&mut buffer, ModbusRtuFrameDecoderDirection::Request);

        // GetComEventCounter - shortest valid request (slave + fc + crc)
        let _ = decoder.push_data(0x01);
        let _ = decoder.push_data(0x0B);
        let _ = decoder.push_data(0x41);
        assert!(matches!(decoder.push_data(0xE7), ModbusRtuFrameDecoderResult::Success(_)));
    }

    #[test]
    fn rtu_frame_decoder_reset_after_partial_frame() {
        let mut buffer = [0u8; MODBUS_RTU_FRAME_DATA_LENGTH];
        let mut decoder = ModbusRtuFrameDecoder::new(&mut buffer, ModbusRtuFrameDecoderDirection::Request);

        // Start a frame but don't complete it
        let _ = decoder.push_data(0x01);
        let _ = decoder.push_data(0x03);
        let _ = decoder.push_data(0x00);

        // Manually reset
        decoder.reset();

        // Now send a complete valid frame
        let _ = decoder.push_data(0x01);
        let _ = decoder.push_data(0x03);
        let _ = decoder.push_data(0x00);
        let _ = decoder.push_data(0x00);
        let _ = decoder.push_data(0x00);
        let _ = decoder.push_data(0x02);
        let _ = decoder.push_data(0xC4);
        assert!(matches!(decoder.push_data(0x0B), ModbusRtuFrameDecoderResult::Success(_)));
    }
}



