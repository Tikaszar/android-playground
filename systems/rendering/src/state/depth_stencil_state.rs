use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CompareFunc {
    Never,
    Less,
    Equal,
    LessEqual,
    Greater,
    NotEqual,
    GreaterEqual,
    Always,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StencilOp {
    Keep,
    Zero,
    Replace,
    IncrementClamp,
    DecrementClamp,
    Invert,
    IncrementWrap,
    DecrementWrap,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct StencilState {
    pub compare: CompareFunc,
    pub pass_op: StencilOp,
    pub fail_op: StencilOp,
    pub depth_fail_op: StencilOp,
    pub read_mask: u32,
    pub write_mask: u32,
    pub reference: u32,
}

impl Default for StencilState {
    fn default() -> Self {
        Self {
            compare: CompareFunc::Always,
            pass_op: StencilOp::Keep,
            fail_op: StencilOp::Keep,
            depth_fail_op: StencilOp::Keep,
            read_mask: 0xFF,
            write_mask: 0xFF,
            reference: 0,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct DepthStencilState {
    pub depth_test_enabled: bool,
    pub depth_write_enabled: bool,
    pub depth_compare: CompareFunc,
    pub stencil_test_enabled: bool,
    pub front_stencil: StencilState,
    pub back_stencil: StencilState,
}

impl Default for DepthStencilState {
    fn default() -> Self {
        Self {
            depth_test_enabled: false,
            depth_write_enabled: true,
            depth_compare: CompareFunc::Less,
            stencil_test_enabled: false,
            front_stencil: StencilState::default(),
            back_stencil: StencilState::default(),
        }
    }
}