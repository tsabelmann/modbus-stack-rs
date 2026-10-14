use super::{RegisterData, ReadRegisterData, WriteRegisterData, RegisterResult, read_whole, write_whole};
use crate::Registers;

/// Modbus register that can be converter to and from [i64].
#[derive(Debug, PartialEq, Default, Clone)]
pub struct RegI64 {
    data: [u16; 4]
}

impl RegI64 {
    /// Creates a new [i64] based modbus register.
    /// 
    /// # Example
    ///
    /// ```
    /// use modbus_stack::register::RegI64;
    /// 
    /// let reg = RegI64::new(0x1122334455667788);
    /// ````
    pub const fn new(value: i64) -> RegI64 {
        let data = [
            ((value >> 48) & 0xFFFF) as u16,
            ((value >> 32) & 0xFFFF) as u16,
            ((value >> 16) & 0xFFFF) as u16,
            (value & 0xFFFF) as u16,
        ];

        RegI64 {
            data
        }
    }
}

impl From<i64> for RegI64 {
    fn from(value: i64) -> Self {
        RegI64::new(value)
    }
}

impl From<&i64> for RegI64 {
    fn from(value: &i64) -> Self {
        RegI64::new(*value)
    }
}

impl From<RegI64> for i64 {
    fn from(value: RegI64) -> Self {
        i64::from(&value)
    }
}

impl From<&RegI64> for i64 {
    fn from(value: &RegI64) -> Self {
        ((value.data[0] as i64) << 48) | ((value.data[1] as i64) << 32) | ((value.data[2] as i64) << 16) | (value.data[3] as i64)
    }
}

/* RegisterData */

impl RegisterData for RegI64 {
    #[inline]
    fn register_span(&self) -> usize {
        4
    }
}

/* ReadRegisterData */

impl ReadRegisterData for RegI64 {
    #[inline]
    fn read(&self, offset: usize, buf: &mut [u16]) -> RegisterResult<()> {
        read_whole(&self.data, offset, buf)
    }
}

/* WriteRegisterData */

impl WriteRegisterData for RegI64 {
    #[inline]
    fn write(&mut self, offset: usize, values: Registers<'_>) -> RegisterResult<()> {
        write_whole(&mut self.data, offset, values)
    }
}

#[cfg(test)]
mod reg_i64_tests {
    use super::*;

    #[test]
    fn read_001() {
        let regs = [RegI64::new(-1), RegI64::new(-2), RegI64::new(0x1122_3344_5566_7788)];
        let expected = [[0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF], [0xFFFF, 0xFFFF, 0xFFFF, 0xFFFE], [0x1122, 0x3344, 0x5566, 0x7788]];
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
            ([0xFFFF, 0xFFFF, 0xFFFF, 0xFFFF], -1),
        ];
        for (words, expected) in cases {
            let mut reg = RegI64::new(-1);
            assert_eq!(reg.write(0, Registers::from(&words[..])), Ok(()));
            assert_eq!(i64::from(&reg), expected);
        }
    }

    #[test]
    fn from_to_i64_001() {
        let reg = RegI64::new(-1);
        let value = i64::from(reg);
        assert_eq!(value, -1);
    }
}