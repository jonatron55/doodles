// Copyright (c) 2025 Jonathon Burnham Cobb
// Licensed under the MIT-0 license.

use std::io::{Result as IoResult, Write, stdout};

use crossterm::{
    cursor::MoveTo,
    queue,
    style::{ContentStyle, PrintStyledContent},
};
use doodles::common::{Lerp, color::Color};
use rand::{
    Rng,
    distr::{Bernoulli, Distribution},
    prelude::IndexedRandom,
};

use crate::{Args, ColorArg};

/// Default alphabet to use if none is provided.
const DEFAULT_ALPHABET: &str = include_str!("alphabet.txt");

/// A board of cells for the Digital Rain effect.
pub struct Board {
    /// Width of the board in characters.
    width: usize,

    /// Height of the board in characters.
    height: usize,

    /// Double buffer of cells (each in row-major order).
    buffers: (Vec<Cell>, Vec<Cell>),

    /// Alphabet of characters to randomly choose from.
    alphabet: Vec<char>,
}

/// A single cell on the board.
///
/// Empty cells have age `u32::MAX` and content `'\0'`.
#[derive(Clone)]
pub struct Cell {
    /// Number of frames since the cell was spawned.
    pub age: u32,

    /// Character content of the cell or `'\0'` if empty.
    pub content: char,

    /// Length of the trail leading to this cell. If the trail length exceeds [`crate::Args::max_trail`], or is randomly
    /// decided to end after reaching [`crate::Args::min_trail`], no new cells will be spawned below this one.
    pub trail_length: u32,

    /// Color of the cell.
    pub color: Color,
}

impl Board {
    /// Creates a new empty board with the given dimensions and optional alphabet.
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

    /// Resize the board to new dimensions. Existing cell data will be preserved where possible and new cells will be
    /// initially empty.
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

    /// Advance the board by one frame, consuming the current board and producing `Some(new_board)` if there are still
    /// cells alive, or `None` if the board is completely dead.
    pub fn next<R: Rng>(
        mut self,
        args: &Args,
        frame: usize,
        color: &ColorArg,
        rand: &mut R,
        dead: bool,
    ) -> Option<Self> {
        self.buffers.1.fill(Cell::default());

        // Random distribution for spawning new head cells. Will be rolled for each empty cell.
        let spawn = Bernoulli::new(f64::saturating_lerp(
            0.0,
            args.spawnprob,
            (frame as f64 / args.warmup as f64).powi(2),
        ))
        .unwrap();

        // Random distribution for continuing trails. Will be rolled for each cell with a `trail_length` between
        // `min_trail` and `max_trail` (cells shorter than `min_trail` always continue and cells longer than `max_trail`
        // never continue).
        let trail = Bernoulli::new(((args.max_trail - args.min_trail) as f64).recip()).unwrap();

        for y in 0..self.height {
            for x in 0..self.width {
                let index = self.cell_index(x, y);

                if self.buffers.0[index].is_alive(args) {
                    // If we have a living cell, age it and possibly spawn a new cell below.
                    let mut cell = self.buffers.0[index].clone();
                    cell.content = *self.alphabet.choose(rand).unwrap();

                    if cell.age == 0 {
                        // This is a head cell; possibly spawn depending on the existing trail length.
                        let continue_trail = match cell.trail_length {
                            len if len < args.min_trail => true, // Below minimum trail length; always continue.
                            len if len >= args.max_trail => false, // Above maximum trail length; never continue.
                            _ => trail.sample(rand), // Between min and max; continue at random.
                        };

                        if continue_trail {
                            let lower = self.cell_index(x, y + 1);
                            self.buffers.1[lower] = Cell::new_head(
                                *self.alphabet.choose(rand).unwrap(),
                                cell.trail_length + 1,
                                cell.color,
                            );
                        }
                    }

                    cell.age += 1;
                    self.buffers.1[index] = cell;
                } else if !dead && !self.buffers.1[index].is_alive(args) && spawn.sample(rand) {
                    // We have an empty cell and randomly decided to spawn a new head here.
                    let color = match color {
                        ColorArg::Color(color) => *color,
                        ColorArg::Cycle => Color::from(
                            ((frame.wrapping_mul(2) / ((args.lifespan as usize).saturating_mul(3)))
                                % 7
                                + 1) as u8,
                        ),
                        ColorArg::Random => Color::choose(rand),
                    };
                    self.buffers.1[index] =
                        Cell::new_head(*self.alphabet.choose(rand).unwrap(), 1, color);
                }
            }
        }

        if frame < args.warmup || self.buffers.1.iter().any(|cell| cell.is_alive(args)) {
            // There are still living cells; return the new board with buffers swapped.
            Some(Self {
                width: self.width,
                height: self.height,
                buffers: (self.buffers.1, self.buffers.0),
                alphabet: self.alphabet,
            })
        } else {
            None
        }
    }

    /// Render the board to the terminal.
    pub fn render(&self, args: &Args) -> IoResult<()> {
        let mut stdout = stdout();

        for y in 0..self.height {
            queue!(stdout, MoveTo(0, y as u16))?;

            for x in 0..self.width {
                let cell = &self.buffers.0[self.cell_index(x, y)];
                if cell.is_alive(args) {
                    // Display head cells in bold and trail cells in dim.
                    let style = if cell.age == 0 {
                        cell.color.bold_style()
                    } else {
                        cell.color.dim_style()
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

    /// Returns the linear index of the cell at the given coordinates, wrapping at boundaries.
    fn cell_index(&self, x: usize, y: usize) -> usize {
        y % self.height * self.width + x % self.width
    }
}

impl Cell {
    /// Creates a new empty cell.
    fn new() -> Self {
        Self {
            age: u32::MAX,
            content: '\0',
            trail_length: 0,
            color: Color::default(),
        }
    }

    /// Creates a new head cell with the given content, trail length, and color.
    fn new_head(content: char, trail_length: u32, color: Color) -> Self {
        Self {
            age: 0,
            content,
            trail_length: trail_length,
            color,
        }
    }

    /// Check if the cell's age is below the lifespan defined in `args`.
    fn is_alive(&self, args: &Args) -> bool {
        self.age < args.lifespan
    }
}

impl Default for Cell {
    fn default() -> Self {
        Cell::new()
    }
}
