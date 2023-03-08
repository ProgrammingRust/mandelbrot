use rug::{Complex, Float};
use rug::ops::CompleteRound;
use crate::ImageInfo;

/// Try to determine if `c` is in the Mandelbrot set, using at most `limit`
/// iterations to decide.
///
/// If `c` is not a member, return `Some(i)`, where `i` is the number of
/// iterations it took for `c` to leave the circle of radius two centered on the
/// origin. If `c` seems to be a member (more precisely, if we reached the
/// iteration limit without being able to prove that `c` is not a member),
/// return `None`.
pub(crate) fn escape_time(img_info: &ImageInfo, c: &Complex, limit: usize) -> Option<usize> {
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
pub(crate) fn pixel_to_point(pixel: (usize, usize), img_info: &ImageInfo) -> Complex
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
                    iterations: 255,
                    filename: "".to_string(),
                }
            ),
            Complex::with_val(40, (-0.5, -0.75))
        )
    }
}