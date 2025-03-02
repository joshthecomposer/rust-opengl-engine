use std::collections::HashMap;

use glam::Mat4;
use russimp::{node::Node, scene::{PostProcess, Scene}};

use super::{ani_bone::AniBone, ani_model::{AniModel, BoneInfo}};

pub struct AssimpNodeData {
    transformation: Mat4,
    name: String,
    children: Vec<AssimpNodeData>,
}

impl AssimpNodeData {
    pub fn new() -> Self {
        Self {
            transformation: Mat4::IDENTITY,
            name: "".to_string(),
            children: Vec::new(),
        }
    }
}

pub struct Animation {
    duration: f32,
    ticks_per_second: usize,
    bones: Vec<AniBone>,
    root_node: AssimpNodeData,
    bone_info_map: HashMap<String, BoneInfo>,
}

impl Animation {
    pub fn new(animation_path: String, model: &mut AniModel) {
        println!("=============================================================");
        println!("BEGIN LOADING OF SCENE FOR ANIMATION: {}", animation_path);
        println!("=============================================================");

        let scene = Scene::from_file(
            animation_path.as_str(),
            vec![
                PostProcess::Triangulate,
                PostProcess::FlipUVs,
            ],
        ).unwrap();

        assert!(scene.root.is_some());

        let animation = scene.animations.get(0).unwrap();
        let duration = animation.duration;
        let ticks_per_second = animation.ticks_per_second;
    }

    pub fn read_heirarchy_data(dest: &mut AssimpNodeData, src: &Node) {
        dest.name = src.name.clone();
        dest.transformation = AniModel::russimp_mat4_to_glam(src.transformation);

        for child in src.children.borrow().iter() {
            let mut new_data = AssimpNodeData::new();
            Self::read_heirarchy_data(&mut new_data, &child);
        }
    }
}
