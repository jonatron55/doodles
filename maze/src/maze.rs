// Copyright (c) 2025 Jonathon Burnham Cobb
// Licensed under the MIT-0 license.

pub mod generator;

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
use doodle::{
    borders::BorderStyle,
    color::Color,
    dir::Directions,
    image::Image,
    row_major::IterRowMajor,
    vec::{UVec2, uvec2},
};
use rand::Rng;

use crate::{
    agent::{Agent, RenderStyle as AgentRenderStyle},
    maze::generator::{DfsMazeBuilder, PrimsMazeBuilder, WilsonsMazeBuilder},
    trinket::Trinket,
};

/// A two-dimensional maze.
///
/// The maze is represented as a grid of cells, each of which may have walls blocking movement to adjacent cells. Each
/// cell stores whether it has walls to the east and south; walls to the north and west implied by its neighbors in
/// these directions. While generating the maze, each cell also tracks whether it has been visited by the generation
/// algorithm.
///
/// The entrance is implied to be at the northwest corner (0, 0) and all other cells at the north or west edges are
/// implied to be impassable in those directions. The exit is explicitly placed at the southeast corner by removing the
/// east wall of that cell.
///
/// The maze starts with all interior passages walled off and is carved by a maze generation algorithm.
///
/// Generation is implemented by [`MazeBuilder`] variants (for example, [`DfsMazeBuilder`] and [`WilsonsMazeBuilder`]).
/// Call [`MazeBuilder::build_next`] repeatedly until it returns `false`, indicating that the maze is fully generated.
#[derive(Clone, Debug)]
pub struct Maze {
    size: UVec2,

    /// Cells in row-major order.
    cells: Vec<Cell>,

    /// Cached bitmap representation for rendering.
    bitmap: RefCell<Option<BitVec>>,
}

/// Maze generation bias mode.
///
/// This controls the likelihood of horizontal passages being carved before vertical passages during maze generation.
#[derive(Clone, Debug)]
pub enum BiasMode {
    /// Uniform bias value.
    Uniform(f64),

    /// Bias sampled from an image.
    Image(Image),
}

/// A maze generation algorithm.
#[derive(Debug)]
pub enum MazeBuilder<'a> {
    Dfs(DfsMazeBuilder<'a>),
    Prims(PrimsMazeBuilder<'a>),
    Wilsons(WilsonsMazeBuilder<'a>),
}

/// Maze rendering style.
#[derive(Clone, Debug)]
pub struct RenderStyle {
    /// Style for the outer border walls.
    pub outer: WallStyle,

    /// Style for the interior walls.
    pub inner: WallStyle,

    /// Wall color.
    pub color: Color,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WallStyle {
    Solid,
    Curved,
    Double,
    Bold,
    Block,
    Hedge,
    Fence,
}

const HEDGE_CHARS: [char; 51] = [
    '⡟', '⡪', '⡯', '⡳', '⡵', '⡵', '⡷', '⡹', '⡺', '⡻', '⡼', '⡽', '⡾', '⡿', '⢏', '⢕', '⢗', '⢜', '⢝', '⢞', '⢟', '⢮', '⢯',
    '⢷', '⢻', '⢽', '⢾', '⢿', '⣎', '⣏', '⣕', '⣗', '⣝', '⣞', '⣟', '⣣', '⣧', '⣪', '⣫', '⣮', '⣯', '⣳', '⣵', '⣷', '⣹', '⣺',
    '⣻', '⣼', '⣽', '⣾', '⣿',
];

bitflags! {
    /// Representation of a single maze cell.
    ///
    /// Note that walls to the north and west are not stored explicitly, but are implied by neighboring cells.
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub struct Cell: u8 {
        /// This cell is impassable to the east.
        const WALL_EAST = 0b0000_0001;

        /// This cell is impassable to the south.
        const WALL_SOUTH = 0b0000_0010;

        /// This cell has been visited during maze generation.
        const VISITED = 0b0000_1000;
    }
}

impl Maze {
    /// Create a new, ungenerated maze of the given size.
    pub fn new(size: UVec2) -> Self {
        let mut cells = vec![Cell::default(); size.x * size.y];
        cells[size.x * size.y - 1].remove(Cell::WALL_EAST); // Exit

        Maze {
            size,
            cells,
            bitmap: RefCell::new(None),
        }
    }

    pub fn dead_ends(&mut self) -> impl Iterator<Item = UVec2> + '_ {
        (0..self.size.x)
            .flat_map(|x| (0..self.size.y).map(move |y| uvec2(x, y)))
            .filter(|&p| self.walls(p).bits().count_ones() >= 3)
    }

    /// Compute all walls present at the given cell.
    pub fn walls(&self, p: UVec2) -> Directions {
        let cell = self.cells[self.cell_index(p)];
        let mut walls = Directions::empty();

        // East and South walls are stored by the cell itself.
        if cell.contains(Cell::WALL_EAST) {
            walls |= Directions::EAST;
        }

        if cell.contains(Cell::WALL_SOUTH) {
            walls |= Directions::SOUTH;
        }

        // North and West walls are implied by neighboring cells or the maze border.
        if p.x == 0 || self.cells[self.cell_index(uvec2(p.x - 1, p.y))].contains(Cell::WALL_EAST) {
            walls |= Directions::WEST;
        }

        if p.y == 0 || self.cells[self.cell_index(uvec2(p.x, p.y - 1))].contains(Cell::WALL_SOUTH) {
            walls |= Directions::NORTH;
        }

        walls
    }

    /// Render the maze to the terminal.
    ///
    /// Note that the total size of the rendered maze will be `(width * 2 + 1)` by `(height * 2 + 1)` characters to
    /// accommodate cells, internal walls, and outer borders.
    pub fn render(
        &self,
        style: &RenderStyle,
        agents: &[Agent],
        trinkets: &[Trinket],
        agent_style: &AgentRenderStyle,
        random_state: &RandomState,
    ) -> IoResult<()> {
        let mut stdout = stdout();
        self.render_bitmap();
        let bmp = self.bitmap.borrow();
        let bmp = bmp.as_ref().unwrap();
        let bmp_size = self.bitmap_size();

        for y in 0..bmp_size.y {
            queue!(stdout, MoveTo(0, y as u16))?;
            for x in 0..bmp_size.x {
                let idx = y * bmp_size.x + x;

                if let Some(agent) = agents.iter().find(|a| a.render_position() == uvec2(x, y)) {
                    agent.render(agent_style)?;
                    continue;
                } else if let Some(trinket) = trinkets.iter().find(|t| t.render_position() == uvec2(x, y)) {
                    trinket.render()?;
                    continue;
                }

                if !bmp[idx] {
                    let cell_pos = uvec2((x.wrapping_sub(1)) / 2, (y.wrapping_sub(1)) / 2);
                    if (x.wrapping_sub(1)) % 2 == 0
                        && (y.wrapping_sub(1)) % 2 == 0
                        && cell_pos.x < self.size.x
                        && cell_pos.y < self.size.y
                    {
                        let cell = self.cells[self.cell_index(cell_pos)];
                        if !cell.contains(Cell::VISITED) {
                            let style = &style.color.dim_style();
                            queue!(stdout, PrintStyledContent(style.apply('∎')))?;
                            continue;
                        }
                    }

                    queue!(stdout, Print(' '))?;
                    continue;
                }

                let mut dirs = Directions::empty();

                if y > 0 && bmp[(y - 1) * bmp_size.x + x] {
                    dirs |= Directions::NORTH;
                }
                if y + 1 < bmp_size.y && bmp[(y + 1) * bmp_size.x + x] {
                    dirs |= Directions::SOUTH;
                }
                if x > 0 && bmp[y * bmp_size.x + (x - 1)] {
                    dirs |= Directions::WEST;
                }
                if x + 1 < bmp_size.x && bmp[y * bmp_size.x + (x + 1)] {
                    dirs |= Directions::EAST;
                }

                let x_border = x == 0 || x + 1 == bmp_size.x;
                let y_border = y == 0 || y + 1 == bmp_size.y;

                let mut print_hedge = |x: usize, y: usize| -> IoResult<()> {
                    let hash = {
                        let mut hasher = random_state.build_hasher();
                        x.hash(&mut hasher);
                        y.hash(&mut hasher);
                        hasher.finish()
                    };
                    let ch = (hash as usize) % HEDGE_CHARS.len();

                    queue!(stdout, PrintStyledContent(style.color.style().apply(HEDGE_CHARS[ch])))
                };

                let is_outer = x_border || y_border;

                if style.outer == WallStyle::Block && is_outer {
                    let color = if matches!(style.inner, WallStyle::Hedge | WallStyle::Fence) {
                        style.color.complement().medium_style()
                    } else {
                        style.color.medium_style()
                    };

                    queue!(stdout, PrintStyledContent(color.apply('█')))?;
                } else if style.outer == WallStyle::Hedge && is_outer || style.inner == WallStyle::Hedge && !is_outer {
                    print_hedge(x, y)?;
                } else if style.outer == WallStyle::Fence && is_outer || style.inner == WallStyle::Fence && !is_outer {
                    let ch = if dirs == Directions::NORTH | Directions::SOUTH {
                        '┊'
                    } else if dirs == Directions::EAST | Directions::WEST {
                        '╌'
                    } else {
                        '•'
                    };

                    queue!(stdout, PrintStyledContent(style.color.style().apply(ch)))?;
                } else if style.inner == WallStyle::Block && !is_outer {
                    queue!(stdout, PrintStyledContent(style.color.medium_style().apply('█')))?;
                } else {
                    let horizontal_style = if x_border { style.outer } else { style.inner };
                    let vertical_style = if y_border { style.outer } else { style.inner };

                    let horizontal_style = match horizontal_style {
                        WallStyle::Solid => BorderStyle::Single,
                        WallStyle::Curved => BorderStyle::Curved,
                        WallStyle::Double => BorderStyle::Double,
                        WallStyle::Bold => BorderStyle::Bold,
                        _ => unreachable!(),
                    };

                    let vertical_style = match vertical_style {
                        WallStyle::Solid => BorderStyle::Single,
                        WallStyle::Curved => BorderStyle::Curved,
                        WallStyle::Double => BorderStyle::Double,
                        WallStyle::Bold => BorderStyle::Bold,
                        _ => unreachable!(),
                    };

                    queue!(
                        stdout,
                        PrintStyledContent(style.color.style().apply(dirs.border(horizontal_style, vertical_style)))
                    )?;
                }
            }
        }

        stdout.flush()
    }

    /// Remove the wall between two adjacent cells.
    pub fn tunnel_between(&mut self, from: UVec2, to: UVec2) {
        assert!(from.manhattan_dist(to) == 1, "Cells {from} and {to} are not adjacent");

        let from_idx = self.cell_index(from);
        let to_idx = self.cell_index(to);

        if from.x < to.x {
            self.cells[from_idx].remove(Cell::WALL_EAST);
        } else if from.x > to.x {
            self.cells[to_idx].remove(Cell::WALL_EAST);
        } else if from.y < to.y {
            self.cells[from_idx].remove(Cell::WALL_SOUTH);
        } else if from.y > to.y {
            self.cells[to_idx].remove(Cell::WALL_SOUTH);
        }
    }

    /// Get the size of the maze in cells.
    pub fn size(&self) -> UVec2 {
        self.size
    }

    /// Render the maze into a bitmap for efficient repeated rendering.
    fn render_bitmap(&self) {
        if self.bitmap.borrow().is_some() {
            return;
        }

        let bmp_size = self.bitmap_size();
        let mut bitmap = BitVec::repeat(false, bmp_size.x * bmp_size.y);

        for pos in self.size.iter_row_major() {
            let cell = self.cells[self.cell_index(pos)];
            let visited = cell.contains(Cell::VISITED);

            let bmppos = pos * 2 + UVec2::ONE;

            if visited {
                // Set diagonal corners
                bitmap.set((bmppos.y - 1) * bmp_size.x + (bmppos.x - 1), true);
                bitmap.set((bmppos.y - 1) * bmp_size.x + (bmppos.x + 1), true);
                bitmap.set((bmppos.y + 1) * bmp_size.x + (bmppos.x + 1), true);
                bitmap.set((bmppos.y + 1) * bmp_size.x + (bmppos.x - 1), true);

                // Set walls as needed
                if cell.contains(Cell::WALL_EAST) {
                    bitmap.set(bmppos.y * bmp_size.x + (bmppos.x + 1), true);
                }

                if cell.contains(Cell::WALL_SOUTH) {
                    bitmap.set((bmppos.y + 1) * bmp_size.x + bmppos.x, true);
                }

                if pos.x == 0 || self.cells[self.cell_index(pos - uvec2(1, 0))].contains(Cell::WALL_EAST) {
                    bitmap.set(bmppos.y * bmp_size.x + (bmppos.x - 1), true);
                }

                if pos.y == 0 || self.cells[self.cell_index(pos - uvec2(0, 1))].contains(Cell::WALL_SOUTH) {
                    bitmap.set((bmppos.y - 1) * bmp_size.x + bmppos.x, true);
                }
            }
        }

        bitmap.set(bmp_size.x, false); // Entrance

        self.bitmap.replace(Some(bitmap));
    }

    /// Get the linear index of a cell at the given coordinates.
    pub fn cell_index(&self, p: UVec2) -> usize {
        p.y * self.size.x + p.x
    }

    pub fn invalidate(&mut self) {
        self.bitmap.replace(None);
    }

    /// Gets the total rendered bitmap size in characters.
    fn bitmap_size(&self) -> UVec2 {
        uvec2(self.size.x * 2 + 1, self.size.y * 2 + 1)
    }
}

impl Default for Cell {
    fn default() -> Self {
        Cell::WALL_EAST | Cell::WALL_SOUTH
    }
}

impl RenderStyle {
    pub fn with_color(self, color: Color) -> Self {
        RenderStyle {
            outer: self.outer,
            inner: self.inner,
            color,
        }
    }
}

impl MazeBuilder<'_> {
    /// Build the next step of the maze generation.
    pub fn build_next(&mut self, rand: &mut impl Rng, bias: &BiasMode) -> bool {
        match self {
            MazeBuilder::Dfs(builder) => builder.build_next(rand, bias),
            MazeBuilder::Prims(builder) => builder.build_next(rand, bias),
            MazeBuilder::Wilsons(builder) => builder.build_next(rand, bias),
        }
    }

    pub fn render(&self, style: &RenderStyle, random_state: &RandomState) -> IoResult<()> {
        match self {
            MazeBuilder::Dfs(builder) => builder.render(style, random_state),
            MazeBuilder::Prims(builder) => builder.render(style, random_state),
            MazeBuilder::Wilsons(builder) => builder.render(style, random_state),
        }
    }
}

impl BiasMode {
    /// Get the bias value at the given coordinates.
    pub fn sample(&self, p: UVec2) -> f64 {
        match self {
            BiasMode::Uniform(b) => *b,
            BiasMode::Image(img) => {
                if p.x < img.size().x && p.y < img.size().y {
                    img.pixel(p)
                } else {
                    0.5
                }
            }
        }
    }
}
