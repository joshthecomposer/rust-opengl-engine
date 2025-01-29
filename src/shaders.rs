use std::{ffi::CString, fs::read_to_string, ptr};

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
        let mut info_log:[i8; 512] = [0; 512];

        gl::GetShaderiv(input, gl::COMPILE_STATUS, &mut success);

        if success == 0 {
            gl::GetShaderInfoLog(input, 512, core::ptr::null_mut(), info_log.as_mut_ptr());
            println!("Problem compiling shader: {:?}", info_log);
        }
    }
}
