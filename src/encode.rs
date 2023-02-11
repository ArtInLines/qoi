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

impl EncodeError {
    pub fn buffer_too_small<T, S>(header: &Header, buffer: &T) -> Self
    where
        T: BufIterType<S>,
        S: Copy,
    {
        BufferTooSmall {
            expected_size: header.max_size(),
            received_size: buffer.len(),
        }
    }
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
                write!(f, "Output Buffer too small: Only {} bytes were received. Try again with a size of at least {} bytes", received_size, expected_size)
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

#[derive(Debug)]
enum EncodeAttemptRes {
    Success,
    Invalid,
    Failure(EncodeError),
}

impl From<EncodeAttemptRes> for Result<(), EncodeError> {
    fn from(value: EncodeAttemptRes) -> Self {
        match value {
            Success => Ok(()),
            Invalid => Ok(()),
            Failure(err) => Err(err),
        }
    }
}

use EncodeAttemptRes::*;
use EncodeError::*;

fn encode_header<'a>(
    header: &Header,
    buffer: &mut MutBufIter<'a, u8>,
) -> Result<usize, EncodeError> {
    if buffer.len() < HEADER_SIZE {
        return Err(EncodeError::buffer_too_small(header, buffer));
    }
    buffer.copy_from_slice(&MAGIC, 4, true);
    buffer.copy_from_slice(&header.width.to_be_bytes(), 4, true);
    buffer.copy_from_slice(&header.height.to_be_bytes(), 4, true);
    buffer.copy_from_slice(&[header.channels.into(), header.colorspace.into()], 2, true);
    Ok(HEADER_SIZE)
}

fn try_op_run(
    header: &Header,
    pixel: Pixel,
    buffer: &mut MutBufIter<u8>,
    prev_pixel: &Pixel,
    run: &mut u8,
    is_last: bool,
    is_first: bool,
) -> EncodeAttemptRes {
    if is_first {
        // See https://github.com/phoboslab/qoi/issues/258
        Invalid
    } else if pixel == *prev_pixel {
        *run += 1;
        if *run == 62 || is_last {
            if let None = buffer.set_next_one(OP_RUN | (*run - 1)) {
                return Failure(EncodeError::buffer_too_small(header, buffer));
            }
            *run = 0;
        }
        Success
    } else if *run > 0 {
        match buffer.set_next_one(OP_RUN | (*run - 1)) {
            None => Failure(EncodeError::buffer_too_small(header, buffer)),
            Some(_) => {
                *run = 0;
                Success
            }
        }
    } else {
        Invalid
    }
}

fn try_op_index(
    header: &Header,
    pixel: Pixel,
    index: usize,
    buffer: &mut MutBufIter<u8>,
    prev_arr: &mut [Pixel; PREV_ARR_SIZE],
) -> EncodeAttemptRes {
    if prev_arr[index] == pixel {
        match buffer.set_next_one(OP_INDEX | index as u8) {
            None => Failure(EncodeError::buffer_too_small(header, buffer)),
            Some(_) => Success,
        }
    } else {
        Invalid
    }
}

fn try_op_diff_luma(
    header: &Header,
    pixel: Pixel,
    buffer: &mut MutBufIter<u8>,
    prev_pixel: &Pixel,
) -> EncodeAttemptRes {
    if pixel.a == prev_pixel.a {
        let dr = pixel.r.wrapping_sub(prev_pixel.r).wrapping_add(DIFF_BIAS);
        let dg = pixel.g.wrapping_sub(prev_pixel.g).wrapping_add(DIFF_BIAS);
        let db = pixel.b.wrapping_sub(prev_pixel.b).wrapping_add(DIFF_BIAS);

        if dr <= 3 && dg <= 3 && db <= 3 {
            return match buffer.set_next_one(OP_DIFF | (dr << 4) | (dg << 2) | (db << 0)) {
                None => Failure(EncodeError::buffer_too_small(header, buffer)),
                Some(_) => Success,
            };
        }

        let dg = dg.wrapping_add(LUMA_GREEN_BIAS - DIFF_BIAS);
        let dr = dr.wrapping_sub(dg).wrapping_add(LUMA_BIAS);
        let db = db.wrapping_sub(dg).wrapping_add(LUMA_BIAS);

        if dg <= 63 && dr <= 15 && db <= 15 {
            return match buffer.set_next(&[OP_LUMA | dg, (dr << 4) | (db << 0)]) {
                None => Failure(EncodeError::buffer_too_small(header, buffer)),
                Some(_) => Success,
            };
        }
    }
    Invalid
}

fn try_op_pixel(
    header: &Header,
    pixel: Pixel,
    prev_pixel: &Pixel,
    buffer: &mut MutBufIter<u8>,
) -> EncodeAttemptRes {
    if header.channels == ColorChannel::RGB || prev_pixel.a == pixel.a {
        match buffer.step_forward_mut(4) {
            None => Failure(EncodeError::buffer_too_small(header, buffer)),
            Some(bytes) => {
                bytes[0] = OP_RGB;
                bytes[1..4].copy_from_slice(&[pixel.r, pixel.g, pixel.b]);
                Success
            }
        }
    } else {
        match buffer.step_forward_mut(5) {
            None => Failure(EncodeError::buffer_too_small(header, buffer)),
            Some(bytes) => {
                bytes[0] = OP_RGBA;
                bytes[1..5].copy_from_slice(&[pixel.r, pixel.g, pixel.b, pixel.a]);
                Success
            }
        }
    }
}

fn encode_pixel<'a>(
    header: &Header,
    pixels: &mut BufIter<'a, Pixel>,
    buffer: &mut MutBufIter<'a, u8>,
    prev_arr: &mut [Pixel; PREV_ARR_SIZE],
    prev_pixel: &Pixel,
    run: &mut u8,
    is_last: bool,
    is_first: bool,
) -> Result<Pixel, EncodeError> {
    let &pixel = match pixels.step_one() {
        None => Err(MissingPixels {
            expected_size: header.pixel_amount(),
            received_size: pixels.len(),
        }),
        Some(pixel) => Ok(pixel),
    }?;

    let index = pixel.pixel_hash() as usize;
    let res = Ok(pixel);
    let res = match try_op_run(header, pixel, buffer, prev_pixel, run, is_last, is_first) {
        Success => res,
        Failure(e) => Err(e),
        Invalid => match try_op_index(header, pixel, index, buffer, prev_arr) {
            Success => res,
            Failure(e) => Err(e),
            Invalid => match try_op_diff_luma(header, pixel, buffer, prev_pixel) {
                Success => res,
                Failure(e) => Err(e),
                Invalid => match try_op_pixel(header, pixel, prev_pixel, buffer) {
                    Success => res,
                    Failure(e) => Err(e),
                    Invalid => unreachable!(),
                },
            },
        },
    };
    prev_arr[index] = pixel;
    res
}

pub fn encode<'a>(
    header: &Header,
    pixels: &[Pixel],
    buffer: &mut [u8],
) -> Result<usize, EncodeError> {
    let pixel_amount = header.pixel_amount();
    let mut pixels = match BufIter::from(pixels, ..pixel_amount) {
        None => Err(MissingPixels {
            expected_size: pixel_amount,
            received_size: pixels.len(),
        }),
        Some(pixels) => Ok(pixels),
    }?;
    let mut buffer = MutBufIter::new(buffer);

    let mut prev_arr = [Pixel::def(); PREV_ARR_SIZE];
    let mut prev_pixel = Pixel::def();
    let mut run = 0;
    let mut is_first = true;

    encode_header(header, &mut buffer)?;
    while pixels.idx() < pixel_amount {
        let is_last = pixels.idx() == pixel_amount - 1;
        prev_pixel = encode_pixel(
            header,
            &mut pixels,
            &mut buffer,
            &mut prev_arr,
            &prev_pixel,
            &mut run,
            is_last,
            is_first,
        )?;
        is_first = false;
    }

    buffer.copy_from_slice(&STREAM_END, STREAM_END_SIZE, true);

    Ok(buffer.idx())
}

pub fn encode_allocated(header: &Header, pixels: &[Pixel]) -> Result<Vec<u8>, EncodeError> {
    let mut buffer = vec![0; header.max_size()];
    let out_size = encode(header, pixels, &mut buffer)?;
    buffer.truncate(out_size);
    Ok(buffer)
}
