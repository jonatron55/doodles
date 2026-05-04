// Copyright (c) 2025 Jonathon Burnham Cobb
// Licensed under the MIT-0 license.

use std::{
    cmp::Ordering,
    io::{Result as IoResult, Write, stdout},
};

use bitvec::bitvec;
use clap::ValueEnum;
use crossterm::{cursor::MoveTo, queue, style::PrintStyledContent};
use doodle::{color::Color, vec::UVec2};
use rand::{Rng, RngExt};

#[derive(Debug, Clone, Copy, ValueEnum)]
#[repr(u8)]
#[clap(rename_all = "kebab-case")]
pub enum RenderStyle {
    #[clap(alias = "b", alias = "0")]
    Block = 0,

    #[clap(alias = "d", alias = "1")]
    Dots = 1,

    #[clap(alias = "f", alias = "2")]
    Fraction = 2,

    #[clap(alias = "o", alias = "3")]
    Octal = 3,
}

const BLOCK_GLYPHS: [&str; 9] = [" ", "▁", "▂", "▃", "▄", "▅", "▆", "▇", "█"];
const DOT_GLYPHS_ASC: [&str; 9] = [" ", "⢀", "⣀", "⣠", "⣤", "⣴", "⣶", "⣾", "⣿"];
const DOT_GLYPHS_DESC: [&str; 9] = [" ", "⡀", "⣀", "⣄", "⣤", "⣦", "⣶", "⣷", "⣿"];
const FRACTION_GLYPHS: [&str; 9] = ["0", "⅛", "¼", "⅜", "½", "⅝", "¾", "⅞", "1"];
const OCTAL_GLYPHS: [&str; 9] = ["0", "1", "2", "3", "4", "5", "6", "7", "8"];

pub fn render(
    displayed: &mut [usize],
    actual: &[usize],
    size: UVec2,
    colors: [Color; 2],
    style: RenderStyle,
    ordering: Ordering,
) -> IoResult<bool> {
    let mut stdout = stdout();

    let mut converged = true;
    let mut changed = bitvec![0; size.x];

    let glyphs = match style {
        RenderStyle::Block => BLOCK_GLYPHS,
        RenderStyle::Dots => match ordering {
            Ordering::Less => DOT_GLYPHS_ASC,
            _ => DOT_GLYPHS_DESC,
        },
        RenderStyle::Fraction => FRACTION_GLYPHS,
        RenderStyle::Octal => OCTAL_GLYPHS,
    };

    for x in 0..size.x {
        changed.set(
            x,
            if displayed[x] < actual[x] {
                displayed[x] += 1;
                true
            } else if displayed[x] > actual[x] {
                displayed[x] -= 1;
                true
            } else {
                false
            },
        );

        if changed[x] {
            converged = false;
        }
    }

    for y in 0..size.y {
        queue!(stdout, MoveTo(0, y as u16),)?;

        for x in 0..size.x {
            let value = displayed[x];
            let y = size.y - 1 - y;

            let frac = value % 8;
            let whole = value / 8;

            let color = if changed[x] { colors[1] } else { colors[0] };

            let style = if y < whole || (y == whole && frac > 0) {
                color.style()
            } else {
                color.dim_style()
            };

            let glyph = if y < whole {
                glyphs[8]
            } else if y == whole {
                glyphs[frac]
            } else {
                glyphs[0]
            };

            queue!(stdout, PrintStyledContent(style.apply(glyph)))?;
        }
    }

    stdout.flush()?;

    Ok(converged)
}

impl RenderStyle {
    pub fn choose(rand: &mut impl Rng) -> Self {
        let value = rand.random_range(0..4);
        RenderStyle::from(value)
    }
}

impl Into<u8> for RenderStyle {
    fn into(self) -> u8 {
        self as u8
    }
}

impl From<u8> for RenderStyle {
    fn from(value: u8) -> Self {
        match value % 4 {
            0 => RenderStyle::Block,
            1 => RenderStyle::Dots,
            2 => RenderStyle::Fraction,
            3 => RenderStyle::Octal,
            _ => unreachable!(),
        }
    }
}
