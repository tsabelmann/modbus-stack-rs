use super::{RegisterData, ReadRegisterData, WriteRegisterData, RegisterResult, read_whole, write_whole};
use crate::Registers;

/// Modbus register that can be converter to and from [u64].
#[derive(Debug, PartialEq, Default, Clone)]
pub struct RegU64 {
    data: [u16; 4]
}

impl RegU64 {
    /// Creates a new [u64] based modbus register.
    /// 
    /// # Example
    ///
    /// ```
    /// use modbus_stack::register::RegU64;
    /// 
    /// let reg = RegU64::new(0x1122334455667788);
    /// ````
    pub const fn new(value: u64) -> RegU64 {
        let data = [
            ((value >> 48) & 0xFFFF) as u16,
            ((value >> 32) & 0xFFFF) as u16,
            ((value >> 16) & 0xFFFF) as u16,
            (value & 0xFFFF) as u16,
        ];

        RegU64 {
            data
        }
    }
}

impl From<u64> for RegU64 {
    fn from(value: u64) -> Self {
        RegU64::new(value)
    }
}

impl From<&u64> for RegU64 {
    fn from(value: &u64) -> Self {
        RegU64::new(*value)
    }
}

impl From<RegU64> for u64 {
    fn from(value: RegU64) -> Self {
        u64::from(&value)
    }
}

impl From<&RegU64> for u64 {
    fn from(value: &RegU64) -> Self {
        ((value.data[0] as u64) << 48) | ((value.data[1] as u64) << 32) | ((value.data[2] as u64) << 16) | (value.data[3] as u64)
    }
}

/* Read/Write Register */

/* RegisterData */

impl RegisterData for RegU64 {
    #[inline]
    fn register_span(&self) -> usize {
        4
    }
}

/* ReadRegisterData */

impl ReadRegisterData for RegU64 {
    #[inline]
    fn read(&self, offset: usize, buf: &mut [u16]) -> RegisterResult<()> {
        read_whole(&self.data, offset, buf)
    }
}

/* WriteRegisterData */

impl WriteRegisterData for RegU64 {
    #[inline]
    fn write(&mut self, offset: usize, values: Registers<'_>) -> RegisterResult<()> {
        write_whole(&mut self.data, offset, values)
    }
}

#[cfg(test)]
mod reg_u64_tests {
    use super::*;

    #[test]
    fn read_001() {
        let regs = [RegU64::new(0x1122334455667788), RegU64::new(0x11223344)];
        let expected = [[0x1122, 0x3344, 0x5566, 0x7788], [0x0000, 0x0000, 0x1122, 0x3344]];
        for (reg, expected) in regs.iter().zip(expected) {
            let mut buf = [0u16; 4];
            assert_eq!(reg.read(0, &mut buf), Ok(()));
            assert_eq!(buf, expected);
        }
    }

    #[test]
    fn write_001() {
        let cases = [
            ([0x00FF, 0xEEDD, 0xCCBB, 0xAA99], 0x00FF_EEDD_CCBB_AA99),
            ([0x8877, 0x6655, 0x4433, 0x2211], 0x8877_6655_4433_2211),
        ];
        for (words, expected) in cases {
            let mut reg = RegU64::new(0x1122334455667788);
            assert_eq!(reg.write(0, Registers::from(&words[..])), Ok(()));
            assert_eq!(u64::from(&reg), expected);
        }
    }

    #[test]
    fn from_to_u32_001() {
        let reg = RegU64::new(0x1122_3344);
        let value = u64::from(reg);
        assert_eq!(value, 0x0000_0000_1122_3344);
    }
}