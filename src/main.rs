#![warn(rust_2018_idioms)]
#![allow(elided_lifetimes_in_paths)]

extern crate core;

use thiserror::Error as ThisError;
use rug::{Complex, Float};
use rayon::prelude::*;

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


use std::str::FromStr;

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

#[test]
fn test_parse_pair() {
    assert_eq!(parse_pair::<i32>("",        ','), None);
    assert_eq!(parse_pair::<i32>("10,",     ','), None);
    assert_eq!(parse_pair::<i32>(",10",     ','), None);
    assert_eq!(parse_pair::<i32>("10,20",   ','), Some((10, 20)));
    assert_eq!(parse_pair::<i32>("10,20xy", ','), None);
    assert_eq!(parse_pair::<f64>("0.5x",    'x'), None);
    assert_eq!(parse_pair::<f64>("0.5x1.5", 'x'), Some((0.5, 1.5)));
}

/// Parse a pair of floating-point numbers separated by a comma as a complex
/// number.
fn parse_complex(s: &str) -> Option<Complex> {
    match parse_pair::<Float>(s, ',') {
        Some((re, im)) => Some(Complex::with_val(PREC, (re, im) )),
        None => None
    }
}

#[test]
fn test_parse_complex() {
    assert_eq!(parse_complex("1.25,-0.0625"),
               Some(Complex::with_val(PREC, (1.25, -0.0625))) );
    assert_eq!(parse_complex(",-0.0625"), None);
}

/// Given the row and column of a pixel in the output image, return the
/// corresponding point on the complex plane.
///
/// `bounds` is a pair giving the width and height of the image in pixels.
/// `pixel` is a (column, row) pair indicating a particular pixel in that image.
/// The `upper_left` and `lower_right` parameters are points on the complex
/// plane designating the area our image covers.
fn pixel_to_point(bounds: (usize, usize),
                  pixel: (usize, usize),
                  upper_left: &Complex,
                  lower_right: &Complex)
    -> Complex
{
    let (width, height) =
        ( (lower_right.real() - upper_left.real()).complete(PREC),
          (upper_left.imag() - lower_right.imag()).complete(PREC) );

    Complex::with_val (PREC,
                       (upper_left.real() + Float::with_val(PREC, pixel.0 as f64) * width  / Float::with_val(PREC,bounds.0 as f64),
                             upper_left.imag() - pixel.1 as f64 * height / bounds.1 as f64)
        // Why subtraction here? pixel.1 increases as we go down,
        // but the imaginary component increases as we go up.
    )
}

#[test]
fn test_pixel_to_point() {
    assert_eq!(pixel_to_point((100, 200), (25, 175),
                              &Complex::with_val(PREC, (-1.0, 1.0)),
                              &Complex::with_val(PREC, (  1.0, -1.0 )) ),
               Complex::with_val(PREC, (-0.5, -0.75 )) );
}

/// Render a rectangle of the Mandelbrot set into a buffer of pixels.
///
/// The `bounds` argument gives the width and height of the buffer `pixels`,
/// which holds one grayscale pixel per byte. The `upper_left` and `lower_right`
/// arguments specify points on the complex plane corresponding to the upper-
/// left and lower-right corners of the pixel buffer.
fn render(pixels: &mut [u8],
          bounds: (usize, usize),
          upper_left: &Complex,
          lower_right: &Complex)
{
    assert!(pixels.len() == bounds.0 * bounds.1);

    for row in 0..bounds.1 {
        for column in 0..bounds.0 {
            let point = pixel_to_point(bounds, (column, row),
                                       upper_left, lower_right);
            pixels[row * bounds.0 + column] =
                match escape_time(&point, 255) {
                    None => 0,
                    Some(count) => 255 - count as u8
                };
        }
    }
}

use image::ColorType;
use image::png::PNGEncoder;
use std::fs::File;

/// Write the buffer `pixels`, whose dimensions are given by `bounds`, to the
/// file named `filename`.
fn write_image(filename: &str, pixels: &[u8], bounds: (usize, usize))
    -> Result<(), std::io::Error>
{
    let output = File::create(filename)?;

    let encoder = PNGEncoder::new(output);
    encoder.encode(&pixels,
                   bounds.0 as u32, bounds.1 as u32,
                   ColorType::Gray(8))?;

    Ok(())
}

use std::env;
use std::io::Error;
use std::num::ParseIntError;
use rug::float::ParseFloatError;
use rug::ops::CompleteRound;

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
    let upper_left = parse_complex(&args[3])
        .expect("error parsing upper left corner point");
    let lower_right = parse_complex(&args[4])
        .expect("error parsing lower right corner point");

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
                let band_bounds = (bounds.0, 1);
                let band_upper_left = pixel_to_point(bounds, (0, top),
                                                     &upper_left, &lower_right);
                let band_lower_right = pixel_to_point(bounds, (bounds.0, top + 1),
                                                      &upper_left, &lower_right);
                render(band, band_bounds, &band_upper_left, &band_lower_right);
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
            Err(err) => { Err(MyError::from(err) ) }
        }
    }
}

impl Parseable for usize {
    fn from_str(s: &str) -> Result<Self, MyError> where Self: Sized {
       <usize as FromStr>::from_str(s).map_err(|err| MyError::from(err))
    }
}

impl From<rug::float::ParseFloatError> for MyError {
    fn from(rug_err: ParseFloatError) -> Self {
        Self::ParseError(rug_err.to_string())
    }
}

impl From<ParseIntError> for MyError {
    fn from(num_err: ParseIntError) -> Self {
        Self::ParseError(num_err.to_string())
    }
}