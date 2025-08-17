use crate::resources::Handle;

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct Texture;
pub type TextureHandle = Handle<Texture>;