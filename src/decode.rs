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

    NotImplemented,
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
                write!(
                    f,
                    "Not enough bytes were received to contain the file's Header."
                )
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
            Self::NotImplemented => {
                write!(f, "Not yet implemented.")
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
fn decode_pixels<'a>(
    header: &Header,
    buffer: &mut BufIter<'a, u8>,
    pixels: &mut MutBufIter<'a, u8>,
) -> Result<(), DecodeError> {
    let mut done = false;
    while !done {
        done = match buffer.step_one() {
            None => Err(DecodeError::MissingPixels {
                expected_size: header.pixel_amount() * header.bytes_per_pixel(),
                received_size: buffer.idx(),
            }),
            Some(&byte) => match byte {
                OP_RGB => match buffer.step_forward(3) {
                    None => Err(DecodeError::MissingPixels {
                        expected_size: header.pixel_amount() * header.bytes_per_pixel(),
                        received_size: buffer.idx(),
                    }),
                    Some(bytes) => {
                        pixels.copy_from_slice(bytes, 3, true);
                        Ok(false)
                    }
                },
                OP_RGBA => match buffer.step_forward(4) {
                    None => Err(DecodeError::MissingPixels {
                        expected_size: header.pixel_amount() * header.bytes_per_pixel(),
                        received_size: buffer.idx(),
                    }),
                    Some(bytes) => {
                        dbg!(byte);
                        let bytes = dbg!(bytes);
                        pixels.copy_from_slice(bytes, 4, true);
                        Ok(false)
                    }
                },
                _ => match buffer.slide(1, STREAM_END_SIZE) {
                    None => Err(DecodeError::NotImplemented),
                    Some(bytes) => {
                        if bytes
                            .into_iter()
                            .enumerate()
                            .all(|(i, &x)| x == STREAM_END[i])
                        {
                            Ok(true)
                        } else {
                            Err(DecodeError::NotImplemented)
                        }
                    }
                },
            },
        }?;
    }
    Ok(())
}

pub fn decode<'a>(buffer: &[u8], pixels: &mut [u8]) -> Result<Header, DecodeError> {
    let header = decode_header(buffer)?;
    let pixels_size = header.pixel_amount() * header.bytes_per_pixel();

    let mut pixels = match MutBufIter::from(pixels, ..pixels_size) {
        None => Err(DecodeError::PixelBufferTooSmall {
            expected_size: pixels_size,
            received_size: pixels.len(),
        }),
        Some(pixels) => Ok(pixels),
    }?;
    let mut buffer = BufIter::from(buffer, HEADER_SIZE.into()..).unwrap();

    decode_pixels(&header, &mut buffer, &mut pixels)?;
    Ok(header)
}

pub fn decode_allocated<'a>(buffer: &[u8]) -> Result<(Header, Vec<u8>), DecodeError> {
    let header = decode_header(buffer)?;
    let pixels_size = header.pixel_amount() * header.bytes_per_pixel();

    let mut pixels_vec = vec![0; pixels_size];
    let mut pixels = match MutBufIter::from(&mut pixels_vec, ..pixels_size) {
        None => Err(DecodeError::PixelBufferTooSmall {
            expected_size: pixels_size,
            received_size: pixels_vec.len(),
        }),
        Some(pixels) => Ok(pixels),
    }?;
    let mut buffer = BufIter::from(buffer, HEADER_SIZE.into()..).unwrap();

    decode_pixels(&header, &mut buffer, &mut pixels)?;
    Ok((header, pixels_vec))
}
