use super::{ReadRegisterData, RegisterError, RegisterResult};

/// Distributes a read request over consecutive registers.
///
/// Each visited register receives the part of the request that falls into its span;
/// registers before or after the requested range are skipped.
///
/// # Example
///
/// ```
/// use modbus_stack::register::{RO, RW, ReadCursor, ReadRegisterData, RegisterData, RegisterResult, RegU16, RegU32};
///
/// struct Model {
///     a: RO<RegU16>,
///     b: RW<RegU32>,
/// }
///
/// impl RegisterData for Model {
///     fn register_span(&self) -> usize {
///         self.a.register_span() + self.b.register_span()
///     }
/// }
///
/// impl ReadRegisterData for Model {
///     fn read(&self, offset: usize, buf: &mut [u16]) -> RegisterResult<()> {
///         let mut cursor = ReadCursor::new(offset, buf);
///         cursor.visit(&self.a)?;
///         cursor.visit(&self.b)?;
///         cursor.finish()
///     }
/// }
///
/// let model = Model { a: RO::new(RegU16::new(1)), b: RW::new(RegU32::new(0x00020003)) };
/// let mut buf = [0u16; 3];
/// model.read(0, &mut buf).unwrap();
/// assert_eq!(buf, [1, 2, 3]);
/// ```
pub struct ReadCursor<'a> {
    /// Requested offset, used for error reporting.
    start: usize,
    /// Distance from the current register to the start of the requested range.
    offset: usize,
    /// Not yet filled part of the output buffer.
    buf: &'a mut [u16],
}

impl<'a> ReadCursor<'a> {
    pub fn new(offset: usize, buf: &'a mut [u16]) -> ReadCursor<'a> {
        ReadCursor {
            start: offset,
            offset,
            buf,
        }
    }

    /// Reads the part of the request that falls into `reg`, then advances past `reg`.
    pub fn visit(&mut self, reg: &dyn ReadRegisterData) -> RegisterResult<()> {
        let span = reg.register_span();

        // `reg` lies completely before the requested range
        if self.offset >= span {
            self.offset -= span;
            return Ok(());
        }

        // `reg` lies after the requested range
        if self.buf.is_empty() {
            self.offset = 0;
            return Ok(());
        }

        let n = self.buf.len().min(span - self.offset);
        let (head, tail) = core::mem::take(&mut self.buf).split_at_mut(n);
        reg.read(self.offset, head)?;
        self.buf = tail;
        self.offset = 0;
        Ok(())
    }

    /// Fails with [RegisterError::OutOfBounds] if the requested range reaches beyond the last visited register.
    pub fn finish(self) -> RegisterResult<()> {
        if self.offset == 0 && self.buf.is_empty() {
            Ok(())
        } else {
            Err(RegisterError::OutOfBounds { offset: self.start })
        }
    }
}

#[cfg(test)]
mod cursor_tests {
    use super::*;
    use crate::register::{At, RO, RW, RegU16, RegU16Array, RegU32, RegisterData};

    /// Model with a header (`id`, `len`) and a single point.
    struct Inner {
        id: RO<RegU16>,
        len: RO<RegU16>,
        value: RW<RegU32>,
    }

    impl RegisterData for Inner {
        fn register_span(&self) -> usize {
            self.id.register_span() + self.len.register_span() + self.value.register_span()
        }
    }

    impl ReadRegisterData for Inner {
        fn read(&self, offset: usize, buf: &mut [u16]) -> RegisterResult<()> {
            let mut cursor = ReadCursor::new(offset, buf);
            cursor.visit(&self.id)?;
            cursor.visit(&self.len)?;
            cursor.visit(&self.value)?;
            cursor.finish()
        }
    }

    /// SunSpec-like map: marker, model, end marker.
    struct Map {
        suns: RO<RegU16Array<2>>,
        model: Inner,
        end: RO<RegU16Array<2>>,
    }

    impl RegisterData for Map {
        fn register_span(&self) -> usize {
            self.suns.register_span() + self.model.register_span() + self.end.register_span()
        }
    }

    impl ReadRegisterData for Map {
        fn read(&self, offset: usize, buf: &mut [u16]) -> RegisterResult<()> {
            let mut cursor = ReadCursor::new(offset, buf);
            for reg in [&self.suns as &dyn ReadRegisterData, &self.model, &self.end] {
                cursor.visit(reg)?;
            }
            cursor.finish()
        }
    }

    fn map() -> Map {
        Map {
            suns: RO::new(RegU16Array::new([0x5375, 0x6E53])),
            model: Inner {
                id: RO::new(RegU16::new(1)),
                len: RO::new(RegU16::new(2)),
                value: RW::new(RegU32::new(0x11223344)),
            },
            end: RO::new(RegU16Array::new([0xFFFF, 0x0000])),
        }
    }

    #[test]
    fn read_all_001() {
        let map = map();
        assert_eq!(map.register_span(), 8);

        let mut buf = [0u16; 8];
        assert_eq!(map.read(0, &mut buf), Ok(()));
        assert_eq!(buf, [0x5375, 0x6E53, 1, 2, 0x1122, 0x3344, 0xFFFF, 0x0000]);
    }

    #[test]
    fn read_within_register_001() {
        let map = map();

        let mut buf = [0u16; 1];
        assert_eq!(map.read(1, &mut buf), Ok(()));
        assert_eq!(buf, [0x6E53]);

        let mut buf = [0u16; 2];
        assert_eq!(map.read(4, &mut buf), Ok(()));
        assert_eq!(buf, [0x1122, 0x3344]);
    }

    #[test]
    fn read_across_registers_001() {
        let map = map();

        // suns -> model
        let mut buf = [0u16; 3];
        assert_eq!(map.read(1, &mut buf), Ok(()));
        assert_eq!(buf, [0x6E53, 1, 2]);

        // model -> end
        let mut buf = [0u16; 3];
        assert_eq!(map.read(4, &mut buf), Ok(()));
        assert_eq!(buf, [0x1122, 0x3344, 0xFFFF]);

        // suns -> model -> end
        let mut buf = [0u16; 6];
        assert_eq!(map.read(1, &mut buf), Ok(()));
        assert_eq!(buf, [0x6E53, 1, 2, 0x1122, 0x3344, 0xFFFF]);
    }

    #[test]
    fn read_partial_access_001() {
        let map = map();

        let mut buf = [0u16; 2];
        assert_eq!(map.read(5, &mut buf), Err(RegisterError::PartialAccess));

        let mut buf = [0u16; 3];
        assert_eq!(map.read(2, &mut buf), Err(RegisterError::PartialAccess));
    }

    #[test]
    fn read_out_of_bounds_001() {
        let map = map();

        let mut buf = [0u16; 2];
        assert_eq!(map.read(7, &mut buf), Err(RegisterError::OutOfBounds { offset: 7 }));

        let mut buf = [0u16; 9];
        assert_eq!(map.read(0, &mut buf), Err(RegisterError::OutOfBounds { offset: 0 }));

        let mut buf = [0u16; 1];
        assert_eq!(map.read(8, &mut buf), Err(RegisterError::OutOfBounds { offset: 8 }));
    }

    #[test]
    fn read_empty_buffer_001() {
        let map = map();
        let mut buf = [0u16; 0];

        assert_eq!(map.read(3, &mut buf), Ok(()));
        assert_eq!(map.read(8, &mut buf), Ok(()));
        assert_eq!(map.read(9, &mut buf), Err(RegisterError::OutOfBounds { offset: 9 }));
    }

    #[test]
    fn read_at_start_address_001() {
        let map = At::new(40000, map());

        let mut buf = [0u16; 4];
        assert_eq!(map.read(40000, &mut buf), Ok(()));
        assert_eq!(buf, [0x5375, 0x6E53, 1, 2]);

        assert_eq!(map.read(39999, &mut buf), Err(RegisterError::OutOfBounds { offset: 39999 }));
    }
}
