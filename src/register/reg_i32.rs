use super::{IntoRegIter, IntoRegIterMut};
use super::{RegisterData, ReadRegisterData, WriteRegisterData, RegisterResult, RegisterError};

/// Modbus register that can be converter to and from [u8].
#[derive(Debug, PartialEq, Default, Clone)]
pub struct RegI32 {
    data: [u16; 2]
}

impl RegI32 {
    /// Creates a new [i32] based modbus register.
    /// 
    /// # Example
    ///
    /// ```
    /// use modbus_stack::register::RegI32;
    /// 
    /// let reg = RegI32::new(42);
    /// ````
    pub const fn new(value: i32) -> RegI32 {
        RegI32 {
            data: [((value >> 16) & 0xFFFF) as u16, (value & 0xFFFF) as u16]
        }
    }
}

impl From<i32> for RegI32 {
    fn from(value: i32) -> Self {
        RegI32::new(value)
    }
}

impl From<&i32> for RegI32 {
    fn from(value: &i32) -> Self {
        RegI32::new(*value)
    }
}

impl From<RegI32> for i32 {
    fn from(value: RegI32) -> Self {
        i32::from(&value)
    }
}

impl From<&RegI32> for i32 {
    fn from(value: &RegI32) -> Self {
        ((value.data[0] as i32) << 16) | (value.data[1] as i32)
    }
}

impl IntoIterator for RegI32 {
    type Item = u16;
    type IntoIter = core::array::IntoIter<u16, 2>;
    fn into_iter(self) -> Self::IntoIter {
        self.data.into_iter()
    }
}

impl<'a> IntoIterator for &'a RegI32 {
    type Item = &'a u16;
    type IntoIter = core::slice::Iter<'a, u16>;
    fn into_iter(self) -> Self::IntoIter {
        (&self.data).into_iter()
    }
}

impl<'a> IntoIterator for &'a mut RegI32 {
    type Item = &'a mut u16;
    type IntoIter = core::slice::IterMut<'a, u16>;
    fn into_iter(self) -> Self::IntoIter {
        (&mut self.data).into_iter()
    }
}

impl<'a> IntoRegIter<'a> for &'a RegI32 {
    type IntoIter = <&'a RegI32 as IntoIterator>::IntoIter;
    fn into_reg_iter(self) -> Self::IntoIter {
        self.into_iter()
    }
}

impl<'a> IntoRegIterMut<'a> for &'a mut RegI32 {
    type IntoIter = <&'a mut RegI32 as IntoIterator>::IntoIter;
    fn into_reg_iter_mut(self) -> Self::IntoIter {
        self.into_iter()
    }
}

/* RegisterData */

impl RegisterData for RegI32 {
    fn register_span(&self) -> usize {
        2
    }
}

/* ReadRegisterData */

impl ReadRegisterData for RegI32 {
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

impl WriteRegisterData for RegI32 {
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
mod reg_i32_tests {
    use crate::register::{IntoRegIter, IntoRegIterMut};
    use super::*;

    #[test]
    fn reg_iter_001() {
        let reg0 = RegI32::new(-1);
        let reg1 = RegI32::new(-2);

        let iter0 = reg0.into_reg_iter();
        let iter1 = reg1.into_reg_iter();

        let mut iter = iter0.chain(iter1);
        assert_eq!(Some(&0xFFFF), iter.next());
        assert_eq!(Some(&0xFFFF), iter.next());
        assert_eq!(Some(&0xFFFF), iter.next());
        assert_eq!(Some(&0xFFFE), iter.next());
        assert_eq!(None, iter.next());
    }

    #[test]
    fn reg_iter_mut_001() {
        let mut reg0 = RegI32::new(-1);
        let mut reg1 = RegI32::new(-2);
        let mut reg2 = RegI32::new(-3);
        let mut reg3 = RegI32::new(-4);

        let iter0 = reg0.into_reg_iter();
        let iter1 = reg1.into_reg_iter();
        let iter2 = reg2.into_reg_iter();
        let iter3 = reg3.into_reg_iter();

        // check values equal initialization
        let mut iter = iter0.chain(iter1).chain(iter2).chain(iter3);
        assert_eq!(Some(&0xFFFF), iter.next());
        assert_eq!(Some(&0xFFFF), iter.next());
        assert_eq!(Some(&0xFFFF), iter.next());
        assert_eq!(Some(&0xFFFE), iter.next());
        assert_eq!(Some(&0xFFFF), iter.next());
        assert_eq!(Some(&0xFFFD), iter.next());
        assert_eq!(Some(&0xFFFF), iter.next());
        assert_eq!(Some(&0xFFFC), iter.next());
        assert_eq!(None, iter.next());

        // Change values
        let iter0 = reg0.into_reg_iter_mut();
        let iter1 = reg1.into_reg_iter_mut();
        let iter2 = reg2.into_reg_iter_mut();
        let iter3 = reg3.into_reg_iter_mut();
        
        let mut iter = iter0.chain(iter1).chain(iter2).chain(iter3);
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
        let iter2 = reg2.into_reg_iter();
        let iter3 = reg3.into_reg_iter();
        
        let mut iter = iter0.chain(iter1).chain(iter2).chain(iter3);
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
    fn from_to_i32_001() {
        let reg = RegI32::new(-1);
        let value = i32::from(reg);
        assert_eq!(value, -1);
    }
}