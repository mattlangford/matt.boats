use crate::geom;
use crate::utils::assert_true;
use crate::utils::log;

use nalgebra as na;

use tobj;

const MODEL: &[u8] = include_bytes!("/Users/mattlangford/Downloads/ball.obj");
const MATERIAL: &[u8] = include_bytes!("/Users/mattlangford/Downloads/ball.mtl");

// TODO: make these references?
pub struct Face2d {
    pub a: geom::Vec2f,
    pub b: geom::Vec2f,
    pub c: geom::Vec2f,
}

pub struct ProjectedModel {
    pub points: Vec<geom::Vec2f>,
    pub faces: Vec<Face2d>,
}

#[derive(Debug)]
pub struct Camera {
    world_from_camera: na::Transform3<f32>,
    focal_length: f32,
}

impl Camera {
    pub fn new() -> Camera {
        let dir = geom::Vec3f::new(1.0, 0.0, 0.0);
        let up = -geom::Vec3f::z();
        let rotation = na::Rotation3::face_towards(&dir, &up);

        let shift = geom::Vec3f::new(-2.0, 0.0, 0.0);
        let translation = na::Translation3::<f32>::from(shift);
        Camera {
            world_from_camera: na::Transform3::identity() * translation * rotation,
            focal_length: 35.0,
        }
    }

    fn camera_from_world(&self) -> na::Transform3<f32> {
        self.world_from_camera.try_inverse().unwrap()
    }
    fn camera_matrix(&self) -> na::Matrix3x4<f32> {
        na::Matrix3x4::from_partial_diagonal(&[self.focal_length, self.focal_length, 1.0])
    }

    pub fn rotation(&self) -> na::Rotation3<f32> {
        let mat: na::Matrix3<f32> = From::from(
            self.world_from_camera
                .matrix()
                .fixed_rows::<3>(0)
                .fixed_columns::<3>(0),
        );
        na::Rotation3::from_matrix(&mat)
    }
    pub fn position(&self) -> geom::Vec3f {
        self.world_from_camera.matrix().column(3).xyz()
    }
    pub fn orbit(&mut self, pt: geom::Vec3f, dx: f32, dy: f32) {
        let r = (self.position() - pt).norm();

        let dcamera = geom::Vec3f::new(dx, dy, 0.0);
        let dworld = self.world_from_camera * dcamera;

        let camera_up = geom::Vec3f::new(0.0, 1.0, 0.0);
        let world_up = self.world_from_camera.transform_vector(&camera_up);

        let new_position = (self.position() + dworld).normalize() * r;

        let dir = pt - new_position;
        let up = world_up;

        let rotation = na::Rotation3::face_towards(&dir, &up);
        let translation = na::Translation3::<f32>::from(new_position);

        self.world_from_camera = na::Transform3::identity() * translation * rotation;
    }
}

pub struct Model {
    mesh: tobj::Mesh,
}

impl Model {
    pub fn load() -> Model {
        let mut model = MODEL.clone();
        let (models, _materials) =
            tobj::load_obj_buf(&mut model, &tobj::LoadOptions::default(), move |_| {
                tobj::load_mtl_buf(&mut MATERIAL.clone())
            })
            .expect("Unable to load mesh.");
        let mut mesh = models
            .first()
            .expect("No meshes defined in obj file.")
            .mesh
            .clone();

        let points = mesh.positions.len() / 3;
        let center_x = mesh.positions.iter().step_by(3).sum::<f32>() / points as f32;
        let center_y = mesh.positions.iter().skip(1).step_by(3).sum::<f32>() / points as f32;
        let center_z = mesh.positions.iter().skip(2).step_by(3).sum::<f32>() / points as f32;
        for i in 0..points {
            mesh.positions[3 * i] -= center_x;
            mesh.positions[3 * i + 1] -= center_y;
            mesh.positions[3 * i + 2] -= center_z;
        }

        Model { mesh: mesh }
    }

    pub fn project(&self, camera: &Camera) -> ProjectedModel {
        let camera_from_world = camera.camera_from_world();
        let projection = camera.camera_matrix() * camera_from_world.matrix();

        let pts = &self.mesh.positions;
        let index = &self.mesh.indices;

        let mut output = ProjectedModel {
            points: Vec::with_capacity(pts.len() / 3),
            faces: Vec::with_capacity(index.len() / 3),
        };
        output.points = (0..pts.len() / 3)
            .map(|i| na::vector!(pts[3 * i], pts[3 * i + 1], pts[3 * i + 2], 1.0))
            .map(|pt| projection * pt)
            .map(|pt| (pt / pt.z).xy())
            .collect();
        output.faces = (0..index.len() / 3)
            .map(|i| Face2d {
                a: output.points[index[3 * i] as usize],
                b: output.points[index[3 * i + 1] as usize],
                c: output.points[index[3 * i + 2] as usize],
            })
            .collect();
        output
    }
}

#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;

    #[test]
    fn test_basic_camera() {
        let camera = Camera::new();

        let pt1 = na::Point3::<f32>::new(0.0, -2.0, -1.0);
        let pt2 = na::Point3::<f32>::new(10.0, -2.0, -5.0);

        let camera_from_world = camera.camera_from_world();
        let projection = camera.camera_matrix() * camera_from_world.matrix();

        let mut projected1 = projection * na::vector!(pt1.x, pt1.y, pt1.z, 1.0);
        projected1 /= projected1.z;
        let mut projected2 = projection * na::vector!(pt2.x, pt2.y, pt2.z, 1.0);
        projected2 /= projected2.z;

        assert_true!(projected1.y < projected2.y);
        assert_true!(projected2.x < projected1.x);
    }

    #[test]
    fn test_load_model() {
        let camera = Camera::new();
        let model = Model::load();

        let projected = model.project(&camera);
        assert_true!(projected.points.len() > 0);
        assert_true!(projected.faces.len() > 0);
    }

    #[test]
    fn test_orbit() {
        let mut camera = Camera::new();

        let camera_up = geom::Vec3f::new(0.0, -1.0, 0.0);

        let before = camera.world_from_camera * camera_up;
        camera.orbit(geom::Vec3f::new(0.0, 0.0, 0.0), 0.1, 1.0);
        let after = camera.world_from_camera * camera_up;

        println!("before: {:?}, after: {:?}", before, after);
        assert_true!(before.dot(&after) > 0.9);
    }
}
