use crate::geom;
use nalgebra as na;
use tobj;

struct Face2d<'a> {
    a: &'a geom::Vec2f,
    b: &'a geom::Vec2f,
    c: &'a geom::Vec2f,
}

struct ProjectedModel<'a> {
    points: Vec<geom::Vec2f>,
    faces: Vec<Face2d<'a>>,
}

#[derive(Debug)]
struct Camera {
    world_from_camera: na::Transform3<f32>,
    focal_length: f32,
}

impl Camera {
    fn new() -> Camera {
        let dir = geom::Vec3f::new(1.0, 0.0, 0.0);
        let up = -geom::Vec3f::z();
        let rotation = na::Rotation3::face_towards(&dir, &up);

        let shift = geom::Vec3f::new(-10.0, 0.0, 0.0);
        let translation = na::Translation3::<f32>::from(shift);
        Camera {
            world_from_camera: na::Transform3::identity() * translation * rotation,
            focal_length: 10.0,
        }
    }

    fn camera_from_world(&self) -> na::Transform3<f32> {
        self.world_from_camera.try_inverse().unwrap()
    }
    fn camera_matrix(&self) -> na::Matrix3x4<f32> {
        na::Matrix3x4::from_partial_diagonal(&[self.focal_length, self.focal_length, 1.0])
    }
}

struct Model {
    mesh: tobj::Mesh,
}

impl Model {
    fn load() -> Model {
        let (models, materials) =
            tobj::load_obj("~/Downloads/ball.obj", &tobj::LoadOptions::default())
                .expect("Unable to load mesh.");
        Model {
            mesh: models
                .first()
                .expect("No meshes defined in obj file.")
                .mesh
                .clone(),
        }
    }

    fn project(&self, camera: &Camera) -> ProjectedModel {
        ProjectedModel {
            points: Vec::new(),
            faces: Vec::new(),
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

        let pt = na::Point3::<f32>::new(0.0, -2.0, -1.0);
        let camera_from_world = camera.camera_from_world();
        let projection = camera.camera_matrix() * camera_from_world.matrix();

        let mut projected = projection * na::vector!(pt.x, pt.y, pt.z, 1.0);
        projected /= projected.z;
        println!("{}", camera_from_world * pt);
        println!("{}", projected.xy());

        assert!(false);
    }
}
