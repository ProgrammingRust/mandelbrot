use std::cell::SyncUnsafeCell;
use image::{ImageBuffer, Luma};

/// Write the buffer `pixels`, whose dimensions are given by `bounds`, to the
/// file named `filename`.
pub(crate) fn write_image(filename: &str, pixels: &mut SyncUnsafeCell<&mut [u8]>, bounds: (usize, usize))
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