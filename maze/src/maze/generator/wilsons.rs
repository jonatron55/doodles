// Copyright (c) 2025 Jonathon Burnham Cobb
// Licensed under the MIT-0 license.

use std::{
    collections::HashMap,
    hash::RandomState,
    io::Result as IoResult,
    ops::{Index, RangeBounds, RangeFrom},
    slice::SliceIndex,
    vec::Drain,
};

use doodle::{
    dir::Direction,
    vec::{UVec2, uvec2},
};
use rand::{Rng, seq::SliceRandom};

use crate::{
    agent::RenderStyle as AgentRenderStyle,
    maze::{BiasMode, Cell, Maze, RenderStyle, generator::MazeBuilder},
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
pub struct WilsonsMazeBuilder<'a, R: Rng> {
    maze: &'a mut Maze,
    open: Vec<UVec2>,
    path: Path,
    seeding: bool,
    seed_length: usize,
    rand: &'a mut R,
}

#[derive(Debug)]
struct Path {
    points: Vec<UVec2>,
    indices: HashMap<UVec2, usize>,
}

impl<'a, R: Rng> WilsonsMazeBuilder<'a, R> {
    pub fn new(maze: &'a mut Maze, rand: &'a mut R) -> Self {
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

        let mut path = Path::new();
        path.push(head);

        WilsonsMazeBuilder {
            maze,
            open,
            path,
            seeding: true,
            seed_length,
            rand,
        }
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

impl<'a, R: Rng> MazeBuilder for WilsonsMazeBuilder<'a, R> {
    fn build_next(&mut self, bias: &BiasMode) -> bool {
        let head = *self.path.last().unwrap();
        let from = if self.path.len() >= 2 {
            Some(self.path[self.path.len() - 2])
        } else {
            None
        };

        // We’ll try to move randomly in each direction until we find a valid move. Directions are shuffled according to
        // the sampled bias, which may favour horizontal or vertical movement.
        let bias = bias.sample(head);
        let dirs = Direction::biased_shuffle(self.rand, bias);

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

            if let Some(loop_start) = self.path.index_of(&next) {
                // We’ve hit a visited cell that is part of our current walk and created a loop.
                let loop_idx = self.maze.cell_index(self.path[loop_start]);
                let loop_pos = self.path[loop_start];
                let tail_pos = self.path[loop_start + 1];

                // Erase the loop by resetting all cells added to the path after this point. The walk will continue
                // from the point of intersection.
                for point in self.path.drain(loop_start + 1..) {
                    let idx = self.maze.cell_index(point);
                    self.maze.cells[idx] = Cell::default();
                }

                // Finally, restore the wall that was removed to enter the loop so that the current path ends in a dead
                // end. This only matters if the loop started in the East or South direction since other directions
                // would have restored walls already when those cells were reset.
                if loop_pos.x < tail_pos.x {
                    self.maze.cells[loop_idx].insert(Cell::WALL_EAST);
                } else if loop_pos.y < tail_pos.y {
                    self.maze.cells[loop_idx].insert(Cell::WALL_SOUTH);
                }

                self.maze.ensure_exit();
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
                    self.path.clear();
                    self.path.push(new_head);

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

    fn render(&self, style: &RenderStyle, random_state: &RandomState) -> IoResult<()> {
        self.maze.render(style, &[], &[], &AgentRenderStyle::default(), random_state)
    }
}

impl Path {
    fn new() -> Self {
        Path {
            points: Vec::new(),
            indices: HashMap::new(),
        }
    }

    fn clear(&mut self) {
        self.points.clear();
        self.indices.clear();
    }

    fn len(&self) -> usize {
        self.points.len()
    }

    fn index_of(&self, point: &UVec2) -> Option<usize> {
        self.indices.get(point).copied()
    }

    fn push(&mut self, point: UVec2) {
        self.indices.insert(point, self.points.len());
        self.points.push(point);
    }

    fn last(&self) -> Option<&UVec2> {
        self.points.last()
    }

    fn drain<'a>(
        &'a mut self,
        range: impl RangeBounds<usize> + SliceIndex<[UVec2], Output = [UVec2]> + Clone,
    ) -> Drain<'a, UVec2> {
        for point in &self.points[range.clone()] {
            self.indices.remove(point);
        }
        self.points.drain(range)
    }
}

impl Index<usize> for Path {
    type Output = UVec2;

    fn index(&self, index: usize) -> &Self::Output {
        &self.points[index]
    }
}

impl Index<RangeFrom<usize>> for Path {
    type Output = [UVec2];

    fn index(&self, index: RangeFrom<usize>) -> &Self::Output {
        &self.points[index]
    }
}
