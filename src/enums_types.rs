use glam::{Mat4, Vec3};

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
    Text
}

pub struct Transform {
    pub position: Vec3,
    pub rotation: Mat4,
    pub scale: Vec3,
}

#[derive(Clone, Debug)]
pub enum EntityType {
    Donut,
    ArcherTower01,
    BlockGrass,
    Tree,
    Grass,
}

#[derive(Clone, Debug)]
pub enum CellType {
    Grass,
    Tree,
    Path
}
