/*
Wed 14 Dec 2022 22:55:53 EST
The following is a conversation with an AI assistant. The assistant is helpful, creative, clever, and very friendly.

Human: Hello, who are you?
AI: I am an AI created by OpenAI. How can I help you today?
Human: Please convert this Go code to Rust:
...
AI: Here is the equivalent code in Rust:
*/
use std::cell::SyncUnsafeCell;
use rayon::Scope;
use crate::{escape_time, ImageInfo, pixel_to_point};

/// Represents a subset of the image to be worked on.
/// The primary coordinate system is pixels, with the complex numbers being derived
/// from pixels coordinates.
/// See: [pixel_to_point]
///
#[derive(Debug)]
pub(crate) struct Partition {
    // x coordinates of the upper left corner, in pixels.
    pub(crate) x_offset: usize,

    // y coordinates of the upper left corner, in pixels.
    pub(crate) y_offset: usize,

    /// Width in pixels
    pub(crate) width: usize,

    // Height in pixels
    pub(crate) height: usize,
}

impl Partition {
    pub(crate) fn from_points(x1: usize, y1: usize, x2: usize, y2: usize) -> Self {
        Partition {
            x_offset: x1,
            y_offset: y1,
            width: x2 - x1 + 1,
            height: y2 - y1 + 1,
        }
    }
}

const ESCAPE_TIME: usize = 255;

pub(crate) unsafe fn process_partition(image_info: &ImageInfo, p: Partition, pixels: &mut [u8]) -> Option<Vec<Partition>> {
    let mut pixels_processed: u64 = 0;

    let mut perimeter_in_set = true;

    let min_x = p.x_offset;
    let max_x = min_x + p.width - 1;

    let x_values = [min_x, max_x];

    let min_y = p.y_offset;
    let max_y = min_y + p.height - 1;

    let y_values = [min_y, max_y];

    // Check the top and bottom of the rectangle
    for y in y_values {
        for x in min_x..=max_x {
            let escape_time = process_point(x, y, pixels, image_info);

            if escape_time.is_some() {
                perimeter_in_set = false;
            }
        }
    }

    // Check the left and right sides of the rectangle
    for x in x_values {
        for y in min_y..=max_y {
            let escape_time = process_point(x, y, pixels, image_info);

            if escape_time.is_some() {
                perimeter_in_set = false;
            }
        }
    }

    /* Since the mandelbrot set is a connected set, if the perimeter of the rectangle is in the set,
       Then this means that the inside of the rectangle must also be in the set. When this happens, we
       fill in the entire inside of the rectangle with the 'set' color (black) and exit without doing any further work */
    if perimeter_in_set {
        println!("Perimeter in set: {:?}\n", p);
        for x in min_x + 1..max_x {
            for y in min_y + 1..max_y {
                set_pixel(None, x, y, pixels, image_info);
                pixels_processed += 1;
            }
        }
        return None;
        //If we encounter these little rectangles, we just compute their points individually.
    } else if p.width <= 2 || p.height <= 2 {
        for x in min_x..=max_x {
            for y in min_y..=max_y {
                println!("Base case: width: {} height: {}\n", p.width, p.height);
                process_point(x, y, pixels, image_info);
            }
        }
        return None;
    // Split the current rectangle up into four rectangles and recurse.
    } else {
        let mut x_midpoint;
        let mut y_midpoint;
        let mut x_midpoint_plus_one;
        let mut y_midpoint_plus_one;

        let width = max_x - min_x;
        let height = max_y - min_y;

        x_midpoint = min_x + width / 2 + width % 2;
        if x_midpoint < min_x {
            x_midpoint = min_x;
        }

        y_midpoint = min_y + height / 2 + height % 2;
        if y_midpoint < min_y {
            y_midpoint = min_y;
        }

        x_midpoint_plus_one = x_midpoint + 1;
        if x_midpoint_plus_one > max_x {
            x_midpoint_plus_one = max_x;
        }

        y_midpoint_plus_one = y_midpoint + 1;
        if y_midpoint_plus_one > max_y {
            y_midpoint_plus_one = max_y;
        }

        let upper_left = Partition::from_points(min_x, min_y, x_midpoint, y_midpoint);
        println!("Upper Left: {:03?}", upper_left);

        let upper_right = Partition::from_points(x_midpoint_plus_one, min_y, max_x, y_midpoint);
        println!("Upper Right: {:03?}", upper_right);

        let lower_left = Partition::from_points(min_x, y_midpoint_plus_one, x_midpoint, max_y);
        println!("Lower Left: {:03?}", lower_left);

        let lower_right = Partition::from_points(x_midpoint_plus_one, y_midpoint_plus_one, max_x, max_y);
        println!("Lower Right: {:03?}\n", lower_right);

        return Some(vec![upper_left, upper_right, lower_left, lower_right])
    }
}

unsafe fn process_point(x: usize, y: usize, pixels: &mut [u8], image_info: &ImageInfo) -> Option<usize> {
    let point = pixel_to_point((x, y), image_info);
    let escape_time = escape_time(&point, ESCAPE_TIME);

    set_pixel(escape_time, x, y, pixels, image_info);

    return escape_time;
}

unsafe fn set_pixel(value: Option<usize>, x: usize, y: usize, pixels: &mut [u8], image_info: &ImageInfo) {
    let i = y * image_info.width + x;

    pixels[i] =  match value {
            None => 0,  // Point is in set if there is no escape time.
            Some(count) => 255 - count as u8
        };
}


