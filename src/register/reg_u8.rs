use super::{IntoRegIter, IntoRegIterMut};
use super::{RegisterData, ReadRegisterData, WriteRegisterData, RegisterResult, RegisterError};

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

impl IntoIterator for RegU8 {
    type Item = u16;
    type IntoIter = core::array::IntoIter<u16, 1>;
    fn into_iter(self) -> Self::IntoIter {
        self.data.into_iter()
    }
}

impl<'a> IntoIterator for &'a RegU8 {
    type Item = &'a u16;
    type IntoIter = core::slice::Iter<'a, u16>;
    fn into_iter(self) -> Self::IntoIter {
        self.data.iter()
    }
}

impl<'a> IntoIterator for &'a mut RegU8 {
    type Item = &'a mut u16;
    type IntoIter = core::slice::IterMut<'a, u16>;
    fn into_iter(self) -> Self::IntoIter {
        self.data.iter_mut()
    }
}

impl<'a> IntoRegIter<'a> for &'a RegU8 {
    type IntoIter = <&'a RegU8 as IntoIterator>::IntoIter;
    fn into_reg_iter(self) -> Self::IntoIter {
        self.into_iter()
    }
}

impl<'a> IntoRegIterMut<'a> for &'a mut RegU8 {
    type IntoIter = <&'a mut RegU8 as IntoIterator>::IntoIter;
    fn into_reg_iter_mut(self) -> Self::IntoIter {
        self.into_iter()
    }
}

/* RegisterData */

impl RegisterData for RegU8 {
    fn register_span(&self) -> usize {
        1
    }
}

/* ReadRegisterData */

impl ReadRegisterData for RegU8 {
    fn read(&self, offset: usize, buf: &mut [u16]) -> RegisterResult<usize> {
        match offset {
            0 => {
                match buf.len() {
                    0 => Ok(0),
                    _ => {
                        buf[0] = self.data[0] & 0x00FF;
                        Ok(1)
                    }
                }
            },
            _ => {
                RegisterResult::Err(RegisterError::OutOfBounds { offset })
            }
        }
    }
}

/* WriteRegisterData */

impl WriteRegisterData for RegU8 {
    fn write(&mut self, offset: usize, buf: &[u16]) -> RegisterResult<usize> {
        match offset {
            0 => {
                match buf.len() {
                    0 => Ok(0),
                    _ => {
                        self.data[0] = buf[0] & 0x00FF;
                        Ok(1)
                    }
                }
            },
            _ => {
                RegisterResult::Err(RegisterError::OutOfBounds { offset })
            }
        }
    }
}

#[cfg(test)]
mod reg_u8_tests {
    use crate::register::{IntoRegIter, IntoRegIterMut};
    use super::*;

    #[test]
    fn reg_iter_001() {
        let reg0 = RegU8::new(0x00);
        let reg1 = RegU8::new(0x01);
        let reg2 = RegU8::new(0x02);
        let reg3 = RegU8::new(0x03);

        let iter0 = reg0.into_reg_iter();
        let iter1 = reg1.into_reg_iter();
        let iter2 = reg2.into_reg_iter();
        let iter3 = reg3.into_reg_iter();

        let mut iter = iter0.chain(iter1).chain(iter2).chain(iter3);
        assert_eq!(Some(&0), iter.next());
        assert_eq!(Some(&1), iter.next());
        assert_eq!(Some(&2), iter.next());
        assert_eq!(Some(&3), iter.next());
        assert_eq!(None, iter.next());
    }

    #[test]
    fn reg_iter_mut_001() {
        let mut reg0 = RegU8::new(0x00);
        let mut reg1 = RegU8::new(0x01);
        let mut reg2 = RegU8::new(0x02);
        let mut reg3 = RegU8::new(0x03);

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
    fn from_to_u8_001() {
        let reg = RegU8::new(42);
        let value = u8::from(reg);
        assert_eq!(value, 42);
    }

}