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

    /// Issued when the encoding doesn't conform to the spec.
    InvalidEncoding,

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

impl DecodeError {
    pub fn missing_pixels<T, S>(header: &Header, pixels: &T) -> Self
    where
        T: BufIterType<S>,
        S: Copy,
    {
        MissingPixels {
            expected_size: header.max_size(),
            received_size: pixels.len(),
        }
    }

    pub fn pixel_buffer_too_small<T, S>(header: &Header, pixels: &T) -> Self
    where
        T: BufIterType<S>,
        S: Copy,
    {
        PixelBufferTooSmall {
            expected_size: header.pixel_amount(),
            received_size: pixels.len(),
        }
    }
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
            Self::InvalidEncoding => {
                write!(f, "The buffer is not properly encoded. Make sure your encoder conforms to the spec found at https://qoiformat.org/qoi-specification.pdf.")
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
                write!(f, "Pixel Buffer too small: It can fit only {} pixels, but should be able to fit {} pixels.", received_size, expected_size)
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

use DecodeError::*;

fn decode_header(buffer: &[u8]) -> Result<Header, DecodeError> {
    let header_bytes = match buffer.get(..HEADER_SIZE) {
        Some(bytes) => Ok(bytes),
        None => Err(DecodeError::MissingHeader),
    }?;
    if !header_bytes.starts_with(&MAGIC) {
        let mut res = [0; 4];
        res.clone_from_slice(&header_bytes[0..4]);
        return Err(InvalidMagic(res));
    }

    let header = Header {
        width: u32::from_be_bytes(header_bytes[4..8].try_into().unwrap()),
        height: u32::from_be_bytes(header_bytes[8..12].try_into().unwrap()),
        channels: header_bytes[12]
            .try_into()
            .map_err(|e| InvalidChannels(e))?,
        colorspace: header_bytes[13]
            .try_into()
            .map_err(|e| InvalidColorspace(e))?,
    };
    Ok(header)
}

fn get_prev_pixel<'a>(pixels: &MutBufIter<'a, Pixel>) -> Pixel {
    match pixels.look_one_backward() {
        None => Pixel::default(),
        Some(pixel) => *pixel,
    }
}

// Return value indicates whether the decoding process is done
fn decode_pixels<'a>(
    header: &Header,
    buffer: &mut BufIter<'a, u8>,
    pixels: &mut MutBufIter<'a, Pixel>,
    prev_pixels: &mut [Pixel; PREV_ARR_SIZE],
) -> Result<(), DecodeError> {
    loop {
        // Check if stream is over
        if let Some(bytes) = buffer.look_forward(STREAM_END_SIZE) {
            if bytes.starts_with(&STREAM_END) {
                break;
            }
        }

        // Decode next pixelo
        let pixel = match buffer.step_one() {
            None => Err(DecodeError::missing_pixels(header, pixels)),
            Some(&byte) => match byte & MASK_2 {
                OP_INDEX => {
                    // Demasking isn't needed, since OP_INDEX = 0, but for readability & symmetry sake, it's still here
                    let index = (byte & DEMASK_2) as usize;
                    let px = prev_pixels[index];
                    match pixels.set_next_one(prev_pixels[index]) {
                        None => Err(DecodeError::pixel_buffer_too_small(header, pixels)),
                        Some(_) => Ok(px),
                    }
                }
                OP_DIFF => {
                    let prev_pixel = get_prev_pixel(pixels);
                    let dr = (byte & 0b00110000) >> 4;
                    let dg = (byte & 0b00001100) >> 2;
                    let db = (byte & 0b00000011) >> 0;
                    let px = Pixel {
                        r: prev_pixel.r.wrapping_add(dr).wrapping_sub(DIFF_BIAS),
                        g: prev_pixel.g.wrapping_add(dg).wrapping_sub(DIFF_BIAS),
                        b: prev_pixel.b.wrapping_add(db).wrapping_sub(DIFF_BIAS),
                        a: prev_pixel.a,
                    };
                    match pixels.set_next_one(px) {
                        None => Err(DecodeError::pixel_buffer_too_small(header, pixels)),
                        Some(_) => Ok(px),
                    }
                }
                OP_LUMA => {
                    let prev_pixel = get_prev_pixel(pixels);
                    let second_byte = match buffer.step_one() {
                        None => Err(DecodeError::missing_pixels(header, pixels)),
                        Some(byte) => Ok(byte),
                    }?;
                    let dg = (byte & DEMASK_2) as i8 - LUMA_GREEN_BIAS as i8;
                    let dr = (second_byte & 0b11110000) >> 4;
                    let db = (second_byte & 0b00001111) >> 0;

                    let px = if dg < 0 {
                        Pixel {
                            r: prev_pixel
                                .r
                                .wrapping_sub((-1 * dg) as u8)
                                .wrapping_sub(LUMA_BIAS)
                                .wrapping_add(dr),
                            g: prev_pixel.g.wrapping_sub((-1 * dg) as u8),
                            b: prev_pixel
                                .b
                                .wrapping_sub((-1 * dg) as u8)
                                .wrapping_sub(LUMA_BIAS)
                                .wrapping_add(db),
                            a: prev_pixel.a,
                        }
                    } else {
                        Pixel {
                            r: prev_pixel
                                .r
                                .wrapping_add(dg as u8)
                                .wrapping_sub(LUMA_BIAS)
                                .wrapping_add(dr),
                            g: prev_pixel.g.wrapping_add(dg as u8),
                            b: prev_pixel
                                .b
                                .wrapping_add(dg as u8)
                                .wrapping_sub(LUMA_BIAS)
                                .wrapping_add(db),
                            a: prev_pixel.a,
                        }
                    };

                    match pixels.set_next_one(px) {
                        None => Err(DecodeError::pixel_buffer_too_small(header, pixels)),
                        Some(_) => Ok(px),
                    }
                }
                MASK_2 => match byte | 1 {
                    OP_RGB | OP_RGBA => {
                        let step = if byte == OP_RGB { 3 } else { 4 };
                        match buffer.step_forward(step) {
                            None => Err(DecodeError::missing_pixels(header, pixels)),
                            Some(bytes) => {
                                let px = bytes.into();
                                match pixels.set_next_one(px) {
                                    None => {
                                        Err(DecodeError::pixel_buffer_too_small(header, pixels))
                                    }
                                    Some(_) => Ok(px),
                                }
                            }
                        }
                    }
                    _ => {
                        let run = byte & DEMASK_2;
                        let &px = match pixels.look_one_backward() {
                            None => Err(InvalidEncoding),
                            Some(pixel) => Ok(pixel),
                        }?;

                        for _ in 0..run + 1 {
                            if let None = pixels.set_next_one(px.clone()) {
                                return Err(DecodeError::pixel_buffer_too_small(header, pixels));
                            }
                        }

                        Ok(px)
                    }
                },
                _ => unreachable!(),
            },
        }?;

        // Update prev_pixels to include the newly added pixel
        prev_pixels[pixel.pixel_hash() as usize] = pixel;
    }
    Ok(())
}

pub fn decode<'a>(buffer: &[u8], pixels: &mut [Pixel]) -> Result<Header, DecodeError> {
    let header = decode_header(buffer)?;
    let pixel_amount = header.pixel_amount();
    let mut prev_pixels = [Pixel::def(); PREV_ARR_SIZE];

    let mut pixels = match MutBufIter::from(pixels, ..pixel_amount) {
        None => Err(DecodeError::PixelBufferTooSmall {
            expected_size: pixel_amount,
            received_size: pixels.len(),
        }),
        Some(pixels) => Ok(pixels),
    }?;
    let mut buffer = BufIter::from(buffer, HEADER_SIZE.into()..).unwrap();

    decode_pixels(&header, &mut buffer, &mut pixels, &mut prev_pixels)?;
    Ok(header)
}

pub fn decode_allocated<'a>(buffer: &[u8]) -> Result<(Header, Vec<Pixel>), DecodeError> {
    let header = decode_header(buffer)?;
    let pixel_amount = header.pixel_amount();
    let mut prev_pixels = [Pixel::def(); PREV_ARR_SIZE];

    let mut pixels_vec = vec![Pixel::def(); pixel_amount];
    let mut pixels = match MutBufIter::from(&mut pixels_vec, ..pixel_amount) {
        None => Err(DecodeError::PixelBufferTooSmall {
            expected_size: pixel_amount,
            received_size: pixels_vec.len(),
        }),
        Some(pixels) => Ok(pixels),
    }?;
    let mut buffer = BufIter::from(buffer, HEADER_SIZE.into()..).unwrap();

    decode_pixels(&header, &mut buffer, &mut pixels, &mut prev_pixels)?;
    Ok((header, pixels_vec))
}
