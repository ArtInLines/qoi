pub enum EncodeError {}

#[allow(unused_variables)]
pub fn encode(pixels: &[u8], buffer: &mut [u8]) -> Result<usize, EncodeError> {
    Ok(1)
}

pub enum DecodeError {}

#[allow(unused_variables)]
pub fn decode(bytes: &[u8], buffer: &mut [u8]) -> Result<usize, DecodeError> {
    Ok(1)
}
