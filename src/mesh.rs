use std::{ffi::CString, mem::{self, offset_of}};

use glam::{vec2, vec3, Vec2, Vec3};
use russimp::mesh::Mesh as RMesh;

use crate::{gl_call, shaders::Shader};

#[repr(C)]
#[derive(Debug)]
pub struct Vertex {
    pub position: Vec3,
    pub normal: Vec3,
    pub tex_coords: Vec2,
}

impl Vertex {
    pub fn new() -> Self {
        Self {
            position: vec3(0.0, 0.0, 0.0),
            normal: vec3(0.0, 0.0, 0.0),
            tex_coords: vec2(0.0, 0.0),
        }
    }
}

#[repr(C)]
#[derive(Debug)]
pub struct Texture {
    pub id: u32,
    pub _type: String,
}

#[repr(C)]
#[derive(Debug)]
pub struct Mesh {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
    pub textures: Vec<Texture>,
    pub vao: u32,
    pub vbo: u32,
    pub ebo: u32,
}

impl Mesh {
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
            gl::GenVertexArrays(1, &mut vao);
            gl::GenBuffers(1, &mut vbo);
            gl::GenBuffers(1, &mut ebo);
            
            gl::BindVertexArray(vao);
            gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
            
            gl::BufferData(
                gl::ARRAY_BUFFER, 
                self.vertices.len() as isize,
                self.vertices.as_ptr().cast(),
                gl::STATIC_DRAW,
            );

            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, ebo);
            gl::BufferData(
                gl::ELEMENT_ARRAY_BUFFER,
                self.indices.len() as isize,
                self.indices.as_ptr().cast(),
                gl::STATIC_DRAW
            );

            gl::EnableVertexAttribArray(0);
            gl::VertexAttribPointer(
                0, 
                3, 
                gl::FLOAT, 
                gl::FALSE, 
                mem::size_of::<Vertex>() as i32,
                0 as *const _
            );

            gl::EnableVertexAttribArray(1);
            gl::VertexAttribPointer(
                1, 
                3, 
                gl::FLOAT, 
                gl::FALSE, 
                mem::size_of::<Vertex>() as i32,
                offset_of!(Vertex, normal) as *const _
            );

            gl::EnableVertexAttribArray(2);
            gl::VertexAttribPointer(
                2, 
                2, 
                gl::FLOAT, 
                gl::FALSE, 
                mem::size_of::<Vertex>() as i32, 
                offset_of!(Vertex, tex_coords) as *const _
            );

            gl::BindVertexArray(0);
        }
    }


    pub fn draw(&self, shader: &mut Shader) {
        let mut diffuse_nr = 1;
        let mut specular_nr = 1;

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
            gl_call!(gl::DrawElements(gl::TRIANGLES, self.indices.len() as i32, gl::UNSIGNED_INT, 0 as *const _));
            gl_call!(gl::BindVertexArray(0));
        }
    }
}
