use std::io::{Result as IoResult, stdout};

use clap::Parser;
use crossterm::terminal;
use doodles::common::term::{CommonArgs, WaitResult, cleanup_term, setup_term};

use crate::maze::Maze;

mod maze;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about=None)]
pub struct Args {
    #[clap(flatten)]
    common: CommonArgs,
}

fn main() -> IoResult<()> {
    let args = Args::parse();

    setup_term()?;

    'outer: loop {
        let mut rand = rand::rng();
        let (mut width, mut height) = terminal::size()?;
        width /= 2;
        width -= 1;

        height /= 2;
        height -= 1;

        let mut maze = Maze::new(width as usize, height as usize);

        'inner: loop {
            if !maze.build_next(&mut rand) {
                break 'inner;
            }

            maze.render()?;

            match args.common.wait()? {
                WaitResult::Continue => {}
                WaitResult::Resize(_, _) => continue 'outer,
                WaitResult::Exit => break 'outer,
            }
        }

        for _ in 0..128 {
            maze.render()?;

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
