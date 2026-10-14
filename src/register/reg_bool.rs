use super::{
    ReadRegisterData, RegisterData, RegisterResult, WriteRegisterData, read_words, write_words_with,
};
use crate::Registers;
use crate::register::{IntoRegIter, IntoRegIterMut};

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

impl IntoIterator for RegBool {
    type Item = u16;
    type IntoIter = core::array::IntoIter<u16, 1>;
    fn into_iter(self) -> Self::IntoIter {
        self.data.into_iter()
    }
}

impl<'a> IntoIterator for &'a RegBool {
    type Item = &'a u16;
    type IntoIter = core::slice::Iter<'a, u16>;
    fn into_iter(self) -> Self::IntoIter {
        self.data.iter()
    }
}

impl<'a> IntoIterator for &'a mut RegBool {
    type Item = &'a mut u16;
    type IntoIter = core::slice::IterMut<'a, u16>;
    fn into_iter(self) -> Self::IntoIter {
        self.data.iter_mut()
    }
}

impl<'a> IntoRegIter<'a> for &'a RegBool {
    type IntoIter = <&'a RegBool as IntoIterator>::IntoIter;
    fn into_reg_iter(self) -> Self::IntoIter {
        self.into_iter()
    }
}

impl<'a> IntoRegIterMut<'a> for &'a mut RegBool {
    type IntoIter = <&'a mut RegBool as IntoIterator>::IntoIter;
    fn into_reg_iter_mut(self) -> Self::IntoIter {
        self.into_iter()
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
    use crate::register::{IntoRegIter, IntoRegIterMut};

    #[test]
    fn reg_iter_001() {
        let reg0 = RegBool::new(false);
        let reg1 = RegBool::new(true);
        let reg2 = RegBool::new(false);
        let reg3 = RegBool::new(true);

        let iter0 = reg0.into_reg_iter();
        let iter1 = reg1.into_reg_iter();
        let iter2 = reg2.into_reg_iter();
        let iter3 = reg3.into_reg_iter();

        let mut iter = iter0.chain(iter1).chain(iter2).chain(iter3);
        assert_eq!(Some(&0x0000), iter.next());
        assert_eq!(Some(&0xFFFF), iter.next());
        assert_eq!(Some(&0x0000), iter.next());
        assert_eq!(Some(&0xFFFF), iter.next());
        assert_eq!(None, iter.next());
    }

    #[test]
    fn reg_iter_mut_001() {
        let mut reg0 = RegBool::new(false);
        let mut reg1 = RegBool::new(true);
        let mut reg2 = RegBool::new(true);
        let mut reg3 = RegBool::new(false);

        let iter0 = reg0.into_reg_iter();
        let iter1 = reg1.into_reg_iter();
        let iter2 = reg2.into_reg_iter();
        let iter3 = reg3.into_reg_iter();

        // check values equal initialization
        let mut iter = iter0.chain(iter1).chain(iter2).chain(iter3);
        assert_eq!(Some(&0x0000), iter.next());
        assert_eq!(Some(&0xFFFF), iter.next());
        assert_eq!(Some(&0xFFFF), iter.next());
        assert_eq!(Some(&0x0000), iter.next());
        assert_eq!(None, iter.next());

        // Change values
        let iter0 = reg0.into_reg_iter_mut();
        let iter1 = reg1.into_reg_iter_mut();
        let iter2 = reg2.into_reg_iter_mut();
        let iter3 = reg3.into_reg_iter_mut();

        let mut iter = iter0.chain(iter1).chain(iter2).chain(iter3);
        if let Some(reff) = iter.next() {
            *reff = 42;
        }

        if let Some(reff) = iter.next() {
            *reff = 1337;
        }

        if let Some(reff) = iter.next() {
            *reff = 65525;
        }

        if let Some(reff) = iter.next() {
            *reff = 1;
        }

        // check for change
        let iter0 = reg0.into_reg_iter();
        let iter1 = reg1.into_reg_iter();
        let iter2 = reg2.into_reg_iter();
        let iter3 = reg3.into_reg_iter();

        let mut iter = iter0.chain(iter1).chain(iter2).chain(iter3);
        assert_eq!(Some(&42), iter.next());
        assert_eq!(Some(&1337), iter.next());
        assert_eq!(Some(&65525), iter.next());
        assert_eq!(Some(&1), iter.next());
        assert_eq!(None, iter.next());
    }

    #[test]
    fn from_to_u8_001() {
        let reg = RegBool::new(true);
        let value = bool::from(reg);
        assert!(value);
    }
}
