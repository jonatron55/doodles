// Copyright (c) 2025 Jonathon Burnham Cobb
// Licensed under the MIT-0 license.

use std::{hash::RandomState, io::Result as IoResult};

use doodle::{
    dir::Direction,
    vec::{UVec2, uvec2},
};
use rand::{Rng, seq::SliceRandom};

use crate::{
    agent::RenderStyle as AgentRenderStyle,
    maze::{BiasMode, Cell, Maze, RenderStyle},
};

/// A maze generator using Wilson’s algorithm (loop-erased random walks).
///
/// This algorithm selects a cell at random to initialize the maze, then performs random walks from unvisited cells
/// until they connect to the existing maze. If a walk intersects itself, the loop is erased and the walk continues from
/// the point of intersection. The algorithm  continues until all cells have been visited
///
/// Compared to other algorithms, Wilson’s method produces mazes with a uniform distribution of passage lengths and dead
/// ends. However, it can be very slow to converge for larger mazes.
#[derive(Debug)]
pub struct WilsonsMazeBuilder<'a> {
    maze: &'a mut Maze,
    open: Vec<UVec2>,
    path: Vec<UVec2>,
    seeding: bool,
    seed_length: usize,
}

impl<'a> WilsonsMazeBuilder<'a> {
    pub fn new(maze: &'a mut Maze, rand: &mut impl Rng) -> Self {
        // Add all cells to the open set and shuffle it.
        let mut open: Vec<_> = (0..maze.size().x)
            .flat_map(|x| (0..maze.size().y).map(move |y| uvec2(x, y)))
            .collect();

        open.shuffle(rand);

        // The length of the seed passage will be 0.5% of the maze's area. This is a departure from the true Wilson’s
        // algorithm, which starts with a single seed cell. However, the larger seed helps to speed up convergence and
        // the resulting bias is negligible (larger seed lengths would have a more significant impact).

        let seed_length = (maze.size().prod() / 200).max(1);

        // Start the first walk a random cell. This walk will continue until the seed length is reached, since there are
        // no existing paths to connect to yet.
        let head = open.pop().unwrap();
        let head_idx = maze.cell_index(head);
        maze.cells[head_idx].insert(Cell::VISITED);

        WilsonsMazeBuilder {
            maze,
            open,
            path: vec![head],
            seeding: true,
            seed_length,
        }
    }

    /// Performs a single step of maze generation. Returns `true` if further calls are needed to complete the maze, or
    /// `false` if generation is complete.
    pub fn build_next(&mut self, rand: &mut impl Rng, bias: &BiasMode) -> bool {
        let head = *self.path.last().unwrap();
        let from = if self.path.len() >= 2 {
            Some(self.path[self.path.len() - 2])
        } else {
            None
        };

        // We’ll try to move randomly in each direction until we find a valid move. Directions are shuffled according to
        // the sampled bias, which may favour horizontal or vertical movement.
        let bias = bias.sample(head);
        let dirs = Direction::biased_shuffle(rand, bias);

        for dir in dirs {
            // Don’t move outside the maze.
            let Some(next) = dir.move_point_within(head, self.maze.size) else {
                continue;
            };

            // Don’t immediately backtrack.
            if let Some(from) = from
                && next == from
            {
                continue;
            }

            let next_idx = self.maze.cell_index(next);

            if let Some(loop_start) = self.path.iter().position(|&p| p == next) {
                // We’ve hit a visited cell that is part of our current walk and created a loop.
                let loop_idx = self.maze.cell_index(self.path[loop_start]);
                let loop_pos = self.path[loop_start];
                let tail_pos = self.path[loop_start + 1];

                // Erase the loop by resetting all cells added to the path after this point.
                for point in &self.path[loop_start + 1..] {
                    let idx = self.maze.cell_index(*point);
                    self.maze.cells[idx] = if *point == self.maze.size() - UVec2::ONE {
                        // Don’t accidentally close the exit.
                        Cell::WALL_SOUTH
                    } else {
                        Cell::default()
                    }
                }

                // Remove the loop from the path so that the current walk continues from the point of intersection.
                self.path.truncate(loop_start + 1);

                // Finally, restore the wall that was removed to enter the loop so that the current path ends in a dead
                // end. This only matters if the loop started in the East or South direction since other directions
                // would have restored walls already when those cells were reset.
                if loop_pos.x < tail_pos.x {
                    self.maze.cells[loop_idx].insert(Cell::WALL_EAST);
                } else if loop_pos.y < tail_pos.y {
                    self.maze.cells[loop_idx].insert(Cell::WALL_SOUTH);
                }

                self.maze.invalidate();
                return true;
            } else {
                // Continue our drunken walk by removing the wall between the current cell and the next cell.
                self.path.push(next);
                self.maze.tunnel_between(head, next);

                if self.maze.cells[next_idx].contains(Cell::VISITED)
                    || (self.seeding && self.path.len() >= self.seed_length)
                {
                    // We’ve found a cell that is already part of the maze (we know it cannot be part of our current
                    // walk since we checked for that above). Complete this walk and start a new one.
                    let Some(new_head) = self.pop_unvisited() else {
                        // No more open cells; maze generation is complete.
                        self.maze.invalidate();
                        return false;
                    };

                    let head_idx = self.maze.cell_index(new_head);

                    self.maze.cells[head_idx].insert(Cell::VISITED);
                    self.path = vec![new_head];

                    if self.seeding {
                        self.maze.cells[next_idx].insert(Cell::VISITED);
                        self.seeding = false;
                    }
                } else {
                    // Still walking; just mark the cell as visited.
                    self.maze.cells[next_idx].insert(Cell::VISITED);
                }

                self.maze.invalidate();
                return true;
            }
        }

        // No valid moves found; this should be unreachable since there should always be at least one direction to walk.
        unreachable!("No available directions to walk from {head}");
    }

    pub fn render(&self, style: &RenderStyle, random_state: &RandomState) -> IoResult<()> {
        self.maze.render(style, &[], &[], &AgentRenderStyle::default(), random_state)
    }

    /// Pop an unvisited cell from the open set.
    fn pop_unvisited(&mut self) -> Option<UVec2> {
        while let Some(point) = self.open.pop() {
            let idx = self.maze.cell_index(point);
            if !self.maze.cells[idx].contains(Cell::VISITED) {
                return Some(point);
            }
        }

        None
    }
}
