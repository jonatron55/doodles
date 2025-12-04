// Copyright (c) 2025 Jonathon Burnham Cobb
// Licensed under the MIT-0 license.

use std::io::{Result as IoResult, Write, stdout};

use bitvec::bitvec;
use crossterm::{cursor::MoveTo, queue, style::PrintStyledContent};
use doodles::common::term::{DIM_STYLES, STYLES};

#[derive(Clone, Copy)]
pub enum RenderStyle {
    Block,
    DotsAsc,
    DotsDesc,
    Fraction,
    Octal,
}

const BLOCK_GLYPHS: [&str; 9] = [" ", "▁", "▂", "▃", "▄", "▅", "▆", "▇", "█"];
const DOT_GLYPHS_ASC: [&str; 9] = [" ", "⢀", "⣀", "⣠", "⣤", "⣴", "⣶", "⣾", "⣿"];
const DOT_GLYPHS_DESC: [&str; 9] = [" ", "⡀", "⣀", "⣄", "⣤", "⣦", "⣶", "⣷", "⣿"];
const FRACTION_GLYPHS: [&str; 9] = ["0", "⅛", "¼", "⅜", "½", "⅝", "¾", "⅞", "1"];
const OCTAL_GLYPHS: [&str; 9] = ["0", "1", "2", "3", "4", "5", "6", "7", "8"];

pub fn render(
    displayed: &mut [usize],
    actual: &[usize],
    width: usize,
    height: usize,
    colors: [u8; 2],
    style: RenderStyle,
) -> IoResult<bool> {
    let mut stdout = stdout();

    let mut converged = true;
    let mut changed = bitvec![0; width];

    let glyphs = match style {
        RenderStyle::Block => BLOCK_GLYPHS,
        RenderStyle::DotsAsc => DOT_GLYPHS_ASC,
        RenderStyle::DotsDesc => DOT_GLYPHS_DESC,
        RenderStyle::Fraction => FRACTION_GLYPHS,
        RenderStyle::Octal => OCTAL_GLYPHS,
    };

    for x in 0..width {
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

    for y in 0..height {
        queue!(stdout, MoveTo(0, y as u16),)?;

        for x in 0..width {
            let value = displayed[x];
            let y = height - 1 - y;

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
