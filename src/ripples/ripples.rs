// Copyright (c) 2025 Jonathon Burnham Cobb
// Licensed under the MIT-0 license.

use std::{
    f64::consts::PI,
    hash::{BuildHasher, Hash, Hasher, RandomState},
    io::{Result as IoResult, Write, stdout},
    str::FromStr,
};

use clap::{ValueEnum, builder::PossibleValue};
use crossterm::{
    cursor::MoveTo,
    queue,
    style::{Attributes, ContentStyle, PrintStyledContent},
};
use doodles::common::{color::Color, math::Lerp, vec::UVec2};
use rand::{
    Rng,
    distr::{Bernoulli, Distribution},
};

#[derive(Clone, Debug)]
pub struct Medium {
    size: UVec2,
    sources: Vec<Source>,
    waves: Vec<Wave>,
    period: usize,
    base_color: Color,
    peak_color: Color,
    trough_color: Color,
    buffer: Vec<f32>,
    age: usize,
    initial_amplitude: f64,
}

#[derive(Clone, Debug)]
pub struct Source {
    focus: UVec2,
    age: usize,
    lifetime: usize,
}

#[derive(Clone, Debug)]
pub struct Wave {
    focus: UVec2,
    amplitude: f64,
    age: usize,
}

#[derive(Clone, Copy, Debug)]
#[repr(u8)]
pub enum RenderStyle {
    Block = 0,
    Splash = 1,
    Dots = 2,
}

const BLOCK_CHARS: [char; 5] = [' ', '░', '▒', '▓', '█'];

const SPLASH_CHARS: [[char; 8]; 8] = [
    [' ', '.', ':', '-', '=', '+', '%', '@'],
    [' ', ',', ';', '/', '(', '*', '$', '#'],
    [' ', '\'', '^', '\\', ')', 'X', '?', '8'],
    [' ', '"', '!', '~', '≈', '∞', '¿', 'Φ'],
    [' ', '`', '∙', '⌐', '[', 'δ', 'S', '0'],
    [' ', '·', '¡', '¬', ']', '≡', 'ß', '&'],
    [' ', '°', '÷', 'ℓ', '{', '±', 'ñ', 'B'],
    [' ', 'ⁿ', 'σ', 'ƒ', '}', 'æ', 'ü', 'Θ'],
];

const DOT_CHARS: [&'static [char]; 9] = [
    &[' '],
    &['⠁', '⠐', '⠂', '⠄', '⠈', '⠠', '⡀', '⢀'],
    &[
        '⠃', '⠅', '⠆', '⠉', '⠊', '⠌', '⠑', '⠒', '⠔', '⠘', '⠡', '⠢', '⠨', '⠰', '⡁', '⡂', '⡄', '⡈', '⡐', '⡠', '⢁', '⢂',
        '⢄', '⢈', '⢐', '⣀',
    ],
    &[
        '⠇', '⠋', '⠍', '⠎', '⠓', '⠕', '⠖', '⠙', '⠚', '⠜', '⠣', '⠥', '⠦', '⠩', '⠪', '⠬', '⠱', '⠲', '⠴', '⠸', '⡃', '⡅',
        '⡆', '⡉', '⡊', '⡌', '⡑', '⡒', '⡔', '⡘', '⡡', '⡢', '⡤', '⡨', '⡰', '⢃', '⢅', '⢆', '⢉', '⢊', '⢌', '⢑', '⢒', '⢔',
        '⢘', '⢡', '⢢', '⢤', '⢨', '⢰', '⣁', '⣂', '⣄', '⣈', '⣐', '⣠',
    ],
    &[
        '⠏', '⠛', '⠞', '⠧', '⠫', '⠭', '⠮', '⠳', '⠵', '⠶', '⠺', '⠼', '⡇', '⡋', '⡍', '⡎', '⡓', '⡕', '⡖', '⡙', '⡚', '⡜',
        '⡣', '⡥', '⡦', '⡩', '⡪', '⡬', '⡱', '⡲', '⡴', '⡸', '⢇', '⢋', '⢍', '⢎', '⢓', '⢕', '⢖', '⢙', '⢚', '⢜', '⢣', '⢥',
        '⢦', '⢩', '⢪', '⢬', '⢱', '⢲', '⢴', '⢸', '⣃', '⣅', '⣆', '⣉', '⣊', '⣌', '⣑', '⣒', '⣔', '⣘', '⣡', '⣢', '⣤', '⣨',
        '⣰',
    ],
    &[
        '⠟', '⠯', '⠷', '⠻', '⠽', '⠾', '⡏', '⡗', '⡛', '⡝', '⡞', '⡧', '⡫', '⡭', '⡮', '⡳', '⡵', '⡶', '⡹', '⡺', '⡼', '⢏',
        '⢗', '⢛', '⢝', '⢞', '⢧', '⢭', '⢮', '⢵', '⢹', '⢺', '⢼', '⣇', '⣋', '⣍', '⣎', '⣓', '⣕', '⣖', '⣙', '⣚', '⣜', '⣣',
        '⣥', '⣦', '⣩', '⣬', '⣴', '⣸',
    ],
    &[
        '⠿', '⡟', '⡯', '⡷', '⡻', '⡾', '⢟', '⢯', '⢷', '⢻', '⢽', '⢾', '⣕', '⣗', '⣛', '⣝', '⣞', '⣧', '⣫', '⣭', '⣮', '⣳',
        '⣵', '⣶', '⣹', '⣺',
    ],
    &['⡿', '⢿', '⣟', '⣯', '⣻', '⣽', '⣾'],
    &['⣿'],
];

impl Medium {
    pub fn new(
        size: UVec2,
        period: usize,
        base_color: Color,
        peak_color: Color,
        trough_color: Color,
        initial_amplitude: f64,
    ) -> Self {
        Medium {
            size,
            sources: vec![],
            waves: vec![],
            period,
            base_color,
            peak_color,
            trough_color,
            buffer: vec![0.0; size.prod()],
            age: 0,
            initial_amplitude,
        }
    }

    pub fn next(&mut self, spawnprob: f64, rand: &mut impl Rng) {
        let spawn = Bernoulli::new(spawnprob).unwrap();
        if spawn.sample(rand) {
            let focus = UVec2 {
                x: rand.random_range(0..self.size.x),
                y: rand.random_range(0..self.size.y),
            };
            let lifetime = rand.random_range(10..50);
            self.sources.push(Source::new(focus, lifetime));
        }

        for wave in &mut self.waves {
            wave.age += 1;
        }

        let mut sources = vec![];

        for source in self.sources.drain(..) {
            let (next_source, wave) = source.next(self.period, self.initial_amplitude);

            if let Some(next_source) = next_source {
                sources.push(next_source);
            }

            if let Some(wave) = wave {
                self.waves.push(wave);
            }
        }

        self.sources = sources;

        for y in 0..self.size.y {
            for x in 0..self.size.x {
                let mut value = 0.0;
                let pos = UVec2 { x, y };

                for wave in &self.waves {
                    let distance = pos.euclidean_dist(wave.focus);
                    let radius = wave.radius(self.period);

                    let phase = PI * f64::inverse_lerp(radius - self.period as f64, radius, distance).max(0.0);
                    if phase > 0.0 && phase < 2.0 * PI {
                        value -= wave.amplitude * phase.sin();
                    }
                }

                self.buffer[(y * self.size.x + x) as usize] = value as f32;
            }
        }

        self.waves
            .retain(|wave| wave.radius(self.period) < (self.size.x + self.size.y) as f64);

        self.age += 1;
    }

    pub fn render(&self, render_style: RenderStyle, random_state: &RandomState) -> IoResult<()> {
        let mut stdout = stdout();

        for y in 0..self.size.y {
            queue!(stdout, MoveTo(0, y as u16),)?;
            for x in 0..self.size.x {
                let value = self.buffer[(y * self.size.x + x) as usize].clamp(-1.0, 1.0);
                let color = if value > 0.0 {
                    self.peak_color.to_term_color()
                } else {
                    self.trough_color.to_dark_term_color()
                };

                let ch = match render_style {
                    RenderStyle::Block => {
                        let ch_idx = (value.abs() * (BLOCK_CHARS.len() - 1) as f32).round() as usize;
                        BLOCK_CHARS[ch_idx]
                    }
                    RenderStyle::Splash => {
                        let sub_idx = {
                            let mut hasher = random_state.build_hasher();
                            self.age.hash(&mut hasher);
                            x.hash(&mut hasher);
                            y.hash(&mut hasher);
                            hasher.finish() as usize
                        } % SPLASH_CHARS.len();
                        let ch_idx = (value.abs() * (SPLASH_CHARS[0].len() - 1) as f32).round() as usize;
                        SPLASH_CHARS[sub_idx][ch_idx]
                    }
                    RenderStyle::Dots => {
                        let sub_idx = (value.abs() * (DOT_CHARS.len() - 1) as f32).round() as usize;
                        let ch_idx = {
                            let mut hasher = random_state.build_hasher();
                            self.age.hash(&mut hasher);
                            x.hash(&mut hasher);
                            y.hash(&mut hasher);
                            hasher.finish() as usize
                        } % DOT_CHARS[sub_idx].len();
                        DOT_CHARS[sub_idx][ch_idx]
                    }
                };

                let style = ContentStyle {
                    foreground_color: Some(color),
                    background_color: Some(self.base_color.to_dark_term_color()),
                    underline_color: None,
                    attributes: Attributes::none(),
                };

                queue!(stdout, PrintStyledContent(style.apply(ch)))?;
            }
        }

        stdout.flush()
    }

    pub fn age(&self) -> usize {
        self.age
    }

    pub fn converged(&self) -> bool {
        self.waves.is_empty() && self.sources.is_empty()
    }
}

impl Source {
    pub fn new(focus: UVec2, lifetime: usize) -> Self {
        Source {
            focus,
            age: 0,
            lifetime,
        }
    }

    pub fn next(self, period: usize, initial_amplitude: f64) -> (Option<Self>, Option<Wave>) {
        if self.age >= self.lifetime {
            (None, None)
        } else if self.age % period == 0 {
            let amplitude = 1.0 - self.age as f64 / self.lifetime as f64;

            (
                Some(Source {
                    age: self.age + 1,
                    ..self
                }),
                Some(Wave::new(self.focus, amplitude * initial_amplitude)),
            )
        } else {
            (
                Some(Source {
                    age: self.age + 1,
                    ..self
                }),
                None,
            )
        }
    }
}

impl Wave {
    pub fn new(focus: UVec2, amplitude: f64) -> Self {
        Wave {
            focus,
            amplitude,
            age: 0,
        }
    }

    pub fn radius(&self, period: usize) -> f64 {
        self.age as f64 * period as f64 / (2.0 * PI)
    }
}

impl RenderStyle {
    pub fn choose(rand: &mut impl Rng) -> Self {
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
                "S" | "SPLASH" => Ok(RenderStyle::Splash),
                "B" | "BLOCK" => Ok(RenderStyle::Block),
                "D" | "DOTS" => Ok(RenderStyle::Dots),
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
        match value % 3 {
            0 => RenderStyle::Splash,
            1 => RenderStyle::Block,
            2 => RenderStyle::Dots,
            _ => unreachable!(),
        }
    }
}

impl ValueEnum for RenderStyle {
    fn value_variants<'a>() -> &'a [Self] {
        &[RenderStyle::Splash, RenderStyle::Block, RenderStyle::Dots]
    }

    fn to_possible_value(&self) -> Option<PossibleValue> {
        match self {
            RenderStyle::Splash => Some(PossibleValue::new("splash").alias("s").alias("0")),
            RenderStyle::Block => Some(PossibleValue::new("block").alias("b").alias("1")),
            RenderStyle::Dots => Some(PossibleValue::new("dots").alias("d").alias("2")),
        }
    }
}
