use crate::FunctionKind;

pub trait PduData {
    fn pdu_data(&self) -> &[u8];

    fn function_code(&self) -> FunctionKind {
        FunctionKind::from(self.pdu_data().first().copied().unwrap_or(0))
    }
}
