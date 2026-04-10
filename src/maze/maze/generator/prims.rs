// Copyright (c) 2025 Jonathon Burnham Cobb
// Licensed under the MIT-0 license.

use std::{collections::HashMap, hash::RandomState, io::Result as IoResult};

use doodles::common::{
    dir::Direction,
    vec::{UVec2, uvec2},
};
use rand::{Rng, RngExt, seq::IteratorRandom};

use crate::{
    agent::RenderStyle as AgentRenderStyle,
    maze::{Cell, Maze, RenderStyle},
};

/// A maze generator using Prim's algorithm.
///
/// This algorithm starts with a single cell marked as visited and adds its unvisited neighbors to a frontier set. At
/// each step, it randomly selects a cell from the frontier, carves a passage to an adjacent visited cell, and expands
/// the frontier. This continues until all cells have been visited.
///
/// This algorithm tends to produce mazes with many short dead ends and has a more uniform distribution of passage
/// lengths. It does not support a bias in passage direction.
#[derive(Debug)]
pub struct PrimsMazeBuilder<'a> {
    maze: &'a mut Maze,

    /// Maps a frontier cell to the direction taken to reach it.
    frontier: HashMap<UVec2, Direction>,
}

impl<'a> PrimsMazeBuilder<'a> {
    pub fn new<R: Rng>(maze: &'a mut Maze, rand: &mut R) -> Self {
        let initial = uvec2(rand.random_range(0..maze.size.x), rand.random_range(0..maze.size.y));

        let initial_idx = maze.cell_index(initial);
        maze.cells[initial_idx].insert(Cell::VISITED);

        let mut frontier = HashMap::new();

        if initial.x > 0 {
            frontier.insert(uvec2(initial.x - 1, initial.y), Direction::West);
        }
        if initial.x + 1 < maze.size.x {
            frontier.insert(uvec2(initial.x + 1, initial.y), Direction::East);
        }
        if initial.y > 0 {
            frontier.insert(uvec2(initial.x, initial.y - 1), Direction::North);
        }
        if initial.y + 1 < maze.size.y {
            frontier.insert(uvec2(initial.x, initial.y + 1), Direction::South);
        }

        PrimsMazeBuilder { maze, frontier }
    }

    pub fn build_next<R: Rng>(&mut self, rand: &mut R) -> bool {
        if self.frontier.is_empty() {
            return false;
        }

        let next = *self.frontier.keys().choose(rand).unwrap();
        let next_idx = self.maze.cell_index(next);
        let dir = self.frontier.remove(&next).unwrap();
        let from = dir.opposite().move_point(next);

        self.maze.cells[next_idx].insert(Cell::VISITED);
        self.maze.tunnel_between(from, next);

        self.maze.invalidate();

        // Add neighbors of next to the frontier.
        for dir in Direction::ALL.iter() {
            if let Some(neighbor) = dir.move_point_within(next, self.maze.size) {
                let neighbor_idx = self.maze.cell_index(neighbor);
                if !self.maze.cells[neighbor_idx].contains(Cell::VISITED) {
                    self.frontier.entry(neighbor).or_insert(*dir);
                }
            }
        }

        true
    }

    pub fn render(&self, style: &RenderStyle, random_state: &RandomState) -> IoResult<()> {
        self.maze.render(style, &[], &[], &AgentRenderStyle::default(), random_state)
    }
}
