use std::cell::SyncUnsafeCell;
use std::ops::Div;
use image::{ImageBuffer, Luma, Rgb, RgbImage};
use rug::Float;
use crate::errors::MyError;
use crate::ImageInfo;
use crate::math::{Iteration, Pixel};

/// Write the buffer `pixels`, whose dimensions are given by `bounds`, to the
/// file named `filename`.
pub(crate) fn write_image(image_info: &ImageInfo, palette: Vec<Rgb<u8>>, pixels: &mut SyncUnsafeCell<&mut [Option<Pixel>]>)
                          -> Result<(), MyError>
{
    let pixels = pixels.get_mut();

    let width = image_info.width as u32;
    let height = image_info.height as u32;

    let mut image_buffer = RgbImage::new(width, height);
    let black = Rgb::from([0,0,0]);

    for (x, y, pixel) in image_buffer.enumerate_pixels_mut() {
        let iteration = pixels[(x + y * width) as usize].clone();


        match iteration {
            None => { *pixel = black }
            Some(index_u16) => {
                let index: usize = index_u16 as usize;
                *pixel = palette[index.clamp(0, (palette.len() - 1) as usize )];
            }
        }
    }

    image_buffer.save(&image_info.filename)?;

    Ok(())
}

pub(crate) fn smooth_colour_index(image_info: &ImageInfo, it: &Iteration) -> Pixel {

    let prec = image_info.precision;

    let n = Float::with_val(prec, it.n);

    // The code below computes: nSmooth := float64(n + 1) - math.Log( math.Log( cmplx.Abs(zn) ))/math.Log(2)
    // See: http://linas.org/art-gallery/escape/smooth.html

    let ln_2 = Float::with_val(prec, 2.0).ln();

    // log_log_abs = ln(ln(abs))/ln(2)
    let log_log_abs = it.norm.clone().ln().ln().div(ln_2);

    // n_smoothed = (iteration.n + 1) - logAbs
    let n_smoothed:Float = (n + 1) - log_log_abs;

    return n_smoothed.to_u32_saturating().unwrap() as Pixel;
}