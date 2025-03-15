#![allow(dead_code)]
use glam::{vec3, Mat4, Vec3};
use glfw::{Action, Key, PWindow, WindowEvent};

use crate::entity_manager::EntityManager;

pub struct Camera {
    pub yaw: f64,
    pub pitch: f64,
    pub direction: Vec3,
    pub position: Vec3,
    pub forward: Vec3,
    pub up: Vec3,
    pub target: Vec3,
    pub right: Vec3,
    pub fovy: f32,
    pub movement_speed: f32,
    pub sensitivity: f64,

    pub first_mousing: bool,
    pub last_x: f64, 
    pub last_y: f64,

    pub z_near: f32,
    pub z_far: f32,

    pub projection: Mat4,
    pub view: Mat4,
    pub model: Mat4,
    pub light_space: Mat4,
}

impl Camera {
    pub fn new() -> Self {
        Self {
            yaw: -90.0,
            pitch: 0.0,
            direction: vec3(0.0, 0.0, 0.0),
            position: vec3(0.0, 0.0, 15.0),
            forward: vec3(0.0, 0.0, -1.0),
            up: vec3(0.0, 1.0, 0.0),
            target: vec3(0.0, 0.0, 0.0),
            right: vec3(0.0, 0.0, 0.0),
            fovy: 45.0_f32.to_radians(),
            movement_speed: 25.0,
            sensitivity: 0.1,
            first_mousing: true,
            last_x: 0.0,
            last_y: 0.0,

            z_near: 0.1,
            z_far: 10000.0,

            projection: Mat4::IDENTITY,
            view: Mat4::IDENTITY,
            model: Mat4::IDENTITY,
            light_space: Mat4::IDENTITY,
        }
    }

    pub fn update(&mut self, em: &EntityManager) {
        // let player_pos = em.transforms.get(0).unwrap();

        // self.target = player_pos.position;

        // self.position = self.target + vec3(3.0, 3.0, 0.0);
    }

    pub fn get_view_matrix(&mut self) {
        self.view = Mat4::look_at_rh(self.position, self.target, self.up);
    }

    pub fn reset_matrices(&mut self, aspect: f32) {
        self.projection = Mat4::IDENTITY;
        self.projection = Mat4::perspective_rh_gl(self.fovy, aspect, self.z_near, self.z_far);
        
        self.view = Mat4::IDENTITY;
        self.target = self.position + self.forward;

        self.view = Mat4::look_at_rh(self.position, self.target, self.up);

        self.model = Mat4::IDENTITY;
    }

    pub fn process_mouse_input(&mut self, window: &PWindow, event: &WindowEvent) {
        match event {
        // Pitch yaw stuff
            glfw::WindowEvent::CursorPos(xpos, ypos) => {
                if self.first_mousing {
                    self.last_x = *xpos;
                    self.last_y = *ypos;
                    self.first_mousing = false;
                    return;
                }

                let mut x_offset = xpos - self.last_x;
                let mut y_offset = self.last_y - ypos;

                self.last_x = *xpos;
                self.last_y = *ypos;

                x_offset *= self.sensitivity; 
                y_offset *= self.sensitivity;

                self.yaw += x_offset;
                self.pitch += y_offset;

                if self.yaw >= 360.0 { 
                    self.yaw -= 360.0;
                } else if self.yaw < 0.0 {
                    self.yaw += 360.0;
                }

                if self.pitch > 89.0 {
                    self.pitch = 89.0;
                }
                if self.pitch < -89.0 {
                    self.pitch = -89.0;
                }

                self.direction.x = (self.yaw.to_radians().cos() * self.pitch.to_radians().cos()) as f32;
                self.direction.y = self.pitch.to_radians().sin() as f32;
                self.direction.z = (self.yaw.to_radians().sin() * self.pitch.to_radians().cos()) as f32;
                self.direction = self.direction.normalize();

                self.forward = self.direction;
            },
            _ => {}

        }
        
        // Zoom
        if window.get_key(Key::U) == Action::Press {
            self.fovy = 5.0_f32.to_radians();
        } else {
            self.fovy = 45.0_f32.to_radians();
        }

    }

    pub fn process_key_event(&mut self, window: &PWindow, delta: f64) {
        if window.get_key(Key::W) == Action::Press {
            self.position += (self.movement_speed * self.forward) * delta as f32;
            // self.position.z += (50.0 * delta) as f32;
        }
        if window.get_key(Key::S) == Action::Press {
            self.position -= (self.movement_speed * self.forward) * delta as f32;
        }
        if window.get_key(Key::A) == Action::Press {
            // cameraPos -= glm::normalize(glm::cross(cameraFront, cameraUp)) * cameraSpeed;
            self.position += ((self.up.cross(self.forward).normalize()) * self.movement_speed) * delta as f32;
        }
        if window.get_key(Key::D) == Action::Press {
            self.position -= ((self.up.cross(self.forward).normalize()) * self.movement_speed) * delta as f32;
        }

        // self.position.y = 0.5;
    }
}
