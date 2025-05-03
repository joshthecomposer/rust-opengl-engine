#![allow(dead_code)]
use std::fmt::{self, Display, Formatter};

use glam::{Quat, Vec3};
use serde::Deserialize;

#[derive(Debug, Eq, PartialEq, Hash)]
pub enum VaoType {
    Cube,
    Skybox,
    DebugLight,
    GroundPlane
}

#[derive(Debug, Eq, PartialEq, Hash)]
pub enum VboType {
    Cube,
    Skybox,
    DebugLight,
    GroundPlane,
}

#[derive(Debug, Eq, PartialEq, Hash)]
pub enum EboType {
    Cube,
    Skybox,
    DebugLight,
}

#[derive(Debug, Eq, PartialEq, Hash)]
pub enum FboType {
    DepthMap,
}

#[derive(Debug, Eq, PartialEq, Hash)]
pub enum ShaderType {
    Skybox,
    DebugLight,
    Depth,
    GroundPlane,
    DebugShadowMap,
    Model,
    Text,
    Gizmo,
}

/// A struct to carry some rotation state for blending between rotations smoothly
/// different than the Transform which just holds the current true simulation state
/// which might be blended between cur_rot and next_rot
#[derive(Debug)]
pub struct Rotator {
    pub cur_rot: Quat,
    pub next_rot: Quat,
    pub blend_factor: f32,
    pub blend_time: f32,
}

#[derive(Debug, Clone)]
pub struct Transform {
    pub position: Vec3,
    pub rotation: Quat,
    pub scale: Vec3,

    pub original_rotation: Quat,
}

#[derive(Clone, Debug, PartialEq, Deserialize)]
pub enum EntityType {
    Donut,
    ArcherTower01,
    BlockGrass,
    Tree,
    TreeFoliage,
    TreeTrunk,
    Grass,
    DemonLady,
    BigGuy,
    MooseMan,
    YRobot,
    Terrain,
    Cylinder,
}

#[derive(Clone, Debug, PartialEq, Deserialize)]
pub enum Faction {
    Enemy,
    Static,
    World,
    Player,
    Gizmo,
}

#[derive(Clone, Debug)]
pub enum CellType {
    Grass,
    Tree,
    Path
}

#[derive(Clone, Debug, PartialEq)]
pub enum TextureType {
    Diffuse, 
    Specular,
    Emissive,
    NormalMap,
    Roughness,
    Metalness,
    Displacement,
    Opacity,
}

impl Display for TextureType {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            TextureType::Diffuse => write!(f, "Diffuse"),
            TextureType::Specular => write!(f, "Specular"),
            TextureType::Emissive => write!(f, "Emissive"),
            TextureType::NormalMap => write!(f, "Normal Map"),
            TextureType::Roughness => write!(f, "Roughness"),
            TextureType::Metalness => write!(f, "Metalness"),
            TextureType::Displacement => write!(f, "Displacement"),
            TextureType::Opacity => write!(f, "Opacity"),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum CameraState {
    Free,
    Third,
    Locked,
}

#[derive(Clone, Debug)]
pub struct Size3 {
    pub w: f32,
    pub h: f32,
    pub d: f32,
}
<<<<<<< HEAD
=======

pub struct Parent {
    pub parent_id: usize,
}
>>>>>>> aabb-collision-refine
