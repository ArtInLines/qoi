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
pub const RUN: [u8; 64] = [0 as u8; 64];
pub const OP_RGB: u8 = 0b11111110;
pub const OP_RGBA: u8 = 0b11111111;
pub const HEADER_SIZE: usize = 14;

pub fn open_file<P>(filepath: P) -> Result<File, IOErr>
where
    P: AsRef<Path>,
{
    let res = if filepath.as_ref().exists() {
        File::options().read(true).write(true).open(filepath)?
    } else {
        File::create(filepath)?
    };
    Ok(res)
}

pub fn read<P>(filepath: P) -> Result<(Header, Vec<u8>), DecodeError>
where
    P: AsRef<Path>,
{
    let mut file = open_file(filepath)?;
    let mut bytes = Vec::<u8>::new();
    file.read_to_end(&mut bytes)?;
    decode_allocated(&mut bytes)
}

pub fn write<P>(filepath: P, header: &Header, pixels: &mut [u8]) -> Result<usize, EncodeError>
where
    P: AsRef<Path>,
{
    let mut file = open_file(filepath)?;
    let buffer = encode_allocated(header, pixels)?;
    file.write_all(&buffer)?;
    Ok(buffer.len())
}
