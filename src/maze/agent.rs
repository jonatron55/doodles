use std::io::{Result as IoResult, stdout};

use crossterm::{cursor::MoveTo, queue, style::PrintStyledContent};
use doodles::common::{dir::Direction, term::BOLD_STYLES};
use rand::Rng;

use crate::maze::Maze;

pub struct Agent {
    position: (usize, usize),
    state: State,
    color: u8,
}

enum State {
    Thinking,
    Moving(Direction),
    Halted,
}

impl Agent {
    pub fn new(color: u8) -> Self {
        Agent {
            position: (0, 1),
            state: State::Moving(Direction::East),
            color,
        }
    }

    pub fn update<R: Rng>(&mut self, maze: &Maze, rand: &mut R) {
        match &self.state {
            State::Thinking => {
                let (x, y) = self.position;
                let (cell_x, cell_y) = ((x - 1) / 2, (y - 1) / 2);
                let walls = maze.walls(cell_x, cell_y);

                if let Some(dir) = walls.complement().choose(rand) {
                    self.state = State::Moving(dir);
                    self.position = match dir {
                        Direction::North => (x, y - 1),
                        Direction::East => (x + 1, y),
                        Direction::South => (x, y + 1),
                        Direction::West => (x - 1, y),
                    };
                }
            }
            State::Moving(dir) => {
                let (x, y) = self.position;
                let (x, y) = match dir {
                    Direction::North => (x, y - 1),
                    Direction::East => (x + 1, y),
                    Direction::South => (x, y + 1),
                    Direction::West => (x - 1, y),
                };

                let (w, h) = maze.size();

                if x > w * 2 || y > h * 2 {
                    self.state = State::Halted;
                } else {
                    self.position = (x, y);
                    self.state = State::Thinking;
                }
            }
            State::Halted => {}
        }
    }

    pub fn render(&self) -> IoResult<()> {
        let (x, y) = self.position;

        // let s = match &self.state {
        //     State::Thinking => "●",
        //     State::Moving(Direction::North) => "▲",
        //     State::Moving(Direction::East) => "▶",
        //     State::Moving(Direction::South) => "▼",
        //     State::Moving(Direction::West) => "◀",
        //     State::Halted => "■",
        // };

        queue!(
            stdout(),
            MoveTo(x as u16, y as u16),
            PrintStyledContent(BOLD_STYLES[(self.color as usize) % BOLD_STYLES.len()].apply("☻")),
        )
    }

    pub fn position(&self) -> (usize, usize) {
        self.position
    }

    pub fn is_halted(&self) -> bool {
        matches!(self.state, State::Halted)
    }
}
