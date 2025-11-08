// Copyright (c) 2025 Jonathon Burnham Cobb
// Licensed under the MIT-0 license.

use std::{
    cell::RefCell,
    hash::{BuildHasher, Hash, Hasher, RandomState},
    io::{Result as IoResult, Write, stdout},
};

use bitflags::bitflags;
use bitvec::vec::BitVec;
use crossterm::{
    cursor::MoveTo,
    queue,
    style::{Print, PrintStyledContent},
};
use doodles::common::{
    borders::BorderStyle,
    dir::Directions,
    term::{DIM_STYLES, STYLES},
};
use rand::{Rng, seq::SliceRandom};

use crate::agent::{Agent, RenderStyle as AgentRenderStyle};

pub struct Maze {
    width: usize,
    height: usize,
    cells: Vec<Cell>,
    bitmap: RefCell<Option<BitVec>>,
    open: Vec<OpenCell>,
}

#[derive(Clone, Debug)]
pub struct RenderStyle {
    pub outer: WallStyle,
    pub inner: WallStyle,
    pub color: u8,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WallStyle {
    Solid,
    Curved,
    Double,
    Bold,
    Block,
    Hedge,
}

struct OpenCell {
    cell: (usize, usize),
    from: (usize, usize),
}

const HEDGE_CHARS: [char; 51] = [
    '⡟', '⡪', '⡯', '⡳', '⡵', '⡵', '⡷', '⡹', '⡺', '⡻', '⡼', '⡽', '⡾', '⡿', '⢏', '⢕', '⢗', '⢜', '⢝',
    '⢞', '⢟', '⢮', '⢯', '⢷', '⢻', '⢽', '⢾', '⢿', '⣎', '⣏', '⣕', '⣗', '⣝', '⣞', '⣟', '⣣', '⣧', '⣪',
    '⣫', '⣮', '⣯', '⣳', '⣵', '⣷', '⣹', '⣺', '⣻', '⣼', '⣽', '⣾', '⣿',
];

bitflags! {
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub struct Cell: u8 {
        const WALL_EAST  = 0b0000_0010;
        const WALL_SOUTH = 0b0000_0100;
        const VISITED    = 0b1000_0000;
    }
}

impl Maze {
    pub fn new(width: usize, height: usize) -> Self {
        let mut cells = vec![Cell::default(); width * height];
        cells[width * height - 1].remove(Cell::WALL_EAST); // Exit

        Maze {
            width,
            height,
            cells,
            bitmap: RefCell::new(None),
            open: vec![OpenCell {
                cell: (0, 0),
                from: (0, 0),
            }],
        }
    }

    pub fn walls(&self, x: usize, y: usize) -> Directions {
        let cell = self.cells[self.cell_index(x, y)];
        let mut walls = Directions::empty();

        if cell.contains(Cell::WALL_EAST) {
            walls |= Directions::EAST;
        }
        if cell.contains(Cell::WALL_SOUTH) {
            walls |= Directions::SOUTH;
        }

        if x == 0 || self.cells[self.cell_index(x - 1, y)].contains(Cell::WALL_EAST) {
            walls |= Directions::WEST;
        }

        if y == 0 || self.cells[self.cell_index(x, y - 1)].contains(Cell::WALL_SOUTH) {
            walls |= Directions::NORTH;
        }

        walls
    }

    pub fn build_next<R: Rng>(&mut self, rand: &mut R) -> bool {
        let mut dirs = [
            Directions::NORTH,
            Directions::EAST,
            Directions::SOUTH,
            Directions::WEST,
        ];

        let Some(OpenCell {
            cell: (x, y),
            from: (from_x, from_y),
        }) = self.pop_unvisited()
        else {
            return false;
        };

        let current = self.cell_index(x, y);
        self.cells[current].insert(Cell::VISITED);

        let from = self.cell_index(from_x, from_y);

        // Remove wall between cells
        if x < from_x {
            self.cells[current].remove(Cell::WALL_EAST);
        } else if x > from_x {
            self.cells[from].remove(Cell::WALL_EAST);
        } else if y < from_y {
            self.cells[current].remove(Cell::WALL_SOUTH);
        } else if y > from_y {
            self.cells[from].remove(Cell::WALL_SOUTH);
        }

        dirs.shuffle(rand);

        for &dir in &dirs {
            let (nx, ny) = match dir {
                Directions::NORTH if y > 0 => (x, y - 1),
                Directions::EAST if x + 1 < self.width => (x + 1, y),
                Directions::SOUTH if y + 1 < self.height => (x, y + 1),
                Directions::WEST if x > 0 => (x - 1, y),
                _ => continue,
            };

            let next = self.cell_index(nx, ny);
            let neighbor = self.cells[next];
            if !neighbor.contains(Cell::VISITED) {
                self.open.push(OpenCell {
                    cell: (nx, ny),
                    from: (x, y),
                });
            }
        }

        self.bitmap.replace(None);

        true
    }

    pub fn render(
        &self,
        style: &RenderStyle,
        agents: &[Agent],
        agent_style: &AgentRenderStyle,
        random_state: &RandomState,
    ) -> IoResult<()> {
        let mut stdout = stdout();
        self.render_bitmap();
        let bmp = self.bitmap.borrow();
        let bmp = bmp.as_ref().unwrap();
        let (bmp_width, bmp_height) = self.bitmap_size();

        for y in 0..bmp_height {
            queue!(stdout, MoveTo(0, y as u16),)?;
            for x in 0..bmp_width {
                let idx = y * bmp_width + x;

                if let Some(agent) = agents.iter().find(|a| a.render_position() == (x, y)) {
                    agent.render(agent_style)?;
                    continue;
                }

                if !bmp[idx] {
                    let (cell_x, cell_y) = ((x - 1) / 2, (y - 1) / 2);
                    if (x - 1) % 2 == 0
                        && (y - 1) % 2 == 0
                        && cell_x < self.width
                        && cell_y < self.height
                    {
                        let cell = self.cells[self.cell_index(cell_x, cell_y)];
                        if !cell.contains(Cell::VISITED) {
                            let style = &DIM_STYLES[style.color as usize];
                            queue!(stdout, PrintStyledContent(style.apply('∎')))?;
                            continue;
                        }
                    }

                    queue!(stdout, Print(' '))?;
                    continue;
                }

                let mut dirs = Directions::empty();

                if y > 0 && bmp[(y - 1) * bmp_width + x] {
                    dirs |= Directions::NORTH;
                }
                if y + 1 < bmp_height && bmp[(y + 1) * bmp_width + x] {
                    dirs |= Directions::SOUTH;
                }
                if x > 0 && bmp[y * bmp_width + (x - 1)] {
                    dirs |= Directions::WEST;
                }
                if x + 1 < bmp_width && bmp[y * bmp_width + (x + 1)] {
                    dirs |= Directions::EAST;
                }

                let x_border = x == 0 || x + 1 == bmp_width;
                let y_border = y == 0 || y + 1 == bmp_height;

                let mut print_hedge = |x: usize, y: usize| -> IoResult<()> {
                    let hash = {
                        let mut hasher = random_state.build_hasher();
                        x.hash(&mut hasher);
                        y.hash(&mut hasher);
                        hasher.finish()
                    };
                    let ch = (hash as usize) % HEDGE_CHARS.len();

                    queue!(
                        stdout,
                        PrintStyledContent(STYLES[style.color as usize].apply(HEDGE_CHARS[ch]))
                    )
                };

                if style.outer == WallStyle::Block && (x_border || y_border) {
                    queue!(
                        stdout,
                        PrintStyledContent(STYLES[style.color as usize].apply('█'))
                    )?;
                } else if style.outer == WallStyle::Hedge && (x_border || y_border) {
                    print_hedge(x, y)?;
                } else if style.inner == WallStyle::Block && !(x_border || y_border) {
                    queue!(
                        stdout,
                        PrintStyledContent(STYLES[style.color as usize].apply('█'))
                    )?;
                } else if style.inner == WallStyle::Hedge && !(x_border || y_border) {
                    print_hedge(x, y)?;
                } else {
                    let horizontal_style = if x_border { style.outer } else { style.inner };
                    let vertical_style = if y_border { style.outer } else { style.inner };

                    let horizontal_style = match horizontal_style {
                        WallStyle::Solid => BorderStyle::Single,
                        WallStyle::Curved => BorderStyle::Curved,
                        WallStyle::Double => BorderStyle::Double,
                        WallStyle::Bold => BorderStyle::Bold,
                        WallStyle::Block | WallStyle::Hedge => unreachable!(),
                    };

                    let vertical_style = match vertical_style {
                        WallStyle::Solid => BorderStyle::Single,
                        WallStyle::Curved => BorderStyle::Curved,
                        WallStyle::Double => BorderStyle::Double,
                        WallStyle::Bold => BorderStyle::Bold,
                        WallStyle::Block | WallStyle::Hedge => unreachable!(),
                    };

                    queue!(
                        stdout,
                        PrintStyledContent(
                            STYLES[style.color as usize]
                                .apply(dirs.border(horizontal_style, vertical_style))
                        )
                    )?;
                }
            }
        }

        stdout.flush()
    }

    pub fn size(&self) -> (usize, usize) {
        (self.width, self.height)
    }

    fn render_bitmap(&self) {
        if self.bitmap.borrow().is_some() {
            return;
        }

        let (bmp_width, bmp_height) = self.bitmap_size();
        let mut bitmap = BitVec::repeat(false, bmp_width * bmp_height);

        for y in 0..self.height {
            for x in 0..self.width {
                let cell = self.cells[self.cell_index(x, y)];
                let visited = cell.contains(Cell::VISITED);

                let bx = x * 2 + 1;
                let by = y * 2 + 1;

                if visited {
                    bitmap.set((by - 1) * bmp_width + (bx - 1), true);
                    bitmap.set((by - 1) * bmp_width + (bx + 1), true);
                    bitmap.set((by + 1) * bmp_width + (bx + 1), true);
                    bitmap.set((by + 1) * bmp_width + (bx - 1), true);

                    if cell.contains(Cell::WALL_EAST) {
                        bitmap.set(by * bmp_width + (bx + 1), true);
                    }

                    if cell.contains(Cell::WALL_SOUTH) {
                        bitmap.set((by + 1) * bmp_width + bx, true);
                    }

                    if x == 0 || self.cells[self.cell_index(x - 1, y)].contains(Cell::WALL_EAST) {
                        bitmap.set(by * bmp_width + (bx - 1), true);
                    }

                    if y == 0 || self.cells[self.cell_index(x, y - 1)].contains(Cell::WALL_SOUTH) {
                        bitmap.set((by - 1) * bmp_width + bx, true);
                    }
                }
            }
        }

        bitmap.set(bmp_width, false); // Entrance

        self.bitmap.replace(Some(bitmap));
    }

    fn cell_index(&self, x: usize, y: usize) -> usize {
        y * self.width + x
    }

    fn bitmap_size(&self) -> (usize, usize) {
        (self.width * 2 + 1, self.height * 2 + 1)
    }

    fn pop_unvisited(&mut self) -> Option<OpenCell> {
        while let Some(open_cell) = self.open.pop() {
            let (x, y) = open_cell.cell;
            let idx = self.cell_index(x, y);
            if !self.cells[idx].contains(Cell::VISITED) {
                return Some(open_cell);
            }
        }
        None
    }
}

impl Default for Cell {
    fn default() -> Self {
        Cell::WALL_EAST | Cell::WALL_SOUTH
    }
}

impl RenderStyle {
    pub fn with_color(self, color: u8) -> Self {
        RenderStyle {
            outer: self.outer,
            inner: self.inner,
            color,
        }
    }
}
