use obj::{Obj, ObjError};
use raylib::prelude::{Color, Vector3};
use std::collections::HashSet;
use std::path::Path;

pub struct Model {
    pub vertices: Vec<Vector3>,
    pub edges: HashSet<(usize, usize)>,
    pub triangles: Vec<[usize; 3]>,
    pub colors: Vec<Color>,
    pub center: Vector3,
}

impl Model {
    pub fn load(path: impl AsRef<Path>) -> Result<Self, ObjError> {
        let obj = Obj::load(path)?;
        Ok(Self::from_obj(obj))
    }

    pub fn from_obj(obj: Obj) -> Self {
        let vertices: Vec<Vector3> = obj
            .data
            .position
            .iter()
            .map(|pos| Vector3::new(pos[0], pos[1], pos[2]))
            .collect();

        assert!(!vertices.is_empty(), "OBJ has no vertices.");

        let center = Self::get_center(&vertices);
        let colors = Self::get_vertex_colors(&vertices);
        let edges = Self::get_edges(&obj);
        let triangles = Self::get_triangles(&obj);

        Self {
            vertices,
            edges,
            triangles,
            colors,
            center,
        }
    }

    fn get_vertex_colors(vertices: &[Vector3]) -> Vec<Color> {
        let mut minimum = vertices[0];
        let mut maximum = vertices[0];
        for vertex in &vertices[1..] {
            minimum.x = minimum.x.min(vertex.x);
            minimum.y = minimum.y.min(vertex.y);
            minimum.z = minimum.z.min(vertex.z);
            maximum.x = maximum.x.max(vertex.x);
            maximum.y = maximum.y.max(vertex.y);
            maximum.z = maximum.z.max(vertex.z);
        }

        let extent = maximum - minimum;
        let normalize = |value: f32, low: f32, range: f32| {
            if range.abs() <= f32::EPSILON {
                0.5
            } else {
                (value - low) / range
            }
        };
        let channel = |value: f32| (35.0 + value * 220.0) as u8;

        vertices
            .iter()
            .map(|vertex| {
                Color::new(
                    channel(normalize(vertex.x, minimum.x, extent.x)),
                    channel(normalize(vertex.y, minimum.y, extent.y)),
                    channel(normalize(vertex.z, minimum.z, extent.z)),
                    255,
                )
            })
            .collect()
    }

    pub fn get_center(vertices: &Vec<Vector3>) -> Vector3 {
        let mut maximum = vertices[0];
        let mut minimum = vertices[0];

        for v in &vertices[1..] {
            maximum.x = maximum.x.max(v.x);
            maximum.y = maximum.y.max(v.y);
            maximum.z = maximum.z.max(v.z);

            minimum.x = minimum.x.min(v.x);
            minimum.y = minimum.y.min(v.y);
            minimum.z = minimum.z.min(v.z);
        }

        (maximum + minimum) * 0.5
    }

    pub fn get_edges(obj: &Obj) -> HashSet<(usize, usize)> {
        let mut edges = HashSet::new();

        for object in &obj.data.objects {
            for group in &object.groups {
                for polygon in &group.polys {
                    let indices = &polygon.0;

                    if indices.len() < 2 {
                        continue;
                    }

                    for index in 0..indices.len() {
                        let first = indices[index].0;
                        let second = indices[(index + 1) % indices.len()].0;

                        // guardar índice más pequeño
                        let edge = (first.min(second), first.max(second));

                        edges.insert(edge);
                    }
                }
            }
        }

        edges
    }

    pub fn get_triangles(obj: &Obj) -> Vec<[usize; 3]> {
        let mut triangles = Vec::new();

        for object in &obj.data.objects {
            for group in &object.groups {
                for polygon in &group.polys {
                    let indices = &polygon.0;

                    // Triangulate polygons as a fan: (0, 1, 2), (0, 2, 3), ...
                    for index in 1..indices.len().saturating_sub(1) {
                        triangles.push([indices[0].0, indices[index].0, indices[index + 1].0]);
                    }
                }
            }
        }

        triangles
    }
}
