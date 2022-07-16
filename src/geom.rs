use crate::utils::*;
use nalgebra as na;
use rand::distributions::Distribution;

pub type Vec2f = na::Vector2<f32>;

#[derive(Debug, Default)]
pub struct Line {
    pub start: Vec2f,
    pub direction: Vec2f,
    pub length: f32,
}

#[derive(Debug, Default, Clone)]
pub struct AABox {
    pub start: Vec2f,
    pub dim: Vec2f,
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

    pub fn start(&self) -> Vec2f {
        self.start
    }

    pub fn end(&self) -> Vec2f {
        self.start + self.length.min(1E10) * self.direction
    }
}

impl AABox {
    pub fn new_square(start: Vec2f, dim: f32) -> AABox {
        AABox {
            start: start,
            dim: Vec2f::new(dim, dim),
        }
    }
    pub fn new_square_center(center: Vec2f, dim: f32) -> AABox {
        let dim2d = Vec2f::new(dim, dim);
        AABox {
            start: center - dim2d,
            dim: dim2d,
        }
    }

    pub fn corners(&self) -> [Vec2f; 4] {
        [
            self.start,
            self.start + Vec2f::new(self.dim[0], 0.0),
            self.start + self.dim,
            self.start + Vec2f::new(0.0, self.dim[1]),
        ]
    }

    pub fn edges(&self) -> [Line; 4] {
        let corners = self.corners();
        [
            Line::new_segment(corners[0], corners[1]),
            Line::new_segment(corners[1], corners[2]),
            Line::new_segment(corners[2], corners[3]),
            Line::new_segment(corners[3], corners[0]),
        ]
    }

    pub fn center(&self) -> Vec2f {
        self.start + 0.5 * self.dim
    }

    pub fn split_mut(&mut self) -> AABox {
        let mut new = self.clone();
        let index = if self.dim[0] >= self.dim[1] { 0 } else { 1 };

        self.dim[index] *= 0.5;
        new.dim[index] *= 0.5;
        new.start[index] += self.dim[index];
        return new;
    }
}

pub fn aabox_are_adjacent(lhs: &AABox, rhs: &AABox) -> bool {
    const TOL: f32 = 1E-3;

    let eq = |lhs: f32, rhs: f32| (lhs - rhs).abs() < TOL;
    let lt = |lhs: f32, rhs: f32| lhs - rhs < -TOL;
    let gt = |lhs: f32, rhs: f32| lhs - rhs > TOL;

    let lhs_corners = lhs.corners();
    let rhs_corners = rhs.corners();

    let lhs_x_range = minmax(lhs_corners[0].x, lhs_corners[2].x);
    let rhs_x_range = minmax(rhs_corners[0].x, rhs_corners[2].x);
    let lhs_y_range = minmax(lhs_corners[0].y, lhs_corners[2].y);
    let rhs_y_range = minmax(rhs_corners[0].y, rhs_corners[2].y);

    let range_union = |lhs: &(f32, f32), rhs: &(f32, f32)| {
        if eq(lhs.0, rhs.0) && eq(lhs.1, rhs.1) {
            // same line
            return true;
        }
        if (lt(lhs.0, rhs.0) || eq(lhs.0, rhs.0)) && gt(lhs.1, rhs.0) {
            // lhs starts before rhs and ends either in or after rhs
            return true;
        }
        if (lt(rhs.0, lhs.0) || eq(rhs.0, lhs.0)) && gt(rhs.1, lhs.0) {
            // rhs starts before lhs and ends either in or after rhs
            return true;
        }
        return false;
    };

    ((eq(lhs_x_range.0, rhs_x_range.1) || eq(lhs_x_range.1, rhs_x_range.0))
        && range_union(&lhs_y_range, &rhs_y_range))
        || ((eq(lhs_y_range.0, rhs_y_range.1) || eq(lhs_y_range.1, rhs_y_range.0))
            && range_union(&lhs_x_range, &rhs_x_range))
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
    let ray = Line::new_ray(*pt, Vec2f::new(1.0, 1.0));
    let hits = poly
        .iter()
        .zip(ring_iter(poly.iter(), 1))
        .filter(|(start, end)| intersect_segment(&ray, start, end).is_some())
        .count();
    hits % 2 == 1
}

pub fn intersect_polygon(line: &Line, poly: &[Vec2f]) -> bool {
    poly.iter()
        .zip(ring_iter(poly.iter(), 1))
        .any(|(start, end)| intersect_segment(&line, &start, &end).is_some())
}

pub fn point_in_aabox(pt: &Vec2f, b: &AABox) -> bool {
    let end = b.start + b.dim;
    pt[0] >= b.start[0] && pt[0] < end[0] && pt[1] >= b.start[1] && pt[1] < end[1]
}

pub fn generate_random_points(count: usize, lower: &Vec2f, upper: &Vec2f) -> Vec<Vec2f> {
    let mut rng = rand::thread_rng();
    let x_gen = rand::distributions::Uniform::from(lower[0]..upper[0]);
    let y_gen = rand::distributions::Uniform::from(lower[1]..upper[1]);
    (0..count)
        .map(|_| Vec2f::new(x_gen.sample(&mut rng), y_gen.sample(&mut rng)))
        .collect()
}

pub fn generate_points_on_line(count: usize, line: &Line) -> Vec<Vec2f> {
    (0..count)
        .map(|i| i as f32 / count as f32)
        .map(|t| line.start + line.length * t * line.direction)
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
    fn test_aabox_adjacent() {
        let lhs = AABox::new_square(Vec2f::new(0.0, 0.0), 1.0);
        assert_true!(aabox_are_adjacent(
            &lhs,
            &AABox::new_square(Vec2f::new(1.0, 0.0), 1.0)
        ));
        assert_false!(aabox_are_adjacent(
            &lhs,
            &AABox::new_square(Vec2f::new(1.1, 0.0), 1.0)
        ));
        assert_true!(aabox_are_adjacent(
            &lhs,
            &AABox::new_square(Vec2f::new(0.0, 1.0), 0.1)
        ));
        assert_fakse!(aabox_are_adjacent(
            &lhs,
            &AABox::new_square(Vec2f::new(0.5, 0.5), 0.1)
        ));
        assert_true!(aabox_are_adjacent(
            &lhs,
            &AABox::new_square(Vec2f::new(0.1, -1.0), 1.0)
        ));
    }

    #[test]
    fn test_point_in_polygon() {
        let triangle = [v(1.0, 1.0), v(1.0, -1.0), v(-1.0, 0.0)];
        // parallel
        assert_true!(point_in_polygon(&v(0.0, 0.0), &triangle));
        assert_false!(point_in_polygon(&v(2.0, 0.0), &triangle));
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
