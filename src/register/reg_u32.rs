use super::{RegisterData, ReadRegisterData, WriteRegisterData, RegisterResult, read_whole, write_whole};
use crate::Registers;

/// Modbus register that can be converter to and from [u32].
#[derive(Debug, PartialEq, Default, Clone)]
pub struct RegU32 {
    data: [u16; 2]
}

impl RegU32 {
    /// Creates a new [u32] based modbus register.
    /// 
    /// # Example
    ///
    /// ```
    /// use modbus_stack::register::RegU32;
    /// 
    /// let reg = RegU32::new(42);
    /// ````
    pub const fn new(value: u32) -> RegU32 {
        RegU32 {
            data: [((value >> 16) & 0xFFFF) as u16, (value & 0xFFFF) as u16]
        }
    }
}

impl From<u32> for RegU32 {
    fn from(value: u32) -> Self {
        RegU32::new(value)
    }
}

impl From<&u32> for RegU32 {
    fn from(value: &u32) -> Self {
        RegU32::new(*value)
    }
}

impl From<RegU32> for u32 {
    fn from(value: RegU32) -> Self {
        u32::from(&value)
    }
}

impl From<&RegU32> for u32 {
    fn from(value: &RegU32) -> Self {
        ((value.data[0] as u32) << 16) | (value.data[1] as u32)
    }
}

/* RegisterData */

impl RegisterData for RegU32 {
    #[inline]
    fn register_span(&self) -> usize {
        2
    }
}

/* ReadRegisterData */

impl ReadRegisterData for RegU32 {
    #[inline]
    fn read(&self, offset: usize, buf: &mut [u16]) -> RegisterResult<()> {
        read_whole(&self.data, offset, buf)
    }
}

/* WriteRegisterData */

impl WriteRegisterData for RegU32 {
    #[inline]
    fn write(&mut self, offset: usize, values: Registers<'_>) -> RegisterResult<()> {
        write_whole(&mut self.data, offset, values)
    }
}

#[cfg(test)]
mod reg_u32_tests {
    use super::*;

    #[test]
    fn read_001() {
        let regs = [RegU32::new(0x11223344), RegU32::new(0xAABBCCDD)];
        let expected = [[0x1122, 0x3344], [0xAABB, 0xCCDD]];
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
            ([0x8877, 0x6655], 0x8877_6655),
        ];
        for (words, expected) in cases {
            let mut reg = RegU32::new(0x11223344);
            assert_eq!(reg.write(0, Registers::from(&words[..])), Ok(()));
            assert_eq!(u32::from(&reg), expected);
        }
    }

    #[test]
    fn from_to_u32_001() {
        let reg = RegU32::new(0x11223344);
        let value = u32::from(reg);
        assert_eq!(value, 0x11223344);
    }
}