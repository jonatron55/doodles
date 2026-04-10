// Copyright (c) 2025 Jonathon Burnham Cobb
// Licensed under the MIT-0 license.

use std::{hash::RandomState, io::Result as IoResult};

use doodles::common::{
    dir::Direction,
    vec::{UVec2, uvec2},
};
use rand::{Rng, RngExt};

use crate::{
    agent::RenderStyle as AgentRenderStyle,
    maze::{BiasMode, Cell, Maze, RenderStyle},
};

/// A maze generator using randomized depth-first search.
///
/// This algorithm starts at the entrance cell and carves passages by performing a depth-first search through the maze.
/// At each step, it randomly selects an unvisited neighbor to travel to. If no unvisited neighbors are available, it
/// backtracks until it finds a cell with unvisited neighbors. This continues until all cells have been visited.
///
/// This algorithm tends to produce mazes with long, winding passages and few short dead ends.
#[derive(Debug)]
pub struct DfsMazeBuilder<'a> {
    maze: &'a mut Maze,
    open: Vec<DfsOpenCell>,
}

/// A cell that has been encountered during DFS maze generation but not yet visited.
#[derive(Clone, Copy, Debug)]
struct DfsOpenCell {
    /// Position of the cell.
    head: UVec2,

    /// Position from which this cell was reached.
    from: UVec2,
}

impl<'a> DfsMazeBuilder<'a> {
    pub fn new(maze: &'a mut Maze, rand: &mut impl Rng) -> Self {
        let initial = uvec2(rand.random_range(0..maze.size.x), rand.random_range(0..maze.size.y));

        DfsMazeBuilder {
            maze,
            open: vec![DfsOpenCell {
                head: initial,
                from: initial,
            }],
        }
    }

    pub fn build_next(&mut self, rand: &mut impl Rng, bias: &BiasMode) -> bool {
        // Get the next unvisited cell.
        let Some(DfsOpenCell { head, from }) = self.pop_unvisited() else {
            // No more open cells; maze generation is complete.
            return false;
        };

        let head_idx = self.maze.cell_index(head);

        // Mark cell as visited.
        self.maze.cells[head_idx].insert(Cell::VISITED);

        // Remove wall between current and previous cell.
        if from != head {
            self.maze.tunnel_between(from, head);
        }

        // Push unvisited neighbors in random order.
        let bias = bias.sample(head);
        let dirs = Direction::biased_shuffle(rand, bias);

        for &dir in &dirs {
            let Some(next) = dir.move_point_within(head, self.maze.size) else {
                continue;
            };

            let next_idx = self.maze.cell_index(next);
            if !self.maze.cells[next_idx].contains(Cell::VISITED) {
                self.open.push(DfsOpenCell { head: next, from: head });
            }
        }

        self.maze.invalidate();

        true
    }

    pub fn render(&self, style: &RenderStyle, random_state: &RandomState) -> IoResult<()> {
        self.maze.render(style, &[], &[], &AgentRenderStyle::default(), random_state)
    }

    /// Pop the next unvisited open cell from the stack, skipping any that have already been visited.
    ///
    /// Returns `None` if there are no unvisited open cells remaining (i.e., maze generation is complete).
    fn pop_unvisited(&mut self) -> Option<DfsOpenCell> {
        while let Some(open_cell) = self.open.pop() {
            let p = open_cell.head;
            let idx = self.maze.cell_index(p);
            if !self.maze.cells[idx].contains(Cell::VISITED) {
                return Some(open_cell);
            }
        }
        None
    }
}
