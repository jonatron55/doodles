// Copyright (c) 2025 Jonathon Burnham Cobb
// Licensed under the MIT-0 license.

use std::{cmp::Ordering, io::Result as IoResult};

use clap::{Parser, ValueEnum};
use crossterm::terminal;
use doodles::common::{
    color::Color,
    term::{CommonArgs, WaitResult, cleanup_term, setup_term},
};
use rand::{random_bool, seq::SliceRandom};

use crate::{
    bubble::{BubbleState, step_bubble},
    qsort::{QsortState, step_qsort},
    renderer::RenderStyle,
};

mod bubble;
mod qsort;
mod renderer;

/// Visualizes different sorting algorithms.
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about=None)]
pub struct Args {
    #[clap(flatten)]
    common: CommonArgs,

    /// Rendering style.
    #[arg(short = 's', long)]
    style: Option<RenderStyle>,

    /// Inactive color.
    #[arg(short = 'c', long)]
    color1: Option<Color>,

    /// Active color.
    #[arg(short = 'C', long)]
    color2: Option<Color>,

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

    let (width, height) = terminal::size()?;
    let (mut width, mut height) = (width as usize, height as usize);

    let mut rand = rand::rng();

    let mut actual: Vec<usize> = (0..width).map(|x| 8 * x * height / width).collect();
    let mut displayed: Vec<usize> = vec![0; width];

    let mut iteration = 0;

    'outer: loop {
        if let Some(max_iterations) = args.common.iter
            && iteration >= max_iterations
        {
            break 'outer;
        }

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
            args.color1.unwrap_or_else(|| Color::choose(&mut rand)),
            args.color2.unwrap_or_else(|| Color::choose(&mut rand)),
        ];

        let style = args.style.unwrap_or_else(|| RenderStyle::choose(&mut rand));

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
                renderer::render(
                    &mut displayed,
                    &actual,
                    width,
                    height,
                    colors,
                    style,
                    ordering,
                )?;

                match args.common.wait()? {
                    WaitResult::Continue => {}
                    WaitResult::Resize(new_width, new_height) => {
                        (width, height) = (new_width, new_height);
                        actual = (0..width).map(|x| 8 * x * height / width).collect();
                        displayed = vec![0; width];
                        continue 'outer;
                    }
                    WaitResult::Exit => break 'outer,
                }
            }
        }

        iteration += 1;
    }

    cleanup_term()
}
