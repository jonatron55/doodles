use std::io::{Result as IoResult, Write, stdout};

use crossterm::{
    cursor::MoveTo,
    queue,
    style::{ContentStyle, PrintStyledContent},
};
use doodles::common::{BOLD_STYLES, DIM_STYLES};
use rand::{
    Rng,
    distr::{Bernoulli, Distribution},
    prelude::IndexedRandom,
};

use crate::Args;

const DEFAULT_ALPHABET: &str = include_str!("alphabet.txt");

pub struct Board {
    width: usize,
    height: usize,
    buffers: (Vec<Cell>, Vec<Cell>),
    alphabet: Vec<char>,
}

#[derive(Clone)]
pub struct Cell {
    pub age: u32,
    pub content: char,
    pub trail_length: u32,
}

impl Board {
    pub fn new(width: usize, height: usize, alphabet: Option<&str>) -> Self {
        let alphabet = alphabet
            .unwrap_or(DEFAULT_ALPHABET)
            .chars()
            .filter(|ch| !ch.is_whitespace())
            .collect::<Vec<char>>();

        let buffers = (
            vec![Cell::default(); width * height],
            vec![Cell::default(); width * height],
        );

        Self {
            width,
            height,
            buffers,
            alphabet,
        }
    }

    pub fn resize(self, new_width: usize, new_height: usize) -> Self {
        let mut new_board = Board {
            width: new_width,
            height: new_height,
            alphabet: self.alphabet.clone(),
            buffers: (
                vec![Cell::default(); new_width * new_height],
                vec![Cell::default(); new_width * new_height],
            ),
        };

        for y in 0..new_height.min(self.height) {
            for x in 0..new_width.min(self.width) {
                let src = self.cell_index(x, y);
                let dst = new_board.cell_index(x, y);
                new_board.buffers.0[dst] = self.buffers.0[src].clone();
            }
        }

        new_board
    }

    pub fn next<R: Rng>(mut self, args: &Args, rand: &mut R) -> Self {
        self.buffers.1.fill(Cell::default());

        let spawn = Bernoulli::new(args.spawnprob).unwrap();
        let trail = Bernoulli::new(((args.max_trail - args.min_trail) as f64).recip()).unwrap();

        for y in 0..self.height {
            for x in 0..self.width {
                let index = self.cell_index(x, y);

                if self.buffers.0[index].is_alive(args) {
                    let mut cell = self.buffers.0[index].clone();
                    cell.content = *self.alphabet.choose(rand).unwrap();

                    if cell.age == 0 {
                        let continue_trail = match cell.trail_length {
                            len if len < args.min_trail => true,
                            len if len >= args.max_trail => false,
                            _ => trail.sample(rand),
                        };

                        if continue_trail {
                            let lower = self.cell_index(x, y + 1);
                            self.buffers.1[lower] = Cell::new_head(
                                *self.alphabet.choose(rand).unwrap(),
                                cell.trail_length + 1,
                            );
                        }
                    }

                    cell.age += 1;
                    self.buffers.1[index] = cell;
                } else if !self.buffers.1[index].is_alive(args) && spawn.sample(rand) {
                    self.buffers.1[index] = Cell::new_head(*self.alphabet.choose(rand).unwrap(), 1);
                }
            }
        }

        Self {
            width: self.width,
            height: self.height,
            buffers: (self.buffers.1, self.buffers.0),
            alphabet: self.alphabet,
        }
    }

    pub fn render(&self, args: &Args) -> IoResult<()> {
        let mut stdout = stdout();

        for y in 0..self.height {
            queue!(stdout, MoveTo(0, y as u16))?;

            for x in 0..self.width {
                let cell = &self.buffers.0[self.cell_index(x, y)];
                if cell.is_alive(args) {
                    let style = if cell.age == 0 {
                        BOLD_STYLES[args.color]
                    } else {
                        DIM_STYLES[args.color]
                    };

                    queue!(stdout, PrintStyledContent(style.apply(cell.content)))?;
                } else {
                    queue!(
                        stdout,
                        PrintStyledContent(ContentStyle::default().apply(" "))
                    )?;
                }
            }
        }

        stdout.flush()?;

        Ok(())
    }

    fn cell_index(&self, x: usize, y: usize) -> usize {
        y % self.height * self.width + x % self.width
    }
}

impl Cell {
    fn new() -> Self {
        Self {
            age: u32::MAX,
            content: '\0',
            trail_length: 0,
        }
    }

    fn new_head(content: char, trail_length: u32) -> Self {
        Self {
            age: 0,
            content,
            trail_length: trail_length,
        }
    }

    fn is_alive(&self, args: &Args) -> bool {
        self.age < args.lifespan
    }
}

impl Default for Cell {
    fn default() -> Self {
        Cell::new()
    }
}
