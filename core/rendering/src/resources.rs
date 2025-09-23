//! Resource types for the rendering system

use serde::{Serialize, Deserialize};
use crate::types::{ResourceId, Vec2, Vec3, Color};

// Render target resources
#[cfg(feature = "targets")]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RenderTargetInfo {
    pub id: ResourceId,
    pub width: u32,
    pub height: u32,
    pub format: String,  // "RGBA8", "RGBA16F", etc. - backend-agnostic
    pub samples: u32,
    pub has_depth: bool,
    pub has_stencil: bool,
    pub is_default: bool,
}

#[cfg(feature = "targets")]
impl Default for RenderTargetInfo {
    fn default() -> Self {
        Self {
            id: 0,
            width: 1920,
            height: 1080,
            format: "RGBA8".to_string(),
            samples: 1,
            has_depth: true,
            has_stencil: false,
            is_default: true,
        }
    }
}

// Shader resources
#[cfg(feature = "shaders")]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ShaderInfo {
    pub id: ResourceId,
    pub stage: ShaderStage,
    pub source_type: ShaderSourceType,
    pub entry_point: String,
}

#[cfg(feature = "shaders")]
#[derive(Clone, Debug, Copy, Serialize, Deserialize)]
pub enum ShaderStage {
    Vertex,
    Fragment,
    Compute,
    Geometry,
    TessellationControl,
    TessellationEvaluation,
}

#[cfg(feature = "shaders")]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ShaderSourceType {
    GLSL,
    HLSL,
    WGSL,
    SPIRV,
    MSL,
}

// Texture resources
#[cfg(feature = "textures")]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TextureInfo {
    pub id: ResourceId,
    pub width: u32,
    pub height: u32,
    pub depth: u32,
    pub format: TextureFormat,
    pub usage: TextureUsage,
    pub mip_levels: u32,
    pub array_layers: u32,
    pub sample_count: u32,
}

#[cfg(feature = "textures")]
#[derive(Clone, Debug, Copy, Serialize, Deserialize)]
pub enum TextureFormat {
    // 8-bit formats
    R8,
    RG8,
    RGB8,
    RGBA8,

    // 16-bit formats
    R16F,
    RG16F,
    RGB16F,
    RGBA16F,

    // 32-bit formats
    R32F,
    RG32F,
    RGB32F,
    RGBA32F,

    // Special formats
    Depth24,
    Depth32F,
    Depth24Stencil8,
    SRGBA8,

    // Compressed formats
    BC1,
    BC2,
    BC3,
    BC4,
    BC5,
    BC6H,
    BC7,
    ETC2_RGB,
    ETC2_RGBA,
    ASTC_4x4,
    ASTC_8x8,
}

#[cfg(feature = "textures")]
#[derive(Clone, Debug, Copy, Serialize, Deserialize)]
pub struct TextureUsage {
    pub sample: bool,
    pub render_target: bool,
    pub storage: bool,
    pub copy_src: bool,
    pub copy_dst: bool,
}

#[cfg(feature = "textures")]
impl Default for TextureUsage {
    fn default() -> Self {
        Self {
            sample: true,
            render_target: false,
            storage: false,
            copy_src: true,
            copy_dst: true,
        }
    }
}

#[cfg(feature = "textures")]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TextureRegion {
    pub x: u32,
    pub y: u32,
    pub z: u32,
    pub width: u32,
    pub height: u32,
    pub depth: u32,
    pub mip_level: u32,
    pub array_layer: u32,
}

// Buffer resources
#[cfg(feature = "buffers")]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BufferInfo {
    pub id: ResourceId,
    pub size: usize,
    pub usage: BufferUsage,
    pub mapped_at_creation: bool,
}

#[cfg(feature = "buffers")]
#[derive(Clone, Debug, Copy, Serialize, Deserialize)]
pub struct BufferUsage {
    pub vertex: bool,
    pub index: bool,
    pub uniform: bool,
    pub storage: bool,
    pub indirect: bool,
    pub copy_src: bool,
    pub copy_dst: bool,
}

#[cfg(feature = "buffers")]
impl Default for BufferUsage {
    fn default() -> Self {
        Self {
            vertex: false,
            index: false,
            uniform: false,
            storage: false,
            indirect: false,
            copy_src: true,
            copy_dst: true,
        }
    }
}

#[cfg(feature = "buffers")]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct VertexLayout {
    pub stride: u32,
    pub attributes: Vec<VertexAttribute>,
}

#[cfg(feature = "buffers")]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct VertexAttribute {
    pub location: u32,
    pub offset: u32,
    pub format: VertexFormat,
    pub semantic: String,  // "POSITION", "NORMAL", "TEXCOORD0", etc.
}

#[cfg(feature = "buffers")]
#[derive(Clone, Debug, Copy, Serialize, Deserialize)]
pub enum VertexFormat {
    Float,
    Float2,
    Float3,
    Float4,
    Int,
    Int2,
    Int3,
    Int4,
    UInt,
    UInt2,
    UInt3,
    UInt4,
    Byte4,
    UByte4,
    Short2,
    Short4,
    UShort2,
    UShort4,
}

// Uniform buffer resources
#[cfg(feature = "uniforms")]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UniformBufferInfo {
    pub id: ResourceId,
    pub size: usize,
    pub layout: UniformLayout,
}

#[cfg(feature = "uniforms")]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UniformLayout {
    pub bindings: Vec<UniformBinding>,
}

#[cfg(feature = "uniforms")]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UniformBinding {
    pub name: String,
    pub offset: u32,
    pub size: u32,
    pub uniform_type: UniformType,
}

#[cfg(feature = "uniforms")]
#[derive(Clone, Debug, Copy, Serialize, Deserialize)]
pub enum UniformType {
    Float,
    Float2,
    Float3,
    Float4,
    Int,
    Int2,
    Int3,
    Int4,
    Mat3,
    Mat4,
    Sampler2D,
    SamplerCube,
}

// Sampler resources
#[cfg(feature = "samplers")]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SamplerInfo {
    pub id: ResourceId,
    pub min_filter: FilterMode,
    pub mag_filter: FilterMode,
    pub mip_filter: FilterMode,
    pub wrap_u: WrapMode,
    pub wrap_v: WrapMode,
    pub wrap_w: WrapMode,
    pub anisotropy: f32,
    pub compare_func: Option<CompareFunc>,
}

#[cfg(feature = "samplers")]
#[derive(Clone, Debug, Copy, Serialize, Deserialize)]
pub enum FilterMode {
    Nearest,
    Linear,
}

#[cfg(feature = "samplers")]
#[derive(Clone, Debug, Copy, Serialize, Deserialize)]
pub enum WrapMode {
    Repeat,
    MirrorRepeat,
    ClampToEdge,
    ClampToBorder,
}

#[cfg(feature = "samplers")]
#[derive(Clone, Debug, Copy, Serialize, Deserialize)]
pub enum CompareFunc {
    Never,
    Less,
    LessEqual,
    Greater,
    GreaterEqual,
    Equal,
    NotEqual,
    Always,
}

// Pipeline resources
#[cfg(feature = "pipelines")]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PipelineInfo {
    pub id: ResourceId,
    pub vertex_shader: ResourceId,
    pub fragment_shader: Option<ResourceId>,
    pub vertex_layout: VertexLayout,
    pub blend_state: BlendState,
    pub depth_stencil_state: DepthStencilState,
    pub rasterizer_state: RasterizerState,
    pub primitive_type: PrimitiveType,
}

#[cfg(feature = "pipelines")]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BlendState {
    pub enabled: bool,
    pub color_src: BlendFactor,
    pub color_dst: BlendFactor,
    pub color_op: BlendOp,
    pub alpha_src: BlendFactor,
    pub alpha_dst: BlendFactor,
    pub alpha_op: BlendOp,
}

#[cfg(feature = "pipelines")]
impl Default for BlendState {
    fn default() -> Self {
        Self {
            enabled: false,
            color_src: BlendFactor::One,
            color_dst: BlendFactor::Zero,
            color_op: BlendOp::Add,
            alpha_src: BlendFactor::One,
            alpha_dst: BlendFactor::Zero,
            alpha_op: BlendOp::Add,
        }
    }
}

#[cfg(feature = "pipelines")]
#[derive(Clone, Debug, Copy, Serialize, Deserialize)]
pub enum BlendFactor {
    Zero,
    One,
    SrcColor,
    OneMinusSrcColor,
    DstColor,
    OneMinusDstColor,
    SrcAlpha,
    OneMinusSrcAlpha,
    DstAlpha,
    OneMinusDstAlpha,
    ConstantColor,
    OneMinusConstantColor,
    ConstantAlpha,
    OneMinusConstantAlpha,
    SrcAlphaSaturate,
}

#[cfg(feature = "pipelines")]
#[derive(Clone, Debug, Copy, Serialize, Deserialize)]
pub enum BlendOp {
    Add,
    Subtract,
    ReverseSubtract,
    Min,
    Max,
}

#[cfg(feature = "pipelines")]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DepthStencilState {
    pub depth_test_enabled: bool,
    pub depth_write_enabled: bool,
    pub depth_func: CompareFunc,
    pub stencil_test_enabled: bool,
    pub stencil_read_mask: u32,
    pub stencil_write_mask: u32,
    pub stencil_front: StencilState,
    pub stencil_back: StencilState,
}

#[cfg(feature = "pipelines")]
impl Default for DepthStencilState {
    fn default() -> Self {
        Self {
            depth_test_enabled: true,
            depth_write_enabled: true,
            depth_func: CompareFunc::Less,
            stencil_test_enabled: false,
            stencil_read_mask: 0xFF,
            stencil_write_mask: 0xFF,
            stencil_front: StencilState::default(),
            stencil_back: StencilState::default(),
        }
    }
}

#[cfg(feature = "pipelines")]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StencilState {
    pub fail: StencilOp,
    pub depth_fail: StencilOp,
    pub pass: StencilOp,
    pub func: CompareFunc,
    pub reference: u32,
}

#[cfg(feature = "pipelines")]
impl Default for StencilState {
    fn default() -> Self {
        Self {
            fail: StencilOp::Keep,
            depth_fail: StencilOp::Keep,
            pass: StencilOp::Keep,
            func: CompareFunc::Always,
            reference: 0,
        }
    }
}

#[cfg(feature = "pipelines")]
#[derive(Clone, Debug, Copy, Serialize, Deserialize)]
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

#[cfg(feature = "pipelines")]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RasterizerState {
    pub cull_mode: CullMode,
    pub front_face: FrontFace,
    pub polygon_mode: PolygonMode,
    pub depth_bias: f32,
    pub depth_bias_slope_scale: f32,
    pub depth_bias_clamp: f32,
}

#[cfg(feature = "pipelines")]
impl Default for RasterizerState {
    fn default() -> Self {
        Self {
            cull_mode: CullMode::Back,
            front_face: FrontFace::CCW,
            polygon_mode: PolygonMode::Fill,
            depth_bias: 0.0,
            depth_bias_slope_scale: 0.0,
            depth_bias_clamp: 0.0,
        }
    }
}

#[cfg(feature = "pipelines")]
#[derive(Clone, Debug, Copy, Serialize, Deserialize)]
pub enum CullMode {
    None,
    Front,
    Back,
}

#[cfg(feature = "pipelines")]
#[derive(Clone, Debug, Copy, Serialize, Deserialize)]
pub enum FrontFace {
    CCW,
    CW,
}

#[cfg(feature = "pipelines")]
#[derive(Clone, Debug, Copy, Serialize, Deserialize)]
pub enum PolygonMode {
    Fill,
    Line,
    Point,
}

#[cfg(feature = "pipelines")]
#[derive(Clone, Debug, Copy, Serialize, Deserialize)]
pub enum PrimitiveType {
    Points,
    Lines,
    LineStrip,
    Triangles,
    TriangleStrip,
    TriangleFan,
}

// Render pass resources
#[cfg(feature = "passes")]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RenderPassInfo {
    pub id: ResourceId,
    pub attachments: Vec<Attachment>,
    pub subpasses: Vec<Subpass>,
}

#[cfg(feature = "passes")]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Attachment {
    pub format: String,
    pub samples: u32,
    pub load_op: LoadOp,
    pub store_op: StoreOp,
    pub initial_layout: String,
    pub final_layout: String,
}

#[cfg(feature = "passes")]
#[derive(Clone, Debug, Copy, Serialize, Deserialize)]
pub enum LoadOp {
    Load,
    Clear,
    DontCare,
}

#[cfg(feature = "passes")]
#[derive(Clone, Debug, Copy, Serialize, Deserialize)]
pub enum StoreOp {
    Store,
    DontCare,
}

#[cfg(feature = "passes")]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Subpass {
    pub color_attachments: Vec<u32>,
    pub depth_attachment: Option<u32>,
    pub input_attachments: Vec<u32>,
}

// Query resources
#[cfg(feature = "queries")]
#[derive(Clone, Debug, Copy, Serialize, Deserialize)]
pub enum QueryType {
    Occlusion,
    PipelineStatistics,
    Timestamp,
}

#[cfg(feature = "queries")]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct QueryInfo {
    pub id: ResourceId,
    pub query_type: QueryType,
    pub index: u32,
}

// Command buffer resources
#[cfg(feature = "commands")]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CommandBufferInfo {
    pub id: ResourceId,
    pub state: CommandBufferState,
    pub level: CommandBufferLevel,
}

#[cfg(feature = "commands")]
#[derive(Clone, Debug, Copy, Serialize, Deserialize)]
pub enum CommandBufferState {
    Initial,
    Recording,
    Executable,
    Invalid,
}

#[cfg(feature = "commands")]
#[derive(Clone, Debug, Copy, Serialize, Deserialize)]
pub enum CommandBufferLevel {
    Primary,
    Secondary,
}

// Use directives for pipelines that need buffers
#[cfg(all(feature = "pipelines", not(feature = "buffers")))]
pub use placeholder::{VertexLayout, VertexAttribute, VertexFormat};

#[cfg(all(feature = "pipelines", not(feature = "buffers")))]
mod placeholder {
    use serde::{Serialize, Deserialize};

    #[derive(Clone, Debug, Serialize, Deserialize)]
    pub struct VertexLayout {
        pub stride: u32,
        pub attributes: Vec<VertexAttribute>,
    }

    #[derive(Clone, Debug, Serialize, Deserialize)]
    pub struct VertexAttribute {
        pub location: u32,
        pub offset: u32,
        pub format: VertexFormat,
        pub semantic: String,
    }

    #[derive(Clone, Debug, Copy, Serialize, Deserialize)]
    pub enum VertexFormat {
        Float3,
        Float2,
        Float4,
    }
}

// Use directives for pipelines/samplers that need CompareFunc
#[cfg(all(feature = "pipelines", not(feature = "samplers")))]
pub use compare_placeholder::CompareFunc;

#[cfg(all(feature = "samplers", not(feature = "pipelines")))]
pub use compare_placeholder::CompareFunc;

#[cfg(all(not(feature = "samplers"), not(feature = "pipelines")))]
mod compare_placeholder {
    use serde::{Serialize, Deserialize};

    #[derive(Clone, Debug, Copy, Serialize, Deserialize)]
    pub enum CompareFunc {
        Never,
        Less,
        LessEqual,
        Greater,
        GreaterEqual,
        Equal,
        NotEqual,
        Always,
    }
}