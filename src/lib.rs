use decode::*;
use encode::*;
use std::{
    fs::File,
    io::{Error as IOErr, Read, Write},
    path::Path,
};

pub mod decode;
pub mod encode;

pub static MAGIC: [u8; 4] = [b'q', b'o', b'i', b'f'];
pub const STREAM_END_SIZE: usize = 8;
pub static STREAM_END: [u8; STREAM_END_SIZE] = [0, 0, 0, 0, 0, 0, 0, 1];
pub static RUN: [u8; 64] = [0 as u8; 64];
pub static OP_RGB: u8 = 0b11111110;
pub static OP_RGBA: u8 = 0b11111111;
pub static HEADER_SIZE: usize = 14;

pub trait Pixel {
    fn rgb(&self) -> [u8; 3];

    fn rgba(&self) -> [u8; 4];

    fn r(&self) -> u8;

    fn g(&self) -> u8;

    fn b(&self) -> u8;

    fn a(&self) -> u8;
}

#[derive(Debug, Clone, Copy)]
pub enum ColorChannel {
    RGB,
    RGBA,
}

impl From<u8> for ColorChannel {
    fn from(value: u8) -> Self {
        match value {
            3 => ColorChannel::RGB,
            4 => ColorChannel::RGBA,
            _ => panic!("Invalid ColorChannel value"),
        }
    }
}

impl From<ColorChannel> for u8 {
    fn from(value: ColorChannel) -> Self {
        match value {
            ColorChannel::RGB => 3,
            ColorChannel::RGBA => 4,
        }
    }
}

impl From<ColorChannel> for u32 {
    fn from(value: ColorChannel) -> Self {
        match value {
            ColorChannel::RGB => 3,
            ColorChannel::RGBA => 4,
        }
    }
}

impl From<ColorChannel> for usize {
    fn from(value: ColorChannel) -> Self {
        match value {
            ColorChannel::RGB => 3,
            ColorChannel::RGBA => 4,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum ColorSpace {
    SRGB,
    LINEAR,
}

impl From<u8> for ColorSpace {
    fn from(value: u8) -> Self {
        match value {
            0 => ColorSpace::SRGB,
            1 => ColorSpace::LINEAR,
            _ => panic!("Invalid ColorSpace value"),
        }
    }
}

impl From<ColorSpace> for u8 {
    fn from(value: ColorSpace) -> Self {
        match value {
            ColorSpace::SRGB => 0,
            ColorSpace::LINEAR => 1,
        }
    }
}

pub struct Header {
    pub width: u32,
    pub height: u32,
    pub channels: ColorChannel,
    pub colorspace: ColorSpace,
}

impl Header {
    pub fn new(width: u32, height: u32, channels: ColorChannel, colorspace: ColorSpace) -> Self {
        Header {
            width,
            height,
            channels,
            colorspace,
        }
    }

    pub fn pixel_amount(&self) -> usize {
        (self.width * self.height) as usize
    }

    pub fn max_bytes_per_pixel(&self) -> usize {
        self.bytes_per_pixel() + 1
    }

    pub fn bytes_per_pixel(&self) -> usize {
        match self.channels {
            ColorChannel::RGB => 3,
            ColorChannel::RGBA => 4,
        }
    }

    pub fn max_size(&self) -> usize {
        self.pixel_amount() * self.max_bytes_per_pixel() * (self.channels as usize + 1)
            + HEADER_SIZE
            + STREAM_END_SIZE
    }
}

fn open_file<P>(filepath: P) -> Result<File, IOErr>
where
    P: AsRef<Path>,
{
    let res = if filepath.as_ref().exists() {
        File::options().write(true).open(filepath)?
    } else {
        File::create(filepath)?
    };
    Ok(res)
}

pub fn read<P>(filepath: P, pixels: &mut [u8]) -> Result<Header, DecodeError>
where
    P: AsRef<Path>,
{
    let mut file = open_file(filepath)?;
    let mut bytes = Vec::<u8>::new();
    file.read(&mut bytes)?;
    decode(&mut bytes, pixels)
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
