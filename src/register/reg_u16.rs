use super::{IntoRegIter, IntoRegIterMut};
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

impl AsMut<[u16]> for RegU16 {
    fn as_mut(&mut self) -> &mut [u16] {
        &mut self.data
    }
}

impl AsRef<[u16]> for RegU16 {
    fn as_ref(&self) -> &[u16] {
        &self.data
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

/* Register Iterator */

impl IntoIterator for RegU16 {
    type Item = u16;
    type IntoIter = core::array::IntoIter<u16, 1>;
    fn into_iter(self) -> Self::IntoIter {
        self.data.into_iter()
    }
}

impl<'a> IntoIterator for &'a RegU16 {
    type Item = &'a u16;
    type IntoIter = core::slice::Iter<'a, u16>;
    fn into_iter(self) -> Self::IntoIter {
        self.data.iter()
    }
}

impl<'a> IntoIterator for &'a mut RegU16 {
    type Item = &'a mut u16;
    type IntoIter = core::slice::IterMut<'a, u16>;
    fn into_iter(self) -> Self::IntoIter {
        self.data.iter_mut()
    }
}

impl<'a> IntoRegIter<'a> for &'a RegU16 {
    type IntoIter = <&'a RegU16 as IntoIterator>::IntoIter;
    fn into_reg_iter(self) -> Self::IntoIter {
        self.into_iter()
    }
}

impl<'a> IntoRegIterMut<'a> for &'a mut RegU16 {
    type IntoIter = <&'a mut RegU16 as IntoIterator>::IntoIter;
    fn into_reg_iter_mut(self) -> Self::IntoIter {
        self.into_iter()
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
    use crate::register::{IntoRegIter, IntoRegIterMut};

    #[test]
    fn reg_iter_001() {
        let reg0 = RegU16::new(0x1234);
        let reg1 = RegU16::new(0x5678);
        let reg2 = RegU16::new(0x9ABC);
        let reg3 = RegU16::new(0xDEF0);

        let iter0 = reg0.into_reg_iter();
        let iter1 = reg1.into_reg_iter();
        let iter2 = reg2.into_reg_iter();
        let iter3 = reg3.into_reg_iter();

        let mut iter = iter0.chain(iter1).chain(iter2).chain(iter3);
        assert_eq!(Some(&0x1234), iter.next());
        assert_eq!(Some(&0x5678), iter.next());
        assert_eq!(Some(&0x9ABC), iter.next());
        assert_eq!(Some(&0xDEF0), iter.next());
        assert_eq!(None, iter.next());
    }

    #[test]
    fn reg_iter_mut_001() {
        let mut reg0 = RegU16::new(0x1234);
        let mut reg1 = RegU16::new(0x5678);
        let mut reg2 = RegU16::new(0x9ABC);
        let mut reg3 = RegU16::new(0xDEF0);

        let iter0 = reg0.into_reg_iter();
        let iter1 = reg1.into_reg_iter();
        let iter2 = reg2.into_reg_iter();
        let iter3 = reg3.into_reg_iter();

        let mut iter = iter0.chain(iter1).chain(iter2).chain(iter3);
        assert_eq!(Some(&0x1234), iter.next());
        assert_eq!(Some(&0x5678), iter.next());
        assert_eq!(Some(&0x9ABC), iter.next());
        assert_eq!(Some(&0xDEF0), iter.next());
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
