// Copyright (c) 2025 Jonathon Burnham Cobb
// Licensed under the MIT-0 license.

use std::{hash::RandomState, io::Result as IoResult};

use doodles::common::{
    dir::Direction,
    vec::{UVec2, uvec2},
};
use rand::Rng;

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
pub struct DfsMazeBuilder<'a> {
    maze: &'a mut Maze,
    open: Vec<DfsOpenCell>,
}

/// A cell that has been encountered during DFS maze generation but not yet visited.
struct DfsOpenCell {
    /// Position of the cell.
    cell: UVec2,

    /// Position from which this cell was reached.
    from: UVec2,
}

impl<'a> DfsMazeBuilder<'a> {
    pub fn new<R: Rng>(maze: &'a mut Maze, rand: &mut R) -> Self {
        let initial = uvec2(
            rand.random_range(0..maze.size.x),
            rand.random_range(0..maze.size.y),
        );

        DfsMazeBuilder {
            maze,
            open: vec![DfsOpenCell {
                cell: initial,
                from: initial,
            }],
        }
    }

    pub fn build_next<R: Rng>(&mut self, rand: &mut R, bias: &BiasMode) -> bool {
        // Get the next unvisited cell.
        let Some(DfsOpenCell {
            cell: UVec2 { x, y },
            from: UVec2 {
                x: from_x,
                y: from_y,
            },
        }) = self.pop_unvisited()
        else {
            // No more open cells; maze generation is complete.
            return false;
        };

        let current = self.maze.cell_index(uvec2(x, y));

        // Mark cell as visited.
        self.maze.cells[current].insert(Cell::VISITED);

        let from = self.maze.cell_index(uvec2(from_x, from_y));

        // Remove wall between current and previous cell.
        if x < from_x {
            self.maze.cells[current].remove(Cell::WALL_EAST);
        } else if x > from_x {
            self.maze.cells[from].remove(Cell::WALL_EAST);
        } else if y < from_y {
            self.maze.cells[current].remove(Cell::WALL_SOUTH);
        } else if y > from_y {
            self.maze.cells[from].remove(Cell::WALL_SOUTH);
        }

        // Push unvisited neighbors in random order.
        let bias = bias.sample(uvec2(x, y));
        let dirs = Direction::biased_shuffle(rand, bias);

        for &dir in &dirs {
            let Some(n) = dir.move_point_within(uvec2(x, y), self.maze.size) else {
                continue;
            };

            let next = self.maze.cell_index(n);
            let neighbor = self.maze.cells[next];
            if !neighbor.contains(Cell::VISITED) {
                self.open.push(DfsOpenCell {
                    cell: n,
                    from: uvec2(x, y),
                });
            }
        }

        self.maze.invalidate();

        true
    }

    pub fn render(&self, style: &RenderStyle, random_state: &RandomState) -> IoResult<()> {
        self.maze
            .render(style, &[], &[], &AgentRenderStyle::default(), random_state)
    }

    /// Pop the next unvisited open cell from the stack, skipping any that have already been visited.
    ///
    /// Returns `None` if there are no unvisited open cells remaining (i.e., maze generation is complete).
    fn pop_unvisited(&mut self) -> Option<DfsOpenCell> {
        while let Some(open_cell) = self.open.pop() {
            let p = open_cell.cell;
            let idx = self.maze.cell_index(p);
            if !self.maze.cells[idx].contains(Cell::VISITED) {
                return Some(open_cell);
            }
        }
        None
    }
}
