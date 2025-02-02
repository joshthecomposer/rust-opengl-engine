use std::{collections::HashMap, ffi::CString, fs::read_to_string, ptr};

use gl::types::{GLint, GLuint};
use glam::{Mat4, Vec3};

use crate::gl_call;

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

}

pub fn init_shader_program(vs: &str, fs: &str) -> u32 {
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

        return shader;
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
