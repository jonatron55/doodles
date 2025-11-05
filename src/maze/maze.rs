use bitflags::bitflags;

pub struct Maze {
    width: usize,
    height: usize,
    cells: Vec<Cell>,
}

pub struct Cell {
    walls: Walls,
    visited: bool,
}

bitflags! {
    pub struct Walls: u8 {
        const NORTH = 0b0001;
        const EAST  = 0b0010;
        const SOUTH = 0b0100;
        const WEST  = 0b1000;
    }
}
