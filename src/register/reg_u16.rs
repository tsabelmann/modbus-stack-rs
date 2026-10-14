use super::{
    ReadRegisterData, RegisterData, RegisterResult, WriteRegisterData, read_words, write_words,
};
use crate::Registers;

/// Modbus register that can be converter to and from [u16].
#[derive(Debug, PartialEq, Default, Clone)]
pub struct RegU16 {
    data: [u16; 1],
}

impl RegU16 {
    /// Creates a new [u16] based modbus register.
    ///
    /// # Example
    ///
    /// ```
    /// use modbus_stack::register::RegU16;
    ///
    /// let reg = RegU16::new(42);
    /// ````
    pub const fn new(value: u16) -> RegU16 {
        RegU16 { data: [value] }
    }
}

impl From<u16> for RegU16 {
    fn from(value: u16) -> Self {
        RegU16::new(value)
    }
}

impl From<&u16> for RegU16 {
    fn from(value: &u16) -> Self {
        RegU16::new(*value)
    }
}

impl From<RegU16> for u16 {
    fn from(value: RegU16) -> Self {
        u16::from(&value)
    }
}

impl From<&RegU16> for u16 {
    fn from(value: &RegU16) -> Self {
        value.data[0]
    }
}

/* RegisterData */

impl RegisterData for RegU16 {
    #[inline]
    fn register_span(&self) -> usize {
        1
    }
}

/* ReadRegisterData */

impl ReadRegisterData for RegU16 {
    #[inline]
    fn read(&self, offset: usize, buf: &mut [u16]) -> RegisterResult<()> {
        read_words(&self.data, offset, buf)
    }
}

/* WriteRegisterData */

impl WriteRegisterData for RegU16 {
    #[inline]
    fn write(&mut self, offset: usize, values: Registers<'_>) -> RegisterResult<()> {
        write_words(&mut self.data, offset, values)
    }
}

#[cfg(test)]
mod reg_u16_tests {
    use super::*;

    #[test]
    fn read_001() {
        let regs = [RegU16::new(0x1234), RegU16::new(0xDEF0)];
        let expected = [[0x1234], [0xDEF0]];
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
            ([65525], 65525),
        ];
        for (words, expected) in cases {
            let mut reg = RegU16::new(0x1234);
            assert_eq!(reg.write(0, Registers::from(&words[..])), Ok(()));
            assert_eq!(u16::from(&reg), expected);
        }
    }

    #[test]
    fn from_to_u8_001() {
        let reg = RegU16::new(42);
        let value = u16::from(reg);
        assert_eq!(value, 42);
    }

    #[test]
    fn from_to_u8_002() {
        let reg = RegU16::new(0x1234);
        let value = u16::from(reg);
        assert_eq!(value, 0x1234);
    }
}
