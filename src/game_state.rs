#![allow(dead_code)]
use std::collections::HashSet;

use gl::{AttachShader, PixelStoref};
use glam::{vec3, Quat, Vec2, Vec3};
use glfw::{Context, Glfw, GlfwReceiver, Key, PWindow, WindowEvent};
use image::GrayImage;
use rusttype::{point, Font, Scale};

use crate::{animation::animation_system, camera::Camera, collision_system, config::{entity_config::{self, EntityConfig}, game_config::GameConfig}, debug::gizmos::Cylinder, entity_manager::{self, EntityManager}, enums_types::{CameraState, EntityType, Faction, ShaderType, Transform}, gl_call, grid::Grid, input::{handle_keyboard_input, handle_mouse_input}, lights::{DirLight, Lights}, movement_system, renderer::Renderer, sound::{fmod::FMOD_Studio_System_Update, sound_manager::SoundManager}, state_machines, terrain::Terrain, ui::{font::{self, FontManager}, imgui::ImguiManager}};
// use rand::prelude::*;
// use rand_chacha::ChaCha8Rng;

pub struct GameState {
    pub delta_time: f32,
    pub last_frame: f32,
    pub elapsed: f32,
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

    pub terrain: Terrain,
    pub cursor_pos: Vec2,
    pub font_manager: FontManager,
    pub fps: u32,
    pub last_fps_update: f32,
}

impl GameState {
    pub fn new() -> Self {
        let mut glfw = glfw::init(glfw::fail_on_errors).expect("Failed to init glfw");

        glfw.window_hint(glfw::WindowHint::ContextVersion(4, 6)); // OpenGL 3.3
        glfw.window_hint(glfw::WindowHint::OpenGlProfile(glfw::OpenGlProfileHint::Core));
        glfw.window_hint(glfw::WindowHint::Resizable(true));
        #[cfg(target_os = "macos")]
        glfw.window_hint(glfw::WindowHint::OpenGlForwardCompat(true));

        let (mut width, mut height):(i32, i32) = (1920, 1080);

        let (mut window, events) = glfw
            .create_window(width as u32, height as u32, "Hello this is window", glfw::WindowMode::Windowed)
            .expect("Failed to create GLFW window.");
        // window.set_key_polling(true);
        // window.set_sticky_keys(true); 
        window.set_cursor_mode(glfw::CursorMode::Disabled);
        window.set_all_polling(true);
        window.make_current();

        glfw.with_primary_monitor(|_glfw, maybe_monitor| {
            if let Some(monitor) = maybe_monitor {
                if let Some(video_mode) = monitor.get_video_mode() {
                    // Extract the current resolution & refresh rate from the monitor
                    (width, height) = (video_mode.width as i32, video_mode.height as i32);
                    let refresh_rate    = video_mode.refresh_rate; // e.g. 60, 144, etc.

                    window.set_monitor(
                        // glfw::WindowMode::Windowed,
                        glfw::WindowMode::FullScreen(monitor),
                        100,      // X-position on that monitor
                        100,      // Y-position on that monitor
                        width as u32,
                        height as u32,
                        Some(refresh_rate)
                    );
                }
            }
        });

        glfw.set_swap_interval(glfw::SwapInterval::Sync(1));


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

        //TEMP

        let mut terrain = Terrain::from_height_map("resources/textures/grid_height.png");
        let model = terrain.into_opengl_model();

        entity_manager.transforms.insert(entity_manager.next_entity_id, Transform {
            position: Vec3::splat(0.0),
            rotation: Quat::IDENTITY,
            scale: Vec3::splat(1.0),
            original_rotation: Quat::IDENTITY,
        });
        entity_manager.factions.insert(entity_manager.next_entity_id, Faction::World);
        entity_manager.entity_types.insert(entity_manager.next_entity_id, EntityType::Terrain);
        // entity_manager.models.insert(entity_manager.next_entity_id, model);

        entity_manager.next_entity_id += 1;

        // sound_manager.play_sound_3d("moose3D".to_string(), &vec3(0.0, 0.0, 4.0));

        let mut font_manager = FontManager::new();
        font_manager.load_chars("ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789:,.!?()[]{}<>");
        font_manager.setup_buffers();

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

            terrain,
            cursor_pos: Vec2::new(0.0, 0.0),
            font_manager,
            fps: 0,
            last_fps_update: 0.0,
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
                glfw::WindowEvent::CursorPos(xpos, ypos) => {
                    self.camera.process_mouse_input(&self.window, &event);
                    self.cursor_pos.x = xpos as f32;
                    self.cursor_pos.y = ypos as f32;
                },
                glfw::WindowEvent::Key(key, _, action, _) => {
                    handle_keyboard_input(key, action, &mut self.pressed_keys);
                },
                glfw::WindowEvent::MouseButton(btn, action, _) => {
                    handle_mouse_input(btn, action, self.cursor_pos, Vec2::new(self.fb_width as f32, self.fb_height as f32), &self.camera, &mut self.entity_manager, &self.pressed_keys);
                },
                _ => (),
            }
        }
    }

    pub fn update(&mut self) {
        let player_key = self.entity_manager.factions.iter().find(|f| f.value() == &Faction::Player).unwrap().key();
        let animator = self.entity_manager.animators.get_mut(player_key).unwrap();

        if self.pressed_keys.contains(&glfw::Key::P) {
            animator.set_next_animation("Death");
        }

        if self.pressed_keys.contains(&glfw::Key::O) {
            animator.set_next_animation("Idle");
        }

        if self.pressed_keys.contains(&glfw::Key::Delete) {
            for id in self.entity_manager.selected.iter() {
                self.entity_manager.entity_trashcan.push(*id);
            }
        }

        // CALC DELTA TIME
        let current_frame = self.glfw.get_time() as f32;
        self.delta_time = current_frame - self.last_frame;
        self.last_frame = current_frame;
        self.elapsed += self.delta_time;

        // SHOULD WE QUIT THE GAME?
        if self.paused { return; }
        if self.pressed_keys.contains(&glfw::Key::Escape) {
            self.window.set_should_close(true);
        }

        // UPDATE OOP-ESQUE STRUCTS
        self.camera.update(&self.entity_manager, self.delta_time);
        self.sound_manager.update(&self.camera);
        self.light_manager.update(&self.delta_time);

        // UPDATE SYSTEMS
        movement_system::update(
            &mut self.entity_manager, &self.terrain, self.delta_time, &self.camera, &self.pressed_keys
        );
        animation_system::update(&mut self.entity_manager, self.delta_time);
        state_machines::update(&mut self.entity_manager);
        collision_system::update(&mut self.entity_manager);
        self.entity_manager.update();
    }

    pub fn render(&mut self) {
        self.camera.reset_matrices(self.window_width as f32 / self.window_height as f32);
        self.renderer.draw(&self.entity_manager, &mut self.camera, &self.light_manager, &mut self.grid, &mut self.sound_manager, self.fb_width, self.fb_height, self.elapsed);

        self.imgui_manager.draw(&mut self.window, self.fb_width as f32, self.fb_height as f32, self.delta_time, &mut self.light_manager, &mut self.renderer, &mut self.sound_manager, &self.camera, &mut self.entity_manager);
        let fps_now = (1.0 / self.delta_time.max(0.0001)) as u32;

        if self.elapsed - self.last_fps_update >= 0.5 {
            self.fps = fps_now;
            self.last_fps_update = self.elapsed;
        }

        let phrase = format!("FPS: {}", self.fps);

        self.font_manager.render_phrase(
            &phrase,
            100.0,
            100.0,
            self.fb_width as f32,
            self.fb_height as f32,
            self.renderer.shaders.get_mut(&ShaderType::Text).unwrap(),
            0.5,
        );

        self.window.swap_buffers();
        self.glfw.poll_events()
    }
}
