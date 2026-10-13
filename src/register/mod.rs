// pub mod atomic;

use crate::{ExceptionCode, Registers};
use core::ops::{Deref, DerefMut};

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

// Placement

mod at;
pub use at::At;

// Composition

mod cursor;
pub use cursor::{ReadCursor, WriteCursor};

mod reserved;
pub use reserved::Reserved;

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

/// Mögliche Fehler bei Register-Operationen
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RegisterError {
    /// The range `[offset, offset + len)` is not (completely) inside the register.
    OutOfBounds { offset: usize },
    /// Only a part of a multi-register value was addressed.
    PartialAccess,
    /// Register is not readable/writable
    AccessDenied,
    /// The value is not valid for the register.
    InvalidValue,
}

impl From<RegisterError> for ExceptionCode {
    fn from(value: RegisterError) -> Self {
        match value {
            RegisterError::OutOfBounds { .. }
            | RegisterError::PartialAccess
            | RegisterError::AccessDenied => ExceptionCode::IllegalDataAddress,
            RegisterError::InvalidValue => ExceptionCode::IllegalDataValue,
        }
    }
}

pub type RegisterResult<T> = Result<T, RegisterError>;

pub trait RegisterData {
    /// Retrieves the maximum register span, including holes
    /// that are not accessible.
    fn register_span(&self) -> usize;
}

pub trait ReadRegisterData: RegisterData {
    /// Reads exactly `buf.len()` registers starting at `offset`.
    /// Fails if any of them can not be read; `buf` may then be partially written.
    fn read(&self, offset: usize, buf: &mut [u16]) -> RegisterResult<()>;
}

pub trait WriteRegisterData: RegisterData {
    /// Writes all `values` starting at `offset`.
    /// On `Err` a single register is left unchanged;
    /// registers composed with [WriteCursor] may be partially written.
    fn write(&mut self, offset: usize, values: Registers<'_>) -> RegisterResult<()>;
}

impl RegisterData for &[u16] {
    fn register_span(&self) -> usize {
        self.len()
    }
}

/// Copies `buf.len()` registers starting at `offset` from `data` into `buf`.
pub fn read_words(data: &[u16], offset: usize, buf: &mut [u16]) -> RegisterResult<()> {
    let src = offset
        .checked_add(buf.len())
        .and_then(|end| data.get(offset..end))
        .ok_or(RegisterError::OutOfBounds { offset })?;
    buf.copy_from_slice(src);
    Ok(())
}

/// Like [read_words], but only the whole `data` can be read at once.
pub fn read_whole(data: &[u16], offset: usize, buf: &mut [u16]) -> RegisterResult<()> {
    if offset >= data.len() || buf.len() > data.len() - offset {
        return Err(RegisterError::OutOfBounds { offset });
    }
    if offset != 0 || buf.len() != data.len() {
        return Err(RegisterError::PartialAccess);
    }
    read_words(data, offset, buf)
}

/// Copies all `values` into `data` starting at `offset`.
pub fn write_words(data: &mut [u16], offset: usize, values: Registers<'_>) -> RegisterResult<()> {
    write_words_with(data, offset, values, |_| true)
}

/// Like [write_words], but fails with [RegisterError::InvalidValue] if any value is not valid.
pub fn write_words_with(
    data: &mut [u16],
    offset: usize,
    values: Registers<'_>,
    is_valid: fn(u16) -> bool,
) -> RegisterResult<()> {
    let dst = offset
        .checked_add(values.len())
        .and_then(|end| data.get_mut(offset..end))
        .ok_or(RegisterError::OutOfBounds { offset })?;
    if !values.clone().all(is_valid) {
        return Err(RegisterError::InvalidValue);
    }
    for (dst, value) in dst.iter_mut().zip(values) {
        *dst = value;
    }
    Ok(())
}

/// Like [write_words], but only the whole `data` can be written at once.
pub fn write_whole(data: &mut [u16], offset: usize, values: Registers<'_>) -> RegisterResult<()> {
    if offset >= data.len() || values.len() > data.len() - offset {
        return Err(RegisterError::OutOfBounds { offset });
    }
    if offset != 0 || values.len() != data.len() {
        return Err(RegisterError::PartialAccess);
    }
    write_words(data, offset, values)
}

impl ReadRegisterData for &[u16] {
    fn read(&self, offset: usize, buf: &mut [u16]) -> RegisterResult<()> {
        read_words(self, offset, buf)
    }
}

impl RegisterData for &mut [u16] {
    fn register_span(&self) -> usize {
        self.len()
    }
}

impl ReadRegisterData for &mut [u16] {
    fn read(&self, offset: usize, buf: &mut [u16]) -> RegisterResult<()> {
        read_words(self, offset, buf)
    }
}

impl WriteRegisterData for &mut [u16] {
    fn write(&mut self, offset: usize, values: Registers<'_>) -> RegisterResult<()> {
        write_words(self, offset, values)
    }
}

/// Read-only register enforced by the type system.
#[derive(Debug)]
pub struct RO<T> {
    inner: T,
}

impl<T> RO<T> {
    pub const fn new(value: T) -> RO<T> {
        RO { inner: value }
    }

    /// Returns a reference to the wrapped register.
    pub const fn get(&self) -> &T {
        &self.inner
    }

    pub fn modify<F, U>(&mut self, f: F) -> U
    where
        F: FnOnce(&mut T) -> U,
    {
        f(&mut self.inner)
    }
}

impl<T> DerefMut for RO<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl<T> Deref for RO<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<'a, T> IntoIterator for &'a RO<T>
where
    &'a T: IntoRegIter<'a>,
{
    type Item = &'a u16;
    type IntoIter = <&'a T as IntoRegIter<'a>>::IntoIter;
    fn into_iter(self) -> Self::IntoIter {
        (&self.inner).into_reg_iter()
    }
}

impl<T: Clone> Clone for RO<T> {
    fn clone(&self) -> Self {
        RO::new(self.inner.clone())
    }
}

impl<T: Default> Default for RO<T> {
    fn default() -> Self {
        RO::new(T::default())
    }
}

impl<T: RegisterData> RegisterData for RO<T> {
    fn register_span(&self) -> usize {
        self.inner.register_span()
    }
}

impl<T: ReadRegisterData> ReadRegisterData for RO<T> {
    fn read(&self, offset: usize, buf: &mut [u16]) -> RegisterResult<()> {
        self.inner.read(offset, buf)
    }
}

/// Rejects every write within its span, so read-only registers can be part of writable models.
impl<T: RegisterData> WriteRegisterData for RO<T> {
    fn write(&mut self, offset: usize, values: Registers<'_>) -> RegisterResult<()> {
        match offset.checked_add(values.len()) {
            Some(end) if end <= self.register_span() => Err(RegisterError::AccessDenied),
            _ => Err(RegisterError::OutOfBounds { offset }),
        }
    }
}

/// Read and writable register.
#[derive(Debug)]
pub struct RW<T> {
    inner: T,
}

impl<T> RW<T> {
    pub const fn new(value: T) -> RW<T> {
        RW { inner: value }
    }

    /// Returns a reference to the wrapped register.
    pub const fn get(&self) -> &T {
        &self.inner
    }

    pub fn modify<F, U>(&mut self, f: F) -> U
    where
        F: FnOnce(&mut T) -> U,
    {
        f(&mut self.inner)
    }
}

impl<T> DerefMut for RW<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl<T> Deref for RW<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<'a, T> IntoIterator for &'a RW<T>
where
    &'a T: IntoRegIter<'a>,
{
    type Item = &'a u16;
    type IntoIter = <&'a T as IntoRegIter<'a>>::IntoIter;
    fn into_iter(self) -> Self::IntoIter {
        (&self.inner).into_reg_iter()
    }
}

impl<'a, T> IntoIterator for &'a mut RW<T>
where
    &'a mut T: IntoRegIterMut<'a>,
{
    type Item = &'a mut u16;
    type IntoIter = <&'a mut T as IntoRegIterMut<'a>>::IntoIter;
    fn into_iter(self) -> Self::IntoIter {
        (&mut self.inner).into_reg_iter_mut()
    }
}

impl<T: Clone> Clone for RW<T> {
    fn clone(&self) -> Self {
        RW::new(self.inner.clone())
    }
}

impl<T: Default> Default for RW<T> {
    fn default() -> Self {
        RW::new(T::default())
    }
}

impl<T: RegisterData> RegisterData for RW<T> {
    fn register_span(&self) -> usize {
        self.inner.register_span()
    }
}

impl<T: ReadRegisterData> ReadRegisterData for RW<T> {
    fn read(&self, offset: usize, buf: &mut [u16]) -> RegisterResult<()> {
        self.inner.read(offset, buf)
    }
}

impl<T: WriteRegisterData> WriteRegisterData for RW<T> {
    fn write(&mut self, offset: usize, values: Registers<'_>) -> RegisterResult<()> {
        self.inner.write(offset, values)
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

    #[test]
    fn read_slice_001() {
        let data: &[u16] = &[1, 2, 3, 4];

        let mut buf = [0u16; 4];
        assert_eq!(data.read(0, &mut buf), Ok(()));
        assert_eq!(buf, [1, 2, 3, 4]);

        let mut buf = [0u16; 2];
        assert_eq!(data.read(1, &mut buf), Ok(()));
        assert_eq!(buf, [2, 3]);

        let mut buf = [0u16; 2];
        assert_eq!(
            data.read(3, &mut buf),
            Err(RegisterError::OutOfBounds { offset: 3 })
        );
        assert_eq!(
            data.read(usize::MAX, &mut buf),
            Err(RegisterError::OutOfBounds { offset: usize::MAX })
        );

        let mut buf = [0u16; 0];
        assert_eq!(data.read(2, &mut buf), Ok(()));
    }

    #[test]
    fn read_whole_register_001() {
        let reg = RegU32::new(0x11223344);

        let mut buf = [0u16; 2];
        assert_eq!(reg.read(0, &mut buf), Ok(()));
        assert_eq!(buf, [0x1122, 0x3344]);

        let mut buf = [0u16; 1];
        assert_eq!(reg.read(0, &mut buf), Err(RegisterError::PartialAccess));
        assert_eq!(reg.read(1, &mut buf), Err(RegisterError::PartialAccess));
        assert_eq!(
            reg.read(2, &mut buf),
            Err(RegisterError::OutOfBounds { offset: 2 })
        );

        let mut buf = [0u16; 3];
        assert_eq!(
            reg.read(0, &mut buf),
            Err(RegisterError::OutOfBounds { offset: 0 })
        );
    }

    #[test]
    fn read_ro_register_001() {
        let reg = RO::new(RegU16::new(0xABCD));
        let mut buf = [0u16; 1];
        assert_eq!(reg.read(0, &mut buf), Ok(()));
        assert_eq!(buf, [0xABCD]);
        assert_eq!(
            reg.read(1, &mut buf),
            Err(RegisterError::OutOfBounds { offset: 1 })
        );
    }

    #[test]
    fn register_error_into_exception_code_001() {
        let errors = [
            RegisterError::OutOfBounds { offset: 0 },
            RegisterError::PartialAccess,
            RegisterError::AccessDenied,
        ];
        for error in errors {
            assert_eq!(
                ExceptionCode::from(error),
                ExceptionCode::IllegalDataAddress
            );
        }
    }

    #[test]
    fn read_dyn_registers_001() {
        let reg0 = RegBool::new(true);
        let reg1 = RegU32::new(0x11223344);
        let reg2 = RO::new(RegU16::new(0xABCD));
        let regs: [&dyn ReadRegisterData; 3] = [&reg0, &reg1, &reg2];

        let mut buf = [0u16; 4];
        let mut offset = 0;
        for reg in regs {
            let span = reg.register_span();
            assert_eq!(reg.read(0, &mut buf[offset..offset + span]), Ok(()));
            offset += span;
        }
        assert_eq!(buf, [0xFFFF, 0x1122, 0x3344, 0xABCD]);
    }

    #[test]
    fn invalid_value_into_exception_code_001() {
        assert_eq!(
            ExceptionCode::from(RegisterError::InvalidValue),
            ExceptionCode::IllegalDataValue
        );
    }

    #[test]
    fn write_slice_001() {
        let mut data = [1u16, 2, 3, 4];
        let mut slice: &mut [u16] = &mut data;

        assert_eq!(slice.write(1, Registers::from(&[20, 30][..])), Ok(()));
        assert_eq!(slice, [1, 20, 30, 4]);

        assert_eq!(
            slice.write(3, Registers::from(&[40, 50][..])),
            Err(RegisterError::OutOfBounds { offset: 3 })
        );
        assert_eq!(
            slice.write(usize::MAX, Registers::from(&[0][..])),
            Err(RegisterError::OutOfBounds { offset: usize::MAX })
        );
        assert_eq!(slice, [1, 20, 30, 4]);

        assert_eq!(slice.write(4, Registers::from(&[][..])), Ok(()));
    }

    #[test]
    fn write_whole_register_001() {
        let mut reg = RegU32::new(0);

        assert_eq!(reg.write(0, Registers::from(&[0x1122, 0x3344][..])), Ok(()));
        assert_eq!(u32::from(&reg), 0x11223344);

        assert_eq!(
            reg.write(0, Registers::from(&[0xFFFF][..])),
            Err(RegisterError::PartialAccess)
        );
        assert_eq!(
            reg.write(1, Registers::from(&[0xFFFF][..])),
            Err(RegisterError::PartialAccess)
        );
        assert_eq!(
            reg.write(2, Registers::from(&[0xFFFF][..])),
            Err(RegisterError::OutOfBounds { offset: 2 })
        );
        assert_eq!(
            reg.write(0, Registers::from(&[1, 2, 3][..])),
            Err(RegisterError::OutOfBounds { offset: 0 })
        );
        assert_eq!(u32::from(&reg), 0x11223344);
    }

    #[test]
    fn write_invalid_value_001() {
        let mut reg = RegU8::new(0x12);
        assert_eq!(reg.write(0, Registers::from(&[0x00FF][..])), Ok(()));
        assert_eq!(u8::from(&reg), 0xFF);
        assert_eq!(
            reg.write(0, Registers::from(&[0x0100][..])),
            Err(RegisterError::InvalidValue)
        );
        assert_eq!(
            reg.write(1, Registers::from(&[0x0100][..])),
            Err(RegisterError::OutOfBounds { offset: 1 })
        );
        assert_eq!(u8::from(&reg), 0xFF);

        let mut reg = RegBool::new(false);
        assert_eq!(reg.write(0, Registers::from(&[0xFFFF][..])), Ok(()));
        assert!(bool::from(&reg));
        assert_eq!(reg.write(0, Registers::from(&[0x0000][..])), Ok(()));
        assert!(!bool::from(&reg));
        assert_eq!(
            reg.write(0, Registers::from(&[0x0001][..])),
            Err(RegisterError::InvalidValue)
        );
        assert!(!bool::from(&reg));
    }

    #[test]
    fn write_ro_register_001() {
        let mut reg = RO::new(RegU16::new(0xABCD));
        assert_eq!(
            reg.write(0, Registers::from(&[1][..])),
            Err(RegisterError::AccessDenied)
        );
        assert_eq!(
            reg.write(1, Registers::from(&[1][..])),
            Err(RegisterError::OutOfBounds { offset: 1 })
        );
        assert_eq!(
            reg.write(0, Registers::from(&[1, 2][..])),
            Err(RegisterError::OutOfBounds { offset: 0 })
        );
        assert_eq!(u16::from(reg.get()), 0xABCD);
    }

    #[test]
    fn write_rw_register_001() {
        let mut reg = RW::new(RegU16::new(0));
        assert_eq!(reg.write(0, Registers::from(&[0xABCD][..])), Ok(()));
        assert_eq!(u16::from(reg.get()), 0xABCD);
        assert_eq!(
            reg.write(1, Registers::from(&[0][..])),
            Err(RegisterError::OutOfBounds { offset: 1 })
        );
    }
}
