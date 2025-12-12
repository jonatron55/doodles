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
    color::Color,
    dir::{Direction, Directions},
    image::Image,
    term::{DIM_STYLES, STYLES},
};
use rand::Rng;

use crate::agent::{Agent, RenderStyle as AgentRenderStyle};

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
/// The maze is initially completely impassable and is generated using a randomized depth-first search algorithm. The
/// function [`Maze::build_next`] should be called repeatedly until it returns `false`, indicating that the maze is
/// fully generated.
pub struct Maze {
    /// Total number of cells in the horizontal direction.
    width: usize,

    /// Total number of cells in the vertical direction.
    height: usize,

    /// Cells in row-major order.
    cells: Vec<Cell>,

    /// Cached bitmap representation for rendering.
    bitmap: RefCell<Option<BitVec>>,

    /// Remaining open cells to process during maze generation.
    open: Vec<OpenCell>,
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
}

pub enum BiasMode {
    Uniform(f64),
    Image(Image),
}

/// A cell that has been encountered during maze generation but not yet visited.
struct OpenCell {
    /// Position of the cell.
    cell: (usize, usize),

    /// Position from which this cell was reached.
    from: (usize, usize),
}

const HEDGE_CHARS: [char; 51] = [
    '⡟', '⡪', '⡯', '⡳', '⡵', '⡵', '⡷', '⡹', '⡺', '⡻', '⡼', '⡽', '⡾', '⡿', '⢏', '⢕', '⢗', '⢜', '⢝',
    '⢞', '⢟', '⢮', '⢯', '⢷', '⢻', '⢽', '⢾', '⢿', '⣎', '⣏', '⣕', '⣗', '⣝', '⣞', '⣟', '⣣', '⣧', '⣪',
    '⣫', '⣮', '⣯', '⣳', '⣵', '⣷', '⣹', '⣺', '⣻', '⣼', '⣽', '⣾', '⣿',
];

bitflags! {
    /// Representation of a single maze cell.
    ///
    /// Note that walls to the north and west are not stored explicitly, but are implied by neighboring cells.
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub struct Cell: u8 {
        /// This cell is impassable to the east.
        const WALL_EAST  = 0b0000_0010;

        /// This cell is impassable to the south.
        const WALL_SOUTH = 0b0000_0100;

        /// This cell has been visited during maze generation.
        const VISITED    = 0b1000_0000;
    }
}

impl Maze {
    /// Create a new, ungenerated maze of the given size.
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

    /// Compute all walls present at the given cell.
    pub fn walls(&self, x: usize, y: usize) -> Directions {
        let cell = self.cells[self.cell_index(x, y)];
        let mut walls = Directions::empty();

        // East and South walls are stored by the cell itself.
        if cell.contains(Cell::WALL_EAST) {
            walls |= Directions::EAST;
        }

        if cell.contains(Cell::WALL_SOUTH) {
            walls |= Directions::SOUTH;
        }

        // North and West walls are implied by neighboring cells or the maze border.
        if x == 0 || self.cells[self.cell_index(x - 1, y)].contains(Cell::WALL_EAST) {
            walls |= Directions::WEST;
        }

        if y == 0 || self.cells[self.cell_index(x, y - 1)].contains(Cell::WALL_SOUTH) {
            walls |= Directions::NORTH;
        }

        walls
    }

    /// Perform the next step of maze generation.
    ///
    /// Returns `true` if more steps are needed, or `false` if the maze is fully generated.
    pub fn build_next<R: Rng>(&mut self, rand: &mut R, bias: &BiasMode) -> bool {
        // Get the next unvisited cell.
        let Some(OpenCell {
            cell: (x, y),
            from: (from_x, from_y),
        }) = self.pop_unvisited()
        else {
            // No more open cells; maze generation is complete.
            return false;
        };

        let current = self.cell_index(x, y);

        // Mark cell as visited.
        self.cells[current].insert(Cell::VISITED);

        let from = self.cell_index(from_x, from_y);

        // Remove wall between current and previous cell.
        if x < from_x {
            self.cells[current].remove(Cell::WALL_EAST);
        } else if x > from_x {
            self.cells[from].remove(Cell::WALL_EAST);
        } else if y < from_y {
            self.cells[current].remove(Cell::WALL_SOUTH);
        } else if y > from_y {
            self.cells[from].remove(Cell::WALL_SOUTH);
        }

        // Push unvisited neighbors in random order.
        let horz = if rand.random_bool(0.5) {
            [Direction::East, Direction::West]
        } else {
            [Direction::West, Direction::East]
        };
        let vert = if rand.random_bool(0.5) {
            [Direction::North, Direction::South]
        } else {
            [Direction::South, Direction::North]
        };

        let bias = match bias {
            BiasMode::Uniform(b) => *b,
            BiasMode::Image(img) => {
                let (img_width, img_height) = img.size();
                if x < img_width && y < img_height {
                    img.pixel(x, y) as f64
                } else {
                    0.5
                }
            }
        };

        let dirs = if rand.random_bool(bias) {
            [horz[0], horz[1], vert[0], vert[1]]
        } else {
            [vert[0], vert[1], horz[0], horz[1]]
        };

        for &dir in &dirs {
            let Some((nx, ny)) = dir.move_point_within((x, y), (self.width, self.height)) else {
                continue;
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

        // Invalidate cached bitmap.
        self.bitmap.replace(None);

        true
    }

    /// Render the maze to the terminal.
    ///
    /// Note that the total size of the rendered maze will be `(width * 2 + 1)` by `(height * 2 + 1)` characters to
    /// accommodate cells, internal walls, and outer borders.
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
                    let (cell_x, cell_y) = ((x.wrapping_sub(1)) / 2, (y.wrapping_sub(1)) / 2);
                    if (x.wrapping_sub(1)) % 2 == 0
                        && (y.wrapping_sub(1)) % 2 == 0
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

    /// Get the size of the maze in cells.
    pub fn size(&self) -> (usize, usize) {
        (self.width, self.height)
    }

    /// Render the maze into a bitmap for efficient repeated rendering.
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
                    // Set diagonal corners
                    bitmap.set((by - 1) * bmp_width + (bx - 1), true);
                    bitmap.set((by - 1) * bmp_width + (bx + 1), true);
                    bitmap.set((by + 1) * bmp_width + (bx + 1), true);
                    bitmap.set((by + 1) * bmp_width + (bx - 1), true);

                    // Set walls as needed
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

    /// Get the linear index of a cell at the given coordinates.
    fn cell_index(&self, x: usize, y: usize) -> usize {
        y * self.width + x
    }

    /// Gets the total rendered bitmap size in characters.
    fn bitmap_size(&self) -> (usize, usize) {
        (self.width * 2 + 1, self.height * 2 + 1)
    }

    /// Pop the next unvisited open cell from the stack, skipping any that have already been visited.
    ///
    /// Returns `None` if there are no unvisited open cells remaining (i.e., maze generation is complete).
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
    pub fn with_color(self, color: Color) -> Self {
        RenderStyle {
            outer: self.outer,
            inner: self.inner,
            color,
        }
    }
}
