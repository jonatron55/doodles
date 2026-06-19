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
/// Compared to other algorithms, Wilson’s method tends to produce mazes with a more uniform distribution of passage
/// lengths and dead ends. However, it can be very slow to converge for larger mazes.
#[derive(Debug)]
pub struct WilsonsMazeBuilder<'a> {
    maze: &'a mut Maze,
    open: Vec<UVec2>,
    path: Vec<UVec2>,
}

impl<'a> WilsonsMazeBuilder<'a> {
    pub fn new(maze: &'a mut Maze, rand: &mut impl Rng) -> Self {
        // Add all cells to the open set and shuffle it.
        let mut open: Vec<_> = (0..maze.size().x)
            .flat_map(|x| (0..maze.size().y).map(move |y| uvec2(x, y)))
            .collect();

        open.shuffle(rand);

        // Start by marking one random solitary cell as visited.
        let initial = open.pop().unwrap();
        let initial_idx = maze.cell_index(initial);
        maze.cells[initial_idx].insert(Cell::VISITED);

        // Start the first walk from another random cell.
        let head = open.pop().unwrap();
        let head_idx = maze.cell_index(head);
        maze.cells[head_idx].insert(Cell::VISITED);

        WilsonsMazeBuilder {
            maze,
            open,
            path: vec![head],
        }
    }

    pub fn build_next(&mut self, rand: &mut impl Rng, bias: &BiasMode) -> bool {
        let head = *self.path.last().unwrap();
        let from = if self.path.len() >= 2 {
            Some(self.path[self.path.len() - 2])
        } else {
            None
        };

        // We’ll try to move randomly in each direction until we find a valid move.
        let bias = bias.sample(head);
        let dirs = Direction::biased_shuffle(rand, bias);

        for dir in dirs.iter() {
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
                // We’ve hit a visited cell that is part of our current walk.
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

                if self.maze.cells[next_idx].contains(Cell::VISITED) {
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

    pub fn seed(maze: &mut Maze, rand: &mut impl Rng) {
        // Mark a small loop of cells as visited to create an initial maze. Strictly speaking, Wilson’s algorithm
        // requires only one cell to be visited to start. However, this is very slow to converge for larger mazes during
        // early iterations. Instead, we make a small rectangle at a random position based on the maze size. This
        // creates an small bias in the final maze but improves early convergence time.
    }
}
