use crate::resources::Handle;

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct RenderTarget;
pub type RenderTargetHandle = Handle<RenderTarget>;