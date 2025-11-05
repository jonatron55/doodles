use std::{
    fs::OpenOptions,
    hash::RandomState,
    io::{Result as IoResult, stdout},
    path::PathBuf,
};

use clap::Parser;
use doodles::common::{CommonArgs, WaitResult, cleanup_term, setup_term};
use doodles::error;

use crate::board::Board;
use crate::term_renderer::render;

mod board;
mod term_renderer;

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
    /// Board files should be plain text and must contain a board size
    /// declaration on the first line in the format "width,height", followed by
    /// rows of cells where white spaces represent dead cells and any other
    /// alphanumeric character represents a living cell.
    ///
    /// If the number of rows or columns in the file does not match the declared
    /// size, excess cells will be discarded or missing cells will be treated
    /// empty.
    #[arg()]
    path: PathBuf,

    /// Number of generations to simulate before rendering.
    #[arg(short = 'n', long, default_value_t = 0)]
    generations: usize,

    /// Maximum number of generations to simulate (0 for no limit).
    ///
    /// Once this limit is reached, the board will reset to the initial state
    /// read from the file.
    #[arg(short = 'm', long, default_value_t = 0)]
    max: usize,
}

fn main() -> IoResult<()> {
    let args = Args::parse();

    // Load the board from the specified file.
    let file = OpenOptions::new().read(true).open(&args.path);
    let file = match file {
        Ok(file) => file,
        Err(err) => {
            error!("Could not open '{}': {err}", args.path.display());
            return Err(err);
        }
    };

    let initial_board = match Board::from_file(file) {
        Ok(board) => board,
        Err(err) => {
            error!("Could not read board from '{}': {err}", args.path.display());
            return Err(err);
        }
    };

    // Prewarm the board by simulating the specified number of generations.
    let mut board = initial_board.clone();
    for _ in 0..args.generations {
        board.next();
    }

    // Create a random state for rendering.
    let random_state = RandomState::new();

    setup_term()?;
    // Main simulation loop.
    loop {
        render(&board, &random_state)?;

        if args.common.wait()? == WaitResult::Exit {
            break;
        }

        board.next();

        if args.max > 0 && board.generation() >= args.max {
            board = initial_board.clone();
        }
    }

    cleanup_term()?;

    Ok(())
}
