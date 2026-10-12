pub trait AduData {
    fn adu_data(&self) -> &[u8];
    fn adu_length(&self) -> usize;
}
