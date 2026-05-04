// Copyright (c) 2025 Jonathon Burnham Cobb
// Licensed under the MIT-0 license.

use std::io::{Result as IoResult, stdout};

use crossterm::{queue, style::PrintStyledContent};
use doodle::{color::Color, vec::UVec2};

const TRINKET_CHARS: [char; 16] = [
    '♦', '♣', '♥', '♠', '☼', '✶', 'Ω', '∞', '♪', '♬', '$', '¢', '£', '¥', '☙', '❧',
];
const COLLECT_CHARS: [char; 16] = [
    '●', '○', '◌', '¤', '*', '¤', '%', '*', '×', '+', '×', '+', '∙', '◦', '·', ' ',
];
const TRINKET_COLORS: [Color; 6] = [
    Color::Red,
    Color::Green,
    Color::Blue,
    Color::Yellow,
    Color::Cyan,
    Color::Magenta,
];

#[derive(Clone, Debug)]
pub struct Trinket {
    pub position: UVec2,
    character: char,
    color: Color,
    state: TrinketState,
}

#[derive(Clone, Debug)]
enum TrinketState {
    Idle { frame: u8 },
    Collected { frame: u8 },
}

impl Trinket {
    pub fn new_collection(positions: &[UVec2]) -> Vec<Trinket> {
        let count = positions.len().min(TRINKET_CHARS.len() * TRINKET_COLORS.len());
        let mut trinkets = Vec::with_capacity(count);

        for i in 0..count {
            let char_idx = i % TRINKET_CHARS.len();
            let color_idx = (i / TRINKET_CHARS.len()) % TRINKET_COLORS.len();
            trinkets.push(Trinket {
                position: positions[i],
                character: TRINKET_CHARS[char_idx],
                color: TRINKET_COLORS[color_idx],
                state: TrinketState::Idle { frame: i as u8 % 32 },
            });
        }

        trinkets
    }

    pub fn update(&mut self) {
        match &mut self.state {
            TrinketState::Idle { frame } => {
                *frame = (*frame + 1) % 32;
            }
            TrinketState::Collected { frame } => {
                *frame = (*frame + 1).min(COLLECT_CHARS.len() as u8 - 1);
            }
        }
    }

    pub fn collect(&mut self) {
        self.state = TrinketState::Collected { frame: 0 };
    }

    pub fn is_collected(&self) -> bool {
        matches!(self.state, TrinketState::Collected { frame: _ })
    }

    pub fn render_position(&self) -> UVec2 {
        self.position * 2 + UVec2::ONE
    }

    pub fn render(&self) -> IoResult<()> {
        queue!(
            stdout(),
            match &self.state {
                TrinketState::Idle { frame } => {
                    let style = match frame / 8 {
                        0 => self.color.bold_style(),
                        1 => self.color.style(),
                        2 => self.color.medium_style(),
                        _ => self.color.style(),
                    };

                    PrintStyledContent(style.apply(self.character))
                }
                TrinketState::Collected { frame } => {
                    let idx = (*frame as usize).min(COLLECT_CHARS.len() - 1);
                    PrintStyledContent(self.color.medium_style().apply(COLLECT_CHARS[idx]))
                }
            }
        )
    }
}
