<<<<<<< HEAD
use glam::Vec3;
=======
use glam::{Mat3, Vec3};
>>>>>>> aabb-collision-refine

use crate::{entity_manager::EntityManager, enums_types::{Size3, Transform}};

pub fn update(em: &mut EntityManager) {
    handle_entity_collisions(em);
}

fn handle_entity_collisions(em: &mut EntityManager) {
<<<<<<< HEAD
    let mut collisions = vec![];

    for t1 in em.transforms.iter() {
        for t2 in em.transforms.iter() {

            if t1.key() == t2.key() {
                continue;
            }

            if let (Some(s1), Some(s2)) = (em.sizes.get(t1.key()), em.sizes.get(t2.key())) {
                if check_collision_cube_cube(s1, s2, t1.value(), t2.value()) {
                    collisions.push((t1.key(), t2.key()));
=======
    let mut collisions: Vec<(usize, usize)> = vec![];

    for c1 in em.cylinders.iter() {
        for c2 in em.cylinders.iter() {
            if c1.key() >= c2.key() {
                continue;
            }

            let id1 = c1.key();
            let id2 = c2.key();

            if let (Some(t1), Some(t2)) = (em.transforms.get(id1), em.transforms.get(id2)) {
                let cyl1 = c1.value();
                let cyl2 = c2.value();

                let dx = t1.position.x - t2.position.x;
                let dz = t1.position.z - t2.position.z;
                let dist_sq = dx * dx + dz * dz;
                let radius_sum = cyl1.r + cyl2.r;
                let overlap_horizontal = dist_sq <= radius_sum * radius_sum;

                let y1_min = t1.position.y;
                let y1_max = y1_min + cyl1.h;
                let y2_min = t2.position.y;
                let y2_max = y2_min + cyl2.h;
                let overlap_vertical = y1_min < y2_max && y1_max > y2_min;

                if overlap_horizontal && overlap_vertical {
                    collisions.push((id1, id2));
>>>>>>> aabb-collision-refine
                }
            }
        }
    }

<<<<<<< HEAD
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
=======
    if !collisions.is_empty() {
        println!("COLLISION DETECTED!!!!!!");
    }
}
>>>>>>> aabb-collision-refine
