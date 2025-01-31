#[derive(Debug, Eq, PartialEq, Hash)]
pub enum VaoType {
    CubeVao,
    SkyboxVao
}

#[derive(Debug, Eq, PartialEq, Hash)]
pub enum VboType {
    CubeVbo,
    SkyboxVbo
}

#[derive(Debug, Eq, PartialEq, Hash)]
pub enum EboType {
    CubeEbo,
    SkyboxEbo
}

#[derive(Debug, Eq, PartialEq, Hash)]
pub enum ShaderType {
    MainShader,
    SkyboxShader
}
