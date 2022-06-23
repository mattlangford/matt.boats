use nalgebra as na;

pub type Vector2f = na::Vector2<f32>;

pub struct Line {
    start: Vector2f,
    direction: Vector2f,
    length: f32,
}

impl Line {
    pub fn new_from_start_and_end(start: Vector2f, end: Vector2f) -> Line {
        let diff = end - start;
        let length = diff.norm();
        Line::new(start, diff, length)
    }

    pub fn new_ray(start: Vector2f, direction: Vector2f) -> Line {
        Line::new(start, direction, std::f32::INFINITY)
    }

    pub fn new(start: Vector2f, direction: Vector2f, length: f32) -> Line {
        Line {
            start: start,
            direction: direction.normalize(),
            length: length,
        }
    }
}

pub fn intersect_lines(a: &Line, b: &Line) -> Option<Vector2f> {
    na::linalg::LU::new(na::Matrix2::<f32>::from_rows(&[
        a.direction.transpose(),
        -b.direction.transpose(),
    ]))
    .solve(&(b.start - a.start))
    .filter(|tu| tu[0] >= 0.0 && tu[0] < a.length && tu[1] >= 0.0 && tu[1] < b.length)
    .map(|tu| a.start + tu[0] * a.direction)
}

#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;

    #[test]
    fn test_add() {
        let a = Line::new_ray(na::vector![0.0, 0.0], na::vector![1.0, 0.0]);
        let b = Line::new_ray(na::vector![1.0, 1.0], na::vector![0.0, -1.0]);
        println!("{:?}", intersect_lines(&a, &b));

        assert_eq!(3, 2);
    }
}
