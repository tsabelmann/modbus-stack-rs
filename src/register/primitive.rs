//! Register implementations for primitive types.
//!
//! The encoding is identical to the corresponding `RegXxx` types:
//!
//! | Type | Registers | Encoding |
//! |---|---|---|
//! | [bool] | 1 | `0x0000` (false) or `0xFFFF` (true) |
//! | [u8] | 1 | `0x0000..=0x00FF` |
//! | [i8] | 1 | sign-extended, i.e. `0x0000..=0x007F` or `0xFF80..=0xFFFF` |
//! | [u16], [i16] | 1 | any value |
//! | [u32], [i32], [f32] | 2 | big endian word order, only accessible as a whole |
//! | [u64], [i64], [f64] | 4 | big endian word order, only accessible as a whole |
//! | `[u16; N]` | N | any value, partial access allowed |
//!
//! Writing an invalid value fails with [RegisterError::InvalidValue](super::RegisterError::InvalidValue);
//! on any error the value is left unchanged.

use super::{
    ReadRegisterData, RegisterData, RegisterResult, WriteRegisterData, read_whole, read_words,
    write_whole, write_words, write_words_with,
};
use crate::Registers;

/* bool */

impl RegisterData for bool {
    #[inline]
    fn register_span(&self) -> usize {
        1
    }
}

impl ReadRegisterData for bool {
    #[inline]
    fn read(&self, offset: usize, buf: &mut [u16]) -> RegisterResult<()> {
        read_words(&[if *self { 0xFFFF } else { 0x0000 }], offset, buf)
    }
}

impl WriteRegisterData for bool {
    #[inline]
    fn write(&mut self, offset: usize, values: Registers<'_>) -> RegisterResult<()> {
        let mut words = [if *self { 0xFFFF } else { 0x0000 }];
        write_words_with(&mut words, offset, values, |value| matches!(value, 0x0000 | 0xFFFF))?;
        *self = words[0] != 0x0000;
        Ok(())
    }
}

/* u8 */

impl RegisterData for u8 {
    #[inline]
    fn register_span(&self) -> usize {
        1
    }
}

impl ReadRegisterData for u8 {
    #[inline]
    fn read(&self, offset: usize, buf: &mut [u16]) -> RegisterResult<()> {
        read_words(&[*self as u16], offset, buf)
    }
}

impl WriteRegisterData for u8 {
    #[inline]
    fn write(&mut self, offset: usize, values: Registers<'_>) -> RegisterResult<()> {
        let mut words = [*self as u16];
        write_words_with(&mut words, offset, values, |value| value <= 0x00FF)?;
        *self = words[0] as u8;
        Ok(())
    }
}

/* i8 */

impl RegisterData for i8 {
    #[inline]
    fn register_span(&self) -> usize {
        1
    }
}

impl ReadRegisterData for i8 {
    #[inline]
    fn read(&self, offset: usize, buf: &mut [u16]) -> RegisterResult<()> {
        read_words(&[*self as i16 as u16], offset, buf)
    }
}

impl WriteRegisterData for i8 {
    #[inline]
    fn write(&mut self, offset: usize, values: Registers<'_>) -> RegisterResult<()> {
        let mut words = [*self as i16 as u16];
        write_words_with(&mut words, offset, values, |value| {
            (-128..=127).contains(&(value as i16))
        })?;
        *self = words[0] as i16 as i8;
        Ok(())
    }
}

/* u16 */

impl RegisterData for u16 {
    #[inline]
    fn register_span(&self) -> usize {
        1
    }
}

impl ReadRegisterData for u16 {
    #[inline]
    fn read(&self, offset: usize, buf: &mut [u16]) -> RegisterResult<()> {
        read_words(core::slice::from_ref(self), offset, buf)
    }
}

impl WriteRegisterData for u16 {
    #[inline]
    fn write(&mut self, offset: usize, values: Registers<'_>) -> RegisterResult<()> {
        write_words(core::slice::from_mut(self), offset, values)
    }
}

/* i16 */

impl RegisterData for i16 {
    #[inline]
    fn register_span(&self) -> usize {
        1
    }
}

impl ReadRegisterData for i16 {
    #[inline]
    fn read(&self, offset: usize, buf: &mut [u16]) -> RegisterResult<()> {
        read_words(&[*self as u16], offset, buf)
    }
}

impl WriteRegisterData for i16 {
    #[inline]
    fn write(&mut self, offset: usize, values: Registers<'_>) -> RegisterResult<()> {
        let mut words = [*self as u16];
        write_words(&mut words, offset, values)?;
        *self = words[0] as i16;
        Ok(())
    }
}

/* [u16; N] */

impl<const N: usize> RegisterData for [u16; N] {
    #[inline]
    fn register_span(&self) -> usize {
        N
    }
}

impl<const N: usize> ReadRegisterData for [u16; N] {
    #[inline]
    fn read(&self, offset: usize, buf: &mut [u16]) -> RegisterResult<()> {
        read_words(self, offset, buf)
    }
}

impl<const N: usize> WriteRegisterData for [u16; N] {
    #[inline]
    fn write(&mut self, offset: usize, values: Registers<'_>) -> RegisterResult<()> {
        write_words(self, offset, values)
    }
}

/* multi-register types */

/// Splits big endian bytes into registers.
#[inline]
fn to_words<const W: usize, const B: usize>(bytes: [u8; B]) -> [u16; W] {
    core::array::from_fn(|i| u16::from_be_bytes([bytes[2 * i], bytes[2 * i + 1]]))
}

/// Joins registers into big endian bytes.
#[inline]
fn to_bytes<const W: usize, const B: usize>(words: [u16; W]) -> [u8; B] {
    core::array::from_fn(|i| words[i / 2].to_be_bytes()[i % 2])
}

/// Implements the register traits for a type spanning `$words` registers,
/// based on its big endian byte representation of `$bytes` bytes.
macro_rules! impl_multi_register {
    ($($ty:ty: $words:literal, $bytes:literal;)*) => {$(
        impl RegisterData for $ty {
            #[inline]
            fn register_span(&self) -> usize {
                $words
            }
        }

        impl ReadRegisterData for $ty {
            #[inline]
            fn read(&self, offset: usize, buf: &mut [u16]) -> RegisterResult<()> {
                read_whole(&to_words::<$words, $bytes>(self.to_be_bytes()), offset, buf)
            }
        }

        impl WriteRegisterData for $ty {
            #[inline]
            fn write(&mut self, offset: usize, values: Registers<'_>) -> RegisterResult<()> {
                let mut words = to_words::<$words, $bytes>(self.to_be_bytes());
                write_whole(&mut words, offset, values)?;
                *self = <$ty>::from_be_bytes(to_bytes::<$words, $bytes>(words));
                Ok(())
            }
        }
    )*};
}

impl_multi_register! {
    u32: 2, 4;
    i32: 2, 4;
    f32: 2, 4;
    u64: 4, 8;
    i64: 4, 8;
    f64: 4, 8;
}

#[cfg(test)]
mod primitive_tests {
    use super::*;
    use crate::register::{
        At, RO, RW, ReadCursor, RegBool, RegF32, RegF64, RegI8, RegI16, RegI32, RegI64, RegU8,
        RegU16, RegU16Array, RegU32, RegU64, RegisterError, WriteCursor,
    };

    /// Asserts that `a` and `b` span and read the same registers.
    fn assert_same_words(a: &dyn ReadRegisterData, b: &dyn ReadRegisterData) {
        let span = a.register_span();
        assert_eq!(span, b.register_span());

        let mut buf_a = [0u16; 8];
        let mut buf_b = [0u16; 8];
        assert_eq!(a.read(0, &mut buf_a[..span]), Ok(()));
        assert_eq!(b.read(0, &mut buf_b[..span]), Ok(()));
        assert_eq!(buf_a, buf_b);
    }

    #[test]
    fn same_words_as_reg_types_001() {
        for value in [false, true] {
            assert_same_words(&value, &RegBool::new(value));
        }
        for value in [0, 1, 0x7F, 0xFF] {
            assert_same_words(&value, &RegU8::new(value));
        }
        for value in [0, 1, -1, 127, -128] {
            assert_same_words(&value, &RegI8::new(value));
        }
        for value in [0, 1, 0x1234, 0xFFFF] {
            assert_same_words(&value, &RegU16::new(value));
        }
        for value in [0, 1, -1, i16::MIN, i16::MAX] {
            assert_same_words(&value, &RegI16::new(value));
        }
        for value in [0, 1, 0x1122_3344, u32::MAX] {
            assert_same_words(&value, &RegU32::new(value));
        }
        for value in [0, 1, -1, i32::MIN, i32::MAX] {
            assert_same_words(&value, &RegI32::new(value));
        }
        for value in [0, 1, 0x1122_3344_5566_7788, u64::MAX] {
            assert_same_words(&value, &RegU64::new(value));
        }
        for value in [0, 1, -1, i64::MIN, i64::MAX] {
            assert_same_words(&value, &RegI64::new(value));
        }
        for value in [0.0, -1.5, 42.0, f32::MAX, f32::INFINITY] {
            assert_same_words(&value, &RegF32::new(value));
        }
        for value in [0.0, -1.5, 42.0, f64::MAX, f64::INFINITY] {
            assert_same_words(&value, &RegF64::new(value));
        }
        let value = [0x1234, 0x5678, 0x9ABC];
        assert_same_words(&value, &RegU16Array::new(value));
    }

    #[test]
    fn write_001() {
        let mut value = false;
        assert_eq!(value.write(0, Registers::from(&[0xFFFF][..])), Ok(()));
        assert!(value);

        let mut value = 0u8;
        assert_eq!(value.write(0, Registers::from(&[0x00FF][..])), Ok(()));
        assert_eq!(value, 0xFF);

        let mut value = 0i8;
        assert_eq!(value.write(0, Registers::from(&[0xFF80][..])), Ok(()));
        assert_eq!(value, -128);

        let mut value = 0u16;
        assert_eq!(value.write(0, Registers::from(&[0xABCD][..])), Ok(()));
        assert_eq!(value, 0xABCD);

        let mut value = 0i16;
        assert_eq!(value.write(0, Registers::from(&[0xFFFE][..])), Ok(()));
        assert_eq!(value, -2);

        let mut value = 0u32;
        assert_eq!(value.write(0, Registers::from(&[0x1122, 0x3344][..])), Ok(()));
        assert_eq!(value, 0x1122_3344);

        let mut value = 0i32;
        assert_eq!(value.write(0, Registers::from(&[0xFFFF, 0xFFFC][..])), Ok(()));
        assert_eq!(value, -4);

        let mut value = 0u64;
        let words = [0x1122, 0x3344, 0x5566, 0x7788];
        assert_eq!(value.write(0, Registers::from(&words[..])), Ok(()));
        assert_eq!(value, 0x1122_3344_5566_7788);

        let mut value = 0i64;
        let words = [0xFFFF, 0xFFFF, 0xFFFF, 0xFFFE];
        assert_eq!(value.write(0, Registers::from(&words[..])), Ok(()));
        assert_eq!(value, -2);

        let mut value = 0f32;
        let words = to_words::<2, 4>(42.0f32.to_be_bytes());
        assert_eq!(value.write(0, Registers::from(&words[..])), Ok(()));
        assert_eq!(value, 42.0);

        let mut value = 0f64;
        let words = to_words::<4, 8>(42.0f64.to_be_bytes());
        assert_eq!(value.write(0, Registers::from(&words[..])), Ok(()));
        assert_eq!(value, 42.0);

        let mut value = [0u16; 3];
        assert_eq!(value.write(1, Registers::from(&[2, 3][..])), Ok(()));
        assert_eq!(value, [0, 2, 3]);
    }

    #[test]
    fn write_invalid_value_001() {
        let mut value = false;
        assert_eq!(
            value.write(0, Registers::from(&[0x0001][..])),
            Err(RegisterError::InvalidValue)
        );
        assert!(!value);

        let mut value = 0x12u8;
        assert_eq!(
            value.write(0, Registers::from(&[0x0100][..])),
            Err(RegisterError::InvalidValue)
        );
        assert_eq!(value, 0x12);

        let mut value = 42i8;
        for word in [0x0080, 0x00FF, 0xFF7F] {
            assert_eq!(
                value.write(0, Registers::from(&[word][..])),
                Err(RegisterError::InvalidValue)
            );
        }
        assert_eq!(value, 42);
    }

    #[test]
    fn partial_access_001() {
        let mut value = 0x1122_3344u32;
        let mut buf = [0u16; 1];
        assert_eq!(value.read(1, &mut buf), Err(RegisterError::PartialAccess));
        assert_eq!(
            value.write(0, Registers::from(&[0xFFFF][..])),
            Err(RegisterError::PartialAccess)
        );
        assert_eq!(
            value.write(2, Registers::from(&[0xFFFF][..])),
            Err(RegisterError::OutOfBounds { offset: 2 })
        );
        assert_eq!(value, 0x1122_3344);

        let mut value = 42.0f64;
        let mut buf = [0u16; 2];
        assert_eq!(value.read(2, &mut buf), Err(RegisterError::PartialAccess));
        assert_eq!(
            value.write(0, Registers::from(&[0, 0, 0][..])),
            Err(RegisterError::PartialAccess)
        );
        assert_eq!(value, 42.0);

        let value = [1u16, 2, 3];
        let mut buf = [0u16; 2];
        assert_eq!(value.read(1, &mut buf), Ok(()));
        assert_eq!(buf, [2, 3]);
    }

    #[test]
    fn empty_write_001() {
        let mut value = true;
        assert_eq!(value.write(1, Registers::from(&[][..])), Ok(()));
        assert!(value);

        let mut value = -5i8;
        assert_eq!(value.write(0, Registers::from(&[][..])), Ok(()));
        assert_eq!(value, -5);
    }

    #[test]
    fn wrapped_001() {
        let a = RO::new(0x1122_3344u32);
        let mut b = RW::new(0u16);
        let c = RO::new([0xAAAAu16, 0xBBBB]);

        let mut buf = [0u16; 5];
        let mut cursor = ReadCursor::new(0, &mut buf);
        cursor.visit(&a).unwrap();
        cursor.visit(&b).unwrap();
        cursor.visit(&c).unwrap();
        assert_eq!(cursor.finish(), Ok(()));
        assert_eq!(buf, [0x1122, 0x3344, 0x0000, 0xAAAA, 0xBBBB]);

        let mut cursor = WriteCursor::new(2, Registers::from(&[0x0042][..]));
        cursor.visit(&mut RO::new(0u32)).unwrap();
        cursor.visit(&mut b).unwrap();
        assert_eq!(cursor.finish(), Ok(()));
        assert_eq!(*b, 0x0042);

        let mut a = a;
        assert_eq!(
            a.write(0, Registers::from(&[0, 0][..])),
            Err(RegisterError::AccessDenied)
        );
        assert_eq!(*a, 0x1122_3344);

        let mut reg = At::new(100, RW::new(5i16));
        let mut buf = [0u16; 1];
        assert_eq!(reg.read(100, &mut buf), Ok(()));
        assert_eq!(buf, [0x0005]);
        assert_eq!(reg.write(100, Registers::from(&[0xFFFF][..])), Ok(()));
        assert_eq!(**reg, -1);
    }
}
