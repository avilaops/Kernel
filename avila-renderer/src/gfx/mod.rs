// Copyright (c) 2025 Avila Contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Graphics rendering module

pub mod api;
pub mod backend;
pub mod framegraph;

pub use api::*;
pub use backend::create_device;
pub use framegraph::{FrameGraphBuilder, CompiledFrameGraph};
