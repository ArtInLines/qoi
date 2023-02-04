use crate::*;
use std::fmt;

pub enum DecodeError {
    /// Encoded Data doesn't start with the magic value `b"qoif"`.
    /// This magic value is used to indicate, that the data was actually encoded with QOI
    /// The received bytes are returned
    InvalidMagic([u8; 4]),

    /// The provided channels value is invalid.
    /// The only allowed values for the channels are \
    /// `3` for RGB or `4` for RGBA.
    /// The received value is returned
    InvalidChannels(u8),

    /// The provided colorspace value is invalid.
    /// The only allowed values for the colorspace are \
    /// `0` for sRGB with linear alpha or \
    /// `1` for having all channels encoded linearly
    InvalidColorspace(u8),

    /// The provided input doesn't contain enough bytes \
    /// for a header.
    MissingHeader,

    /// The end marker for the encoded data was received \
    /// before all expected pixels could be decoded.
    /// The data should contain a chunk of data for each pixel \
    /// of which there should be `width * height` many.
    MissingPixels {
        expected_size: usize,
        received_size: usize,
    },

    /// The output buffer is too small to fit all pixels.
    PixelBufferTooSmall {
        expected_size: usize,
        received_size: usize,
    },

    IOError(std::io::Error),
}

impl fmt::Debug for DecodeError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &*self {
            Self::InvalidMagic(received_magic) => {
                write!(
                    f,
                    "Invalid magic: Expected {:?}, instead received {:?}.",
                    MAGIC, received_magic
                )
            }
            Self::InvalidChannels(received_channels) => {
                write!(f, "Invalid number of channels: {}. Expected 3 or 4 channels for RGB and RGBA, respectively.", received_channels)
            }
            Self::InvalidColorspace(received_colorspace) => {
                write!(f, "Invalid color space: {}. Expected 0 or 1, where 0 = sRGB with linear alpha, and 1 = all channels linear.", received_colorspace)
            }
            Self::MissingHeader => {
                write!(f, "Not enough bytes were received to contain ")
            }
            Self::MissingPixels {
                expected_size,
                received_size,
            } => {
                write!(
                    f,
                    "Missing Data: {} pixels were expected, but only {} pixels were read before reaching the end marker.",
                    expected_size,
					received_size
                )
            }
            Self::PixelBufferTooSmall {
                expected_size,
                received_size,
            } => {
                write!(f, "Output Buffer too small: It can fit only {} bytes, but should be able to fit {} bytes.", received_size, expected_size)
            }
            Self::IOError(err) => {
                write!(f, "IO Error: {}", err)
            }
        }
    }
}

impl From<std::io::Error> for DecodeError {
    fn from(value: std::io::Error) -> Self {
        DecodeError::IOError(value)
    }
}

fn decode_header(buffer: &[u8]) -> Result<Header, DecodeError> {
    let header_bytes = match buffer.get(..HEADER_SIZE) {
        Some(bytes) => Ok(bytes),
        None => Err(DecodeError::MissingHeader),
    }?;
    let header = Header {
        width: u32::from_be_bytes(header_bytes[4..8].try_into().unwrap()),
        height: u32::from_be_bytes(header_bytes[8..12].try_into().unwrap()),
        channels: header_bytes[12].into(),
        colorspace: header_bytes[13].into(),
    };
    Ok(header)
}

// Return value indicates whether the decoding process is done
fn decode_pixel(
    header: &Header,
    buffer: &[u8],
    pixels: &mut [u8],
    buf_idx: &mut usize,
    px_idx: &mut usize,
) -> Result<bool, DecodeError> {
    Ok(false)
}

pub fn decode(buffer: &[u8], pixels: &mut [u8]) -> Result<Header, DecodeError> {
    let header = decode_header(buffer)?;
    let pixels_size = header.pixel_amount() * header.bytes_per_pixel();
    let mut pixels = match pixels.get_mut(..pixels_size) {
        None => Err(DecodeError::PixelBufferTooSmall {
            expected_size: pixels_size,
            received_size: buffer.len(),
        }),
        Some(pixels) => Ok(pixels),
    }?;

    let mut done = false;
    let mut buf_idx = HEADER_SIZE;
    let mut px_idx = 0;
    while !done {
        done = decode_pixel(&header, buffer, &mut pixels, &mut buf_idx, &mut px_idx)?;
    }

    Ok(header)
}
