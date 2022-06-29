use nalgebra as na;
use rand::distributions::Distribution;

pub type Vec2f = na::Vector2<f32>;

#[derive(Debug, Default)]
pub struct Line {
    pub start: Vec2f,
    pub direction: Vec2f,
    pub length: f32,
}

pub fn ring_iter<'a, T: 'a>(
    v: impl Iterator<Item = T> + Clone,
    start: usize,
) -> impl Iterator<Item = T> {
    v.clone().skip(start).chain(v.take(start))
}

impl Line {
    pub fn new_segment(start: Vec2f, end: Vec2f) -> Line {
        let diff = end - start;
        let length = diff.norm();
        Line::new(start, diff, length)
    }

    pub fn new_ray(start: Vec2f, direction: Vec2f) -> Line {
        Line::new(start, direction, std::f32::INFINITY)
    }

    pub fn new(start: Vec2f, direction: Vec2f, length: f32) -> Line {
        Line {
            start: start,
            direction: direction.normalize(),
            length: length,
        }
    }
}

pub fn intersect_segment(l: &Line, start: &Vec2f, end: &Vec2f) -> Option<Vec2f> {
    na::Matrix2::<f32>::from_columns(&[l.direction, start - end])
        .try_inverse()
        .map(|m| m * (start - l.start))
        .filter(|tu| tu[0] >= 0.0 && tu[0] < l.length && tu[1] >= 0.0 && tu[1] < 1.0)
        .map(|tu| l.start + tu[0] * l.direction)
}

pub fn intersect_ray(a: &Line, start: &Vec2f, direction: &Vec2f) -> Option<Vec2f> {
    na::Matrix2::<f32>::from_columns(&[a.direction, -direction])
        .try_inverse()
        .map(|m| m * (start - a.start))
        .filter(|tu| tu[0] >= 0.0 && tu[0] < a.length && tu[1] >= 0.0)
        .map(|tu| a.start + tu[0] * a.direction)
}

pub fn point_in_polygon(pt: &Vec2f, poly: &[Vec2f]) -> bool {
    let ray = Line::new_ray(*pt, Vec2f::new(1.0, 0.0));
    let hits = poly
        .iter()
        .zip(ring_iter(poly.iter(), 1))
        .filter(|(start, end)| intersect_segment(&ray, start, end).is_some())
        .count();
    hits % 2 == 1
}

pub fn generate_random_points(count: usize, lower: &Vec2f, upper: &Vec2f) -> Vec<Vec2f> {
    let mut rng = rand::thread_rng();
    let x_gen = rand::distributions::Uniform::from(lower[0]..upper[0]);
    let y_gen = rand::distributions::Uniform::from(lower[1]..upper[1]);
    (0..count)
        .map(|_| Vec2f::new(x_gen.sample(&mut rng), y_gen.sample(&mut rng)))
        .collect()
}

#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;

    fn v(x: f32, y: f32) -> Vec2f {
        na::vector![x, y]
    }

    #[test]
    fn test_point_in_polygon() {
        let triangle = [v(1.0, 1.0), v(1.0, -1.0), v(-1.0, 0.0)];
        // parallel
        assert_eq!(point_in_polygon(&v(0.0, 0.0), &triangle), true);
        assert_eq!(point_in_polygon(&v(2.0, 0.0), &triangle), false);
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
        //assert_eq!(
        //    intersect_segment(
        //        &Line::new_segment(v(4.0, 2.0), v(-4.0, 4.0)),
        //        &v(-2.0, 1.0),
        //        &v(2.0, 5.0)
        //    ),
        //    Some(v(0.0, 3.0))
        //);
        //assert_eq!(
        //    intersect_ray(
        //        &Line::new_segment(v(4.0, 2.0), v(-4.0, 4.0)),
        //        &v(-2.0, 1.0),
        //        &v(1.0, 1.0)
        //    ),
        //    Some(v(0.0, 3.0))
        //);
    }
}
