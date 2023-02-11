use crate::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Pixel {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

// Function implementation taken from https://github.com/KokaKiwi/rust-hex
const fn hex_val(c: u8) -> u8 {
    match c {
        b'A'..=b'F' => c - b'A' + 10,
        b'a'..=b'f' => c - b'a' + 10,
        b'0'..=b'9' => c - b'0',
        _ => panic!("Unable to parse hex-value from given byte"),
    }
}

impl Pixel {
    pub fn from_hex(hex: &str) -> Self {
        match hex.len() {
			3 => Pixel {
				r: 17 * hex_val(hex.bytes().nth(0).unwrap()),
				g: 17 * hex_val(hex.bytes().nth(1).unwrap()),
				b: 17 * hex_val(hex.bytes().nth(2).unwrap()),
				a: 255,
			},
			4 => Pixel {
				r: 17 * hex_val(hex.bytes().nth(0).unwrap()),
				g: 17 * hex_val(hex.bytes().nth(1).unwrap()),
				b: 17 * hex_val(hex.bytes().nth(2).unwrap()),
				a: 17 * hex_val(hex.bytes().nth(3).unwrap()),
			},
			6 => Pixel {
				r: 16 * hex_val(hex.bytes().nth(0).unwrap()) + hex_val(hex.bytes().nth(1).unwrap()),
				g: 16 * hex_val(hex.bytes().nth(2).unwrap()) + hex_val(hex.bytes().nth(3).unwrap()),
				b: 16 * hex_val(hex.bytes().nth(4).unwrap()) + hex_val(hex.bytes().nth(5).unwrap()),
				a: 255,
			},
			8 => Pixel {
				r: 16 * hex_val(hex.bytes().nth(0).unwrap()) + hex_val(hex.bytes().nth(1).unwrap()),
				g: 16 * hex_val(hex.bytes().nth(2).unwrap()) + hex_val(hex.bytes().nth(3).unwrap()),
				b: 16 * hex_val(hex.bytes().nth(4).unwrap()) + hex_val(hex.bytes().nth(5).unwrap()),
				a: 16 * hex_val(hex.bytes().nth(6).unwrap()) + hex_val(hex.bytes().nth(7).unwrap()),
			},
			_ => panic!("Unable to parse Pixel values from a hex-string that isn't 3,4,6 or 8 characters long"),
		}
    }

    pub fn pixel_hash(&self) -> usize {
        let r = self.r as usize;
        let g = self.g as usize;
        let b = self.b as usize;
        let a = self.a as usize;
        (r * 3 + g * 5 + b * 7 + a * 11) % 64
    }

    pub const fn def() -> Self {
        Pixel {
            r: 0,
            g: 0,
            b: 0,
            a: 255,
        }
    }

    pub const fn zero() -> Self {
        Pixel {
            r: 0,
            g: 0,
            b: 0,
            a: 0,
        }
    }
}

impl Default for Pixel {
    fn default() -> Self {
        Pixel::def()
    }
}

impl From<[u8; 4]> for Pixel {
    fn from(value: [u8; 4]) -> Self {
        Pixel {
            r: value[0],
            g: value[1],
            b: value[2],
            a: value[3],
        }
    }
}

impl From<[u8; 3]> for Pixel {
    fn from(value: [u8; 3]) -> Self {
        Pixel {
            r: value[0],
            g: value[1],
            b: value[2],
            a: 255,
        }
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

impl TryFrom<u8> for ColorChannel {
    type Error = u8;
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            3 => Ok(ColorChannel::RGB),
            4 => Ok(ColorChannel::RGBA),
            _ => Err(value),
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

impl TryFrom<u8> for ColorSpace {
    type Error = u8;
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(ColorSpace::SRGB),
            1 => Ok(ColorSpace::LINEAR),
            _ => Err(value),
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
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
