use std::cell::SyncUnsafeCell;
use image::{ImageBuffer, Luma, Rgb, RgbImage};
use crate::math::Iteration;

/// Write the buffer `pixels`, whose dimensions are given by `bounds`, to the
/// file named `filename`.
pub(crate) fn write_image(filename: &str, pixels: &mut SyncUnsafeCell<&mut [Option<Iteration>]>, bounds: (usize, usize), palette: Vec<Rgb<u8>>)
                          -> Result<(), std::io::Error>
{
    let pixels = pixels.get_mut();

    let width = bounds.0 as u32;
    let height = bounds.1 as u32;

    let mut image_buffer = RgbImage::new(width, height);
    let black = Rgb::from([0,0,0]);

    for (x, y, pixel) in image_buffer.enumerate_pixels_mut() {
        let pixel_iteration_value = &pixels[(x + y * width) as usize];

        match pixel_iteration_value {
            None => { *pixel = black }
            Some(it) => { *pixel = palette[it.n]; }
        }
    }

    image_buffer.save(filename).expect("Image error");

    Ok(())
}

