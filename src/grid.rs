use std::collections::HashMap;

use glam::{vec2, vec3, Vec3};
use image::{ImageBuffer, Rgb};

use crate::{mesh::{Mesh, Vertex}, model::Model};

#[derive(Debug)]
pub struct GridCell {
    pub id: usize,
    pub position:  Vec3,
    pub width: f32,
    // used to determine if this is traversable in A*
    pub blocked: bool,
    // used to precalculate the neighbors for A*
    pub adjacent_cells: Vec<usize>,
}

pub struct Grid {
    pub cells: HashMap<usize, GridCell>,
    pub next_cell_id: usize,
    pub mesh: Option<Mesh>,
    pub cell_size: f32,
    pub num_cells_x: usize,
    pub num_cells_z: usize,
}

impl Grid {
    pub fn new() -> Grid {
        Grid {
            cells: HashMap::new(),
            next_cell_id: 0,
            mesh: None,
            cell_size: 0.0,
            num_cells_x: 0,
            num_cells_z: 0,
        }
    }

    pub fn generate_grid() -> Grid {
        unimplemented!();
    }

    fn generate_grid_model() -> Model {
        unimplemented!();
    }

    pub fn generate_grid_mesh(grid_width: usize, grid_height: usize, cell_size: f32) -> Mesh {
        let mut vertices = Vec::<Vertex>::new();
        let mut indices = Vec::<u32>::new();
        let mut gray = false;
        let mut imgbuf = ImageBuffer::new(cell_size as u32, cell_size as u32);

        let c_gray:u8 = 67;
        let c_dark:u8 = 25;

        for (x, y, pixel) in imgbuf.enumerate_pixels_mut() {
            *pixel = Rgb([c_gray, c_gray, c_gray]);
        }

        imgbuf.save("./light.png").expect("Failed to save grid texture");

        for (x, y, pixel) in imgbuf.enumerate_pixels_mut() {
            *pixel = Rgb([c_dark, c_dark, c_dark]);
        }

        imgbuf.save("./dark.png").expect("Failed to save dark grid texture");
        for row in 0..grid_height {
            for col in 0..grid_width {
                let x = -(col as f32 * cell_size);
                let z = -(row as f32 * cell_size);
                let mut c:u8 = 0;

                let color = if gray {
                    c = 67;
                } else {
                    c = 25;
                };
                for (x, y, pixel) in imgbuf.enumerate_pixels_mut() {
                    *pixel = Rgb([c, c, c]);
                }

                gray = !gray;

                let base_index = vertices.len() as u32;

                // Add vertices for the cell
                vertices.push(Vertex { position: vec3(x, 0.0, z), normal: vec3(0.0, 1.0, 0.0), tex_coords: vec2(0.0, 1.0) });
                vertices.push(Vertex { position: vec3(x + cell_size, 0.0, z), normal: vec3(0.0, 1.0, 0.0), tex_coords: vec2(0.0, 1.0) });
                vertices.push(Vertex { position: vec3(x + cell_size, 0.0, z + cell_size), normal: vec3(0.0, 1.0, 0.0), tex_coords: vec2(0.0, 1.0) });
                vertices.push(Vertex { position: vec3(x, 0.0, z + cell_size), normal: vec3(0.0, 1.0, 0.0), tex_coords: vec2(0.0, 1.0)});

                // Add indices for two triangles
                indices.push(base_index);
                indices.push(base_index + 1);
                indices.push(base_index + 2);
                indices.push(base_index);
                indices.push(base_index + 2);
                indices.push(base_index + 3);
            }
            if grid_width % 2 == 0 {
                gray = !gray;
            }
        }

        let mut mesh = Mesh::new();
        mesh.vertices.append(&mut vertices);
        mesh.indices.append(&mut indices);
        mesh.setup_mesh();
        
        mesh;

        unimplemented!();
    }
}
