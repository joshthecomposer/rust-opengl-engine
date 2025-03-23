#![allow(dead_code)]
use std::collections::HashSet;

use glam::{vec3, Quat, Vec3};
use glfw::{Context, Glfw, GlfwReceiver, PWindow, WindowEvent};
use image::GrayImage;
use rusttype::{point, Font, Scale};

use crate::{camera::Camera, config::{entity_config::{self, EntityConfig}, game_config::GameConfig}, entity_manager::EntityManager, enums_types::{CameraState, EntityType, Faction}, gl_call, grid::Grid, input::handle_keyboard_input, lights::{DirLight, Lights}, renderer::Renderer, sound::sound_manager::SoundManager, ui::imgui::ImguiManager};
// use rand::prelude::*;
// use rand_chacha::ChaCha8Rng;

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
    pub imgui_manager: ImguiManager,


    pub paused: bool,

    pub grid: Grid,
    pub renderer: Renderer,

    pub pressed_keys: HashSet<glfw::Key>,

    pub sound_manager: SoundManager,
}

impl GameState {
    pub fn new() -> Self {
        let mut glfw = glfw::init(glfw::fail_on_errors).expect("Failed to init glfw");

        glfw.window_hint(glfw::WindowHint::ContextVersion(3, 3)); // OpenGL 3.3
        glfw.window_hint(glfw::WindowHint::OpenGlProfile(glfw::OpenGlProfileHint::Core));
        glfw.window_hint(glfw::WindowHint::Resizable(true));
        #[cfg(target_os = "macos")]
        glfw.window_hint(glfw::WindowHint::OpenGlForwardCompat(true));

        let (width,height):(i32, i32) = (1920, 1080);

        let (mut window, events) = glfw
            .create_window(width as u32, height as u32, "Hello this is window", glfw::WindowMode::Windowed)
            .expect("Failed to create GLFW window.");
        window.set_key_polling(true);
        // window.set_sticky_keys(true); 
        window.set_cursor_mode(glfw::CursorMode::Disabled);
        window.set_all_polling(true);
        window.make_current();

        let (fb_width, fb_height) = window.get_framebuffer_size();

        println!("Framebuffer size: {}x{}", fb_width, fb_height);

        gl::load_with(|symbol| window.get_proc_address(symbol) as *const _);

        unsafe {
            gl_call!(gl::Enable(gl::BLEND));
            gl_call!(gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA));
            gl_call!(gl::Enable(gl::TEXTURE_CUBE_MAP_SEAMLESS));
            gl_call!(gl::Viewport(0, 0, width, height));
            gl_call!(gl::Enable(gl::DEPTH_TEST));
            gl::Enable(gl::DEBUG_OUTPUT);
            gl::Enable(gl::BLEND);
            gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
        }

        // =============================================================
        // Set up systems
        // =============================================================
        let mut light_manager = Lights::new(50);
        light_manager.dir_light = DirLight::default_white();

        let renderer = Renderer::new();

        let game_config = GameConfig::load_from_file("config/game_config.json");
        
        let sound_manager = SoundManager::new(&game_config);
    
        let mut entity_config = EntityConfig::load_from_file("config/entity_config.json");
        let mut entity_manager = EntityManager::new(10_000);
        entity_manager.populate_initial_entity_data(&mut entity_config);

        let mut grid = Grid::new(game_config.grid_width, game_config.grid_height, game_config.cell_size);
        grid.generate();

        let imgui_manager = ImguiManager::new(&mut window);

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
            imgui_manager,

            paused: false,

            grid,
            renderer,

            pressed_keys: HashSet::new(),
            sound_manager,
        }
    }

    pub fn process_events(&mut self) {
        self.camera.process_key_event(&self.window, self.delta_time);
        let events: Vec<(f64, glfw::WindowEvent)> = glfw::flush_messages(&self.events).collect();

        for (_, event) in events {
            self.imgui_manager.handle_imgui_event(&event);
            match event {
                glfw::WindowEvent::FramebufferSize(w, h) => {
                    self.window_width = w as u32;
                    self.window_height = h as u32;
                    unsafe {
                        gl::Viewport(0, 0, self.window_width as i32, self.window_height as i32);
                    }
                },
                glfw::WindowEvent::Key(key, _, action, _) => {
                    handle_keyboard_input(key, action, &mut self.pressed_keys);
                },
                _ => {
                    self.camera.process_mouse_input(&self.window, &event);
                },
            }
        }
    }

    pub fn update(&mut self) {
        // DELTA TIME 
        let current_frame = self.glfw.get_time();
        self.delta_time = current_frame - self.last_frame;
        self.last_frame = current_frame;
        self.elapsed += self.delta_time;

        // CHECK IF PAUSED OR SHOULD QUIT
        if self.paused { return; }
        if self.pressed_keys.contains(&glfw::Key::Escape) {
            self.window.set_should_close(true);
        }

        // UPDATE SYSTEMS
        self.sound_manager.update();
        self.entity_manager.update(&self.pressed_keys, self.delta_time, self.elapsed as f32, &self.camera);
        self.light_manager.update(&self.delta_time);
        self.camera.update(&self.entity_manager);
    }

    pub fn render(&mut self) {
        self.camera.reset_matrices(self.window_width as f32 / self.window_height as f32);
        self.renderer.draw(&self.entity_manager, &mut self.camera, &self.light_manager, &mut self.grid, self.fb_width, self.fb_height, &mut self.sound_manager);

        if self.camera.move_state == CameraState::Locked {
            self.window.set_cursor_mode(glfw::CursorMode::Normal);
            self.imgui_manager.draw(&mut self.window, self.fb_width as f32, self.fb_height as f32, self.delta_time, &mut self.light_manager, &mut self.renderer, &mut self.sound_manager);
        } else {
            self.window.set_cursor_mode(glfw::CursorMode::Disabled);
        }
        self.window.swap_buffers();
        self.glfw.poll_events()
    }
}
