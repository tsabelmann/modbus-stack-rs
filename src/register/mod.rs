// pub mod atomic;

mod reg_bool;
pub use reg_bool::RegBool;

mod reg_u8;
pub use reg_u8::RegU8;

mod reg_u16;
pub use reg_u16::RegU16;

mod reg_u32;
pub use reg_u32::RegU32;

mod reg_u64;
pub use reg_u64::RegU64;

// signed integer based registers

mod reg_i8;
pub use reg_i8::RegI8;

mod reg_i16;
pub use reg_i16::RegI16;

mod reg_i32;
pub use reg_i32::RegI32;

mod reg_i64;
pub use reg_i64::RegI64;

// IEEE-754 floats

mod reg_f32;
pub use reg_f32::RegF32;

mod reg_f64;
pub use reg_f64::RegF64;

// Array

mod reg_u16_array;
pub use reg_u16_array::RegU16Array;

/// Trait for turning a reference into a immutable register iterator.
pub trait IntoRegIter<'a> {
    type IntoIter: Iterator<Item = &'a u16>;
    fn into_reg_iter(self) -> Self::IntoIter;
}

/// Trait for turning a reference into a mutable register iterator.
pub trait IntoRegIterMut<'a> {
    type IntoIter: Iterator<Item = &'a mut u16>;
    fn into_reg_iter_mut(self) -> Self::IntoIter;
}

// /// Auto implementation for [IntoRegIter] if it implements [IntoIterator<Item = &'a u16>].
// impl<'a, T: IntoIterator<Item = &'a u16>> IntoRegIter<'a> for T {
//     type IntoIter = T::IntoIter;
//     fn into_reg_iter(self) -> Self::IntoIter {
//         self.into_iter()
//     }
// }

// /// Auto implementation for [IntoRegIterMut] if it implements [IntoIterator<Item = &'a u16>].
// impl<'a, T: IntoIterator<Item = &'a mut u16>> IntoRegIterMut<'a> for T {
//     type IntoIter = T::IntoIter;
//     fn into_reg_iter_mut(self) -> Self::IntoIter {
//         self.into_iter()
//     }
// }

/// Mögliche Fehler bei Register-Operationen
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RegisterError {
    /// Offset out of bounds
    OutOfBounds { offset: usize },
    /// Register is not readable/writable
    AccessDenied,
    /// Read/Write would exceed the maximum allowed offset.
    /// Should **not** be implemented for basic registers.
    Overflow,
}

pub type RegisterResult<T> = Result<T, RegisterError>;

pub trait RegisterData {
    /// Retrieves the maximum register span, including holes
    /// that are not accessible.
    fn register_span(&self) -> usize;
}

pub trait ReadRegisterData {
    fn read(&self, offset: usize, buf: &mut [u16]) -> RegisterResult<usize>;
}

pub trait WriteRegisterData {
    fn write(&mut self, offset: usize, buf: &[u16]) -> RegisterResult<usize>;
}

impl RegisterData for &[u16] {
    fn register_span(&self) -> usize {
        self.len()
    }
}

impl ReadRegisterData for &[u16] {
    fn read(&self, offset: usize, buf: &mut [u16]) -> RegisterResult<usize> {
        if offset >= self.register_span() {
            return Err(RegisterError::OutOfBounds { offset });
        }

        if buf.is_empty() {
            return Ok(0);
        }

        let available = self.register_span() - offset;
        let to_read = buf.len().min(available);
        buf[..to_read].copy_from_slice(&self[offset..offset + to_read]);
        Ok(to_read)
    }
}

impl RegisterData for &mut [u16] {
    fn register_span(&self) -> usize {
        self.len()
    }
}

impl ReadRegisterData for &mut [u16] {
    fn read(&self, offset: usize, buf: &mut [u16]) -> RegisterResult<usize> {
        let slice: &[u16] = self;
        ReadRegisterData::read(&slice, offset, buf)
    }
}

impl WriteRegisterData for &mut [u16] {
    fn write(&mut self, offset: usize, buf: &[u16]) -> RegisterResult<usize> {
        if offset >= self.register_span() {
            return Err(RegisterError::OutOfBounds { offset });
        }

        let available = self.register_span() - offset;
        if buf.len() > available {
            return Err(RegisterError::OutOfBounds { offset });
        }

        self[offset..offset + buf.len()].copy_from_slice(buf);
        Ok(buf.len())
    }
}

/// Read-only register enforced by the type system.
#[derive(Debug)]
pub struct RO<T> {
    data: T,
}

impl<T> RO<T> {
    pub const fn new(value: T) -> RO<T> {
        RO { data: value }
    }

    /// Returns a reference to the wrapped register.
    pub const fn get(&self) -> &T {
        &self.data
    }

    pub fn modify<F, U>(&mut self, f: F) -> U
    where
        F: FnOnce(&mut T) -> U,
    {
        f(&mut self.data)
    }
}

impl<'a, T> IntoIterator for &'a RO<T>
where
    &'a T: IntoRegIter<'a>,
{
    type Item = &'a u16;
    type IntoIter = <&'a T as IntoRegIter<'a>>::IntoIter;
    fn into_iter(self) -> Self::IntoIter {
        (&self.data).into_reg_iter()
    }
}

// impl<T: IntoIterator<Item = u16>> IntoIterator for RO<T> {
//     type Item = T::Item;
//     type IntoIter = T::IntoIter;
//     fn into_iter(self) -> Self::IntoIter {
//         self.data.into_iter()
//     }
// }

// impl<'a, T> IntoIterator for &'a RO<T>
// where
//     &'a T : IntoIterator<Item = &'a u16>
// {
//     type Item = <&'a T as IntoIterator>::Item;
//     type IntoIter = <&'a T as IntoIterator>::IntoIter;
//     fn into_iter(self) -> Self::IntoIter {
//         (&self.data).into_iter()
//     }
// }

impl<T: Clone> Clone for RO<T> {
    fn clone(&self) -> Self {
        RO::new(self.data.clone())
    }
}

impl<T: Default> Default for RO<T> {
    fn default() -> Self {
        RO::new(T::default())
    }
}

impl<T: RegisterData> RegisterData for RO<T> {
    fn register_span(&self) -> usize {
        self.data.register_span()
    }
}

impl<T: ReadRegisterData> ReadRegisterData for RO<T> {
    fn read(&self, offset: usize, buf: &mut [u16]) -> RegisterResult<usize> {
        self.data.read(offset, buf)
    }
}

/// Read and writable register.
#[derive(Debug)]
pub struct RW<T> {
    data: T,
}

impl<T> RW<T> {
    pub const fn new(value: T) -> RW<T> {
        RW { data: value }
    }

    /// Returns a reference to the wrapped register.
    pub const fn get(&self) -> &T {
        &self.data
    }

    pub fn modify<F, U>(&mut self, f: F) -> U
    where
        F: FnOnce(&mut T) -> U,
    {
        f(&mut self.data)
    }
}

impl<'a, T> IntoIterator for &'a RW<T>
where
    &'a T: IntoRegIter<'a>,
{
    type Item = &'a u16;
    type IntoIter = <&'a T as IntoRegIter<'a>>::IntoIter;
    fn into_iter(self) -> Self::IntoIter {
        (&self.data).into_reg_iter()
    }
}

impl<'a, T> IntoIterator for &'a mut RW<T>
where
    &'a mut T: IntoRegIterMut<'a>,
{
    type Item = &'a mut u16;
    type IntoIter = <&'a mut T as IntoRegIterMut<'a>>::IntoIter;
    fn into_iter(self) -> Self::IntoIter {
        (&mut self.data).into_reg_iter_mut()
    }
}

// impl<T: IntoIterator<Item = u16>> IntoIterator for RW<T> {
//     type Item = T::Item;
//     type IntoIter = T::IntoIter;
//     fn into_iter(self) -> Self::IntoIter {
//         self.data.into_iter()
//     }
// }

// impl<'a, T> IntoIterator for &'a RW<T>
// where
//     &'a T : IntoIterator<Item = &'a u16>
// {
//     type Item = <&'a T as IntoIterator>::Item;
//     type IntoIter = <&'a T as IntoIterator>::IntoIter;
//     fn into_iter(self) -> Self::IntoIter {
//         (&self.data).into_iter()
//     }
// }

// impl<'a, T> IntoIterator for &'a mut RW<T>
// where
//     &'a mut T : IntoIterator<Item = &'a mut u16>
// {
//     type Item = <&'a mut T as IntoIterator>::Item;
//     type IntoIter = <&'a mut T as IntoIterator>::IntoIter;
//     fn into_iter(self) -> Self::IntoIter {
//         (&mut self.data).into_iter()
//     }
// }

impl<T: Clone> Clone for RW<T> {
    fn clone(&self) -> Self {
        RW::new(self.data.clone())
    }
}

impl<T: Default> Default for RW<T> {
    fn default() -> Self {
        RW::new(T::default())
    }
}

impl<T: RegisterData> RegisterData for RW<T> {
    fn register_span(&self) -> usize {
        self.data.register_span()
    }
}

impl<T: ReadRegisterData> ReadRegisterData for RW<T> {
    fn read(&self, offset: usize, buf: &mut [u16]) -> RegisterResult<usize> {
        self.data.read(offset, buf)
    }
}

impl<T: WriteRegisterData> WriteRegisterData for RW<T> {
    fn write(&mut self, offset: usize, buf: &[u16]) -> RegisterResult<usize> {
        self.data.write(offset, buf)
    }
}

#[cfg(test)]
mod register_tests {
    use super::*;
    use core::mem::size_of;

    #[test]
    fn reg_iter_001() {
        let reg0 = RegBool::new(true);
        let reg1 = RegU8::new(0x12);
        let reg2 = RegU16::new(0xADAC);
        let reg3 = RegU32::new(0x11223344);
        let reg4 = RegU64::new(0x0123456789ABCDEF);

        let iter0 = reg0.into_reg_iter();
        let iter1 = reg1.into_reg_iter();
        let iter2 = reg2.into_reg_iter();
        let iter3 = reg3.into_reg_iter();
        let iter4 = reg4.into_reg_iter();

        let mut iter = iter0.chain(iter1).chain(iter2).chain(iter3).chain(iter4);
        assert_eq!(Some(&0xFFFF), iter.next());
        assert_eq!(Some(&0x0012), iter.next());
        assert_eq!(Some(&0xADAC), iter.next());
        assert_eq!(Some(&0x1122), iter.next());
        assert_eq!(Some(&0x3344), iter.next());
        assert_eq!(Some(&0x0123), iter.next());
        assert_eq!(Some(&0x4567), iter.next());
        assert_eq!(Some(&0x89AB), iter.next());
        assert_eq!(Some(&0xCDEF), iter.next());
        assert_eq!(None, iter.next());
    }

    #[test]
    fn reg_iter_ro_register_001() {
        let reg0 = RegBool::new(true);
        let reg0 = RO::new(reg0);

        let reg1 = RegU8::new(0x12);
        let reg1 = RO::new(reg1);

        let reg2 = RegU16::new(0xADAC);
        let reg2 = RO::new(reg2);

        let reg3 = RegU32::new(0x11223344);
        let reg3 = RO::new(reg3);

        let reg4 = RegU64::new(0x0123456789ABCDEF);
        let reg4 = RO::new(reg4);

        let iter0 = reg0.into_iter();
        let iter1 = reg1.into_iter();
        let iter2 = reg2.into_iter();
        let iter3 = reg3.into_iter();
        let iter4 = reg4.into_iter();

        let mut iter = iter0.chain(iter1).chain(iter2).chain(iter3).chain(iter4);
        assert_eq!(Some(&0xFFFF), iter.next());
        assert_eq!(Some(&0x0012), iter.next());
        assert_eq!(Some(&0xADAC), iter.next());
        assert_eq!(Some(&0x1122), iter.next());
        assert_eq!(Some(&0x3344), iter.next());
        assert_eq!(Some(&0x0123), iter.next());
        assert_eq!(Some(&0x4567), iter.next());
        assert_eq!(Some(&0x89AB), iter.next());
        assert_eq!(Some(&0xCDEF), iter.next());
        assert_eq!(None, iter.next());
    }

    #[test]
    fn reg_iter_rw_register_001() {
        let reg0 = RegBool::new(true);
        let reg0 = RW::new(reg0);

        let reg1 = RegU8::new(0x12);
        let reg1 = RW::new(reg1);

        let reg2 = RegU16::new(0xADAC);
        let reg2 = RW::new(reg2);

        let reg3 = RegU32::new(0x11223344);
        let reg3 = RW::new(reg3);

        let reg4 = RegU64::new(0x0123456789ABCDEF);
        let reg4 = RW::new(reg4);

        let iter0 = reg0.into_iter();
        let iter1 = reg1.into_iter();
        let iter2 = reg2.into_iter();
        let iter3 = reg3.into_iter();
        let iter4 = reg4.into_iter();

        let mut iter = iter0.chain(iter1).chain(iter2).chain(iter3).chain(iter4);
        assert_eq!(Some(&0xFFFF), iter.next());
        assert_eq!(Some(&0x0012), iter.next());
        assert_eq!(Some(&0xADAC), iter.next());
        assert_eq!(Some(&0x1122), iter.next());
        assert_eq!(Some(&0x3344), iter.next());
        assert_eq!(Some(&0x0123), iter.next());
        assert_eq!(Some(&0x4567), iter.next());
        assert_eq!(Some(&0x89AB), iter.next());
        assert_eq!(Some(&0xCDEF), iter.next());
        assert_eq!(None, iter.next());
    }

    #[test]
    fn size_of_ro_001() {
        assert_eq!(size_of::<RO<RegBool>>(), size_of::<RegBool>());
        assert_eq!(size_of::<RO<RegU8>>(), size_of::<RegU8>());
        assert_eq!(size_of::<RO<RegU16>>(), size_of::<RegU16>());
        assert_eq!(size_of::<RO<RegU32>>(), size_of::<RegU32>());
        assert_eq!(size_of::<RO<RegU64>>(), size_of::<RegU64>());
        assert_eq!(size_of::<RO<RegI8>>(), size_of::<RegI8>());
        assert_eq!(size_of::<RO<RegI16>>(), size_of::<RegI16>());
        assert_eq!(size_of::<RO<RegI32>>(), size_of::<RegI32>());
        assert_eq!(size_of::<RO<RegI64>>(), size_of::<RegI64>());
        assert_eq!(size_of::<RO<RegF32>>(), size_of::<RegF32>());
        assert_eq!(size_of::<RO<RegF64>>(), size_of::<RegF64>());
    }

    #[test]
    fn get_ro_register_001() {
        let reg = RO::new(RegU32::new(42));
        assert_eq!(u32::from(reg.get()), 42);
    }

    #[test]
    fn get_rw_register_001() {
        let reg = RW::new(RegF32::new(1.5));
        assert_eq!(f32::from(reg.get()), 1.5);
    }
}
