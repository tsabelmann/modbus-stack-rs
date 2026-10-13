use core::{
    fmt::Debug,
    ops::{Deref, DerefMut},
};

use super::{ReadRegisterData, RegisterData, RegisterError, RegisterResult, WriteRegisterData};
use crate::Registers;

/// Places a register (model) at a fixed start address.
///
/// Addresses below `start` are not accessible.
///
/// # Example
///
/// ```
/// use modbus_stack::register::{At, ReadRegisterData, RegU32};
///
/// let reg = At::new(40000, RegU32::new(0x11223344));
/// let mut buf = [0u16; 2];
/// reg.read(40000, &mut buf).unwrap();
/// assert_eq!(buf, [0x1122, 0x3344]);
/// ```
pub struct At<T> {
    start: usize,
    inner: T,
}

impl<T> At<T> {
    pub const fn new(start: u16, inner: T) -> At<T> {
        At {
            start: start as usize,
            inner,
        }
    }

    /// Returns the start address.
    pub const fn start(&self) -> usize {
        self.start
    }

    /// Returns a reference to the wrapped register.
    pub const fn get(&self) -> &T {
        &self.inner
    }

    /// Returns a mutable reference to the wrapped register.
    pub fn get_mut(&mut self) -> &mut T {
        &mut self.inner
    }

    /// Returns the wrapped register.
    pub fn into_inner(self) -> T {
        self.inner
    }

    /// Translates an absolute `address` into an offset of the wrapped register.
    fn offset(&self, address: usize) -> RegisterResult<usize> {
        address
            .checked_sub(self.start)
            .ok_or(RegisterError::OutOfBounds { offset: address })
    }
}

impl<T> DerefMut for At<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl<T> Deref for At<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<T: PartialEq> PartialEq for At<T> {
    fn eq(&self, other: &Self) -> bool {
        self.start == other.start && self.inner == other.inner
    }
}

impl<T: Eq> Eq for At<T> {}

impl<T: Clone> Clone for At<T> {
    fn clone(&self) -> Self {
        Self {
            start: self.start.clone(),
            inner: self.inner.clone(),
        }
    }
}

impl<T: Debug> Debug for At<T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("At")
            .field("start", &self.start)
            .field("inner", &self.inner)
            .finish()
    }
}

impl<T: RegisterData> RegisterData for At<T> {
    /// `start` plus the span of the wrapped register; `[0, start)` is an inaccessible hole.
    fn register_span(&self) -> usize {
        self.start + self.inner.register_span()
    }
}

impl<T: ReadRegisterData> ReadRegisterData for At<T> {
    fn read(&self, address: usize, buf: &mut [u16]) -> RegisterResult<()> {
        self.inner.read(self.offset(address)?, buf)
    }
}

impl<T: WriteRegisterData> WriteRegisterData for At<T> {
    fn write(&mut self, address: usize, values: Registers<'_>) -> RegisterResult<()> {
        self.inner.write(self.offset(address)?, values)
    }
}

#[cfg(test)]
mod at_tests {
    use super::*;
    use crate::register::RegU32;

    #[test]
    fn read_slice_001() {
        let data = [1u16, 2, 3, 4];
        let reg = At::new(100, &data[..]);

        let mut buf = [0u16; 4];
        assert_eq!(reg.read(100, &mut buf), Ok(()));
        assert_eq!(buf, [1, 2, 3, 4]);

        let mut buf = [0u16; 2];
        assert_eq!(reg.read(102, &mut buf), Ok(()));
        assert_eq!(buf, [3, 4]);

        let mut buf = [0u16; 1];
        assert_eq!(
            reg.read(99, &mut buf),
            Err(RegisterError::OutOfBounds { offset: 99 })
        );

        let mut buf = [0u16; 4];
        assert_eq!(
            reg.read(98, &mut buf),
            Err(RegisterError::OutOfBounds { offset: 98 })
        );

        let mut buf = [0u16; 2];
        assert_eq!(
            reg.read(103, &mut buf),
            Err(RegisterError::OutOfBounds { offset: 3 })
        );
    }

    #[test]
    fn read_whole_register_001() {
        let reg = At::new(100, RegU32::new(0x11223344));

        let mut buf = [0u16; 2];
        assert_eq!(reg.read(100, &mut buf), Ok(()));
        assert_eq!(buf, [0x1122, 0x3344]);

        let mut buf = [0u16; 1];
        assert_eq!(reg.read(101, &mut buf), Err(RegisterError::PartialAccess));
    }

    #[test]
    fn write_slice_001() {
        let mut data = [1u16, 2, 3, 4];
        let mut reg = At::new(100, &mut data[..]);

        assert_eq!(reg.write(100, Registers::from(&[10, 20][..])), Ok(()));
        assert_eq!(reg.get(), &[10, 20, 3, 4]);

        assert_eq!(
            reg.write(99, Registers::from(&[0][..])),
            Err(RegisterError::OutOfBounds { offset: 99 })
        );
        assert_eq!(reg.get(), &[10, 20, 3, 4]);

        reg.get_mut()[3] = 40;
        assert_eq!(reg.into_inner(), &[10, 20, 3, 40]);
    }

    #[test]
    fn register_span_001() {
        let reg = At::new(100, RegU32::new(0));
        assert_eq!(reg.start(), 100);
        assert_eq!(reg.register_span(), 102);
    }
}
