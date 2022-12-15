//mod rectangle;

use std::env;
use std::num::ParseIntError;
use std::str::FromStr;
use image::{ImageBuffer, Luma};

use rayon::prelude::*;
use rug::{Complex, Float};
use rug::float::ParseFloatError;
use rug::ops::CompleteRound;
use thiserror::Error as ThisError;

const PREC: u32 = 40;


#[derive(ThisError, Debug)]
pub enum MyError {
    /// Errors encountered while parsing numbers.
    #[error("Parse error: {0}")]
    ParseError(String),
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


/// Parse the string `s` as a coordinate pair, like `"400x600"` or `"1.0,0.5"`.
///
/// Specifically, `s` should have the form <left><sep><right>, where <sep> is
/// the character given by the `separator` argument, and <left> and <right> are both
/// strings that can be parsed by `T::from_str`.
///
/// If `s` has the proper form, return `Some<(x, y)>`. If it doesn't parse
/// correctly, return `None`.
fn parse_pair<T: Parseable>(s: &str, separator: char) -> Option<(T, T)> {
    match s.find(separator) {
        None => None,
        Some(index) => {
            match (T::from_str(&s[..index]), T::from_str(&s[index + 1..])) {
                (Ok(l), Ok(r)) => Some((l, r)),
                _ => None
            }
        }
    }
}


/// Parse a pair of floating-point numbers separated by a comma as a complex
/// number.
fn parse_complex(s: &str) -> Option<Complex> {
    match parse_pair::<Float>(s, ',') {
        Some((re, im)) => Some(Complex::with_val(PREC, (re, im))),
        None => None
    }
}

/// Represents a subset of the image to be worked on.
/// The primary coordinate system is pixels, with the complex numbers being derived
/// from pixels coordinates.
/// See: [pixel_to_point]
struct Partition<'p> {
    /// Width in pixels
    width: usize,

    // Height in pixels
    height: usize,

    // Complex number at the upper left of this partition.
    cplx_upper_left: &'p Complex,

    // Complex number at the lower_right of this partition.
    cplx_lower_right: &'p Complex,
}

/// Given the row and column of a pixel in the output image, return the
/// corresponding point on the complex plane.
fn pixel_to_point(pixel: (usize, usize), p: &Partition) -> Complex
{
    let (set_width, set_height) =
        ((p.cplx_lower_right.real() - p.cplx_upper_left.real()).complete(PREC),
         (p.cplx_upper_left.imag() - p.cplx_lower_right.imag()).complete(PREC));

    Complex::with_val(PREC,
                      (
                          p.cplx_upper_left.real() + Float::with_val(PREC, pixel.0 as f64) * set_width / p.width as f64,
                          p.cplx_upper_left.imag() - Float::with_val(PREC,pixel.1 as f64) * set_height / p.height as f64
                      ),
                      // Why subtraction here? pixel.1 increases as we go down,
                      // but the imaginary component increases as we go up.
    )
}

/// Render a rectangle of the Mandelbrot set into a buffer of pixels.
///
/// The `bounds` argument gives the width and height of the buffer `pixels`,
/// which holds one grayscale pixel per byte. The `upper_left` and `lower_right`
/// arguments specify points on the complex plane corresponding to the upper-
/// left and lower-right corners of the pixel buffer.
fn render(pixels: &mut [u8],  p: &Partition)
{
    assert!(pixels.len() == p.width * p.height);

    for row in 0..p.height {
        for column in 0..p.width {
            let point = pixel_to_point( (column, row),p);
            pixels[row * p.width + column] =
                match escape_time(&point, 255) {
                    None => 0,
                    Some(count) => 255 - count as u8
                };
        }
    }
}


/// Write the buffer `pixels`, whose dimensions are given by `bounds`, to the
/// file named `filename`.
fn write_image(filename: &str, pixels: &[u8], bounds: (usize, usize))
               -> Result<(), std::io::Error>
{
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
    let cplx_upper_left = &parse_complex(&args[3])
        .expect("error parsing upper left corner point");
    let cplx_lower_right = &parse_complex(&args[4])
        .expect("error parsing lower right corner point");

    let partition = Partition {
        width: bounds.0,
        height: bounds.1,
        cplx_upper_left,
        cplx_lower_right,
    };

    let mut pixels = vec![0; bounds.0 * bounds.1];

    // Scope of slicing up `pixels` into horizontal bands.
    {
        let bands: Vec<(usize, &mut [u8])> = pixels
            .chunks_mut(bounds.0)
            .enumerate()
            .collect();

        bands.into_par_iter()
            .for_each(|(i, band)| {
                let top = i;

                let band_partition = Partition {
                    width: partition.width,
                    height: 1,
                    cplx_upper_left: &pixel_to_point((0, top), &partition),
                    cplx_lower_right: &pixel_to_point((partition.width, top + 1), &partition),
                };
                
                render(band, &band_partition);
            });
    }

    write_image(&args[1], &pixels, bounds)
        .expect("error writing PNG file");
}

trait Parseable {
    fn from_str(s: &str) -> Result<Self, MyError> where Self: Sized;
}

impl Parseable for Float {
    fn from_str(s: &str) -> Result<Self, MyError> where Self: Sized {
        let incomplete = Float::parse(s);

        match incomplete {
            Ok(incomplete) => { Ok(incomplete.complete(PREC)) }
            Err(err) => { Err(MyError::from(err)) }
        }
    }
}

impl Parseable for usize {
    fn from_str(s: &str) -> Result<Self, MyError> where Self: Sized {
        <usize as FromStr>::from_str(s).map_err(|err| MyError::from(err))
    }
}

impl Parseable for i32 {
    fn from_str(s: &str) -> Result<Self, MyError> where Self: Sized {
        <i32 as FromStr>::from_str(s).map_err(|err| MyError::from(err))
    }
}

impl Parseable for f64 {
    fn from_str(s: &str) -> Result<Self, MyError> where Self: Sized {
        <f64 as FromStr>::from_str(s).map_err(|err| MyError::from(err))
    }
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
                &Partition {
                    width: 100,
                    height: 200,
                    cplx_upper_left: &Complex::with_val(PREC, (-1.0, 1.0)),
                    cplx_lower_right: &Complex::with_val(PREC, (1.0, -1.0))
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