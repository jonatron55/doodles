// Copyright (c) 2025 Jonathon Burnham Cobb
// Licensed under the MIT-0 license.

use std::{hash::RandomState, io::Result as IoResult};

use clap::Parser;
use crossterm::terminal;
use doodles::common::{
    color::Color,
    term::{CommonArgs, WaitResult, cleanup_term, setup_term},
    vec::uvec2,
};
use rand::RngExt;

use crate::ripples::{Medium, RenderStyle};

mod ripples;

/// Ripples terminal animation.
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about)]
pub struct Args {
    #[clap(flatten)]
    common: CommonArgs,

    /// Period of a wave in frames. If not specified, this will be randomly chosen for each animation.
    #[clap(short = 'P', long)]
    period: Option<usize>,

    /// Probability of spawning a new wave per frame.
    #[clap(short = 'p', long, default_value_t = 10.0)]
    spawnprob: f64,

    /// Initial amplitude of new waves.
    #[clap(short = 'a', long, default_value_t = 50.0)]
    amplitude: f64,

    /// Medium base color.
    #[clap(short = 'c', long)]
    color: Option<Color>,

    /// Color of wave peaks.
    #[clap(short = 'k', long)]
    peak_color: Option<Color>,

    /// Color of wave troughs.
    #[clap(short = 'u', long)]
    trough_color: Option<Color>,

    /// Rendering style
    #[clap(short = 's', long)]
    style: Option<RenderStyle>,

    /// Total number of frames to render.
    ///
    /// Once this limit is reached, no new waves will spawn and the existing waves will be allowed to die out. The
    /// animation will restart with a new random seed until `--iter` total animations have been completed (if
    /// specified).
    #[arg(short = 't', long)]
    frames: Option<usize>,
}

fn main() -> IoResult<()> {
    let args = Args::parse();

    setup_term()?;

    match args.common.wait()? {
        WaitResult::Exit => return cleanup_term(),
        _ => {}
    }

    let mut rand = rand::rng();
    let random_state = RandomState::new();

    // Outer loop
    'outer: loop {
        let (width, height) = terminal::size()?;

        let base_color = args.color.unwrap_or_else(|| {
            if rand.random_bool(0.5) {
                Color::choose_non_mono(&mut rand)
            } else {
                Color::Black
            }
        });

        let peak_color = args.peak_color.unwrap_or_else(|| {
            if base_color == Color::Black {
                Color::choose_non_mono(&mut rand)
            } else {
                Color::White
            }
        });

        let trough_color = args.trough_color.unwrap_or_else(|| {
            if peak_color == Color::White {
                Color::Black
            } else {
                peak_color.complement()
            }
        });

        let period = args.period.unwrap_or_else(|| rand.random_range(4..=16));

        let mut medium = Medium::new(
            uvec2(width as usize, height as usize),
            period,
            base_color,
            peak_color,
            trough_color,
            args.amplitude * 0.01,
        );

        let render_style = args.style.unwrap_or_else(|| RenderStyle::choose(&mut rand));

        // Inner simulation loop
        'sim: loop {
            medium.render(render_style, &random_state)?;
            match args.common.wait()? {
                WaitResult::Resize(_) | WaitResult::Next => continue 'outer,
                WaitResult::Exit => break 'outer,
                _ => {}
            }

            let spawnprob = if let Some(frames) = args.frames
                && medium.age() >= frames
            {
                0.0
            } else {
                args.spawnprob * 0.01
            };

            medium.next(spawnprob, &mut rand);

            if let Some(frames) = args.frames
                && medium.age() >= frames
                && medium.converged()
            {
                break 'sim;
            }
        }
    }

    cleanup_term()
}
