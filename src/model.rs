#![allow(dead_code)]
use std::{ffi::c_void, path::Path};

use glam::{vec3, Vec3};
use image::{GenericImageView, ImageBuffer, Rgba};
use russimp::{material::{Material as RMaterial, PropertyTypeInfo, TextureType}, mesh::Mesh as RMesh, node::Node, scene::{PostProcess, Scene}, Vector3D};

use crate::{gl_call, mesh::{Mesh, Texture, Vertex}, shaders::Shader};

#[derive(Clone)]
pub struct Model {
    pub meshes: Vec<Mesh>,
    pub directory: String,
    pub textures_loaded: Vec<Texture>,
    pub full_path: String,
}

impl Model {
    pub fn new() -> Self {
        Self {
            meshes: vec![],
            directory: "".to_string(),
            textures_loaded: vec![],
            full_path: "".to_string(),
        }
    }

    pub fn load(path: &str) -> Model {
        let mut model = Model::new();
        model.full_path = path.to_string();
        println!("=============================================================");
        println!("BEGIN LOADING OF SCENE FROM PATH: {}", path);
        println!("=============================================================");

        let scene = Scene::from_file(
            path, 
            vec![
                 PostProcess::Triangulate,
                // PostProcess::GenerateSmoothNormals,
                PostProcess::FlipUVs,
            ],
        ).unwrap();

        if scene.root.is_none() {
            panic!("Scene had no root node :/");
        }

        let directory = Path::new(path).parent().unwrap().to_str().unwrap();

        println!("Directory of model is: {}", &directory);
        println!("=============================================================");

        model.directory = directory.to_string();

        println!("BEGIN PROCESSING OF NODES RECURSIVELY");
        if let Some(root_node) = &scene.root {
            model.traverse_nodes(root_node, &scene);
        }

        model
    }

    fn traverse_nodes(&mut self, node: &Node, scene: &Scene) {
        for mesh_index in node.meshes.clone() {
            let ai_mesh = &scene.meshes[mesh_index as usize];
            let mesh = self.process_mesh(ai_mesh, scene);
            self.meshes.push(mesh);
        }

        for child in node.children.borrow().iter() {
           self.traverse_nodes(child, scene);
        }
    }

    pub fn process_mesh(&mut self, ai_mesh: &RMesh, scene: &Scene) -> Mesh {
        let mut mesh = Mesh::new();
        // Vertices
        for (i, ai_vertex) in ai_mesh.vertices.iter().enumerate() {
            let mut vertex = Vertex::new();

            vertex.position = vec3(ai_vertex.x, ai_vertex.y, ai_vertex.z);

            let ai_norm = ai_mesh.normals.get(i).unwrap_or(&Vector3D {x: 0.0, y: 1.0, z: 0.0});
            vertex.normal = vec3(ai_norm.x, ai_norm.y, ai_norm.z);

            if let Some(tex_coord_list) = ai_mesh.texture_coords.first().unwrap() {
                if let Some(ai_tex_coord) = tex_coord_list.get(i) {
                    let tex_coord = glam::Vec2 {x: ai_tex_coord.x, y: ai_tex_coord.y};
                    vertex.tex_coords = tex_coord;
                }
            } else {
                let tex_coord = glam::Vec2 {x: 0.0, y: 0.0};
                vertex.tex_coords = tex_coord;
            }
            mesh.vertices.push(vertex);
        }
        
        // Indices
        for face in ai_mesh.faces.iter() {
            for index in face.0.iter() {
                mesh.indices.push(*index);
            }
        }

        // Materials
        let material_index = ai_mesh.material_index;
        // if material_index >= 0 {
            if let Some(ai_mat) = scene.materials.get(material_index as usize) {
                let mut diffuse_textures = self.load_material_textures(ai_mat, TextureType::Diffuse, "texture_diffuse".to_string());
                let mut specular_textures = self.load_material_textures(ai_mat, TextureType::Specular, "texture_specular".to_string());
                mesh.textures.append(&mut diffuse_textures);
                mesh.textures.append(&mut specular_textures);
            }
        // }
        mesh.setup_mesh();
        mesh
    }

    pub fn load_material_textures(&mut self, ai_mat: &RMaterial, texture_type: TextureType, my_type: String) -> Vec<Texture> {
        let mut textures: Vec<Texture> = vec![];
        let path;
        let mut skip = false;

        if let Some(ai_texes_cell) = ai_mat.textures.get(&texture_type) {
            dbg!("found their diffuse");
            let ai_tex = ai_texes_cell.borrow();
            path = ai_tex.filename.clone();
        } else if let Some(found_path) = Self::try_parse_diffuse_texture_path(ai_mat, texture_type) {
                path = found_path.clone();
        } else if let Some(diffuse_color) = Self::try_parse_diffuse_color(ai_mat) {
            dbg!("Falling back to diffuse_color");
            // TODO: This overwrites multiple times potentially, we should check if the texture has already been saved.
            let mut imgbuf = ImageBuffer::new(1,1);

            let color_u8 = [
                (diffuse_color.x * 255.0) as u8,
                (diffuse_color.y * 255.0) as u8,
                (diffuse_color.z * 255.0) as u8,
                255,
            ];

            for pixel in imgbuf.pixels_mut() {
                *pixel = Rgba(color_u8);
            }

            path = format!("{:.3}-{:.3}-{:.3}.png" ,diffuse_color.x, diffuse_color.y, diffuse_color.z);
            let save_loc = format!("{}/{:.3}-{:.3}-{:.3}.png", self.directory ,diffuse_color.x, diffuse_color.y, diffuse_color.z);

            imgbuf
                .save(save_loc)
                .expect("Failed to save texture image");
        } else {
            dbg!("Didn't find a texture and no color found");
            return vec![];
        }
            
        for tex in self.textures_loaded.iter() {
            if tex.path == path  {
                textures.push(tex.clone());
                skip = true;
                break;
            }
        }

        if !skip {
            let tex_id = Self::texture_from_file(self, path.clone());
            let texture = Texture {id: tex_id, _type: my_type, path: path.clone()};
            textures.push(texture.clone());
            self.textures_loaded.push(texture.clone());
        }

        textures
    }

    pub fn generate_texture_from_color(_input: Vec3) -> String {
        "".to_string()
    }

    pub fn try_parse_diffuse_color(ai_mat: &RMaterial) -> Option<Vec3> {
        for prop in ai_mat.properties.iter() {
            if prop.key == "$clr.diffuse" {
                if let PropertyTypeInfo::FloatArray(ref data) = prop.data {
                    if data.len() >= 3 {
                        return Some(vec3(data[0], data[1], data[2]));
                    }
                }
            }
        }
        None
    }

    pub fn try_parse_diffuse_texture_path(ai_mat: &RMaterial, tex_type: TextureType) -> Option<String> {
        for prop in ai_mat.properties.iter() {
            if prop.key == "$tex.file" && prop.semantic == tex_type {
                if let PropertyTypeInfo::String(ref filename) = prop.data {
                    return Some(filename.clone());
                }
            }
        }
        None
    }

    pub fn texture_from_file(model: &Model, path: String) -> u32 {
        let file_name = model.directory.clone() + "/" + path.as_str();

        dbg!(&path);
        dbg!(&file_name);

        let mut texture_id = 0;
        unsafe {
            gl_call!(gl::GenTextures(1, &mut texture_id));

            let img = image::open(file_name).unwrap();
            let (img_width, img_height) = img.dimensions();
            let rgba = img.to_rgba8();
            let raw = rgba.as_raw();

            gl_call!(gl::BindTexture(gl::TEXTURE_2D, texture_id));
            gl_call!(gl::TexImage2D(
                gl::TEXTURE_2D, 
                0, 
                gl::RGBA as i32, 
                img_width as i32, 
                img_height as i32, 
                0, 
                gl::RGBA, 
                gl::UNSIGNED_BYTE, 
                raw.as_ptr() as *const c_void
            ));

            gl_call!(gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::REPEAT as i32));
            gl_call!(gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::REPEAT as i32));
            gl_call!(gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST as i32));
            gl_call!(gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as i32));
            gl_call!(gl::GenerateMipmap(gl::TEXTURE_2D));
        }
        
        texture_id
    }

    pub fn draw(&self, shader: &mut Shader) {
        for mesh in self.meshes.iter() {
            mesh.draw(shader);
        }
    }

    pub fn shadow_pass(&self, shader: &mut Shader) {
        shader.activate();
        for mesh in self.meshes.iter() {
            unsafe {
                gl_call!(gl::BindVertexArray(mesh.vao));
            }
        }
    }
}

