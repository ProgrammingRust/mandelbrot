#![allow(non_snake_case)]

use std::process::exit;
use clap::Parser;
use rug::{Complex, Float};
use crate::ImageInfo;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Print detailed program execution information.
    //verbose:bool,

    /// Name of the file in which to save the image.
    #[arg(short,long,default_value_t=String::from("mandelbrot.png"))]
    filename:String,

    /// The number of digits of floating point precision to use for calculations.
    #[arg(long)]
    numDigits:Option<u32>,

    /// The maximum number of iterations to run before bailing out.
    #[arg(short,long, default_value_t=1024)]
    iterations:u32,

    /// x value in the complex plane of the center of the image
    #[arg(short,long, default_value_t=String::from("-0.7"))]
    xCenter:String,

    /// y value in the complex plane of the center of the image
    #[arg(short,long, default_value_t=String::from("0.0"))]
    yCenter:String,

    /// Horizontal scale of the image in the complex plane, will be added to the center point to determine image dimensions
    #[arg(short, long, default_value_t=String::from("1.53845"))]
    scale:String,

    /// Horizontal resolution of the overall image.
    #[arg(long, default_value_t=1680)]
    width:usize,

    /// Vertical resolution of the overall image.
    #[arg(long, default_value_t=1120)]
    height:usize,
    
    /// Amount to scale the palette index by. Larger numbers should produce greater color variation.
    #[arg(short,long, default_value_t=255.0)]
    paletteScaleFactor:f64
}

pub(crate) fn parse_cmdline_args() -> ImageInfo {
    let cli = Cli::parse();

    /* If the numDigits parameter wasn't specified, set numDigits to a super high value temporarily so we can
       perform the following local calculations.

       Near the bottom of this function we will automatically calculate the actual number of digits that will be used
       for generating the image.
    */

    let local_prec = match cli.numDigits {
        None => 200,
        Some(val) => val,
    };

    let scale: Float;
    let image_width: Float;
    let image_height: Float;
    let x_center: Float;
    let y_center: Float;
    let aspect_ratio: Float;

    scale = Float::with_val(local_prec, Float::parse(cli.scale).expect("scale parameter invalid"));

    if scale <=0 {
        eprintln!("The scale parameter must be larger than zero.");
        exit(1);
    }

    image_width = Float::with_val(local_prec, cli.width);
    image_height = Float::with_val(local_prec, cli.height);

    x_center = Float::with_val(local_prec, Float::parse(cli.xCenter).expect("xCenter parameter invalid"));
    y_center = Float::with_val(local_prec, Float::parse(cli.yCenter).expect("yCenter parameter invalid"));

    aspect_ratio = image_height / image_width;

    let y_scale = Float::with_val(local_prec,&aspect_ratio * &scale);

    let x_max = Float::with_val(local_prec,&x_center + &scale);
    let x_min = Float::with_val(local_prec,&x_center - &scale);
    let y_max = Float::with_val(local_prec,&y_center + &y_scale);
    let y_min = Float::with_val(local_prec,&y_center - &y_scale);


    let x_pixel_density = Float::with_val(local_prec,&x_max - &x_min) / cli.width;
    let y_pixel_density = Float::with_val(local_prec,&y_max - &y_min) / cli.height;

    let max_pixel_density: Float;

    // A bit counter-intuitive: The 'maximum' pixel density,
    // or rather greatest pixel density is the _lesser_ of the two numbers.
    if x_pixel_density <= y_pixel_density {
        max_pixel_density = x_pixel_density
    } else {
        max_pixel_density = y_pixel_density
    }

    /* If the numDigits arg was not specified, derive the actual number of digits
       we will use to generate the set.

       Precision is in bits! i.e.: `f64` has a precision of 53.

       See: https://stackoverflow.com/a/10484553
     */
    let precision:u32 = match cli.numDigits {
        Some(val) => val,
        None => {
            // d = Num decimal digits required.
            let d = max_pixel_density.log10().to_f64();

            // b = num binary digits required.
            // b = d / log(2)

            let b = d / std::f64::consts::LOG10_2;

            let mut precision = b as u32;

            if precision < 20 {
                precision = 20;
            }

            precision
        }
    };

    /*
    println!("aspect_ratio:\t{}", aspect_ratio);
    println!("scale:\t{}", scale);
    println!("upper_left:\t({}, {})", x_min, y_min);
    println!("lower_right:\t({}, {})", x_max, y_max);
    */

    return ImageInfo {
        width: cli.width,
        height: cli.height,
        cplx_upper_left: Complex::with_val(precision, (x_min, y_min)),
        cplx_lower_right: Complex::with_val(precision, (x_max, y_max)),
        precision,
        filename: cli.filename,
    }

}