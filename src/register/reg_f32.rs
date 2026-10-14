use super::{RegisterData, ReadRegisterData, WriteRegisterData, RegisterResult, read_whole, write_whole};
use crate::Registers;

/// Modbus register that can be converter to and from [f32].
#[derive(Debug, PartialEq, Default, Clone)]
pub struct RegF32 {
    data: [u16; 2]
}

impl RegF32 {
    /// Creates a new [f32] based modbus register.
    /// 
    /// # Example
    ///
    /// ```
    /// use modbus_stack::register::RegF32;
    /// 
    /// let reg = RegF32::new(42.0);
    /// ````
    pub const fn new(value: f32) -> RegF32 {
        let data = f32::to_be_bytes(value);
        let data = [
            ((data[0] as u16) << 8) | (data[1] as u16),
            ((data[2] as u16) << 8) | (data[3] as u16),
        ];
        
        RegF32 {
            data
        }
    }
}

impl From<f32> for RegF32 {
    fn from(value: f32) -> Self {
        RegF32::new(value)
    }
}

impl From<&f32> for RegF32 {
    fn from(value: &f32) -> Self {
        RegF32::new(*value)
    }
}

impl From<RegF32> for f32 {
    fn from(value: RegF32) -> Self {
        f32::from(&value)
    }
}

impl From<&RegF32> for f32 {
    fn from(value: &RegF32) -> Self {
        let mut data = [0u8; 4];
        data[0] = ((value.data[0] >> 8) & 0xFF) as u8;
        data[1] = (value.data[0] & 0xFF) as u8;
        data[2] = ((value.data[1] >> 8) & 0xFF) as u8;
        data[3] = (value.data[1] & 0xFF) as u8;
        f32::from_be_bytes(data)
    }
}

/* RegisterData */

impl RegisterData for RegF32 {
    #[inline]
    fn register_span(&self) -> usize {
        2
    }
}

/* ReadRegisterData */

impl ReadRegisterData for RegF32 {
    #[inline]
    fn read(&self, offset: usize, buf: &mut [u16]) -> RegisterResult<()> {
        read_whole(&self.data, offset, buf)
    }
}

/* WriteRegisterData */

impl WriteRegisterData for RegF32 {
    #[inline]
    fn write(&mut self, offset: usize, values: Registers<'_>) -> RegisterResult<()> {
        write_whole(&mut self.data, offset, values)
    }
}

#[cfg(test)]
mod reg_f32_tests {
    use super::*;

    /// Big endian register representation of `value`.
    fn words(value: f32) -> [u16; 2] {
        let bytes = value.to_be_bytes();
        core::array::from_fn(|i| u16::from_be_bytes([bytes[2 * i], bytes[2 * i + 1]]))
    }

    #[test]
    fn read_001() {
        for value in [0.0, 42.0, -1.5] {
            let reg = RegF32::new(value);
            let mut buf = [0u16; 2];
            assert_eq!(reg.read(0, &mut buf), Ok(()));
            assert_eq!(buf, words(value));
        }
    }

    #[test]
    fn write_001() {
        let mut reg = RegF32::new(0.0);
        assert_eq!(reg.write(0, Registers::from(&words(42.0)[..])), Ok(()));
        assert_eq!(f32::from(&reg), 42.0);
    }

    #[test]
    fn from_to_f32_001() {
        let reg = RegF32::new(42.0);
        let value = f32::from(reg);
        assert_eq!(value, 42.0);
    }
}