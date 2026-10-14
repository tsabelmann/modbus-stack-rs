use super::{
    ReadRegisterData, RegisterData, RegisterResult, WriteRegisterData, read_words, write_words,
};
use crate::Registers;
use core::ops::{Index, IndexMut};

/// Modbus register that can be converter to and from [u16; N].
#[derive(Debug, PartialEq, Clone)]
pub struct RegU16Array<const N: usize> {
    data: [u16; N],
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

/* RegisterData */

impl<const N: usize> RegisterData for RegU16Array<N> {
    #[inline]
    fn register_span(&self) -> usize {
        N
    }
}

/* ReadRegisterData */

impl<const N: usize> ReadRegisterData for RegU16Array<N> {
    #[inline]
    fn read(&self, offset: usize, buf: &mut [u16]) -> RegisterResult<()> {
        read_words(&self.data, offset, buf)
    }
}

/* WriteRegisterData */

impl<const N: usize> WriteRegisterData for RegU16Array<N> {
    #[inline]
    fn write(&mut self, offset: usize, values: Registers<'_>) -> RegisterResult<()> {
        write_words(&mut self.data, offset, values)
    }
}

#[cfg(test)]
mod reg_u16_array_tests {
    use super::*;

    #[test]
    fn read_001() {
        let reg = RegU16Array::new([0x1234, 0x5678]);
        let mut buf = [0u16; 2];
        assert_eq!(reg.read(0, &mut buf), Ok(()));
        assert_eq!(buf, [0x1234, 0x5678]);
    }
}
