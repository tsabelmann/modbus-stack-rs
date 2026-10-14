
/// Modbus register that can be converter to and from [i8].
#[derive(Debug, PartialEq, Default, Clone)]
pub struct RegI8 {
    data: [u16; 1]
}

impl RegI8 {
    /// Creates a new [i8] based modbus register.
    /// 
    /// # Example
    ///
    /// ```
    /// use modbus_stack::register::RegI8;
    /// 
    /// let reg = RegI8::new(42);
    /// ````
    pub const fn new(value: i8) -> RegI8 {
        RegI8 {
            data: [(value as u16) & 0xFF]
        }
    }
}

impl From<i8> for RegI8 {
    fn from(value: i8) -> Self {
        RegI8::new(value)
    }
}

impl From<&i8> for RegI8 {
    fn from(value: &i8) -> Self {
        RegI8::new(*value)
    }
}

impl From<RegI8> for i8 {
    fn from(value: RegI8) -> Self {
        i8::from(&value)
    }
}

impl From<&RegI8> for i8 {
    fn from(value: &RegI8) -> Self {
        (value.data[0] & 0xFF) as i8
    }
}

#[cfg(test)]
mod reg_i8_tests {
    use super::*;

    #[test]
    fn from_to_i8_001() {
        let reg = RegI8::new(-1);
        let value = i8::from(reg);
        assert_eq!(value, -1);
    }

}