use crate::{FunctionKind, AduData, FrameEncoder, PduData};
use crate::constants::MODBUS_RTU_FRAME_DATA_LENGTH;
use crate::rtu::rtu_crc;


#[derive(Debug, PartialEq)]
pub struct ModbusRtuFrame<'a> {
    pub(crate) data: &'a mut [u8; MODBUS_RTU_FRAME_DATA_LENGTH],
    pub(crate) data_length: usize
}

impl<'a> ModbusRtuFrame<'a> {
    /// Creates a new outgoing frame with the given slave address and an empty PDU.
    pub fn new(data: &'a mut [u8; MODBUS_RTU_FRAME_DATA_LENGTH], slave_address: u8) -> ModbusRtuFrame<'a> {
        data[0] = slave_address;
        ModbusRtuFrame { data, data_length: 1 }
    }

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
        u16::from_le_bytes(data)
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
        let end_index: usize = self.data_length.saturating_sub(2);
        self.data.get(1..end_index).unwrap_or(&[])
    }

    fn function_code(&self) -> FunctionKind {
        FunctionKind::from(self.data[1])
    }
}

/* FRAME ENCODER */

impl<'a> FrameEncoder for ModbusRtuFrame<'a> {
    fn payload_mut(&mut self) -> &mut [u8] {
        // slave address and function code before, CRC after the payload
        &mut self.data[2..MODBUS_RTU_FRAME_DATA_LENGTH - 2]
    }

    fn finalize(&mut self, code: FunctionKind, payload_len: usize) {
        let pdu_end = 2 + payload_len;
        assert!(pdu_end <= MODBUS_RTU_FRAME_DATA_LENGTH - 2, "payload exceeds frame capacity");

        // function code
        self.data[1] = u8::from(code);

        // CRC
        let crc = rtu_crc(&self.data[..pdu_end]).to_le_bytes();
        self.data[pdu_end..pdu_end + 2].copy_from_slice(&crc);

        // Total frame length
        self.data_length = pdu_end + 2;
    }
}

#[cfg(test)]
mod frame_tests {
    use super::*;
    use crate::rtu::check_rtu_crc;
    use crate::{ExceptionCode, FunctionCode};
    use crate::write_single_register::Response;

    #[test]
    fn new_frame_001() {
        let mut data = [0u8; MODBUS_RTU_FRAME_DATA_LENGTH];
        let frame = ModbusRtuFrame::new(&mut data, 0x11);
        assert_eq!(frame.slave_address(), 0x11);
        assert_eq!(frame.adu_data(), &[0x11]);
        assert!(frame.pdu_data().is_empty());
    }

    #[test]
    fn encode_001() {
        // Write single register response: address 0x0001, value 0x0003
        let mut data = [0u8; MODBUS_RTU_FRAME_DATA_LENGTH];
        let mut frame = ModbusRtuFrame::new(&mut data, 0x11);
        Response::new(0x0001, 0x0003).encode(&mut frame).unwrap();

        assert_eq!(frame.adu_length(), 8);
        assert_eq!(frame.pdu_data(), &[0x06, 0x00, 0x01, 0x00, 0x03]);
        assert_eq!(frame.function_code(), FunctionKind::Normal(FunctionCode::WriteSingleRegister));

        let adu = frame.adu_data();
        assert!(check_rtu_crc(&adu[..6], adu[6], adu[7]));
        assert_eq!(frame.checksum(), rtu_crc(&adu[..6]));
    }

    #[test]
    fn encode_002() {
        // Exception keeps the slave address
        let mut data = [0u8; MODBUS_RTU_FRAME_DATA_LENGTH];
        let mut frame = ModbusRtuFrame::new(&mut data, 0x22);
        Response::new_exception(ExceptionCode::IllegalDataAddress).encode(&mut frame).unwrap();

        let adu = frame.adu_data();
        assert_eq!(&adu[..3], &[0x22, 0x86, 0x02]);
        assert_eq!(adu.len(), 5);
        assert!(check_rtu_crc(&adu[..3], adu[3], adu[4]));
    }

    #[test]
    fn payload_capacity_001() {
        // Maximum PDU of 253 bytes = function code + 252 bytes payload
        let mut data = [0u8; MODBUS_RTU_FRAME_DATA_LENGTH];
        let mut frame = ModbusRtuFrame::new(&mut data, 0x11);
        assert_eq!(frame.payload_mut().len(), 252);
    }
}
