// Copyright (c) 2025 Jonathon Burnham Cobb
// Licensed under the MIT-0 license.

use std::{
    fs,
    io::{Result as IoResult, stdout},
    path::PathBuf,
};

use clap::{Parser, ValueEnum};
use crossterm::{
    cursor::MoveTo,
    execute,
    terminal::{self, Clear, ClearType},
};

use doodles::{
    common::term::{CommonArgs, WaitResult, cleanup_term, setup_term, validate_color},
    error,
};

#[derive(Clone, Copy, Debug, ValueEnum)]
enum ColorChoice {
    Black,
    Red,
    Green,
    Yellow,
    Blue,
    Magenta,
    Cyan,
    White,
}

mod board;

use board::Board;

/// Digital rain terminal animation.
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about=None)]
pub struct Args {
    #[clap(flatten)]
    common: CommonArgs,

    /// Alphabet file to use.
    #[arg(short = 'a', long)]
    alphabet: Option<PathBuf>,

    /// How long each character lives (in frames).
    #[arg(short = 'l', long, default_value_t = 8)]
    lifespan: u32,

    /// Maximum trail length for each stream.
    #[arg(short = 'T', long, default_value_t = 32)]
    max_trail: u32,

    /// Minimum trail length for each stream.
    #[arg(short = 't', long, default_value_t = 8)]
    min_trail: u32,

    /// Probability of spawning a new stream in each cell per frame.
    #[arg(short = 'p', long, default_value_t = 0.005)]
    spawnprob: f64,

    /// Color of the rain (0-7).
    #[arg(short = 'c', long, default_value_t = 2, value_parser = validate_color)]
    color: usize,
}

fn main() -> IoResult<()> {
    let args = Args::parse();

    let mut stdout = stdout();

    setup_term()?;

    let (width, height) = terminal::size()?;
    let mut rand = rand::rng();
    let alphabet = match &args.alphabet {
        Some(path) => match fs::read_to_string(&path) {
            Ok(content) => Some(content),
            Err(err) => {
                error!("Failed to read alphabet file {}: {}", path.display(), err);
                return Err(err);
            }
        },
        None => None,
    };

    let mut board = Board::new(width as usize, height as usize, alphabet.as_deref());

    loop {
        board = board.next(&args, &mut rand);
        board.render(&args)?;

        match args.common.wait()? {
            WaitResult::Resize(width, height) => {
                execute!(stdout, MoveTo(0, 0), Clear(ClearType::All))?;
                board = board.resize(width, height);
            }
            WaitResult::Continue => continue,
            WaitResult::Exit => break,
        }
    }

    cleanup_term()?;

    Ok(())
}
