use crate::*;
use std::fmt;

pub enum EncodeError {
    /// The amount of pixels in the `pixels` buffer \
    /// doesn't correspond to the expected amount of \
    /// pixels: `width * height`.
    MissingPixels {
        expected_size: usize,
        received_size: usize,
    },

    /// The output buffer is too small to fit the encoded image.
    /// The maximum of bytes needed to fit into the buffer is returned.
    BufferTooSmall {
        expected_size: usize,
        received_size: usize,
    },

    IOError(std::io::Error),
}

impl fmt::Debug for EncodeError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &*self {
            Self::MissingPixels {
                expected_size,
                received_size,
            } => {
                write!(
                    f,
                    "Missing Pixels: Expected {} bytes for the pixels, instead received {} bytes for the pixels.",
                    expected_size,
					received_size
                )
            }
            Self::BufferTooSmall {
                expected_size,
                received_size,
            } => {
                write!(f, "Output Buffer too small: At least {} bytes were expected, but only {} bytes were received", expected_size, received_size)
            }
            Self::IOError(err) => {
                write!(f, "IO Error: {}", err)
            }
        }
    }
}

impl From<std::io::Error> for EncodeError {
    fn from(value: std::io::Error) -> Self {
        EncodeError::IOError(value)
    }
}

use EncodeError::*;

fn encode_header(header: &Header, buffer: &mut [u8]) -> Result<usize, EncodeError> {
    if buffer.len() < HEADER_SIZE {
        return Err(EncodeError::BufferTooSmall {
            expected_size: HEADER_SIZE,
            received_size: buffer.len(),
        });
    }
    buffer[0..4].copy_from_slice(&MAGIC);
    buffer[4..8].copy_from_slice(&header.width.to_be_bytes());
    buffer[8..12].copy_from_slice(&header.height.to_be_bytes());
    buffer[12..HEADER_SIZE].copy_from_slice(&[header.channels as u8, header.colorspace as u8]);
    Ok(HEADER_SIZE)
}

fn encode_pixel(
    header: &Header,
    pixels: &[u8],
    buffer: &mut [u8],
    px_idx: &mut usize,
    buf_idx: &mut usize,
) -> Result<(), EncodeError> {
    match header.channels {
        ColorChannel::RGB => {
            buffer[*buf_idx..*buf_idx + 1].copy_from_slice(&OP_RGB.to_be_bytes());
            match pixels.get(..3) {
                None => Err(EncodeError::MissingPixels {
                    expected_size: header.pixel_amount() * header.bytes_per_pixel(),
                    received_size: *px_idx + pixels.len(),
                }),
                Some(pixel) => {
                    buffer[*buf_idx + 1..*buf_idx + 4].copy_from_slice(pixel);
                    *px_idx += 3;
                    *buf_idx += 4;
                    Ok(())
                }
            }
        }
        ColorChannel::RGBA => {
            buffer[*buf_idx..*buf_idx + 1].copy_from_slice(&OP_RGBA.to_be_bytes());
            match pixels.get(..4) {
                None => Err(EncodeError::MissingPixels {
                    expected_size: header.pixel_amount() * header.bytes_per_pixel(),
                    received_size: *px_idx + pixels.len(),
                }),
                Some(pixel) => {
                    buffer[*buf_idx + 1..*buf_idx + 5].copy_from_slice(pixel);
                    *px_idx += 4;
                    *buf_idx += 5;
                    Ok(())
                }
            }
        }
    }
}

pub fn encode(header: &Header, pixels: &[u8], buffer: &mut [u8]) -> Result<usize, EncodeError> {
    let pixels_size = header.pixel_amount() * header.bytes_per_pixel();
    let pixels = match pixels.get(..pixels_size) {
        None => Err(MissingPixels {
            expected_size: pixels_size,
            received_size: pixels.len(),
        }),
        Some(pixels) => Ok(pixels),
    }?;

    let mut buf_idx = encode_header(header, buffer)?;
    let mut px_idx = 0;
    while px_idx < pixels_size {
        encode_pixel(header, &pixels[px_idx..], buffer, &mut px_idx, &mut buf_idx)?;
    }

    buffer[buf_idx..buf_idx + STREAM_END_SIZE].copy_from_slice(&STREAM_END);
    buf_idx += STREAM_END_SIZE;

    Ok(buf_idx)
}

pub fn encode_allocated(header: &Header, pixels: &[u8]) -> Result<Vec<u8>, EncodeError> {
    let mut buffer = vec![0; header.max_size()];
    let out_size = encode(header, pixels, &mut buffer)?;
    buffer.truncate(out_size);
    Ok(buffer)
}
