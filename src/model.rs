use crate::geom;
use crate::utils::assert_true;
use crate::utils::log;
use itertools::izip;

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

        let shift = geom::Vec3f::new(-1.5, 0.0, 0.0);
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
}

pub struct Model {
    mesh: tobj::Mesh,
    points: Vec<geom::Vec3f>,
    world_from_model: na::Transform3<f32>,
}

impl Model {
    pub fn load() -> Model {
        let mut model = MODEL.clone();
        let (models, _materials) =
            tobj::load_obj_buf(&mut model, &tobj::LoadOptions::default(), move |_| {
                tobj::load_mtl_buf(&mut MATERIAL.clone())
            })
            .expect("Unable to load mesh.");
        let mesh = models
            .first()
            .expect("No meshes defined in obj file.")
            .mesh
            .clone();

        let x_it = mesh.positions.iter().step_by(3);
        let y_it = mesh.positions.iter().skip(1).step_by(3);
        let z_it = mesh.positions.iter().skip(2).step_by(3);
        let points: Vec<geom::Vec3f> = izip!(x_it, y_it, z_it)
            .map(|(&x, &y, &z)| geom::Vec3f::new(x, y, z))
            .collect();

        let mut center = geom::Vec3f::new(0.0, 0.0, 0.0);
        let normalization = 1.0 / points.len() as f32;
        for pt in &points {
            center += normalization * pt;
        }

        Model {
            mesh: mesh,
            points: points,
            world_from_model: na::Transform3::identity() * na::Translation3::from(center),
        }
    }

    fn model_from_world(&self) -> na::Transform3<f32> {
        self.world_from_model.try_inverse().unwrap()
    }

    pub fn rotate(&mut self, dr: na::Rotation3<f32>) {
        self.world_from_model *= dr;
    }

    pub fn project(&self, camera: &Camera) -> ProjectedModel {
        let model_from_world = self.model_from_world();
        let camera_from_world = camera.camera_from_world();
        let projection = camera.camera_matrix() * (camera_from_world * model_from_world).matrix();

        let points3d: Vec<geom::Vec3f> = self
            .points
            .iter()
            .map(|pt| na::vector!(pt.x, pt.y, pt.z, 1.0))
            .map(|pt| projection * pt)
            .collect();

        let a_it = self.mesh.indices.iter().step_by(3);
        let b_it = self.mesh.indices.iter().skip(1).step_by(3);
        let c_it = self.mesh.indices.iter().skip(2).step_by(3);
        let mut faces3d: Vec<[geom::Vec3f; 3]> = izip!(a_it, b_it, c_it)
            .map(|(&a, &b, &c)| {
                [
                    points3d[a as usize],
                    points3d[b as usize],
                    points3d[c as usize],
                ]
            })
            .filter(|[a, b, c]| a.z > 0.0 && b.z > 0.0 && c.z > 0.0)
            .collect();

        faces3d.sort_by_key(|[a, b, c]| {
            let dist = a.z.min(b.z).min(c.z);
            (1E3 * dist) as u32
        });

        ProjectedModel {
            points: points3d.iter().map(|pt| pt.xy() / pt.z).collect(),
            faces: faces3d
                .iter()
                .map(|[a, b, c]| Face2d {
                    a: a.xy() / a.z,
                    b: b.xy() / b.z,
                    c: c.xy() / c.z,
                })
                .collect(),
        }
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
}
