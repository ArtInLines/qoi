mod buf_iter;
mod util;
use decode::*;
use encode::*;
use std::{
    fs::File,
    io::{Error as IOErr, Read, Write},
    path::Path,
};

pub mod decode;
pub mod encode;
pub use buf_iter::*;
pub use util::*;

pub const MAGIC: [u8; 4] = [b'q', b'o', b'i', b'f'];
pub const STREAM_END_SIZE: usize = 8;
pub const STREAM_END: [u8; STREAM_END_SIZE] = [0, 0, 0, 0, 0, 0, 0, 1];
pub const PREV_ARR_SIZE: usize = 64;
pub const OP_RGB: u8 = 0b11111110;
pub const OP_RGBA: u8 = 0b11111111;
pub const OP_INDEX: u8 = 0b00000000;
pub const OP_DIFF: u8 = 0b01000000;
pub const OP_LUMA: u8 = 0b10000000;
pub const OP_RUN: u8 = 0b11000000;
pub const MASK_2: u8 = 0b11000000;
pub const DEMASK_2: u8 = 0b00111111;
pub const DIFF_BIAS: u8 = 2;
pub const LUMA_GREEN_BIAS: u8 = 32;
pub const LUMA_BIAS: u8 = 8;
pub const HEADER_SIZE: usize = 14;

pub fn open_file_w<P>(filepath: P) -> Result<File, IOErr>
where
    P: AsRef<Path>,
{
    let res = if filepath.as_ref().exists() {
        File::options().write(true).truncate(true).open(filepath)?
    } else {
        File::create(filepath)?
    };
    Ok(res)
}

pub fn read<P>(filepath: P) -> Result<(Header, Vec<Pixel>), DecodeError>
where
    P: AsRef<Path>,
{
    let mut file = File::open(filepath)?;
    let mut bytes = Vec::<u8>::new();
    file.read_to_end(&mut bytes)?;
    decode_allocated(&mut bytes)
}

pub fn write<P>(filepath: P, header: &Header, pixels: &mut [Pixel]) -> Result<usize, EncodeError>
where
    P: AsRef<Path>,
{
    let mut file = open_file_w(filepath)?;
    let buffer = encode_allocated(header, pixels)?;
    file.write_all(&buffer)?;
    Ok(buffer.len())
}
