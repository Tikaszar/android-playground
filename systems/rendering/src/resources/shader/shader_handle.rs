use crate::resources::Handle;

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct Shader;
pub type ShaderHandle = Handle<Shader>;