use std::mem::{self, offset_of};

use glam::{vec2, vec3, Vec2, Vec3};

use crate::{gl_call, mesh::Texture, shaders::Shader, some_data::MAX_BONE_INFLUENCE};

#[derive(Debug, Clone)]
pub struct AniVertex {
    pub position: Vec3,
    pub normal: Vec3,
    pub tex_coords: Vec2,

    pub tangent: Vec3,
    pub bitangent: Vec3,

    pub bone_ids: [i32; MAX_BONE_INFLUENCE],
    pub weights: [f32; MAX_BONE_INFLUENCE],
}

impl AniVertex {
    pub fn new() -> Self {
        Self {
            position: vec3(0.0, 0.0, 0.0),
            normal: vec3(0.0, 0.0, 0.0),
            tex_coords: vec2(0.0, 0.0),

            tangent: vec3(0.0, 0.0, 0.0),
            bitangent: vec3(0.0, 0.0, 0.0),

            bone_ids: [-1; MAX_BONE_INFLUENCE],
            weights: [0.0; MAX_BONE_INFLUENCE],
        }
    }
}

#[derive(Debug, Clone)]
pub struct AniMesh {
    pub vertices: Vec<AniVertex>,
    pub indices: Vec<u32>,
    pub textures: Vec<Texture>,
    pub vao: u32,
    pub vbo: u32,
    pub ebo: u32,
}

impl AniMesh {
    pub fn new() -> Self {
        Self {
            vertices: vec![],
            indices: vec![],
            textures: vec![],
            vao: 0,
            vbo: 0,
            ebo: 0,
        }
    }

    pub fn setup_mesh(&mut self) {
        let mut vao = 0;
        let mut vbo = 0;
        let mut ebo = 0;
        
        unsafe {
            gl_call!(gl::GenVertexArrays(1, &mut vao));
            gl_call!(gl::GenBuffers(1, &mut vbo));
            gl_call!(gl::GenBuffers(1, &mut ebo));
            
            gl_call!(gl::BindVertexArray(vao));
            gl_call!(gl::BindBuffer(gl::ARRAY_BUFFER, vbo));
            
            gl_call!(gl::BufferData(
                gl::ARRAY_BUFFER, 
                (mem::size_of::<AniVertex>() * self.vertices.len()) as isize,
                self.vertices.as_ptr().cast(),
                gl::STATIC_DRAW,
            ));

            gl_call!(gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, ebo));
            gl_call!(gl::BufferData(
                gl::ELEMENT_ARRAY_BUFFER,
                (mem::size_of::<u32>() * self.indices.len()) as isize,
                self.indices.as_ptr().cast(),
                gl::STATIC_DRAW
            ));

            gl_call!(gl::EnableVertexAttribArray(0));
            gl_call!(gl::VertexAttribPointer(
                0, 
                3, 
                gl::FLOAT, 
                gl::FALSE, 
                mem::size_of::<AniVertex>() as i32,
                0 as *const _
            ));

            gl_call!(gl::EnableVertexAttribArray(1));
            gl_call!(gl::VertexAttribPointer(
                1, 
                3, 
                gl::FLOAT, 
                gl::FALSE, 
                mem::size_of::<AniVertex>() as i32,
                offset_of!(AniVertex, normal) as *const _
            ));

            gl_call!(gl::EnableVertexAttribArray(2));
            gl_call!(gl::VertexAttribPointer(
                2, 
                2, 
                gl::FLOAT, 
                gl::FALSE, 
                mem::size_of::<AniVertex>() as i32, 
                offset_of!(AniVertex, tex_coords) as *const _
            ));

            gl_call!(gl::EnableVertexAttribArray(3));
            gl_call!(gl::VertexAttribIPointer( 
                3,
                4,
                gl::INT,
                mem::size_of::<AniVertex>() as i32,
                offset_of!(AniVertex, bone_ids) as *const _
            ));

            gl_call!(gl::EnableVertexAttribArray(4));
            gl_call!(gl::VertexAttribPointer(
                4,
                4,
                gl::FLOAT,
                gl::FALSE,
                mem::size_of::<AniVertex>() as i32,
                offset_of!(AniVertex, weights) as *const _
            ));


            self.vao = vao;
            self.vbo = vbo;
            self.ebo = ebo;
            
            gl_call!(gl::BindVertexArray(0));
        }
    }


    pub fn draw(&self, shader: &mut Shader) {
        let mut diffuse_nr = 1;
        let mut specular_nr = 1;

        shader.activate();

        for (i, texture) in self.textures.iter().enumerate() {
            unsafe {
                gl_call!(gl::ActiveTexture(gl::TEXTURE0 + i as u32));
            }

            let mut number = "".to_string();
            let name = &texture._type;
            if name == "texture_diffuse" {
                number = diffuse_nr.to_string();
                diffuse_nr += 1;
            } else if name == "texture_specular" {
                number = specular_nr.to_string();
                specular_nr += 1;
            }
            let final_str = ("material.".to_string() + name.as_str()) + number.as_str();
            shader.store_uniform_location(final_str.as_str());
            shader.set_int(final_str.as_str(), i as u32);

            unsafe {
                gl_call!(gl::BindTexture(gl::TEXTURE_2D, texture.id));
            }
        }

        unsafe {
            gl_call!(gl::BindVertexArray(self.vao));

            gl_call!(gl::DrawElements(
                gl::TRIANGLES, 
                self.indices.len() as i32, 
                gl::UNSIGNED_INT, 
                0 as *const _
            ));
            gl_call!(gl::BindVertexArray(0));
        }
    }
}


