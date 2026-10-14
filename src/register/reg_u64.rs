use super::{IntoRegIter, IntoRegIterMut};
use super::{RegisterData, ReadRegisterData, WriteRegisterData, RegisterResult, read_whole, write_whole};
use crate::Registers;

/// Modbus register that can be converter to and from [u64].
#[derive(Debug, PartialEq, Default, Clone)]
pub struct RegU64 {
    data: [u16; 4]
}

impl RegU64 {
    /// Creates a new [u64] based modbus register.
    /// 
    /// # Example
    ///
    /// ```
    /// use modbus_stack::register::RegU64;
    /// 
    /// let reg = RegU64::new(0x1122334455667788);
    /// ````
    pub const fn new(value: u64) -> RegU64 {
        let data = [
            ((value >> 48) & 0xFFFF) as u16,
            ((value >> 32) & 0xFFFF) as u16,
            ((value >> 16) & 0xFFFF) as u16,
            (value & 0xFFFF) as u16,
        ];

        RegU64 {
            data
        }
    }
}

impl From<u64> for RegU64 {
    fn from(value: u64) -> Self {
        RegU64::new(value)
    }
}

impl From<&u64> for RegU64 {
    fn from(value: &u64) -> Self {
        RegU64::new(*value)
    }
}

impl From<RegU64> for u64 {
    fn from(value: RegU64) -> Self {
        u64::from(&value)
    }
}

impl From<&RegU64> for u64 {
    fn from(value: &RegU64) -> Self {
        ((value.data[0] as u64) << 48) | ((value.data[1] as u64) << 32) | ((value.data[2] as u64) << 16) | (value.data[3] as u64)
    }
}

impl IntoIterator for RegU64 {
    type Item = u16;
    type IntoIter = core::array::IntoIter<u16, 4>;
    fn into_iter(self) -> Self::IntoIter {
        self.data.into_iter()
    }
}

impl<'a> IntoIterator for &'a RegU64 {
    type Item = &'a u16;
    type IntoIter = core::slice::Iter<'a, u16>;
    fn into_iter(self) -> Self::IntoIter {
        self.data.iter()
    }
}

impl<'a> IntoIterator for &'a mut RegU64 {
    type Item = &'a mut u16;
    type IntoIter = core::slice::IterMut<'a, u16>;
    fn into_iter(self) -> Self::IntoIter {
        self.data.iter_mut()
    }
}

impl<'a> IntoRegIter<'a> for &'a RegU64 {
    type IntoIter = <&'a RegU64 as IntoIterator>::IntoIter;
    fn into_reg_iter(self) -> Self::IntoIter {
        self.into_iter()
    }
}

impl<'a> IntoRegIterMut<'a> for &'a mut RegU64 {
    type IntoIter = <&'a mut RegU64 as IntoIterator>::IntoIter;
    fn into_reg_iter_mut(self) -> Self::IntoIter {
        self.into_iter()
    }
}

/* Read/Write Register */

/* RegisterData */

impl RegisterData for RegU64 {
    #[inline]
    fn register_span(&self) -> usize {
        4
    }
}

/* ReadRegisterData */

impl ReadRegisterData for RegU64 {
    #[inline]
    fn read(&self, offset: usize, buf: &mut [u16]) -> RegisterResult<()> {
        read_whole(&self.data, offset, buf)
    }
}

/* WriteRegisterData */

impl WriteRegisterData for RegU64 {
    #[inline]
    fn write(&mut self, offset: usize, values: Registers<'_>) -> RegisterResult<()> {
        write_whole(&mut self.data, offset, values)
    }
}

#[cfg(test)]
mod reg_u64_tests {
    use crate::register::{IntoRegIter, IntoRegIterMut};
    use super::*;

    #[test]
    fn reg_iter_001() {
        let reg0 = RegU64::new(0x1122334455667788);
        let reg1 = RegU64::new(0x8877665544332211);

        let iter0 = reg0.into_reg_iter();
        let iter1 = reg1.into_reg_iter();

        let mut iter = iter0.chain(iter1);
        assert_eq!(Some(&0x1122), iter.next());
        assert_eq!(Some(&0x3344), iter.next());
        assert_eq!(Some(&0x5566), iter.next());
        assert_eq!(Some(&0x7788), iter.next());
        assert_eq!(Some(&0x8877), iter.next());
        assert_eq!(Some(&0x6655), iter.next());
        assert_eq!(Some(&0x4433), iter.next());
        assert_eq!(Some(&0x2211), iter.next());
        assert_eq!(None, iter.next());
    }

    #[test]
    fn reg_iter_mut_001() {
        let mut reg0 = RegU64::new(0x11223344);
        let mut reg1 = RegU64::new(0x55667788);

        let iter0 = reg0.into_reg_iter();
        let iter1 = reg1.into_reg_iter();

        // check values equal initialization
        let mut iter = iter0.chain(iter1);
        assert_eq!(Some(&0x0000), iter.next());
        assert_eq!(Some(&0x0000), iter.next());
        assert_eq!(Some(&0x1122), iter.next());
        assert_eq!(Some(&0x3344), iter.next());
        assert_eq!(Some(&0x0000), iter.next());
        assert_eq!(Some(&0x0000), iter.next());
        assert_eq!(Some(&0x5566), iter.next());
        assert_eq!(Some(&0x7788), iter.next());
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
    fn from_to_u32_001() {
        let reg = RegU64::new(0x1122_3344);
        let value = u64::from(reg);
        assert_eq!(value, 0x0000_0000_1122_3344);
    }
}