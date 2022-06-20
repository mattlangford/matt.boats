use nalgebra as na;

// Load data at compile time since loading files in JS is a mess.
const MAP_DATA: &[u8] = include_bytes!("map.bin");

#[allow(unused_macros)]
macro_rules! log {
    ($($arg:tt)+) => (
        gloo::console::log!(format!($($arg)+));
    );
}

pub struct Map {
    // Each col is [lon, lat]
    pub data: na::Matrix2xX<f32>,
}

impl Map {
    pub fn generate() -> Result<Map, Box<bincode::ErrorKind>> {
        let data: Vec<f32> = bincode::deserialize(MAP_DATA)?;
        Ok(Self {
            data: na::Matrix2xX::<f32>::from_vec(data),
        })
    }

    pub fn num_points(&self) -> usize {
        self.data.ncols()
    }

    pub fn center_and_crop(
        &self,
        index: usize,
        width_m: f32,
        height_m: f32,
    ) -> Vec<na::Vector2<f32>> {
        // lon/lat
        let ref_point = self.data.column(index);

        let lat_rad = ref_point[1] * std::f32::consts::PI / 180.0;
        let m_per_lat =
            111132.954 - 559.822 * (2.0 * lat_rad).cos() + 1.175 * (4.0 * lat_rad).cos();
        let m_per_lon = 111132.954 * lat_rad.cos();

        let mut min_lon = ref_point[0] - 0.5 * width_m / m_per_lon;
        let mut max_lon = ref_point[0] + 0.5 * width_m / m_per_lon;
        if min_lon > max_lon {
            std::mem::swap(&mut min_lon, &mut max_lon);
        }
        let mut min_lat = ref_point[1] - 0.5 * height_m / m_per_lat;
        let mut max_lat = ref_point[1] + 0.5 * height_m / m_per_lat;
        if min_lat > max_lat {
            std::mem::swap(&mut min_lat, &mut max_lat);
        }

        let within_bounds = |lon_lat: na::VectorSlice2<f32>| {
            lon_lat[0] >= min_lon
                && lon_lat[0] < max_lon
                && lon_lat[1] >= min_lat
                && lon_lat[1] < max_lat
        };

        let to_xy = |lon_lat: na::VectorSlice2<f32>| {
            let mut xy = lon_lat - ref_point;
            xy[0] *= m_per_lon;
            xy[1] *= m_per_lat;
            xy
        };

        let mut start = index;
        loop {
            if start == 0 {
                start = self.data.ncols();
            }
            start -= 1;
            if start == index || !within_bounds(self.data.column(start)) {
                break;
            }
        }

        let mut result = Vec::<na::Vector2<f32>>::new();
        result.push(to_xy(self.data.column(start)));

        let mut end = start;
        loop {
            end += 1;
            let pt = self.data.column(end % self.data.ncols());
            result.push(to_xy(pt));
            if !within_bounds(pt) {
                break;
            }
        }

        let ref_xy = to_xy(ref_point);
        let step_size = |xy: &na::Vector2<f32>| {
            let dist = (xy - ref_xy).norm();
            let max_dim = width_m.max(height_m);
            if dist < 1.5 * max_dim {
                return 1;
            }
            if dist < 2.0 * max_dim {
                return 10;
            }
            if dist < 2.5 * max_dim {
                return 100;
            }
            if dist < 3.0 * max_dim {
                return 500;
            }
            if dist < 4.0 * max_dim {
                return 1000;
            }
            if dist < 5.0 * max_dim {
                return 2000;
            }
            5000
        };

        let stopping_point = start + self.data.ncols();
        end += 1;
        while end < stopping_point {
            let pt = to_xy(self.data.column(end % self.data.ncols()));
            let step = step_size(&pt);
            result.push(pt);
            end += step;
        }

        result
    }
}
