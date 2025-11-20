// Copyright (c) 2025 Avila Contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Backend implementation of the GPU device
//!
//! This module provides a concrete implementation of the GpuDevice trait.
//! It wraps the native graphics API (Vulkan/D3D12/Metal/OpenGL) and translates
//! Avila's API to backend-specific calls.

use crate::gfx::api::*;
use std::collections::HashMap;

/// Backend GPU device implementation
pub struct BackendDevice {
    config: RendererConfig,

    // Resource storage (slot allocators)
    textures: ResourcePool<TextureResource>,
    buffers: ResourcePool<BufferResource>,
    shaders: ResourcePool<ShaderResource>,
    pipelines: ResourcePool<PipelineResource>,

    // Native API handles (todo: implement per backend)
    // For Vulkan: VkInstance, VkDevice, VkQueue, VkSwapchain, etc.
    // For now: stubs
    native_device: NativeDevice,

    // Frame synchronization
    current_frame: u64,
}

impl BackendDevice {
    pub fn new(config: RendererConfig) -> Self {
        let native_device = NativeDevice::create(&config);

        Self {
            config,
            textures: ResourcePool::new(),
            buffers: ResourcePool::new(),
            shaders: ResourcePool::new(),
            pipelines: ResourcePool::new(),
            native_device,
            current_frame: 0,
        }
    }
}

impl GpuDevice for BackendDevice {
    fn create_texture(&mut self, desc: &TextureDesc) -> TextureHandle {
        let native_texture = self.native_device.create_texture_native(desc);

        let resource = TextureResource {
            desc: desc.clone(),
            native: native_texture,
        };

        let id = self.textures.allocate(resource);
        TextureHandle(id)
    }

    fn create_buffer(&mut self, desc: &BufferDesc, initial_data: Option<&[u8]>) -> BufferHandle {
        let native_buffer = self.native_device.create_buffer_native(desc, initial_data);

        let resource = BufferResource {
            desc: desc.clone(),
            native: native_buffer,
        };

        let id = self.buffers.allocate(resource);
        BufferHandle(id)
    }

    fn create_shader(&mut self, desc: &ShaderDesc) -> ShaderHandle {
        let native_shader = self.native_device.create_shader_native(desc);

        let resource = ShaderResource {
            desc: desc.clone(),
            native: native_shader,
        };

        let id = self.shaders.allocate(resource);
        ShaderHandle(id)
    }

    fn create_pipeline(&mut self, desc: &PipelineDesc) -> PipelineHandle {
        let native_pipeline = self
            .native_device
            .create_pipeline_native(desc, &self.shaders);

        let resource = PipelineResource {
            desc: desc.clone(),
            native: native_pipeline,
        };

        let id = self.pipelines.allocate(resource);
        PipelineHandle(id)
    }

    fn destroy_texture(&mut self, handle: TextureHandle) {
        if let Some(resource) = self.textures.free(handle.0) {
            self.native_device.destroy_texture_native(resource.native);
        }
    }

    fn destroy_buffer(&mut self, handle: BufferHandle) {
        if let Some(resource) = self.buffers.free(handle.0) {
            self.native_device.destroy_buffer_native(resource.native);
        }
    }

    fn destroy_shader(&mut self, handle: ShaderHandle) {
        if let Some(resource) = self.shaders.free(handle.0) {
            self.native_device.destroy_shader_native(resource.native);
        }
    }

    fn destroy_pipeline(&mut self, handle: PipelineHandle) {
        if let Some(resource) = self.pipelines.free(handle.0) {
            self.native_device.destroy_pipeline_native(resource.native);
        }
    }

    fn update_buffer(&mut self, buffer: BufferHandle, offset: usize, data: &[u8]) {
        if let Some(resource) = self.buffers.get(buffer.0) {
            self.native_device
                .update_buffer_native(resource.native, offset, data);
        }
    }

    fn map_buffer(&mut self, buffer: BufferHandle) -> *mut u8 {
        if let Some(resource) = self.buffers.get(buffer.0) {
            self.native_device.map_buffer_native(resource.native)
        } else {
            std::ptr::null_mut()
        }
    }

    fn unmap_buffer(&mut self, buffer: BufferHandle) {
        if let Some(resource) = self.buffers.get(buffer.0) {
            self.native_device.unmap_buffer_native(resource.native);
        }
    }

    fn begin_frame(&mut self) -> CommandList {
        self.native_device.begin_frame_native();
        CommandList::new()
    }

    fn submit(&mut self, cmd: CommandList) {
        // Translate Avila commands to native API calls
        for command in cmd.commands {
            match command {
                Command::BeginRenderPass(desc) => {
                    self.native_device
                        .begin_render_pass_native(&desc, &self.textures);
                }
                Command::EndRenderPass => {
                    self.native_device.end_render_pass_native();
                }
                Command::BindPipeline(handle) => {
                    if let Some(resource) = self.pipelines.get(handle.0) {
                        self.native_device.bind_pipeline_native(resource.native);
                    }
                }
                Command::SetViewport(viewport) => {
                    self.native_device.set_viewport_native(&viewport);
                }
                Command::SetScissor(scissor) => {
                    self.native_device.set_scissor_native(&scissor);
                }
                Command::BindVertexBuffer {
                    slot,
                    buffer,
                    offset,
                } => {
                    if let Some(resource) = self.buffers.get(buffer.0) {
                        self.native_device
                            .bind_vertex_buffer_native(slot, resource.native, offset);
                    }
                }
                Command::BindIndexBuffer {
                    buffer,
                    offset,
                    index_type,
                } => {
                    if let Some(resource) = self.buffers.get(buffer.0) {
                        self.native_device.bind_index_buffer_native(
                            resource.native,
                            offset,
                            index_type,
                        );
                    }
                }
                Command::Draw {
                    vertex_count,
                    instance_count,
                    first_vertex,
                    first_instance,
                } => {
                    self.native_device.draw_native(
                        vertex_count,
                        instance_count,
                        first_vertex,
                        first_instance,
                    );
                }
                Command::DrawIndexed {
                    index_count,
                    instance_count,
                    first_index,
                    vertex_offset,
                    first_instance,
                } => {
                    self.native_device.draw_indexed_native(
                        index_count,
                        instance_count,
                        first_index,
                        vertex_offset,
                        first_instance,
                    );
                }
            }
        }
    }

    fn present(&mut self) {
        self.native_device.present_native();
        self.current_frame += 1;
    }

    fn get_swapchain_texture(&self) -> TextureHandle {
        // Return handle to current swapchain image
        // For now: stub
        TextureHandle(0)
    }

    fn resize(&mut self, width: u32, height: u32) {
        self.config.width = width;
        self.config.height = height;
        self.native_device.resize_native(width, height);
    }

    fn wait_idle(&mut self) {
        self.native_device.wait_idle_native();
    }
}

// ============================================================================
// Resource Storage
// ============================================================================

struct TextureResource {
    desc: TextureDesc,
    native: NativeTexture,
}

struct BufferResource {
    desc: BufferDesc,
    native: NativeBuffer,
}

struct ShaderResource {
    desc: ShaderDesc,
    native: NativeShader,
}

struct PipelineResource {
    desc: PipelineDesc,
    native: NativePipeline,
}

/// Generic resource pool with slot allocation
struct ResourcePool<T> {
    resources: HashMap<u32, T>,
    next_id: u32,
    free_list: Vec<u32>,
}

impl<T> ResourcePool<T> {
    fn new() -> Self {
        Self {
            resources: HashMap::new(),
            next_id: 0,
            free_list: Vec::new(),
        }
    }

    fn allocate(&mut self, resource: T) -> u32 {
        let id = if let Some(id) = self.free_list.pop() {
            id
        } else {
            let id = self.next_id;
            self.next_id += 1;
            id
        };

        self.resources.insert(id, resource);
        id
    }

    fn free(&mut self, id: u32) -> Option<T> {
        if let Some(resource) = self.resources.remove(&id) {
            self.free_list.push(id);
            Some(resource)
        } else {
            None
        }
    }

    fn get(&self, id: u32) -> Option<&T> {
        self.resources.get(&id)
    }

    fn get_mut(&mut self, id: u32) -> Option<&mut T> {
        self.resources.get_mut(&id)
    }
}

// ============================================================================
// Native API Stubs (to be implemented per backend)
// ============================================================================

/// Native device wrapper
///
/// TODO: Implement per backend:
/// - Vulkan: VkInstance, VkDevice, VkQueue, VkSwapchain, VkCommandPool
/// - D3D12: ID3D12Device, ID3D12CommandQueue, IDXGISwapChain
/// - Metal: MTLDevice, MTLCommandQueue
/// - OpenGL: Context, FBO management
struct NativeDevice {
    // Backend-specific fields will go here
}

impl NativeDevice {
    fn create(_config: &RendererConfig) -> Self {
        // TODO: Initialize native graphics API
        println!("Creating native device (stub)");
        Self {}
    }

    fn create_texture_native(&mut self, desc: &TextureDesc) -> NativeTexture {
        println!(
            "Creating texture: {}x{} {:?}",
            desc.width, desc.height, desc.format
        );
        NativeTexture { handle: 0 }
    }

    fn create_buffer_native(&mut self, desc: &BufferDesc, _data: Option<&[u8]>) -> NativeBuffer {
        println!("Creating buffer: {} bytes, {:?}", desc.size, desc.usage);
        NativeBuffer { handle: 0 }
    }

    fn create_shader_native(&mut self, desc: &ShaderDesc) -> NativeShader {
        println!(
            "Creating shader: {:?}, {} bytes",
            desc.stage,
            desc.code.len()
        );
        NativeShader { handle: 0 }
    }

    fn create_pipeline_native(
        &mut self,
        _desc: &PipelineDesc,
        _shaders: &ResourcePool<ShaderResource>,
    ) -> NativePipeline {
        println!("Creating pipeline (stub)");
        NativePipeline { handle: 0 }
    }

    fn destroy_texture_native(&mut self, _texture: NativeTexture) {}
    fn destroy_buffer_native(&mut self, _buffer: NativeBuffer) {}
    fn destroy_shader_native(&mut self, _shader: NativeShader) {}
    fn destroy_pipeline_native(&mut self, _pipeline: NativePipeline) {}

    fn update_buffer_native(&mut self, _buffer: NativeBuffer, _offset: usize, data: &[u8]) {
        println!("Updating buffer with {} bytes", data.len());
    }

    fn map_buffer_native(&mut self, _buffer: NativeBuffer) -> *mut u8 {
        std::ptr::null_mut()
    }

    fn unmap_buffer_native(&mut self, _buffer: NativeBuffer) {}

    fn begin_frame_native(&mut self) {
        println!("Begin frame");
    }

    fn begin_render_pass_native(
        &mut self,
        _desc: &RenderPassDesc,
        _textures: &ResourcePool<TextureResource>,
    ) {
        println!("Begin render pass");
    }

    fn end_render_pass_native(&mut self) {
        println!("End render pass");
    }

    fn bind_pipeline_native(&mut self, _pipeline: NativePipeline) {
        println!("Bind pipeline");
    }

    fn set_viewport_native(&mut self, viewport: &Viewport) {
        println!("Set viewport: {}x{}", viewport.width, viewport.height);
    }

    fn set_scissor_native(&mut self, _scissor: &Rect) {
        println!("Set scissor");
    }

    fn bind_vertex_buffer_native(&mut self, slot: u32, _buffer: NativeBuffer, _offset: u64) {
        println!("Bind vertex buffer at slot {}", slot);
    }

    fn bind_index_buffer_native(
        &mut self,
        _buffer: NativeBuffer,
        _offset: u64,
        _index_type: IndexType,
    ) {
        println!("Bind index buffer");
    }

    fn draw_native(
        &mut self,
        vertex_count: u32,
        instance_count: u32,
        _first_vertex: u32,
        _first_instance: u32,
    ) {
        println!(
            "Draw: {} vertices, {} instances",
            vertex_count, instance_count
        );
    }

    fn draw_indexed_native(
        &mut self,
        index_count: u32,
        instance_count: u32,
        _first_index: u32,
        _vertex_offset: i32,
        _first_instance: u32,
    ) {
        println!(
            "Draw indexed: {} indices, {} instances",
            index_count, instance_count
        );
    }

    fn present_native(&mut self) {
        println!("Present");
    }

    fn resize_native(&mut self, width: u32, height: u32) {
        println!("Resize: {}x{}", width, height);
    }

    fn wait_idle_native(&mut self) {
        println!("Wait idle");
    }
}

// Native handles (opaque, backend-specific)
#[derive(Clone, Copy)]
struct NativeTexture {
    handle: u64, // VkImage, ID3D12Resource*, MTLTexture, GLuint, etc.
}

#[derive(Clone, Copy)]
struct NativeBuffer {
    handle: u64, // VkBuffer, ID3D12Resource*, MTLBuffer, GLuint, etc.
}

#[derive(Clone, Copy)]
struct NativeShader {
    handle: u64, // VkShaderModule, ID3DBlob*, MTLFunction, GLuint, etc.
}

#[derive(Clone, Copy)]
struct NativePipeline {
    handle: u64, // VkPipeline, ID3D12PipelineState*, MTLRenderPipelineState, GLuint, etc.
}

// ============================================================================
// Public API for creating device
// ============================================================================

/// Create a GPU device with the given configuration
pub fn create_device(config: RendererConfig) -> BackendDevice {
    BackendDevice::new(config)
}
