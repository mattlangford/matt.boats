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

pub fn intersect_segment(a: &Line, start: &Vector2f, end: &Vector2f) -> Option<Vector2f> {
    let len_sq = (end - start).norm_squared();
    na::linalg::LU::new(na::Matrix2::<f32>::from_columns(&[
        a.direction,
        start - end,
    ]))
    .solve(&(start - a.start))
    .filter(|tu| tu[0] >= 0.0 && tu[0] < a.length && tu[1] >= 0.0 && (tu[1] * tu[1]) < len_sq)
    .map(|tu| a.start + tu[0] * a.direction)
}

pub fn intersect_ray(a: &Line, start: &Vector2f, direction: &Vector2f) -> Option<Vector2f> {
    na::linalg::LU::new(na::Matrix2::<f32>::from_columns(&[a.direction, -direction]))
        .solve(&(start - a.start))
        .filter(|tu| tu[0] >= 0.0 && tu[0] < a.length && tu[1] >= 0.0)
        .map(|tu| a.start + tu[0] * a.direction)
}

#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;

    fn v(x: f32, y: f32) -> Vector2f {
        na::vector![x, y]
    }

    #[test]
    fn test_intersect() {
        // parallel
        assert_eq!(
            intersect_segment(
                &Line::new_ray(v(0.0, 0.0), v(1.0, 0.0)),
                &v(0.0, 1.0),
                &v(1.0, 1.0)
            ),
            None
        );

        // simple
        assert_eq!(
            intersect_segment(
                &Line::new_ray(v(0.0, 0.0), v(1.0, 0.0)),
                &v(1.0, 1.0),
                &v(1.0, -2.0)
            ),
            Some(v(1.0, 0.0))
        );

        // too short
        assert_eq!(
            intersect_segment(
                &Line::new(v(0.0, 0.0), v(1.0, 0.0), 0.1),
                &v(1.0, 1.0),
                &v(1.0, -10.0)
            ),
            None
        );

        // complex
        assert_eq!(
            intersect_segment(
                &Line::new_segment(v(4.0, 2.0), v(-4.0, 4.0)),
                &v(-2.0, 1.0),
                &v(2.0, 5.0)
            ),
            Some(v(0.0, 3.0))
        );
        assert_eq!(
            intersect_ray(
                &Line::new_segment(v(4.0, 2.0), v(-4.0, 4.0)),
                &v(-2.0, 1.0),
                &v(1.0, 1.0)
            ),
            Some(v(0.0, 3.0))
        );
    }
}
