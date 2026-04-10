// Copyright (c) 2025 Jonathon Burnham Cobb
// Licensed under the MIT-0 license.

use std::{
    fmt::{Display, Formatter, Result as FmtResult},
    str::FromStr,
};

use clap::ValueEnum;
use crossterm::style::{Attribute, Attributes, Color as TermColor, ContentStyle};
use rand::{Rng, RngExt};

/// A primary terminal color.
///
/// Can be parsed as a command line argument and is convertible to a [`crossterm::style::Color`].
#[derive(Clone, Copy, PartialEq, Eq, Debug, ValueEnum)]
#[repr(u8)]
#[clap(rename_all = "kebab-case")]
pub enum Color {
    #[clap(alias = "k", alias = "0")]
    Black = 0,

    #[clap(alias = "r", alias = "1")]
    Red = 1,

    #[clap(alias = "g", alias = "2")]
    Green = 2,

    #[clap(alias = "y", alias = "3")]
    Yellow = 3,

    #[clap(alias = "b", alias = "4")]
    Blue = 4,

    #[clap(alias = "m", alias = "5")]
    Magenta = 5,

    #[clap(alias = "c", alias = "6")]
    Cyan = 6,

    #[clap(alias = "w", alias = "7")]
    White = 7,
}

impl Color {
    /// Choose a random color excluding black.
    pub fn choose(rand: &mut impl Rng) -> Self {
        let value = rand.random_range(1..8);
        Color::from(value)
    }

    /// Choose a random color excluding black or white.
    pub fn choose_non_mono(rand: &mut impl Rng) -> Self {
        let value = rand.random_range(1..7);
        Color::from(value)
    }

    /// Returns a complementary color.
    pub fn complement(&self) -> Self {
        match self {
            Color::Red => Color::Cyan,
            Color::Green => Color::Magenta,
            Color::Yellow => Color::Blue,
            Color::Blue => Color::Yellow,
            Color::Magenta => Color::Green,
            Color::Cyan => Color::Red,
            Color::White | Color::Black => *self,
        }
    }

    /// Returns the corresponding [`ContentStyle`] from [`STYLES`].
    pub fn style(&self) -> ContentStyle {
        ContentStyle {
            foreground_color: Some(self.to_term_color()),
            background_color: None,
            underline_color: None,
            attributes: Attributes::none(),
        }
    }

    /// Returns the corresponding bold [`ContentStyle`] from [`BOLD_STYLES`].
    pub fn bold_style(&self) -> ContentStyle {
        ContentStyle {
            foreground_color: Some(self.to_term_color()),
            background_color: None,
            underline_color: None,
            attributes: Attributes::none().with(Attribute::Bold),
        }
    }

    /// Returns the corresponding dim [`ContentStyle`] from [`MEDIUM_STYLES`].
    pub fn medium_style(&self) -> ContentStyle {
        ContentStyle {
            foreground_color: Some(self.to_dark_term_color()),
            background_color: None,
            underline_color: None,
            attributes: Attributes::none(),
        }
    }

    /// Returns the corresponding dim [`ContentStyle`] from [`DIM_STYLES`].
    pub fn dim_style(&self) -> ContentStyle {
        ContentStyle {
            foreground_color: Some(self.to_dark_term_color()),
            background_color: None,
            underline_color: None,
            attributes: Attributes::none().with(Attribute::Dim),
        }
    }

    /// Returns the corresponding dark [`crossterm::style::Color`].
    pub fn to_dark_term_color(&self) -> TermColor {
        match self {
            Color::Black => TermColor::Black,
            Color::Red => TermColor::DarkRed,
            Color::Green => TermColor::DarkGreen,
            Color::Yellow => TermColor::DarkYellow,
            Color::Blue => TermColor::DarkBlue,
            Color::Magenta => TermColor::DarkMagenta,
            Color::Cyan => TermColor::DarkCyan,
            Color::White => TermColor::Grey,
        }
    }

    /// Returns the corresponding bright [`crossterm::style::Color`].
    pub fn to_term_color(&self) -> TermColor {
        match self {
            Color::Black => TermColor::DarkGrey,
            Color::Red => TermColor::Red,
            Color::Green => TermColor::Green,
            Color::Yellow => TermColor::Yellow,
            Color::Blue => TermColor::Blue,
            Color::Magenta => TermColor::Magenta,
            Color::Cyan => TermColor::Cyan,
            Color::White => TermColor::White,
        }
    }
}

impl FromStr for Color {
    type Err = ();

    /// Parses a color from either its name/abbreviation or its numeric value (0-7).
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Ok(value) = s.parse::<u8>() {
            Ok(Color::from(value))
        } else {
            let s = s.to_uppercase();
            match s.as_str() {
                "K" | "BLACK" => Ok(Color::Black),
                "R" | "RED" => Ok(Color::Red),
                "G" | "GREEN" => Ok(Color::Green),
                "Y" | "YELLOW" => Ok(Color::Yellow),
                "B" | "BLUE" => Ok(Color::Blue),
                "M" | "MAGENTA" => Ok(Color::Magenta),
                "C" | "CYAN" => Ok(Color::Cyan),
                "W" | "WHITE" => Ok(Color::White),
                _ => Err(()),
            }
        }
    }
}

impl Into<u8> for Color {
    fn into(self) -> u8 {
        self as u8
    }
}

impl From<u8> for Color {
    fn from(value: u8) -> Self {
        match value & 7 {
            0 => Color::Black,
            1 => Color::Red,
            2 => Color::Green,
            3 => Color::Yellow,
            4 => Color::Blue,
            5 => Color::Magenta,
            6 => Color::Cyan,
            7 => Color::White,
            _ => unreachable!(),
        }
    }
}

impl Default for Color {
    fn default() -> Self {
        Color::White
    }
}

impl Display for Color {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "{self:?}")
    }
}
