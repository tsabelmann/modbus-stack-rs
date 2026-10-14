use super::{RegisterData, ReadRegisterData, WriteRegisterData, RegisterResult, read_whole, write_whole};
use crate::Registers;

/// Modbus register that can be converter to and from [u8].
#[derive(Debug, PartialEq, Default, Clone)]
pub struct RegI32 {
    data: [u16; 2]
}

impl RegI32 {
    /// Creates a new [i32] based modbus register.
    /// 
    /// # Example
    ///
    /// ```
    /// use modbus_stack::register::RegI32;
    /// 
    /// let reg = RegI32::new(42);
    /// ````
    pub const fn new(value: i32) -> RegI32 {
        RegI32 {
            data: [((value >> 16) & 0xFFFF) as u16, (value & 0xFFFF) as u16]
        }
    }
}

impl From<i32> for RegI32 {
    fn from(value: i32) -> Self {
        RegI32::new(value)
    }
}

impl From<&i32> for RegI32 {
    fn from(value: &i32) -> Self {
        RegI32::new(*value)
    }
}

impl From<RegI32> for i32 {
    fn from(value: RegI32) -> Self {
        i32::from(&value)
    }
}

impl From<&RegI32> for i32 {
    fn from(value: &RegI32) -> Self {
        ((value.data[0] as i32) << 16) | (value.data[1] as i32)
    }
}

/* RegisterData */

impl RegisterData for RegI32 {
    #[inline]
    fn register_span(&self) -> usize {
        2
    }
}

/* ReadRegisterData */

impl ReadRegisterData for RegI32 {
    #[inline]
    fn read(&self, offset: usize, buf: &mut [u16]) -> RegisterResult<()> {
        read_whole(&self.data, offset, buf)
    }
}

/* WriteRegisterData */

impl WriteRegisterData for RegI32 {
    #[inline]
    fn write(&mut self, offset: usize, values: Registers<'_>) -> RegisterResult<()> {
        write_whole(&mut self.data, offset, values)
    }
}

#[cfg(test)]
mod reg_i32_tests {
    use super::*;

    #[test]
    fn read_001() {
        let regs = [RegI32::new(-1), RegI32::new(-2)];
        let expected = [[0xFFFF, 0xFFFF], [0xFFFF, 0xFFFE]];
        for (reg, expected) in regs.iter().zip(expected) {
            let mut buf = [0u16; 2];
            assert_eq!(reg.read(0, &mut buf), Ok(()));
            assert_eq!(buf, expected);
        }
    }

    #[test]
    fn write_001() {
        let cases = [
            ([0x00FF, 0xEEDD], 0x00FF_EEDD),
            ([0xFFFF, 0xFFFC], -4),
        ];
        for (words, expected) in cases {
            let mut reg = RegI32::new(-1);
            assert_eq!(reg.write(0, Registers::from(&words[..])), Ok(()));
            assert_eq!(i32::from(&reg), expected);
        }
    }

    #[test]
    fn from_to_i32_001() {
        let reg = RegI32::new(-1);
        let value = i32::from(reg);
        assert_eq!(value, -1);
    }
}