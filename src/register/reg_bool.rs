use super::{
    ReadRegisterData, RegisterData, RegisterResult, WriteRegisterData, read_words, write_words_with,
};
use crate::Registers;

/// Modbus register that can be converter to and from [bool].
#[derive(Debug, PartialEq, Default, Clone)]
pub struct RegBool {
    data: [u16; 1],
}

impl RegBool {
    /// Creates a new [bool] based modbus register.
    ///
    /// # Example
    ///
    /// ```
    /// use modbus_stack::register::RegBool;
    ///
    /// let reg = RegBool::new(true);
    /// ````
    pub const fn new(value: bool) -> RegBool {
        let data = match value {
            false => [0x0000],
            true => [0xFFFF],
        };

        RegBool { data }
    }
}

impl From<bool> for RegBool {
    fn from(value: bool) -> Self {
        RegBool::new(value)
    }
}

impl From<&bool> for RegBool {
    fn from(value: &bool) -> Self {
        RegBool::new(*value)
    }
}

impl From<RegBool> for bool {
    fn from(value: RegBool) -> Self {
        bool::from(&value)
    }
}

impl From<&RegBool> for bool {
    fn from(value: &RegBool) -> Self {
        value.data[0] != 0x0000
    }
}

/* RegisterData */

impl RegisterData for RegBool {
    #[inline]
    fn register_span(&self) -> usize {
        1
    }
}

/* ReadRegisterData */

impl ReadRegisterData for RegBool {
    #[inline]
    fn read(&self, offset: usize, buf: &mut [u16]) -> RegisterResult<()> {
        read_words(&self.data, offset, buf)
    }
}

/* WriteRegisterData */

impl WriteRegisterData for RegBool {
    #[inline]
    fn write(&mut self, offset: usize, values: Registers<'_>) -> RegisterResult<()> {
        write_words_with(&mut self.data, offset, values, |value| matches!(value, 0x0000 | 0xFFFF))
    }
}

#[cfg(test)]
mod reg_bool_tests {
    use super::*;

    #[test]
    fn read_001() {
        let regs = [RegBool::new(false), RegBool::new(true)];
        let expected = [[0x0000], [0xFFFF]];
        for (reg, expected) in regs.iter().zip(expected) {
            let mut buf = [0u16; 1];
            assert_eq!(reg.read(0, &mut buf), Ok(()));
            assert_eq!(buf, expected);
        }
    }

    #[test]
    fn write_001() {
        let cases = [
            ([0xFFFF], true),
            ([0x0000], false),
        ];
        for (words, expected) in cases {
            let mut reg = RegBool::new(false);
            assert_eq!(reg.write(0, Registers::from(&words[..])), Ok(()));
            assert_eq!(bool::from(&reg), expected);
        }
    }

    #[test]
    fn from_to_u8_001() {
        let reg = RegBool::new(true);
        let value = bool::from(reg);
        assert!(value);
    }
}
