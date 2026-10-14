use super::{RegisterData, ReadRegisterData, WriteRegisterData, RegisterResult, read_words, write_words_with};
use crate::Registers;

/// Modbus register that can be converter to and from [i8].
///
/// The value is stored sign-extended, i.e. as an [i16] register.
#[derive(Debug, PartialEq, Default, Clone)]
pub struct RegI8 {
    data: [u16; 1]
}

impl RegI8 {
    /// Creates a new [i8] based modbus register.
    ///
    /// # Example
    ///
    /// ```
    /// use modbus_stack::register::RegI8;
    ///
    /// let reg = RegI8::new(42);
    /// ````
    pub const fn new(value: i8) -> RegI8 {
        RegI8 {
            data: [value as i16 as u16]
        }
    }
}

impl From<i8> for RegI8 {
    fn from(value: i8) -> Self {
        RegI8::new(value)
    }
}

impl From<&i8> for RegI8 {
    fn from(value: &i8) -> Self {
        RegI8::new(*value)
    }
}

impl From<RegI8> for i8 {
    fn from(value: RegI8) -> Self {
        i8::from(&value)
    }
}

impl From<&RegI8> for i8 {
    fn from(value: &RegI8) -> Self {
        (value.data[0] & 0xFF) as i8
    }
}

/* RegisterData */

impl RegisterData for RegI8 {
    #[inline]
    fn register_span(&self) -> usize {
        1
    }
}

/* ReadRegisterData */

impl ReadRegisterData for RegI8 {
    #[inline]
    fn read(&self, offset: usize, buf: &mut [u16]) -> RegisterResult<()> {
        read_words(&self.data, offset, buf)
    }
}

/* WriteRegisterData */

impl WriteRegisterData for RegI8 {
    #[inline]
    fn write(&mut self, offset: usize, values: Registers<'_>) -> RegisterResult<()> {
        write_words_with(&mut self.data, offset, values, |value| {
            (-128..=127).contains(&(value as i16))
        })
    }
}

#[cfg(test)]
mod reg_i8_tests {
    use super::*;
    use crate::register::RegisterError;

    #[test]
    fn read_001() {
        let regs = [RegI8::new(-1), RegI8::new(-128), RegI8::new(127), RegI8::new(0)];
        let expected = [[0xFFFF], [0xFF80], [0x007F], [0x0000]];
        for (reg, expected) in regs.iter().zip(expected) {
            let mut buf = [0u16; 1];
            assert_eq!(reg.read(0, &mut buf), Ok(()));
            assert_eq!(buf, expected);
        }
    }

    #[test]
    fn write_001() {
        let cases = [([0xFFFF], -1), ([0xFF80], -128), ([0x007F], 127), ([0x0000], 0)];
        for (words, expected) in cases {
            let mut reg = RegI8::new(42);
            assert_eq!(reg.write(0, Registers::from(&words[..])), Ok(()));
            assert_eq!(i8::from(&reg), expected);
        }
    }

    #[test]
    fn write_invalid_value_001() {
        let mut reg = RegI8::new(42);
        for value in [0x0080, 0x00FF, 0xFF7F, 0x0100] {
            assert_eq!(
                reg.write(0, Registers::from(&[value][..])),
                Err(RegisterError::InvalidValue)
            );
        }
        assert_eq!(
            reg.write(1, Registers::from(&[0x0000][..])),
            Err(RegisterError::OutOfBounds { offset: 1 })
        );
        assert_eq!(i8::from(&reg), 42);
    }

    #[test]
    fn from_to_i8_001() {
        let reg = RegI8::new(-1);
        let value = i8::from(reg);
        assert_eq!(value, -1);
    }

}
