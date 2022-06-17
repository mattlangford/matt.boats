use nalgebra as na;

// Load data at compile time since loading files in JS is a mess.
const MAP_DATA: &[u8] = include_bytes!("map.bin");

pub struct Map {
    // Each col is [lon, lat]
    data: na::Matrix2xX<f32>
}

impl Map {
    pub fn generate() -> Result<Map, Box<bincode::ErrorKind>> {
        let data: Vec<f32> = bincode::deserialize(MAP_DATA)?;
        Ok(Self{data: na::Matrix2xX::<f32>::from_vec(data)})
    }

    pub fn num_points(&self) -> usize {
        self.data.ncols()
    }

    pub fn center_and_crop(&self, index: usize, width_m: f64, height_m: f64) {
        let ref_point = self.data.row(index);

        let to_xy = |lon_lat: &na::VectorSlice2<f32>| {
            let lat_rad = lon_lat[1] * 180.0 / std::f32::consts::PI;
            let m_per_lat = 111132.954 - 559.822 * (2.0 * lat_rad).cos() + 1.175 * (4.0 * lat_rad).cos();
            let m_per_lon = 111132.954 * lat_rad.cos();
            //(lon_lat - ref_point) * na::Vector2::from(m_per_lon, m_per_lat)
        };

        to_xy(&self.data.column(10));
    }
}
