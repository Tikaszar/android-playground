use crate::resources::Handle;

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct Pipeline;
pub type PipelineHandle = Handle<Pipeline>;