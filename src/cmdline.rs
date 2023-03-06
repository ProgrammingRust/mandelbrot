#![allow(non_snake_case)]
use clap::Parser;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Print detailed program execution information.
    verbose:bool,

    /// Name of the file in which to save the image.
    #[arg(default_value_t=String::from("out.png"))]
    filename:String,

    /// The number of digits of floating point precision to use for calculations.
    numDigits:Option<u32>,

    /// The maximum number of iterations to run before bailing out.
    #[arg(default_value_t=1024)]
    iterations:u32,

    /// x value in the complex plane of the center of the image
    #[arg(default_value_t=String::from("-0.7"))]
    xCenter:String,

    /// y value in the complex plane of the center of the image
    #[arg(default_value_t=String::from("0.0"))]
    yCenter:String,

    /// Horizontal scale of the image in the complex plane, will be added to the center point to determine image dimensions
    #[arg(default_value_t=String::from("1.53845"))]
    scale:String,

    /// Horizontal resolution of the overall image.
    #[arg(default_value_t=1680)]
    width:u32,

    /// Vertical resolution of the overall image.
    #[arg(default_value_t=1120)]
    height:u32,
    
    /// Amount to scale the palette index by. Larger numbers should produce greater color variation.
    #[arg(default_value_t=255.0)]
    paletteScaleFactor:f64
}