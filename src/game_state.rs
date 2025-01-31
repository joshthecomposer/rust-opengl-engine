
use std::{collections::HashMap, ffi::{c_void, CString}, mem};

use gl::TEXTURE_CUBE_MAP;
use glam::{vec3, vec4, Mat4};
use glfw::{Context, Glfw, GlfwReceiver, PWindow, WindowEvent};
use image::GenericImageView;

use crate::{camera::Camera, enums_types::{EboType, ShaderType, VaoType, VboType}, gl_call, shaders, some_data::{CUBE_POSITIONS, FACES_CUBEMAP, SKYBOX_INDICES, SKYBOX_VERTICES, UNIT_CUBE_VERTICES}};

pub struct GameState {
    pub delta_time: f64,
    pub last_frame: f64,
    pub camera: Camera,
    pub window_width: u32,
    pub window_height: u32,

    // GLFW context
    pub glfw: Glfw,
    pub events: GlfwReceiver<(f64, WindowEvent)>,
    pub window: PWindow,

    pub shaders: HashMap<ShaderType, u32>, // TODO: make this an enum
    pub vaos: HashMap<VaoType, u32>,
    pub vbos: HashMap<VboType, u32>,
    pub ebos: HashMap<EboType, u32>,

    pub texture: u32,
    pub cubemap_texture: u32,
}

impl GameState {
    pub fn new() -> Self {
        let mut glfw = glfw::init(glfw::fail_on_errors).expect("Failed to init glfw");

        glfw.window_hint(glfw::WindowHint::ContextVersion(3, 3)); // OpenGL 3.3
        glfw.window_hint(glfw::WindowHint::OpenGlProfile(glfw::OpenGlProfileHint::Core));
        glfw.window_hint(glfw::WindowHint::Resizable(true));
        #[cfg(target_os = "macos")]
        glfw.window_hint(glfw::WindowHint::OpenGlForwardCompat(true));

let (mut width,mut height):(i32, i32) = (1280, 720);

        let (mut window, events) = glfw
            .create_window(width as u32, height as u32, "Hello this is window", glfw::WindowMode::Windowed)
            .expect("Failed to create GLFW window.");
        window.set_key_polling(true);
        // window.set_sticky_keys(true); 
        window.set_cursor_pos_polling(true);
        window.set_cursor_mode(glfw::CursorMode::Disabled);
        window.make_current();

        gl::load_with(|symbol| window.get_proc_address(symbol) as *const _);

        unsafe {
            gl_call!(gl::Enable(gl::BLEND));
            gl_call!(gl::Enable(gl::TEXTURE_CUBE_MAP_SEAMLESS));
            gl_call!(gl::Viewport(0, 0, 1280, 720));
            gl_call!(gl::Enable(gl::DEPTH_TEST));
        }

        
        let mut shaders = HashMap::new();
        let mut vaos = HashMap::new();
        let mut vbos = HashMap::new();
        let mut ebos = HashMap::new();

        let main_shader = shaders::init_shader_program("resources/shaders/shader.vs", "resources/shaders/shader.fs");
        let skybox_shader = shaders::init_shader_program("resources/shaders/skybox.vs", "resources/shaders/skybox.fs");

        shaders.insert(ShaderType::MainShader, main_shader);
        shaders.insert(ShaderType::SkyboxShader, skybox_shader);


        let mut vao = 0;
        let mut vbo = 0;
        let mut ebo = 0;
        let mut texture = 0;
        let mut cubemap_texture = 0;
        // =============================================================
        // Skybox memes
        // =============================================================
        unsafe {
            gl_call!(gl::GenVertexArrays(1, &mut vao));
            gl_call!(gl::GenBuffers(1, &mut vbo));
            gl_call!(gl::GenBuffers(1, &mut ebo));

            vaos.insert(VaoType::SkyboxVao, vao);
            vbos.insert(VboType::SkyboxVbo, vbo);
            ebos.insert(EboType::SkyboxEbo, ebo);

            println!("vao skybox: {}", vao);

            gl_call!(gl::BindVertexArray(vao));

            gl_call!(gl::BindBuffer(gl::ARRAY_BUFFER, vbo));
            gl_call!(gl::BufferData(
                gl::ARRAY_BUFFER, 
                (mem::size_of::<f32>() * SKYBOX_VERTICES.len()) as isize,
                SKYBOX_VERTICES.as_ptr().cast(),
                gl::STATIC_DRAW
            ));

            gl_call!(gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, ebo));
            gl_call!(gl::BufferData(
                gl::ELEMENT_ARRAY_BUFFER,
                (mem::size_of::<u32>() * SKYBOX_INDICES.len()) as isize,
                SKYBOX_INDICES.as_ptr().cast(),
                gl::STATIC_DRAW
            ));

            gl_call!(gl::VertexAttribPointer(
                0, 
                3, 
                gl::FLOAT, 
                gl::FALSE, 
                (3 * mem::size_of::<f32>()) as i32, 
                0 as *const _
            ));
            gl_call!(gl::EnableVertexAttribArray(0));

            gl_call!(gl::BindVertexArray(0));
            gl_call!(gl::BindBuffer(gl::ARRAY_BUFFER, 0));
            gl_call!(gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, 0));

            // SKYBOX TEXTURES
            gl_call!(gl::GenTextures(1, &mut cubemap_texture));
gl_call!(gl::BindTexture(gl::TEXTURE_CUBE_MAP, cubemap_texture));
            gl_call!(gl::TexParameteri(gl::TEXTURE_CUBE_MAP, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32));
            gl_call!(gl::TexParameteri(gl::TEXTURE_CUBE_MAP, gl::TEXTURE_MIN_FILTER, gl::LINEAR as i32));
            // These are very important to prevent seams
            gl_call!(gl::TexParameteri(gl::TEXTURE_CUBE_MAP, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_EDGE as i32));
            gl_call!(gl::TexParameteri(gl::TEXTURE_CUBE_MAP, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_EDGE as i32));
            gl_call!(gl::TexParameteri(gl::TEXTURE_CUBE_MAP, gl::TEXTURE_WRAP_R, gl::CLAMP_TO_EDGE as i32));

            for i in 0..FACES_CUBEMAP.len() {
                let img = image::open(FACES_CUBEMAP[i]).unwrap();
                let (img_width, img_height) = img.dimensions();
                let rgba = img.to_rgb8();
                let raw = rgba.as_raw();

                gl_call!(gl::TexImage2D(
                    gl::TEXTURE_CUBE_MAP_POSITIVE_X + i as u32, 
                    0, 
                    gl::RGB as i32, 
img_width as i32, 
                    img_height as i32, 
                    0, 
                    gl::RGB, 
                    gl::UNSIGNED_BYTE, 
                    raw.as_ptr().cast()
                ));
            }
        }


        // =============================================================
        // Setup vertex data
        // =============================================================
        unsafe {
            gl_call!(gl::GenVertexArrays(1, &mut vao));
            gl_call!(gl::GenBuffers(1, &mut vbo));

            vaos.insert(VaoType::CubeVao, vao);
            vbos.insert(VboType::CubeVbo, vbo);

            println!("vao is now: {}", vao);

            gl_call!(gl::BindVertexArray(vao));

            gl_call!(gl::BindBuffer(gl::ARRAY_BUFFER, vbo));

            gl_call!(gl::BufferData(
                gl::ARRAY_BUFFER, 
                (mem::size_of::<f32>() * UNIT_CUBE_VERTICES.len()) as isize, 
                UNIT_CUBE_VERTICES.as_ptr().cast(), 
                gl::STATIC_DRAW
            ));

            gl_call!(gl::VertexAttribPointer(
                0,
                3,
gl::FLOAT,
                gl::FALSE,
                8 * mem::size_of::<f32>() as i32,
                0 as *const _
            ));
            gl_call!(gl::EnableVertexAttribArray(0));

            gl_call!(gl::VertexAttribPointer(
                1, 
                2, 
                gl::FLOAT, 
                gl::FALSE, 
                8 * mem::size_of::<f32>() as i32,
                (3 * mem::size_of::<f32>()) as *const c_void
            ));

            gl_call!(gl::EnableVertexAttribArray(1));

            gl_call!(gl::VertexAttribPointer(
                2,
                3,
                gl::FLOAT,
                gl::FALSE,
                8 * mem::size_of::<f32>() as i32,
                (5 * mem::size_of::<f32>()) as *const c_void
            ));
            gl_call!(gl::EnableVertexAttribArray(2));
            // =============================================================
            // Load Textures and Set Texture Params 
            // =============================================================
            
            gl_call!(gl::GenTextures(1, &mut texture));
            gl_call!(gl::BindTexture(gl::TEXTURE_2D, texture));

            gl_call!(gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::REPEAT as i32));
            gl_call!(gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::REPEAT as i32));

            gl_call!(gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as i32));
            gl_call!(gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32));

            let img = image::open("resources/textures/container2.png").unwrap();
            let (img_width, img_height) = img.dimensions();
            let rgba = img.to_rgba8();
            let raw = rgba.as_raw();

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
                //raw.as_ptr() as *const c_void
            ));

gl_call!(gl::GenerateMipmap(gl::TEXTURE_2D));
        }

        Self {
            delta_time: 0.0,
            last_frame: 0.0,
            camera: Camera::new(),
            window_width: width as u32,
            window_height: height as u32,

            glfw,
            events,
            window,

            shaders,
            vaos, 
            vbos,
            ebos,

            texture,
            cubemap_texture,
        }
    }

    pub fn process_events(&mut self) {
        self.camera.process_key_event(&self.window, self.delta_time);

        for (_, event) in glfw::flush_messages(&self.events) {
            match event {
                glfw::WindowEvent::FramebufferSize(w, h) => {
                    self.window_width = w as u32;
                    self.window_height = h as u32;
                    unsafe {
                        gl::Viewport(0, 0, self.window_width as i32, self.window_height as i32);
                    }
                },
                glfw:: WindowEvent::Key(glfw::Key::Escape, _, glfw::Action::Press, _) => {
                    self.window.set_should_close(true);
                },
                _ => {
                    self.camera.process_mouse_input(&self.window, &event);
                },
            }
        }
    }

    pub fn update(&mut self) {
        let current_frame = self.glfw.get_time();
        self.delta_time = current_frame - self.last_frame;
        self.last_frame = current_frame;
    }

    pub fn render(&mut self) {
        self.camera.reset_matrices(self.window_width as f32 / self.window_height as f32);

        unsafe {
            gl_call!(gl::ClearColor(0.1, 0.2, 0.33, 1.0));
            gl_call!(gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT));

            let projection_c_string = CString::new("projection").unwrap();
            let view_c_string = CString::new("view").unwrap();
            let model_c_string = CString::new("model").unwrap();

            let main_shader_prog = *self.shaders.get(&ShaderType::MainShader).unwrap();
            let skybox_shader_prog = *self.shaders.get(&ShaderType::SkyboxShader).unwrap();

            // =============================================================
            // Skybox
            // =============================================================
            let view_no_translation = Mat4 {
                x_axis: self.camera.view.x_axis.clone(),
                y_axis: self.camera.view.y_axis.clone(),
                z_axis: self.camera.view.z_axis.clone(),
                w_axis: vec4(0.0, 0.0, 0.0, 1.0),
            };
            gl::DepthFunc(gl::LEQUAL);

            gl::UseProgram(skybox_shader_prog);

            gl_call!(gl::UniformMatrix4fv(
                gl::GetUniformLocation(skybox_shader_prog, view_c_string.as_ptr()),
                1,
                gl::FALSE,
                view_no_translation.to_cols_array().as_ptr(),
            ));

            gl_call!(gl::UniformMatrix4fv(
                gl_call!(gl::GetUniformLocation(skybox_shader_prog, projection_c_string.as_ptr())),
                1,
                gl::FALSE,
                self.camera.projection.to_cols_array().as_ptr(),
            ));
            gl::BindVertexArray(*self.vaos.get(&VaoType::SkyboxVao).unwrap());
            
            gl::ActiveTexture(gl::TEXTURE0);
            gl::BindTexture(gl::TEXTURE_CUBE_MAP, self.cubemap_texture);
            gl::DrawElements(gl::TRIANGLES, 36, gl::UNSIGNED_INT, 0 as *const _);
            gl::BindVertexArray(0);

            gl::DepthFunc(gl::LESS);
            // =============================================================
            // Draw cubes
            // =============================================================
            self.camera.reset_matrices(self.window_width as f32 / self.window_height as f32);

            gl_call!(gl::UseProgram(main_shader_prog));

            let texture_c_str = CString::new("u_texture").unwrap();
            gl_call!(gl::Uniform1i(gl::GetUniformLocation(main_shader_prog, texture_c_str.as_ptr()), 0));
            
            gl_call!(gl::ActiveTexture(gl::TEXTURE0));
            gl_call!(gl::BindTexture(gl::TEXTURE_2D, self.texture));

            gl_call!(gl::UniformMatrix4fv(
                gl_call!(gl::GetUniformLocation(main_shader_prog, projection_c_string.as_ptr())),
                1,
                gl::FALSE,
                self.camera.projection.to_cols_array().as_ptr(),
            ));

            gl_call!(gl::UniformMatrix4fv(
                gl::GetUniformLocation(main_shader_prog, view_c_string.as_ptr()),
                1,
                gl::FALSE,
                self.camera.view.to_cols_array().as_ptr(),
            ));

            gl_call!(gl::BindVertexArray(*self.vaos.get(&VaoType::CubeVao).unwrap()));
            self.camera.model = Mat4::IDENTITY;

             for i in 0..CUBE_POSITIONS.len() {
                 self.camera.model = Mat4::IDENTITY;
                 self.camera.model = Mat4::from_translation(CUBE_POSITIONS[i]);
                 
                 // rotate the cube
                 let angle = 20.0 * i as f32;
                 let axis = vec3(1.0, 0.3, 0.5).normalize();
                 self.camera.model *= Mat4::from_axis_angle(axis, angle);
                 
                 gl::UniformMatrix4fv(
                     gl::GetUniformLocation(main_shader_prog, model_c_string.as_ptr()),
                     1,
                     gl::FALSE,
                     self.camera.model.to_cols_array().as_ptr(),
                 );
                 gl::DrawArrays(gl::TRIANGLES, 0, 36);
             }

            self.window.swap_buffers();
            self.glfw.poll_events()
        }
    }
}
