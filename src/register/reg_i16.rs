use super::{IntoRegIter, IntoRegIterMut};
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

impl IntoIterator for RegI16 {
    type Item = u16;
    type IntoIter = core::array::IntoIter<u16, 1>;
    fn into_iter(self) -> Self::IntoIter {
        self.data.into_iter()
    }
}

impl<'a> IntoIterator for &'a RegI16 {
    type Item = &'a u16;
    type IntoIter = core::slice::Iter<'a, u16>;
    fn into_iter(self) -> Self::IntoIter {
        self.data.iter()
    }
}

impl<'a> IntoIterator for &'a mut RegI16 {
    type Item = &'a mut u16;
    type IntoIter = core::slice::IterMut<'a, u16>;
    fn into_iter(self) -> Self::IntoIter {
        self.data.iter_mut()
    }
}

impl<'a> IntoRegIter<'a> for &'a RegI16 {
    type IntoIter = <&'a RegI16 as IntoIterator>::IntoIter;
    fn into_reg_iter(self) -> Self::IntoIter {
        self.into_iter()
    }
}

impl<'a> IntoRegIterMut<'a> for &'a mut RegI16 {
    type IntoIter = <&'a mut RegI16 as IntoIterator>::IntoIter;
    fn into_reg_iter_mut(self) -> Self::IntoIter {
        self.into_iter()
    }
}

/* RegisterData */

impl RegisterData for RegI16 {
    fn register_span(&self) -> usize {
        1
    }
}

/* ReadRegisterData */

impl ReadRegisterData for RegI16 {
    fn read(&self, offset: usize, buf: &mut [u16]) -> RegisterResult<()> {
        read_words(&self.data, offset, buf)
    }
}

/* WriteRegisterData */

impl WriteRegisterData for RegI16 {
    fn write(&mut self, offset: usize, values: Registers<'_>) -> RegisterResult<()> {
        write_words(&mut self.data, offset, values)
    }
}

#[cfg(test)]
mod reg_i16_tests {
    use crate::register::{IntoRegIter, IntoRegIterMut};
    use super::*;

    #[test]
    fn reg_iter_001() {
        let reg0 = RegI16::new(-1);
        let reg1 = RegI16::new(-2);
        let reg2 = RegI16::new(-3);
        let reg3 = RegI16::new(-4);

        let iter0 = reg0.into_reg_iter();
        let iter1 = reg1.into_reg_iter();
        let iter2 = reg2.into_reg_iter();
        let iter3 = reg3.into_reg_iter();

        let mut iter = iter0.chain(iter1).chain(iter2).chain(iter3);
        assert_eq!(Some(&0xFFFF), iter.next());
        assert_eq!(Some(&0xFFFE), iter.next());
        assert_eq!(Some(&0xFFFD), iter.next());
        assert_eq!(Some(&0xFFFC), iter.next());
        assert_eq!(None, iter.next());
    }

    #[test]
    fn reg_iter_mut_001() {
        let mut reg0 = RegI16::new(-1);
        let mut reg1 = RegI16::new(-2);
        let mut reg2 = RegI16::new(-3);
        let mut reg3 = RegI16::new(-4);

        let iter0 = reg0.into_reg_iter();
        let iter1 = reg1.into_reg_iter();
        let iter2 = reg2.into_reg_iter();
        let iter3 = reg3.into_reg_iter();

        let mut iter = iter0.chain(iter1).chain(iter2).chain(iter3);
        assert_eq!(Some(&0xFFFF), iter.next());
        assert_eq!(Some(&0xFFFE), iter.next());
        assert_eq!(Some(&0xFFFD), iter.next());
        assert_eq!(Some(&0xFFFC), iter.next());
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
    fn from_to_i16_001() {
        let reg = RegI16::new(-2);
        let value = i16::from(reg);
        assert_eq!(value, -2);
    }
}