use glam::{vec3, Mat3, Mat4, Quat, Vec3};
use rand::{rng, Rng};

use crate::{camera::Camera, gl_call, shaders::Shader};

pub struct Particles {
    pub positions: Vec<Vec3>,
    pub times_alive: Vec<f32>,
    pub lifetimes: Vec<f32>,
    pub velocities: Vec<Vec3>,
    pub scales: Vec<Vec3>,

    pub vao: u32,
}

impl Particles {
    pub fn new() -> Self {
        Self {
            positions: vec![],
            times_alive: vec![],
            lifetimes: vec![],
            velocities: vec![],
            scales: vec![],

            vao: 0,
        }
    }

    pub fn update(&mut self, dt: f32) {
        let gravity = vec3(0.0, -9.8, 0.0);
        let mut i = 0;

        while i < self.positions.len() {
            if self.times_alive[i] >= self.lifetimes[i] {
                let last = self.positions.len() - 1;

                self.positions.swap(i, last);
                self.times_alive.swap(i, last);
                self.lifetimes.swap(i, last);
                self.velocities.swap(i, last);

                self.positions.pop();
                self.times_alive.pop();
                self.lifetimes.pop();
                self.velocities.pop();

            } else {
                self.velocities[i] += gravity * dt;
                self.positions[i] += self.velocities[i] * dt;
                self.times_alive[i] += dt;

                if self.positions[i].y <= 0.0 {
                    self.positions[i].y = 0.0;
                }
                i += 1;
            }
        }
    }

    pub fn setup_opengl(&mut self) {
        let mut vao = 0;
        let mut vbo = 0;

        let quad_vertices: [f32; 18] = [
            // Positions
            -1.0,  1.0, 0.0,
            -1.0, -1.0, 0.0,
             1.0, -1.0, 0.0,

            -1.0,  1.0, 0.0,
             1.0, -1.0, 0.0,
             1.0,  1.0, 0.0,
        ];

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

            let stride = (3 * std::mem::size_of::<f32>()) as i32;

            // Position Attribute
            gl_call!(gl::EnableVertexAttribArray(0));
            gl_call!(gl::VertexAttribPointer(0, 3, gl::FLOAT, gl::FALSE, stride, std::ptr::null()));

            gl_call!(gl::BindBuffer(gl::ARRAY_BUFFER, 0));
            gl_call!(gl::BindVertexArray(0));
        }

        self.vao = vao;
    }

    pub fn render(&mut self, shader: &mut Shader, camera: &Camera) {
        unsafe {
            gl::Enable(gl::BLEND);
            gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
        }
        shader.activate();

        for (i, p) in self.positions.iter().enumerate() {
            let view = camera.view;
            let view_rot = Mat3::from_cols(
                view.x_axis.truncate(),
                view.y_axis.truncate(),
                view.z_axis.truncate(),
            );
            let inv_view_rot = view_rot.transpose();

            let model_rot = Mat4::from_mat3(inv_view_rot);
            let model = Mat4::from_translation(*p) * model_rot * Mat4::from_scale(self.scales[i]);

            shader.set_mat4("model", model);
            shader.set_mat4("view", camera.view);
            shader.set_mat4("projection", camera.projection);

            unsafe {
                gl_call!(gl::BindVertexArray(self.vao));
                gl_call!(gl::DrawArrays(gl::TRIANGLES, 0, 6));
                gl_call!(gl::BindVertexArray(0));

            }
        }
       unsafe { gl::Enable(gl::BLEND); }
    }

    pub fn spawn_particles(&mut self, count: u32, origin: Vec3) {
        let mut rng = rng();

        for _ in 0..count {
            let angle = rng.random_range(0.0..std::f32::consts::TAU);
            let radius = rng.random_range(0.01..0.1);

            let x = radius * angle.cos();
            let z = radius * angle.sin();
            let position = origin + vec3(x, 0.0, z);

            let outward = vec3(x, 0.0, z).normalize();
            let upward = vec3(0.0, rng.random_range(0.0..2.0), 0.0);
            let velocity = (outward + upward).normalize() * rng.random_range(1.0..3.0);

            let lifetime = rng.random_range(2.0..3.0);
            let scale = Vec3::splat(rng.random_range(0.005..0.010));

            self.positions.push(position);
            self.velocities.push(velocity);
            self.lifetimes.push(lifetime);
            self.scales.push(scale);
            self.times_alive.push(0.0);
        }
    }
}


