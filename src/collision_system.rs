use glam::{Mat3, Vec3};

use crate::{entity_manager::EntityManager, enums_types::{Size3, Transform}};

pub fn update(em: &mut EntityManager) {
    handle_entity_collisions(em);
}

fn handle_entity_collisions(em: &mut EntityManager) {
    let mut collisions = vec![];

    for t1 in em.transforms.iter() {
        for t2 in em.transforms.iter() {

            if t1.key() == t2.key() {
                continue;
            }

            if let (Some(s1), Some(s2)) = (em.sizes.get(t1.key()), em.sizes.get(t2.key())) {
                if check_collision_obb_obb(s1, s2, t1.value(), t2.value()) {
                    collisions.push((t1.key(), t2.key()));
                }
            }
        }
    }

    // Resolve collisions
    if !collisions.is_empty() {
        println!("COLLISION DETECTED");
    } else {
        println!("no collisions....");
    }
}

pub fn check_collision_obb_obb(s1: &Size3, s2: &Size3, t1: &Transform, t2: &Transform) -> bool {
    const EPSILON: f32 = 1e-6;

    // Local half-extents
    let half1 = Vec3::new(s1.w, s1.h, s1.d) * t1.scale * 0.5;
    let half2 = Vec3::new(s2.w, s2.h, s2.d) * t2.scale * 0.5;

    // Orthonormal basis vectors for each box
    let rot1 = Mat3::from_quat(t1.rotation);
    let rot2 = Mat3::from_quat(t2.rotation);

    let axes1 = [rot1.x_axis, rot1.y_axis, rot1.z_axis];
    let axes2 = [rot2.x_axis, rot2.y_axis, rot2.z_axis];

    // Translation vector from box1 to box2 in world space
    let t = t2.position - t1.position;

    // Translation in box1's local space
    let t = Vec3::new(t.dot(axes1[0]), t.dot(axes1[1]), t.dot(axes1[2]));

    // Rotation matrix: dot products between each pair of axes
    let mut r = [[0.0f32; 3]; 3];
    let mut abs_r = [[0.0f32; 3]; 3];
    for i in 0..3 {
        for j in 0..3 {
            r[i][j] = axes1[i].dot(axes2[j]);
            abs_r[i][j] = r[i][j].abs();
        }
    }

    // Test axes L = A0, A1, A2
    for i in 0..3 {
        let ra = half1[i];
        let rb = half2[0] * abs_r[i][0] + half2[1] * abs_r[i][1] + half2[2] * abs_r[i][2];
        if t[i].abs() > ra + rb {
            return false;
        }
    }

    // Test axes L = B0, B1, B2
    for i in 0..3 {
        let ra = half1[0] * abs_r[0][i] + half1[1] * abs_r[1][i] + half1[2] * abs_r[2][i];
        let rb = half2[i];
        let proj = t[0] * r[0][i] + t[1] * r[1][i] + t[2] * r[2][i];
        if proj.abs() > ra + rb {
            return false;
        }
    }

    // Test cross-product axes
    for i in 0..3 {
        for j in 0..3 {
            let ra = half1[(i + 1) % 3] * abs_r[(i + 2) % 3][j] +
                     half1[(i + 2) % 3] * abs_r[(i + 1) % 3][j];
            let rb = half2[(j + 1) % 3] * abs_r[i][(j + 2) % 3] +
                     half2[(j + 2) % 3] * abs_r[i][(j + 1) % 3];
            let proj = (t[(i + 2) % 3] * r[(i + 1) % 3][j] -
                        t[(i + 1) % 3] * r[(i + 2) % 3][j]).abs();
            if proj > ra + rb {
                return false;
            }
        }
    }

    true // No separating axis found
}
