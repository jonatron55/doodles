// Copyright (c) 2025 Jonathon Burnham Cobb
// Licensed under the MIT-0 license.

use bitflags::bitflags;
use rand::{Rng, seq::IteratorRandom};

use crate::common::{
    borders::*,
    vec::{UVec2, uvec2},
};

/// A single cardinal direction.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Direction {
    North = 0b0001,
    East = 0b0010,
    South = 0b0100,
    West = 0b1000,
}

bitflags! {
    /// A set of cardinal directions.
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub struct Directions: u8 {
        const NORTH = 0b0001;
        const EAST  = 0b0010;
        const SOUTH = 0b0100;
        const WEST  = 0b1000;
    }
}

impl Direction {
    pub const ALL: [Self; 4] = [Direction::North, Direction::East, Direction::South, Direction::West];

    /// Choose a random direction.
    pub fn choose(rand: &mut impl Rng) -> Self {
        match rand.random_range(0..4) {
            0 => Direction::North,
            1 => Direction::East,
            2 => Direction::South,
            3 => Direction::West,
            _ => unreachable!(),
        }
    }

    /// Returns a list of directions shuffled with the given bias.
    ///
    /// The `bias` parameter controls the likelihood of horizontal directions appearing before vertical directions. A
    /// bias of `1.0` means vertical directions will always come first, while a bias of `0.0` means horizontal
    /// directions will always come first. A bias of `0.5` results in a uniform random shuffle.
    pub fn biased_shuffle(rand: &mut impl Rng, bias: f64) -> [Self; 4] {
        let horz = if rand.random_bool(0.5) {
            (Direction::East, Direction::West)
        } else {
            (Direction::West, Direction::East)
        };
        let vert = if rand.random_bool(0.5) {
            (Direction::North, Direction::South)
        } else {
            (Direction::South, Direction::North)
        };

        if rand.random_bool(bias) {
            [vert.0, vert.1, horz.0, horz.1]
        } else {
            [horz.0, horz.1, vert.0, vert.1]
        }
    }

    /// Returns the opposite direction.
    pub fn opposite(self) -> Direction {
        match self {
            Direction::North => Direction::South,
            Direction::East => Direction::West,
            Direction::South => Direction::North,
            Direction::West => Direction::East,
        }
    }

    /// Returns the clockwise direction.
    pub fn clockwise(self) -> Direction {
        match self {
            Direction::North => Direction::East,
            Direction::East => Direction::South,
            Direction::South => Direction::West,
            Direction::West => Direction::North,
        }
    }

    /// Returns the counterclockwise direction.
    pub fn counterclockwise(self) -> Direction {
        match self {
            Direction::North => Direction::West,
            Direction::East => Direction::North,
            Direction::South => Direction::East,
            Direction::West => Direction::South,
        }
    }

    /// Moves an integer point one step in this direction.
    ///
    /// The point uses terminal coordinates, with the y-axis increasing downward:
    ///
    /// - [`Direction::North`] will decrease *y*
    /// - [`Direction::East`] will increase *x*
    /// - [`Direction::South`] will increase *y*
    /// - [`Direction::West`] will decrease *x*
    ///
    /// The coordinates are not checked for overflow or underflow.
    pub fn move_point(&self, point: UVec2) -> UVec2 {
        let UVec2 { x, y } = point;
        match self {
            Direction::North => uvec2(x, y - 1),
            Direction::East => uvec2(x + 1, y),
            Direction::South => uvec2(x, y + 1),
            Direction::West => uvec2(x - 1, y),
        }
    }

    /// Moves an integer point one step in this direction without exceeding the given bounds.
    ///
    /// The point uses terminal coordinates, with the y-axis increasing downward:
    ///
    /// - [`Direction::North`] will decrease *y*
    /// - [`Direction::East`] will increase *x*
    /// - [`Direction::South`] will increase *y*
    /// - [`Direction::West`] will decrease *x*
    ///
    /// The coordinates are checked against the given width and height, and `None` is returned
    /// if the move would exceed the bounds.
    pub fn move_point_within(&self, point: UVec2, bounds: UVec2) -> Option<UVec2> {
        let UVec2 { x, y } = point;
        let UVec2 { x: w, y: h } = bounds;
        match self {
            Direction::North if y > 0 => Some(uvec2(x, y - 1)),
            Direction::East if x + 1 < w => Some(uvec2(x + 1, y)),
            Direction::South if y + 1 < h => Some(uvec2(x, y + 1)),
            Direction::West if x > 0 => Some(uvec2(x - 1, y)),
            _ => None,
        }
    }
}

impl Directions {
    /// Choose a random direction from the set.
    pub fn choose(&self, rand: &mut impl Rng) -> Option<Direction> {
        self.iter().choose(rand).and_then(|d| d.try_into().ok())
    }

    /// Returns the border character for this combination of vertical and horizontal styles.
    pub fn border(self, vertical_style: BorderStyle, horizontal_style: BorderStyle) -> char {
        let borders = match (vertical_style, horizontal_style) {
            (BorderStyle::Single, BorderStyle::Single) => &BORDERS_SINGLE,
            (BorderStyle::Curved, BorderStyle::Curved) => &BORDERS_CURVED,
            (BorderStyle::Bold, BorderStyle::Bold) => &BORDERS_BOLD,
            (BorderStyle::Bold, BorderStyle::Single) | (BorderStyle::Bold, BorderStyle::Curved) => &BORDERS_BOLD_SINGLE,
            (BorderStyle::Single, BorderStyle::Bold) | (BorderStyle::Curved, BorderStyle::Bold) => &BORDERS_SINGLE_BOLD,
            (BorderStyle::Double, BorderStyle::Double) => &BORDERS_DOUBLE,
            (BorderStyle::Double, BorderStyle::Single) | (BorderStyle::Double, BorderStyle::Curved) => {
                &BORDERS_DOUBLE_SINGLE
            }
            (BorderStyle::Single, BorderStyle::Double) | (BorderStyle::Curved, BorderStyle::Double) => {
                &BORDERS_SINGLE_DOUBLE
            }
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
