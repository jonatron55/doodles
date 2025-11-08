// Copyright (c) 2025 Jonathon Burnham Cobb
// Licensed under the MIT-0 license.

use std::{fs::OpenOptions, hash::RandomState, io::Result as IoResult, path::PathBuf};

use clap::Parser;
use crossterm::terminal;
use doodles::common::term::{CommonArgs, WaitResult, cleanup_term, setup_term};
use doodles::error;

use crate::board::Board;
use crate::renderer::render;

mod board;
mod renderer;

/// Conway's Game of Life simulator and renderer.
///
/// This program reads an initial board configuration from a file, and simulates
/// Conway's Game of Life, rendering the board to the terminal using colored
/// output.
///
/// In addition to the usual rules of Conway's Game of Life, this implementation
/// includes colored cells, which modifies the rules as follows:
///
/// 1. Any live cell with fewer than two live neighbours **of the same color**
///    dies, as if by underpopulation.
///
/// 2. Any live cell with two or three live neighbours **of the same color**
///    survives.
///
/// 3. Any live cell with more than three live neighbours **of any color** dies,
///    as if by overpopulation.
///
/// 4. Any dead cell with exactly three live neighbours **of the same color**
///    becomes a live cell, as if by reproduction.
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about)]
struct Args {
    #[clap(flatten)]
    common: CommonArgs,

    /// Path to the board file to render.
    ///
    /// Board files should be plain text and contain rows of cells where white
    /// spaces represent dead cells and any other alphanumeric character
    /// represents a living cell. If no path is provided, a random board will be
    /// generated.
    #[arg()]
    path: Option<PathBuf>,

    /// Maximum number of generations to simulate (0 for no limit).
    ///
    /// Once this limit is reached, the board will reset to the initial state
    /// read from the file.
    ///
    /// If not specified, the simulation will continue until the board
    /// converges to either a stable or oscillating state.
    #[arg(short = 'm', long, default_value_t = 0)]
    max: usize,
}

fn main() -> IoResult<()> {
    let args = Args::parse();

    setup_term()?;

    // Outer loop
    'outer: loop {
        let (width, height) = terminal::size()?;
        let mut board = Board::new(width as usize, height as usize);

        board = if let Some(path) = &args.path {
            // Load the board from the specified file.
            let file = OpenOptions::new().read(true).open(&path);
            let file = match file {
                Ok(file) => file,
                Err(err) => {
                    error!("Could not open '{}': {err}", path.display());
                    return Err(err);
                }
            };

            match board.with_cells_from_file(file) {
                Ok(board) => board,
                Err(err) => {
                    error!("Could not read board from '{}': {err}", path.display());
                    return Err(err);
                }
            }
        } else {
            let mut rand = rand::rng();
            board.with_random_cells(&mut rand, 0.33)
        };

        // Create a random state for rendering.
        let random_state = RandomState::new();

        // Inner simulation loop
        'sim: loop {
            render(&board, &random_state)?;

            if args.common.wait()? == WaitResult::Exit {
                break 'outer;
            }

            board.next();

            if board.converged() || args.max > 0 && board.generation() >= args.max {
                break 'sim;
            }
        }
    }
    cleanup_term()?;

    Ok(())
}
