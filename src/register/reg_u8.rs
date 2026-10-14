use super::{RegisterData, ReadRegisterData, WriteRegisterData, RegisterResult, read_words, write_words_with};
use crate::Registers;

/// Modbus register that can be converter to and from [u8].
#[derive(Debug, PartialEq, Default, Clone)]
pub struct RegU8 {
    data: [u16; 1]
}

impl RegU8 {
    /// Creates a new [u8] based modbus register.
    /// 
    /// # Example
    ///
    /// ```
    /// use modbus_stack::register::RegU8;
    /// 
    /// let reg = RegU8::new(42);
    /// ````
    pub const fn new(value: u8) -> RegU8 {
        RegU8 {
            data: [(value as u16) & 0xFF]
        }
    }
}

impl From<u8> for RegU8 {
    fn from(value: u8) -> Self {
        RegU8::new(value)
    }
}

impl From<&u8> for RegU8 {
    fn from(value: &u8) -> Self {
        RegU8::new(*value)
    }
}

impl From<RegU8> for u8 {
    fn from(value: RegU8) -> Self {
        u8::from(&value)
    }
}

impl From<&RegU8> for u8 {
    fn from(value: &RegU8) -> Self {
        (value.data[0] & 0xFF) as u8
    }
}

/* RegisterData */

impl RegisterData for RegU8 {
    #[inline]
    fn register_span(&self) -> usize {
        1
    }
}

/* ReadRegisterData */

impl ReadRegisterData for RegU8 {
    #[inline]
    fn read(&self, offset: usize, buf: &mut [u16]) -> RegisterResult<()> {
        read_words(&self.data, offset, buf)
    }
}

/* WriteRegisterData */

impl WriteRegisterData for RegU8 {
    #[inline]
    fn write(&mut self, offset: usize, values: Registers<'_>) -> RegisterResult<()> {
        write_words_with(&mut self.data, offset, values, |value| value <= 0x00FF)
    }
}

#[cfg(test)]
mod reg_u8_tests {
    use super::*;

    #[test]
    fn read_001() {
        let regs = [RegU8::new(0x00), RegU8::new(0x01), RegU8::new(0xFF)];
        let expected = [[0x0000], [0x0001], [0x00FF]];
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
            ([0x00FF], 0xFF),
            ([0], 0),
        ];
        for (words, expected) in cases {
            let mut reg = RegU8::new(0x00);
            assert_eq!(reg.write(0, Registers::from(&words[..])), Ok(()));
            assert_eq!(u8::from(&reg), expected);
        }
    }

    #[test]
    fn from_to_u8_001() {
        let reg = RegU8::new(42);
        let value = u8::from(reg);
        assert_eq!(value, 42);
    }

}