use crate::{AduData, FrameEncoder, FunctionKind, PduData};

// ── Frame ──

/// Modbus/TCP frame (ADU) with internal storage for the largest possible frame.
///
/// A frame is either produced by [`FrameParser`](crate::tcp::FrameParser), in which
/// case it is always complete and valid, or created via [`Frame::new`] and completed
/// by an encoder.
#[derive(Debug, Clone)]
pub struct Frame {
    data: [u8; Frame::MAX_SIZE],
    len: usize,
}

impl Frame {
    /// Maximum number of bytes in a Modbus/TCP frame (MBAP header + 253 bytes PDU).
    pub const MAX_SIZE: usize = 260;

    /// Number of bytes in the MBAP header (including the unit identifier).
    pub const HEADER_SIZE: usize = 7;

    /// Creates a new outgoing frame with the given header and an empty PDU.
    pub const fn new(transaction_id: u16, unit_id: u8) -> Frame {
        let mut data = [0u8; Frame::MAX_SIZE];
        let bytes = transaction_id.to_be_bytes();
        data[0] = bytes[0];
        data[1] = bytes[1];
        data[5] = 1;
        data[6] = unit_id;
        Frame {
            data,
            len: Frame::HEADER_SIZE,
        }
    }

    /// Creates a frame from bytes that were already validated by the parser.
    pub(crate) fn from_valid(bytes: &[u8]) -> Frame {
        debug_assert!(bytes.len() > Frame::HEADER_SIZE && bytes.len() <= Frame::MAX_SIZE);
        let mut data = [0u8; Frame::MAX_SIZE];
        data[..bytes.len()].copy_from_slice(bytes);
        Frame {
            data,
            len: bytes.len(),
        }
    }

    // ── MBAP header ──

    pub fn transaction_id(&self) -> u16 {
        u16::from_be_bytes([self.data[0], self.data[1]])
    }

    pub fn protocol_id(&self) -> u16 {
        u16::from_be_bytes([self.data[2], self.data[3]])
    }

    pub fn length(&self) -> u16 {
        u16::from_be_bytes([self.data[4], self.data[5]])
    }

    pub fn unit_id(&self) -> u8 {
        self.data[6]
    }

    pub fn set_transaction_id(&mut self, id: u16) {
        self.data[0..2].copy_from_slice(&id.to_be_bytes());
    }

    pub fn set_unit_id(&mut self, id: u8) {
        self.data[6] = id;
    }

    // ── PDU ──

    pub fn function_code(&self) -> FunctionKind {
        FunctionKind::from(self.data[Frame::HEADER_SIZE])
    }

    /// Function code and data.
    pub fn pdu(&self) -> &[u8] {
        &self.data[Frame::HEADER_SIZE..self.len]
    }

    /// The complete frame, ready to be sent.
    pub fn as_bytes(&self) -> &[u8] {
        &self.data[..self.len]
    }
}

impl PartialEq for Frame {
    fn eq(&self, other: &Self) -> bool {
        self.as_bytes() == other.as_bytes()
    }
}

impl Eq for Frame {}

impl AduData for Frame {
    fn adu_data(&self) -> &[u8] {
        self.as_bytes()
    }

    fn adu_length(&self) -> usize {
        self.len
    }
}

impl PduData for Frame {
    fn pdu_data(&self) -> &[u8] {
        self.pdu()
    }

    fn function_code(&self) -> FunctionKind {
        Frame::function_code(self)
    }
}

impl FrameEncoder for Frame {
    fn payload_mut(&mut self) -> &mut [u8] {
        &mut self.data[Frame::HEADER_SIZE + 1..]
    }

    fn finalize(&mut self, code: FunctionKind, payload_len: usize) {
        assert!(
            payload_len < Self::MAX_SIZE - Self::HEADER_SIZE,
            "payload exceeds frame capacity"
        );

        // function code
        self.data[Self::HEADER_SIZE] = u8::from(code);

        // MBAP length = unit_id + function_code + payload
        let length = (1 + 1 + payload_len) as u16;
        self.data[4..6].copy_from_slice(&length.to_be_bytes());

        // Total frame length
        self.len = Frame::HEADER_SIZE + 1 + payload_len;
    }
}

#[cfg(test)]
mod frame_tests {
    use super::*;
    use crate::FunctionCode;
    use crate::write_single_register::Response;

    // Write single register request: address 0x0001, value 0x0003
    const REQUEST: [u8; 12] = [
        0x00, 0x2A, 0x00, 0x00, 0x00, 0x06, 0x11, 0x06, 0x00, 0x01, 0x00, 0x03,
    ];

    #[test]
    fn header_and_pdu_access() {
        let frame = Frame::from_valid(&REQUEST);
        assert_eq!(frame.transaction_id(), 0x002A);
        assert_eq!(frame.protocol_id(), 0x0000);
        assert_eq!(frame.length(), 0x0006);
        assert_eq!(frame.unit_id(), 0x11);
        assert_eq!(
            frame.function_code(),
            FunctionKind::Normal(FunctionCode::WriteSingleRegister)
        );
        assert_eq!(frame.pdu(), &REQUEST[7..]);
        assert_eq!(frame.as_bytes(), &REQUEST);
    }

    #[test]
    fn set_header_fields() {
        let mut frame = Frame::from_valid(&REQUEST);
        frame.set_transaction_id(0xBEEF);
        frame.set_unit_id(0x22);
        assert_eq!(frame.transaction_id(), 0xBEEF);
        assert_eq!(frame.unit_id(), 0x22);
    }

    #[test]
    fn equality_ignores_unused_bytes() {
        let mut a = Frame::new(1, 2);
        let b = Frame::new(1, 2);
        a.data[100] = 0xFF;
        assert_eq!(a, b);
    }

    #[test]
    fn encode_reply_in_place() {
        // the echo response of write single register equals the request
        let mut frame = Frame::from_valid(&REQUEST);
        Response::new(0x0001, 0x0003).encode(&mut frame).unwrap();
        assert_eq!(frame.as_bytes(), &REQUEST);
    }

    #[test]
    fn encode_exception_in_place_keeps_header() {
        let mut frame = Frame::from_valid(&REQUEST);
        Response::new_exception(crate::ExceptionCode::IllegalDataAddress)
            .encode(&mut frame)
            .unwrap();
        assert_eq!(
            frame.as_bytes(),
            &[0x00, 0x2A, 0x00, 0x00, 0x00, 0x03, 0x11, 0x86, 0x02]
        );
    }

    #[test]
    fn new_frame_with_encoder() {
        let mut frame = Frame::new(0x002A, 0x11);
        assert_eq!(frame.as_bytes().len(), Frame::HEADER_SIZE);
        assert!(frame.pdu().is_empty());

        Response::new(0x0001, 0x0003).encode(&mut frame).unwrap();
        assert_eq!(frame.as_bytes(), &REQUEST);
    }
}
