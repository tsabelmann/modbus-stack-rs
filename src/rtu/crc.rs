/// Compute checksum to verify the validity of Modbus/RTU frames.
pub fn rtu_crc(data: &[u8]) -> u16 {
    let mut crc = 0xFFFF;
    for byte in data {
        crc ^= *byte as u16;
        for _ in 0..8 {
            if crc & 0x0001 != 0 {
                crc >>= 1;
                crc ^= 0xA001;
            } else {
                crc >>= 1;
            }
        }
    }

    crc
}

/// Check the computed checksum against the transmitted/received checksum.
pub fn check_rtu_crc(data: &[u8], crc_low: u8, crc_high: u8) -> bool {
    let computed_crc = rtu_crc(data);
    let bytes = computed_crc.to_le_bytes();

    bytes[0] == crc_low && bytes[1] == crc_high
}
