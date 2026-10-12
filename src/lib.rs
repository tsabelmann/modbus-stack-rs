#![no_std]

#[cfg(test)]
#[macro_use]
extern crate std;

/// Modbus related constants for frame size, number of registers, etc.
pub mod constants;

/// Function related types, grouped by function code.
mod function;
pub use function::{
    read_holding_registers, read_input_registers, write_single_register, write_multiple_registers,
    DecodeError, Malformed, Registers,
};

pub mod encode;

/// Register related types and helper functions.
pub mod register;

/// Function code related types.
mod codes;
pub use codes::{FunctionCode, FunctionKind};

/// Exception code related types and helper functions.
mod exception;
pub use exception::ExceptionCode;

/// ADU related types.
mod adu;
pub use adu::AduData;

/// PDU related types.
mod pdu;
pub use pdu::PduData;

/// TCP/IP related types and helper functions.
pub mod tcp;

/// RTU related types and helper functions.
pub mod rtu;