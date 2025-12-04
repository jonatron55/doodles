// Copyright (c) 2025 Jonathon Burnham Cobb
// Licensed under the MIT-0 license.

use std::{cmp::Ordering, io::Result as IoResult};

use clap::{Parser, ValueEnum};
use crossterm::terminal;
use doodles::common::term::{CommonArgs, WaitResult, cleanup_term, setup_term};
use rand::{Rng, random_bool, seq::SliceRandom};

use crate::{
    bubble::{BubbleState, step_bubble},
    qsort::{QsortState, step_qsort},
    renderer::RenderStyle,
};

mod bubble;
mod qsort;
mod renderer;

/// sorty sort animation.
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about=None)]
pub struct Args {
    #[clap(flatten)]
    common: CommonArgs,

    /// Rendering style (0-3).
    #[arg(short = 's', long)]
    style: Option<usize>,

    /// Inactive color (0-7).
    #[arg(short = 'c', long)]
    color1: Option<u8>,

    /// Active color (0-7).
    #[arg(short = 'C', long)]
    color2: Option<u8>,

    /// Sort in descending order.
    #[arg(short = 'd', long)]
    descending: bool,

    /// Sort in ascending order.
    #[arg(short = 'u', long, conflicts_with = "descending")]
    ascending: bool,

    /// Sorting algorithm.
    #[arg(short = 'a', long, default_value = "qsort")]
    algo: Algorithm,
}

#[derive(ValueEnum, Clone, Debug)]
enum Algorithm {
    Bubble,
    Qsort,
}

enum SortState {
    Bubble(BubbleState),
    QSort(QsortState),
}

fn main() -> IoResult<()> {
    let args = Args::parse();

    setup_term()?;

    match args.common.wait()? {
        WaitResult::Exit => return cleanup_term(),
        _ => {}
    }

    let mut iteration = 0;

    'outer: loop {
        if let Some(max_iterations) = args.common.iter
            && iteration >= max_iterations
        {
            break 'outer;
        }

        let (width, height) = terminal::size()?;
        let (width, height) = (width as usize, height as usize);

        let mut rand = rand::rng();

        let mut actual: Vec<usize> = (0..width).map(|x| 8 * x * height / width).collect();
        let mut displayed: Vec<usize> = vec![0; width];

        let ordering = if args.ascending {
            Ordering::Greater
        } else if args.descending {
            Ordering::Less
        } else if random_bool(0.5) {
            Ordering::Greater
        } else {
            Ordering::Less
        };

        let colors = [
            args.color1.unwrap_or_else(|| rand.random_range(0..8) as u8) % 8,
            args.color2.unwrap_or_else(|| rand.random_range(1..8) as u8) % 8,
        ];

        let style = match args.style.unwrap_or_else(|| rand.random_range(0..4)) % 4 {
            0 => RenderStyle::Block,
            1 => match ordering {
                Ordering::Greater => RenderStyle::DotsAsc,
                _ => RenderStyle::DotsDesc,
            },
            2 => RenderStyle::Fraction,
            3 => RenderStyle::Octal,
            _ => unreachable!(),
        };

        actual.shuffle(&mut rand);

        let mut sort_state = match args.algo {
            Algorithm::Bubble => SortState::Bubble(BubbleState::new()),
            Algorithm::Qsort => SortState::QSort(QsortState::new(&actual)),
        };

        while !match &mut sort_state {
            SortState::Bubble(state) => step_bubble(&mut actual, width, ordering, state),
            SortState::QSort(state) => step_qsort(&mut actual, ordering, state),
        } {
            while displayed != actual {
                renderer::render(&mut displayed, &actual, width, height, colors, style)?;

                match args.common.wait()? {
                    WaitResult::Continue => {}
                    WaitResult::Resize(_, _) => continue 'outer,
                    WaitResult::Exit => break 'outer,
                }
            }
        }

        iteration += 1;
    }

    cleanup_term()
}
