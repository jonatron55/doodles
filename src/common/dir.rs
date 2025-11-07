use bitflags::bitflags;
use rand::{Rng, seq::IteratorRandom};

use crate::common::borders::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Direction {
    North,
    East,
    South,
    West,
}

bitflags! {
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub struct Directions: u8 {
        const NORTH = 0b0001;
        const EAST  = 0b0010;
        const SOUTH = 0b0100;
        const WEST  = 0b1000;
    }
}

impl Direction {
    pub fn choose<R: Rng>(rand: &mut R) -> Self {
        match rand.random_range(0..4) {
            0 => Direction::North,
            1 => Direction::East,
            2 => Direction::South,
            3 => Direction::West,
            _ => unreachable!(),
        }
    }

    pub fn opposite(self) -> Direction {
        match self {
            Direction::North => Direction::South,
            Direction::East => Direction::West,
            Direction::South => Direction::North,
            Direction::West => Direction::East,
        }
    }

    pub fn clockwise(self) -> Direction {
        match self {
            Direction::North => Direction::East,
            Direction::East => Direction::South,
            Direction::South => Direction::West,
            Direction::West => Direction::North,
        }
    }

    pub fn counterclockwise(self) -> Direction {
        match self {
            Direction::North => Direction::West,
            Direction::East => Direction::North,
            Direction::South => Direction::East,
            Direction::West => Direction::South,
        }
    }
}

impl Directions {
    pub fn choose<R: Rng>(&self, rand: &mut R) -> Option<Direction> {
        self.iter().choose(rand).and_then(|d| d.try_into().ok())
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

impl Into<Directions> for Direction {
    fn into(self) -> Directions {
        match self {
            Direction::North => Directions::NORTH,
            Direction::East => Directions::EAST,
            Direction::South => Directions::SOUTH,
            Direction::West => Directions::WEST,
        }
    }
}

impl TryInto<Direction> for Directions {
    type Error = ();

    fn try_into(self) -> Result<Direction, Self::Error> {
        match self {
            Directions::NORTH => Ok(Direction::North),
            Directions::EAST => Ok(Direction::East),
            Directions::SOUTH => Ok(Direction::South),
            Directions::WEST => Ok(Direction::West),
            _ => Err(()),
        }
    }
}
