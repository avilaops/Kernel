# Avila Renderer

High-performance rendering engine with modern graphics API abstraction.

## ⚠️ Status: Early Development

**AvilaRenderer** is the graphics core of the Avila project. Currently in **prototype/skeleton** phase with API design complete and backend stubs.

### Current State

**✅ Implemented:**
- **Complete GPU abstraction API** - Backend-agnostic types for textures, buffers, shaders, pipelines
- **Command recording system** - Type-safe command list with render passes, draws, state management
- **Frame graph system** - Automatic resource management and render pass scheduling
- **Resource management** - Slot-based allocation with handle-based API
- **Clean architecture** - Separation between API (what) and backend (how)

**🚧 In Progress:**
- **Backend implementations** - Native graphics API wrappers (Vulkan/D3D12/Metal/OpenGL)

**📋 Planned:**
- Vulkan backend (primary target)
- GPU memory allocator
- Shader compilation pipeline (GLSL → SPIR-V)
- Material system
- Scene rendering
- Post-processing effects
- Debug rendering utilities

## Architecture

The renderer is structured in layers:

```
┌─────────────────────────────────────┐
│      Engine / Game Code             │  ← Uses only gfx::api types
├─────────────────────────────────────┤
│    Frame Graph (Render Graph)       │  ← Automatic resource management
├─────────────────────────────────────┤
│    GPU Abstraction (gfx::api)       │  ← Backend-agnostic API
├─────────────────────────────────────┤
│    Backend (gfx::backend)           │  ← Translates to native API
├─────────────────────────────────────┤
│  Native APIs (Vulkan/D3D12/Metal)   │  ← Platform-specific implementation
└─────────────────────────────────────┘
```

### Key Design Principles

1. **Backend Agnostic**: Engine code never sees native API types
2. **Type Safety**: Rust's type system prevents common GPU programming errors
3. **Zero Cost**: Thin abstractions compile down to direct API calls
4. **Modern Design**: Inspired by Vulkan, D3D12, Metal architecture
5. **Explicit Control**: No hidden state, all operations explicit

## API Overview

### Resource Creation

```rust
use avila_renderer::gfx::*;

// Create device
let config = RendererConfig::default();
let mut device = create_device(config);

// Create texture
let texture = device.create_texture(&TextureDesc::new_2d(
    1280, 720,
    TextureFormat::Rgba8,
    TextureUsage::COLOR_ATTACHMENT | TextureUsage::SAMPLED,
));

// Create buffer with initial data
let vertices = [...];
let buffer = device.create_buffer(
    &BufferDesc::vertex(vertices.len() * 4),
    Some(bytemuck::cast_slice(&vertices)),
);

// Create shader
let shader = device.create_shader(&ShaderDesc {
    stage: ShaderStage::Vertex,
    entry_point: "main".to_string(),
    code: spirv_bytecode.to_vec(),
});

// Create pipeline
let pipeline = device.create_pipeline(&PipelineDesc {
    vertex_shader: vs_handle,
    fragment_shader: fs_handle,
    vertex_layout: VertexLayout { ... },
    topology: PrimitiveTopology::TriangleList,
    rasterizer: RasterizerState::default(),
    depth_stencil: DepthStencilState::default(),
    blend_states: vec![BlendState::ALPHA_BLENDING],
    color_formats: vec![TextureFormat::Rgba8],
    depth_format: Some(TextureFormat::Depth24),
});
```

### Command Recording

```rust
// Begin frame
let mut cmd = device.begin_frame();

// Begin render pass
cmd.begin_render_pass(RenderPassDesc {
    color_attachments: vec![
        ColorAttachment {
            texture: color_target,
            clear: Some(ClearColor::BLACK),
        }
    ],
    depth_attachment: Some(DepthAttachment {
        texture: depth_target,
        clear: Some(ClearDepthStencil::default()),
    }),
});

// Set state
cmd.bind_pipeline(pipeline);
cmd.set_viewport(Viewport { x: 0.0, y: 0.0, width: 1280.0, height: 720.0, min_depth: 0.0, max_depth: 1.0 });
cmd.set_scissor(Rect { x: 0, y: 0, width: 1280, height: 720 });

// Bind resources
cmd.bind_vertex_buffer(0, vertex_buffer, 0);
cmd.bind_index_buffer(index_buffer, 0, IndexType::UInt32);

// Draw
cmd.draw_indexed(index_count, 1, 0, 0, 0);

// End render pass
cmd.end_render_pass();

// Submit and present
device.submit(cmd);
device.present();
```

### Frame Graph (Render Graph)

```rust
use avila_renderer::gfx::*;

let mut fg = FrameGraphBuilder::new();

// Import swapchain
let backbuffer = fg.import_texture("backbuffer", device.get_swapchain_texture());

// Create transient resources
let depth = fg.create_texture(
    "depth",
    TextureDesc::new_2d(1280, 720, TextureFormat::Depth24, TextureUsage::DEPTH_ATTACHMENT),
);

let shadow_map = fg.create_texture(
    "shadow_map",
    TextureDesc::new_2d(2048, 2048, TextureFormat::Depth32f,
        TextureUsage::DEPTH_ATTACHMENT | TextureUsage::SAMPLED),
);

// Shadow pass
fg.add_pass(
    "shadow_pass",
    |pass| {
        pass.write(&shadow_map);
    },
    Box::new(|cmd, resources| {
        let shadow = resources.get_texture("shadow_map");
        // Render shadows to shadow_map...
    }),
);

// Main pass (reads shadow_map, writes to backbuffer)
fg.add_pass(
    "main_pass",
    |pass| {
        pass.read(&shadow_map);
        pass.write(&backbuffer);
        pass.write(&depth);
    },
    Box::new(|cmd, resources| {
        let shadow = resources.get_texture("shadow_map");
        let color = resources.get_texture("backbuffer");
        let depth = resources.get_texture("depth");
        // Render scene with shadows...
    }),
);

// Compile and execute
let compiled = fg.compile();
compiled.execute(&mut device);
```

The frame graph automatically:
- Allocates transient resources
- Schedules passes in correct order
- Inserts barriers for synchronization
- Deallocates resources after use
- Culls unused passes (future)

## Module Structure

```
avila-renderer/
├── src/
│   ├── lib.rs              # Library entry point
│   └── gfx/
│       ├── mod.rs          # Graphics module root
│       ├── api.rs          # GPU abstraction (400+ lines)
│       ├── backend/
│       │   └── mod.rs      # Backend implementation
│       └── framegraph.rs   # Frame graph system
└── Cargo.toml
```

## Type System

### Handles (Opaque, Type-Safe)
- `TextureHandle` - References a GPU texture
- `BufferHandle` - References a GPU buffer
- `ShaderHandle` - References a compiled shader
- `PipelineHandle` - References a graphics/compute pipeline

### Descriptors (Create Resources)
- `TextureDesc` - Texture dimensions, format, usage
- `BufferDesc` - Buffer size, usage, CPU visibility
- `ShaderDesc` - Shader stage, entry point, SPIR-V code
- `PipelineDesc` - Complete graphics pipeline state

### Enums (Type-Safe Configuration)
- `TextureFormat` - Pixel formats (RGBA8, Depth24, BC7, etc.)
- `BufferUsage` - Buffer types (Vertex, Index, Uniform, Storage)
- `ShaderStage` - Pipeline stages (Vertex, Fragment, Compute)
- `PrimitiveTopology` - Geometry type (TriangleList, LineStrip, etc.)
- `CompareFunction` - Depth/stencil comparison
- `BlendFactor` / `BlendOp` - Blending configuration

## Future Backend Implementations

### Vulkan (Primary Target)
```toml
[dependencies]
ash = "0.38"              # Vulkan bindings
gpu-allocator = "0.26"    # GPU memory management
```

### Direct3D 12
```toml
[dependencies]
windows = { version = "0.58", features = ["Win32_Graphics_Direct3D12"] }
```

### Metal (macOS/iOS)
```toml
[dependencies]
metal = "0.27"
objc = "0.2"
```

### OpenGL (Fallback)
```toml
[dependencies]
glow = "0.13"
```

## Testing

```bash
# Build renderer
cargo build -p avila-renderer

# Run tests (when implemented)
cargo test -p avila-renderer

# Check API without backend
cargo check -p avila-renderer
```

## Design Inspiration

- **Vulkan** - Explicit API design, resource barriers, command buffers
- **D3D12** - Resource binding model, descriptor heaps
- **Metal** - Clean API, shader reflection
- **Frostbite FrameGraph** - Automatic resource management
- **wgpu** - Rust-native graphics API design
- **bgfx** - Multi-backend abstraction architecture

## Contributing

See workspace root for contribution guidelines, CLA, and Code of Conduct.

## License

Dual-licensed under MIT OR Apache-2.0 (same as Avila workspace).

---

**AvilaRenderer** - Part of the Avila project
