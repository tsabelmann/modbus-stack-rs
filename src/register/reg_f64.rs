use super::{RegisterData, ReadRegisterData, WriteRegisterData, RegisterResult, read_whole, write_whole};
use crate::Registers;

/// Modbus register that can be converter to and from [f64].
#[derive(Debug, PartialEq, Default, Clone)]
pub struct RegF64 {
    data: [u16; 4]
}

impl RegF64 {
    /// Creates a new [f64] based modbus register.
    /// 
    /// # Example
    ///
    /// ```
    /// use modbus_stack::register::RegF64;
    /// 
    /// let reg = RegF64::new(42.0);
    /// ````
    pub const fn new(value: f64) -> RegF64 {
        let data = f64::to_be_bytes(value);
        let data = [
            ((data[0] as u16) << 8) | (data[1] as u16),
            ((data[2] as u16) << 8) | (data[3] as u16),
            ((data[4] as u16) << 8) | (data[5] as u16),
            ((data[6] as u16) << 8) | (data[7] as u16),
        ];
        
        RegF64 {
            data
        }
    }
}

impl From<f64> for RegF64 {
    fn from(value: f64) -> Self {
        RegF64::new(value)
    }
}

impl From<&f64> for RegF64 {
    fn from(value: &f64) -> Self {
        RegF64::new(*value)
    }
}

impl From<RegF64> for f64 {
    fn from(value: RegF64) -> Self {
        f64::from(&value)
    }
}

impl From<&RegF64> for f64 {
    fn from(value: &RegF64) -> Self {
        let mut data = [0u8; 8];
        data[0] = ((value.data[0] >> 8) & 0xFF) as u8;
        data[1] = (value.data[0] & 0xFF) as u8;
        data[2] = ((value.data[1] >> 8) & 0xFF) as u8;
        data[3] = (value.data[1] & 0xFF) as u8;
        data[4] = ((value.data[2] >> 8) & 0xFF) as u8;
        data[5] = (value.data[2] & 0xFF) as u8;
        data[6] = ((value.data[3] >> 8) & 0xFF) as u8;
        data[7] = (value.data[3] & 0xFF) as u8;
        f64::from_be_bytes(data)
    }
}

/* RegisterData */

impl RegisterData for RegF64 {
    #[inline]
    fn register_span(&self) -> usize {
        4
    }
}

/* ReadRegisterData */

impl ReadRegisterData for RegF64 {
    #[inline]
    fn read(&self, offset: usize, buf: &mut [u16]) -> RegisterResult<()> {
        read_whole(&self.data, offset, buf)
    }
}

/* WriteRegisterData */

impl WriteRegisterData for RegF64 {
    #[inline]
    fn write(&mut self, offset: usize, values: Registers<'_>) -> RegisterResult<()> {
        write_whole(&mut self.data, offset, values)
    }
}

#[cfg(test)]
mod reg_f64_tests {
    use super::*;

    /// Big endian register representation of `value`.
    fn words(value: f64) -> [u16; 4] {
        let bytes = value.to_be_bytes();
        core::array::from_fn(|i| u16::from_be_bytes([bytes[2 * i], bytes[2 * i + 1]]))
    }

    #[test]
    fn read_001() {
        for value in [0.0, 42.0, -1.5] {
            let reg = RegF64::new(value);
            let mut buf = [0u16; 4];
            assert_eq!(reg.read(0, &mut buf), Ok(()));
            assert_eq!(buf, words(value));
        }
    }

    #[test]
    fn write_001() {
        let mut reg = RegF64::new(0.0);
        assert_eq!(reg.write(0, Registers::from(&words(42.0)[..])), Ok(()));
        assert_eq!(f64::from(&reg), 42.0);
    }

    #[test]
    fn from_to_f64_001() {
        let reg = RegF64::new(42.0);
        let value = f64::from(reg);
        assert_eq!(value, 42.0);
    }
}