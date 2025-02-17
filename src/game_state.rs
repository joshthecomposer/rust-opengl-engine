
use glam::{vec3, Vec3};
use glfw::{Action, Context, Glfw, GlfwReceiver, MouseButton, PWindow, WindowEvent};
use imgui::{Ui};

use crate::{camera::Camera, entity_manager::EntityManager, enums_types::EntityType, gl_call, grid::Grid, lights::{DirLight, Lights}, renderer::Renderer};

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

        let mut entity_manager = EntityManager::new(10_000);
        entity_manager.populate_floor_tiles(&grid, "resources/models/my_obj/ground_01.obj");
        entity_manager.create_entity(EntityType::ArcherTower_01, vec3(0.0, 0.0, 0.0), vec3(0.2, 0.13, 0.2), "resources/models/my_obj/tower.obj");
        entity_manager.create_entity(EntityType::Donut, vec3(1.0, 1.0, 1.0), Vec3::splat(2.0), "resources/models/my_obj/donut.obj");
        

//        entity_manager.create_entity(EntityType::Tree, grid.cells.get(5).unwrap().position, Vec3::splat(1.0), "resources/models/obj/tree_default.obj");
//        entity_manager.create_entity(EntityType::Tree, grid.cells.get(15).unwrap().position, Vec3::splat(1.0), "resources/models/obj/tree_oak.obj");
//        entity_manager.create_entity(EntityType::Tree, grid.cells.get(7).unwrap().position, Vec3::splat(1.0), "resources/models/obj/tree_oak_dark.obj");
//
        for i in 16..=25 {
            entity_manager.create_entity(EntityType::Tree, grid.cells.get(i).unwrap().position, Vec3::splat(0.01), "resources/models/my_obj/grass_07.fbx");
        }


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

        self.window.swap_buffers();
        self.glfw.poll_events()
    }
}
