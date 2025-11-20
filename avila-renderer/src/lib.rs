// Copyright (c) 2025 Avila Contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Avila Renderer
//!
//! High-performance rendering engine with modern graphics API abstraction.
//!
//! # Architecture
//!
//! - `gfx::api` - Backend-agnostic GPU abstraction (textures, buffers, pipelines, commands)
//! - `gfx::backend` - Concrete implementation wrapping native graphics APIs
//! - `gfx::framegraph` - Automatic resource management and render pass scheduling
//!
//! # Example
//!
//! ```rust
//! use avila_renderer::gfx::*;
//!
//! // Create device
//! let config = RendererConfig::default();
//! let mut device = create_device(config);
//!
//! // Create resources
//! let texture = device.create_texture(&TextureDesc::new_2d(
//!     1280, 720,
//!     TextureFormat::Rgba8,
//!     TextureUsage::COLOR_ATTACHMENT,
//! ));
//!
//! let buffer = device.create_buffer(
//!     &BufferDesc::vertex(1024),
//!     Some(&vertex_data),
//! );
//!
//! // Record commands
//! let mut cmd = device.begin_frame();
//! cmd.begin_render_pass(RenderPassDesc {
//!     color_attachments: vec![
//!         ColorAttachment {
//!             texture,
//!             clear: Some(ClearColor::BLACK),
//!         }
//!     ],
//!     depth_attachment: None,
//! });
//! cmd.bind_pipeline(pipeline);
//! cmd.bind_vertex_buffer(0, buffer, 0);
//! cmd.draw(3, 1, 0, 0);
//! cmd.end_render_pass();
//!
//! // Submit and present
//! device.submit(cmd);
//! device.present();
//! ```

pub mod gfx;

pub use gfx::*;
