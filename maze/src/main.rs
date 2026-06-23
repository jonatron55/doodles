// Copyright (c) 2025 Jonathon Burnham Cobb
// Licensed under the MIT-0 license.

mod agent;
mod args;
mod maze;
mod trinket;

use std::{hash::RandomState, io::stdout};

use anyhow::Result as AnyResult;
use clap::Parser;
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
use rand::{RngExt, seq::SliceRandom};

use crate::{
    agent::{Agent, RenderStyle as AgentRenderStyle},
    args::{BiasArg, MAZE_STYLES, MazeAlgorithm, MazeRenderArg},
    maze::{
        BiasMode, Maze, MazeBuilder,
        generator::{DfsMazeBuilder, PrimsMazeBuilder, WilsonsMazeBuilder},
    },
    trinket::Trinket,
};

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
