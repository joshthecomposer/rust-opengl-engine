
use glam::{vec3, Vec3};
use glfw::{Action, Context, Glfw, GlfwReceiver, MouseButton, PWindow, WindowEvent};
use image::GrayImage;
use imgui::{Ui};
use rusttype::{point, Font, Scale};

use crate::{animation::ani_model::AniModel, camera::Camera, entity_manager::EntityManager, enums_types::{EntityType, ShaderType}, gl_call, grid::Grid, lights::{DirLight, Lights}, model::Model, renderer::Renderer};
use rand::prelude::*;
use rand_chacha::ChaCha8Rng;

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

    pub entity_manager: EntityManager,
    pub light_manager: Lights,
    pub imgui: imgui::Context,
    pub im_renderer: imgui_opengl_renderer::Renderer,
    pub paused: bool,

    pub grid: Grid,
    pub renderer: Renderer,
    pub glyph_tex: u32,
    pub tex_vao: u32
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

        println!("Begin testing load anim model");
        let test_model = AniModel::load("resources/models/animation/run_test.fbx");
        panic!("End Testing loading anim model");
        
        // =============================================================
        // text
        // ============================================================
        
        let font_data = include_bytes!("../resources/fonts/JetBrainsMonoNL-Regular.ttf");
        let font = Font::try_from_bytes(font_data).unwrap();

        let scale = Scale::uniform(256.0);
        let v_metrics = font.v_metrics(scale);
        let glyph = font.glyph('B').scaled(scale).positioned(point(0.0, v_metrics.ascent));

        let mut glyph_tex: u32 = 0;
        
        let mut glyph_width = 0.0;
        let mut glyph_height = 0.0;


        if let Some(bb) = glyph.pixel_bounding_box() {
            glyph_width = bb.width() as f32;
            glyph_height = bb.height() as f32;
            let width = bb.width() as usize;
            let height = bb.height() as usize;
            let mut pixel_data = vec![0u8; width * height];

            glyph.draw(|x, y, v| {
                let index = (y as usize * width) + x as usize;
                pixel_data[index] = (v * 255.0) as u8;
            });

            let img = GrayImage::from_vec(width as u32, height as u32, pixel_data.clone())
                .expect("Failed to create image");
            img.save("glyph_debug.png").expect("Failed to save image");

    println!("Saved glyph_debug.png ({}x{})", width, height);

            dbg!(width, height, &pixel_data[..10]); // Debug first few pixel values

            unsafe {
                gl_call!(gl::GenTextures(1, &mut glyph_tex));
                gl_call!(gl::BindTexture(gl::TEXTURE_2D, glyph_tex));

                gl_call!(gl::PixelStorei(gl::UNPACK_ALIGNMENT, 1));

                gl_call!(gl::TexImage2D(
                    gl::TEXTURE_2D,
                    0,
                    gl::RED as i32,
                    width as i32,
                    height as i32,
                    0,
                    gl::RED,
                    gl::UNSIGNED_BYTE,
                    pixel_data.as_ptr() as *const _,
                ));

                gl_call!(gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST as i32));
                gl_call!(gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as i32));
                 gl_call!(gl::PixelStorei(gl::UNPACK_ALIGNMENT, 4)); 
            }

        }

        let aspect_ratio = glyph_width / glyph_height;
        let pixel_scale_x = 2.0 / fb_width as f32;
        let pixel_scale_y = 2.0 / fb_height as f32;

        let width_ndc = glyph_width * pixel_scale_x;
        let height_ndc = glyph_height * pixel_scale_y;
        let scale = 1.0;
        let quad_vertices: [f32; 30] = [
            // Positions         // Flipped Texture Coords (Swap Y)
            -scale * aspect_ratio,  scale, 0.0,  0.0, 0.0,  // Top-left (was 1.0)
            -scale * aspect_ratio, -scale, 0.0,  0.0, 1.0,  // Bottom-left (was 0.0)
            scale * aspect_ratio, -scale, 0.0,  1.0, 1.0,  // Bottom-right

            -scale * aspect_ratio,  scale, 0.0,  0.0, 0.0,  // Top-left
            scale * aspect_ratio, -scale, 0.0,  1.0, 1.0,  // Bottom-right
            scale * aspect_ratio,  scale, 0.0,  1.0, 0.0   // Top-right
        ];

        let mut tex_vao = 0;
        let mut tex_vbo = 0;

        unsafe {
            gl_call!(gl::GenVertexArrays(1, &mut tex_vao));
            gl_call!(gl::GenBuffers(1, &mut tex_vbo));

            gl_call!(gl::BindVertexArray(tex_vao));

            gl_call!(gl::BindBuffer(gl::ARRAY_BUFFER, tex_vbo));
            gl_call!(gl::BufferData(
                gl::ARRAY_BUFFER,
                (quad_vertices.len() * std::mem::size_of::<f32>()) as isize,
                quad_vertices.as_ptr() as *const _,
                gl::STATIC_DRAW,
            ));

            // Position attribute
            gl_call!(gl::VertexAttribPointer(0, 3, gl::FLOAT, gl::FALSE, (5 * std::mem::size_of::<f32>()) as i32, std::ptr::null()));
            gl_call!(gl::EnableVertexAttribArray(0));

            // Texture coordinate attribute
            gl_call!(gl::VertexAttribPointer(1, 2, gl::FLOAT, gl::FALSE, (5 * std::mem::size_of::<f32>()) as i32, (3 * std::mem::size_of::<f32>()) as *const _));
            gl::EnableVertexAttribArray(1);

            gl_call!(gl::BindBuffer(gl::ARRAY_BUFFER, 0));
            gl_call!(gl::BindVertexArray(0));
        }

        let mut grid = Grid::parse_grid_data("resources/test_level.txt");

        unsafe {
            gl_call!(gl::Enable(gl::BLEND));
            gl_call!(gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA));
            gl_call!(gl::Enable(gl::TEXTURE_CUBE_MAP_SEAMLESS));
            gl_call!(gl::Viewport(0, 0, width, height));
            gl_call!(gl::Enable(gl::DEPTH_TEST));
            gl::Enable(gl::DEBUG_OUTPUT);
            // gl_call!(gl::Enable(gl::FRAMEBUFFER_SRGB)); 
            gl::Enable(gl::CULL_FACE);  
            gl::CullFace(gl::BACK);  
        }

        let mut entity_manager = EntityManager::new(10_000);

        entity_manager.populate_cell_rng(&grid);
        entity_manager.populate_floor_tiles(&grid, "resources/models/my_obj/tile_01.obj");
        // entity_manager.create_entity(EntityType::ArcherTower01, vec3(0.0, 0.0, 0.0), vec3(1.0, 1.0, 1.0), "resources/models/sponza/sponza.obj");
        entity_manager.create_entity(EntityType::ArcherTower01, vec3(0.0, 0.0, 0.0), Vec3::splat(0.1), "resources/models/my_obj/tower.obj");
        entity_manager.create_entity(EntityType::Donut, vec3(1.0, 1.0, 1.0), Vec3::splat(2.0), "resources/models/my_obj/donut.obj");


        let mut light_manager = Lights::new(50);
        light_manager.dir_light = DirLight::default_white();

        let renderer = Renderer::new();

        // =============================================================
        // imgui
        // =============================================================
        let mut imgui = imgui::Context::create();
        imgui.set_ini_filename(None);
        
        let im_renderer = imgui_opengl_renderer::Renderer::new(&mut imgui, |s| {
            window.get_proc_address(s) as *const _
        });

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

            entity_manager,
            light_manager,
            imgui,
            im_renderer,
            paused: false,

            grid,
            renderer,
            glyph_tex,
            tex_vao,
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

        // let radius = 0.5;
        // let speed = 1.0;
        // let angle = (self.elapsed * speed) as f32;

        // self.donut_pos.x = radius * angle.cos();
        // self.donut_pos.z = radius * angle.sin();
        // self.donut_pos.y = 1.0;

        // let donut2_r = 0.2;
        // let speed2 = 2.0;
        // let angle2 = (self.elapsed * speed2) as f32;

        // self.donut2_pos.x = self.donut_pos.x + donut2_r * angle2.cos();
        // self.donut2_pos.z = self.donut_pos.z + donut2_r * angle2.sin();
        // self.donut2_pos.y = 1.0; // Same height as Donut 1

        if self.paused { return; }
        self.entity_manager.update(&self.delta_time);
        self.light_manager.update(&self.delta_time);

        self.camera.update();
    }

    pub fn render(&mut self) {
        self.camera.reset_matrices(self.window_width as f32 / self.window_height as f32);
        self.renderer.draw(&self.entity_manager, &mut self.camera, &self.light_manager, &mut self.grid, self.fb_width, self.fb_height);

        let shader = self.renderer.shaders.get_mut(&ShaderType::Text).unwrap();

        shader.activate();
        unsafe {
            // Bind texture
            gl_call!(gl::ActiveTexture(gl::TEXTURE0));
            gl_call!(gl::BindTexture(gl::TEXTURE_2D, self.glyph_tex));

            // Bind VAO and draw quad
            gl_call!(gl::BindVertexArray(self.tex_vao));
            gl_call!(gl::DrawArrays(gl::TRIANGLES, 0, 6));

            // Cleanup
            gl_call!(gl::BindVertexArray(0));
            gl_call!(gl::UseProgram(0));
        }

        self.window.swap_buffers();
        self.glfw.poll_events()
    }
}
