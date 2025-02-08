use std::{collections::HashMap, ffi::c_void, mem, ptr::null_mut};

use glam::{vec3, vec4, Mat4};
use glfw::{Action, Context, Glfw, GlfwReceiver, MouseButton, PWindow, WindowEvent};
use image::GenericImageView;
use imgui::{Ui};

use crate::{camera::Camera, entity_manager::EntityManager, enums_types::{FboType, ShaderType, VaoType}, gl_call, lights::{DirLight, Lights}, model::Model, shaders::Shader, some_data::{BISEXUAL_BLUE, BISEXUAL_BLUE_SCALE, BISEXUAL_PINK, BISEXUAL_PINK_SCALE, BISEXUAL_PURPLE, BISEXUAL_PURPLE_SCALE, CUBE_POSITIONS, FACES_CUBEMAP, GROUND_PLANE, POINT_LIGHT_POSITIONS, SHADOW_HEIGHT, SHADOW_WIDTH, SKYBOX_INDICES, SKYBOX_VERTICES, UNIT_CUBE_VERTICES, WHITE}};

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

    pub shaders: HashMap<ShaderType, Shader>, // TODO: make this an enum
    pub vaos: HashMap<VaoType, u32>,
    pub fbos: HashMap<FboType, u32>,

    pub container_diffuse: u32,
    pub container_specular: u32,
    pub cubemap_texture: u32,
    pub depth_map: u32,

    pub entity_manager: EntityManager,
    pub light_manager: Lights,
    pub imgui: imgui::Context,
    pub renderer: imgui_opengl_renderer::Renderer,
    pub paused: bool,

    pub model: Model,
}

impl GameState {
    pub fn new() -> Self {
        let mut glfw = glfw::init(glfw::fail_on_errors).expect("Failed to init glfw");

        glfw.window_hint(glfw::WindowHint::ContextVersion(3, 3)); // OpenGL 3.3
        glfw.window_hint(glfw::WindowHint::OpenGlProfile(glfw::OpenGlProfileHint::Core));
        glfw.window_hint(glfw::WindowHint::Resizable(true));
        #[cfg(target_os = "macos")]
        glfw.window_hint(glfw::WindowHint::OpenGlForwardCompat(true));

        let (mut width,mut height):(i32, i32) = (1920, 1080);

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
            gl_call!(gl::Viewport(0, 0, width, height));
            gl_call!(gl::Enable(gl::DEPTH_TEST));
            gl::Enable(gl::DEBUG_OUTPUT);
        }

        // =============================================================
        // imgui
        // =============================================================
        let mut imgui = imgui::Context::create();
        imgui.set_ini_filename(None);
        
        let renderer = imgui_opengl_renderer::Renderer::new(&mut imgui, |s| {
            window.get_proc_address(s) as *const _
        });

        let mut shaders = HashMap::new();
        let mut vaos = HashMap::new();
        let mut fbos = HashMap::new();

        let mut main_shader = Shader::new("resources/shaders/shader.vs", "resources/shaders/shader.fs");
        main_shader.store_uniform_location("model");
        main_shader.store_uniform_location("view");
        main_shader.store_uniform_location("projection");
        main_shader.store_uniform_location("point_light_color");
        main_shader.store_uniform_location("normal_color");
        main_shader.store_uniform_location("material.diffuse");
        main_shader.store_uniform_location("material.specular");
        main_shader.store_uniform_location("material.shininess");
        for i in 0..4 {
            main_shader.store_uniform_location(format!("point_lights[{}].position", i).as_str());
            main_shader.store_uniform_location(format!("point_lights[{}].ambient", i).as_str());
            main_shader.store_uniform_location(format!("point_lights[{}].diffuse",i).as_str());
            main_shader.store_uniform_location(format!("point_lights[{}].specular",i).as_str());
            main_shader.store_uniform_location(format!("point_lights[{}].constant",i).as_str());
            main_shader.store_uniform_location(format!("point_lights[{}].linear",i).as_str());
            main_shader.store_uniform_location(format!("point_lights[{}].quadratic",i).as_str());
        }
        main_shader.store_uniform_location("dir_light.direction");
        main_shader.store_uniform_location("dir_light.ambient");
        main_shader.store_uniform_location("dir_light.diffuse");
        main_shader.store_uniform_location("dir_light.specular");
        main_shader.store_uniform_location("ViewPosition");
        main_shader.store_uniform_location("shadow_map");

        let mut skybox_shader = Shader::new("resources/shaders/skybox.vs", "resources/shaders/skybox.fs");

        skybox_shader.store_uniform_location("view");
        skybox_shader.store_uniform_location("projection");
        skybox_shader.store_uniform_location("skybox");
        
        let mut debug_light_shader = Shader::new("resources/shaders/point_light.vs", "resources/shaders/point_light.fs");
        debug_light_shader.store_uniform_location("model");
        debug_light_shader.store_uniform_location("view");
        debug_light_shader.store_uniform_location("projection");
        debug_light_shader.store_uniform_location("LightColor");

        let mut depth_shader = Shader::new("resources/shaders/depth_shader.vs","resources/shaders/depth_shader.fs");
        depth_shader.store_uniform_location("light_space_mat");
        depth_shader.store_uniform_location("model");
        
        let mut ground_plane_shader = Shader::new("resources/shaders/ground_plane.vs", "resources/shaders/ground_plane.fs");

        ground_plane_shader.store_uniform_location("model");
        ground_plane_shader.store_uniform_location("view");
        ground_plane_shader.store_uniform_location("projection");
        ground_plane_shader.store_uniform_location("point_light_color");
        ground_plane_shader.store_uniform_location("shadow_map");
        ground_plane_shader.store_uniform_location("dir_light.direction");
        ground_plane_shader.store_uniform_location("dir_light.view_pos");
        ground_plane_shader.store_uniform_location("dir_light.ambient");
        ground_plane_shader.store_uniform_location("dir_light.diffuse");
        ground_plane_shader.store_uniform_location("dir_light.specular");
        ground_plane_shader.store_uniform_location("ViewPosition");
        ground_plane_shader.store_uniform_location("ground_color");
        ground_plane_shader.store_uniform_location("light_space_mat");

        let mut model_test_shader = Shader::new("resources/shaders/model_test.vs", "resources/shaders/model_test.fs");

        let mut vao = 0;
        let mut vbo = 0;
        let mut ebo = 0;
        let mut container_diffuse = 0;
        let mut container_specular = 0;
        let mut cubemap_texture = 0;
        // =============================================================
        // Skybox memes
        // =============================================================
        unsafe {
            skybox_shader.activate();
            gl_call!(gl::GenVertexArrays(1, &mut vao));
            gl_call!(gl::GenBuffers(1, &mut vbo));
            gl_call!(gl::GenBuffers(1, &mut ebo));

            vaos.insert(VaoType::Skybox, vao);

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
                let img = match image::open(FACES_CUBEMAP[i]) {
                    Ok(img) => img,
                    _=> panic!("Error opening {}", FACES_CUBEMAP[i]),
                };
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
            main_shader.activate();
            gl_call!(gl::GenVertexArrays(1, &mut vao));
            gl_call!(gl::GenBuffers(1, &mut vbo));

            vaos.insert(VaoType::Cube, vao);

            println!("vao is now: {}", vao);

            gl_call!(gl::BindVertexArray(vao));

            gl_call!(gl::BindBuffer(gl::ARRAY_BUFFER, vbo));

            gl_call!(gl::BufferData(
                gl::ARRAY_BUFFER, 
                (mem::size_of::<f32>() * UNIT_CUBE_VERTICES.len()) as isize, 
                UNIT_CUBE_VERTICES.as_ptr().cast(), 
                gl::STATIC_DRAW
            ));
            // Position 
            gl_call!(gl::VertexAttribPointer(
                0,
                3,
                gl::FLOAT,
                gl::FALSE,
                8 * mem::size_of::<f32>() as i32,
                0 as *const _
            ));
            gl_call!(gl::EnableVertexAttribArray(0));
            
            // Tex Coords
            gl_call!(gl::VertexAttribPointer(
                1, 
                2, 
                gl::FLOAT, 
                gl::FALSE, 
                8 * mem::size_of::<f32>() as i32,
                (3 * mem::size_of::<f32>()) as *const c_void
            ));

            gl_call!(gl::EnableVertexAttribArray(1));
            
            // Normal
            gl_call!(gl::VertexAttribPointer(
                2,
                3,
                gl::FLOAT,
                gl::FALSE,
                8 * mem::size_of::<f32>() as i32,
                (5 * mem::size_of::<f32>()) as *const c_void
            ));
            gl_call!(gl::EnableVertexAttribArray(2));
        }

        // =============================================================
        // Container Diffuse Texture
        // =============================================================
        unsafe {

            gl_call!(gl::GenTextures(1, &mut container_diffuse));
            gl_call!(gl::BindTexture(gl::TEXTURE_2D, container_diffuse));

            gl_call!(gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_EDGE as i32));
            gl_call!(gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_EDGE as i32));

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

            main_shader.activate();
            main_shader.set_int("material.diffuse", 0);
            gl_call!(gl::ActiveTexture(gl::TEXTURE0));
            gl_call!(gl::BindTexture(gl::TEXTURE_2D, container_diffuse));
        }

        // =============================================================
        // Container Specular
        // =============================================================
        unsafe {
            gl_call!(gl::GenTextures(1, &mut container_specular));
            gl_call!(gl::BindTexture(gl::TEXTURE_2D, container_specular));
            gl_call!(gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_EDGE as i32));
            gl_call!(gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_EDGE as i32));
            gl_call!(gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as i32));
            gl_call!(gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32));

            let img = image::open("resources/textures/container2_specular.png").unwrap();
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

            main_shader.activate();
            main_shader.set_int("material.specular", 1);
            gl_call!(gl::ActiveTexture(gl::TEXTURE0));
            gl_call!(gl::BindTexture(gl::TEXTURE_2D, container_specular));
        }

        // =============================================================
        // Debug point light setup
        // =============================================================
        unsafe {
            debug_light_shader.activate();

            gl_call!(gl::GenVertexArrays(1, &mut vao));
            gl_call!(gl::GenBuffers(1, &mut vbo));

            vaos.insert(VaoType::DebugLight, vao);

            gl_call!(gl::BindVertexArray(vao));

            gl_call!(gl::BindBuffer(gl::ARRAY_BUFFER, vbo));
            gl_call!(gl::BufferData(
                gl::ARRAY_BUFFER, 
                (mem::size_of::<f32>() * UNIT_CUBE_VERTICES.len()) as isize, 
                UNIT_CUBE_VERTICES.as_ptr().cast(), 
                gl::STATIC_DRAW
            ));

            // Position 
            gl_call!(gl::VertexAttribPointer(
                0,
                3,
                gl::FLOAT,
                gl::FALSE,
                8 * mem::size_of::<f32>() as i32,
                0 as *const _
            ));
            gl_call!(gl::EnableVertexAttribArray(0));
        
            // Normal
            gl_call!(gl::VertexAttribPointer(
                1,
                3,
                gl::FLOAT,
                gl::FALSE,
                8 * mem::size_of::<f32>() as i32,
                (5 * mem::size_of::<f32>()) as *const c_void
            ));
            gl_call!(gl::EnableVertexAttribArray(1));
        } 
        // =============================================================
        // Ground Vao
        // =============================================================
        unsafe {
            gl_call!(gl::GenVertexArrays(1, &mut vao));
            gl_call!(gl::GenBuffers(1, &mut vbo));
            gl_call!(gl::BindVertexArray(vao));
            gl_call!(gl::BindBuffer(gl::ARRAY_BUFFER, vbo));
            gl_call!(gl::BufferData(
                gl::ARRAY_BUFFER,
                (mem::size_of::<f32>() * GROUND_PLANE.len()) as isize, 
                GROUND_PLANE.as_ptr().cast(), 
                gl::STATIC_DRAW
            ));

            vaos.insert(VaoType::GroundPlane, vao);
            
            // Positions
            gl_call!(gl::VertexAttribPointer(
                0, 
                3, 
                gl::FLOAT, 
                gl::FALSE,
                6 * mem::size_of::<f32>() as i32,
                0 as *const _,
            ));
            gl_call!(gl::EnableVertexAttribArray(0));
            
            // Normals
            gl_call!(gl::VertexAttribPointer(
                1, 
                3, 
                gl::FLOAT, 
                gl::FALSE,
                6 * mem::size_of::<f32>() as i32,
                (3 * mem::size_of::<f32>()) as *const c_void
            ));
            gl_call!(gl::EnableVertexAttribArray(1));
            gl::BindVertexArray(0);
        }
        // =============================================================
        // Model
        // =============================================================
        let model = Model::load("resources/models/TD 3D Low Poly Pack/fbx/Full/_archer_tower_LVL_1.fbx");

        // =============================================================
        // Shadow Mapping
        // =============================================================
        // The general idea is that we need to create a depth map rendered 
        // from the perspective of the light source. In this case one 
        // directional light.
        // We can do this using a "framebuffer". We have been using a 
        // framebuffer all along, just the "default" one given to us.
        let mut fbo = 0;
        let mut depth_map = 0;
        unsafe {
            main_shader.activate();
            gl_call!(gl::GenFramebuffers(1, &mut fbo));

            fbos.insert(FboType::DepthMap, fbo);

            gl_call!(gl::GenTextures(1, &mut depth_map));
            gl_call!(gl::BindTexture(gl::TEXTURE_2D, depth_map));
            gl_call!(gl::TexImage2D(gl::TEXTURE_2D, 0, gl::DEPTH_COMPONENT as i32, SHADOW_WIDTH, SHADOW_HEIGHT, 0, gl::DEPTH_COMPONENT, gl::FLOAT, null_mut()));
            gl_call!(gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST as i32));
            gl_call!(gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as i32));
            gl_call!(gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_BORDER as i32));
            gl_call!(gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_BORDER as i32));
            gl_call!(gl::TexParameterfv(
                gl::TEXTURE_2D, 
                gl::TEXTURE_BORDER_COLOR, 
                [1.0, 1.0, 1.0].as_ptr().cast() 
            ));

            gl_call!(gl::BindFramebuffer(gl::FRAMEBUFFER, fbo));
            gl_call!(gl::FramebufferTexture2D(gl::FRAMEBUFFER, gl::DEPTH_ATTACHMENT, gl::TEXTURE_2D, depth_map, 0));
            gl_call!(gl::DrawBuffer(gl::NONE));
            gl_call!(gl::ReadBuffer(gl::NONE));
            gl_call!(gl::BindFramebuffer(gl::FRAMEBUFFER, 0));
        }

        shaders.insert(ShaderType::Main, main_shader);
        shaders.insert(ShaderType::Skybox, skybox_shader);
        shaders.insert(ShaderType::DebugLight, debug_light_shader);
        shaders.insert(ShaderType::Depth, depth_shader);
        shaders.insert(ShaderType::GroundPlane, ground_plane_shader);

        let entity_manager = EntityManager::new(10_000);
        let mut light_manager = Lights::new(50);
        light_manager.dir_light = DirLight::default_white();
        

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
            fbos,

            container_diffuse,
            container_specular,
            cubemap_texture,
            depth_map,

            entity_manager,
            light_manager,
            imgui,
            renderer,
            paused: false,
            model,
        }
    }

    pub fn process_events(&mut self) {
        self.camera.process_key_event(&self.window, self.delta_time);
        let events: Vec<(f64, glfw::WindowEvent)> = glfw::flush_messages(&self.events).collect();

        for (_, event) in events {
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
                    // self.paused = !self.paused;

                },
                _ => {
                    self.camera.process_mouse_input(&self.window, &event);
                },
            }
        }
    }

    fn handle_imgui_event(&mut self, event: &WindowEvent) {
        let io = self.imgui.io_mut();
        match *event {
            // Mouse Buttons
            WindowEvent::MouseButton(btn, action, _) => {
                let pressed = action != Action::Release;
                match btn {
                    MouseButton::Button1 => io.mouse_down[0] = pressed,
                    MouseButton::Button2 => io.mouse_down[1] = pressed,
                    MouseButton::Button3 => io.mouse_down[2] = pressed,
                    _ => {}
                }
            }
            // Mouse Position
            WindowEvent::CursorPos(x, y) => {
                io.mouse_pos = [x as f32, y as f32];
            }
            // Scroll Wheel
            WindowEvent::Scroll(_x, scroll_y) => {
                io.mouse_wheel = scroll_y as f32;
            }
            // Text input
            WindowEvent::Char(ch) => {
                io.add_input_character(ch);
            }
            // Key press/release
            WindowEvent::Key(key, _, action, mods) => {
                let pressed = action != Action::Release;
                // If you want to track ImGui’s internal key map, do something like:
                // io.keys_down[imgui_key_index] = pressed;
                // or handle advanced shortcuts, etc.
            }

            _ => {}
        }
    }

    pub fn update(&mut self) {
        let current_frame = self.glfw.get_time();
        self.delta_time = current_frame - self.last_frame;
        self.last_frame = current_frame;


        if self.paused { return; }
        self.entity_manager.update(&self.delta_time);
        self.light_manager.update(&self.delta_time);

        self.camera.update();

        // self.camera.position = self.light_manager.dir_light.view_pos;
    }

    pub fn render(&mut self) {
        self.camera.reset_matrices(self.window_width as f32 / self.window_height as f32);
        unsafe {


            // =============================================================
            // Render scene from light's perspective
            // =============================================================
            let depth_shader_prog = self.shaders.get(&ShaderType::Depth).unwrap();

            self.camera.reset_matrices(self.window_width as f32 / self.window_height as f32);
            let near_plane = 0.01;
            let far_plane = 100.0;
            // Ortho works fine for only directional lights, but probably not for point apparently.
            // correct let mut light_projection = Mat4::orthographic_rh_gl(-20.0, 20.0, -20.0, 20.0, near_plane, far_plane);
            let mut light_projection = Mat4::orthographic_rh_gl(20.0, -20.0, 20.0, -20.0, near_plane, far_plane);

            // self.light_manager.dir_light.view_pos = vec3(5.0, 10.0, 5.0);
            let mut light_view = Mat4::look_at_rh(self.light_manager.dir_light.view_pos, vec3(0.0, 0.0, 0.0), vec3(0.0, 1.0, 0.0));
            self.camera.light_space = light_projection * light_view;
            depth_shader_prog.activate();
            depth_shader_prog.set_mat4("light_space_mat", self.camera.light_space);

            gl_call!(gl::Viewport(0, 0, SHADOW_WIDTH, SHADOW_HEIGHT));
            gl_call!(gl::BindFramebuffer(gl::FRAMEBUFFER, *self.fbos.get(&FboType::DepthMap).unwrap()));
            gl_call!(gl::Clear(gl::DEPTH_BUFFER_BIT));
            // Render scene

            let status = gl::CheckFramebufferStatus(gl::FRAMEBUFFER);
            if status != gl::FRAMEBUFFER_COMPLETE {
                println!("Framebuffer incomplete: {}", status);
            }
            self.render_sample_depth();

            gl_call!(gl::BindFramebuffer(gl::FRAMEBUFFER,0));
            gl_call!(gl::Viewport(0, 0, self.window_width as i32, self.window_height as i32));

            // =============================================================
            // Skybox
            // =============================================================
            self.camera.reset_matrices(self.window_width as f32 / self.window_height as f32);

            let status = gl::CheckFramebufferStatus(gl::FRAMEBUFFER);
            if status != gl::FRAMEBUFFER_COMPLETE {
                println!("Framebuffer incomplete: {}", status);
            }
            let skybox_shader_prog = self.shaders.get(&ShaderType::Skybox).unwrap();

            gl_call!(gl::ClearColor(0.14, 0.13, 0.15, 1.0));
            gl_call!(gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT));

            let view_no_translation = Mat4 {
                x_axis: self.camera.view.x_axis.clone(),
                y_axis: self.camera.view.y_axis.clone(),
                z_axis: self.camera.view.z_axis.clone(),
                w_axis: vec4(0.0, 0.0, 0.0, 1.0),
            };
            gl_call!(gl::DepthFunc(gl::LEQUAL));
        
            skybox_shader_prog.activate();
            skybox_shader_prog.set_mat4("view", view_no_translation);
            skybox_shader_prog.set_mat4("projection", self.camera.projection);

            gl_call!(gl::BindVertexArray(*self.vaos.get(&VaoType::Skybox).unwrap()));
            gl_call!(gl::ActiveTexture(gl::TEXTURE0));
            gl_call!(gl::BindTexture(gl::TEXTURE_CUBE_MAP, self.cubemap_texture));
            gl_call!(gl::DrawElements(gl::TRIANGLES, 36, gl::UNSIGNED_INT, 0 as *const _));
            gl_call!(gl::BindVertexArray(0));

            gl_call!(gl::DepthFunc(gl::LESS));

            // =============================================================
            // Draw Debug Lights
            // =============================================================
            let debug_light_shader = self.shaders.get(&ShaderType::DebugLight).unwrap();
            debug_light_shader.activate();
            debug_light_shader.set_mat4("view", self.camera.view);
            debug_light_shader.set_mat4("projection", self.camera.projection);

            gl_call!(gl::BindVertexArray(*self.vaos.get(&VaoType::DebugLight).unwrap()));
            for i in 0..POINT_LIGHT_POSITIONS.len() {
                self.camera.model = Mat4::IDENTITY;
                self.camera.model *= Mat4::from_translation(POINT_LIGHT_POSITIONS[i]);
                self.camera.model *= Mat4::from_scale(vec3(0.2, 0.2, 0.2)); 

                debug_light_shader.set_mat4("model", self.camera.model);
                debug_light_shader.set_vec3("LightColor", vec3(1.0, 1.0, 1.0));

                gl_call!(gl::DrawArrays(gl::TRIANGLES, 0, 36));
            }

            gl_call!(gl::BindVertexArray(0));

            // =============================================================
            // Render scene normally
            // =============================================================

            let status = gl::CheckFramebufferStatus(gl::FRAMEBUFFER);
            if status != gl::FRAMEBUFFER_COMPLETE {
                println!("Framebuffer incomplete: {}", status);
            }
            self.render_sample();

            self.model.draw(self.shaders.get_mut(&ShaderType::Main).unwrap());

            self.window.swap_buffers();
            self.glfw.poll_events()
        }
    }

    pub fn render_sample_depth(&mut self) {
        let depth_shader = self.shaders.get(&ShaderType::Depth).unwrap();
        depth_shader.activate();

        // =========================
        // Render floor for shadows
        // =========================
        unsafe {
            gl::BindVertexArray(*self.vaos.get(&VaoType::GroundPlane).unwrap());
            // Because we only need positions for the depth pass,
            // we typically only set the `model` matrix (and in the shader
            // multiply by `light_space_mat`). Normals & materials aren’t needed.
        }
        let model_floor = Mat4::IDENTITY;
        depth_shader.set_mat4("model", model_floor);

        // draw the floor
        unsafe {
            gl::DrawArrays(gl::TRIANGLES, 0, 6);
        }
        // =========================
        // Render cubes for shadows
        // =========================
        unsafe {
            gl::BindVertexArray(*self.vaos.get(&VaoType::Cube).unwrap());
        }

        for (i, &cube_pos) in CUBE_POSITIONS.iter().enumerate() {
            let mut model_cube = Mat4::IDENTITY;
            model_cube = Mat4::from_translation(cube_pos);

            // rotate the cube
            let angle = 20.0 * i as f32;
            let axis  = vec3(1.0, 0.3, 0.5).normalize();
            model_cube *= Mat4::from_axis_angle(axis, angle);

            depth_shader.set_mat4("model", model_cube);

            unsafe { gl::DrawArrays(gl::TRIANGLES, 0, 36); }
        }

        unsafe { gl::BindVertexArray(0); }
    }

    pub fn render_sample(&mut self) {
        let floor_shader = self.shaders.get(&ShaderType::GroundPlane).unwrap();
        let main_shader = self.shaders.get(&ShaderType::Main).unwrap();

        // =============================================================
        // Render floor
        // =============================================================
        floor_shader.activate();
        unsafe {
            gl_call!(gl::ActiveTexture(gl::TEXTURE0));
            gl_call!(gl::BindTexture(gl::TEXTURE_2D, self.depth_map));
            floor_shader.set_int("shadow_map", 0);
        }

        self.camera.model = Mat4::IDENTITY;
        floor_shader.set_mat4("model", self.camera.model);
        floor_shader.set_vec3("ground_color", vec3(0.67, 0.67, 0.67));
        floor_shader.set_mat4("view", self.camera.view);
        floor_shader.set_mat4("projection", self.camera.projection);
        floor_shader.set_mat4("light_space_mat", self.camera.light_space);
        floor_shader.set_vec3("dir_light.direction", self.light_manager.dir_light.direction);
        floor_shader.set_vec3("dir_light.view_pos", self.light_manager.dir_light.view_pos);
        floor_shader.set_vec3("dir_light.ambient", self.light_manager.dir_light.ambient);
        floor_shader.set_vec3("dir_light.diffuse", self.light_manager.dir_light.diffuse);
        floor_shader.set_vec3("dir_light.specular", self.light_manager.dir_light.specular);

        unsafe {
            gl_call!(gl::BindVertexArray(*self.vaos.get(&VaoType::GroundPlane).unwrap()));
            gl_call!(gl::DrawArrays(gl::TRIANGLES, 0, 6));
        }
        // =============================================================
        // Render cubes
        // =============================================================
        unsafe {
            gl_call!(gl::ActiveTexture(gl::TEXTURE0));
            gl_call!(gl::BindTexture(gl::TEXTURE_2D, self.container_diffuse));

            gl_call!(gl::ActiveTexture(gl::TEXTURE1));
            gl_call!(gl::BindTexture(gl::TEXTURE_2D, self.container_specular));

            gl_call!(gl::ActiveTexture(gl::TEXTURE2));
            gl_call!(gl::BindTexture(gl::TEXTURE_2D, self.depth_map));
        }

        main_shader.activate();

        main_shader.set_int("shadow_map", 2);
        main_shader.set_int("material.diffuse", 0);
        main_shader.set_mat4("projection", self.camera.projection);
        main_shader.set_mat4("view", self.camera.view);
        main_shader.set_float("material.shininess", 64.0);

        for i in 0..4 {
            main_shader.set_vec3(format!("point_lights[{}].position",i).as_str(), POINT_LIGHT_POSITIONS[i]);
            main_shader.set_vec3(format!("point_lights[{}].ambient",i).as_str(), BISEXUAL_BLUE_SCALE);
            main_shader.set_vec3(format!("point_lights[{}].diffuse",i).as_str(), BISEXUAL_PURPLE_SCALE);
            main_shader.set_vec3(format!("point_lights[{}].specular",i).as_str(), BISEXUAL_PINK_SCALE);
            main_shader.set_float(format!("point_lights[{}].constant",i).as_str(), 1.0);
            main_shader.set_float(format!("point_lights[{}].linear",i).as_str(), 0.09);
            main_shader.set_float(format!("point_lights[{}].quadratic",i).as_str(), 0.0032);
        }

        main_shader.set_vec3("dir_light.direction", self.light_manager.dir_light.direction);
        main_shader.set_vec3("dir_light.view_pos", self.light_manager.dir_light.view_pos);
        main_shader.set_vec3("dir_light.ambient", self.light_manager.dir_light.ambient);
        main_shader.set_vec3("dir_light.diffuse", self.light_manager.dir_light.diffuse);
        main_shader.set_vec3("dir_light.specular", self.light_manager.dir_light.specular);

        unsafe {
            gl_call!(gl::BindVertexArray(*self.vaos.get(&VaoType::Cube).unwrap()));
        }
        self.camera.model = Mat4::IDENTITY;

        for i in 0..CUBE_POSITIONS.len() {
            self.camera.model = Mat4::IDENTITY;
            self.camera.model = Mat4::from_translation(CUBE_POSITIONS[i]);

            // rotate the cube
            let angle = 20.0 * i as f32;
            let axis = vec3(1.0, 0.3, 0.5).normalize();
            self.camera.model *= Mat4::from_axis_angle(axis, angle);

            main_shader.set_mat4("model", self.camera.model);

            unsafe { gl::DrawArrays(gl::TRIANGLES, 0, 36); }
        }


    }
}
