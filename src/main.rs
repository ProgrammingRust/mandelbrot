#![feature(sync_unsafe_cell)]

mod parsing;
mod partition;

use std::cell::SyncUnsafeCell;
use std::env;
use std::num::ParseIntError;
use image::{ImageBuffer, Luma};

use rug::{Complex, Float};
use rug::float::ParseFloatError;
use rug::ops::CompleteRound;
use thiserror::Error as ThisError;
use crate::parsing::{parse_complex, parse_pair};
use crate::partition::{Partition, process_partition};

const PREC: u32 = 40;


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

    // Height in pixels
    height: usize,

    // Complex number at the upper left of this partition.
    cplx_upper_left: Complex,

    // Complex number at the lower_right of this partition.
    cplx_lower_right: Complex,
}

/// Try to determine if `c` is in the Mandelbrot set, using at most `limit`
/// iterations to decide.
///
/// If `c` is not a member, return `Some(i)`, where `i` is the number of
/// iterations it took for `c` to leave the circle of radius two centered on the
/// origin. If `c` seems to be a member (more precisely, if we reached the
/// iteration limit without being able to prove that `c` is not a member),
/// return `None`.
fn escape_time(c: &Complex, limit: usize) -> Option<usize> {
    let four: Float = Float::with_val(PREC, 4.0);

    let mut z = Complex::with_val(PREC, (0.0, 0.0));// { re: 0.0, im: 0.0 };

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
    let (set_width, set_height) =
        ((img_info.cplx_lower_right.real() - img_info.cplx_upper_left.real()).complete(PREC),
         (img_info.cplx_upper_left.imag() - img_info.cplx_lower_right.imag()).complete(PREC));

    Complex::with_val(PREC,
                      (
                          img_info.cplx_upper_left.real() + Float::with_val(PREC, (pixel.0) as f64) * set_width / img_info.width as f64,
                          img_info.cplx_upper_left.imag() - Float::with_val(PREC, (pixel.1) as f64) * set_height / img_info.height as f64
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
    let args: Vec<String> = env::args().collect();

    if args.len() != 5 {
        eprintln!("Usage: {} FILE PIXELS UPPERLEFT LOWERRIGHT",
                  args[0]);
        eprintln!("Example: {} mandel.png 1000x750 -1.20,0.35 -1,0.20",
                  args[0]);
        std::process::exit(1);
    }

    let bounds = parse_pair(&args[2], 'x')
        .expect("error parsing image dimensions");
    let cplx_upper_left = parse_complex(&args[3])
        .expect("error parsing upper left corner point");
    let cplx_lower_right = parse_complex(&args[4])
        .expect("error parsing lower right corner point");

    let image_info = ImageInfo {
        width: bounds.0,
        height: bounds.1,
        cplx_upper_left,
        cplx_lower_right,
    };

    let root_partition = Partition {
        x_offset: 0,
        y_offset: 0,
        width: image_info.width,
        height: image_info.height,
    };

    let mut pixels_vec = vec![0u8; bounds.0 * bounds.1];
    let mut pixels = SyncUnsafeCell::new(pixels_vec.as_mut_slice());

    unsafe {
        process_partition(&image_info, &root_partition, &pixels);
    }

    write_image(&args[1], &mut pixels, bounds)
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
                    cplx_upper_left: Complex::with_val(PREC, (-1.0, 1.0)),
                    cplx_lower_right: Complex::with_val(PREC, (1.0, -1.0))
                }
            ),
            Complex::with_val(PREC, (-0.5, -0.75))
        )
    }

    #[test]
    fn test_parse_complex() {
        assert_eq!(parse_complex("1.25,-0.0625"),
                   Some(Complex::with_val(PREC, (1.25, -0.0625))));
        assert_eq!(parse_complex(",-0.0625"), None);
    }

    #[test]
    fn test_parse_pair() {
        assert_eq!(parse_pair::<i32>("", ','), None);
        assert_eq!(parse_pair::<i32>("10,", ','), None);
        assert_eq!(parse_pair::<i32>(",10", ','), None);
        assert_eq!(parse_pair::<i32>("10,20", ','), Some((10, 20)));
        assert_eq!(parse_pair::<i32>("10,20xy", ','), None);
        assert_eq!(parse_pair::<f64>("0.5x", 'x'), None);
        assert_eq!(parse_pair::<f64>("0.5x1.5", 'x'), Some((0.5, 1.5)));
    }
}