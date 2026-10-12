//! Minimal Modbus/TCP server on port 1502.
//!
//! Serves 100 holding registers starting at address 0 and supports
//! read holding registers (0x03), write single register (0x06) and
//! write multiple registers (0x10).

use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};

use modbus_stack::{
    encode_exception, read_holding_registers, write_multiple_registers, write_single_register, DecodeError, Registers,
};
use modbus_stack::tcp::{Frame, FrameParser};
use modbus_stack::{ExceptionCode, FunctionCode, FunctionKind};
use modbus_stack::register::{ReadRegisterData, WriteRegisterData};

const REGISTER_COUNT: usize = 100;

fn main() -> std::io::Result<()> {
    let listener = TcpListener::bind("0.0.0.0:1502")?;
    let mut registers = [0u16; REGISTER_COUNT];
    for (i, register) in registers.iter_mut().enumerate() {
        *register = i as u16;
    }

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                if let Err(err) = serve(stream, &mut registers) {
                    println!("Connection error: {err}");
                }
            }
            Err(err) => println!("Accept error: {err}"),
        }
    }
    Ok(())
}

fn serve(mut stream: TcpStream, registers: &mut [u16; REGISTER_COUNT]) -> std::io::Result<()> {
    println!("Connected: {}", stream.peer_addr()?);

    let mut parser = FrameParser::new();
    let mut buffer = [0u8; 1024];

    loop {
        let size = stream.read(&mut buffer)?;
        if size == 0 {
            println!("Disconnected");
            return Ok(());
        }

        for frame in parser.frames(&buffer[..size]) {
            let mut frame = match frame {
                Ok(frame) => frame,
                Err(err) => {
                    println!("Parse error: {err:?}, closing connection");
                    return Ok(());
                }
            };

            println!(
                "Request: tid={} unit={} fc={:?}",
                frame.transaction_id(),
                frame.unit_id(),
                frame.function_code()
            );

            handle(&mut frame, registers);
            stream.write_all(frame.as_bytes())?;
        }
    }
}

/// Answers the request in place.
fn handle(frame: &mut Frame, registers: &mut [u16; REGISTER_COUNT]) {
    let mut buffer = [0u16; read_holding_registers::Request::MAX_QUANTITY_OF_REGISTERS as usize];

    match frame.function_code() {
        FunctionKind::Normal(FunctionCode::ReadHoldingRegisters) => {
            let response = match read_holding_registers::Request::decode(frame) {
                Ok(request) => {
                    let start = request.starting_address() as usize;
                    let values = &mut buffer[..request.quantity_of_registers() as usize];
                    match registers.as_slice().read(start, values) {
                        Ok(()) => read_holding_registers::Response::new(values),
                        Err(err) => read_holding_registers::Response::new_exception(err.into()),
                    }
                }
                Err(DecodeError::Malformed(err)) => read_holding_registers::Response::new_exception(err.into()),
                Err(_) => read_holding_registers::Response::new_exception(ExceptionCode::IllegalFunction),
            };
            response.encode(frame).expect("response fits into frame");
        }
        FunctionKind::Normal(FunctionCode::WriteSingleRegister) => {
            let response = match write_single_register::Request::decode(frame) {
                Ok(request) => {
                    let start = request.register_address() as usize;
                    let value = request.register_value();
                    match registers.as_mut_slice().write(start, Registers::from(core::slice::from_ref(&value))) {
                        Ok(()) => write_single_register::Response::new(request.register_address(), value),
                        Err(err) => write_single_register::Response::new_exception(err.into()),
                    }
                }
                Err(DecodeError::Malformed(err)) => write_single_register::Response::new_exception(err.into()),
                Err(_) => write_single_register::Response::new_exception(ExceptionCode::IllegalFunction),
            };
            response.encode(frame).expect("response fits into frame");
        }
        FunctionKind::Normal(FunctionCode::WriteMultipleRegisters) => {
            let response = match write_multiple_registers::Request::decode(frame) {
                Ok(request) => {
                    let start = request.starting_address() as usize;
                    match registers.as_mut_slice().write(start, request.registers()) {
                        Ok(()) => write_multiple_registers::Response::new(
                            request.starting_address(),
                            request.quantity_of_registers(),
                        ),
                        Err(err) => write_multiple_registers::Response::new_exception(err.into()),
                    }
                }
                Err(DecodeError::Malformed(err)) => write_multiple_registers::Response::new_exception(err.into()),
                Err(_) => write_multiple_registers::Response::new_exception(ExceptionCode::IllegalFunction),
            };
            response.encode(frame).expect("response fits into frame");
        }
        FunctionKind::Normal(code) | FunctionKind::Exception(code) => {
            encode_exception(frame, code, ExceptionCode::IllegalFunction).expect("exception fits into frame");
        }
    }
}
