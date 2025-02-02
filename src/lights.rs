use std::collections::HashMap;

use glam::Vec3;

use crate::sparse_set::SparseSet;

pub struct PointStrength {
    pub constant: f32,
    pub linear: f32,
    pub quadratic: f32,
}

pub const POINT_STRENGTHS: [(u32, PointStrength); 12] = [
    (7, PointStrength { constant: 1.0, linear: 0.7, quadratic: 1.8 }),
    (13, PointStrength { constant: 1.0, linear: 0.35, quadratic: 0.44 }),
    (20, PointStrength { constant: 1.0, linear: 0.22, quadratic: 0.20 }),
    (32, PointStrength { constant: 1.0, linear: 0.14, quadratic: 0.07 }),
    (50, PointStrength { constant: 1.0, linear: 0.09, quadratic: 0.032 }),
    (65, PointStrength { constant: 1.0, linear: 0.07, quadratic: 0.017 }),
    (100, PointStrength { constant: 1.0, linear: 0.045, quadratic: 0.0075 }),
    (160, PointStrength { constant: 1.0, linear: 0.027, quadratic: 0.0028 }),
    (200, PointStrength { constant: 1.0, linear: 0.022, quadratic: 0.0019 }),
    (325, PointStrength { constant: 1.0, linear: 0.014, quadratic: 0.0007 }),
    (600, PointStrength { constant: 1.0, linear: 0.007, quadratic: 0.0002 }),
    (3250, PointStrength { constant: 1.0, linear: 0.0014, quadratic: 0.000007 }),
];
    

pub struct PointLight {
    pub position: Vec3,

    pub ambient: Vec3,
    pub diffuse: Vec3,
    pub specular: Vec3,

    pub constant: f32,
    pub linear: f32, 
    pub quadratic: f32,
}

impl PointLight {
    pub fn with_default_strength(
        position: Vec3,
        ambient: Vec3,
        diffuse: Vec3,
        specular: Vec3
    ) -> Self {
        Self {
            position,
            ambient,
            diffuse,
            specular,
            constant: 1.0,
            linear: 0.09,
            quadratic: 0.032,
        }
    }
}

pub struct Lights {
    next_light_id: usize,
    point_lights: SparseSet<PointLight>,
    velocities: SparseSet<Vec3>,
    point_strengths: HashMap<u32, PointStrength>,
}

impl Lights {
    pub fn new(max_lights: usize) -> Self {
        let point_strengths = HashMap::from(POINT_STRENGTHS);
        Self {
            next_light_id: 0,
            point_lights: SparseSet::with_capacity(max_lights),
            velocities: SparseSet::with_capacity(max_lights),
            point_strengths,
        }
    }

    pub fn add_point_light(&mut self, mut light: PointLight, distance: u32) {
        if let Some(strength) = self.point_strengths.get(&distance) {
            light.constant = strength.constant;
            light.linear = strength.linear;
            light.quadratic = strength.quadratic;
        }
        self.point_lights.insert(self.next_light_id, light);
        self.next_light_id += 1;
    }

    pub fn update(&mut self, delta: &f64) {
        for i in self.point_lights.iter_mut() {
            if let Some(velocity) = self.velocities.get(i.key()) {
                i.value.position += velocity;
            }
        }
    }

    pub fn debug_render() {
    }
}
