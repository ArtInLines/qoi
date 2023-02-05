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

fn encode_header<'a>(
    header: &Header,
    buffer: &mut MutBufIter<'a, u8>,
) -> Result<usize, EncodeError> {
    if buffer.len() < HEADER_SIZE {
        return Err(EncodeError::BufferTooSmall {
            expected_size: HEADER_SIZE,
            received_size: buffer.len(),
        });
    }
    buffer.copy_from_slice(&MAGIC, 4, true);
    buffer.copy_from_slice(&header.width.to_be_bytes(), 4, true);
    buffer.copy_from_slice(&header.height.to_be_bytes(), 4, true);
    buffer.copy_from_slice(&[header.channels.into(), header.colorspace.into()], 2, true);
    Ok(HEADER_SIZE)
}

fn encode_pixel<'a>(
    header: &Header,
    pixels: &mut BufIter<'a, u8>,
    buffer: &mut MutBufIter<'a, u8>,
) -> Result<(), EncodeError> {
    match header.channels {
        ColorChannel::RGB => {
            buffer.copy_from_slice(&OP_RGB.to_be_bytes(), 1, true);
            match pixels.step_forward(3) {
                None => Err(EncodeError::MissingPixels {
                    expected_size: header.pixel_amount() * header.bytes_per_pixel(),
                    received_size: pixels.idx() + pixels.len(),
                }),
                Some(pixel) => {
                    buffer.copy_from_slice(pixel, 3, true);
                    Ok(())
                }
            }
        }
        ColorChannel::RGBA => {
            buffer.copy_from_slice(&OP_RGBA.to_be_bytes(), 1, true);
            match pixels.step_forward(4) {
                None => Err(EncodeError::MissingPixels {
                    expected_size: header.pixel_amount() * header.bytes_per_pixel(),
                    received_size: pixels.idx() + pixels.len(),
                }),
                Some(pixel) => {
                    buffer.copy_from_slice(pixel, 4, true);
                    Ok(())
                }
            }
        }
    }
}

pub fn encode<'a>(header: &Header, pixels: &[u8], buffer: &mut [u8]) -> Result<usize, EncodeError> {
    let pixels_size = header.pixel_amount() * header.bytes_per_pixel();
    let mut pixels = match BufIter::from(pixels, ..pixels_size) {
        None => Err(MissingPixels {
            expected_size: pixels_size,
            received_size: pixels.len(),
        }),
        Some(pixels) => Ok(pixels),
    }?;
    let mut buffer = MutBufIter::new(buffer);

    encode_header(header, &mut buffer)?;
    while pixels.idx() < pixels_size {
        encode_pixel(header, &mut pixels, &mut buffer)?;
    }

    buffer.copy_from_slice(&STREAM_END, STREAM_END_SIZE, true);

    Ok(buffer.idx())
}

pub fn encode_allocated(header: &Header, pixels: &[u8]) -> Result<Vec<u8>, EncodeError> {
    let mut buffer = vec![0; header.max_size()];
    let out_size = encode(header, pixels, &mut buffer)?;
    buffer.truncate(out_size);
    Ok(buffer)
}
