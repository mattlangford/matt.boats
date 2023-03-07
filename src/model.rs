use crate::geom;
use crate::utils::assert_true;
use crate::utils::log;
use crate::utils::*;
use itertools::izip;

use nalgebra as na;

use tobj;

const MODEL: &[u8] = include_bytes!("/Users/mattlangford/Downloads/boat.obj");
const MATERIAL: &[u8] = include_bytes!("/Users/mattlangford/Downloads/boat.mtl");

#[derive(Clone, Debug)]
pub struct Polygon {
    start: usize,
    count: usize,
}

#[derive(Default)]
pub struct ProjectedModel {
    pub points: Vec<geom::Vec2f>,
    pub polys: Vec<Polygon>,
}

#[derive(Debug)]
pub struct Camera {
    pub world_from_camera: na::Transform3<f32>,
    pub focal_length: f32,
}

pub struct Model {
    mesh: tobj::Mesh,
    points: Vec<geom::Vec3f>,
    polys: Vec<Polygon>,
    world_from_model: na::Transform3<f32>,
}

impl Polygon {
    pub fn points<'a, T>(&self, buffer: &'a Vec<T>) -> impl Iterator<Item = &'a T> {
        buffer.iter().skip(self.start).take(self.count)
    }

    pub fn center<'a, T: std::iter::Sum<&'a T> + std::ops::Div<f32, Output = T>>(
        &self,
        buffer: &'a Vec<T>,
    ) -> T {
        self.points(&buffer).sum::<T>() / self.count as f32
    }

    // Returns center and normal vector
    pub fn plane(&self, buffer: &Vec<geom::Vec3f>) -> (geom::Vec3f, geom::Vec3f) {
        assert!(self.count > 2);

        let a = buffer[self.start];
        let b = buffer[self.start + 1];
        let c = buffer[self.start + 2];
        let normal = (a - c).cross(&(b - c)).normalize();
        (c, normal)
    }
}

impl Camera {
    pub fn new() -> Camera {
        let dir = geom::Vec3f::new(1.0, 0.0, 0.0);
        let up = -geom::Vec3f::z();
        let rotation = na::Rotation3::face_towards(&dir, &up);

        let shift = geom::Vec3f::new(-4.0, 0.0, 0.0);
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

        if mesh.face_arities.is_empty() {
            mesh.face_arities = std::iter::repeat(3).take(mesh.indices.len() / 3).collect();
        }

        let mut index = 0;
        let mut points =
            Vec::<geom::Vec3f>::with_capacity(mesh.face_arities.iter().sum::<u32>() as usize);
        let mut polys = Vec::<Polygon>::with_capacity(mesh.face_arities.len());
        for arity in &mesh.face_arities {
            let count = *arity as usize;
            polys.push(Polygon {
                start: points.len(),
                count: count,
            });

            for _ in 0..count {
                let idx = 3 * mesh.indices[index] as usize;
                points.push(geom::Vec3f::new(
                    mesh.positions[idx],
                    mesh.positions[idx + 1],
                    mesh.positions[idx + 2],
                ));
                index += 1;
            }
        }

        let mut center = geom::Vec3f::new(0.0, 0.0, 0.0);
        let normalization = 1.0 / points.len() as f32;
        for pt in &points {
            center -= normalization * pt;
        }

        for pt in &mut points {
            *pt += center;
        }

        log!(
            "Loaded model with {} points and {} faces",
            points.len(),
            polys.len(),
        );

        assert!(points.len() > 0);
        assert!(polys.len() > 0);

        Model {
            mesh: mesh,
            points: points,
            polys: polys,
            world_from_model: na::Transform3::identity(),
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
        let projection_from_camera = camera.camera_matrix();

        // In camera frame
        let points3d: Vec<geom::Vec3f> = self
            .points
            .iter()
            .map(|pt| na::point!(pt.x, pt.y, pt.z))
            .map(|pt| camera_from_model.transform_point(&pt))
            .map(|pt| na::vector!(pt.x, pt.y, pt.z))
            .collect();

        let points2d: Vec<geom::Vec2f> = points3d
            .iter()
            .map(|pt| na::vector!(pt.x, pt.y, pt.z, 1.0))
            .map(|pt| projection_from_camera * pt)
            .map(|pt| pt.xy() / pt.z)
            .collect();

        // For each polygon, store the indices of the polys behind it
        let mut behind = vec![std::collections::HashSet::<usize>::new(); self.polys.len()];
        for (i, poly_i) in self.polys.iter().enumerate() {
            if poly_i.count < 3 {
                continue;
            }

            // Ray origin assumed to be (0, 0, 0)
            let dist_to_intersection =
                |plane_pt: &geom::Vec3f, plane_normal: &geom::Vec3f, ray_dir: &geom::Vec3f| {
                    let t = plane_normal.dot(&plane_pt) / plane_normal.dot(&ray_dir);
                    t * ray_dir
                };

            let point3d = poly_i.center(&points3d);
            let point2d = poly_i.center(&points2d);
            let ray_dir = point3d.normalize();
            let (plane_pt, plane_norm) = poly_i.plane(&points3d);
            let dist_i = dist_to_intersection(&plane_pt, &plane_norm, &ray_dir).norm();

            for (j, poly_j) in self.polys.iter().enumerate() {
                if i == j {
                    continue;
                }

                let points = poly_j
                    .points(&points2d)
                    .cloned()
                    .collect::<Vec<geom::Vec2f>>();
                if !geom::point_in_polygon(&point2d, &points[..]) {
                    continue;
                }

                let (plane_pt, plane_norm) = poly_j.plane(&points3d);
                let dist_j = dist_to_intersection(&plane_pt, &plane_norm, &ray_dir).norm();
                if dist_j.is_nan() {
                    log!("Got nan distance!");
                    continue;
                }
                if dist_i < dist_j {
                    if behind[j].contains(&i) {
                        // log!("{} and {} are mutually overlapping.", i, j);
                        continue;
                    }
                    behind[i].insert(j);
                } else {
                    if behind[i].contains(&j) {
                        // log!("{} and {} are mutually overlapping.", i, j);
                        continue;
                    }
                    behind[j].insert(i);
                }
            }
        }

        #[derive(Debug)]
        struct Entry {
            index: usize,
            depth: usize,
        }
        let mut order: Vec<Entry> = (0..behind.len())
            .map(|i| Entry { index: i, depth: 0 })
            .collect();

        let mut queue: std::collections::VecDeque<Entry> = (0..behind.len())
            .map(|i| Entry { index: i, depth: 0 })
            .collect();
        while let Some(entry) = queue.pop_front() {
            if queue.len() > 5000 {
                log!("Queue too big!");
                break;
            }
            // Assume not more than a couple deep. I think this comes from mutally overlapping
            // polygons.
            if entry.depth > 5 {
                continue;
            }

            let next_depth = entry.depth + 1;
            for &j in &behind[entry.index] {
                if order[j].depth < next_depth {
                    order[j].depth = next_depth;
                    queue.push_back(Entry {
                        index: j,
                        depth: next_depth,
                    });
                }
            }
        }
        order.sort_by_key(|entry| {
            (
                entry.depth,
                (1E3 * self.polys[entry.index].center(&points3d).z) as u32,
            )
        });

        ProjectedModel {
            points: points2d,
            polys: order
                .iter()
                .map(|entry| self.polys[entry.index].clone())
                .rev()
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

        assert_true!(false);
    }
}
