// Copyright (c) 2025 Jonathon Burnham Cobb
// Licensed under the MIT-0 license.
mod dfs;
mod prims;
mod wilsons;

use crate::maze::RenderStyle;
use std::{hash::RandomState, io::Result as IoResult};

pub use dfs::DfsMazeBuilder;
pub use prims::PrimsMazeBuilder;
pub use wilsons::WilsonsMazeBuilder;

pub trait MazeBuilder {
    /// Performs a single step of maze generation. Returns `true` if further calls are needed to complete the maze, or
    /// `false` if generation is complete.
    fn build_next(&mut self, bias: &crate::maze::BiasMode) -> bool;
    fn render(&self, style: &RenderStyle, random_state: &RandomState) -> IoResult<()>;
}
