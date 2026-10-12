use super::{IntoRegIter, IntoRegIterMut};
use core::{ops::{Index, IndexMut}};
use super::{RegisterData, ReadRegisterData, WriteRegisterData, RegisterResult, RegisterError};

/// Modbus register that can be converter to and from [u16; N].
#[derive(Debug, PartialEq, Clone)]
pub struct RegU16Array<const N: usize> {
    data: [u16; N]
}

impl<const N: usize> RegU16Array<N> {
    /// Creates a new [[u16; N]] based modbus register.
    /// 
    /// # Example
    ///
    /// ```
    /// use modbus_stack::register::RegF32;
    /// 
    /// let reg = RegF32::new(42.0);
    /// ````
    pub const fn new(value: [u16; N]) -> RegU16Array<N> {
        RegU16Array { data: value }
    }

    pub const fn from_str(string: &str) -> RegU16Array<N> {
        let mut data = [0u16; N];
        let slice = string.as_bytes();
        let slice_len = slice.len();

        let mut idx = 0;
        let mut slice_idx = 0;

        while idx < N {
            let hbyte = if slice_idx < slice_len {
                // save current slice index
                let i = slice_idx;
                
                // increment slice index
                slice_idx += 1;

                // slice data
                slice[i]
            } else {
                // padded data
                b'\0'
            };

            let lbyte = if slice_idx < slice_len {
                // save current slice index
                let i = slice_idx;
                
                // increment slice index
                slice_idx += 1;

                // slice data
                slice[i]
            } else {
                // padded data
                b'\0'
            };
            // write data
            data[idx] = u16::from_be_bytes([hbyte, lbyte]);

            // increment idx
            idx += 1;
        }

        RegU16Array { data }
    }

}

impl<const N: usize> Default for RegU16Array<N> {
    fn default() -> Self {
        RegU16Array { data: [0u16; N] }
    }
}

impl<const N: usize> From<&str> for RegU16Array<N> {
    fn from(value: &str) -> Self {
        RegU16Array::from_str(value)
    }
}

impl<const N: usize> From<[u16; N]> for RegU16Array<N> {
    fn from(value: [u16; N]) -> Self {
        RegU16Array::new(value)
    }
}

impl<const N: usize> From<RegU16Array<N>> for [u16; N] {
    fn from(value: RegU16Array<N>) -> Self {
        value.data
    }
}

impl<const N: usize> From<&RegU16Array<N>> for [u16; N] {
    fn from(value: &RegU16Array<N>) -> Self {
        value.data
    }
}

impl<const N: usize> AsRef<[u16]> for RegU16Array<N> {
    fn as_ref(&self) -> &[u16] {
        &self.data
    }
}

impl<const N: usize> Index<usize> for RegU16Array<N> {
    type Output = u16;
    fn index(&self, index: usize) -> &Self::Output {
        &self.data[index]
    }
}

impl<const N: usize> IndexMut<usize> for RegU16Array<N> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.data[index]
    }
}

impl<const N: usize> IntoIterator for RegU16Array<N> {
    type Item = u16;
    type IntoIter = core::array::IntoIter<u16, N>;
    fn into_iter(self) -> Self::IntoIter {
        self.data.into_iter()
    }
}

impl<'a, const N: usize> IntoIterator for &'a RegU16Array<N> {
    type Item = &'a u16;
    type IntoIter = core::slice::Iter<'a, u16>;
    fn into_iter(self) -> Self::IntoIter {
        (&self.data).into_iter()
    }
}

impl<'a, const N: usize> IntoIterator for &'a mut RegU16Array<N> {
    type Item = &'a mut u16;
    type IntoIter = core::slice::IterMut<'a, u16>;
    fn into_iter(self) -> Self::IntoIter {
        (&mut self.data).into_iter()
    }
}

impl<'a, const N: usize> IntoRegIter<'a> for &'a RegU16Array<N>  {
    type IntoIter = <&'a RegU16Array<N> as IntoIterator>::IntoIter;
    fn into_reg_iter(self) -> Self::IntoIter {
        self.into_iter()
    }
}

impl<'a, const N: usize> IntoRegIterMut<'a> for &'a mut RegU16Array<N>  {
    type IntoIter = <&'a mut RegU16Array<N> as IntoIterator>::IntoIter;
    fn into_reg_iter_mut(self) -> Self::IntoIter {
        self.into_iter()
    }
}

/* RegisterData */

impl<const N: usize> RegisterData for RegU16Array<N> {
    fn register_span(&self) -> usize {
        N
    }
}

/* ReadRegisterData */

impl<const N: usize> ReadRegisterData for RegU16Array<N> {
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

impl<const N: usize> WriteRegisterData for RegU16Array<N> {
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
mod reg_u16_array_tests {
    use super::*;

    #[test]
    fn as_ref_001() {
        let reg = RegU16Array::new([0x1234, 0x5678]);
        assert_eq!(reg.as_ref(), &[0x1234, 0x5678]);
    }
}
