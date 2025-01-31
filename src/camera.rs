use glam::{vec3, Mat4, Vec3};

pub struct Camera {
    pub yaw: f32,
    pub pitch: f32,
    pub direction: Vec3,
    pub position: Vec3,
    pub forward: Vec3,
    pub up: Vec3,
    pub target: Vec3,
    pub right: Vec3,
    pub fovy: f32,
    pub movement_speed: f32,
    pub sensitivity: f32,

    pub z_near: f32,
    pub z_far: f32,

    pub projection: Mat4,
    pub view: Mat4,
    pub model: Mat4,
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
            movement_speed: 2.5,
            sensitivity: 0.1,

            z_near: 0.1,
            z_far: 300.0,

            projection: Mat4::IDENTITY,
            view: Mat4::IDENTITY,
            model: Mat4::IDENTITY,
        }
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
}
