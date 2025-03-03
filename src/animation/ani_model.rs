use std::{collections::HashMap, ffi::c_void, path::Path};

use glam::{vec3, Mat4, Vec3, Vec4};
use image::{GenericImageView, ImageBuffer, Rgba};
use russimp::{bone::VertexWeight, material::{Material as RMaterial, MaterialProperty, PropertyTypeInfo, TextureType, }, mesh::Mesh as RMesh, node::Node, scene::{PostProcess, Scene}, Matrix4x4, Vector3D};

use crate::{gl_call, mesh::Texture, shaders::Shader, some_data::MAX_BONE_INFLUENCE};

use super::ani_mesh::{AniMesh, AniVertex};

#[derive(Debug, Clone)]
pub struct BoneInfo {
    pub id: i32,
    pub offset: Mat4,
}

#[derive(Debug, Clone)]
pub struct AniModel {
    pub meshes: Vec<AniMesh>,
    pub directory: String,
    pub textures_loaded: Vec<Texture>,
    pub full_path: String,

    pub bone_info_map: HashMap<String, BoneInfo>,
    pub bone_counter: i32,
}

impl AniModel {
    pub fn new() -> Self {
        Self {
            meshes: vec![],
            directory: "".to_string(),
            textures_loaded: vec![],
            full_path: "".to_string(),
            bone_info_map: HashMap::new(),
            bone_counter: 0,
        }
    }

    pub fn load(path: &str) -> Self {
        let mut model = Self::new();
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

        if !scene.root.is_some() {
            panic!("Scene had no root node :/");
        }

        let directory = Path::new(path).parent().unwrap().to_str().unwrap();

        println!("Directory of ani_model is: {}", &directory);
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

    pub fn process_mesh(&mut self, ai_mesh: &RMesh, scene: &Scene) -> AniMesh {
        let mut mesh = AniMesh::new();
        // Vertices
        for (i, ai_vertex) in ai_mesh.vertices.iter().enumerate() {
            let mut vertex = AniVertex::new();

            vertex.position = vec3(ai_vertex.x, ai_vertex.y, ai_vertex.z);

            let ai_norm = ai_mesh.normals.get(i).unwrap_or(&Vector3D {x: 0.0, y: 1.0, z: 0.0});
            vertex.normal = vec3(ai_norm.x, ai_norm.y, ai_norm.z);

            if let Some(tex_coord_list) = ai_mesh.texture_coords.get(0).unwrap() {
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

        self.extract_bone_weight_for_vertices(&mut mesh.vertices, ai_mesh, scene);
        mesh
    }

    pub fn extract_bone_weight_for_vertices(&mut self, vertices: &mut Vec<AniVertex>, ai_mesh: &RMesh, scene: &Scene) {
        println!("=============================================================");
        println!("EXTRACTING BONE DATA... NUM BONES TO PROCESS: {:?}", ai_mesh.bones.iter().count());
        println!("=============================================================");

        for (b_index, bone) in ai_mesh.bones.iter().enumerate() {
            let mut bone_id: i32 = -1;

            let bone_name = bone.name.clone();
            if let Some(bone_info) = self.bone_info_map.get(&bone_name) {
                bone_id = bone_info.id as i32;
            } else {
                let new_bone_info = BoneInfo {
                    id: self.bone_counter,
                    offset: Self::russimp_mat4_to_glam(bone.offset_matrix),
                };
                self.bone_info_map.insert(bone_name, new_bone_info);
                bone_id = self.bone_counter;
                self.bone_counter += 1;
            }
            assert!(bone_id != -1);

            for weight in bone.weights.iter() {
                let vertex_id = weight.vertex_id;
                assert!(vertex_id <= vertices.len() as u32);
                let vertex = vertices.get_mut(vertex_id as usize).unwrap();
                Self::set_vertex_bone_data(vertex, bone_id, weight.weight);
            }
        }
    }

    pub fn set_vertex_bone_data(vertex: &mut AniVertex, bone_id: i32, weight: f32) {
        // TODO: THis seems really bad, there must be a better way to limit this to MAX_BONE_INFLUENCE.
        for i in 0..MAX_BONE_INFLUENCE {
            if vertex.bone_ids[i] < 0 {
                vertex.bone_ids[i] = bone_id;
                vertex.weights[i] = weight;
                break;
            }
        }
    }

    pub fn russimp_mat4_to_glam(from: Matrix4x4) -> Mat4 {
        Mat4::from_cols_array(&[
            from.a1, from.a2, from.a3, from.a4,
            from.b1, from.b2, from.b3, from.b4,
            from.c1, from.c2, from.c3, from.c4,
            from.d1, from.d2, from.d3, from.d4,
        ])
    }

    pub fn load_material_textures(&mut self, ai_mat: &RMaterial, texture_type: TextureType, my_type: String) -> Vec<Texture> {
        let mut textures: Vec<Texture> = vec![];
        let mut path = "".to_string();
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

    pub fn generate_texture_from_color(input: Vec3) -> String {
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


    pub fn texture_from_file(model: &Self, path: String) -> u32 {
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
