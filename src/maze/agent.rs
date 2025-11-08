use std::{
    collections::HashSet,
    io::{Result as IoResult, stdout},
};

use crossterm::{cursor::MoveTo, queue, style::PrintStyledContent};
use doodles::common::{
    dir::{Direction, Directions},
    term::BOLD_STYLES,
};
use rand::Rng;

use crate::maze::Maze;

pub struct Agent {
    position: (usize, usize),
    state: State,
    color: u8,
    path: Vec<Junction>,
    closed: HashSet<(usize, usize)>,
    dir: Direction,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RenderStyle {
    Smiley,
    Inchworm,
    Turtle,
}

enum State {
    Thinking,
    Moving(Direction),
    Halted,
}

struct Junction {
    open: Directions,
    from: Option<Direction>,
}

impl Agent {
    pub fn new(maze: &Maze, color: u8) -> Self {
        Agent {
            position: (0, 0),
            state: State::Thinking,
            color,
            path: vec![Junction {
                open: maze.walls(0, 0).complement(),
                from: None,
            }],
            closed: HashSet::from([(0, 0)]),
            dir: Direction::East,
        }
    }

    pub fn update<R: Rng>(&mut self, maze: &Maze, rand: &mut R) {
        match &self.state {
            State::Thinking => {
                if let Some(junction) = self.path.last_mut() {
                    if let Some(choice) = junction.open.choose(rand) {
                        self.state = State::Moving(choice);
                        self.dir = choice;
                        junction.open.remove(choice.into());
                    } else {
                        if let Some(from) = junction.from {
                            self.state = State::Moving(from);
                            self.path.pop();
                        } else {
                            self.state = State::Halted;
                        }
                    }
                } else {
                    self.state = State::Halted;
                }
            }
            State::Moving(dir) => {
                self.closed.insert(self.position);
                let (x, y) = dir.move_position(self.position);
                let (w, h) = maze.size();

                if x >= w || y >= h {
                    self.state = State::Halted;
                } else {
                    self.position = (x, y);

                    if !self.closed.contains(&(x, y)) {
                        let mut open = maze.walls(x, y).complement();
                        let from = dir.opposite();
                        open.remove(from.into());
                        self.path.push(Junction {
                            open,
                            from: Some(from),
                        });
                    }
                    self.dir = *dir;
                    self.state = State::Thinking;
                }
            }
            State::Halted => {}
        }
    }

    pub fn render(&self, style: &RenderStyle) -> IoResult<()> {
        let (x, y) = self.render_position();

        let s = match style {
            RenderStyle::Smiley => "☻",
            RenderStyle::Inchworm => match &self.state {
                State::Thinking => "•",
                State::Moving(Direction::North) | State::Moving(Direction::South) => "┃",
                State::Moving(Direction::East) | State::Moving(Direction::West) => "━",
                State::Halted => "•",
            },

            RenderStyle::Turtle => match &self.dir {
                Direction::North => "▲",
                Direction::East => "►",
                Direction::South => "▼",
                Direction::West => "◄",
            },
        };

        queue!(
            stdout(),
            MoveTo(x as u16, y as u16),
            PrintStyledContent(BOLD_STYLES[(self.color as usize) % BOLD_STYLES.len()].apply(s)),
        )
    }

    pub fn render_position(&self) -> (usize, usize) {
        let (mut x, mut y) = self.position;
        x = x * 2 + 1;
        y = y * 2 + 1;

        if let State::Moving(dir) = &self.state {
            (x, y) = dir.move_position((x, y));
        } else if matches!(self.state, State::Halted) {
            x += 1;
        }

        (x, y)
    }

    pub fn is_halted(&self) -> bool {
        matches!(self.state, State::Halted)
    }
}
