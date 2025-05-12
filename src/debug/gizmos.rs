use glam::Vec3;
use serde::Deserialize;

use crate::animation::animation::{Model, Vertex};

pub struct Cuboid {
    pub w: f32,
    pub h: f32,
    pub d: f32
}

impl Cuboid {
    pub fn create_model(&self) -> Model {
        let mut cuboid = Model::new();

        let max_x = self.w / 2.0;
        let min_x = -max_x;

        let min_y = 0.0;
        let max_y = self.h;

        let max_z = self.d / 2.0;
        let min_z = -max_z;

        let px = Vec3::new(1.0, 0.0, 0.0);
        let nx = Vec3::new(-1.0, 0.0, 0.0);
        let py = Vec3::new(0.0, 1.0, 0.0);
        let ny = Vec3::new(0.0, -1.0, 0.0);
        let pz = Vec3::new(0.0, 0.0, 1.0);
        let nz = Vec3::new(0.0, 0.0, -1.0);

        let vertices = vec![
            // Right (+X)
            Vertex::new(Vec3::new(max_x, min_y, min_z), px),
            Vertex::new(Vec3::new(max_x, max_y, min_z), px),
            Vertex::new(Vec3::new(max_x, max_y, max_z), px),
            Vertex::new(Vec3::new(max_x, min_y, max_z), px),

            // Left (-X)
            Vertex::new(Vec3::new(min_x, min_y, max_z), nx),
            Vertex::new(Vec3::new(min_x, max_y, max_z), nx),
            Vertex::new(Vec3::new(min_x, max_y, min_z), nx),
            Vertex::new(Vec3::new(min_x, min_y, min_z), nx),

            // Top (+Y)
            Vertex::new(Vec3::new(min_x, max_y, min_z), py),
            Vertex::new(Vec3::new(min_x, max_y, max_z), py),
            Vertex::new(Vec3::new(max_x, max_y, max_z), py),
            Vertex::new(Vec3::new(max_x, max_y, min_z), py),

            // Bottom (-Y)
            Vertex::new(Vec3::new(min_x, min_y, max_z), ny),
            Vertex::new(Vec3::new(min_x, min_y, min_z), ny),
            Vertex::new(Vec3::new(max_x, min_y, min_z), ny),
            Vertex::new(Vec3::new(max_x, min_y, max_z), ny),

            // Front (+Z)
            Vertex::new(Vec3::new(max_x, min_y, max_z), pz),
            Vertex::new(Vec3::new(max_x, max_y, max_z), pz),
            Vertex::new(Vec3::new(min_x, max_y, max_z), pz),
            Vertex::new(Vec3::new(min_x, min_y, max_z), pz),

            // Back (-Z)
            Vertex::new(Vec3::new(min_x, min_y, min_z), nz),
            Vertex::new(Vec3::new(min_x, max_y, min_z), nz),
            Vertex::new(Vec3::new(max_x, max_y, min_z), nz),
            Vertex::new(Vec3::new(max_x, min_y, min_z), nz),
        ];

        let indices = vec![
            0, 1, 2,  0, 2, 3,       // Right
            4, 5, 6,  4, 6, 7,       // Left
            8, 9,10,  8,10,11,       // Top
            12,13,14, 12,14,15,      // Bottom
            16,17,18, 16,18,19,      // Front
            20,21,22, 20,22,23,      // Back
        ];

        cuboid.vertices = vertices;
        cuboid.indices = indices;
        cuboid.setup_opengl();

        cuboid
    }
}

#[derive(Deserialize, Debug, Clone)]
pub struct Cylinder {
    pub r: f32,
    pub h: f32,
}

impl Cylinder {
    pub fn create_model(&self, segments: u32) -> Model {
        let mut model = Model::new();
        let mut vertices = vec![];
        let mut indices = vec![];

        let angle_step = std::f32::consts::TAU / segments as f32;

        // bottom center vertex
        let bottom_center_index = vertices.len() as u32;
        vertices.push(Vertex::new(Vec3::new(0.0, 0.0, 0.0), Vec3::NEG_Y));

        // Bottom Ring
        for i in 0..segments {
            let theta = i as f32 * angle_step;
            let x = self.r * theta.cos();
            let z = self.r * theta.sin();

            vertices.push(Vertex::new(Vec3::new(x, 0.0, z), Vec3::NEG_Y));
        }

        // Bottom cap indices (triangle fan)
        for i in 0..segments {
            let next = (i + 1) % segments;
            indices.push(bottom_center_index);
            indices.push(bottom_center_index + 1 + next);
            indices.push(bottom_center_index + 2 + i);
        }

        // top center vertex
        let top_center_vertex = vertices.len() as u32;
        vertices.push(Vertex::new(Vec3::new(0.0, self.h, 0.0), Vec3::Y));

        // top ring
        let top_ring_start = vertices.len() as u32;
        for i in 0..segments {
            let theta = i as f32 *angle_step;
            let x = self.r * theta.cos();
            let z = self.r * theta.sin();

            vertices.push(Vertex::new(Vec3::new(x, self.h, z), Vec3::Y));
        }

        // top cap indices ( triangle fan )
        for i in 0..segments {
            let next = (i + 1) % segments;
            indices.push(top_center_vertex);
            indices.push(top_ring_start + i);
            indices.push(top_ring_start + next);
        }

        // build side wall
        let side_start_index = vertices.len() as u32;
        for i in 0..segments {
            let theta = i as f32 * angle_step;
            let x = self.r * theta.cos();
            let z = self.r * theta.sin();
            let normal = Vec3::new(x, 0.0, z).normalize();

            // bottom vertex of the side
            vertices.push(Vertex::new(Vec3::new(x, 0.0, z), normal));
            // top vertex of the side
            vertices.push(Vertex::new(Vec3::new(x, self.h, z), normal));
        }

        // side indices (split the quads into two triangles)
        for i in 0..segments {
            let next = (i + 1) % segments;
            let base = side_start_index;

            let bottom_i = base + i * 2;
            let top_i = base + i * 2 + 1;
            let bottom_next = base + next * 2;
            let top_next = base + next * 2 + 1;

            // triangle 1
            indices.push(bottom_i);
            indices.push(bottom_next);
            indices.push(top_i);

            // triangle 2
            indices.push(top_i);
            indices.push(bottom_next);
            indices.push(top_next);
        }

        model.vertices = vertices;
        model.indices = indices;
        model.setup_opengl();

        model
    }
}
