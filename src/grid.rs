use std::collections::HashMap;

use glam::{vec2, vec3, Vec3};
use image::{ImageBuffer, Rgb, Rgba};

use crate::{mesh::{Mesh, Texture, Vertex}, model::Model, shaders::Shader};

#[derive(Debug)]
pub struct GridCell {
    pub id: usize,
    pub position:  Vec3,
    pub width: f32, // TODO: We don't need width because we have cell_size on the grid itself.
    // used to determine if this is traversable in A*
    pub blocked: bool,
    // used to precalculate the neighbors for A*
    pub adjacent_cells: Vec<usize>,
}

pub struct Grid {
    // pub cells: HashMap<usize, GridCell>,
    pub cells: Vec<GridCell>,
    pub next_cell_id: usize,
    pub model: Model,
    pub cell_size: f32,
    pub width: usize,
    pub height: usize,
}

impl Grid {
    pub fn new(width: usize, height: usize) -> Grid {
        Grid {
            cells: Vec::with_capacity(width * height) ,
            next_cell_id: 0,
            model: Model::new(),
            cell_size: 0.0,
            width,
            height,
        }
    }

    pub fn generate(&mut self) {
        Self::generate_texture();
        self.generate_model();
    }

    fn generate_model(&mut self) {
        // TODO: This only works with even numbered grid sizes, fix
        let mut mesh = self.generate_grid_mesh(1.0);
        self.model.directory = "resources/textures".to_string();

        let tex_id = Model::texture_from_file(&self.model, "half_dark_half_light.png".to_string());

        let tex = Texture {
            id: tex_id,
            _type: "texture_diffuse".to_string(),
            path: "half_dark_half_light.png".to_string(),
        };
        
        mesh.textures.push(tex.clone());

        self.model.meshes.push(mesh);
        self.model.textures_loaded.push(tex);
    }

    fn generate_grid_mesh(&mut self, cell_size: f32) -> Mesh {
        self.cell_size = cell_size;
        let mut vertices = Vec::<Vertex>::new();
        let mut indices = Vec::<u32>::new();
        let mut mesh = Mesh::new();
        let mut dark = false;

        let total_width = self.width as f32 * cell_size;
        let total_height = self.height as f32 * cell_size;

        for row in 0..self.width {
            for col in 0..self.height {
                let x = (col as f32 * cell_size) - (total_width / 2.0);
                let z = (row as f32 * cell_size) - (total_height / 2.0);


                let (bl, br, tr, tl) = if dark {
                    (
                        vec2(0.0, 0.0),
                        vec2(0.5, 0.0),
                        vec2(0.5, 1.0),
                        vec2(0.0, 1.0)
                    )
                } else {
                    ( 
                        vec2(0.5, 0.0),
                        vec2(1.0, 0.0),
                        vec2(1.0, 1.0),
                        vec2(0.5, 1.0),
                    )
                };

                dark = !dark;

                let base_index = vertices.len() as u32;


                // Add vertices for the cell
                vertices.push(Vertex { position: vec3(x, 0.0, z), normal: vec3(0.0, 1.0, 0.0), tex_coords: bl });
                vertices.push(Vertex { position: vec3(x + cell_size, 0.0, z), normal: vec3(0.0, 1.0, 0.0), tex_coords: br });
                vertices.push(Vertex { position: vec3(x + cell_size, 0.0, z + cell_size), normal: vec3(0.0, 1.0, 0.0), tex_coords: tr });
                vertices.push(Vertex { position: vec3(x, 0.0, z + cell_size), normal: vec3(0.0, 1.0, 0.0), tex_coords: tl });

                // Add indices for two triangles
                indices.push(base_index);
                indices.push(base_index + 1);
                indices.push(base_index + 2);
                indices.push(base_index);
                indices.push(base_index + 2);
                indices.push(base_index + 3);

                // =============================================================
                // Create cell object
                // =============================================================
                let cell = GridCell {
                    id: self.next_cell_id,
                    position: vec3(x + (cell_size / 2.0), 0.0, z + (cell_size / 2.0)),
                    width: cell_size, 
                    blocked: false,
                    adjacent_cells: vec![],
                };

                self.cells.push(cell);
                self.next_cell_id += 1;
            }
            
            if self.width % 2 == 0 {
                dark = !dark;
            } 
        }

        mesh.vertices.append(&mut vertices);
        mesh.indices.append(&mut indices);
        mesh.setup_mesh();

        mesh
    }

    pub fn generate_texture() {
        let width: u32 = 10;
        let height: u32 = 10;
        let color_dark: u8 = 25;
        let color_light: u8 = 67;

        let mut imgbuf = ImageBuffer::new(width, height);

        for (x, _y, pixel) in imgbuf.enumerate_pixels_mut() {
            if x < width / 2 {
                *pixel = Rgba([color_dark, color_dark, color_dark, 100]);
            } else {
                *pixel = Rgba([color_light, color_light, color_light, 100]);
            }
        }

        imgbuf
            .save("resources/textures/half_dark_half_light.png")
            .expect("Failed to save half dark / half light texture");
    }

    pub fn get_cell_from_position(&self, input: Vec3) -> Option<&GridCell> {
        let cell_x = (input.x / self.cell_size).floor() as usize;
        let cell_z = (input.z / self.cell_size).floor() as usize;

        // TODO: If you are out of bounds do something else, like a -1 or something to signify.
        let cell_id = cell_z * self.width + cell_x + 1;
        
        self.cells.get(cell_id)
    }

    pub fn draw(&mut self, shader: &mut Shader) {
        self.model.draw(shader);
    }
}
