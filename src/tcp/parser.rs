use crate::tcp::frame::Frame;

// ── ParseError ──

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum ParseError {
    InvalidProtocolIdentifier,
    InvalidLength,
}

// ── FrameParser ──

/// Incremental Modbus/TCP frame parser.
///
/// Bytes are collected in an internal buffer; completed frames are returned by value.
/// Errors are sticky until [`FrameParser::reset`] is called, since a TCP stream cannot
/// be resynchronized and the connection should be closed.
#[derive(Debug, Clone)]
pub struct FrameParser {
    buf: [u8; Frame::MAX_SIZE],
    filled: u16,
    error: Option<ParseError>,
}

impl FrameParser {
    /// Minimum value of the MBAP length field (unit_id + function_code).
    const MIN_LENGTH: usize = 2;

    /// Maximum value of the MBAP length field (unit_id + 253 bytes PDU).
    const MAX_LENGTH: usize = Frame::MAX_SIZE - Frame::HEADER_SIZE + 1;

    pub const fn new() -> FrameParser {
        FrameParser {
            buf: [0u8; Frame::MAX_SIZE],
            filled: 0,
            error: None,
        }
    }

    /// Pushes a single byte; returns the frame once it is complete.
    pub fn push(&mut self, byte: u8) -> Result<Option<Frame>, ParseError> {
        self.push_slice(&[byte]).1
    }

    /// Pushes a chunk of bytes and consumes at most one frame.
    ///
    /// Returns the number of consumed bytes and the parse result. Remaining bytes
    /// belong to the next frame and have to be pushed again.
    pub fn push_slice(&mut self, input: &[u8]) -> (usize, Result<Option<Frame>, ParseError>) {
        if let Some(error) = self.error {
            return (0, Err(error));
        }

        let mut filled = self.filled as usize;
        let mut consumed = 0;

        // collect header
        if filled < Frame::HEADER_SIZE {
            consumed = (Frame::HEADER_SIZE - filled).min(input.len());
            self.buf[filled..filled + consumed].copy_from_slice(&input[..consumed]);
            filled += consumed;
            self.filled = filled as u16;

            if filled < Frame::HEADER_SIZE {
                return (consumed, Ok(None));
            }

            if let Err(error) = self.validate_header() {
                self.error = Some(error);
                return (consumed, Err(error));
            }
        }

        // collect payload
        let total = self.total_length();
        let rest = &input[consumed..];
        let n = (total - filled).min(rest.len());
        self.buf[filled..filled + n].copy_from_slice(&rest[..n]);
        filled += n;
        consumed += n;

        if filled < total {
            self.filled = filled as u16;
            return (consumed, Ok(None));
        }

        self.filled = 0;
        (consumed, Ok(Some(Frame::from_valid(&self.buf[..total]))))
    }

    /// Returns an iterator over all frames completed by `input`.
    pub fn frames<'a>(&'a mut self, input: &'a [u8]) -> Frames<'a> {
        Frames {
            parser: self,
            input,
            done: false,
        }
    }

    pub const fn reset(&mut self) {
        self.filled = 0;
        self.error = None;
    }

    /// Returns `true` if no partial frame is pending and no error occurred.
    pub const fn is_idle(&self) -> bool {
        self.filled == 0 && self.error.is_none()
    }

    fn validate_header(&self) -> Result<(), ParseError> {
        // check for correct protocol identifier
        if self.buf[2] != 0 || self.buf[3] != 0 {
            return Err(ParseError::InvalidProtocolIdentifier);
        }

        // check for correct length
        let length = u16::from_be_bytes([self.buf[4], self.buf[5]]) as usize;
        if !(Self::MIN_LENGTH..=Self::MAX_LENGTH).contains(&length) {
            return Err(ParseError::InvalidLength);
        }

        Ok(())
    }

    /// Total frame length, only valid after the header was validated.
    fn total_length(&self) -> usize {
        Frame::HEADER_SIZE - 1 + u16::from_be_bytes([self.buf[4], self.buf[5]]) as usize
    }
}

impl Default for FrameParser {
    fn default() -> Self {
        FrameParser::new()
    }
}

// ── Frames ──

/// Iterator over all frames completed by a chunk of bytes, see [`FrameParser::frames`].
pub struct Frames<'a> {
    parser: &'a mut FrameParser,
    input: &'a [u8],
    done: bool,
}

impl<'a> Iterator for Frames<'a> {
    type Item = Result<Frame, ParseError>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.done {
            return None;
        }

        let (consumed, result) = self.parser.push_slice(self.input);
        self.input = &self.input[consumed..];

        match result {
            Ok(Some(frame)) => Some(Ok(frame)),
            Ok(None) => {
                self.done = true;
                None
            }
            Err(error) => {
                self.done = true;
                Some(Err(error))
            }
        }
    }
}

#[cfg(test)]
mod parser_tests {
    use super::*;
    use crate::{FunctionCode, FunctionKind};
    use std::vec::Vec;

    const READ_HOLDING_REGISTERS: [u8; 12] = [
        0x00, 0x01, // Transaction ID
        0x00, 0x00, // Protocol ID
        0x00, 0x06, // Length
        0x11, // Unit ID
        0x03, // Function code
        0x01, 0x2B, 0x00, 0x03, // Data
    ];

    const EXCEPTION: [u8; 9] = [0x00, 0x02, 0x00, 0x00, 0x00, 0x03, 0x22, 0x83, 0x01];

    // Helper to feed bytes one by one and return the last result
    fn feed_bytes(parser: &mut FrameParser, bytes: &[u8]) -> Result<Option<Frame>, ParseError> {
        let mut result = Ok(None);
        for &byte in bytes {
            result = parser.push(byte);
        }
        result
    }

    fn header(length: u16) -> [u8; 7] {
        let bytes = length.to_be_bytes();
        [0x00, 0x01, 0x00, 0x00, bytes[0], bytes[1], 0x11]
    }

    // ── Happy path ──

    #[test]
    fn decode_read_holding_registers_request() {
        let mut parser = FrameParser::new();
        let frame = feed_bytes(&mut parser, &READ_HOLDING_REGISTERS)
            .unwrap()
            .unwrap();

        assert_eq!(frame.transaction_id(), 0x0001);
        assert_eq!(frame.protocol_id(), 0x0000);
        assert_eq!(frame.length(), 0x0006);
        assert_eq!(frame.unit_id(), 0x11);
        assert_eq!(
            frame.function_code(),
            FunctionKind::Normal(FunctionCode::ReadHoldingRegisters)
        );
        assert_eq!(frame.as_bytes(), &READ_HOLDING_REGISTERS);
    }

    #[test]
    fn decode_exception_response() {
        let mut parser = FrameParser::new();
        let frame = feed_bytes(&mut parser, &EXCEPTION).unwrap().unwrap();

        assert_eq!(frame.length(), 0x0003);
        assert_eq!(
            frame.function_code(),
            FunctionKind::Exception(FunctionCode::ReadHoldingRegisters)
        );
    }

    #[test]
    fn decode_minimum_length_frame() {
        let mut parser = FrameParser::new();

        // Length = 2: unit_id + function_code, no payload
        let received = [0x00, 0x01, 0x00, 0x00, 0x00, 0x02, 0x11, 0x03];
        let frame = feed_bytes(&mut parser, &received).unwrap().unwrap();
        assert_eq!(frame.pdu(), &[0x03]);
    }

    #[test]
    fn decode_maximum_length_frame() {
        let mut parser = FrameParser::new();

        let mut received = [0xAAu8; Frame::MAX_SIZE];
        received[..7].copy_from_slice(&header(254));

        let (consumed, result) = parser.push_slice(&received);
        let frame = result.unwrap().unwrap();
        assert_eq!(consumed, Frame::MAX_SIZE);
        assert_eq!(frame.as_bytes().len(), Frame::MAX_SIZE);
        assert_eq!(frame.pdu().len(), 253);
    }

    #[test]
    fn decode_two_frames_sequentially() {
        let mut parser = FrameParser::new();

        let frame = feed_bytes(&mut parser, &READ_HOLDING_REGISTERS)
            .unwrap()
            .unwrap();
        assert_eq!(frame.transaction_id(), 0x0001);
        assert_eq!(frame.unit_id(), 0x11);

        let frame = feed_bytes(&mut parser, &EXCEPTION).unwrap().unwrap();
        assert_eq!(frame.transaction_id(), 0x0002);
        assert_eq!(frame.unit_id(), 0x22);
    }

    #[test]
    fn frame_outlives_further_parsing() {
        let mut parser = FrameParser::new();

        let first = feed_bytes(&mut parser, &READ_HOLDING_REGISTERS)
            .unwrap()
            .unwrap();
        let second = feed_bytes(&mut parser, &EXCEPTION).unwrap().unwrap();

        assert_eq!(first.as_bytes(), &READ_HOLDING_REGISTERS);
        assert_eq!(second.as_bytes(), &EXCEPTION);
    }

    // ── Chunks ──

    #[test]
    fn push_slice_consumes_one_frame() {
        let mut parser = FrameParser::new();

        let mut input = Vec::new();
        input.extend_from_slice(&READ_HOLDING_REGISTERS);
        input.extend_from_slice(&EXCEPTION);

        let (consumed, result) = parser.push_slice(&input);
        assert_eq!(consumed, READ_HOLDING_REGISTERS.len());
        assert_eq!(result.unwrap().unwrap().as_bytes(), &READ_HOLDING_REGISTERS);

        let (consumed, result) = parser.push_slice(&input[consumed..]);
        assert_eq!(consumed, EXCEPTION.len());
        assert_eq!(result.unwrap().unwrap().as_bytes(), &EXCEPTION);
        assert!(parser.is_idle());
    }

    #[test]
    fn frames_iterator_yields_all_frames() {
        let mut parser = FrameParser::new();

        let mut input = Vec::new();
        input.extend_from_slice(&READ_HOLDING_REGISTERS);
        input.extend_from_slice(&EXCEPTION);
        input.extend_from_slice(&READ_HOLDING_REGISTERS[..4]);

        let frames: Vec<_> = parser.frames(&input).collect();
        assert_eq!(frames.len(), 2);
        assert_eq!(
            frames[0].as_ref().unwrap().as_bytes(),
            &READ_HOLDING_REGISTERS
        );
        assert_eq!(frames[1].as_ref().unwrap().as_bytes(), &EXCEPTION);

        // partial third frame is completed by the next chunk
        let frames: Vec<_> = parser.frames(&READ_HOLDING_REGISTERS[4..]).collect();
        assert_eq!(frames.len(), 1);
        assert_eq!(
            frames[0].as_ref().unwrap().as_bytes(),
            &READ_HOLDING_REGISTERS
        );
    }

    #[test]
    fn frames_iterator_stops_after_error() {
        let mut parser = FrameParser::new();

        let mut input = Vec::new();
        input.extend_from_slice(&READ_HOLDING_REGISTERS);
        input.extend_from_slice(&[0x00, 0x01, 0x00, 0x01, 0x00, 0x06, 0x11]);
        input.extend_from_slice(&EXCEPTION);

        let frames: Vec<_> = parser.frames(&input).collect();
        assert_eq!(frames.len(), 2);
        assert!(frames[0].is_ok());
        assert_eq!(frames[1], Err(ParseError::InvalidProtocolIdentifier));
    }

    #[test]
    fn arbitrary_chunk_boundaries() {
        let mut input = Vec::new();
        input.extend_from_slice(&READ_HOLDING_REGISTERS);
        input.extend_from_slice(&EXCEPTION);

        for chunk_size in 1..=input.len() {
            let mut parser = FrameParser::new();
            let mut frames = Vec::new();
            for chunk in input.chunks(chunk_size) {
                for frame in parser.frames(chunk) {
                    frames.push(frame.unwrap());
                }
            }
            assert_eq!(frames.len(), 2, "chunk size {}", chunk_size);
            assert_eq!(frames[0].as_bytes(), &READ_HOLDING_REGISTERS);
            assert_eq!(frames[1].as_bytes(), &EXCEPTION);
        }
    }

    // ── Pending ──

    #[test]
    fn partial_header_returns_pending() {
        let mut parser = FrameParser::new();
        assert_eq!(
            feed_bytes(&mut parser, &READ_HOLDING_REGISTERS[..4]),
            Ok(None)
        );
    }

    #[test]
    fn partial_payload_returns_pending() {
        let mut parser = FrameParser::new();
        assert_eq!(
            feed_bytes(&mut parser, &READ_HOLDING_REGISTERS[..9]),
            Ok(None)
        );
    }

    #[test]
    fn empty_slice_returns_pending() {
        let mut parser = FrameParser::new();
        assert_eq!(parser.push_slice(&[]), (0, Ok(None)));
        assert_eq!(parser.frames(&[]).count(), 0);
    }

    #[test]
    fn only_last_byte_returns_frame() {
        let mut parser = FrameParser::new();

        let mut frames = 0;
        let mut pending = 0;
        for &byte in &EXCEPTION {
            match parser.push(byte) {
                Ok(Some(_)) => frames += 1,
                Ok(None) => pending += 1,
                Err(error) => panic!("unexpected error {:?}", error),
            }
        }

        assert_eq!(frames, 1);
        assert_eq!(pending, EXCEPTION.len() - 1);
    }

    // ── Error: invalid protocol ID ──

    #[test]
    fn invalid_protocol_id_returns_error() {
        let mut parser = FrameParser::new();
        let received = [0x00, 0x01, 0x00, 0x01, 0x00, 0x06, 0x11];
        assert_eq!(
            feed_bytes(&mut parser, &received),
            Err(ParseError::InvalidProtocolIdentifier)
        );
    }

    #[test]
    fn invalid_protocol_id_high_byte() {
        let mut parser = FrameParser::new();
        let received = [0x00, 0x01, 0x01, 0x00, 0x00, 0x06, 0x11];
        assert_eq!(
            feed_bytes(&mut parser, &received),
            Err(ParseError::InvalidProtocolIdentifier)
        );
    }

    // ── Error: invalid length ──

    #[test]
    fn invalid_lengths_return_error() {
        for length in [0, 1, 255, 0xFFFF] {
            let mut parser = FrameParser::new();
            assert_eq!(
                feed_bytes(&mut parser, &header(length)),
                Err(ParseError::InvalidLength),
                "length {}",
                length
            );
        }
    }

    // ── Error state behavior ──

    #[test]
    fn error_state_persists_until_reset() {
        let mut parser = FrameParser::new();
        let bad = [0x00, 0x01, 0x00, 0x01, 0x00, 0x06, 0x11];
        feed_bytes(&mut parser, &bad).unwrap_err();

        assert_eq!(
            parser.push(0x00),
            Err(ParseError::InvalidProtocolIdentifier)
        );
        assert_eq!(
            parser.push_slice(&EXCEPTION),
            (0, Err(ParseError::InvalidProtocolIdentifier))
        );
        assert!(!parser.is_idle());
    }

    #[test]
    fn reset_recovers_from_error() {
        let mut parser = FrameParser::new();
        let bad = [0x00, 0x01, 0x00, 0x01, 0x00, 0x06, 0x11];
        feed_bytes(&mut parser, &bad).unwrap_err();

        parser.reset();
        let frame = feed_bytes(&mut parser, &EXCEPTION).unwrap().unwrap();
        assert_eq!(frame.transaction_id(), 0x0002);
        assert_eq!(frame.unit_id(), 0x22);
    }

    // ── Reset ──

    #[test]
    fn reset_during_header() {
        let mut parser = FrameParser::new();
        feed_bytes(&mut parser, &READ_HOLDING_REGISTERS[..3]).unwrap();

        parser.reset();
        let frame = feed_bytes(&mut parser, &EXCEPTION).unwrap().unwrap();
        assert_eq!(frame.transaction_id(), 0x0002);
    }

    #[test]
    fn reset_during_payload() {
        let mut parser = FrameParser::new();
        feed_bytes(&mut parser, &READ_HOLDING_REGISTERS[..8]).unwrap();

        parser.reset();
        let frame = feed_bytes(&mut parser, &EXCEPTION).unwrap().unwrap();
        assert_eq!(frame.transaction_id(), 0x0002);
    }

    // ── Idle ──

    #[test]
    fn is_idle_tracks_partial_frames() {
        let mut parser = FrameParser::new();
        assert!(parser.is_idle());

        feed_bytes(&mut parser, &EXCEPTION[..3]).unwrap();
        assert!(!parser.is_idle());

        feed_bytes(&mut parser, &EXCEPTION[3..]).unwrap().unwrap();
        assert!(parser.is_idle());
    }
}
