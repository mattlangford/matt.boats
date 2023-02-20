use crate::geom;
use crate::utils::assert_true;
use crate::utils::log;
use crate::utils::*;
use itertools::izip;

use nalgebra as na;

use tobj;

const MODEL: &[u8] = include_bytes!("/Users/mattlangford/Downloads/boat.obj");
const MATERIAL: &[u8] = include_bytes!("/Users/mattlangford/Downloads/boat.mtl");

// TODO: make these references?
pub struct Face2d {
    pub a: geom::Vec2f,
    pub b: geom::Vec2f,
    pub c: geom::Vec2f,
}

#[derive(Default)]
pub struct ProjectedModel {
    pub points: Vec<geom::Vec2f>,
    pub faces: Vec<Face2d>,
}

#[derive(Debug)]
pub struct Camera {
    pub world_from_camera: na::Transform3<f32>,
    pub focal_length: f32,
}

impl Camera {
    pub fn new() -> Camera {
        let dir = geom::Vec3f::new(1.0, 0.0, 0.0);
        let up = -geom::Vec3f::z();
        let rotation = na::Rotation3::face_towards(&dir, &up);

        let shift = geom::Vec3f::new(-10.0, 0.0, 0.0);
        let translation = na::Translation3::<f32>::from(shift);
        Camera {
            world_from_camera: na::Transform3::identity() * translation * rotation,
            focal_length: 50.0,
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
            center -= normalization * pt;
        }

        // Picked to look nice
        let rotation = na::Rotation3::<f32>::from_euler_angles(-1.026089, -0.95820314, -0.6794422);

        log!(
            "Loaded model with {} points and {} faces",
            points.len(),
            mesh.indices.len() / 3
        );

        Model {
            mesh: mesh,
            points: points,
            world_from_model: na::Transform3::identity() * na::Translation3::from(center), // * rotation,
        }
    }

    fn model_from_world(&self) -> na::Transform3<f32> {
        self.world_from_model.try_inverse().unwrap()
    }

    pub fn rotate(&mut self, dr: na::Rotation3<f32>) {
        self.world_from_model = dr * self.world_from_model;
    }
    pub fn rotation(&self) -> na::Rotation3<f32> {
        let mat: na::Matrix3<f32> = From::from(
            self.world_from_model
                .matrix()
                .fixed_rows::<3>(0)
                .fixed_columns::<3>(0),
        );
        na::Rotation3::from_matrix(&mat)
    }

    pub fn project(&self, camera: &Camera) -> ProjectedModel {
        let camera_from_world = camera.camera_from_world();
        let camera_from_model = camera_from_world * self.world_from_model;
        let projection = camera.camera_matrix() * camera_from_model.matrix();
        let camera_position = camera.position();

        let camera_points: Vec<geom::Vec3f> = self
            .points
            .iter()
            .map(|pt| camera_from_model * na::point!(pt.x, pt.y, pt.z))
            .map(|pt| geom::Vec3f::new(pt.x, pt.y, pt.z))
            .collect();

        let points3d: Vec<geom::Vec3f> = self
            .points
            .iter()
            .map(|pt| na::vector!(pt.x, pt.y, pt.z, 1.0))
            .map(|pt| projection * pt)
            .collect();

        let a_it = self.mesh.indices.iter().step_by(3);
        let b_it = self.mesh.indices.iter().skip(1).step_by(3);
        let c_it = self.mesh.indices.iter().skip(2).step_by(3);

        #[derive(PartialEq)]
        struct Face3d {
            projected_a: geom::Vec3f,
            projected_b: geom::Vec3f,
            projected_c: geom::Vec3f,

            camera_a: geom::Vec3f,
            camera_b: geom::Vec3f,
            camera_c: geom::Vec3f,

            model_a: geom::Vec3f,
            model_b: geom::Vec3f,
            model_c: geom::Vec3f,
        }
        let mut faces3d: Vec<Face3d> = izip!(a_it, b_it, c_it)
            .map(|(&a, &b, &c)| Face3d {
                projected_a: points3d[a as usize],
                projected_b: points3d[b as usize],
                projected_c: points3d[c as usize],
                camera_a: camera_points[a as usize],
                camera_b: camera_points[b as usize],
                camera_c: camera_points[c as usize],
                model_a: self.points[a as usize],
                model_b: self.points[b as usize],
                model_c: self.points[c as usize],
            })
            // .filter(|f3d| f3d.projected_a.z > 0.0 && f3d.projected_b.z > 0.0 && f3d.projected_c.z > 0.0)
            .collect();

        faces3d.sort_by_key(|f3d| {
            let a = f3d.camera_a;
            let b = f3d.camera_b;
            let c = f3d.camera_c;

            let dist = ((a + b + c) / 3.0).norm();

            (1E3 * dist) as i32
        });

        let to_2d = |f3d: &Face3d| -> Face2d {
            let a = f3d.projected_a;
            let b = f3d.projected_b;
            let c = f3d.projected_c;
            Face2d {
                a: a.xy() / a.z,
                b: b.xy() / b.z,
                c: c.xy() / c.z,
            }
        };

        let mut faces: Vec<Face2d> = faces3d
            .iter()
            .filter_map(|f3d_i| {
                let f2d = to_2d(f3d_i);
                let pt = (f2d.a + f2d.b + f2d.c) / 3.0;
                let ray = geom::Line::new_ray(pt, geom::Vec2f::new(1.0, 1.0));

                let mut intersect = faces3d.iter().filter(|f3d_j| {
                    let f = to_2d(f3d_j);
                    let circle = geom::Circle::wrap(&[f.a, f.b, f.c]);
                    if (circle.center - pt).norm() >= circle.radius {
                        return false;
                    }

                    geom::intersect_segment(&ray, &f.a, &f.c)
                        .or(geom::intersect_segment(&ray, &f.b, &f.c))
                        .or(geom::intersect_segment(&ray, &f.c, &f.a))
                        .is_some()
                });

                if intersect.next().is_some() {
                    Some(f2d)
                } else {
                    None
                }
            })
            .collect();

        ProjectedModel {
            points: points3d.iter().map(|pt| pt.xy() / pt.z).collect(),
            faces: faces,
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
