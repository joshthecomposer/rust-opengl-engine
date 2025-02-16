use std::{collections::HashMap, ffi::c_void, mem, ptr::null_mut};

use glam::{vec3, vec4, Mat4, Vec3};
use glfw::{Action, Context, Glfw, GlfwReceiver, MouseButton, PWindow, WindowEvent};
use image::GenericImageView;
use imgui::{Ui};

use crate::{camera::Camera, entity_manager::EntityManager, enums_types::{FboType, ShaderType, VaoType}, gl_call, grid::Grid, lights::{DirLight, Lights}, model::Model, shaders::Shader, some_data::{BISEXUAL_BLUE, BISEXUAL_BLUE_SCALE, BISEXUAL_PINK, BISEXUAL_PINK_SCALE, BISEXUAL_PURPLE, BISEXUAL_PURPLE_SCALE, CUBE_POSITIONS, FACES_CUBEMAP, GROUND_PLANE, POINT_LIGHT_POSITIONS, SHADOW_HEIGHT, SHADOW_WIDTH, SKYBOX_INDICES, SKYBOX_VERTICES, UNIT_CUBE_VERTICES, WHITE}};

pub struct GameState {
    pub delta_time: f64,
    pub last_frame: f64,
    pub elapsed: f64,
    pub camera: Camera,
    pub window_width: u32,
    pub window_height: u32,
    pub fb_width: u32,
    pub fb_height: u32,

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
    pub model_pos: Vec3,
    pub donut: Model,
    pub donut_pos: Vec3,
    pub donut2: Model,
    pub donut2_pos: Vec3,

    pub grid: Grid,
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

        let (fb_width, fb_height) = window.get_framebuffer_size();

        println!("Framebuffer size: {}x{}", fb_width, fb_height);

        gl::load_with(|symbol| window.get_proc_address(symbol) as *const _);


        let mut grid = Grid::new(10, 10);
        grid.generate();

        unsafe {
            gl_call!(gl::Enable(gl::BLEND));
            gl_call!(gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA));
            gl_call!(gl::Enable(gl::TEXTURE_CUBE_MAP_SEAMLESS));
            gl_call!(gl::Viewport(0, 0, width, height));
            gl_call!(gl::Enable(gl::DEPTH_TEST));
            gl::Enable(gl::DEBUG_OUTPUT);
            // gl_call!(gl::Enable(gl::FRAMEBUFFER_SRGB)); 
            // gl::Enable(gl::CULL_FACE);  
            // gl::CullFace(gl::BACK);  
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

        let mut model_shader = Shader::new("resources/shaders/model.vs", "resources/shaders/model.fs");
        model_shader.store_uniform_location("projection");
        model_shader.store_uniform_location("view");
        model_shader.store_uniform_location("model");
        model_shader.store_dir_light_location("dir_light");
        model_shader.store_uniform_location("light_space_mat");
        model_shader.store_uniform_location("shadow_map");

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
        // Model
        // =============================================================
       // let model = Model::load("resources/models/backpack/backpack.obj");
        let model = Model::load("resources/models/my_obj/tower.obj");
        let donut = Model::load("resources/models/my_obj/donut.obj");
        let donut2 = Model::load("resources/models/my_obj/donut.obj");
        let model_pos = vec3(0.0, 0.0, 0.0);
        let donut_pos = vec3(5.0, 5.0, -10.0);
        let donut2_pos = vec3(5.0, 5.0, -10.0);
        // for mesh in model.meshes.iter() {
        //     for vertex in mesh.vertices.iter() {
        //         dbg!(vertex.normal);
        //     }
        // }

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
                [1.0, 1.0, 1.0, 1.0].as_ptr().cast() 
            ));

            gl_call!(gl::BindFramebuffer(gl::FRAMEBUFFER, fbo));
            gl_call!(gl::FramebufferTexture2D(gl::FRAMEBUFFER, gl::DEPTH_ATTACHMENT, gl::TEXTURE_2D, depth_map, 0));
            gl_call!(gl::DrawBuffer(gl::NONE));
            gl_call!(gl::ReadBuffer(gl::NONE));
            gl_call!(gl::BindFramebuffer(gl::FRAMEBUFFER, 0));
        }

        let mut debug_depth_quad = Shader::new("resources/shaders/debug_depth_quad.vs","resources/shaders/debug_depth_quad.fs");

        debug_depth_quad.activate();
        debug_depth_quad.store_uniform_location("depth_map");
        debug_depth_quad.set_int("depth_map", 0);

        shaders.insert(ShaderType::Model, model_shader);
        shaders.insert(ShaderType::Skybox, skybox_shader);
        shaders.insert(ShaderType::DebugLight, debug_light_shader);
        shaders.insert(ShaderType::Depth, depth_shader);
        shaders.insert(ShaderType::DebugShadowMap, debug_depth_quad);

        let entity_manager = EntityManager::new(10_000);
        let mut light_manager = Lights::new(50);
        light_manager.dir_light = DirLight::default_white();

        Self {
            delta_time: 0.0,
            last_frame: 0.0,
            elapsed: 0.0,
            camera: Camera::new(),
            window_width: width as u32,
            window_height: height as u32,
            fb_width:  fb_width as u32,
            fb_height: fb_height as u32,

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
            model_pos,
            donut,
            donut_pos,
            donut2,
            donut2_pos,

            grid,
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

        self.elapsed += self.delta_time;

        let radius = 0.5;
        let speed = 1.0;
        let angle = (self.elapsed * speed) as f32;

        self.donut_pos.x = radius * angle.cos();
        self.donut_pos.z = radius * angle.sin();
        self.donut_pos.y = 1.0;

        let donut2_r = 0.2;
        let speed2 = 2.0;
        let angle2 = (self.elapsed * speed2) as f32;

        self.donut2_pos.x = self.donut_pos.x + donut2_r * angle2.cos();
        self.donut2_pos.z = self.donut_pos.z + donut2_r * angle2.sin();
        self.donut2_pos.y = 1.0; // Same height as Donut 1

        if self.paused { return; }
        self.entity_manager.update(&self.delta_time);
        self.light_manager.update(&self.delta_time);

        self.camera.update();
    }

    pub fn render(&mut self) {
        self.camera.reset_matrices(self.window_width as f32 / self.window_height as f32);
        unsafe {


            // =============================================================
            // Render scene from light's perspective
            // =============================================================
            let depth_shader_prog = self.shaders.get(&ShaderType::Depth).unwrap();

            self.camera.reset_matrices(self.window_width as f32 / self.window_height as f32);
            let near_plane = 1.0;
            let far_plane = 10.0;
            let light_projection = Mat4::orthographic_rh_gl(-10.0, 10.0, -10.0, 10.0, near_plane, far_plane);
            let light_view = Mat4::look_at_rh(self.light_manager.dir_light.view_pos, vec3(0.0, 0.0, 0.0), vec3(0.0, 1.0, 0.0));
            self.camera.light_space = light_projection * light_view;
            depth_shader_prog.activate();
            depth_shader_prog.set_mat4("light_space_mat", self.camera.light_space);

            gl_call!(gl::Viewport(0, 0, SHADOW_WIDTH, SHADOW_HEIGHT));
            gl_call!(gl::BindFramebuffer(gl::FRAMEBUFFER, *self.fbos.get(&FboType::DepthMap).unwrap()));
            gl_call!(gl::Clear(gl::DEPTH_BUFFER_BIT));
            // Render scene
            self.render_sample_depth();
            gl_call!(gl::BindFramebuffer(gl::FRAMEBUFFER,0));
            gl_call!(gl::Viewport(0, 0, self.fb_width as i32, self.fb_height as i32));

            let do_debug = false;
            if do_debug {
                // =============================================================
                // Render debug depth quad
                // =============================================================
                gl_call!(gl::ClearColor(0.0, 0.0, 0.0, 1.0));
                gl_call!(gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT));
                let depth_debug_quad = self.shaders.get(&ShaderType::DebugShadowMap).unwrap();
                depth_debug_quad.activate();
                gl_call!(gl::ActiveTexture(gl::TEXTURE0));
                gl_call!(gl::BindTexture(gl::TEXTURE_2D, self.depth_map));
                self.render_quad();
            } else {
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

                self.camera.reset_matrices(self.window_width as f32 / self.window_height as f32);
            }

            self.window.swap_buffers();
            self.glfw.poll_events()
        }
    }

    pub fn render_quad(&self) {
        let mut vao = 0;
        let mut vbo = 0;

        let quad_vertices: [f32; 30] = [
            // Positions      // Texture Coords
            -1.0,  1.0, 0.0,  0.0, 1.0,
            -1.0, -1.0, 0.0,  0.0, 0.0,
            1.0, -1.0, 0.0,  1.0, 0.0,

            -1.0,  1.0, 0.0,  0.0, 1.0,
            1.0, -1.0, 0.0,  1.0, 0.0,
            1.0,  1.0, 0.0,  1.0, 1.0
        ];
        // let quad_vertices: [f32; 30] = [
        //     // Positions      // Texture Coords (flip Y)
        //     -1.0,  1.0, 0.0,  0.0, 0.0,  // Change (0,1) -> (0,0)
        //     -1.0, -1.0, 0.0,  0.0, 1.0,  // Change (0,0) -> (0,1)
        //     1.0, -1.0, 0.0,  1.0, 1.0,  // Change (1,0) -> (1,1)

        //     -1.0,  1.0, 0.0,  0.0, 0.0,
        //     1.0, -1.0, 0.0,  1.0, 1.0,
        //     1.0,  1.0, 0.0,  1.0, 0.0   // Change (1,1) -> (1,0)
        // ];

        unsafe {
            gl_call!(gl::GenVertexArrays(1, &mut vao));
            gl_call!(gl::GenBuffers(1, &mut vbo));
            gl_call!(gl::BindVertexArray(vao));

            gl_call!(gl::BindBuffer(gl::ARRAY_BUFFER, vbo));
            gl_call!(gl::BufferData(
                gl::ARRAY_BUFFER,
                (quad_vertices.len() * std::mem::size_of::<f32>()) as isize,
                quad_vertices.as_ptr() as *const _,
                gl::STATIC_DRAW
            ));

            let stride = (5 * std::mem::size_of::<f32>()) as i32;

            // Position Attribute
            gl_call!(gl::EnableVertexAttribArray(0));
            gl_call!(gl::VertexAttribPointer(0, 3, gl::FLOAT, gl::FALSE, stride, std::ptr::null()));

            // Texture Coordinate Attribute
            gl_call!(gl::EnableVertexAttribArray(1));
            gl_call!(gl::VertexAttribPointer(1, 2, gl::FLOAT, gl::FALSE, stride, (3 * std::mem::size_of::<f32>()) as *const _));

            gl_call!(gl::BindBuffer(gl::ARRAY_BUFFER, 0));
            gl_call!(gl::BindVertexArray(0));
        }

        // Draw the quad
        unsafe {
            gl_call!(gl::BindVertexArray(vao));
            gl_call!(gl::DrawArrays(gl::TRIANGLES, 0, 6));
            gl_call!(gl::BindVertexArray(0));
        }
    }

    pub fn render_sample_depth(&mut self) {
        let depth_shader = self.shaders.get(&ShaderType::Depth).unwrap();
        depth_shader.activate();
        // =========================
        // Render Model For Shadows
        // =========================
        let model_model = Mat4::IDENTITY * Mat4::from_translation(self.model_pos) * Mat4::from_scale(Vec3::splat(0.2));
        for mesh in self.model.meshes.iter() {
            unsafe {
                gl::BindVertexArray(mesh.vao);
            }
            depth_shader.set_mat4("model", model_model);
            unsafe {
                gl_call!(gl::DrawElements(
                    gl::TRIANGLES, 
                    mesh.indices.len() as i32, 
                    gl::UNSIGNED_INT, 
                    0 as *const _
                ));

                gl_call!(gl::BindVertexArray(0));
            }
        }
        let model_donut = Mat4::IDENTITY * Mat4::from_translation(self.donut_pos) * Mat4::from_scale(Vec3::splat(1.0));
        for mesh in self.donut.meshes.iter() {
            unsafe {
                gl::BindVertexArray(mesh.vao);
            }
            depth_shader.set_mat4("model", model_donut);
            unsafe {
                gl_call!(gl::DrawElements(
                    gl::TRIANGLES, 
                    mesh.indices.len() as i32, 
                    gl::UNSIGNED_INT, 
                    0 as *const _
                ));

                gl_call!(gl::BindVertexArray(0));
            }
        }
        let model_donut2 = Mat4::IDENTITY * Mat4::from_translation(self.donut2_pos) * Mat4::from_scale(Vec3::splat(0.35));
        for mesh in self.donut2.meshes.iter() {
            unsafe {
                gl::BindVertexArray(mesh.vao);
            }
            depth_shader.set_mat4("model", model_donut2);
            unsafe {
                gl_call!(gl::DrawElements(
                    gl::TRIANGLES, 
                    mesh.indices.len() as i32, 
                    gl::UNSIGNED_INT, 
                    0 as *const _
                ));

                gl_call!(gl::BindVertexArray(0));
            }
        }

        unsafe { gl_call!(gl::BindVertexArray(0)); }
    }

    pub fn render_sample(&mut self) {
        // =============================================================
        // Render Model
        // =============================================================
        self.camera.model = Mat4::IDENTITY * Mat4::from_translation(self.model_pos) * Mat4::from_scale(Vec3::splat(0.2));
        let model_shader = self.shaders.get_mut(&ShaderType::Model).unwrap();
        model_shader.activate();
        model_shader.set_mat4("model", self.camera.model);
        model_shader.set_mat4("view", self.camera.view);
        model_shader.set_mat4("projection", self.camera.projection);
        model_shader.set_mat4("light_space_mat", self.camera.light_space);
        model_shader.set_dir_light("dir_light", &self.light_manager.dir_light);

        unsafe {
            // TODO: This could clash, we need to make sure we reserve texture0 in our dynamic shader code.
            gl_call!(gl::ActiveTexture(gl::TEXTURE2));
            gl_call!(gl::BindTexture(gl::TEXTURE_2D, self.depth_map));
            model_shader.set_int("shadow_map", 2);
        }

        self.model.draw(model_shader);

        self.camera.model = Mat4::IDENTITY  * Mat4::from_translation(self.donut_pos) * Mat4::from_scale(Vec3::splat(1.0));
        model_shader.set_mat4("model", self.camera.model);
        self.donut.draw(model_shader);

        self.camera.model = Mat4::IDENTITY  * Mat4::from_translation(self.donut2_pos) * Mat4::from_scale(Vec3::splat(0.35));
        model_shader.set_mat4("model", self.camera.model);
        self.donut2.draw(model_shader);


        // =============================================================
        // Render Grid
        // =============================================================
        self.camera.model = Mat4::IDENTITY * Mat4::from_translation(self.model_pos);
        let model_shader = self.shaders.get_mut(&ShaderType::Model).unwrap();
        model_shader.activate();
        model_shader.set_mat4("model", self.camera.model);
        model_shader.set_mat4("view", self.camera.view);
        model_shader.set_mat4("projection", self.camera.projection);
        model_shader.set_mat4("light_space_mat", self.camera.light_space);
        model_shader.set_dir_light("dir_light", &self.light_manager.dir_light);
        
        self.grid.draw(model_shader);
    }
}
