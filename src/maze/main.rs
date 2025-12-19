// Copyright (c) 2025 Jonathon Burnham Cobb
// Licensed under the MIT-0 license.

use std::{
    fs,
    hash::RandomState,
    io::{Error as IoError, ErrorKind as IoErrorKind, Result as IoResult, stdout},
    path::PathBuf,
    str::FromStr,
};

use clap::{Parser, ValueEnum, builder::PossibleValue};
use crossterm::{
    execute,
    terminal::{self, Clear, ClearType},
};
use doodles::common::{
    color::Color,
    image::Image,
    term::{CommonArgs, WaitResult, cleanup_term, setup_term},
    vec::{UVec2, uvec2},
};
use rand::Rng;
use rand::seq::SliceRandom;

use crate::maze::{Maze, RenderStyle as MazeRenderStyle, WallStyle};
use crate::{
    agent::{Agent, RenderStyle as AgentRenderStyle},
    maze::BiasMode,
};

mod agent;
mod maze;

/// Generates and solves mazes.
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about=None)]
pub struct Args {
    #[clap(flatten)]
    common: CommonArgs,

    /// Maze render style.
    #[clap(short = 'm', long)]
    maze_style: Option<MazeRenderArg>,

    /// Maze wall color.
    #[clap(short = 'c', long)]
    color: Option<Color>,

    /// Agent render style.
    #[clap(short = 'a', long)]
    agent_style: Option<AgentRenderStyle>,

    /// Number of agents.
    #[clap(short = 'N', long, default_value_t = 6)]
    agents: usize,

    #[clap(flatten)]
    bias: BiasArg,
}

/// Maze render style argument.
#[derive(Clone, Copy, Debug)]
#[repr(u8)]
enum MazeRenderArg {
    /// Plain walls with single-line borders.
    Plain = 0,

    /// Bold outer walls with curved inner walls.
    Curved = 1,

    /// Double-line walls.
    Double = 2,

    /// Block-style walls.
    Block = 3,

    /// Block-style outer walls with hedge-style inner walls.
    BlockHedge = 4,

    /// Hedge-style walls.
    Hedge = 5,
}

/// Maze bias style argument.
#[derive(Parser, Clone, Debug)]
pub struct BiasArg {
    /// Passages are uniformly biased in one direction.
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
    /// This argument may specify a path to an image in a standard format, or one of the predefined patterns:
    ///
    /// - `checkered(width,height)`: A width by height checkered pattern.
    /// - `concentric(width)`: Concentric rings of width `width`.
    #[clap(short = 'I', long, conflicts_with = "bias")]
    pub image: Option<PathBuf>,
}

/// Predefined maze render styles corresponding to `MazeRenderArg`.
const MAZE_STYLES: [MazeRenderStyle; 6] = [
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
];

fn main() -> IoResult<()> {
    let args = Args::parse();

    setup_term()?;

    match args.common.wait()? {
        WaitResult::Exit => return cleanup_term(),
        _ => {}
    }

    let mut iteration = 0;
    let mut rand = rand::rng();

    'outer: loop {
        if let Some(max_iterations) = args.common.iter
            && iteration >= max_iterations
        {
            break 'outer;
        }

        execute!(stdout(), Clear(ClearType::All))?;

        let mut size = UVec2::from(terminal::size()?);
        let random_state = RandomState::new();

        let maze_style = args
            .maze_style
            .unwrap_or_else(|| MazeRenderArg::choose(&mut rand));
        let maze_style = MAZE_STYLES[maze_style as usize].clone();

        let maze_style =
            maze_style.with_color(args.color.unwrap_or_else(|| Color::choose(&mut rand)));

        let agent_style = args
            .agent_style
            .unwrap_or_else(|| AgentRenderStyle::choose(&mut rand));

        // Calculate maze dimensions based on terminal size. Each cell is 2x2 characters, plus a 1-character border.
        size = (size - UVec2::one()) / 2;
        let mut maze = Maze::new(size);

        let bias = if let Some(bias_image_path) = &args.bias.image {
            if fs::exists(bias_image_path)? {
                unimplemented!()
            } else {
                let s = bias_image_path.to_string_lossy();
                if s.starts_with("checkered")
                    && let Ok(check_size) = s["checkered".len()..].parse::<UVec2>()
                {
                    BiasMode::Image(Image::new_checkered(size, check_size))
                } else if s.starts_with("concentric")
                    && let Ok(ring_width) = s["concentric".len()..].parse::<usize>()
                {
                    BiasMode::Image(Image::new_concentric(size, ring_width))
                } else {
                    cleanup_term()?;
                    eprintln!(
                        "Bias image path '{}' does not exist.",
                        bias_image_path.display()
                    );
                    return Err(IoError::from(IoErrorKind::NotFound));
                }
            }
        } else if let Some(bias_value) = args.bias.bias {
            BiasMode::Uniform(bias_value.clamp(0.0, 1.0))
        } else {
            if rand.random_bool(0.5) {
                match rand.random_range(0..3) {
                    0 => BiasMode::Uniform(0.2),
                    1 => BiasMode::Uniform(0.7),
                    _ => BiasMode::Image(Image::new_checkered(size, uvec2(size.x / 5, size.y / 3))),
                }
            } else {
                BiasMode::Uniform(0.5)
            }
        };

        'build: loop {
            if !maze.build_next(&mut rand, &bias) {
                break 'build;
            }

            maze.render(&maze_style, &[], &agent_style, &random_state)?;

            match args.common.wait()? {
                WaitResult::Continue => {}
                WaitResult::Resize(_) => continue 'outer,
                WaitResult::Exit => break 'outer,
            }
        }

        let mut agents = (0..args.agents)
            .map(|i| Agent::new(&maze, Color::from((i as u8 % 7) + 1)))
            .collect::<Vec<_>>();
        agents.shuffle(&mut rand);

        let mut active_agents = 1;
        let mut frames = 0;

        'solve: loop {
            maze.render(
                &maze_style,
                &agents[0..active_agents],
                &agent_style,
                &random_state,
            )?;

            for agent in agents.iter_mut().take(active_agents) {
                agent.update(&mut rand);
            }

            frames += 1;
            if frames % 63 == 0 && active_agents < agents.len() {
                active_agents += 1;
            }

            if active_agents == agents.len() && agents.iter().all(|a| a.is_halted()) {
                break 'solve;
            }

            match args.common.wait()? {
                WaitResult::Continue => {}
                WaitResult::Resize(_) => continue 'outer,
                WaitResult::Exit => break 'outer,
            }
        }

        iteration += 1;
    }

    cleanup_term()?;

    Ok(())
}

impl MazeRenderArg {
    pub fn choose<R: Rng>(rand: &mut R) -> Self {
        let value = rand.random_range(0..6);
        MazeRenderArg::from(value)
    }
}

impl FromStr for MazeRenderArg {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Ok(value) = s.parse::<u8>() {
            Ok(MazeRenderArg::from(value))
        } else {
            let s = s.to_uppercase();
            match s.as_str() {
                "P" | "PLAIN" => Ok(MazeRenderArg::Plain),
                "C" | "CURVED" => Ok(MazeRenderArg::Curved),
                "D" | "DOUBLE" => Ok(MazeRenderArg::Double),
                "B" | "BLOCK" => Ok(MazeRenderArg::Block),
                "G" | "BLOCKHEDGE" => Ok(MazeRenderArg::BlockHedge),
                "H" | "HEDGE" => Ok(MazeRenderArg::Hedge),
                _ => Err(()),
            }
        }
    }
}

impl Into<u8> for MazeRenderArg {
    fn into(self) -> u8 {
        self as u8
    }
}

impl From<u8> for MazeRenderArg {
    fn from(value: u8) -> Self {
        match value % 6 {
            0 => MazeRenderArg::Plain,
            1 => MazeRenderArg::Curved,
            2 => MazeRenderArg::Double,
            3 => MazeRenderArg::Block,
            4 => MazeRenderArg::BlockHedge,
            5 => MazeRenderArg::Hedge,
            _ => unreachable!(),
        }
    }
}

impl ValueEnum for MazeRenderArg {
    fn value_variants<'a>() -> &'a [Self] {
        &[
            MazeRenderArg::Plain,
            MazeRenderArg::Curved,
            MazeRenderArg::Double,
            MazeRenderArg::Block,
            MazeRenderArg::BlockHedge,
            MazeRenderArg::Hedge,
        ]
    }

    fn to_possible_value(&self) -> Option<PossibleValue> {
        match self {
            MazeRenderArg::Plain => Some(PossibleValue::new("plain").alias("p").alias("0")),
            MazeRenderArg::Curved => Some(PossibleValue::new("curved").alias("c").alias("1")),
            MazeRenderArg::Double => Some(PossibleValue::new("double").alias("d").alias("2")),
            MazeRenderArg::Block => Some(PossibleValue::new("block").alias("b").alias("3")),
            MazeRenderArg::BlockHedge => {
                Some(PossibleValue::new("blockhedge").alias("g").alias("4"))
            }
            MazeRenderArg::Hedge => Some(PossibleValue::new("hedge").alias("h").alias("5")),
        }
    }
}
