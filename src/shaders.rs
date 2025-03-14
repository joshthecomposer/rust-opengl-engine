#![allow(dead_code)]
use std::{collections::HashMap, ffi::CString, fs::read_to_string, ptr};

use gl::types::{GLint, GLuint};
use glam::{Mat4, Vec3};

use crate::{gl_call, lights::{DirLight, PointLight}};

pub struct Shader {
    pub id: GLuint,
    pub uniform_locations: HashMap<String, GLint>,
}

impl Shader {
    pub fn new(vs: &str, fs: &str) -> Self {
        let id = init_shader_program(vs, fs);
        Self {
            id,
            uniform_locations: HashMap::new(),
        }
    }

    pub fn activate(&self) {
        unsafe { gl_call!(gl::UseProgram(self.id)) }
    }

    pub fn store_uniform_location(&mut self, name: &str) {
        let c_name = CString::new(name).unwrap();
        let location = unsafe { gl_call!(gl::GetUniformLocation(self.id, c_name.as_ptr())) };
        self.uniform_locations.insert(name.to_string(), location);
    }

    pub fn store_dir_light_location(&mut self, name: &str) {
        self.store_uniform_location(format!("{}.direction", name).as_str());
        self.store_uniform_location(format!("{}.view_pos", name).as_str());
        self.store_uniform_location(format!("{}.ambient", name).as_str());
        self.store_uniform_location(format!("{}.diffuse", name).as_str());
        self.store_uniform_location(format!("{}.specular", name).as_str());
    }

    pub fn store_point_light_location(&mut self, name: &str) {
        self.store_uniform_location(format!("{}.position", name).as_str());
        self.store_uniform_location(format!("{}.ambient", name).as_str());
        self.store_uniform_location(format!("{}.diffuse", name).as_str());
        self.store_uniform_location(format!("{}.specular", name).as_str());
        self.store_uniform_location(format!("{}.constant", name).as_str());
        self.store_uniform_location(format!("{}.linear", name).as_str());
        self.store_uniform_location(format!("{}.quadratic", name).as_str());
    }

    pub fn get_uniform_location(&self, name: &str) -> GLint {
        *self.uniform_locations.get(name).unwrap_or(&-1)
    }

    pub fn set_vec3(&self, name: &str, value: Vec3) {
        let location = self.get_uniform_location(name);
        if location != -1 {
            unsafe { gl_call!(gl::Uniform3f(location, value.x, value.y, value.z)) }
        }
    }

    pub fn set_mat4(&self, name: &str, value: Mat4) {
        let location = self.get_uniform_location(name);
        if location != -1 {
            unsafe { gl_call!(gl::UniformMatrix4fv(location, 1, gl::FALSE, value.to_cols_array().as_ptr() )) }
        }
    }

    pub fn set_int(&self, name: &str, value: u32) {
        let location = self.get_uniform_location(name);
        if location != -1 {
            unsafe { gl_call!(gl::Uniform1i(location, value as i32)) };
        }
    }

    pub fn set_float(&self, name: &str, value: f32) {
        let location = self.get_uniform_location(name);
        if location != -1 {
            unsafe { gl_call!(gl::Uniform1f(location, value)) };
        }
    }

    pub fn set_dir_light(&self, name: &str, value: &DirLight) {
        let direction = self.get_uniform_location(format!("{}.direction", name).as_str());
        let view_pos = self.get_uniform_location(format!("{}.view_pos", name).as_str());
        let ambient = self.get_uniform_location(format!("{}.ambient", name).as_str());
        let diffuse = self.get_uniform_location(format!("{}.diffuse", name).as_str());
        let specular = self.get_uniform_location(format!("{}.specular", name).as_str());
        
        if direction != -1 || view_pos != -1 || ambient != -1 || diffuse != -1 ||
            specular != -1 {
            unsafe { 
                gl_call!(gl::Uniform3f(direction, value.direction.x, value.direction.y, value.direction.z));
                gl_call!(gl::Uniform3f(view_pos, value.view_pos.x, value.view_pos.y, value.view_pos.z));
                gl_call!(gl::Uniform3f(ambient, value.ambient.x, value.ambient.y, value.ambient.z));
                gl_call!(gl::Uniform3f(diffuse, value.diffuse.x, value.diffuse.y, value.diffuse.z));
                gl_call!(gl::Uniform3f(specular, value.specular.x, value.specular.y, value.specular.z));
            }
        }
    }

    pub fn set_point_light(&self, name: &str, value: &PointLight) {
        let position = self.get_uniform_location(format!("{}.position", name).as_str());
        let ambient = self.get_uniform_location(format!("{}.ambient", name).as_str());
        let diffuse = self.get_uniform_location(format!("{}.diffuse", name).as_str());
        let specular = self.get_uniform_location(format!("{}.specular", name).as_str());
        let constant = self.get_uniform_location(format!("{}.constant", name).as_str());
        let linear = self.get_uniform_location(format!("{}.linear", name).as_str());
        let quadratic = self.get_uniform_location(format!("{}.quadratic", name).as_str());
        
        if position != -1 || ambient != -1 || diffuse != -1 || specular != -1 ||
            constant != -1 || linear != -1 || quadratic != -1 {
            unsafe { 
                gl_call!(gl::Uniform3f(position, value.position.x, value.position.y, value.position.z));
                gl_call!(gl::Uniform3f(ambient, value.ambient.x, value.ambient.y, value.ambient.z));
                gl_call!(gl::Uniform3f(diffuse, value.diffuse.x, value.diffuse.y, value.diffuse.z));
                gl_call!(gl::Uniform3f(specular, value.specular.x, value.specular.y, value.specular.z));
                gl_call!(gl::Uniform1f(constant, value.constant));
                gl_call!(gl::Uniform1f(linear, value.linear));
                gl_call!(gl::Uniform1f(quadratic, value.quadratic));
            }
        }
    }

    pub fn set_mat4_array(&self, name: &str, value: &Vec<Mat4>) {
        let location = self.get_uniform_location(name);
        if location != -1 {
            let mut float_data = Vec::with_capacity(value.len() * 16);

            for mat in value {
                float_data.extend_from_slice(&mat.to_cols_array());
            }

            unsafe {
                gl_call!(gl::UniformMatrix4fv(location, value.len() as i32, gl::FALSE, float_data.as_ptr()));
            }
        }
    }

}

pub fn init_shader_program(vs: &str, fs: &str) -> u32 {
    println!("Loading vs: {}", vs);
    println!("Loading fs: {}", fs);
    let vs_source = read_to_string(vs).unwrap();
    let fs_source = read_to_string(fs).unwrap();

    let vs_cstr = CString::new(vs_source).expect("Failed to convert vs source to C string");
    let fs_cstr = CString::new(fs_source).expect("Failed to convert vs source to C string");
    
    unsafe {

        let vertex_shader = gl::CreateShader(gl::VERTEX_SHADER);
        let fragment_shader = gl::CreateShader(gl::FRAGMENT_SHADER);

        gl::ShaderSource(vertex_shader, 1, &vs_cstr.as_ptr(), ptr::null());
        gl::ShaderSource(fragment_shader, 1, &fs_cstr.as_ptr(), ptr::null());

        compile_shader(vertex_shader);
        compile_shader(fragment_shader);

        let shader = gl::CreateProgram();

        gl::AttachShader(shader, vertex_shader);
        gl::AttachShader(shader, fragment_shader);

        gl::LinkProgram(shader);

        gl::DeleteShader(vertex_shader);
        gl::DeleteShader(fragment_shader);

        shader
    }
}

fn compile_shader(input: u32) {
    unsafe {
        gl::CompileShader(input);

        let mut success:i32 = 0;
        let mut info_log = vec![0u8; 512];

        gl::GetShaderiv(input, gl::COMPILE_STATUS, &mut success);

        if success == 0 {
            gl::GetShaderInfoLog(input, 512, core::ptr::null_mut(), info_log.as_mut_ptr() as *mut i8);
            println!("Problem compiling shader: {:?}", String::from_utf8_lossy(&info_log));
        }
    }
}
