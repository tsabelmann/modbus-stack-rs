use super::{IntoRegIter, IntoRegIterMut};
use super::{RegisterData, ReadRegisterData, WriteRegisterData, RegisterResult, read_whole, write_whole};
use crate::Registers;

/// Modbus register that can be converter to and from [f64].
#[derive(Debug, PartialEq, Default, Clone)]
pub struct RegF64 {
    data: [u16; 4]
}

impl RegF64 {
    /// Creates a new [f64] based modbus register.
    /// 
    /// # Example
    ///
    /// ```
    /// use modbus_stack::register::RegF64;
    /// 
    /// let reg = RegF64::new(42.0);
    /// ````
    pub const fn new(value: f64) -> RegF64 {
        let data = f64::to_be_bytes(value);
        let data = [
            ((data[0] as u16) << 8) | (data[1] as u16),
            ((data[2] as u16) << 8) | (data[3] as u16),
            ((data[4] as u16) << 8) | (data[5] as u16),
            ((data[6] as u16) << 8) | (data[7] as u16),
        ];
        
        RegF64 {
            data
        }
    }
}

impl From<f64> for RegF64 {
    fn from(value: f64) -> Self {
        RegF64::new(value)
    }
}

impl From<&f64> for RegF64 {
    fn from(value: &f64) -> Self {
        RegF64::new(*value)
    }
}

impl From<RegF64> for f64 {
    fn from(value: RegF64) -> Self {
        f64::from(&value)
    }
}

impl From<&RegF64> for f64 {
    fn from(value: &RegF64) -> Self {
        let mut data = [0u8; 8];
        data[0] = ((value.data[0] >> 8) & 0xFF) as u8;
        data[1] = (value.data[0] & 0xFF) as u8;
        data[2] = ((value.data[1] >> 8) & 0xFF) as u8;
        data[3] = (value.data[1] & 0xFF) as u8;
        data[4] = ((value.data[2] >> 8) & 0xFF) as u8;
        data[5] = (value.data[2] & 0xFF) as u8;
        data[6] = ((value.data[3] >> 8) & 0xFF) as u8;
        data[7] = (value.data[3] & 0xFF) as u8;
        f64::from_be_bytes(data)
    }
}

impl IntoIterator for RegF64 {
    type Item = u16;
    type IntoIter = core::array::IntoIter<u16, 4>;
    fn into_iter(self) -> Self::IntoIter {
        self.data.into_iter()
    }
}

impl<'a> IntoIterator for &'a RegF64 {
    type Item = &'a u16;
    type IntoIter = core::slice::Iter<'a, u16>;
    fn into_iter(self) -> Self::IntoIter {
        self.data.iter()
    }
}

impl<'a> IntoIterator for &'a mut RegF64 {
    type Item = &'a mut u16;
    type IntoIter = core::slice::IterMut<'a, u16>;
    fn into_iter(self) -> Self::IntoIter {
        self.data.iter_mut()
    }
}

impl<'a> IntoRegIter<'a> for &'a RegF64 {
    type IntoIter = <&'a RegF64 as IntoIterator>::IntoIter;
    fn into_reg_iter(self) -> Self::IntoIter {
        self.into_iter()
    }
}

impl<'a> IntoRegIterMut<'a> for &'a mut RegF64 {
    type IntoIter = <&'a mut RegF64 as IntoIterator>::IntoIter;
    fn into_reg_iter_mut(self) -> Self::IntoIter {
        self.into_iter()
    }
}

/* RegisterData */

impl RegisterData for RegF64 {
    fn register_span(&self) -> usize {
        4
    }
}

/* ReadRegisterData */

impl ReadRegisterData for RegF64 {
    fn read(&self, offset: usize, buf: &mut [u16]) -> RegisterResult<()> {
        read_whole(&self.data, offset, buf)
    }
}

/* WriteRegisterData */

impl WriteRegisterData for RegF64 {
    fn write(&mut self, offset: usize, values: Registers<'_>) -> RegisterResult<()> {
        write_whole(&mut self.data, offset, values)
    }
}

#[cfg(test)]
mod reg_f64_tests {
    use crate::register::{IntoRegIter, IntoRegIterMut};
    use super::*;

    #[test]
    fn reg_iter_001() {
        let reg0 = RegF64::new(0.0);
        let mut iter = reg0.into_reg_iter();
        assert_eq!(Some(&0x0000), iter.next());
        assert_eq!(Some(&0x0000), iter.next());
        assert_eq!(Some(&0x0000), iter.next());
        assert_eq!(Some(&0x0000), iter.next());
        assert_eq!(None, iter.next());
    }

    #[test]
    fn reg_iter_mut_001() {
        let mut reg0 = RegF64::new(0.0);

        // check values equal initialization
        let mut iter = reg0.into_reg_iter();
        assert_eq!(Some(&0x0000), iter.next());
        assert_eq!(Some(&0x0000), iter.next());
        assert_eq!(Some(&0x0000), iter.next());
        assert_eq!(Some(&0x0000), iter.next());

        let data = {
            let data = 42.0f64.to_be_bytes();
            [
                ((data[0] as u16) << 8) | (data[1] as u16),
                ((data[2] as u16) << 8) | (data[3] as u16),
                ((data[4] as u16) << 8) | (data[5] as u16),
                ((data[6] as u16) << 8) | (data[7] as u16),
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

        if let Some(reff) = iter.next() {
            *reff = data[2];
        }

        if let Some(reff) = iter.next() {
            *reff = data[3];
        }

        // check for change
        let value = f64::from(reg0);
        assert_eq!(value, 42.0f64);
    }

    #[test]
    fn from_to_f64_001() {
        let reg = RegF64::new(42.0);
        let value = f64::from(reg);
        assert_eq!(value, 42.0);
    }
}