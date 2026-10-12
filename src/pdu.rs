use crate::FunctionKind;

pub trait PduData {
    fn pdu_data(&self) -> &[u8];

    fn function_code(&self) -> FunctionKind {
        FunctionKind::from(self.pdu_data().first().copied().unwrap_or(0))
    }
}

pub trait FrameEncoder {
    /// Mutable access to the payload area (after function code). Its length is the available capacity.
    fn payload_mut(&mut self) -> &mut [u8];

    /// Finalize the frame with function code and payload length (e.g. MBAP length or CRC).
    fn finalize(&mut self, code: FunctionKind, payload_len: usize);
}
