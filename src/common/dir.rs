use bitflags::bitflags;
use rand::Rng;

bitflags! {
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub struct Direction: u8 {
        const NORTH = 0b0001;
        const EAST  = 0b0010;
        const SOUTH = 0b0100;
        const WEST  = 0b1000;
    }
}

pub enum BorderStyle {
    Single,
    Curved,
    Bold,
    Double,
}

const BORDERS_SINGLE: [char; 16] = [
    ' ', // 0b0000 (NONE)
    '╵', // 0b0001 (NORTH)
    '╶', // 0b0010 (EAST)
    '└', // 0b0011 (NORTH | EAST)
    '╷', // 0b0100 (SOUTH)
    '│', // 0b0101 (NORTH | SOUTH)
    '┌', // 0b0110 (SOUTH | EAST)
    '├', // 0b0111 (NORTH | EAST | SOUTH)
    '╴', // 0b1000 (WEST)
    '┘', // 0b1001 (NORTH | WEST)
    '─', // 0b1010 (EAST | WEST)
    '┴', // 0b1011 (NORTH | EAST | WEST)
    '┐', // 0b1100 (SOUTH | WEST)
    '┤', // 0b1101 (NORTH | SOUTH | WEST)
    '┬', // 0b1110 (SOUTH | EAST | WEST)
    '┼', // 0b1111 (NORTH | EAST | SOUTH | WEST)
];

const BORDERS_CURVED: [char; 16] = [
    ' ', // 0b0000 (NONE)
    '╵', // 0b0001 (NORTH)
    '╶', // 0b0010 (EAST)
    '╰', // 0b0011 (NORTH | EAST)
    '╷', // 0b0100 (SOUTH)
    '│', // 0b0101 (NORTH | SOUTH)
    '╭', // 0b0110 (SOUTH | EAST)
    '├', // 0b0111 (NORTH | EAST | SOUTH)
    '╴', // 0b1000 (WEST)
    '╯', // 0b1001 (NORTH | WEST)
    '─', // 0b1010 (EAST | WEST)
    '┴', // 0b1011 (NORTH | EAST | WEST)
    '╮', // 0b1100 (SOUTH | WEST)
    '┤', // 0b1101 (NORTH | SOUTH | WEST)
    '┬', // 0b1110 (SOUTH | EAST | WEST)
    '┼', // 0b1111 (NORTH | EAST | SOUTH | WEST)
];

const BORDERS_DOUBLE: [char; 16] = [
    ' ', // 0b0000 (NONE)
    '╨', // 0b0001 (NORTH)
    '╞', // 0b0010 (EAST)
    '╚', // 0b0011 (NORTH | EAST)
    '╥', // 0b0100 (SOUTH)
    '║', // 0b0101 (NORTH | SOUTH)
    '╔', // 0b0110 (SOUTH | EAST)
    '╠', // 0b0111 (NORTH | EAST | SOUTH)
    '╡', // 0b1000 (WEST)
    '╝', // 0b1001 (NORTH | WEST)
    '═', // 0b1010 (EAST | WEST)
    '╩', // 0b1011 (NORTH | EAST | WEST)
    '╗', // 0b1100 (SOUTH | WEST)
    '╣', // 0b1101 (NORTH | SOUTH | WEST)
    '╦', // 0b1110 (SOUTH | EAST | WEST)
    '╬', // 0b1111 (NORTH | EAST | SOUTH | WEST)
];

const BORDERS_DOUBLE_SINGLE: [char; 16] = [
    ' ', // 0b0000 (NONE)
    '╨', // 0b0001 (NORTH)
    '╶', // 0b0010 (EAST)
    '╙', // 0b0011 (NORTH | EAST)
    '╥', // 0b0100 (SOUTH)
    '║', // 0b0101 (NORTH | SOUTH)
    '╓', // 0b0110 (SOUTH | EAST)
    '╟', // 0b0111 (NORTH | EAST | SOUTH)
    '╴', // 0b1000 (WEST)
    '╜', // 0b1001 (NORTH | WEST)
    '─', // 0b1010 (EAST | WEST)
    '╨', // 0b1011 (NORTH | EAST | WEST)
    '╖', // 0b1100 (SOUTH | WEST)
    '╢', // 0b1101 (NORTH | SOUTH | WEST)
    '╥', // 0b1110 (SOUTH | EAST | WEST)
    '╫', // 0b1111 (NORTH | EAST | SOUTH | WEST)
];

const BORDERS_SINGLE_DOUBLE: [char; 16] = [
    ' ', // 0b0000 (NONE)
    '╵', // 0b0001 (NORTH)
    '╞', // 0b0010 (EAST)
    '╘', // 0b0011 (NORTH | EAST)
    '╷', // 0b0100 (SOUTH)
    '│', // 0b0101 (NORTH | SOUTH)
    '╒', // 0b0110 (SOUTH | EAST)
    '╞', // 0b0111 (NORTH | EAST | SOUTH)
    '╡', // 0b1000 (WEST)
    '╛', // 0b1001 (NORTH | WEST)
    '═', // 0b1010 (EAST | WEST)
    '╧', // 0b1011 (NORTH | EAST | WEST)
    '╕', // 0b1100 (SOUTH | WEST)
    '╡', // 0b1101 (NORTH | SOUTH | WEST)
    '╤', // 0b1110 (SOUTH | EAST | WEST)
    '╪', // 0b1111 (NORTH | EAST | SOUTH | WEST)
];

const BORDERS_BOLD: [char; 16] = [
    ' ', // 0b0000 (NONE)
    '╹', // 0b0001 (NORTH)
    '╺', // 0b0010 (EAST)
    '┗', // 0b0011 (NORTH | EAST)
    '╻', // 0b0100 (SOUTH)
    '┃', // 0b0101 (NORTH | SOUTH)
    '┏', // 0b0110 (SOUTH | EAST)
    '┣', // 0b0111 (NORTH | EAST | SOUTH)
    '╸', // 0b1000 (WEST)
    '┛', // 0b1001 (NORTH | WEST)
    '━', // 0b1010 (EAST | WEST)
    '┻', // 0b1011 (NORTH | EAST | WEST)
    '┓', // 0b1100 (SOUTH | WEST)
    '┫', // 0b1101 (NORTH | SOUTH | WEST)
    '┳', // 0b1110 (SOUTH | EAST | WEST)
    '╋', // 0b1111 (NORTH | EAST | SOUTH | WEST)
];

const BORDERS_BOLD_SINGLE: [char; 16] = [
    ' ', // 0b0000 (NONE)
    '╹', // 0b0001 (NORTH)
    '╶', // 0b0010 (EAST)
    '┖', // 0b0011 (NORTH | EAST)
    '╻', // 0b0100 (SOUTH)
    '┃', // 0b0101 (NORTH | SOUTH)
    '┎', // 0b0110 (SOUTH | EAST)
    '┠', // 0b0111 (NORTH | EAST | SOUTH)
    '╴', // 0b1000 (WEST)
    '┚', // 0b1001 (NORTH | WEST)
    '─', // 0b1010 (EAST | WEST)
    '┸', // 0b1011 (NORTH | EAST | WEST)
    '┒', // 0b1100 (SOUTH | WEST)
    '┨', // 0b1101 (NORTH | SOUTH | WEST)
    '┰', // 0b1110 (SOUTH | EAST | WEST)
    '╂', // 0b1111 (NORTH | EAST | SOUTH | WEST)
];

const BORDERS_SINGLE_BOLD: [char; 16] = [
    ' ', // 0b0000 (NONE)
    '╵', // 0b0001 (NORTH)
    '╺', // 0b0010 (EAST)
    '┕', // 0b0011 (NORTH | EAST)
    '╷', // 0b0100 (SOUTH)
    '│', // 0b0101 (NORTH | SOUTH)
    '┍', // 0b0110 (SOUTH | EAST)
    '┝', // 0b0111 (NORTH | EAST | SOUTH)
    '╸', // 0b1000 (WEST)
    '┙', // 0b1001 (NORTH | WEST)
    '━', // 0b1010 (EAST | WEST)
    '┷', // 0b1011 (NORTH | EAST | WEST)
    '┑', // 0b1100 (SOUTH | WEST)
    '┥', // 0b1101 (NORTH | SOUTH | WEST)
    '┯', // 0b1110 (SOUTH | EAST | WEST)
    '┿', // 0b1111 (NORTH | EAST | SOUTH | WEST)
];

impl Direction {
    pub fn choose<R: Rng>(rand: &mut R) -> Self {
        match rand.random_range(0..4) {
            0 => Direction::NORTH,
            1 => Direction::EAST,
            2 => Direction::SOUTH,
            3 => Direction::WEST,
            _ => unreachable!(),
        }
    }

    pub fn opposite(self) -> Direction {
        let mut opposite = Direction::empty();
        if self.contains(Direction::NORTH) {
            opposite |= Direction::SOUTH;
        }
        if self.contains(Direction::EAST) {
            opposite |= Direction::WEST;
        }
        if self.contains(Direction::SOUTH) {
            opposite |= Direction::NORTH;
        }
        if self.contains(Direction::WEST) {
            opposite |= Direction::EAST;
        }
        opposite
    }

    pub fn border(self, vertical_style: BorderStyle, horizontal_style: BorderStyle) -> char {
        let borders =
            match (vertical_style, horizontal_style) {
                (BorderStyle::Single, BorderStyle::Single) => &BORDERS_SINGLE,
                (BorderStyle::Curved, BorderStyle::Curved) => &BORDERS_CURVED,
                (BorderStyle::Bold, BorderStyle::Bold) => &BORDERS_BOLD,
                (BorderStyle::Bold, BorderStyle::Single)
                | (BorderStyle::Bold, BorderStyle::Curved) => &BORDERS_BOLD_SINGLE,
                (BorderStyle::Single, BorderStyle::Bold)
                | (BorderStyle::Curved, BorderStyle::Bold) => &BORDERS_SINGLE_BOLD,
                (BorderStyle::Double, BorderStyle::Double) => &BORDERS_DOUBLE,
                (BorderStyle::Double, BorderStyle::Single)
                | (BorderStyle::Double, BorderStyle::Curved) => &BORDERS_DOUBLE_SINGLE,
                (BorderStyle::Single, BorderStyle::Double)
                | (BorderStyle::Curved, BorderStyle::Double) => &BORDERS_SINGLE_DOUBLE,
                _ => &BORDERS_SINGLE,
            };

        borders[self.bits() as usize]
    }
}
