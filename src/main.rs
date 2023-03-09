#![feature(sync_unsafe_cell)]
#![allow(clippy::needless_return)]
#![allow(clippy::needless_late_init)]

mod partition;
mod cmdline;
mod math;
mod output;
mod splines;
mod palette;

use std::cell::SyncUnsafeCell;

use rug::Complex;
use crate::cmdline::parse_cmdline_args;
use crate::output::write_image;
use crate::partition::{Partition, process_partition};
use thiserror::Error as ThisError;
use crate::math::Pixel;
use crate::palette::generate_palette;

#[derive(ThisError, Debug)]
pub enum MyError {
    /// Unrecoverable logic errors.
    #[error("Internal error: {0}")]
    InternalError(String),
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

    /// Number of iterations.
    iterations: usize,

    /// Filename for saving the output.
    filename: String,
}

fn main() {
    let image_info = parse_cmdline_args();

    let root_partition = Partition {
        x_offset: 0,
        y_offset: 0,
        width: image_info.width,
        height: image_info.height,
    };

    let mut pixels_vec:Vec<Option<Pixel>> = vec![None; image_info.width * image_info.height];
    let mut pixels = SyncUnsafeCell::new(pixels_vec.as_mut_slice());

    unsafe {
        rayon::scope(|_| process_partition(&image_info, &root_partition, &pixels) );
    }

    let palette = generate_palette(image_info.iterations);

    write_image(&image_info, palette, &mut pixels);
}
