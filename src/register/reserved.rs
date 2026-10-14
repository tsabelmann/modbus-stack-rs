use super::{ReadRegisterData, RegisterData, RegisterError, RegisterResult, WriteRegisterData};
use crate::Registers;

/// Occupies `span` addresses without storing data.
///
/// Reading yields the fill value (`0` by default), writing is rejected with
/// [RegisterError::AccessDenied].
///
/// # Example
///
/// ```
/// use modbus_stack::register::{ReadRegisterData, Reserved};
///
/// let gap = Reserved::with_value(4, 0x8000);
/// let mut buf = [0u16; 2];
/// gap.read(1, &mut buf).unwrap();
/// assert_eq!(buf, [0x8000, 0x8000]);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Reserved {
    span: usize,
    value: u16,
}

impl Reserved {
    /// Reserves `span` addresses that read as `0`.
    pub const fn new(span: usize) -> Reserved {
        Reserved { span, value: 0 }
    }

    /// Reserves `span` addresses that read as `value`, e.g. `0x8000` or `0xFFFF`
    /// for unimplemented SunSpec points.
    pub const fn with_value(span: usize, value: u16) -> Reserved {
        Reserved { span, value }
    }

    /// Fails with [RegisterError::OutOfBounds] unless `[offset, offset + len)` lies within the span.
    #[inline]
    fn check(&self, offset: usize, len: usize) -> RegisterResult<()> {
        match offset.checked_add(len) {
            Some(end) if end <= self.span => Ok(()),
            _ => Err(RegisterError::OutOfBounds { offset }),
        }
    }
}

impl RegisterData for Reserved {
    #[inline]
    fn register_span(&self) -> usize {
        self.span
    }
}

impl ReadRegisterData for Reserved {
    #[inline]
    fn read(&self, offset: usize, buf: &mut [u16]) -> RegisterResult<()> {
        self.check(offset, buf.len())?;
        buf.fill(self.value);
        Ok(())
    }
}

impl WriteRegisterData for Reserved {
    #[inline]
    fn write(&mut self, offset: usize, values: Registers<'_>) -> RegisterResult<()> {
        self.check(offset, values.len())?;
        Err(RegisterError::AccessDenied)
    }
}

#[cfg(test)]
mod reserved_tests {
    use super::*;
    use crate::register::{At, RW, ReadCursor, RegU16, WriteCursor};
    use core::mem::size_of;

    #[test]
    fn read_001() {
        let gap = Reserved::new(4);
        assert_eq!(gap.register_span(), 4);

        let mut buf = [0xAAAAu16; 4];
        assert_eq!(gap.read(0, &mut buf), Ok(()));
        assert_eq!(buf, [0, 0, 0, 0]);

        let mut buf = [0xAAAAu16; 2];
        assert_eq!(gap.read(1, &mut buf), Ok(()));
        assert_eq!(buf, [0, 0]);

        let gap = Reserved::with_value(4, 0x8000);
        let mut buf = [0u16; 3];
        assert_eq!(gap.read(1, &mut buf), Ok(()));
        assert_eq!(buf, [0x8000, 0x8000, 0x8000]);
    }

    #[test]
    fn read_out_of_bounds_001() {
        let gap = Reserved::new(4);

        let mut buf = [0u16; 2];
        assert_eq!(gap.read(3, &mut buf), Err(RegisterError::OutOfBounds { offset: 3 }));
        assert_eq!(gap.read(usize::MAX, &mut buf), Err(RegisterError::OutOfBounds { offset: usize::MAX }));

        let mut buf = [0u16; 0];
        assert_eq!(gap.read(4, &mut buf), Ok(()));
        assert_eq!(gap.read(5, &mut buf), Err(RegisterError::OutOfBounds { offset: 5 }));
    }

    #[test]
    fn write_001() {
        let mut gap = Reserved::new(4);
        assert_eq!(gap.write(0, Registers::from(&[1, 2][..])), Err(RegisterError::AccessDenied));
        assert_eq!(gap.write(3, Registers::from(&[1, 2][..])), Err(RegisterError::OutOfBounds { offset: 3 }));
    }

    /// Model with a gap between two registers.
    struct Model {
        a: RW<RegU16>,
        gap: Reserved,
        b: RW<RegU16>,
    }

    impl RegisterData for Model {
        fn register_span(&self) -> usize {
            self.a.register_span() + self.gap.register_span() + self.b.register_span()
        }
    }

    impl ReadRegisterData for Model {
        fn read(&self, offset: usize, buf: &mut [u16]) -> RegisterResult<()> {
            let mut cursor = ReadCursor::new(offset, buf);
            cursor.visit(&self.a)?;
            cursor.visit(&self.gap)?;
            cursor.visit(&self.b)?;
            cursor.finish()
        }
    }

    impl WriteRegisterData for Model {
        fn write(&mut self, offset: usize, values: Registers<'_>) -> RegisterResult<()> {
            let mut cursor = WriteCursor::new(offset, values);
            cursor.visit(&mut self.a)?;
            cursor.visit(&mut self.gap)?;
            cursor.visit(&mut self.b)?;
            cursor.finish()
        }
    }

    fn model(gap: usize) -> Model {
        Model {
            a: RW::new(RegU16::new(1)),
            gap: Reserved::new(gap),
            b: RW::new(RegU16::new(2)),
        }
    }

    #[test]
    fn model_001() {
        let mut model = model(2);

        let mut buf = [0u16; 4];
        assert_eq!(model.read(0, &mut buf), Ok(()));
        assert_eq!(buf, [1, 0, 0, 2]);

        assert_eq!(model.write(3, Registers::from(&[20][..])), Ok(()));
        assert_eq!(model.write(1, Registers::from(&[9][..])), Err(RegisterError::AccessDenied));
        assert_eq!(model.write(0, Registers::from(&[9, 9][..])), Err(RegisterError::AccessDenied));

        assert_eq!(model.read(0, &mut buf), Ok(()));
        // `a` is written before the gap rejects the write
        assert_eq!(buf, [9, 0, 0, 20]);
    }

    #[test]
    fn large_gap_001() {
        let model = At::new(0, model(39_900));
        assert_eq!(model.register_span(), 39_902);

        let mut buf = [0u16; 2];
        assert_eq!(model.read(39_900, &mut buf), Ok(()));
        assert_eq!(buf, [0, 2]);

        assert!(size_of::<Reserved>() <= 2 * size_of::<usize>());
    }
}
