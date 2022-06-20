use nalgebra as na;

use rand::distributions::Distribution;
use rand::seq::SliceRandom;
use rand::Rng;
use rand::SeedableRng;

// Load data at compile time since loading files in JS is a mess.
const MAP_DATA: &[u8] = include_bytes!("map.bin");

#[allow(unused_macros)]
macro_rules! log {
    ($($arg:tt)+) => (
        gloo::console::log!(format!($($arg)+));
    );
}

fn lon_lat_scale(lon_lat_ref: na::VectorSlice2<f32>) -> na::Vector2<f32> {
    let lat_rad = lon_lat_ref[1] * std::f32::consts::PI / 180.0;
    const M0: f32 = 111132.954;
    const M1: f32 = 559.822;
    const M2: f32 = 1.175;
    let m_per_lon = M0 * lat_rad.cos();
    let m_per_lat = M0 - M1 * (2.0 * lat_rad).cos() + M2 * (4.0 * lat_rad).cos();
    na::Vector2::<f32>::new(m_per_lon, m_per_lat)
}

fn generate_bounds(
    center: na::VectorSlice2<f32>,
    width_m: f32,
    height_m: f32,
) -> impl Fn(na::VectorSlice2<f32>) -> bool {
    let meter_per_deg = lon_lat_scale(center);
    let mut min_lon = center[0] - 0.5 * width_m / meter_per_deg[0];
    let mut max_lon = center[0] + 0.5 * width_m / meter_per_deg[0];
    if min_lon > max_lon {
        std::mem::swap(&mut min_lon, &mut max_lon);
    }
    let mut min_lat = center[1] - 0.5 * height_m / meter_per_deg[1];
    let mut max_lat = center[1] + 0.5 * height_m / meter_per_deg[1];
    if min_lat > max_lat {
        std::mem::swap(&mut min_lat, &mut max_lat);
    }

    move |lon_lat: na::VectorSlice2<f32>| {
        lon_lat[0] >= min_lon
            && lon_lat[0] < max_lon
            && lon_lat[1] >= min_lat
            && lon_lat[1] < max_lat
    }
}

fn lon_lat_to_xy(
    center: na::VectorSlice2<f32>,
    lon_lat: na::VectorSlice2<f32>,
    scale: &na::Vector2<f32>,
) -> na::Vector2<f32> {
    let diff = lon_lat - center;
    na::Vector2::<f32>::new(diff[0] * scale[0], diff[1] * scale[1])
}

pub struct Map {
    pub coordinates: Vec<na::Vector2<f32>>,
    pub ports: Vec<na::Vector2<f32>>,
}

const SEED: u64 = 42;

impl Map {
    pub fn generate_random(width_m: f32, height_m: f32) -> Map {
        let lon_lat = na::Matrix2xX::<f32>::from_vec(
            bincode::deserialize(MAP_DATA).expect("Unable to load raw map data."),
        );

        //let mut rng = rand::rngs::StdRng::seed_from_u64(SEED);
        let mut rng = rand::thread_rng();
        let dist = rand::distributions::Uniform::from(0..lon_lat.ncols());
        let lon_lat_ref = lon_lat.column(dist.sample(&mut rng)).clone();
        let lon_lat_scale = lon_lat_scale(lon_lat_ref);

        let bounds = generate_bounds(lon_lat_ref, width_m, height_m);
        let raw_it = lon_lat
            .column_iter()
            .enumerate()
            .filter(|&(_, l)| bounds(l))
            .map(|(i, _)| i);
        let it = std::iter::once(0)
            .chain(raw_it)
            .chain(std::iter::once(lon_lat.ncols()));
        let coordinates = it
            .clone()
            .zip(it.skip(1))
            .flat_map(|(end, start)| {
                const STEPS: usize = 100;
                (end..start)
                    .step_by(((start - end) / STEPS).max(1))
                    .map(|i| {
                        log!("{}", i);
                        lon_lat.column(i)
                    })
            })
            .map(|pt| lon_lat_to_xy(lon_lat_ref, pt, &lon_lat_scale));

        Self {
            coordinates: coordinates.collect(),
            ports: Vec::new(),
        }
    }

    pub fn num_points(&self) -> usize {
        self.coordinates.len()
    }
}
