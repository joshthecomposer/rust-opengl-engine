#[derive(Debug, Eq, PartialEq, Hash)]
pub enum VaoType {
    Cube,
    Skybox,
    DebugLight
}

#[derive(Debug, Eq, PartialEq, Hash)]
pub enum VboType {
    Cube,
    Skybox,
    DebugLight,
}

#[derive(Debug, Eq, PartialEq, Hash)]
pub enum EboType {
    Cube,
    Skybox,
    DebugLight,
}

#[derive(Debug, Eq, PartialEq, Hash)]
pub enum ShaderType {
    Main,
    Skybox,
    DebugLight
}
