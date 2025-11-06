use std::{
    cell::RefCell,
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
    dir::{BorderStyle, Direction},
    term::{BOLD_STYLES, DIM_STYLES, STYLES},
};
use rand::{Rng, seq::SliceRandom};

pub struct Maze {
    width: usize,
    height: usize,
    cells: Vec<Cell>,
    bitmap: RefCell<Option<BitVec>>,
    open: Vec<OpenCell>,
}

struct OpenCell {
    cell: (usize, usize),
    from: (usize, usize),
}

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

    pub fn walls(&self, x: usize, y: usize) -> Direction {
        let cell = self.cells[self.cell_index(x, y)];
        let mut walls = Direction::empty();

        if cell.contains(Cell::WALL_EAST) {
            walls |= Direction::EAST;
        }
        if cell.contains(Cell::WALL_SOUTH) {
            walls |= Direction::SOUTH;
        }

        if x == 0 || self.cells[self.cell_index(x - 1, y)].contains(Cell::WALL_EAST) {
            walls |= Direction::WEST;
        }

        if y == 0 || self.cells[self.cell_index(x, y - 1)].contains(Cell::WALL_SOUTH) {
            walls |= Direction::NORTH;
        }

        walls
    }

    pub fn build_next<R: Rng>(&mut self, rand: &mut R) -> bool {
        let mut dirs = [
            Direction::NORTH,
            Direction::EAST,
            Direction::SOUTH,
            Direction::WEST,
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
                Direction::NORTH if y > 0 => (x, y - 1),
                Direction::EAST if x + 1 < self.width => (x + 1, y),
                Direction::SOUTH if y + 1 < self.height => (x, y + 1),
                Direction::WEST if x > 0 => (x - 1, y),
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

    pub fn render(&self) -> IoResult<()> {
        let mut stdout = stdout();
        self.render_bitmap();
        let bmp = self.bitmap.borrow();
        let bmp = bmp.as_ref().unwrap();
        let (bmp_width, bmp_height) = self.bitmap_size();

        for y in 0..bmp_height {
            queue!(stdout, MoveTo(0, y as u16),)?;
            for x in 0..bmp_width {
                let idx = y * bmp_width + x;

                if !bmp[idx] {
                    // let cell_x = x / 2;
                    // let cell_y = y / 2;
                    // if cell_x < self.width && cell_y < self.height && x % 2 == 1 && y % 2 == 1 {
                    //     if !self.open.is_empty()
                    //         && self.open[self.open.len() - 1].cell == (cell_x, cell_y)
                    //     {
                    //         queue!(stdout, PrintStyledContent(BOLD_STYLES[7].apply('•')))?;
                    //     } else if self.open.iter().any(|o| o.cell == (cell_x, cell_y)) {
                    //         queue!(stdout, PrintStyledContent(DIM_STYLES[7].apply('•')))?;
                    //     } else {
                    //         queue!(stdout, Print(' '))?;
                    //     }
                    // } else {
                    queue!(stdout, Print(' '))?;
                    // }

                    continue;
                }

                let mut dirs = Direction::empty();

                if y > 0 && bmp[(y - 1) * bmp_width + x] {
                    dirs |= Direction::NORTH;
                }
                if y + 1 < bmp_height && bmp[(y + 1) * bmp_width + x] {
                    dirs |= Direction::SOUTH;
                }
                if x > 0 && bmp[y * bmp_width + (x - 1)] {
                    dirs |= Direction::WEST;
                }
                if x + 1 < bmp_width && bmp[y * bmp_width + (x + 1)] {
                    dirs |= Direction::EAST;
                }

                let horz_style = if x == 0 || x + 1 == bmp_width {
                    BorderStyle::Bold
                } else {
                    BorderStyle::Curved
                };

                let vert_style = if y == 0 || y + 1 == bmp_height {
                    BorderStyle::Bold
                } else {
                    BorderStyle::Curved
                };

                queue!(
                    stdout,
                    PrintStyledContent(STYLES[7].apply(dirs.border(horz_style, vert_style)))
                )?;
            }
        }

        stdout.flush()
    }

    fn render_bitmap(&self) {
        if self.bitmap.borrow().is_some() {
            return;
        }

        let (bmp_width, bmp_height) = self.bitmap_size();
        let mut bitmap = BitVec::repeat(false, bmp_width * bmp_height);

        bitmap.set(bmp_width, false); // Entrance

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
