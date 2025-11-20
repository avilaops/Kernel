// Copyright (c) 2025 Avila Contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! GPU abstraction API - how Avila sees the GPU
//!
//! This module defines the core graphics API that is backend-agnostic.
//! All engine systems (scene, materials, rendering passes) only see these types.

use std::fmt;

// ============================================================================
// Texture Types
// ============================================================================

/// Texture pixel format
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum TextureFormat {
    // Color formats
    Rgba8,
    Rgba8Srgb,
    Rgba16f,
    Rgba32f,
    Bgra8,

    // Depth/Stencil formats
    Depth24,
    Depth32f,
    Depth24Stencil8,

    // Compressed formats
    Bc1, // DXT1
    Bc3, // DXT5
    Bc7,
}

impl TextureFormat {
    /// Returns the size in bytes per pixel (or block for compressed formats)
    pub fn bytes_per_pixel(&self) -> u32 {
        match self {
            TextureFormat::Rgba8 | TextureFormat::Rgba8Srgb | TextureFormat::Bgra8 => 4,
            TextureFormat::Rgba16f => 8,
            TextureFormat::Rgba32f => 16,
            TextureFormat::Depth24 | TextureFormat::Depth32f => 4,
            TextureFormat::Depth24Stencil8 => 4,
            TextureFormat::Bc1 => 8, // 4x4 block = 8 bytes
            TextureFormat::Bc3 | TextureFormat::Bc7 => 16, // 4x4 block = 16 bytes
        }
    }

    pub fn is_depth(&self) -> bool {
        matches!(
            self,
            TextureFormat::Depth24 | TextureFormat::Depth32f | TextureFormat::Depth24Stencil8
        )
    }

    pub fn is_compressed(&self) -> bool {
        matches!(
            self,
            TextureFormat::Bc1 | TextureFormat::Bc3 | TextureFormat::Bc7
        )
    }
}

/// Texture usage flags (can be combined)
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TextureUsage(u32);

impl TextureUsage {
    pub const NONE: Self = Self(0);
    pub const COLOR_ATTACHMENT: Self = Self(0b0001);
    pub const DEPTH_ATTACHMENT: Self = Self(0b0010);
    pub const SAMPLED: Self = Self(0b0100);
    pub const STORAGE: Self = Self(0b1000);
    pub const TRANSFER_SRC: Self = Self(0b0001_0000);
    pub const TRANSFER_DST: Self = Self(0b0010_0000);

    pub const fn contains(&self, other: Self) -> bool {
        (self.0 & other.0) == other.0
    }

    pub const fn union(&self, other: Self) -> Self {
        Self(self.0 | other.0)
    }
}

impl std::ops::BitOr for TextureUsage {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self {
        Self(self.0 | rhs.0)
    }
}

/// Texture dimension type
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TextureDimension {
    D1,
    D2,
    D3,
    Cube,
}

/// Texture description for creation
#[derive(Clone, Debug)]
pub struct TextureDesc {
    pub width: u32,
    pub height: u32,
    pub depth: u32,
    pub mip_levels: u32,
    pub array_layers: u32,
    pub dimension: TextureDimension,
    pub format: TextureFormat,
    pub usage: TextureUsage,
    pub samples: u32, // For MSAA (1 = no MSAA)
}

impl TextureDesc {
    pub fn new_2d(width: u32, height: u32, format: TextureFormat, usage: TextureUsage) -> Self {
        Self {
            width,
            height,
            depth: 1,
            mip_levels: 1,
            array_layers: 1,
            dimension: TextureDimension::D2,
            format,
            usage,
            samples: 1,
        }
    }

    pub fn with_mips(mut self, mip_levels: u32) -> Self {
        self.mip_levels = mip_levels;
        self
    }

    pub fn with_msaa(mut self, samples: u32) -> Self {
        self.samples = samples;
        self
    }
}

/// Opaque handle to a GPU texture
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct TextureHandle(pub u32);

impl TextureHandle {
    pub const INVALID: Self = Self(u32::MAX);
}

// ============================================================================
// Buffer Types
// ============================================================================

/// Buffer usage type
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BufferUsage {
    Vertex,
    Index,
    Uniform,
    Storage,
    Indirect,
    TransferSrc,
    TransferDst,
}

/// Buffer description for creation
#[derive(Clone, Debug)]
pub struct BufferDesc {
    pub size: usize,
    pub usage: BufferUsage,
    pub cpu_visible: bool, // Can be mapped for CPU access
}

impl BufferDesc {
    pub fn vertex(size: usize) -> Self {
        Self {
            size,
            usage: BufferUsage::Vertex,
            cpu_visible: false,
        }
    }

    pub fn index(size: usize) -> Self {
        Self {
            size,
            usage: BufferUsage::Index,
            cpu_visible: false,
        }
    }

    pub fn uniform(size: usize) -> Self {
        Self {
            size,
            usage: BufferUsage::Uniform,
            cpu_visible: true, // Usually updated frequently
        }
    }

    pub fn storage(size: usize) -> Self {
        Self {
            size,
            usage: BufferUsage::Storage,
            cpu_visible: false,
        }
    }
}

/// Opaque handle to a GPU buffer
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct BufferHandle(pub u32);

impl BufferHandle {
    pub const INVALID: Self = Self(u32::MAX);
}

// ============================================================================
// Shader Types
// ============================================================================

/// Shader stage type
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ShaderStage {
    Vertex,
    Fragment,
    Compute,
    Geometry,
    TessControl,
    TessEvaluation,
}

/// Shader module description
#[derive(Clone, Debug)]
pub struct ShaderDesc {
    pub stage: ShaderStage,
    pub entry_point: String,
    pub code: Vec<u8>, // SPIR-V bytecode
}

/// Opaque handle to a shader module
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ShaderHandle(pub u32);

impl ShaderHandle {
    pub const INVALID: Self = Self(u32::MAX);
}

// ============================================================================
// Pipeline Types
// ============================================================================

/// Vertex attribute format
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VertexFormat {
    Float,
    Float2,
    Float3,
    Float4,
    UInt,
    UInt2,
    UInt3,
    UInt4,
}

impl VertexFormat {
    pub fn size(&self) -> u32 {
        match self {
            VertexFormat::Float | VertexFormat::UInt => 4,
            VertexFormat::Float2 | VertexFormat::UInt2 => 8,
            VertexFormat::Float3 | VertexFormat::UInt3 => 12,
            VertexFormat::Float4 | VertexFormat::UInt4 => 16,
        }
    }
}

/// Vertex attribute description
#[derive(Clone, Debug)]
pub struct VertexAttribute {
    pub format: VertexFormat,
    pub offset: u32,
    pub location: u32,
}

/// Vertex buffer layout
#[derive(Clone, Debug)]
pub struct VertexLayout {
    pub stride: u32,
    pub attributes: Vec<VertexAttribute>,
}

/// Primitive topology
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PrimitiveTopology {
    TriangleList,
    TriangleStrip,
    LineList,
    LineStrip,
    PointList,
}

/// Comparison function for depth/stencil tests
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CompareFunction {
    Never,
    Less,
    Equal,
    LessEqual,
    Greater,
    NotEqual,
    GreaterEqual,
    Always,
}

/// Blend factor
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
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
}

/// Blend operation
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BlendOp {
    Add,
    Subtract,
    ReverseSubtract,
    Min,
    Max,
}

/// Blend state for a color attachment
#[derive(Clone, Copy, Debug)]
pub struct BlendState {
    pub enabled: bool,
    pub src_color: BlendFactor,
    pub dst_color: BlendFactor,
    pub color_op: BlendOp,
    pub src_alpha: BlendFactor,
    pub dst_alpha: BlendFactor,
    pub alpha_op: BlendOp,
}

impl Default for BlendState {
    fn default() -> Self {
        Self {
            enabled: false,
            src_color: BlendFactor::One,
            dst_color: BlendFactor::Zero,
            color_op: BlendOp::Add,
            src_alpha: BlendFactor::One,
            dst_alpha: BlendFactor::Zero,
            alpha_op: BlendOp::Add,
        }
    }
}

impl BlendState {
    pub const ALPHA_BLENDING: Self = Self {
        enabled: true,
        src_color: BlendFactor::SrcAlpha,
        dst_color: BlendFactor::OneMinusSrcAlpha,
        color_op: BlendOp::Add,
        src_alpha: BlendFactor::One,
        dst_alpha: BlendFactor::Zero,
        alpha_op: BlendOp::Add,
    };
}

/// Depth/stencil state
#[derive(Clone, Copy, Debug)]
pub struct DepthStencilState {
    pub depth_test_enabled: bool,
    pub depth_write_enabled: bool,
    pub depth_compare: CompareFunction,
}

impl Default for DepthStencilState {
    fn default() -> Self {
        Self {
            depth_test_enabled: true,
            depth_write_enabled: true,
            depth_compare: CompareFunction::Less,
        }
    }
}

/// Rasterizer state
#[derive(Clone, Copy, Debug)]
pub struct RasterizerState {
    pub cull_mode: CullMode,
    pub front_face: FrontFace,
    pub polygon_mode: PolygonMode,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CullMode {
    None,
    Front,
    Back,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FrontFace {
    Clockwise,
    CounterClockwise,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PolygonMode {
    Fill,
    Line,
    Point,
}

impl Default for RasterizerState {
    fn default() -> Self {
        Self {
            cull_mode: CullMode::Back,
            front_face: FrontFace::CounterClockwise,
            polygon_mode: PolygonMode::Fill,
        }
    }
}

/// Graphics pipeline description
#[derive(Clone, Debug)]
pub struct PipelineDesc {
    pub vertex_shader: ShaderHandle,
    pub fragment_shader: ShaderHandle,
    pub vertex_layout: VertexLayout,
    pub topology: PrimitiveTopology,
    pub rasterizer: RasterizerState,
    pub depth_stencil: DepthStencilState,
    pub blend_states: Vec<BlendState>,
    pub color_formats: Vec<TextureFormat>,
    pub depth_format: Option<TextureFormat>,
}

/// Opaque handle to a graphics pipeline
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct PipelineHandle(pub u32);

impl PipelineHandle {
    pub const INVALID: Self = Self(u32::MAX);
}

// ============================================================================
// Command Recording
// ============================================================================

/// Viewport for rendering
#[derive(Clone, Copy, Debug)]
pub struct Viewport {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub min_depth: f32,
    pub max_depth: f32,
}

/// Scissor rectangle
#[derive(Clone, Copy, Debug)]
pub struct Rect {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}

/// Clear color value
#[derive(Clone, Copy, Debug)]
pub struct ClearColor {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl ClearColor {
    pub const BLACK: Self = Self {
        r: 0.0,
        g: 0.0,
        b: 0.0,
        a: 1.0,
    };
    pub const WHITE: Self = Self {
        r: 1.0,
        g: 1.0,
        b: 1.0,
        a: 1.0,
    };
}

/// Clear depth/stencil value
#[derive(Clone, Copy, Debug)]
pub struct ClearDepthStencil {
    pub depth: f32,
    pub stencil: u32,
}

impl Default for ClearDepthStencil {
    fn default() -> Self {
        Self {
            depth: 1.0,
            stencil: 0,
        }
    }
}

/// Render pass color attachment
#[derive(Clone, Debug)]
pub struct ColorAttachment {
    pub texture: TextureHandle,
    pub clear: Option<ClearColor>,
}

/// Render pass depth attachment
#[derive(Clone, Debug)]
pub struct DepthAttachment {
    pub texture: TextureHandle,
    pub clear: Option<ClearDepthStencil>,
}

/// Render pass description
#[derive(Clone, Debug)]
pub struct RenderPassDesc {
    pub color_attachments: Vec<ColorAttachment>,
    pub depth_attachment: Option<DepthAttachment>,
}

/// Command list for recording GPU commands
pub struct CommandList {
    // Internal implementation hidden from API users
    pub(crate) commands: Vec<Command>,
}

impl CommandList {
    pub(crate) fn new() -> Self {
        Self {
            commands: Vec::new(),
        }
    }

    /// Begin a render pass
    pub fn begin_render_pass(&mut self, desc: RenderPassDesc) {
        self.commands.push(Command::BeginRenderPass(desc));
    }

    /// End the current render pass
    pub fn end_render_pass(&mut self) {
        self.commands.push(Command::EndRenderPass);
    }

    /// Bind a graphics pipeline
    pub fn bind_pipeline(&mut self, pipeline: PipelineHandle) {
        self.commands.push(Command::BindPipeline(pipeline));
    }

    /// Set viewport
    pub fn set_viewport(&mut self, viewport: Viewport) {
        self.commands.push(Command::SetViewport(viewport));
    }

    /// Set scissor rectangle
    pub fn set_scissor(&mut self, scissor: Rect) {
        self.commands.push(Command::SetScissor(scissor));
    }

    /// Bind vertex buffer
    pub fn bind_vertex_buffer(&mut self, slot: u32, buffer: BufferHandle, offset: u64) {
        self.commands.push(Command::BindVertexBuffer {
            slot,
            buffer,
            offset,
        });
    }

    /// Bind index buffer
    pub fn bind_index_buffer(&mut self, buffer: BufferHandle, offset: u64, index_type: IndexType) {
        self.commands.push(Command::BindIndexBuffer {
            buffer,
            offset,
            index_type,
        });
    }

    /// Draw primitives
    pub fn draw(
        &mut self,
        vertex_count: u32,
        instance_count: u32,
        first_vertex: u32,
        first_instance: u32,
    ) {
        self.commands.push(Command::Draw {
            vertex_count,
            instance_count,
            first_vertex,
            first_instance,
        });
    }

    /// Draw indexed primitives
    pub fn draw_indexed(
        &mut self,
        index_count: u32,
        instance_count: u32,
        first_index: u32,
        vertex_offset: i32,
        first_instance: u32,
    ) {
        self.commands.push(Command::DrawIndexed {
            index_count,
            instance_count,
            first_index,
            vertex_offset,
            first_instance,
        });
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IndexType {
    UInt16,
    UInt32,
}

/// Internal command representation
#[derive(Clone, Debug)]
pub(crate) enum Command {
    BeginRenderPass(RenderPassDesc),
    EndRenderPass,
    BindPipeline(PipelineHandle),
    SetViewport(Viewport),
    SetScissor(Rect),
    BindVertexBuffer {
        slot: u32,
        buffer: BufferHandle,
        offset: u64,
    },
    BindIndexBuffer {
        buffer: BufferHandle,
        offset: u64,
        index_type: IndexType,
    },
    Draw {
        vertex_count: u32,
        instance_count: u32,
        first_vertex: u32,
        first_instance: u32,
    },
    DrawIndexed {
        index_count: u32,
        instance_count: u32,
        first_index: u32,
        vertex_offset: i32,
        first_instance: u32,
    },
}

// ============================================================================
// Renderer Configuration
// ============================================================================

/// Renderer configuration
#[derive(Clone, Debug)]
pub struct RendererConfig {
    pub width: u32,
    pub height: u32,
    pub vsync: bool,
    pub msaa_samples: u32, // 1, 2, 4, 8
    pub hdr: bool,
}

impl Default for RendererConfig {
    fn default() -> Self {
        Self {
            width: 1280,
            height: 720,
            vsync: true,
            msaa_samples: 1,
            hdr: false,
        }
    }
}

// ============================================================================
// Main GPU Device Trait
// ============================================================================

/// Main GPU device abstraction trait
///
/// This is the only interface that engine systems see.
/// Backend implementations provide concrete implementations.
pub trait GpuDevice {
    // Resource creation
    fn create_texture(&mut self, desc: &TextureDesc) -> TextureHandle;
    fn create_buffer(&mut self, desc: &BufferDesc, initial_data: Option<&[u8]>) -> BufferHandle;
    fn create_shader(&mut self, desc: &ShaderDesc) -> ShaderHandle;
    fn create_pipeline(&mut self, desc: &PipelineDesc) -> PipelineHandle;

    // Resource destruction
    fn destroy_texture(&mut self, handle: TextureHandle);
    fn destroy_buffer(&mut self, handle: BufferHandle);
    fn destroy_shader(&mut self, handle: ShaderHandle);
    fn destroy_pipeline(&mut self, handle: PipelineHandle);

    // Buffer operations
    fn update_buffer(&mut self, buffer: BufferHandle, offset: usize, data: &[u8]);
    fn map_buffer(&mut self, buffer: BufferHandle) -> *mut u8;
    fn unmap_buffer(&mut self, buffer: BufferHandle);

    // Command recording and submission
    fn begin_frame(&mut self) -> CommandList;
    fn submit(&mut self, cmd: CommandList);
    fn present(&mut self);

    // Swapchain operations
    fn get_swapchain_texture(&self) -> TextureHandle;
    fn resize(&mut self, width: u32, height: u32);

    // Synchronization
    fn wait_idle(&mut self);
}
