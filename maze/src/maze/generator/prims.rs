// Copyright (c) 2025 Jonathon Burnham Cobb
// Licensed under the MIT-0 license.

use std::{cmp::Ordering, collections::BinaryHeap, hash::RandomState, io::Result as IoResult};

use doodle::{
    dir::{Axis, Direction},
    vec::{UVec2, uvec2},
};
use rand::{Rng, RngExt};

use crate::{
    BiasMode,
    agent::RenderStyle as AgentRenderStyle,
    maze::{Cell, Maze, RenderStyle},
};

/// A maze generator using Prim’s algorithm (minimum spanning tree).
///
/// This algorithm starts with a single cell marked as visited and adds its unvisited neighbors to a frontier set.
/// Neighbors are queued with a random weight, based on the specified bias. At each step, it dequeues a cell from the
/// frontier, carves a passage to an adjacent visited cell, and expands the frontier. This continues until all cells
/// have been visited.
///
/// This algorithm tends to produce mazes with many short dead ends and has a more uniform distribution of passage
/// lengths.
#[derive(Debug)]
pub struct PrimsMazeBuilder<'a> {
    maze: &'a mut Maze,

    /// Priority queue of edges connecting visited cells to unvisited neighbors, ordered by weight.
    frontier: BinaryHeap<Edge>,
}

#[derive(Debug, Clone, Eq, PartialEq)]
struct Edge {
    from: UVec2,
    to: UVec2,
    weight: u32,
}

impl<'a> PrimsMazeBuilder<'a> {
    pub fn new(maze: &'a mut Maze, rand: &mut impl Rng, bias: &BiasMode) -> Self {
        let initial = uvec2(rand.random_range(0..maze.size.x), rand.random_range(0..maze.size.y));

        let initial_idx = maze.cell_index(initial);
        maze.cells[initial_idx].insert(Cell::VISITED);

        let mut builder = PrimsMazeBuilder {
            maze,
            frontier: BinaryHeap::new(),
        };
        builder.push_frontier(initial, rand, bias);
        builder
    }

    pub fn build_next(&mut self, rand: &mut impl Rng, bias: &BiasMode) -> bool {
        loop {
            let Some(Edge { from, to, .. }) = self.frontier.pop() else {
                return false;
            };

            let to_idx = self.maze.cell_index(to);

            if !self.maze.cells[to_idx].contains(Cell::VISITED) {
                self.maze.cells[to_idx].insert(Cell::VISITED);
                self.maze.tunnel_between(from, to);

                self.maze.invalidate();

                self.push_frontier(to, rand, bias);

                return true;
            }
        }
    }

    pub fn render(&self, style: &RenderStyle, random_state: &RandomState) -> IoResult<()> {
        self.maze.render(style, &[], &[], &AgentRenderStyle::default(), random_state)
    }

    /// Enqueues the unvisited neighbors of the given cell into the frontier, with weights based on the specified bias.
    fn push_frontier(&mut self, cell: UVec2, rand: &mut impl Rng, bias: &BiasMode) {
        let bias = bias.sample(cell);

        for dir in Direction::ALL {
            let Some(neighbor) = dir.move_point_within(cell, self.maze.size) else {
                continue;
            };

            if self.maze.cells[self.maze.cell_index(neighbor)].contains(Cell::VISITED) {
                continue;
            }

            // When weighting edges, we use the highest bit to encode the bias direction, and the remaining bits to
            // randomize the order of edges with the same bias. This ensures that the bias is respected while still
            // randomly shuffling edges of the same bias category.
            let weight: u32 = match dir.axis() {
                Axis::Horizontal => {
                    if rand.random_bool(bias) {
                        0x0000_0000
                    } else {
                        0x8000_0000
                    }
                }
                Axis::Vertical => {
                    if rand.random_bool(bias) {
                        0x8000_0000
                    } else {
                        0x0000_0000
                    }
                }
            };
            let weight = weight | (rand.random::<u32>() & 0x7FFF_FFFF);

            self.frontier.push(Edge::new(cell, neighbor, weight));
        }
    }
}

impl Edge {
    fn new(from: UVec2, to: UVec2, weight: u32) -> Self {
        Edge { from, to, weight }
    }
}

impl Ord for Edge {
    fn cmp(&self, other: &Self) -> Ordering {
        self.weight.cmp(&other.weight)
    }
}

impl PartialOrd for Edge {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
