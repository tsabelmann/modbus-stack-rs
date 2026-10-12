use super::{IntoRegIter, IntoRegIterMut};

/// Modbus register that can be converter to and from [i8].
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
            data: [(value as u16) & 0xFF]
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

impl IntoIterator for RegI8 {
    type Item = u16;
    type IntoIter = core::array::IntoIter<u16, 1>;
    fn into_iter(self) -> Self::IntoIter {
        self.data.into_iter()
    }
}

impl<'a> IntoIterator for &'a RegI8 {
    type Item = &'a u16;
    type IntoIter = core::slice::Iter<'a, u16>;
    fn into_iter(self) -> Self::IntoIter {
        (&self.data).into_iter()
    }
}

impl<'a> IntoIterator for &'a mut RegI8 {
    type Item = &'a mut u16;
    type IntoIter = core::slice::IterMut<'a, u16>;
    fn into_iter(self) -> Self::IntoIter {
        (&mut self.data).into_iter()
    }
}

impl<'a> IntoRegIter<'a> for &'a RegI8 {
    type IntoIter = <&'a RegI8 as IntoIterator>::IntoIter;
    fn into_reg_iter(self) -> Self::IntoIter {
        self.into_iter()
    }
}

impl<'a> IntoRegIterMut<'a> for &'a mut RegI8 {
    type IntoIter = <&'a mut RegI8 as IntoIterator>::IntoIter;
    fn into_reg_iter_mut(self) -> Self::IntoIter {
        self.into_iter()
    }
}

#[cfg(test)]
mod reg_i8_tests {
    use crate::register::{IntoRegIter, IntoRegIterMut};
    use super::*;

    #[test]
    fn reg_iter_001() {
        let reg0 = RegI8::new(-1);
        let reg1 = RegI8::new(-2);
        let reg2 = RegI8::new(-3);
        let reg3 = RegI8::new(-4);

        let iter0 = reg0.into_reg_iter();
        let iter1 = reg1.into_reg_iter();
        let iter2 = reg2.into_reg_iter();
        let iter3 = reg3.into_reg_iter();

        let mut iter = iter0.chain(iter1).chain(iter2).chain(iter3);
        assert_eq!(Some(&0x00FF), iter.next());
        assert_eq!(Some(&0x00FE), iter.next());
        assert_eq!(Some(&0x00FD), iter.next());
        assert_eq!(Some(&0x00FC), iter.next());
        assert_eq!(None, iter.next());
    }

    #[test]
    fn reg_iter_mut_001() {
        let mut reg0 = RegI8::new(0x00);
        let mut reg1 = RegI8::new(0x01);
        let mut reg2 = RegI8::new(0x02);
        let mut reg3 = RegI8::new(0x03);

        let iter0 = reg0.into_reg_iter();
        let iter1 = reg1.into_reg_iter();
        let iter2 = reg2.into_reg_iter();
        let iter3 = reg3.into_reg_iter();

        // check values equal initialization
        let mut iter = iter0.chain(iter1).chain(iter2).chain(iter3);
        assert_eq!(Some(&0), iter.next());
        assert_eq!(Some(&1), iter.next());
        assert_eq!(Some(&2), iter.next());
        assert_eq!(Some(&3), iter.next());
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
    fn from_to_i8_001() {
        let reg = RegI8::new(-1);
        let value = i8::from(reg);
        assert_eq!(value, -1);
    }

}