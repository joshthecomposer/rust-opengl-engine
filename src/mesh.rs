use std::mem::{self, offset_of};

use glam::{Vec2, Vec3};

use crate::shaders::Shader;

#[repr(C)]
pub struct Vertex {
    pub position: Vec3,
    pub normal: Vec3,
    pub tex_coords: Vec2,
}

#[repr(C)]
pub struct Texture {
    pub id: u32,
    pub _type: String,
}

#[repr(C)]
pub struct Mesh {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
    pub textures: Vec<Texture>,
    pub vao: u32,
    pub vbo: u32,
    pub ebo: u32,
}

impl Mesh {
    pub fn new(
        vertices: Vec<Vertex>,
        indices: Vec<u32>,
        textures: Vec<Texture>,
    ) -> Mesh {
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
                vertices.len() as isize,
                vertices.as_ptr().cast(),
                gl::STATIC_DRAW,
            );

            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, ebo);
            gl::BufferData(
                gl::ELEMENT_ARRAY_BUFFER,
                indices.len() as isize,
                indices.as_ptr().cast(),
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
        Self {
            vertices,
            indices,
            textures,
            vao,
            vbo,
            ebo
        }
    }

    pub fn draw(shader: &Shader) {
    }
}
