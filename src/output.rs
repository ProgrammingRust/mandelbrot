use std::cell::SyncUnsafeCell;
use std::ops::Div;
use image::{ImageBuffer, Luma, Rgb, RgbImage};
use rug::Float;
use crate::ImageInfo;
use crate::math::Iteration;

/// Write the buffer `pixels`, whose dimensions are given by `bounds`, to the
/// file named `filename`.
pub(crate) fn write_image(image_info: &ImageInfo, palette: Vec<Rgb<u8>>, pixels: &mut SyncUnsafeCell<&mut [Option<Iteration>]>)
                          -> Result<(), std::io::Error>
{
    let pixels = pixels.get_mut();

    let width = image_info.width as u32;
    let height = image_info.height as u32;

    let mut image_buffer = RgbImage::new(width, height);
    let black = Rgb::from([0,0,0]);

    for (x, y, pixel) in image_buffer.enumerate_pixels_mut() {
        let iteration = pixels[(x + y * width) as usize].clone();

        let palette_index = smooth_colour_index(image_info, iteration);

        match palette_index {
            None => { *pixel = black }
            Some(index) => { *pixel = palette[index]; }
        }
    }

    image_buffer.save(&image_info.filename).expect("Image error");

    Ok(())
}

pub(crate) fn smooth_colour_index(image_info: &ImageInfo, iteration: Option<Iteration>) -> Option<usize> {

    if iteration.is_none() {
        return None;
    } else {
        let iteration = iteration.as_ref().unwrap();
        let prec = image_info.precision;
        // The code below computes: nSmooth := float64(n + 1) - math.Log( math.Log( cmplx.Abs(zn) ))/math.Log(2)
        // See: http://linas.org/art-gallery/escape/smooth.html

        let ln_2 = Float::with_val(prec, 2.0).ln();

        // log_log_abs = ln(ln(abs))/ln(2)
        let log_log_abs = iteration.norm.clone().ln().ln().div(ln_2);

        // n_smoothed = (iteration.n + 1) - logAbs
        let n_smoothed = iteration.n + 1 - log_log_abs;

        return Some(n_smoothed.to_u32_saturating().unwrap() as usize );
    }
}