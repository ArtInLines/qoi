use crate::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Pixel {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Pixel {
    pub fn pixel_hash(&self) -> u8 {
        (self.r * 3 + self.g * 5 + self.b * 7 + self.a * 11) % 64
    }

    pub const fn def() -> Self {
        Pixel {
            r: 0,
            g: 0,
            b: 0,
            a: 255,
        }
    }
}

impl Default for Pixel {
    fn default() -> Self {
        Pixel::def()
    }
}

impl From<&[u8]> for Pixel {
    fn from(value: &[u8]) -> Self {
        if value.len() < 3 {
            Pixel::default()
        } else if value.len() < 4 {
            Pixel {
                r: value[0],
                g: value[1],
                b: value[2],
                a: 255,
            }
        } else {
            Pixel {
                r: value[0],
                g: value[1],
                b: value[2],
                a: value[3],
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
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

#[derive(Debug, Clone, Copy)]
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

    pub fn pixel_len(&self) -> usize {
        self.pixel_amount() * self.bytes_per_pixel()
    }

    pub fn max_bytes_per_pixel(&self) -> usize {
        self.bytes_per_pixel() + 1
    }

    pub fn bytes_per_pixel(&self) -> usize {
        self.channels.into()
    }

    pub fn max_size(&self) -> usize {
        self.pixel_amount() * self.max_bytes_per_pixel() * (self.channels as usize + 1)
            + HEADER_SIZE
            + STREAM_END_SIZE
    }
}
