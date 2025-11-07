use std::io::Result as IoResult;

use clap::Parser;
use crossterm::terminal;
use doodles::common::term::{CommonArgs, WaitResult, cleanup_term, setup_term};
use rand::Rng;

use crate::agent::Agent;
use crate::maze::{Maze, RenderStyle, WallStyle};

mod agent;
mod maze;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about=None)]
pub struct Args {
    #[clap(flatten)]
    common: CommonArgs,
}

const STYLES: [RenderStyle; 4] = [
    RenderStyle {
        outer: WallStyle::Solid,
        inner: WallStyle::Solid,
        color: 7,
    },
    RenderStyle {
        outer: WallStyle::Bold,
        inner: WallStyle::Curved,
        color: 7,
    },
    RenderStyle {
        outer: WallStyle::Double,
        inner: WallStyle::Double,
        color: 7,
    },
    RenderStyle {
        outer: WallStyle::Block,
        inner: WallStyle::Block,
        color: 7,
    },
];

fn main() -> IoResult<()> {
    let args = Args::parse();

    setup_term()?;

    'outer: loop {
        let mut rand = rand::rng();
        let (mut width, mut height) = terminal::size()?;

        let style = STYLES[rand.random_range(0..STYLES.len())].clone();
        let style = style.with_color(rand.random_range(1..=7));

        width /= 2;
        width -= 1;

        height /= 2;
        height -= 1;

        let mut maze = Maze::new(width as usize, height as usize);

        'build: loop {
            if !maze.build_next(&mut rand) {
                break 'build;
            }

            maze.render(&style, &[])?;

            match args.common.wait()? {
                WaitResult::Continue => {}
                WaitResult::Resize(_, _) => continue 'outer,
                WaitResult::Exit => break 'outer,
            }
        }
        let mut agents = [
            Agent::new(1),
            Agent::new(2),
            Agent::new(3),
            Agent::new(4),
            Agent::new(5),
            Agent::new(6),
            Agent::new(7),
        ];
        let mut active_agents = 1;
        let mut frames = 0;

        'run: loop {
            maze.render(&style, &agents[0..active_agents])?;

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
