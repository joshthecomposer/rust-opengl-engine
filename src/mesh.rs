use std::mem::{self, offset_of};
use glam::{vec2, vec3, Vec2, Vec3};
use crate::{gl_call, shaders::Shader};

#[repr(C)]
#[derive(Debug, Clone)]
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
#[derive(Debug, Clone)]
pub struct Texture {
    pub id: u32,
    pub _type: String,
    pub path: String,
}

#[repr(C)]
#[derive(Debug, Clone)]
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
            gl_call!(gl::GenVertexArrays(1, &mut vao));
            gl_call!(gl::GenBuffers(1, &mut vbo));
            gl_call!(gl::GenBuffers(1, &mut ebo));
            
            gl_call!(gl::BindVertexArray(vao));
            gl_call!(gl::BindBuffer(gl::ARRAY_BUFFER, vbo));
            
            gl_call!(gl::BufferData(
                gl::ARRAY_BUFFER, 
                (mem::size_of::<Vertex>() * self.vertices.len()) as isize,
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
                mem::size_of::<Vertex>() as i32,
                std::ptr::null(),
            ));

            gl_call!(gl::EnableVertexAttribArray(1));
            gl_call!(gl::VertexAttribPointer(
                1, 
                3, 
                gl::FLOAT, 
                gl::FALSE, 
                mem::size_of::<Vertex>() as i32,
                offset_of!(Vertex, normal) as *const _
            ));

            gl_call!(gl::EnableVertexAttribArray(2));
            gl_call!(gl::VertexAttribPointer(
                2, 
                2, 
                gl::FLOAT, 
                gl::FALSE, 
                mem::size_of::<Vertex>() as i32, 
                offset_of!(Vertex, tex_coords) as *const _
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
        let mut alpha_nr = 1;

        shader.activate();

        for (i, texture) in self.textures.iter().enumerate() {
            unsafe {
                gl_call!(gl::ActiveTexture(gl::TEXTURE0 + i as u32));
            }

            let mut number: u32 = 0;
            let name = &texture._type;
            if name == "texture_diffuse" {
                number = diffuse_nr;

                diffuse_nr += 1;
            } else if name == "texture_specular" {
                number = specular_nr;
                specular_nr += 1;
            } else if name == "texture_alpha" {
                number = alpha_nr;
                shader.set_bool("has_opacity_texture", true);
                alpha_nr += 1;
            }

            let final_str = name.to_string() + &number.to_string();
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
                std::ptr::null(),
            ));
            gl_call!(gl::BindVertexArray(0));
            gl_call!(gl::BindTexture(gl::TEXTURE_2D, 0));
        }

        shader.set_bool("has_opacity_texture", false);
    }
}
