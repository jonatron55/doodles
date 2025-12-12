// Copyright (c) 2025 Jonathon Burnham Cobb
// Licensed under the MIT-0 license.

use std::{
    borrow::Cow,
    fs,
    io::{Result as IoResult, stdout},
    path::PathBuf,
    str::FromStr,
};

use clap::{Parser, ValueEnum, builder::PossibleValue};
use crossterm::{
    cursor::MoveTo,
    execute,
    terminal::{self, Clear, ClearType},
};

use doodles::{
    common::{
        color::Color,
        term::{CommonArgs, WaitResult, cleanup_term, setup_term},
    },
    error,
};

mod board;

use board::Board;
use rand::Rng;

/// Digital rain terminal animation.
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about=None)]
pub struct Args {
    #[clap(flatten)]
    common: CommonArgs,

    /// Alphabet file to use.
    ///
    /// This should be a plain text file containing the possible characters to use in the effect. The predefined options
    /// "ascii", "cp850", and "droplets" are also available.
    ///
    /// If absent, a default alphabet of miscellaneous Latin, Greek, Cyrillic, punctuation, currency, control picture,
    /// and dingbat characters will be used.
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

    /// Color of the rain.
    #[arg(short = 'c', long)]
    color: Option<ColorArg>,

    /// Number of frames to "warm up" the effect before reaching full spawn probability.
    #[arg(short = 'W', long, default_value_t = 192)]
    warmup: usize,
}

#[derive(Clone, Copy, Debug)]
#[repr(u8)]
enum ColorArg {
    Color(Color),
    Cycle = 8,
    Random = 9,
}

fn main() -> IoResult<()> {
    let args = Args::parse();

    let mut stdout = stdout();

    setup_term()?;

    match args.common.wait()? {
        WaitResult::Exit => return cleanup_term(),
        _ => {}
    }

    let mut frame = 0;

    let (width, height) = terminal::size()?;
    let mut rand = rand::rng();
    let alphabet = match &args.alphabet {
        Some(path) => match fs::read_to_string(&path) {
            Ok(content) => Some(Cow::Owned(content)),
            Err(err) => match path.to_string_lossy().to_ascii_uppercase().as_str() {
                "ASCII" => Some(Cow::Borrowed(include_str!("ascii.txt"))),
                "CP850" => Some(Cow::Borrowed(include_str!("cp850.txt"))),
                "DROPLETS" => Some(Cow::Borrowed(include_str!("droplets.txt"))),
                _ => {
                    error!("Failed to read alphabet file {}: {}", path.display(), err);
                    return Err(err);
                }
            },
        },
        None => None,
    };

    let mut board = Board::new(width as usize, height as usize, alphabet.as_deref());
    let color = args.color.unwrap_or_else(|| ColorArg::choose(&mut rand));

    loop {
        let dead = args.common.iter.map_or(false, |max_iter| frame >= max_iter);

        frame += 1;

        board = if let Some(next_board) = board.next(&args, frame, &color, &mut rand, dead) {
            next_board
        } else {
            break;
        };

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

impl ColorArg {
    pub fn choose<R: Rng>(rand: &mut R) -> Self {
        match rand.random_range(1..10) {
            8 => ColorArg::Cycle,
            9 => ColorArg::Random,
            n => ColorArg::Color(Color::from(n)),
        }
    }
}

impl FromStr for ColorArg {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Ok(value) = s.parse::<u8>() {
            Ok(ColorArg::from(value))
        } else {
            let s = s.to_uppercase();
            if let Ok(color) = <Color as FromStr>::from_str(&s) {
                Ok(ColorArg::Color(color))
            } else if s == "CYCLE" || s == "L" {
                Ok(ColorArg::Cycle)
            } else if s == "RANDOM" || s == "A" {
                Ok(ColorArg::Random)
            } else {
                Err(())
            }
        }
    }
}

impl Into<u8> for ColorArg {
    fn into(self) -> u8 {
        match self {
            ColorArg::Color(color) => color.into(),
            ColorArg::Cycle => 8,
            ColorArg::Random => 9,
        }
    }
}

impl From<u8> for ColorArg {
    fn from(value: u8) -> Self {
        match value % 10 {
            8 => ColorArg::Cycle,
            9 => ColorArg::Random,
            v => ColorArg::Color(Color::from(v)),
        }
    }
}

impl ValueEnum for ColorArg {
    fn value_variants<'a>() -> &'a [Self] {
        &[
            ColorArg::Color(Color::Black),
            ColorArg::Color(Color::Red),
            ColorArg::Color(Color::Green),
            ColorArg::Color(Color::Yellow),
            ColorArg::Color(Color::Blue),
            ColorArg::Color(Color::Magenta),
            ColorArg::Color(Color::Cyan),
            ColorArg::Color(Color::White),
            ColorArg::Cycle,
            ColorArg::Random,
        ]
    }

    fn to_possible_value(&self) -> Option<PossibleValue> {
        match self {
            ColorArg::Color(color) => color.to_possible_value(),
            ColorArg::Cycle => Some(PossibleValue::new("cycle").alias("l").alias("8")),
            ColorArg::Random => Some(PossibleValue::new("random").alias("a").alias("9")),
        }
    }
}
