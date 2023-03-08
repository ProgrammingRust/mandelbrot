use std::error::Error;
use crate::MyError;

#[derive(Debug, Copy, Clone)]
struct Point<N> {
    pub x: N,
    pub y: N,
}

impl <N>Point<N> {
    pub(crate) fn new(x:N, y:N) -> Self {
        Self{ x, y }
    }
}

fn catmull_rom(control_points: &[Point<f64>], x: f64) -> Result<f64, MyError> {

    let num_points = control_points.len();
    
    if num_points < 4 {
        return Err( MyError::InternalError("There must be at lesst four control points.".to_string()) )
    } else if num_points == 4 {
        // Simple case.
        return Ok( catmull_rom_sub(x, &control_points))
    } else {
        // Find the leftmost four control points which cover this area.

        // Fist, look for the rightmost control point which is less than x.
        let mut index = control_points
            .iter()
            .rev()
            .position(|&p| x <= p.x)
            .unwrap_or(num_points-1);

        if index + 3 > num_points {
            return Ok(catmull_rom_sub(x, &control_points[num_points-4..]))
        }

        return Ok(catmull_rom_sub(x, &control_points[index..index+4]))

    }
}

fn catmull_rom_sub(t: f64, p: &[Point<f64>]) -> f64 {
    assert_eq!(p.len(), 4);

    let alpha: f64 = 0.5;
    let t0: f64 = 0.0;
    let t1 = t0 + alpha * (f64::sqrt(f64::powf(p[1].x - p[0].x, 2.0) + f64::powf(p[1].y - p[0].y, 2.0)));
    let t2 = t1 + alpha * (f64::sqrt(f64::powf(p[2].x - p[1].x, 2.0) + f64::powf(p[2].y - p[1].y, 2.0)));
    let t3 = t2 + alpha * (f64::sqrt(f64::powf(p[3].x - p[2].x, 2.0) + f64::powf(p[3].y - p[2].y, 2.0)));
    let tt = t * (t2 - t1) + t1;

    let a1 = (t1 - tt) / (t1 - t0) * p[0].y + (tt - t0) / (t1 - t0) * p[1].y;
    let a2 = (t2 - tt) / (t2 - t1) * p[1].y + (tt - t1) / (t2 - t1) * p[2].y;
    let a3 = (t3 - tt) / (t3 - t2) * p[2].y + (tt - t2) / (t3 - t2) * p[3].y;

    let b1 = (t2 - tt) / (t2 - t0) * a1 + (tt - t0) / (t2 - t0) * a2;
    let b2 = (t3 - tt) / (t3 - t1) * a2 + (tt - t1) / (t3 - t1) * a3;

    let c = (t2 - tt) / (t2 - t1) * b1 + (tt - t1) / (t2 - t1) * b2;

    return c;
}

// Unit tests
#[cfg(test)]
pub(crate) mod tests {
    use num::range;
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;

    #[test]
    fn test_catmull_rom() {
        let bla:Point<f64> = Point::<f64>::new(0.0, 0.0);

        let mut control_points: [Point<f64>;6] = [
            Point::<f64>::new(0.0, 0.0),
            Point::<f64>::new(465.0, 420.0),
            Point::<f64>::new(1234.0, 40.0),
            Point::<f64>::new(1890.0, 6.0),

            Point::<f64>::new(2520.0, 480.0),
            Point::<f64>::new(2934.0, 485.0)
        ];

        for (i, p) in control_points.iter_mut().enumerate() {
            p.x = p.x/ 2934.0;
            p.y = p.y / 485.0 * 255.0
        }

        println!("{:?}", control_points);

        for i in range(0, 255) {
            let x = i as f64 / 255.0;

            println!("{}\t{}", x, catmull_rom(&control_points, x).expect("catmull_rom() returned an error."));
        }

    }
}