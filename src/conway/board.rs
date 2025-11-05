use std::io::{
    BufRead, BufReader, Error as IoError, ErrorKind as IoErrorKind, Read, Result as IoResult,
};

/// Represents the state of a Conway's Game of Life board.
///
/// In addition to the usual rules of Conway's Game of Life, this implementation
/// includes colored cells, which modify the rules as follows:
///
/// 1. Any live cell with fewer than two live neighbours **of the same color**
///    dies, as if by underpopulation.
/// 2. Any live cell with two or three live neighbours **of the same color**
///    survives.
/// 3. Any live cell with more than three live neighbours **of any color** dies,
///    as if by overpopulation.
/// 4. Any dead cell with exactly three live neighbours **of the same color**
///    becomes a live cell, as if by reproduction.
///
/// The "color" values are arbitrary numbers; it is up to renderers to decide
/// how to display them.
#[derive(Clone)]
pub struct Board {
    width: usize,
    height: usize,
    cell_buffers: [Vec<Cell>; 2],
    generation: usize,
}

/// Maximum age a cell can reach before becoming empty.
const MAX_AGE: u32 = 1024;

/// Represents a single cell on the board.
///
/// Cells may be alive, dead, or empty. If the cell is alive, this value is
/// `Some(color)` and `age` is 0. Dead cells that are not empty have
/// `Some(color)` and `age` > 0.
///
/// With respect to the Game of Life rules, there is no distinction between dead
/// cells of different ages and empty cells; however, the age is may be used by
/// renderers to display visual effects such as fading.
#[derive(Clone, Copy)]
pub struct Cell {
    /// The color of the cell, or `None` if the cell is empty.
    ///
    /// The actual color value is arbitrary; renders may display it however they
    /// choose.
    pub color: Option<u32>,

    /// The age of the cell.
    ///
    /// A cell with age 0 is alive. If the cell dies, its age becomes 1 and
    /// increments with each generation until it reaches `MAX_AGE`, at which
    /// point the cell becomes empty.
    pub age: u32,
}

impl Board {
    /// Creates a new empty board with the given dimensions.
    pub fn _new(width: usize, height: usize) -> Self {
        Board {
            width,
            height,
            cell_buffers: [
                vec![Cell::empty(); width * height],
                vec![Cell::empty(); width * height],
            ],
            generation: 0,
        }
    }

    /// Loads a board from the given reader.
    ///
    /// The reader should provide a plain text representation of the board,
    /// with the first line containing the dimensions in the format
    /// `width,height` followed by rows of cells where white spaces represent
    /// empty cells and any other alphanumeric character represents a living
    /// cell. The color of a living cell is determined by converting the
    /// character to a base-36 digit.
    ///
    /// If the number of rows or columns in the input does not match the
    /// declared size, excess cells will be discarded or missing cells will be
    /// treated as empty.
    ///
    /// Arguments
    /// =========
    ///
    /// - `reader` - A reader providing the board's plain text representation.
    ///
    /// Returns
    /// =======
    ///
    /// `Ok(Board)` if the board was successfully loaded, or a
    /// [`std::io::Error`] if any problems occurred during reading or parsing.
    pub fn from_file<R: Read>(reader: R) -> IoResult<Self> {
        let mut reader = BufReader::new(reader);
        let mut line = String::new();

        reader.read_line(&mut line)?;
        let dims: Vec<_> = line.split(',').map(|s| s.trim().parse::<usize>()).collect();
        if dims.len() != 2 || dims.iter().any(|r| r.is_err()) {
            return Err(IoError::new(
                IoErrorKind::InvalidData,
                "Invalid board dimensions",
            ));
        }

        let width = *dims[0].as_ref().unwrap();
        let height = *dims[1].as_ref().unwrap();

        let mut cells = Vec::with_capacity(width * height);
        for _ in 0..height {
            line.clear();
            if reader.read_line(&mut line)? == 0 {
                for _ in 0..width {
                    cells.push(Cell::empty());
                }
                continue;
            }

            let mut chars = line.chars();
            for _ in 0..width {
                let ch = chars.next();

                match ch {
                    Some(ch) if ch.is_whitespace() => cells.push(Cell::empty()),
                    Some(ch) if ch.is_alphanumeric() => {
                        let color = ch.to_digit(36).unwrap();
                        cells.push(Cell::new(color));
                    }
                    Some(other) => {
                        return Err(IoError::new(
                            IoErrorKind::InvalidData,
                            format!("Invalid character '{other}'"),
                        ));
                    }
                    None => cells.push(Cell::empty()),
                }
            }
        }

        Ok(Board {
            width,
            height,
            cell_buffers: [cells, vec![Cell::empty(); width * height]],
            generation: 0,
        })
    }

    /// Returns the dimensions of the board as (width, height).
    pub fn size(&self) -> (usize, usize) {
        (self.width, self.height)
    }

    /// Returns a reference to the cell at the given coordinates.
    pub fn cell(&self, x: usize, y: usize) -> &Cell {
        let x = x % self.width;
        let y = y % self.height;
        let i = y * self.width + x;
        &self.current_buffer()[i]
    }

    /// Returns the current generation number (the number of times
    /// [`Board::next`] has been called).
    pub fn generation(&self) -> usize {
        self.generation
    }

    /// Advances the board to the next generation by one simulation step.
    pub fn next(&mut self) {
        let mut neighbors = Vec::with_capacity(8);

        for y in 0..self.height {
            for x in 0..self.width {
                neighbors.clear();

                for dy in -1..=1 {
                    for dx in -1..=1 {
                        if dx == 0 && dy == 0 {
                            continue;
                        }

                        neighbors.push(*self.cell(
                            ((x as isize + dx + self.width as isize) % self.width as isize)
                                as usize,
                            ((y as isize + dy + self.height as isize) % self.height as isize)
                                as usize,
                        ));
                    }
                }

                let i = y * self.width + x;
                self.next_buffer()[i] = self.current_buffer()[i].next(&neighbors);
            }
        }

        self.generation += 1;
    }

    fn current_buffer(&self) -> &Vec<Cell> {
        &self.cell_buffers[self.generation % 2]
    }

    fn next_buffer(&mut self) -> &mut Vec<Cell> {
        &mut self.cell_buffers[(self.generation + 1) % 2]
    }
}

impl Cell {
    /// Creates a new living cell with the given color.
    pub fn new(color: u32) -> Self {
        Cell {
            color: Some(color),
            age: 0,
        }
    }

    /// Creates a new empty cell.
    pub fn empty() -> Self {
        Cell {
            color: None,
            age: 0,
        }
    }

    /// Returns `true` if the cell is alive or `false` if it is dead or empty.
    pub fn is_alive(&self) -> bool {
        self.color.is_some() && self.age == 0
    }

    /// Returns `true` if the cell is empty or `false` if it is or was ever
    /// alive.
    pub fn is_empty(&self) -> bool {
        self.color.is_none()
    }

    /// Computes the next state of the cell based on its neighbors.
    ///
    /// The cell's next state is determined by these rules:
    ///
    /// 1. Any live cell with fewer than two live neighbours of the same color
    ///    dies, as if by underpopulation.
    /// 2. Any live cell with two or three live neighbours of the same color
    ///    survives.
    /// 3. Any live cell with more than three live neighbours of any color dies,
    ///    as if by overpopulation.
    /// 4. Any dead cell with exactly three live neighbours of the same color
    ///    becomes a live cell, as if by reproduction.
    ///
    /// Arguments
    /// =========
    ///
    /// - `neighbors` - A slice of neighboring cells. This should contain
    ///   exactly eight cells that are orthogonally and diagonally adjacent to
    ///   this cell.
    ///
    /// Returns
    /// =======
    ///
    /// A new [`Cell`] representing the next state.
    pub fn next(&self, neighbors: &[Cell]) -> Self {
        let living_neighbors = neighbors.iter().filter(|c| c.is_alive()).count();
        let like_neighbors = neighbors
            .iter()
            .filter(|c| c.color == self.color && c.is_alive())
            .count();

        if self.is_alive() {
            if like_neighbors < 2 || like_neighbors > 3 {
                // Cell dies and begins aging
                Cell {
                    color: self.color,
                    age: 1,
                }
            } else {
                // Cell survives
                self.clone()
            }
        } else {
            // Determine if all living neighbors share the same color
            let neighbor_color = {
                let mut alive = neighbors.iter().filter(|c| c.is_alive());
                let mut color = match alive.next() {
                    Some(cell) => cell.color,
                    None => None,
                };
                for cell in alive {
                    if cell.color != color {
                        color = None;
                        break;
                    }
                }
                color
            };

            if living_neighbors == 3 && neighbor_color.is_some() {
                // Cell becomes alive
                Cell {
                    color: neighbor_color,
                    age: 0,
                }
            } else if self.age < MAX_AGE {
                // Dead cell ages
                Cell {
                    color: self.color,
                    age: self.age + 1,
                }
            } else {
                // Cell is dead and fully aged, becomes empty
                Cell::empty()
            }
        }
    }
}

impl Default for Cell {
    fn default() -> Self {
        Cell::empty()
    }
}
