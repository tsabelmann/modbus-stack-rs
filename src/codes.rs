#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FunctionKind {
    /// "Function Code" for encoding valid requests and responses.
    Normal(FunctionCode),
    /// "Function Code" for encoding responses to invalid requests.
    Exception(FunctionCode),
}

impl FunctionKind {
    pub const fn new_normal(code: FunctionCode) -> FunctionKind {
        FunctionKind::Normal(code)
    }

    pub const fn new_exception(code: FunctionCode) -> FunctionKind {
        FunctionKind::Exception(code)
    }

    pub const fn is_exception(&self) -> bool {
        matches!(self, FunctionKind::Exception(_))
    }

    pub const fn is_normal(&self) -> bool {
        matches!(self, FunctionKind::Normal(_))
    }

    pub const fn function_code(&self) -> FunctionCode {
        match self {
            FunctionKind::Normal(code) => *code,
            FunctionKind::Exception(code) => *code
        }
    }
}

#[non_exhaustive]
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FunctionCode {
    ReadCoils = 1,
    ReadDiscreteInputs = 2,
    ReadHoldingRegisters = 3,
    ReadInputRegisters = 4,
    WriteSingleCoil = 5,
    WriteSingleRegister = 6,
    ReadExceptionStatus = 7,
    Diagnostic = 8,
    GetComEventCounter = 11,
    GetComEventLog = 12,
    WriteMultipleCoils = 15,
    WriteMultipleRegisters = 16,
    ReportServerId = 17,
    ReadFileRecord = 20,
    WriteFileRecord = 21, 
    MaskWriteRegister = 22,
    ReadWriteMultipleRegisters = 23,
    ReadFifoQueue = 24,
    Unknown(u8),
}

impl From<FunctionCode> for u8 {
    fn from(value: FunctionCode) -> Self {
        match value {
            FunctionCode::ReadCoils => 1,
            FunctionCode::ReadDiscreteInputs => 2,
            FunctionCode::ReadHoldingRegisters => 3,
            FunctionCode::ReadInputRegisters => 4,
            FunctionCode::WriteSingleCoil => 5,
            FunctionCode::WriteSingleRegister => 6,
            FunctionCode::ReadExceptionStatus => 7,
            FunctionCode::Diagnostic => 8,
            FunctionCode::GetComEventCounter => 11,
            FunctionCode::GetComEventLog => 12,
            FunctionCode::WriteMultipleCoils => 15,
            FunctionCode::WriteMultipleRegisters => 16,
            FunctionCode::ReportServerId => 17,
            FunctionCode::ReadFileRecord => 20,
            FunctionCode::WriteFileRecord => 21,
            FunctionCode::MaskWriteRegister => 22,
            FunctionCode::ReadWriteMultipleRegisters => 23,
            FunctionCode::ReadFifoQueue => 24,
            FunctionCode::Unknown(value) => value & 0x7F,
        }
    }
}

impl From<u8> for FunctionCode {
    fn from(value: u8) -> Self {
        match value & 0x7F {
            1 => FunctionCode::ReadCoils,
            2 => FunctionCode::ReadDiscreteInputs,
            3 => FunctionCode::ReadHoldingRegisters,
            4 => FunctionCode::ReadInputRegisters,
            5 => FunctionCode::WriteSingleCoil,
            6 => FunctionCode::WriteSingleRegister,
            7 => FunctionCode::ReadExceptionStatus,
            8 => FunctionCode::Diagnostic,
            11 => FunctionCode::GetComEventCounter,
            12 => FunctionCode::GetComEventLog,
            15 => FunctionCode::WriteMultipleCoils,
            16 => FunctionCode::WriteMultipleRegisters,
            17 => FunctionCode::ReportServerId,
            20 => FunctionCode::ReadFileRecord,
            21 => FunctionCode::WriteFileRecord,
            22 => FunctionCode::MaskWriteRegister,
            23 => FunctionCode::ReadWriteMultipleRegisters,
            24 => FunctionCode::ReadFifoQueue,
            _ => FunctionCode::Unknown(value),
        }
    }
}

impl From<FunctionKind> for u8 {
    fn from(value: FunctionKind) -> Self {
        match value {
            FunctionKind::Normal(code) => {
                let value = u8::from(code);
                value & 0x7F
            },
            FunctionKind::Exception(code) => {
                let value = u8::from(code);
                value | 0x80
            }
        }
    }
}

impl From<u8> for FunctionKind {
    fn from(value: u8) -> Self {
        if value & 0x80 != 0 {
            let code = FunctionCode::from(value);
            FunctionKind::Exception(code)
        } else {
            let code = FunctionCode::from(value);
            FunctionKind::Normal(code)
        }
    }
}

#[cfg(test)]
mod codes_tests {
    use super::*;

    #[test]
    fn function_kind_001() {
        let code = FunctionCode::ReadCoils;
        let normal = FunctionKind::new_normal(code);
        let exception = FunctionKind::new_exception(code);

        let normal_value = u8::from(normal);
        let exeception_value = u8::from(exception);

        assert_eq!(normal_value, u8::from(code));
        assert_eq!(exeception_value, u8::from(code) | 0x80);
        assert!(exeception_value > normal_value);
        assert!(exeception_value - normal_value == 0x80);
    }

    #[test]
    fn function_kind_002() {
        let code = FunctionCode::ReadDiscreteInputs;
        let normal = FunctionKind::new_normal(code);
        let exception = FunctionKind::new_exception(code);

        let normal_value = u8::from(normal);
        let exeception_value = u8::from(exception);

        assert_eq!(normal_value, u8::from(code));
        assert_eq!(exeception_value, u8::from(code) | 0x80);
        assert!(exeception_value > normal_value);
        assert!(exeception_value - normal_value == 0x80);
    }

    #[test]
    fn function_kind_003() {
        let code = FunctionCode::ReadHoldingRegisters;
        let normal = FunctionKind::new_normal(code);
        let exception = FunctionKind::new_exception(code);

        let normal_value = u8::from(normal);
        let exeception_value = u8::from(exception);

        assert_eq!(normal_value, u8::from(code));
        assert_eq!(exeception_value, u8::from(code) | 0x80);
        assert!(exeception_value > normal_value);
        assert!(exeception_value - normal_value == 0x80);
    }

    #[test]
    fn function_kind_004() {
        let code = FunctionCode::ReadInputRegisters;
        let normal = FunctionKind::new_normal(code);
        let exception = FunctionKind::new_exception(code);

        let normal_value = u8::from(normal);
        let exeception_value = u8::from(exception);

        assert_eq!(normal_value, u8::from(code));
        assert_eq!(exeception_value, u8::from(code) | 0x80);
        assert!(exeception_value > normal_value);
        assert!(exeception_value - normal_value == 0x80);
    }

    #[test]
    fn function_kind_005() {
        let code = FunctionCode::WriteSingleCoil;
        let normal = FunctionKind::new_normal(code);
        let exception = FunctionKind::new_exception(code);

        let normal_value = u8::from(normal);
        let exeception_value = u8::from(exception);

        assert_eq!(normal_value, u8::from(code));
        assert_eq!(exeception_value, u8::from(code) | 0x80);
        assert!(exeception_value > normal_value);
        assert!(exeception_value - normal_value == 0x80);
    }

    #[test]
    fn function_kind_006() {
        let code = FunctionCode::WriteSingleRegister;
        let normal = FunctionKind::new_normal(code);
        let exception = FunctionKind::new_exception(code);

        let normal_value = u8::from(normal);
        let exeception_value = u8::from(exception);

        assert_eq!(normal_value, u8::from(code));
        assert_eq!(exeception_value, u8::from(code) | 0x80);
        assert!(exeception_value > normal_value);
        assert!(exeception_value - normal_value == 0x80);
    }

    #[test]
    fn function_kind_007() {
        let code = FunctionCode::ReadExceptionStatus;
        let normal = FunctionKind::new_normal(code);
        let exception = FunctionKind::new_exception(code);

        let normal_value = u8::from(normal);
        let exeception_value = u8::from(exception);

        assert_eq!(normal_value, u8::from(code));
        assert_eq!(exeception_value, u8::from(code) | 0x80);
        assert!(exeception_value > normal_value);
        assert!(exeception_value - normal_value == 0x80);
    }

    #[test]
    fn function_kind_008() {
        let code = FunctionCode::Diagnostic;
        let normal = FunctionKind::new_normal(code);
        let exception = FunctionKind::new_exception(code);

        let normal_value = u8::from(normal);
        let exeception_value = u8::from(exception);

        assert_eq!(normal_value, u8::from(code));
        assert_eq!(exeception_value, u8::from(code) | 0x80);
        assert!(exeception_value > normal_value);
        assert!(exeception_value - normal_value == 0x80);
    }

    #[test]
    fn function_kind_009() {
        let code = FunctionCode::GetComEventCounter;
        let normal = FunctionKind::new_normal(code);
        let exception = FunctionKind::new_exception(code);

        let normal_value = u8::from(normal);
        let exeception_value = u8::from(exception);

        assert_eq!(normal_value, u8::from(code));
        assert_eq!(exeception_value, u8::from(code) | 0x80);
        assert!(exeception_value > normal_value);
        assert!(exeception_value - normal_value == 0x80);
    }

    #[test]
    fn function_kind_010() {
        let code = FunctionCode::GetComEventLog;
        let normal = FunctionKind::new_normal(code);
        let exception = FunctionKind::new_exception(code);

        let normal_value = u8::from(normal);
        let exeception_value = u8::from(exception);

        assert_eq!(normal_value, u8::from(code));
        assert_eq!(exeception_value, u8::from(code) | 0x80);
        assert!(exeception_value > normal_value);
        assert!(exeception_value - normal_value == 0x80);
    }

    #[test]
    fn function_kind_011() {
        let code = FunctionCode::WriteMultipleCoils;
        let normal = FunctionKind::new_normal(code);
        let exception = FunctionKind::new_exception(code);

        let normal_value = u8::from(normal);
        let exeception_value = u8::from(exception);

        assert_eq!(normal_value, u8::from(code));
        assert_eq!(exeception_value, u8::from(code) | 0x80);
        assert!(exeception_value > normal_value);
        assert!(exeception_value - normal_value == 0x80);
    }

    #[test]
    fn function_kind_012() {
        let code = FunctionCode::WriteMultipleRegisters;
        let normal = FunctionKind::new_normal(code);
        let exception = FunctionKind::new_exception(code);

        let normal_value = u8::from(normal);
        let exeception_value = u8::from(exception);

        assert_eq!(normal_value, u8::from(code));
        assert_eq!(exeception_value, u8::from(code) | 0x80);
        assert!(exeception_value > normal_value);
        assert!(exeception_value - normal_value == 0x80);
    }

    #[test]
    fn function_kind_013() {
        let code = FunctionCode::ReportServerId;
        let normal = FunctionKind::new_normal(code);
        let exception = FunctionKind::new_exception(code);

        let normal_value = u8::from(normal);
        let exeception_value = u8::from(exception);

        assert_eq!(normal_value, u8::from(code));
        assert_eq!(exeception_value, u8::from(code) | 0x80);
        assert!(exeception_value > normal_value);
        assert!(exeception_value - normal_value == 0x80);
    }

    #[test]
    fn function_kind_014() {
        let code = FunctionCode::ReadFileRecord;
        let normal = FunctionKind::new_normal(code);
        let exception = FunctionKind::new_exception(code);

        let normal_value = u8::from(normal);
        let exeception_value = u8::from(exception);

        assert_eq!(normal_value, u8::from(code));
        assert_eq!(exeception_value, u8::from(code) | 0x80);
        assert!(exeception_value > normal_value);
        assert!(exeception_value - normal_value == 0x80);
    }

    #[test]
    fn function_kind_015() {
        let code = FunctionCode::WriteFileRecord;
        let normal = FunctionKind::new_normal(code);
        let exception = FunctionKind::new_exception(code);

        let normal_value = u8::from(normal);
        let exeception_value = u8::from(exception);

        assert_eq!(normal_value, u8::from(code));
        assert_eq!(exeception_value, u8::from(code) | 0x80);
        assert!(exeception_value > normal_value);
        assert!(exeception_value - normal_value == 0x80);
    }

    #[test]
    fn function_kind_016() {
        let code = FunctionCode::MaskWriteRegister;
        let normal = FunctionKind::new_normal(code);
        let exception = FunctionKind::new_exception(code);

        let normal_value = u8::from(normal);
        let exeception_value = u8::from(exception);

        assert_eq!(normal_value, u8::from(code));
        assert_eq!(exeception_value, u8::from(code) | 0x80);
        assert!(exeception_value > normal_value);
        assert!(exeception_value - normal_value == 0x80);
    }

    #[test]
    fn function_kind_017() {
        let code = FunctionCode::ReadWriteMultipleRegisters;
        let normal = FunctionKind::new_normal(code);
        let exception = FunctionKind::new_exception(code);

        let normal_value = u8::from(normal);
        let exeception_value = u8::from(exception);

        assert_eq!(normal_value, u8::from(code));
        assert_eq!(exeception_value, u8::from(code) | 0x80);
        assert!(exeception_value > normal_value);
        assert!(exeception_value - normal_value == 0x80);
    }

    #[test]
    fn function_kind_018() {
        let code = FunctionCode::ReadFifoQueue;
        let normal = FunctionKind::new_normal(code);
        let exception = FunctionKind::new_exception(code);

        let normal_value = u8::from(normal);
        let exeception_value = u8::from(exception);

        assert_eq!(normal_value, u8::from(code));
        assert_eq!(exeception_value, u8::from(code) | 0x80);
        assert!(exeception_value > normal_value);
        assert!(exeception_value - normal_value == 0x80);
    }

    #[test]
    fn function_kind_019() {
        let code = FunctionCode::Unknown(43);
        let normal = FunctionKind::new_normal(code);
        let exception = FunctionKind::new_exception(code);

        let normal_value = u8::from(normal);
        let exeception_value = u8::from(exception);

        assert_eq!(normal_value, u8::from(code));
        assert_eq!(exeception_value, u8::from(code) | 0x80);
        assert!(exeception_value > normal_value);
        assert!(exeception_value - normal_value == 0x80);
    }
}