//! Minimal Modbus/TCP server on port 1502.
//!
//! Serves 100 holding registers starting at address 0 and supports
//! read holding registers (0x03) and write single register (0x06).

use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};

use modbus_stack::decode::request::{read_holding_registers, write_single_register};
use modbus_stack::encode::FrameEncoder;
use modbus_stack::encode::response;
use modbus_stack::tcp::{Frame, FrameParser};
use modbus_stack::{ExceptionCode, FunctionCode, FunctionKind};

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
    match frame.function_code() {
        FunctionKind::Normal(FunctionCode::ReadHoldingRegisters) => {
            let response = match read_holding_registers::Request::new(frame) {
                Ok(request) => {
                    let start = request.starting_address() as usize;
                    let end = start + request.quantity_of_registers() as usize;
                    match registers.get(start..end) {
                        Some(values) => response::read_holding_registers::Response::new(values),
                        None => response::read_holding_registers::Response::new_exception(ExceptionCode::IllegalDataAddress),
                    }
                }
                Err(_) => response::read_holding_registers::Response::new_exception(ExceptionCode::IllegalDataValue),
            };
            response.encode(frame).expect("response fits into frame");
        }
        FunctionKind::Normal(FunctionCode::WriteSingleRegister) => {
            let response = match write_single_register::Request::new(frame) {
                Ok(request) => match registers.get_mut(request.register_address() as usize) {
                    Some(register) => {
                        *register = request.register_value();
                        response::write_single_register::Response::new(request.register_address(), request.register_value())
                    }
                    None => response::write_single_register::Response::new_exception(ExceptionCode::IllegalDataAddress),
                },
                Err(_) => response::write_single_register::Response::new_exception(ExceptionCode::IllegalDataValue),
            };
            response.encode(frame).expect("response fits into frame");
        }
        FunctionKind::Normal(code) | FunctionKind::Exception(code) => {
            frame.payload_mut()[0] = u8::from(ExceptionCode::IllegalFunction);
            frame.finalize(FunctionKind::Exception(code), 1);
        }
    }
}
