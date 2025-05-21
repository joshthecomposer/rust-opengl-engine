use glam::{vec3, Mat3, Mat4, Quat, Vec3};
use rand::{rng, Rng};

use crate::{camera::Camera, gl_call, shaders::Shader};

pub struct Emitter {
    pub positions: Vec<Vec3>,
    pub times_alive: Vec<f32>,
    pub lifetimes: Vec<f32>,
    pub velocities: Vec<Vec3>,
    pub scales: Vec<Vec3>,
    pub count: usize,
    pub alive: bool,

    pub pps: usize,
    pub emit_accumulator: f32,
    pub origin: Vec3,
}

impl Emitter {
    pub fn new() -> Self {
        Self {
            positions: vec![],
            times_alive: vec![],
            lifetimes: vec![],
            velocities: vec![],
            scales: vec![],
            count: 0,
            alive: true,
            pps: 0,
            emit_accumulator: 0.0,
            origin: Vec3::splat(1.0),
        }
    }

    pub fn render(&self, shader: &mut Shader, camera: &Camera, vao: u32) {
        unsafe {
            gl::Enable(gl::BLEND);
            gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
        }

        shader.activate();

        for i in 0..self.count {
            let view = camera.view;
            let view_rot = Mat3::from_cols(
                view.x_axis.truncate(),
                view.y_axis.truncate(),
                view.z_axis.truncate(),
            );
            let inv_view_rot = view_rot.transpose();
            let model_rot = Mat4::from_mat3(inv_view_rot);
            let model = Mat4::from_translation(self.positions[i]) * model_rot * Mat4::from_scale(self.scales[i]);

            shader.set_mat4("model", model);
            shader.set_mat4("view", camera.view);
            shader.set_mat4("projection", camera.projection);

            unsafe {
                gl_call!(gl::BindVertexArray(vao));
                gl_call!(gl::DrawArrays(gl::TRIANGLES, 0, 6));
                gl_call!(gl::BindVertexArray(0));
            }
        }
    }
}

pub struct ParticleSystem {
    pub emitters: Vec<Emitter>,
    pub vao: u32,
}

impl ParticleSystem {
    pub fn new() -> Self {
        let mut vao = 0;
        let mut vbo = 0;

        let quad_vertices: [f32; 18] = [
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
                gl::STATIC_DRAW,
            ));

            gl_call!(gl::EnableVertexAttribArray(0));
            gl_call!(gl::VertexAttribPointer(0, 3, gl::FLOAT, gl::FALSE, 3 * 4, std::ptr::null()));
            gl_call!(gl::BindVertexArray(0));
        }

        Self {
            emitters: Vec::new(),
            vao,
        }
    }

    pub fn spawn_oneshot_emitter(&mut self, count: usize, origin: Vec3) {
        let mut rng = rng();

        let mut emitter = Emitter::new();

        for _ in 0..count {
            let angle = rng.random_range(0.0..std::f32::consts::TAU);
            let radius = rng.random_range(0.01..0.1);

            let x = radius * angle.cos();
            let z = radius * angle.sin();
            let position = origin + vec3(x, 0.0, z);

            let outward = vec3(x, 0.0, z).normalize();
            let upward = vec3(0.0, rng.random_range(0.0..2.0), 0.0);
            let velocity = (outward + upward).normalize() * rng.random_range(1.0..3.0);

            let lifetime = rng.random_range(1.0..5.0);
            let scale = Vec3::splat(rng.random_range(0.005..0.010));

            emitter.positions.push(position);
            emitter.velocities.push(velocity);
            emitter.lifetimes.push(lifetime);
            emitter.scales.push(scale);
            emitter.times_alive.push(0.0);
        }

        emitter.count = count;
        self.emitters.push(emitter);
    }

    pub fn spawn_continuous_emitter(&mut self, pps: usize, origin: Vec3) {
        let mut emitter = Emitter::new();
        emitter.pps = pps;
        emitter.origin = origin;
        self.emitters.push(emitter);
    }

    pub fn update(&mut self, dt: f32) {
        let gravity = vec3(0.0, -6.0, 0.0);

        for emitter in self.emitters.iter_mut() {
            if emitter.pps > 0 {
                emitter.emit_accumulator += dt;
                let seconds_per_particle = 1.0 / emitter.pps as f32;

                while emitter.emit_accumulator >= seconds_per_particle {
                    emitter.emit_accumulator -= seconds_per_particle;
                    Self::spawn_particle(emitter);
                }
            }

            let mut i = 0;
            while i < emitter.count {
                if emitter.times_alive[i] >= emitter.lifetimes[i] {
                    let last = emitter.count - 1;

                    emitter.positions.swap(i, last);
                    emitter.times_alive.swap(i, last);
                    emitter.lifetimes.swap(i, last);
                    emitter.velocities.swap(i, last);

                    emitter.count -= 1;
                } else {
                    emitter.velocities[i] += gravity * dt;
                    emitter.positions[i] += emitter.velocities[i] * dt;
                    emitter.times_alive[i] += dt;

                    if emitter.positions[i].y <= 0.0 {
                        emitter.positions[i].y = 0.0;
                        emitter.velocities[i].y *= -0.3;


                        let friction = 0.9; // Lower = more friction, e.g., 0.8 retains 80% 
                        emitter.velocities[i].x *= friction;
                        emitter.velocities[i].z *= friction;
                    }

                    i += 1;
                }
            }

            if emitter.count == 0 && emitter.pps == 0 {
                emitter.alive = false;
            }
        }

        self.emitters.retain(|e| e.alive);
    }

    pub fn spawn_particle(emitter: &mut Emitter) {
        let mut rng = rng();
        let angle = rng.random_range(0.0..std::f32::consts::TAU);
        let radius = rng.random_range(0.01..0.1);

        let x = radius * angle.cos();
        let z = radius * angle.sin();
        let position = emitter.origin + vec3(x, 0.0, z);

        let outward = vec3(x, 0.0, z).normalize_or_zero();
        let upward = vec3(0.0, rng.random_range(2.0..5.0), 0.0);
        let velocity = outward * rng.random_range(1.0..2.0) + upward;

        let lifetime = rng.random_range(3.0..5.5);
        let scale = Vec3::splat(rng.random_range(0.009..0.013));

        // TODO: Instead allocate the right size at the beginning by multiplying the particles per second by the lifetimes
        if emitter.count < emitter.positions.len() {
            let i = emitter.count;
            emitter.positions[i] = position;
            emitter.velocities[i] = velocity;
            emitter.lifetimes[i] = lifetime;
            emitter.scales[i] = scale;
            emitter.times_alive[i] = 0.0;
        } else {
            emitter.positions.push(position);
            emitter.velocities.push(velocity);
            emitter.lifetimes.push(lifetime);
            emitter.scales.push(scale);
            emitter.times_alive.push(0.0);
        }
        emitter.count += 1;
    }

    pub fn render(&mut self, shader: &mut Shader, camera: &Camera) {
        for emitter in &self.emitters {
            emitter.render(shader, camera, self.vao);
        }
    }

}
