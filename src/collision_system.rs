use glam::Vec3;

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
                if check_collision_cube_cube(s1, s2, t1.value(), t2.value()) {
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

fn check_collision_cube_cube(s1: &Size3, s2: &Size3, t1: &Transform, t2: &Transform) -> bool {
    let half1 = Vec3::new(s1.w * t1.scale.x / 2.0, s1.h * t1.scale.y / 2.0, s1.d * t1.scale.z / 2.0);
    let half2 = Vec3::new(s2.w * t2.scale.x / 2.0, s2.h * t2.scale.y / 2.0, s2.d * t2.scale.z / 2.0);

    let min1 = t1.position - half1;
    let max1 = t1.position + half1;

    let min2 = t2.position - half2;
    let max2 = t2.position + half2;

    // println!(
    //     "Entity 1 AABB: min {:?}, max {:?} | Entity 2 AABB: min {:?}, max {:?}",
    //     min1, max1, min2, max2
    // );

    !(min1.x > max2.x || max1.x < min2.x ||
      min1.y > max2.y || max1.y < min2.y ||
      min1.z > max2.z || max1.z < min2.z)
}
