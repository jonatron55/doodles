use clap::{Parser, ValueEnum};

use doodle::color::Color;
use rand::{Rng, RngExt};

use crate::maze::{RenderStyle as MazeRenderStyle, WallStyle};

/// Maze render style argument.
#[derive(Clone, Copy, Debug, ValueEnum)]
#[repr(u8)]
#[clap(rename_all = "kebab-case")]
pub enum MazeRenderArg {
    /// Plain walls with single-line borders.
    #[clap(alias = "p", alias = "0")]
    Plain = 0,

    /// Bold outer walls with curved inner walls.
    #[clap(alias = "c", alias = "1")]
    Curved = 1,

    /// Double-line walls.
    #[clap(alias = "d", alias = "2")]
    Double = 2,

    /// Block-style walls.
    #[clap(alias = "b", alias = "3")]
    Block = 3,

    /// Block-style outer walls with hedge-style inner walls.
    #[clap(alias = "bh", alias = "4")]
    BlockHedge = 4,

    /// Hedge-style walls.
    #[clap(alias = "h", alias = "5")]
    Hedge = 5,

    /// Block-style outer walls with fence-style inner walls.
    #[clap(alias = "bf", alias = "6")]
    BlockFence = 6,

    /// Fence-style walls.
    #[clap(alias = "f", alias = "7")]
    Fence = 7,
}

#[derive(Clone, Copy, Debug, ValueEnum)]
#[clap(rename_all = "kebab-case")]
pub enum MazeAlgorithm {
    /// Randomized depth-first search.
    Dfs,

    /// Prim’s algorithm (randomized minimum spanning tree).
    Prims,

    /// Wilson’s algorithm (loop-erased random walks).
    Wilsons,
}

/// Maze bias style argument.
#[derive(Parser, Clone, Debug)]
pub struct BiasArg {
    /// Set a uniform direction bias for passage generation.
    ///
    /// A value of 0 causes a completely horizontal bias, and a value of 1 causes a completely vertical bias. A value of
    /// 0.5 results in no bias.
    #[clap(short = 'b', long)]
    pub bias: Option<f64>,

    /// Image file to use for biasing passage generation.
    ///
    /// The brightness of each pixel influences the direction of passages in the corresponding maze cell. Darker pixels
    /// favor horizontal passages, while lighter pixels favor vertical passages.
    ///
    /// This argument may specify a path to an image in a standard format, or one of the following special cases:
    ///
    /// - "smiley": a smiley face emoji.
    /// - "skull": a skull emoji.
    /// - "checkered({width},{height})": a checkerboard pattern with the given check size.
    /// - "concentric{width}": concentric rings with the given ring width.
    /// - "hgrad": a horizontal gradient from black to white.
    /// - "vgrad": a vertical gradient from black to white.
    #[clap(short = 'I', long, conflicts_with = "bias")]
    pub image: Option<String>,
}

/// Predefined maze render styles corresponding to `MazeRenderArg`.
pub const MAZE_STYLES: [MazeRenderStyle; 8] = [
    MazeRenderStyle {
        outer: WallStyle::Solid,
        inner: WallStyle::Solid,
        color: Color::White,
    },
    MazeRenderStyle {
        outer: WallStyle::Bold,
        inner: WallStyle::Curved,
        color: Color::White,
    },
    MazeRenderStyle {
        outer: WallStyle::Double,
        inner: WallStyle::Double,
        color: Color::White,
    },
    MazeRenderStyle {
        outer: WallStyle::Block,
        inner: WallStyle::Block,
        color: Color::White,
    },
    MazeRenderStyle {
        outer: WallStyle::Block,
        inner: WallStyle::Hedge,
        color: Color::White,
    },
    MazeRenderStyle {
        outer: WallStyle::Hedge,
        inner: WallStyle::Hedge,
        color: Color::White,
    },
    MazeRenderStyle {
        outer: WallStyle::Block,
        inner: WallStyle::Fence,
        color: Color::White,
    },
    MazeRenderStyle {
        outer: WallStyle::Fence,
        inner: WallStyle::Fence,
        color: Color::White,
    },
];

impl MazeAlgorithm {
    pub fn choose(rand: &mut impl Rng) -> Self {
        match rand.random_range(0..3) {
            0 => MazeAlgorithm::Dfs,
            1 => MazeAlgorithm::Wilsons,
            2 => MazeAlgorithm::Prims,
            _ => unreachable!(),
        }
    }
}

impl MazeRenderArg {
    pub fn choose(rand: &mut impl Rng) -> Self {
        let value = rand.random_range(0..8);
        MazeRenderArg::from(value)
    }
}

impl Into<u8> for MazeRenderArg {
    fn into(self) -> u8 {
        self as u8
    }
}

impl From<u8> for MazeRenderArg {
    fn from(value: u8) -> Self {
        match value & 7 {
            0 => MazeRenderArg::Plain,
            1 => MazeRenderArg::Curved,
            2 => MazeRenderArg::Double,
            3 => MazeRenderArg::Block,
            4 => MazeRenderArg::BlockHedge,
            5 => MazeRenderArg::Hedge,
            6 => MazeRenderArg::BlockFence,
            7 => MazeRenderArg::Fence,
            _ => unreachable!(),
        }
    }
}
