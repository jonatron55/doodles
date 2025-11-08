// Copyright (c) 2025 Jonathon Burnham Cobb
// Licensed under the MIT-0 license.

use std::hash::RandomState;
use std::io::{Result as IoResult, stdout};

use clap::Parser;
use crossterm::{
    execute,
    terminal::{self, Clear, ClearType},
};
use doodles::common::term::{CommonArgs, WaitResult, cleanup_term, setup_term};
use rand::Rng;
use rand::seq::SliceRandom;

use crate::agent::{Agent, RenderStyle as AgentRenderStyle};
use crate::maze::{Maze, RenderStyle as MazeRenderStyle, WallStyle};

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
    maze_style: Option<usize>,

    /// Maze wall color.
    #[clap(short = 'c', long)]
    color: Option<u8>,

    /// Agent render style.
    #[clap(short = 'a', long)]
    agent_style: Option<usize>,

    /// Number of agents.
    #[clap(short = 'n', long, default_value_t = 4)]
    agents: usize,
}

const AGENT_STYLES: [AgentRenderStyle; 3] = [
    AgentRenderStyle::Smiley,
    AgentRenderStyle::Inchworm,
    AgentRenderStyle::Turtle,
];

const MAZE_STYLES: [MazeRenderStyle; 6] = [
    MazeRenderStyle {
        outer: WallStyle::Solid,
        inner: WallStyle::Solid,
        color: 7,
    },
    MazeRenderStyle {
        outer: WallStyle::Bold,
        inner: WallStyle::Curved,
        color: 7,
    },
    MazeRenderStyle {
        outer: WallStyle::Double,
        inner: WallStyle::Double,
        color: 7,
    },
    MazeRenderStyle {
        outer: WallStyle::Block,
        inner: WallStyle::Block,
        color: 7,
    },
    MazeRenderStyle {
        outer: WallStyle::Block,
        inner: WallStyle::Hedge,
        color: 7,
    },
    MazeRenderStyle {
        outer: WallStyle::Hedge,
        inner: WallStyle::Hedge,
        color: 7,
    },
];

fn main() -> IoResult<()> {
    let args = Args::parse();

    setup_term()?;

    'outer: loop {
        execute!(stdout(), Clear(ClearType::All))?;

        let mut rand = rand::rng();
        let (mut width, mut height) = terminal::size()?;
        let random_state = RandomState::new();

        let maze_style = args
            .maze_style
            .unwrap_or_else(|| rand.random_range(0..MAZE_STYLES.len()));
        let maze_style = MAZE_STYLES[maze_style % MAZE_STYLES.len()].clone();

        let maze_style =
            maze_style.with_color(args.color.unwrap_or_else(|| rand.random_range(1..8) % 8));

        let agent_style = args
            .agent_style
            .unwrap_or_else(|| rand.random_range(0..AGENT_STYLES.len()));
        let agent_style = AGENT_STYLES[agent_style % AGENT_STYLES.len()].clone();

        width /= 2;
        width -= 1;

        height /= 2;
        height -= 1;

        let mut maze = Maze::new(width as usize, height as usize);

        'build: loop {
            if !maze.build_next(&mut rand) {
                break 'build;
            }

            maze.render(&maze_style, &[], &agent_style, &random_state)?;

            match args.common.wait()? {
                WaitResult::Continue => {}
                WaitResult::Resize(_, _) => continue 'outer,
                WaitResult::Exit => break 'outer,
            }
        }

        let mut agents = (0..args.agents)
            .map(|i| Agent::new(&maze, ((i + 1) % 8) as u8))
            .collect::<Vec<_>>();
        agents.shuffle(&mut rand);

        let mut active_agents = 1;
        let mut frames = 0;

        'run: loop {
            maze.render(
                &maze_style,
                &agents[0..active_agents],
                &agent_style,
                &random_state,
            )?;

            for agent in agents.iter_mut().take(active_agents) {
                agent.update(&maze, &mut rand);
            }

            frames += 1;
            if frames % 63 == 0 && active_agents < agents.len() {
                active_agents += 1;
            }

            if active_agents == agents.len() && agents.iter().all(|a| a.is_halted()) {
                break 'run;
            }

            match args.common.wait()? {
                WaitResult::Continue => {}
                WaitResult::Resize(_, _) => continue 'outer,
                WaitResult::Exit => break 'outer,
            }
        }
    }

    cleanup_term()?;

    Ok(())
}
