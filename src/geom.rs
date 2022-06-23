use nalgebra as na;

pub type Vector2f = na::Vector2<f32>;

#[derive(Debug, Default)]
pub struct Line {
    start: Vector2f,
    direction: Vector2f,
    length: f32,
}

impl Line {
    pub fn new_segment(start: Vector2f, end: Vector2f) -> Line {
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
    na::linalg::LU::new(na::Matrix2::<f32>::from_columns(&[
        a.direction,
        -b.direction,
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
    fn test_intersect() {
        // parallel
        assert_eq!(
            intersect_lines(
                &Line::new_ray(na::vector![0.0, 0.0], na::vector![1.0, 0.0]),
                &Line::new_ray(na::vector![1.0, 1.0], na::vector![1.0, 0.0])
            ),
            None
        );

        // simple
        assert_eq!(
            intersect_lines(
                &Line::new_ray(na::vector![0.0, 0.0], na::vector![1.0, 0.0]),
                &Line::new_ray(na::vector![1.0, 1.0], na::vector![0.0, -1.0])
            ),
            Some(na::vector![1.0, 0.0])
        );

        // too short
        assert_eq!(
            intersect_lines(
                &Line::new(na::vector![0.0, 0.0], na::vector![1.0, 0.0], 0.1),
                &Line::new_ray(na::vector![1.0, 1.0], na::vector![0.0, -1.0])
            ),
            None
        );

        // complex
        assert_eq!(
            intersect_lines(
                &Line::new_segment(na::vector![4.0, 2.1], na::vector![-4.0, 4.1]),
                &Line::new_segment(na::vector![-2.0, 1.1], na::vector![2.0, 5.1])
            ),
            Some(na::vector![0.0, 3.1])
        );
    }
}
