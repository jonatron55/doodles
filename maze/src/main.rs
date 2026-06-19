// Copyright (c) 2025 Jonathon Burnham Cobb
// Licensed under the MIT-0 license.

use std::{hash::RandomState, io::stdout};

use anyhow::Result as AnyResult;
use clap::{Parser, ValueEnum};
use crossterm::{
    execute,
    terminal::{self, Clear, ClearType},
};
use doodle::{
    color::Color,
    image::Image,
    term::{CommonArgs, WaitResult, cleanup_term, setup_term},
    vec::UVec2,
};
use rand::{Rng, RngExt, seq::SliceRandom};

use crate::{
    agent::{Agent, RenderStyle as AgentRenderStyle},
    maze::generator::{DfsMazeBuilder, PrimsMazeBuilder, WilsonsMazeBuilder},
    maze::{BiasMode, Maze, MazeBuilder, RenderStyle as MazeRenderStyle, WallStyle},
    trinket::Trinket,
};

mod agent;
mod maze;
mod trinket;

/// Generates and solves mazes.
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about)]
pub struct Args {
    #[clap(flatten)]
    common: CommonArgs,

    /// Maze generation algorithm.
    ///
    /// `dfs` and `prims` both converge in linear time, but `dfs` tends to produce mazes with long, winding  passages,
    /// whereas `prims` produces mazes with many short dead ends. `wilsons` algorithm produces mazes with a uniform
    /// distribution of passage lengths, but converges more slowly than the other two algorithms, especially for larger
    /// mazes.
    ///
    /// If not specified, a random algorithm will be chosen for each maze.
    #[clap(short = 'a', long = "algo", alias = "algorithm")]
    algorithm: Option<MazeAlgorithm>,

    /// Maze render style.
    #[clap(short = 'm', long)]
    maze_style: Option<MazeRenderArg>,

    /// Maze wall color.
    #[clap(short = 'c', long)]
    color: Option<Color>,

    /// Agent render style.
    #[clap(short = 'A', long)]
    agent_style: Option<AgentRenderStyle>,

    /// Number of agents.
    #[clap(short = 'N', long, default_value_t = 6)]
    agents: usize,

    /// Place random trinkets throughout the maze.
    #[clap(short = 't', long, default_value_t = false)]
    trinkets: bool,

    /// Prevent trinkets from being placed in the maze.
    #[clap(short = 'T', long, default_value_t = false, conflicts_with = "trinkets")]
    no_trinkets: bool,

    #[clap(flatten)]
    bias: BiasArg,
}

/// Maze render style argument.
#[derive(Clone, Copy, Debug, ValueEnum)]
#[repr(u8)]
#[clap(rename_all = "kebab-case")]
enum MazeRenderArg {
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
enum MazeAlgorithm {
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
const MAZE_STYLES: [MazeRenderStyle; 8] = [
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

fn main() -> AnyResult<()> {
    let args = Args::parse();

    setup_term()?;

    match args.common.wait()? {
        WaitResult::Exit => return Ok(cleanup_term()?),
        _ => {}
    }

    let mut iteration = 0;
    let mut rand = rand::rng();

    'outer: loop {
        if let Some(max_iterations) = args.common.max_iterations
            && iteration >= max_iterations
        {
            break 'outer;
        }

        execute!(stdout(), Clear(ClearType::All))?;

        let mut size = UVec2::from(terminal::size()?);
        let random_state = RandomState::new();

        let maze_style = args.maze_style.unwrap_or_else(|| MazeRenderArg::choose(&mut rand));
        let maze_style = MAZE_STYLES[maze_style as usize].clone();

        let maze_style = maze_style.with_color(args.color.unwrap_or_else(|| Color::choose(&mut rand)));

        let agent_style = args.agent_style.unwrap_or_else(|| AgentRenderStyle::choose(&mut rand));

        // Calculate maze dimensions based on terminal size. Each cell is 2x2 characters, plus a 1-character border.
        size = (size - UVec2::ONE) / 2;
        let mut maze = Maze::new(size);

        let bias = if let Some(bias_image_path) = &args.bias.image {
            match Image::from_str(bias_image_path, size) {
                Ok(image) => BiasMode::Image(image),
                Err(e) => {
                    cleanup_term()?;
                    eprintln!("Failed to load bias image '{}': {e}", bias_image_path);
                    return Err(e.into());
                }
            }
        } else if let Some(bias_value) = args.bias.bias {
            BiasMode::Uniform(bias_value.clamp(0.0, 1.0))
        } else {
            if rand.random_bool(0.5) {
                match rand.random_range(0..6) {
                    0 => BiasMode::Uniform(0.2),
                    1 => BiasMode::Uniform(0.7),
                    2 => BiasMode::Image(Image::random_graphic(size, &mut rand)),
                    3 => BiasMode::Image(Image::default_checkered(size)),
                    4 => BiasMode::Image(Image::random_concentric(size, &mut rand)),
                    _ => BiasMode::Image(Image::random_gradient(size, &mut rand)),
                }
            } else {
                BiasMode::Uniform(0.5)
            }
        };

        let algorithm = args.algorithm.unwrap_or_else(|| MazeAlgorithm::choose(&mut rand));

        let mut builder = match algorithm {
            MazeAlgorithm::Dfs => MazeBuilder::Dfs(DfsMazeBuilder::new(&mut maze, &mut rand)),
            MazeAlgorithm::Prims => MazeBuilder::Prims(PrimsMazeBuilder::new(&mut maze, &mut rand, &bias)),
            MazeAlgorithm::Wilsons => MazeBuilder::Wilsons(WilsonsMazeBuilder::new(&mut maze, &mut rand)),
        };

        'build: loop {
            if !builder.build_next(&mut rand, &bias) {
                break 'build;
            }

            builder.render(&maze_style, &random_state)?;

            match args.common.wait()? {
                WaitResult::Continue => {}
                WaitResult::Next | WaitResult::Resize(_) => continue 'outer,
                WaitResult::Exit => break 'outer,
            }
        }

        drop(builder);

        let mut trinkets = if args.trinkets || (!args.no_trinkets && rand.random_bool(0.5)) {
            let mut dead_ends: Vec<_> = maze.dead_ends().collect();
            dead_ends.shuffle(&mut rand);
            Trinket::new_collection(&dead_ends)
        } else {
            vec![]
        };

        for i in 0..trinkets.len() {
            maze.render(&maze_style, &[], &trinkets[0..i], &agent_style, &random_state)?;
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
                &trinkets,
                &agent_style,
                &random_state,
            )?;

            for trinket in trinkets.iter_mut() {
                trinket.update();
            }

            for agent in agents.iter_mut().take(active_agents) {
                agent.update(&mut trinkets, &mut rand);
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
                WaitResult::Next | WaitResult::Resize(_) => continue 'outer,
                WaitResult::Exit => break 'outer,
            }
        }

        iteration += 1;
    }

    cleanup_term()?;

    Ok(())
}

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
