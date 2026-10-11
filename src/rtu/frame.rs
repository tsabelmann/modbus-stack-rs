use crate::{FunctionKind, AduData, PduData};
use crate::constants::MODBUS_RTU_FRAME_DATA_LENGTH;


#[derive(Debug, PartialEq)]
pub struct ModbusRtuFrame<'a> {
    pub(crate) data: &'a mut [u8; MODBUS_RTU_FRAME_DATA_LENGTH],
    pub(crate) data_length: usize
}

impl<'a> ModbusRtuFrame<'a> {
    pub unsafe fn new_unchecked(data: &'a mut [u8; MODBUS_RTU_FRAME_DATA_LENGTH], data_length: usize) -> ModbusRtuFrame<'a> {
        let data_length = data_length.min(data.len());
        ModbusRtuFrame { data, data_length }
    }

    pub fn slave_address(&self) -> u8 {
        self.data[0]
    }

    pub fn slave_address_mut(&mut self) -> &mut u8 {
        &mut self.data[0]
    }

    pub fn checksum(&self) -> u16 {
        let data = [
            self.data[self.data_length-2],
            self.data[self.data_length-1]
        ];
        let crc = u16::from_le_bytes(data);
        crc
    }
}

/* ADU DATA */

impl<'a> AduData for ModbusRtuFrame<'a> {
    fn adu_data(&self) -> & [u8] {
        &self.data[..self.data_length]
    }

    fn adu_length(&self) -> usize {
        self.data_length
    }
}

/* PDU DATA */

impl<'a> PduData for ModbusRtuFrame<'a> {
    fn pdu_data(&self) -> & [u8] {
        let end_index: usize = self.data_length - 2;
        &self.data[1..end_index]
    }

    fn function_code(&self) -> FunctionKind {
        FunctionKind::from(self.data[1])
    }
}
