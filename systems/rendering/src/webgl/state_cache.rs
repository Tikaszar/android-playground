use web_sys::WebGl2RenderingContext;
use crate::state::{BlendState, DepthStencilState, RasterizerState, BlendFactor, BlendOp, CompareFunc, CullMode, FillMode};

pub struct StateCache {
    current_blend: Option<BlendState>,
    current_depth_stencil: Option<DepthStencilState>,
    current_rasterizer: Option<RasterizerState>,
    current_viewport: Option<(i32, i32, i32, i32)>,
    current_scissor: Option<(i32, i32, i32, i32)>,
}

impl StateCache {
    pub fn new() -> Self {
        Self {
            current_blend: None,
            current_depth_stencil: None,
            current_rasterizer: None,
            current_viewport: None,
            current_scissor: None,
        }
    }

    pub fn set_blend_state(&mut self, gl: &WebGl2RenderingContext, state: &BlendState) {
        if self.current_blend.as_ref() == Some(state) {
            return;
        }

        if state.enabled {
            gl.enable(WebGl2RenderingContext::BLEND);
            gl.blend_func_separate(
                blend_factor_to_gl(state.src_color),
                blend_factor_to_gl(state.dst_color),
                blend_factor_to_gl(state.src_alpha),
                blend_factor_to_gl(state.dst_alpha),
            );
            gl.blend_equation_separate(
                blend_op_to_gl(state.color_op),
                blend_op_to_gl(state.alpha_op),
            );
            gl.blend_color(
                state.constant_color[0],
                state.constant_color[1],
                state.constant_color[2],
                state.constant_color[3],
            );
        } else {
            gl.disable(WebGl2RenderingContext::BLEND);
        }

        self.current_blend = Some(*state);
    }

    pub fn set_depth_stencil_state(&mut self, gl: &WebGl2RenderingContext, state: &DepthStencilState) {
        if self.current_depth_stencil.as_ref() == Some(state) {
            return;
        }

        // Depth test
        if state.depth_test_enabled {
            gl.enable(WebGl2RenderingContext::DEPTH_TEST);
            gl.depth_func(compare_func_to_gl(state.depth_compare));
        } else {
            gl.disable(WebGl2RenderingContext::DEPTH_TEST);
        }
        gl.depth_mask(state.depth_write_enabled);

        // Stencil test
        if state.stencil_test_enabled {
            gl.enable(WebGl2RenderingContext::STENCIL_TEST);
            // Set front face stencil
            gl.stencil_func_separate(
                WebGl2RenderingContext::FRONT,
                compare_func_to_gl(state.front_stencil.compare),
                state.front_stencil.reference as i32,
                state.front_stencil.read_mask,
            );
            // Set back face stencil
            gl.stencil_func_separate(
                WebGl2RenderingContext::BACK,
                compare_func_to_gl(state.back_stencil.compare),
                state.back_stencil.reference as i32,
                state.back_stencil.read_mask,
            );
        } else {
            gl.disable(WebGl2RenderingContext::STENCIL_TEST);
        }

        self.current_depth_stencil = Some(*state);
    }

    pub fn set_rasterizer_state(&mut self, gl: &WebGl2RenderingContext, state: &RasterizerState) {
        if self.current_rasterizer.as_ref() == Some(state) {
            return;
        }

        // Culling
        match state.cull_mode {
            CullMode::None => gl.disable(WebGl2RenderingContext::CULL_FACE),
            CullMode::Front => {
                gl.enable(WebGl2RenderingContext::CULL_FACE);
                gl.cull_face(WebGl2RenderingContext::FRONT);
            }
            CullMode::Back => {
                gl.enable(WebGl2RenderingContext::CULL_FACE);
                gl.cull_face(WebGl2RenderingContext::BACK);
            }
        }

        // Scissor test
        if state.scissor_test_enabled {
            gl.enable(WebGl2RenderingContext::SCISSOR_TEST);
        } else {
            gl.disable(WebGl2RenderingContext::SCISSOR_TEST);
        }

        // Polygon offset (depth bias)
        if state.depth_bias != 0.0 || state.depth_bias_slope_scale != 0.0 {
            gl.enable(WebGl2RenderingContext::POLYGON_OFFSET_FILL);
            gl.polygon_offset(state.depth_bias_slope_scale, state.depth_bias);
        } else {
            gl.disable(WebGl2RenderingContext::POLYGON_OFFSET_FILL);
        }

        self.current_rasterizer = Some(*state);
    }

    pub fn set_viewport(&mut self, gl: &WebGl2RenderingContext, x: i32, y: i32, width: i32, height: i32) {
        let viewport = (x, y, width, height);
        if self.current_viewport == Some(viewport) {
            return;
        }
        gl.viewport(x, y, width, height);
        self.current_viewport = Some(viewport);
    }

    pub fn set_scissor(&mut self, gl: &WebGl2RenderingContext, x: i32, y: i32, width: i32, height: i32) {
        let scissor = (x, y, width, height);
        if self.current_scissor == Some(scissor) {
            return;
        }
        gl.scissor(x, y, width, height);
        self.current_scissor = Some(scissor);
    }

    pub fn reset(&mut self) {
        self.current_blend = None;
        self.current_depth_stencil = None;
        self.current_rasterizer = None;
        self.current_viewport = None;
        self.current_scissor = None;
    }
}

fn blend_factor_to_gl(factor: BlendFactor) -> u32 {
    match factor {
        BlendFactor::Zero => WebGl2RenderingContext::ZERO,
        BlendFactor::One => WebGl2RenderingContext::ONE,
        BlendFactor::SrcColor => WebGl2RenderingContext::SRC_COLOR,
        BlendFactor::OneMinusSrcColor => WebGl2RenderingContext::ONE_MINUS_SRC_COLOR,
        BlendFactor::DstColor => WebGl2RenderingContext::DST_COLOR,
        BlendFactor::OneMinusDstColor => WebGl2RenderingContext::ONE_MINUS_DST_COLOR,
        BlendFactor::SrcAlpha => WebGl2RenderingContext::SRC_ALPHA,
        BlendFactor::OneMinusSrcAlpha => WebGl2RenderingContext::ONE_MINUS_SRC_ALPHA,
        BlendFactor::DstAlpha => WebGl2RenderingContext::DST_ALPHA,
        BlendFactor::OneMinusDstAlpha => WebGl2RenderingContext::ONE_MINUS_DST_ALPHA,
        BlendFactor::ConstantColor => WebGl2RenderingContext::CONSTANT_COLOR,
        BlendFactor::OneMinusConstantColor => WebGl2RenderingContext::ONE_MINUS_CONSTANT_COLOR,
        BlendFactor::ConstantAlpha => WebGl2RenderingContext::CONSTANT_ALPHA,
        BlendFactor::OneMinusConstantAlpha => WebGl2RenderingContext::ONE_MINUS_CONSTANT_ALPHA,
        BlendFactor::SrcAlphaSaturate => WebGl2RenderingContext::SRC_ALPHA_SATURATE,
    }
}

fn blend_op_to_gl(op: BlendOp) -> u32 {
    match op {
        BlendOp::Add => WebGl2RenderingContext::FUNC_ADD,
        BlendOp::Subtract => WebGl2RenderingContext::FUNC_SUBTRACT,
        BlendOp::ReverseSubtract => WebGl2RenderingContext::FUNC_REVERSE_SUBTRACT,
        BlendOp::Min => WebGl2RenderingContext::MIN,
        BlendOp::Max => WebGl2RenderingContext::MAX,
    }
}

fn compare_func_to_gl(func: CompareFunc) -> u32 {
    match func {
        CompareFunc::Never => WebGl2RenderingContext::NEVER,
        CompareFunc::Less => WebGl2RenderingContext::LESS,
        CompareFunc::Equal => WebGl2RenderingContext::EQUAL,
        CompareFunc::LessEqual => WebGl2RenderingContext::LEQUAL,
        CompareFunc::Greater => WebGl2RenderingContext::GREATER,
        CompareFunc::NotEqual => WebGl2RenderingContext::NOTEQUAL,
        CompareFunc::GreaterEqual => WebGl2RenderingContext::GEQUAL,
        CompareFunc::Always => WebGl2RenderingContext::ALWAYS,
    }
}