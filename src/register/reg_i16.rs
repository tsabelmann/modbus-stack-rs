use super::{RegisterData, ReadRegisterData, WriteRegisterData, RegisterResult, read_words, write_words};
use crate::Registers;

/// Modbus register that can be converter to and from [i16].
#[derive(Debug, PartialEq, Default, Clone)]
pub struct RegI16 {
    data: [u16; 1]
}

impl RegI16 {
    /// Creates a new [i16] based modbus register.
    /// 
    /// # Example
    ///
    /// ```
    /// use modbus_stack::register::RegI16;
    /// 
    /// let reg = RegI16::new(42);
    /// ````
    pub const fn new(value: i16) -> RegI16 {
        RegI16 {
            data: [value as u16]
        }
    }
}

impl From<i16> for RegI16 {
    fn from(value: i16) -> Self {
        RegI16::new(value)
    }
}

impl From<&i16> for RegI16 {
    fn from(value: &i16) -> Self {
        RegI16::new(*value)
    }
}

impl From<RegI16> for i16 {
    fn from(value: RegI16) -> Self {
        i16::from(&value)
    }
}

impl From<&RegI16> for i16 {
    fn from(value: &RegI16) -> Self {
        value.data[0] as i16
    }
}

/* RegisterData */

impl RegisterData for RegI16 {
    #[inline]
    fn register_span(&self) -> usize {
        1
    }
}

/* ReadRegisterData */

impl ReadRegisterData for RegI16 {
    #[inline]
    fn read(&self, offset: usize, buf: &mut [u16]) -> RegisterResult<()> {
        read_words(&self.data, offset, buf)
    }
}

/* WriteRegisterData */

impl WriteRegisterData for RegI16 {
    #[inline]
    fn write(&mut self, offset: usize, values: Registers<'_>) -> RegisterResult<()> {
        write_words(&mut self.data, offset, values)
    }
}

#[cfg(test)]
mod reg_i16_tests {
    use super::*;

    #[test]
    fn read_001() {
        let regs = [RegI16::new(-1), RegI16::new(-4), RegI16::new(42)];
        let expected = [[0xFFFF], [0xFFFC], [0x002A]];
        for (reg, expected) in regs.iter().zip(expected) {
            let mut buf = [0u16; 1];
            assert_eq!(reg.read(0, &mut buf), Ok(()));
            assert_eq!(buf, expected);
        }
    }

    #[test]
    fn write_001() {
        let cases = [
            ([42], 42),
            ([1337], 1337),
            ([65525], -11),
        ];
        for (words, expected) in cases {
            let mut reg = RegI16::new(-1);
            assert_eq!(reg.write(0, Registers::from(&words[..])), Ok(()));
            assert_eq!(i16::from(&reg), expected);
        }
    }

    #[test]
    fn from_to_i16_001() {
        let reg = RegI16::new(-2);
        let value = i16::from(reg);
        assert_eq!(value, -2);
    }
}