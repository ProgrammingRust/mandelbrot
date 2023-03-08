use image::Rgb;
use crate::splines::MonotonicCubicSpline;


pub(crate) fn generate_palette(num_entries: usize) -> Vec<Rgb<u8>> {
    assert!(num_entries > 0);

    // Palette is based on the default palette from Ultra Fractal.
    // See: https://stackoverflow.com/a/25816111 for more information.
    let x = vec![0.0, 0.15848670756646216, 0.42058623040218135, 0.6441717791411042, 0.8588957055214724, 1.0];

    let y_red = vec![0.0, 0.128364389, 0.917184265, 0.98757764, 0.004140787, 0.0].iter().map(|y| y * 255.0).collect::<Vec<f64>>();
    let y_green = vec![0.0, 0.424430642, 1.0, 0.677018634, 0.026915114, 0.039337474].iter().map(|y| y * 255.0).collect::<Vec<f64>>();
    let y_blue = vec![0.399585921, 0.813664596, 1.0, 0.022774327, 0.014492754, 0.399585921].iter().map(|y| y * 255.0).collect::<Vec<f64>>();

    let red_spline = MonotonicCubicSpline::partial(x.clone(), y_red.clone());
    let green_spline = MonotonicCubicSpline::partial(x.clone(), y_green.clone());
    let blue_spline = MonotonicCubicSpline::partial(x.clone(), y_blue.clone());

    let mut result:Vec<Rgb<u8>> = vec![];

    let step = 1.0 / (num_entries as f64);
    let mut x=0.0;

    for _i in 0..num_entries {

        result.push(Rgb::from([red_spline(x) as u8, green_spline(x) as u8, blue_spline(x) as u8]));

        x += step;
    }

    return result;
}