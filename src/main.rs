#![feature(sync_unsafe_cell)]
#![allow(clippy::needless_return)]

mod partition;
mod cmdline;

use std::cell::SyncUnsafeCell;
use std::env;
use std::num::ParseIntError;
use image::{ImageBuffer, Luma};

use rug::{Complex, Float};
use rug::float::ParseFloatError;
use rug::ops::CompleteRound;
use thiserror::Error as ThisError;
use crate::cmdline::parse_cmdline_args;
use crate::partition::{Partition, process_partition};

#[derive(ThisError, Debug)]
pub enum MyError {
    /// Errors encountered while parsing numbers.
    #[error("Parse error: {0}")]
    ParseError(String),
}

/// Represents the image to be worked on.
/// The primary coordinate system is pixels, with the complex numbers being derived
/// from pixels coordinates.
/// See: [pixel_to_point]
struct ImageInfo {
    /// Width in pixels
    width: usize,

    /// Height in pixels
    height: usize,

    /// Complex number at the upper left of this partition.
    cplx_upper_left: Complex,

    /// Complex number at the lower_right of this partition.
    cplx_lower_right: Complex,

    /// Precision for calculations in bits.
    precision: u32,

    /// Filename for saving the output.
    filename: String,
}

/// Try to determine if `c` is in the Mandelbrot set, using at most `limit`
/// iterations to decide.
///
/// If `c` is not a member, return `Some(i)`, where `i` is the number of
/// iterations it took for `c` to leave the circle of radius two centered on the
/// origin. If `c` seems to be a member (more precisely, if we reached the
/// iteration limit without being able to prove that `c` is not a member),
/// return `None`.
fn escape_time(img_info: &ImageInfo, c: &Complex, limit: usize) -> Option<usize> {
    let precision = img_info.precision;

    let four: Float = Float::with_val(precision, 4.0);

    let mut z = Complex::with_val(precision, (0.0, 0.0));// { re: 0.0, im: 0.0 };

    for i in 0..limit {
        if z.clone().norm().real() > &four {
            return Some(i);
        }
        z = z.square() + c;
    }

    None
}

/// Given the row and column of a pixel in the output image, return the
/// corresponding point on the complex plane.
fn pixel_to_point(pixel: (usize, usize), img_info: &ImageInfo) -> Complex
{
    let precision = img_info.precision;

    let (set_width, set_height) =
        ((img_info.cplx_lower_right.real() - img_info.cplx_upper_left.real()).complete(precision),
         (img_info.cplx_upper_left.imag() - img_info.cplx_lower_right.imag()).complete(precision));

    Complex::with_val(precision,
                      (
                          img_info.cplx_upper_left.real() + Float::with_val(precision, pixel.0) * set_width / Float::with_val(precision, img_info.width) ,
                          img_info.cplx_upper_left.imag() - Float::with_val(precision, pixel.1) * set_height / Float::with_val(precision, img_info.height)
                      ),
                      // Why subtraction here? pixel.1 increases as we go down,
                      // but the imaginary component increases as we go up.
    )
}

/// Write the buffer `pixels`, whose dimensions are given by `bounds`, to the
/// file named `filename`.
fn write_image(filename: &str, pixels: &mut SyncUnsafeCell<&mut [u8]>, bounds: (usize, usize))
               -> Result<(), std::io::Error>
{
    let pixels = pixels.get_mut();

    let width = bounds.0 as u32;
    let height = bounds.1 as u32;

    let mut image_buffer = ImageBuffer::new(width, height);

    for (x, y, pixel) in image_buffer.enumerate_pixels_mut() {

        *pixel = Luma( [pixels[ (x + y  * width) as usize] ])
    }

    image_buffer.save(filename).expect("Image error");

    Ok(())
}



fn main() {
    let image_info = parse_cmdline_args();

    let root_partition = Partition {
        x_offset: 0,
        y_offset: 0,
        width: image_info.width,
        height: image_info.height,
    };

    let mut pixels_vec = vec![0u8; image_info.width * image_info.height];
    let mut pixels = SyncUnsafeCell::new(pixels_vec.as_mut_slice());

    unsafe {
        rayon::scope(|_| process_partition(&image_info, &root_partition, &pixels) );
    }

    write_image(&image_info.filename, &mut pixels, (image_info.width, image_info.height) )
        .expect("error writing PNG file");
}

impl From<rug::float::ParseFloatError> for MyError {
    fn from(rug_err: ParseFloatError) -> Self {
        Self::ParseError(rug_err.to_string())
    }
}

impl From<std::num::ParseFloatError> for MyError {
    fn from(num_err: std::num::ParseFloatError) -> Self {
        Self::ParseError(num_err.to_string())
    }
}

impl From<ParseIntError> for MyError {
    fn from(num_err: ParseIntError) -> Self {
        Self::ParseError(num_err.to_string())
    }
}

// Unit tests
#[cfg(test)]
pub(crate) mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;

    #[test]
    fn test_pixel_to_point() {
        assert_eq!(
            pixel_to_point(
                (25, 175),
                &ImageInfo {
                    width: 100,
                    height: 200,
                    cplx_upper_left: Complex::with_val(40, (-1.0, 1.0)),
                    cplx_lower_right: Complex::with_val(40, (1.0, -1.0)),
                    precision: 40,
                    filename: "".to_string(),
                }
            ),
            Complex::with_val(40, (-0.5, -0.75))
        )
    }
}