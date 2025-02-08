use std::{ffi::c_void, path::Path};

use glam::vec3;
use image::GenericImageView;
use russimp::{material::{Material as RMaterial, PropertyTypeInfo, TextureType}, mesh::Mesh as RMesh, node::Node, scene::{PostProcess, Scene}};
use russimp_sys::AI_SCENE_FLAGS_INCOMPLETE;

use crate::{gl_call, mesh::{Mesh, Texture, Vertex}, shaders::Shader};

pub struct Model {
    pub meshes: Vec<Mesh>,
    pub directory: String,
}

impl Model {
    fn new() -> Self {
        Self {

            meshes: vec![],
            directory: "".to_string(),
        }
    }

    pub fn load(path: &str) -> Model {
        let mut model = Model::new();
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

        if !scene.root.is_some() {
            panic!("Scene had no root node :/");
        }

        let directory = Path::new(path).parent().unwrap().to_str().unwrap();

        println!("Directory of model is: {}", &directory);
        println!("=============================================================");

        model.directory = directory.to_string();

        println!("BEGIN PROCESSING OF NODES RECURSIVELY");
        if let Some(root_node) = &scene.root {
            model.traverse_nodes(&*root_node, &scene);
        }

        return model;
    }

    fn traverse_nodes(&mut self, node: &Node, scene: &Scene) {
        for mesh_index in node.meshes.clone() {
            let ai_mesh = &scene.meshes[mesh_index as usize];
            let mut mesh = self.process_mesh(ai_mesh, scene);
            self.meshes.push(mesh);
        }

        for child in node.children.borrow().iter() {
           self.traverse_nodes(&*child, scene);
        }
    }

    pub fn process_mesh(&mut self, ai_mesh: &RMesh, scene: &Scene) -> Mesh {
        let mut mesh = Mesh::new();
        // Vertices
        for (i, ai_vertex) in ai_mesh.vertices.iter().enumerate() {
            let mut vertex = Vertex::new();

            vertex.position = vec3(ai_vertex.x, ai_vertex.y, ai_vertex.z);

            let ai_norm = ai_mesh.normals.get(i).unwrap();
            vertex.normal = vec3(ai_norm.x, ai_norm.y, ai_norm.z);

            if let Some(tex_coord_list) = ai_mesh.texture_coords.get(0).unwrap() {
                if let Some(ai_tex_coord) = tex_coord_list.get(i) {
                    let tex_coord = glam::Vec2 {x: ai_tex_coord.x, y: ai_tex_coord.y};
                    vertex.tex_coords = tex_coord;
                }
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
                let mut more_textures = self.load_material_textures(ai_mat, TextureType::None, "texture_diffuse".to_string());
                let mut base_color = self.load_material_textures(ai_mat, TextureType::BaseColor, "texture_diffuse".to_string());
                let mut unknown = self.load_material_textures(ai_mat, TextureType::Unknown, "texture_diffuse".to_string());
                let mut specular_textures = self.load_material_textures(ai_mat, TextureType::Specular, "texture_specular".to_string());
                mesh.textures.append(&mut diffuse_textures);
                mesh.textures.append(&mut more_textures);
                mesh.textures.append(&mut base_color);
                mesh.textures.append(&mut unknown);
                mesh.textures.append(&mut specular_textures);
            }
        // }
        mesh.setup_mesh();
        mesh
    }

    pub fn load_material_textures(&self, ai_mat: &RMaterial, texture_type: TextureType, my_type: String) -> Vec<Texture> {
        let mut textures: Vec<Texture> = vec![];
        if let Some(ai_texes_cell) = ai_mat.textures.get(&texture_type) {
            let ai_tex = ai_texes_cell.borrow();
            let tex_id = Self::texture_from_file(self, ai_tex.filename.clone());
            textures.push(Texture {id: tex_id, _type: my_type});
        } else {
            if texture_type == TextureType::Diffuse {
                if let Some(found_path) = Self::try_parse_diffuse_texture_path(ai_mat) {
                    let tex_id = Self::texture_from_file(self, found_path);
                    dbg!(tex_id);
                    textures.push(Texture {id: tex_id, _type: my_type});
                }
            }
        }
        textures
    }

    pub fn try_parse_diffuse_texture_path(ai_mat: &RMaterial) -> Option<String> {
        for prop in ai_mat.properties.iter() {
            if prop.key == "$tex.file" && prop.semantic == TextureType::Diffuse {
                if let PropertyTypeInfo::String(ref filename) = prop.data {
                    return Some(filename.clone());
                }
            }
        }
        None
    }

    pub fn texture_from_file(model: &Model, path: String) -> u32 {
        let file_name = model.directory.clone() + "/" + path.as_str();

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
            gl_call!(gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR_MIPMAP_LINEAR as i32));
            gl_call!(gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32));
            gl_call!(gl::GenerateMipmap(gl::TEXTURE_2D));
        }
        
        texture_id
    }

    pub fn draw(&self, shader: &mut Shader) {
        for mesh in self.meshes.iter() {
            mesh.draw(shader);
        }
    }
}

