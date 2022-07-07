use nalgebra as na;

use rand::distributions::Distribution;
use rand::seq::SliceRandom;
use rand::Rng;
use rand::SeedableRng;

use crate::geom::*;
use crate::utils::*;

// Load data at compile time since loading files in JS is a mess.
const MAP_DATA: &[u8] = include_bytes!("map.bin");

fn lon_lat_scale(lon_lat_ref: na::VectorSlice2<f32>) -> Vec2f {
    let lat_rad = lon_lat_ref[1] * std::f32::consts::PI / 180.0;
    const M0: f32 = 111132.954;
    const M1: f32 = 559.822;
    const M2: f32 = 1.175;
    let m_per_lon = M0 * lat_rad.cos();
    let m_per_lat = M0 - M1 * (2.0 * lat_rad).cos() + M2 * (4.0 * lat_rad).cos();
    na::Vector2::<f32>::new(m_per_lon, m_per_lat)
}

pub fn generate_corners(width_m: f32, height_m: f32) -> [Vec2f; 4] {
    [
        Vec2f::new(0.5 * width_m, 0.5 * height_m),
        Vec2f::new(0.5 * width_m, -0.5 * height_m),
        Vec2f::new(-0.5 * width_m, -0.5 * height_m),
        Vec2f::new(-0.5 * width_m, 0.5 * height_m),
    ]
}

pub fn generate_edges(width_m: f32, height_m: f32) -> [Line; 4] {
    let corners = generate_corners(width_m, height_m);
    [
        Line::new_segment(corners[0], corners[1]),
        Line::new_segment(corners[1], corners[2]),
        Line::new_segment(corners[2], corners[3]),
        Line::new_segment(corners[3], corners[0]),
    ]
}

fn generate_to_xy(center: na::VectorSlice2<f32>) -> impl Fn(na::VectorSlice2<f32>) -> Vec2f + '_ {
    let scale = lon_lat_scale(center);
    move |lon_lat: na::VectorSlice2<f32>| {
        let diff = lon_lat - center;
        Vec2f::new(diff[0] * scale[0], diff[1] * scale[1])
    }
}

pub struct Map {
    pub width_m: f32,
    pub height_m: f32,
    pub coordinates: Vec<Vec2f>,
    pub ports: Vec<Vec2f>,
}

const SEED: u64 = 42;

impl Map {
    pub fn generate_random(width_m: f32, height_m: f32) -> Map {
        return Map {
            width_m: width_m,
            height_m: height_m,
            coordinates: Vec::new(),
            ports: Vec::new(),
        };
        let lon_lat = na::Matrix2xX::<f32>::from_vec(
            bincode::deserialize(MAP_DATA).expect("Unable to load raw map data."),
        );

        //let mut rng = rand::rngs::StdRng::seed_from_u64(SEED);
        let mut rng = rand::thread_rng();
        let dist = rand::distributions::Uniform::from(0..lon_lat.ncols());
        let start_index = dist.sample(&mut rng);
        let lon_lat_ref = lon_lat.column(start_index).clone();

        let corners = generate_corners(width_m, height_m);
        let bounds = [
            Line::new_segment(corners[0], corners[1]),
            Line::new_segment(corners[1], corners[2]),
            Line::new_segment(corners[2], corners[3]),
            Line::new_segment(corners[3], corners[0]),
            Line::new_ray(corners[0], corners[0]),
            Line::new_ray(corners[1], corners[1]),
            Line::new_ray(corners[2], corners[2]),
            Line::new_ray(corners[3], corners[3]),
        ];

        let to_xy = generate_to_xy(lon_lat_ref);
        let ref_xy = to_xy(lon_lat_ref);
        let mut iter = ring_iter(lon_lat.column_iter(), start_index)
            .map(&to_xy)
            .peekable();
        let mut coordinates = vec![ref_xy];
        let mut ports = Vec::<Vec2f>::new();
        let mut inside = true;

        while let Some(start) = iter.next() {
            let end = iter.peek().unwrap_or(&ref_xy);
            let mut intersections = bounds
                .iter()
                .filter_map(|l| intersect_segment(l, &start, &end))
                .collect::<Vec<Vec2f>>();
            if !intersections.is_empty() {
                inside = point_in_polygon(end, &corners);
            }

            intersections.sort_by_key(|pt| (pt - start).norm_squared() as u64);
            for intersection in intersections {
                coordinates.push(intersection);
            }

            if inside {
                coordinates.push(start);
            }
        }

        let ports = coordinates.choose_multiple(&mut rng, 2).cloned().collect();

        Self {
            width_m: width_m,
            height_m: height_m,
            coordinates: coordinates,
            ports: ports,
        }
    }
}
