use super::{IntoRegIter, IntoRegIterMut};
use super::{RegisterData, ReadRegisterData, WriteRegisterData, RegisterResult, RegisterError};

/// Modbus register that can be converter to and from [i64].
#[derive(Debug, PartialEq, Default, Clone)]
pub struct RegI64 {
    data: [u16; 4]
}

impl RegI64 {
    /// Creates a new [i64] based modbus register.
    /// 
    /// # Example
    ///
    /// ```
    /// use modbus_stack::register::RegI64;
    /// 
    /// let reg = RegI64::new(0x1122334455667788);
    /// ````
    pub const fn new(value: i64) -> RegI64 {
        let data = [
            ((value >> 48) & 0xFFFF) as u16,
            ((value >> 32) & 0xFFFF) as u16,
            ((value >> 16) & 0xFFFF) as u16,
            (value & 0xFFFF) as u16,
        ];

        RegI64 {
            data
        }
    }
}

impl From<i64> for RegI64 {
    fn from(value: i64) -> Self {
        RegI64::new(value)
    }
}

impl From<&i64> for RegI64 {
    fn from(value: &i64) -> Self {
        RegI64::new(*value)
    }
}

impl From<RegI64> for i64 {
    fn from(value: RegI64) -> Self {
        i64::from(&value)
    }
}

impl From<&RegI64> for i64 {
    fn from(value: &RegI64) -> Self {
        ((value.data[0] as i64) << 48) | ((value.data[1] as i64) << 32) | ((value.data[2] as i64) << 16) | (value.data[3] as i64)
    }
}

impl IntoIterator for RegI64 {
    type Item = u16;
    type IntoIter = core::array::IntoIter<u16, 4>;
    fn into_iter(self) -> Self::IntoIter {
        self.data.into_iter()
    }
}

impl<'a> IntoIterator for &'a RegI64 {
    type Item = &'a u16;
    type IntoIter = core::slice::Iter<'a, u16>;
    fn into_iter(self) -> Self::IntoIter {
        self.data.iter()
    }
}

impl<'a> IntoIterator for &'a mut RegI64 {
    type Item = &'a mut u16;
    type IntoIter = core::slice::IterMut<'a, u16>;
    fn into_iter(self) -> Self::IntoIter {
        self.data.iter_mut()
    }
}

impl<'a> IntoRegIter<'a> for &'a RegI64 {
    type IntoIter = <&'a RegI64 as IntoIterator>::IntoIter;
    fn into_reg_iter(self) -> Self::IntoIter {
        self.into_iter()
    }
}

impl<'a> IntoRegIterMut<'a> for &'a mut RegI64 {
    type IntoIter = <&'a mut RegI64 as IntoIterator>::IntoIter;
    fn into_reg_iter_mut(self) -> Self::IntoIter {
        self.into_iter()
    }
}

/* RegisterData */

impl RegisterData for RegI64 {
    fn register_span(&self) -> usize {
        4
    }
}

/* ReadRegisterData */

impl ReadRegisterData for RegI64 {
    fn read(&self, offset: usize, buf: &mut [u16]) -> RegisterResult<usize> {
        if offset >= self.register_span() {
            return Err(RegisterError::OutOfBounds { offset });
        }

        let available = self.register_span() - offset;
        let to_read = buf.len().min(available);

        buf[..to_read].copy_from_slice(&self.data[offset..offset + to_read]);
        Ok(to_read)
    }
}

/* WriteRegisterData */

impl WriteRegisterData for RegI64 {
    fn write(&mut self, offset: usize, buf: &[u16]) -> RegisterResult<usize> {
        if offset >= self.register_span() {
            return Err(RegisterError::OutOfBounds { offset });
        }

        let available = self.register_span() - offset;
        let to_write = buf.len().min(available);

        self.data[offset..offset + to_write].copy_from_slice(&buf[..to_write]);
        Ok(to_write)
    }
}

#[cfg(test)]
mod reg_i64_tests {
    use crate::register::{IntoRegIter, IntoRegIterMut};
    use super::*;

    #[test]
    fn reg_iter_001() {
        let reg0 = RegI64::new(-1);
        let reg1 = RegI64::new(-2);

        let iter0 = reg0.into_reg_iter();
        let iter1 = reg1.into_reg_iter();

        let mut iter = iter0.chain(iter1);
        assert_eq!(Some(&0xFFFF), iter.next());
        assert_eq!(Some(&0xFFFF), iter.next());
        assert_eq!(Some(&0xFFFF), iter.next());
        assert_eq!(Some(&0xFFFF), iter.next());
        assert_eq!(Some(&0xFFFF), iter.next());
        assert_eq!(Some(&0xFFFF), iter.next());
        assert_eq!(Some(&0xFFFF), iter.next());
        assert_eq!(Some(&0xFFFE), iter.next());
        assert_eq!(None, iter.next());
    }

    #[test]
    fn reg_iter_mut_001() {
        let mut reg0 = RegI64::new(-1);
        let mut reg1 = RegI64::new(-2);

        let iter0 = reg0.into_reg_iter();
        let iter1 = reg1.into_reg_iter();

        // check values equal initialization
        let mut iter = iter0.chain(iter1);
        assert_eq!(Some(&0xFFFF), iter.next());
        assert_eq!(Some(&0xFFFF), iter.next());
        assert_eq!(Some(&0xFFFF), iter.next());
        assert_eq!(Some(&0xFFFF), iter.next());
        assert_eq!(Some(&0xFFFF), iter.next());
        assert_eq!(Some(&0xFFFF), iter.next());
        assert_eq!(Some(&0xFFFF), iter.next());
        assert_eq!(Some(&0xFFFE), iter.next());
        assert_eq!(None, iter.next());

        // Change values
        let iter0 = reg0.into_reg_iter_mut();
        let iter1 = reg1.into_reg_iter_mut();
        
        let mut iter = iter0.chain(iter1);
        if let Some(reff) = iter.next() {
            *reff = 0x00FF;
        }

        if let Some(reff) = iter.next() {
            *reff = 0xEEDD;
        }

        if let Some(reff) = iter.next() {
            *reff = 0xCCBB;
        }

        if let Some(reff) = iter.next() {
            *reff = 0xAA99;
        }

        if let Some(reff) = iter.next() {
            *reff = 0x8877;
        }

        if let Some(reff) = iter.next() {
            *reff = 0x6655;
        }

        if let Some(reff) = iter.next() {
            *reff = 0x4433;
        }

        if let Some(reff) = iter.next() {
            *reff = 0x2211;
        }

        // check for change
        let iter0 = reg0.into_reg_iter();
        let iter1 = reg1.into_reg_iter();
        
        let mut iter = iter0.chain(iter1);
        assert_eq!(Some(&0x00FF), iter.next());
        assert_eq!(Some(&0xEEDD), iter.next());
        assert_eq!(Some(&0xCCBB), iter.next());
        assert_eq!(Some(&0xAA99), iter.next());
        assert_eq!(Some(&0x8877), iter.next());
        assert_eq!(Some(&0x6655), iter.next());
        assert_eq!(Some(&0x4433), iter.next());
        assert_eq!(Some(&0x2211), iter.next());
        assert_eq!(None, iter.next());
    }

    #[test]
    fn reg_iter_mut_002() {
        let mut reg0 = RegI64::new(0x1122_3344_5566_7788);

        // check values equal initialization
        let mut iter = reg0.into_reg_iter();
        assert_eq!(Some(&0x1122), iter.next());
        assert_eq!(Some(&0x3344), iter.next());
        assert_eq!(Some(&0x5566), iter.next());
        assert_eq!(Some(&0x7788), iter.next());
        assert_eq!(None, iter.next());

        // change values
        let mut iter = reg0.into_reg_iter_mut();
        if let Some(reff) = iter.next() {
            *reff = 0xFFFF;
        }

        if let Some(reff) = iter.next() {
            *reff = 0xFFFF;
        }

        if let Some(reff) = iter.next() {
            *reff = 0xFFFF;
        }

        if let Some(reff) = iter.next() {
            *reff = 0xFFFF;
        }

        // check values equal initialization
        let mut iter = reg0.into_reg_iter();
        assert_eq!(Some(&0xFFFF), iter.next());
        assert_eq!(Some(&0xFFFF), iter.next());
        assert_eq!(Some(&0xFFFF), iter.next());
        assert_eq!(Some(&0xFFFF), iter.next());
        assert_eq!(None, iter.next());

        // check conversion
        let value = i64::from(reg0);
        assert_eq!(value, -1);
    }

    #[test]
    fn from_to_i64_001() {
        let reg = RegI64::new(-1);
        let value = i64::from(reg);
        assert_eq!(value, -1);
    }
}