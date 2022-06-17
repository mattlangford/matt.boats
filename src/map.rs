use nalgebra as na;

const MAP_DATA: &[u8] = include_bytes!("map.bin");

pub fn parse_map() -> Result<na::Matrix2xX<f32>, Box<bincode::ErrorKind>> {
    let data: Vec<f32> = bincode::deserialize(MAP_DATA)?;
    Ok(na::Matrix2xX::<f32>::from_vec(data))
}
