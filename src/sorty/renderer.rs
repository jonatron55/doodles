// Copyright (c) 2025 Jonathon Burnham Cobb
// Licensed under the MIT-0 license.

use std::{
    cmp::Ordering,
    io::{Result as IoResult, Write, stdout},
    str::FromStr,
};

use bitvec::bitvec;
use clap::{ValueEnum, builder::PossibleValue};
use crossterm::{cursor::MoveTo, queue, style::PrintStyledContent};
use doodles::common::{
    color::Color,
    term::{DIM_STYLES, STYLES},
    vec::UVec2,
};
use rand::Rng;

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum RenderStyle {
    Block = 0,
    Dots = 1,
    Fraction = 2,
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

            let styles = if y < whole || (y == whole && frac > 0) {
                &STYLES
            } else {
                &DIM_STYLES
            };

            let style = if changed[x] {
                styles[(colors[1] as usize) % styles.len()]
            } else {
                styles[(colors[0] as usize) % styles.len()]
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
    pub fn choose<R: Rng>(rand: &mut R) -> Self {
        let value = rand.random_range(0..4);
        RenderStyle::from(value)
    }
}

impl FromStr for RenderStyle {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Ok(value) = s.parse::<u8>() {
            Ok(RenderStyle::from(value))
        } else {
            let s = s.to_uppercase();
            match s.as_str() {
                "B" | "BLACK" => Ok(RenderStyle::Block),
                "D" | "DOTS" => Ok(RenderStyle::Dots),
                "F" | "FRACTION" => Ok(RenderStyle::Fraction),
                "O" | "OCTAL" => Ok(RenderStyle::Octal),
                _ => Err(()),
            }
        }
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

impl ValueEnum for RenderStyle {
    fn value_variants<'a>() -> &'a [Self] {
        &[
            RenderStyle::Block,
            RenderStyle::Dots,
            RenderStyle::Fraction,
            RenderStyle::Octal,
        ]
    }

    fn to_possible_value(&self) -> Option<PossibleValue> {
        match self {
            RenderStyle::Block => Some(PossibleValue::new("block").alias("b").alias("0")),
            RenderStyle::Dots => Some(PossibleValue::new("dots").alias("d").alias("1")),
            RenderStyle::Fraction => Some(PossibleValue::new("fraction").alias("f").alias("2")),
            RenderStyle::Octal => Some(PossibleValue::new("octal").alias("o").alias("3")),
        }
    }
}
