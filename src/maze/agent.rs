// Copyright (c) 2025 Jonathon Burnham Cobb
// Licensed under the MIT-0 license.

use std::{
    collections::HashSet,
    io::{Result as IoResult, stdout},
    str::FromStr,
};

use clap::ValueEnum;
use crossterm::{queue, style::PrintStyledContent};
use doodles::common::{
    color::Color,
    dir::{Direction, Directions},
    vec::{UVec2, uvec2},
};
use rand::{Rng, RngExt};

use crate::{maze::Maze, trinket::Trinket};

/// A maze-solving agent.
///
/// Navigates the maze using a randomized depth-first search until it either finds the exit or exhausts all options.
#[derive(Clone, Debug)]
pub struct Agent<'a> {
    /// Reference to the maze being solved.
    maze: &'a Maze,

    /// Current position within the maze.
    ///
    /// This is in maze cell coordinates, not terminal character coordinates. The rendering position of the agent
    /// depends on whether it is in the center of a cell or moving between cells.
    position: UVec2,

    /// Current state.
    state: State,

    /// Render color.
    color: Color,

    /// Stack of junctions already visited.
    path: Vec<Junction>,

    /// Set of closed (already visited) positions.
    closed: HashSet<UVec2>,

    /// Current facing direction.
    dir: Direction,
}

/// Method for rendering the agent.
#[derive(Clone, Copy, Debug, PartialEq, Eq, ValueEnum)]
#[repr(u8)]
#[clap(rename_all = "kebab-case")]
pub enum RenderStyle {
    /// Agent rendered as a smiley face (`☻`).
    #[clap(alias = "s", alias = "0")]
    Smiley = 0,

    /// Agent rendered as alternating dots (`•`) and lines (`┃` or `━`).
    #[clap(alias = "i", alias = "1")]
    Inchworm = 1,

    /// Agent rendered as directional arrows (`▲`, `▶`, `▼`, `◀`).
    #[clap(alias = "t", alias = "2")]
    Turtle = 2,

    /// Agent rendered as alternating slashes (`/` and `\`).
    #[clap(alias = "w", alias = "3")]
    Walker = 3,
}

/// A maze-solving agent state.
#[derive(Clone, Debug)]
enum State {
    /// Agent is at a junction, deciding which way to go.
    Thinking,

    /// Agent is moving from one cell to another in the given direction.
    Moving(Direction),

    /// Agent has solved the maze and halted.
    Exited,

    /// Agent has exhausted all options without finding an exit and has halted.
    Stuck,
}

/// An agent’s memory of a junction.
#[derive(Clone, Debug)]
struct Junction {
    /// Unexplored directions at this junction.
    open: Directions,

    /// Direction from which the agent arrived at this junction.
    from: Option<Direction>,
}

impl<'a> Agent<'a> {
    /// Create a new agent at the start of the maze.
    pub fn new(maze: &'a Maze, color: Color) -> Self {
        Agent {
            maze,
            position: UVec2::ZERO,
            state: State::Thinking,
            color,
            path: vec![Junction {
                open: maze.walls(UVec2::ZERO).complement(),
                from: None,
            }],
            closed: HashSet::from([UVec2::ZERO]),
            dir: Direction::East,
        }
    }

    /// Update the agent’s state by performing one step of a randomized depth-first search. This will have no effect if
    /// the agent has already halted.
    pub fn update(&mut self, trinkets: &mut [Trinket], rand: &mut impl Rng) {
        match &self.state {
            State::Thinking => {
                // We are at a junction. If there are any unexplored paths from here, then take one at random.
                // Otherwise, backtrack to the previous junction.

                if let Some(idx) = trinkets.iter().position(|t| t.position == self.position && !t.is_collected()) {
                    trinkets[idx].collect();
                }

                if let Some(junction) = self.path.last_mut() {
                    if let Some(choice) = junction.open.choose(rand) {
                        // Take this unexplored path.
                        self.state = State::Moving(choice);
                        self.dir = choice;
                        junction.open.remove(choice.into());
                    } else {
                        // We’ve completely explored this junction; backtrack to the previous one.
                        if let Some(from) = junction.from {
                            self.state = State::Moving(from);
                            self.dir = from;
                            self.path.pop();
                        } else {
                            // No unexplored paths and no way back. This means the maze is insoluble and we should not
                            // reach this point if the maze generation algorithm is correct.
                            self.state = State::Stuck;
                        }
                    }
                } else {
                    // We’ve backed all the way to the start and exhausted all options. The maze is insoluble and we
                    // should not reach this point if the maze generation algorithm is correct.
                    self.state = State::Stuck;
                }
            }
            State::Moving(dir) => {
                // We are moving in the given direction. Update our position accordingly and prepare to think again.
                self.closed.insert(self.position);
                let position = dir.move_point(self.position);
                let size = self.maze.size();

                if position.x >= size.x || position.y >= size.y {
                    // We found the exit!
                    self.state = State::Exited;
                } else {
                    // Move into the new cell.
                    self.position = position;

                    if !self.closed.contains(&position) {
                        // We’ve never been here before; add a new junction to the stack.
                        let mut open = self.maze.walls(position).complement();
                        let from = dir.opposite();
                        open.remove(from.into());
                        self.path.push(Junction { open, from: Some(from) });
                    }
                    self.dir = *dir;
                    self.state = State::Thinking;
                }
            }
            State::Exited | State::Stuck => {
                // The agent has halted; do nothing.
            }
        }
    }

    /// Render the agent at its current position.
    pub fn render(&self, style: &RenderStyle) -> IoResult<()> {
        let s = if matches!(self.state, State::Stuck) {
            "×"
        } else {
            match style {
                RenderStyle::Smiley => "☻",
                RenderStyle::Inchworm => match &self.state {
                    State::Thinking | State::Exited => "•",
                    State::Moving(Direction::North) | State::Moving(Direction::South) => "┃",
                    State::Moving(Direction::East) | State::Moving(Direction::West) => "━",
                    State::Stuck => unreachable!(),
                },

                RenderStyle::Turtle => match &self.dir {
                    Direction::North => "▲",
                    Direction::East => "▶",
                    Direction::South => "▼",
                    Direction::West => "◀",
                },

                RenderStyle::Walker => {
                    if (self.position.x + self.position.y) % 2 == 0 {
                        "/"
                    } else {
                        "\\"
                    }
                }
            }
        };

        queue!(stdout(), PrintStyledContent(self.color.bold_style().apply(s)),)
    }

    /// Get the agent’s rendering position in terminal character coordinates.
    ///
    /// If the agent is in the center of a cell, this will be the center of that cell. If the agent is moving between
    /// cells, this will be the position between the two cells in the direction of movement.
    pub fn render_position(&self) -> UVec2 {
        let UVec2 { x, y } = self.position;

        // Each cell occupies a 2x2 character block, and there is a 1-character border around the maze.
        let x = x * 2 + 1;
        let y = y * 2 + 1;

        match self.state {
            State::Moving(dir) => {
                // Adjust the rendering position in the direction of movement, which will place it between cells.
                dir.move_point(uvec2(x, y))
            }
            State::Exited => {
                // Move the rendering position just outside the maze exit.
                uvec2(x + 1, y)
            }
            State::Thinking | State::Stuck => {
                // Agent is stationary in the center of the cell.
                uvec2(x, y)
            }
        }
    }

    /// Check if the agent has halted (either exited the maze or become stuck).
    pub fn is_halted(&self) -> bool {
        matches!(self.state, State::Exited | State::Stuck)
    }
}

impl RenderStyle {
    /// Choose a random render style.
    pub fn choose(rand: &mut impl Rng) -> Self {
        let value = rand.random_range(0..4);
        RenderStyle::from(value)
    }
}

impl FromStr for RenderStyle {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Ok(value) = s.parse::<u8>() {
            Ok(RenderStyle::from(value))
        } else {
            let s = s.to_uppercase();
            match s.as_str() {
                "S" | "SMILEY" => Ok(RenderStyle::Smiley),
                "I" | "INCHWORM" => Ok(RenderStyle::Inchworm),
                "T" | "TURTLE" => Ok(RenderStyle::Turtle),
                "W" | "WALKER" => Ok(RenderStyle::Walker),
                _ => Err(()),
            }
        }
    }
}

impl Into<u8> for RenderStyle {
    fn into(self) -> u8 {
        self as u8
    }
}

impl From<u8> for RenderStyle {
    fn from(value: u8) -> Self {
        match value & 3 {
            0 => RenderStyle::Smiley,
            1 => RenderStyle::Inchworm,
            2 => RenderStyle::Turtle,
            3 => RenderStyle::Walker,
            _ => unreachable!(),
        }
    }
}

impl Default for RenderStyle {
    fn default() -> Self {
        RenderStyle::Smiley
    }
}
