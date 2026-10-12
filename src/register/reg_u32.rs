use super::{IntoRegIter, IntoRegIterMut};
use super::{RegisterData, ReadRegisterData, WriteRegisterData, RegisterResult, RegisterError, read_whole};

/// Modbus register that can be converter to and from [u32].
#[derive(Debug, PartialEq, Default, Clone)]
pub struct RegU32 {
    data: [u16; 2]
}

impl RegU32 {
    /// Creates a new [u32] based modbus register.
    /// 
    /// # Example
    ///
    /// ```
    /// use modbus_stack::register::RegU32;
    /// 
    /// let reg = RegU32::new(42);
    /// ````
    pub const fn new(value: u32) -> RegU32 {
        RegU32 {
            data: [((value >> 16) & 0xFFFF) as u16, (value & 0xFFFF) as u16]
        }
    }
}

impl From<u32> for RegU32 {
    fn from(value: u32) -> Self {
        RegU32::new(value)
    }
}

impl From<&u32> for RegU32 {
    fn from(value: &u32) -> Self {
        RegU32::new(*value)
    }
}

impl From<RegU32> for u32 {
    fn from(value: RegU32) -> Self {
        u32::from(&value)
    }
}

impl From<&RegU32> for u32 {
    fn from(value: &RegU32) -> Self {
        ((value.data[0] as u32) << 16) | (value.data[1] as u32)
    }
}

impl IntoIterator for RegU32 {
    type Item = u16;
    type IntoIter = core::array::IntoIter<u16, 2>;
    fn into_iter(self) -> Self::IntoIter {
        self.data.into_iter()
    }
}

impl<'a> IntoIterator for &'a RegU32 {
    type Item = &'a u16;
    type IntoIter = core::slice::Iter<'a, u16>;
    fn into_iter(self) -> Self::IntoIter {
        self.data.iter()
    }
}

impl<'a> IntoIterator for &'a mut RegU32 {
    type Item = &'a mut u16;
    type IntoIter = core::slice::IterMut<'a, u16>;
    fn into_iter(self) -> Self::IntoIter {
        self.data.iter_mut()
    }
}

impl<'a> IntoRegIter<'a> for &'a RegU32 {
    type IntoIter = <&'a RegU32 as IntoIterator>::IntoIter;
    fn into_reg_iter(self) -> Self::IntoIter {
        self.into_iter()
    }
}

impl<'a> IntoRegIterMut<'a> for &'a mut RegU32 {
    type IntoIter = <&'a mut RegU32 as IntoIterator>::IntoIter;
    fn into_reg_iter_mut(self) -> Self::IntoIter {
        self.into_iter()
    }
}

/* RegisterData */

impl RegisterData for RegU32 {
    fn register_span(&self) -> usize {
        2
    }
}

/* ReadRegisterData */

impl ReadRegisterData for RegU32 {
    fn read(&self, offset: usize, buf: &mut [u16]) -> RegisterResult<()> {
        read_whole(&self.data, offset, buf)
    }
}

/* WriteRegisterData */

impl WriteRegisterData for RegU32 {
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
mod reg_u32_tests {
    use crate::register::{IntoRegIter, IntoRegIterMut};
    use super::*;

    #[test]
    fn reg_iter_001() {
        let reg0 = RegU32::new(0x11223344);
        let reg1 = RegU32::new(0xAABBCCDD);

        let iter0 = reg0.into_reg_iter();
        let iter1 = reg1.into_reg_iter();

        let mut iter = iter0.chain(iter1);
        assert_eq!(Some(&0x1122), iter.next());
        assert_eq!(Some(&0x3344), iter.next());
        assert_eq!(Some(&0xAABB), iter.next());
        assert_eq!(Some(&0xCCDD), iter.next());
        assert_eq!(None, iter.next());
    }

    #[test]
    fn reg_iter_mut_001() {
        let mut reg0 = RegU32::new(0x11223344);
        let mut reg1 = RegU32::new(0x55667788);
        let mut reg2 = RegU32::new(0x99AABBCC);
        let mut reg3 = RegU32::new(0xDDEEFF00);

        let iter0 = reg0.into_reg_iter();
        let iter1 = reg1.into_reg_iter();
        let iter2 = reg2.into_reg_iter();
        let iter3 = reg3.into_reg_iter();

        // check values equal initialization
        let mut iter = iter0.chain(iter1).chain(iter2).chain(iter3);
        assert_eq!(Some(&0x1122), iter.next());
        assert_eq!(Some(&0x3344), iter.next());
        assert_eq!(Some(&0x5566), iter.next());
        assert_eq!(Some(&0x7788), iter.next());
        assert_eq!(Some(&0x99AA), iter.next());
        assert_eq!(Some(&0xBBCC), iter.next());
        assert_eq!(Some(&0xDDEE), iter.next());
        assert_eq!(Some(&0xFF00), iter.next());
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
    fn from_to_u32_001() {
        let reg = RegU32::new(0x11223344);
        let value = u32::from(reg);
        assert_eq!(value, 0x11223344);
    }
}