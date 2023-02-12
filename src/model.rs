use crate::geom;
use crate::utils::assert_true;

use nalgebra as na;

use tobj;

const MODEL: &[u8] = include_bytes!("/Users/mattlangford/Downloads/ball.obj");
const MATERIAL: &[u8] = include_bytes!("/Users/mattlangford/Downloads/ball.mtl");

// TODO: make these references?
struct Face2d {
    a: geom::Vec2f,
    b: geom::Vec2f,
    c: geom::Vec2f,
}

struct ProjectedModel {
    points: Vec<geom::Vec2f>,
    faces: Vec<Face2d>,
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
            tobj::load_obj_buf(&mut MODEL, &tobj::LoadOptions::default(), |_| {
                tobj::load_mtl_buf(&mut MATERIAL)
            })
            .expect("Unable to load mesh.");
        let mesh = models
            .first()
            .expect("No meshes defined in obj file.")
            .mesh
            .clone();
        assert!(
            mesh.face_arities.is_empty(),
            "Mesh doesn't appear to be made of triangles."
        );

        Model { mesh: mesh }
    }

    fn project(&self, camera: &Camera) -> ProjectedModel {
        let camera_from_world = camera.camera_from_world();
        let projection = camera.camera_matrix() * camera_from_world.matrix();

        let pts = &self.mesh.positions;
        let index = &self.mesh.indices;

        let mut output = ProjectedModel {
            points: Vec::with_capacity(pts.len() / 3),
            faces: Vec::with_capacity(index.len() / 3),
        };
        output.points = (0..pts.len() / 3)
            .map(|i| na::vector!(pts[i], pts[i + 1], pts[i + 2], 1.0))
            .map(|pt| projection * pt)
            .map(|pt| (pt / pt.z).xy())
            .collect();
        output.faces = (0..index.len() / 3)
            .map(|i| Face2d {
                a: output.points[index[i] as usize],
                b: output.points[index[i + 1] as usize],
                c: output.points[index[i + 2] as usize],
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
}
