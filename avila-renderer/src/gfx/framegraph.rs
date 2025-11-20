// Copyright (c) 2025 Avila Contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Frame Graph (Render Graph) system
//!
//! Automatic resource management, barrier insertion, and render pass scheduling.
//! Inspired by Frostbite's FrameGraph and modern rendering techniques.

use crate::gfx::api::*;
use std::collections::HashMap;

/// Frame graph builder for declaring rendering passes
pub struct FrameGraphBuilder {
    passes: Vec<PassNode>,
    resources: HashMap<String, ResourceNode>,
    next_pass_id: u32,
}

impl FrameGraphBuilder {
    pub fn new() -> Self {
        Self {
            passes: Vec::new(),
            resources: HashMap::new(),
            next_pass_id: 0,
        }
    }

    /// Create a transient texture resource
    pub fn create_texture(&mut self, name: &str, desc: TextureDesc) -> ResourceId {
        let id = ResourceId::new(name);
        self.resources.insert(
            name.to_string(),
            ResourceNode {
                id: id.clone(),
                desc: ResourceDesc::Texture(desc),
                producer: None,
                consumers: Vec::new(),
            },
        );
        id
    }

    /// Import an external texture (e.g., swapchain)
    pub fn import_texture(&mut self, name: &str, handle: TextureHandle) -> ResourceId {
        let id = ResourceId::new(name);
        self.resources.insert(
            name.to_string(),
            ResourceNode {
                id: id.clone(),
                desc: ResourceDesc::Imported(ImportedResource::Texture(handle)),
                producer: None,
                consumers: Vec::new(),
            },
        );
        id
    }

    /// Add a rendering pass
    pub fn add_pass(
        &mut self,
        name: &str,
        setup: impl FnOnce(&mut PassBuilder),
        execute: PassExecuteFn,
    ) -> PassId {
        let pass_id = PassId(self.next_pass_id);
        self.next_pass_id += 1;

        let mut builder = PassBuilder {
            pass_id,
            reads: Vec::new(),
            writes: Vec::new(),
        };

        setup(&mut builder);

        // Register this pass as producer/consumer of resources
        for read in &builder.reads {
            if let Some(resource) = self.resources.get_mut(&read.0) {
                resource.consumers.push(pass_id);
            }
        }

        for write in &builder.writes {
            if let Some(resource) = self.resources.get_mut(&write.0) {
                if resource.producer.is_some() {
                    panic!("Resource '{}' already has a producer", write.0);
                }
                resource.producer = Some(pass_id);
            }
        }

        self.passes.push(PassNode {
            id: pass_id,
            name: name.to_string(),
            reads: builder.reads,
            writes: builder.writes,
            execute,
        });

        pass_id
    }

    /// Compile the frame graph and return an executable version
    pub fn compile(self) -> CompiledFrameGraph {
        // TODO: Topological sort, culling unused passes, barrier insertion
        println!("Compiling frame graph with {} passes", self.passes.len());

        CompiledFrameGraph {
            passes: self.passes,
            resources: self.resources,
        }
    }
}

impl Default for FrameGraphBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Pass builder for declaring resource dependencies
pub struct PassBuilder {
    pass_id: PassId,
    reads: Vec<ResourceId>,
    writes: Vec<ResourceId>,
}

impl PassBuilder {
    /// Declare that this pass reads from a resource
    pub fn read(&mut self, resource: &ResourceId) {
        self.reads.push(resource.clone());
    }

    /// Declare that this pass writes to a resource
    pub fn write(&mut self, resource: &ResourceId) {
        self.writes.push(resource.clone());
    }
}

/// Pass execution callback
pub type PassExecuteFn = Box<dyn Fn(&mut CommandList, &PassResources)>;

/// Pass resources available during execution
pub struct PassResources {
    textures: HashMap<String, TextureHandle>,
}

impl PassResources {
    pub fn get_texture(&self, name: &str) -> TextureHandle {
        *self.textures.get(name).unwrap_or(&TextureHandle::INVALID)
    }
}

/// Compiled frame graph ready for execution
pub struct CompiledFrameGraph {
    passes: Vec<PassNode>,
    resources: HashMap<String, ResourceNode>,
}

impl CompiledFrameGraph {
    /// Execute the frame graph
    pub fn execute(&self, device: &mut dyn GpuDevice) {
        println!("Executing frame graph with {} passes", self.passes.len());

        // Allocate transient resources
        let mut allocated_textures: HashMap<String, TextureHandle> = HashMap::new();

        for (name, resource) in &self.resources {
            match &resource.desc {
                ResourceDesc::Texture(desc) => {
                    let handle = device.create_texture(desc);
                    allocated_textures.insert(name.clone(), handle);
                }
                ResourceDesc::Imported(ImportedResource::Texture(handle)) => {
                    allocated_textures.insert(name.clone(), *handle);
                }
            }
        }

        // Execute passes in order
        for pass in &self.passes {
            println!("  Pass: {}", pass.name);

            let pass_resources = PassResources {
                textures: allocated_textures.clone(),
            };

            let mut cmd = device.begin_frame();
            (pass.execute)(&mut cmd, &pass_resources);
            device.submit(cmd);
        }

        // Cleanup transient resources
        for (name, handle) in allocated_textures {
            if let Some(resource) = self.resources.get(&name) {
                if !matches!(resource.desc, ResourceDesc::Imported(_)) {
                    device.destroy_texture(handle);
                }
            }
        }
    }
}

// ============================================================================
// Internal Types
// ============================================================================

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ResourceId(String);

impl ResourceId {
    fn new(name: &str) -> Self {
        Self(name.to_string())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct PassId(u32);

struct PassNode {
    id: PassId,
    name: String,
    reads: Vec<ResourceId>,
    writes: Vec<ResourceId>,
    execute: PassExecuteFn,
}

struct ResourceNode {
    id: ResourceId,
    desc: ResourceDesc,
    producer: Option<PassId>,
    consumers: Vec<PassId>,
}

enum ResourceDesc {
    Texture(TextureDesc),
    Imported(ImportedResource),
}

enum ImportedResource {
    Texture(TextureHandle),
}

// ============================================================================
// Example Usage (Documentation)
// ============================================================================

/// Example of using the frame graph
///
/// ```ignore
/// let mut fg = FrameGraphBuilder::new();
///
/// // Import swapchain
/// let backbuffer = fg.import_texture("backbuffer", swapchain_texture);
///
/// // Create depth buffer
/// let depth = fg.create_texture(
///     "depth",
///     TextureDesc::new_2d(1280, 720, TextureFormat::Depth24, TextureUsage::DEPTH_ATTACHMENT),
/// );
///
/// // Shadow pass
/// let shadow_map = fg.create_texture(
///     "shadow_map",
///     TextureDesc::new_2d(2048, 2048, TextureFormat::Depth32f, TextureUsage::DEPTH_ATTACHMENT | TextureUsage::SAMPLED),
/// );
///
/// fg.add_pass(
///     "shadow_pass",
///     |pass| {
///         pass.write(&shadow_map);
///     },
///     Box::new(|cmd, resources| {
///         let shadow_tex = resources.get_texture("shadow_map");
///         // Render shadows...
///     }),
/// );
///
/// // Main pass
/// fg.add_pass(
///     "main_pass",
///     |pass| {
///         pass.read(&shadow_map);
///         pass.write(&backbuffer);
///         pass.write(&depth);
///     },
///     Box::new(|cmd, resources| {
///         let shadow_tex = resources.get_texture("shadow_map");
///         let backbuffer_tex = resources.get_texture("backbuffer");
///         let depth_tex = resources.get_texture("depth");
///         // Render scene...
///     }),
/// );
///
/// let compiled = fg.compile();
/// compiled.execute(&mut device);
/// ```
pub fn _example() {}
