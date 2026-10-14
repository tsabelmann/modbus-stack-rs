use super::{IntoRegIter, IntoRegIterMut};
use super::{RegisterData, ReadRegisterData, WriteRegisterData, RegisterResult, read_whole, write_whole};
use crate::Registers;

/// Modbus register that can be converter to and from [f32].
#[derive(Debug, PartialEq, Default, Clone)]
pub struct RegF32 {
    data: [u16; 2]
}

impl RegF32 {
    /// Creates a new [f32] based modbus register.
    /// 
    /// # Example
    ///
    /// ```
    /// use modbus_stack::register::RegF32;
    /// 
    /// let reg = RegF32::new(42.0);
    /// ````
    pub const fn new(value: f32) -> RegF32 {
        let data = f32::to_be_bytes(value);
        let data = [
            ((data[0] as u16) << 8) | (data[1] as u16),
            ((data[2] as u16) << 8) | (data[3] as u16),
        ];
        
        RegF32 {
            data
        }
    }
}

impl From<f32> for RegF32 {
    fn from(value: f32) -> Self {
        RegF32::new(value)
    }
}

impl From<&f32> for RegF32 {
    fn from(value: &f32) -> Self {
        RegF32::new(*value)
    }
}

impl From<RegF32> for f32 {
    fn from(value: RegF32) -> Self {
        f32::from(&value)
    }
}

impl From<&RegF32> for f32 {
    fn from(value: &RegF32) -> Self {
        let mut data = [0u8; 4];
        data[0] = ((value.data[0] >> 8) & 0xFF) as u8;
        data[1] = (value.data[0] & 0xFF) as u8;
        data[2] = ((value.data[1] >> 8) & 0xFF) as u8;
        data[3] = (value.data[1] & 0xFF) as u8;
        f32::from_be_bytes(data)
    }
}

impl IntoIterator for RegF32 {
    type Item = u16;
    type IntoIter = core::array::IntoIter<u16, 2>;
    fn into_iter(self) -> Self::IntoIter {
        self.data.into_iter()
    }
}

impl<'a> IntoIterator for &'a RegF32 {
    type Item = &'a u16;
    type IntoIter = core::slice::Iter<'a, u16>;
    fn into_iter(self) -> Self::IntoIter {
        self.data.iter()
    }
}

impl<'a> IntoIterator for &'a mut RegF32 {
    type Item = &'a mut u16;
    type IntoIter = core::slice::IterMut<'a, u16>;
    fn into_iter(self) -> Self::IntoIter {
        self.data.iter_mut()
    }
}

impl<'a> IntoRegIter<'a> for &'a RegF32 {
    type IntoIter = <&'a RegF32 as IntoIterator>::IntoIter;
    fn into_reg_iter(self) -> Self::IntoIter {
        self.into_iter()
    }
}

impl<'a> IntoRegIterMut<'a> for &'a mut RegF32 {
    type IntoIter = <&'a mut RegF32 as IntoIterator>::IntoIter;
    fn into_reg_iter_mut(self) -> Self::IntoIter {
        self.into_iter()
    }
}

/* RegisterData */

impl RegisterData for RegF32 {
    #[inline]
    fn register_span(&self) -> usize {
        2
    }
}

/* ReadRegisterData */

impl ReadRegisterData for RegF32 {
    #[inline]
    fn read(&self, offset: usize, buf: &mut [u16]) -> RegisterResult<()> {
        read_whole(&self.data, offset, buf)
    }
}

/* WriteRegisterData */

impl WriteRegisterData for RegF32 {
    #[inline]
    fn write(&mut self, offset: usize, values: Registers<'_>) -> RegisterResult<()> {
        write_whole(&mut self.data, offset, values)
    }
}

#[cfg(test)]
mod reg_f32_tests {
    use crate::register::{IntoRegIter, IntoRegIterMut};
    use super::*;

    #[test]
    fn reg_iter_001() {
        let reg0 = RegF32::new(0.0);
        let mut iter = reg0.into_reg_iter();
        assert_eq!(Some(&0x0000), iter.next());
        assert_eq!(Some(&0x0000), iter.next());
        assert_eq!(None, iter.next());
    }

    #[test]
    fn reg_iter_mut_001() {
        let mut reg0 = RegF32::new(0.0);

        // check values equal initialization
        let mut iter = reg0.into_reg_iter();
        assert_eq!(Some(&0x0000), iter.next());
        assert_eq!(Some(&0x0000), iter.next());

        let data = {
            let data = 42.0f32.to_be_bytes();
            [
                ((data[0] as u16) << 8) | (data[1] as u16),
                ((data[2] as u16) << 8) | (data[3] as u16),
            ]
        };

        // change values
        let mut iter = reg0.into_reg_iter_mut();
        if let Some(reff) = iter.next() {
            *reff = data[0];
        }

        if let Some(reff) = iter.next() {
            *reff = data[1];
        }

        // check for change
        let value = f32::from(reg0);
        assert_eq!(value, 42.0f32);
    }

    #[test]
    fn from_to_f32_001() {
        let reg = RegF32::new(42.0);
        let value = f32::from(reg);
        assert_eq!(value, 42.0);
    }
}