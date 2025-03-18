#![allow(dead_code)]
use std::fmt::{self, Display, Formatter};

use glam::{Mat4, Quat, Vec3};

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
    AniModel
}

pub struct Transform {
    pub position: Vec3,
    pub rotation: Quat,
    pub scale: Vec3,
}

#[derive(Clone, Debug)]
pub enum EntityType {
    Donut,
    ArcherTower01,
    BlockGrass,
    Tree,
    Grass,
    DemonLady,
    BigGuy,
}

#[derive(Clone, Debug)]
pub enum Faction {
    Enemy,
    Static,
    World,
    Player,
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
        }
    }
}
