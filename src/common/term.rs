use std::{
    io::{Result as IoResult, stderr, stdout},
    time::Duration,
};

use clap::Parser;
use crossterm::{
    cursor::{Hide, MoveTo, Show},
    event::{self, Event, KeyCode},
    execute,
    style::{Attribute, Attributes, Color, ContentStyle, PrintStyledContent},
    terminal::{
        Clear, ClearType, DisableLineWrap, EnableLineWrap, EnterAlternateScreen,
        LeaveAlternateScreen,
    },
};

#[derive(Parser, Debug)]
pub struct CommonArgs {
    /// Wait for keypress between frames.
    ///
    /// If set, the board will render one frame at a time and wait for the user
    /// to press Enter before proceeding to the next frame. This is incompatible
    /// with the `--wait` option.
    #[arg(short = 'i', long)]
    interactive: bool,

    /// Delay between frames in milliseconds.
    ///
    /// If set, the program will wait for the specified number of milliseconds
    /// between rendering each frame. If set to 0, the program will render
    /// frames as fast as possible. This option is incompatible with the
    /// `--interactive` option.
    #[arg(
        short = 'w',
        long,
        default_value_t = 60,
        conflicts_with = "interactive"
    )]
    wait: u64,
}

pub fn validate_color(s: &str) -> Result<usize, String> {
    match s.parse::<usize>() {
        Ok(n) if n <= 7 => Ok(n),
        _ => Err(String::from("must be an integer between 0 and 7.")),
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum WaitResult {
    Continue,
    Resize(usize, usize),
    Exit,
}

pub const BOLD_STYLES: [ContentStyle; 8] = [
    ContentStyle {
        foreground_color: Some(Color::DarkGrey),
        background_color: None,
        underline_color: None,
        attributes: Attributes::none().with(Attribute::Bold),
    },
    ContentStyle {
        foreground_color: Some(Color::Red),
        background_color: None,
        underline_color: None,
        attributes: Attributes::none().with(Attribute::Bold),
    },
    ContentStyle {
        foreground_color: Some(Color::Green),
        background_color: None,
        underline_color: None,
        attributes: Attributes::none().with(Attribute::Bold),
    },
    ContentStyle {
        foreground_color: Some(Color::Yellow),
        background_color: None,
        underline_color: None,
        attributes: Attributes::none().with(Attribute::Bold),
    },
    ContentStyle {
        foreground_color: Some(Color::Blue),
        background_color: None,
        underline_color: None,
        attributes: Attributes::none().with(Attribute::Bold),
    },
    ContentStyle {
        foreground_color: Some(Color::Magenta),
        background_color: None,
        underline_color: None,
        attributes: Attributes::none().with(Attribute::Bold),
    },
    ContentStyle {
        foreground_color: Some(Color::Cyan),
        background_color: None,
        underline_color: None,
        attributes: Attributes::none().with(Attribute::Bold),
    },
    ContentStyle {
        foreground_color: Some(Color::White),
        background_color: None,
        underline_color: None,
        attributes: Attributes::none().with(Attribute::Bold),
    },
];

pub const STYLES: [ContentStyle; 8] = [
    ContentStyle {
        foreground_color: Some(Color::Grey),
        background_color: None,
        underline_color: None,
        attributes: Attributes::none(),
    },
    ContentStyle {
        foreground_color: Some(Color::Red),
        background_color: None,
        underline_color: None,
        attributes: Attributes::none(),
    },
    ContentStyle {
        foreground_color: Some(Color::Green),
        background_color: None,
        underline_color: None,
        attributes: Attributes::none(),
    },
    ContentStyle {
        foreground_color: Some(Color::Yellow),
        background_color: None,
        underline_color: None,
        attributes: Attributes::none(),
    },
    ContentStyle {
        foreground_color: Some(Color::Blue),
        background_color: None,
        underline_color: None,
        attributes: Attributes::none(),
    },
    ContentStyle {
        foreground_color: Some(Color::Magenta),
        background_color: None,
        underline_color: None,
        attributes: Attributes::none(),
    },
    ContentStyle {
        foreground_color: Some(Color::Cyan),
        background_color: None,
        underline_color: None,
        attributes: Attributes::none(),
    },
    ContentStyle {
        foreground_color: Some(Color::White),
        background_color: None,
        underline_color: None,
        attributes: Attributes::none(),
    },
];

pub const DIM_STYLES: [ContentStyle; 8] = [
    ContentStyle {
        foreground_color: Some(Color::Black),
        background_color: None,
        underline_color: None,
        attributes: Attributes::none().with(Attribute::Dim),
    },
    ContentStyle {
        foreground_color: Some(Color::DarkRed),
        background_color: None,
        underline_color: None,
        attributes: Attributes::none().with(Attribute::Dim),
    },
    ContentStyle {
        foreground_color: Some(Color::DarkGreen),
        background_color: None,
        underline_color: None,
        attributes: Attributes::none().with(Attribute::Dim),
    },
    ContentStyle {
        foreground_color: Some(Color::DarkYellow),
        background_color: None,
        underline_color: None,
        attributes: Attributes::none().with(Attribute::Dim),
    },
    ContentStyle {
        foreground_color: Some(Color::DarkBlue),
        background_color: None,
        underline_color: None,
        attributes: Attributes::none().with(Attribute::Dim),
    },
    ContentStyle {
        foreground_color: Some(Color::DarkMagenta),
        background_color: None,
        underline_color: None,
        attributes: Attributes::none().with(Attribute::Dim),
    },
    ContentStyle {
        foreground_color: Some(Color::DarkCyan),
        background_color: None,
        underline_color: None,
        attributes: Attributes::none().with(Attribute::Dim),
    },
    ContentStyle {
        foreground_color: Some(Color::Grey),
        background_color: None,
        underline_color: None,
        attributes: Attributes::none().with(Attribute::Dim),
    },
];

pub const ERROR_STYLE: ContentStyle = ContentStyle {
    foreground_color: Some(Color::Red),
    background_color: None,
    underline_color: None,
    attributes: Attributes::none().with(Attribute::Bold),
};

/// Prints a formatted error message to the standard error output.
pub fn error(msg: &str) {
    _ = execute!(
        stderr(),
        PrintStyledContent(ERROR_STYLE.apply("Error: ")),
        PrintStyledContent(ContentStyle::default().apply(msg)),
    );
}

pub fn setup_term() -> IoResult<()> {
    execute!(
        stdout(),
        EnterAlternateScreen,
        Hide,
        DisableLineWrap,
        Clear(ClearType::All)
    )
}

pub fn cleanup_term() -> IoResult<()> {
    execute!(
        stdout(),
        Clear(ClearType::All),
        MoveTo(0, 0),
        Show,
        EnableLineWrap,
        LeaveAlternateScreen
    )
}

/// Prints a formatted error message to the standard error output.
#[macro_export]
macro_rules! error {
    ($($arg:tt)*) => {
        doodles::common::term::error(&format!($($arg)*));
    };
}

/// Prints a formatted warning message to the standard error output and exits
/// the program with a non-zero exit code.
#[macro_export]
macro_rules! abort {
    ($($arg:tt)*) => {
        doodles::common::term::error(&format!($($arg)*));
        std::process::exit(1);
    };
}

impl CommonArgs {
    /// Wait for either a delay to elapse or a keypress event, depending on the
    /// arguments.
    ///
    /// Returns
    /// =======
    ///
    /// - `WaitResult::Continue(Some(Event))` if an event was detected.
    /// - `WaitResult::Continue(None)` if the wait time elapsed without events
    ///   and [`CommonArgs::interactive`] is false.
    /// - `WaitResult::Exit` if the user requested to exit (Esc or 'q' key).
    pub fn wait(&self) -> IoResult<WaitResult> {
        loop {
            let result = if self.interactive {
                self.handle_event()?
            } else if let Ok(true) = event::poll(Duration::from_millis(self.wait)) {
                self.handle_event()?
            } else {
                None
            };

            match result {
                Some(result) => return Ok(result),
                None => {
                    if self.interactive {
                        continue;
                    } else {
                        break Ok(WaitResult::Continue);
                    }
                }
            }
        }
    }

    fn handle_event(&self) -> IoResult<Option<WaitResult>> {
        if let Ok(event) = event::read() {
            match event {
                Event::Key(ev)
                    if ev.is_press()
                        && (!self.interactive
                            || ev.code == KeyCode::Esc
                            || ev.code == KeyCode::Char('q')) =>
                {
                    Ok(Some(WaitResult::Exit))
                }
                Event::Key(ev) if ev.is_press() => Ok(Some(WaitResult::Continue)),
                Event::Resize(width, height) => {
                    Ok(Some(WaitResult::Resize(width as usize, height as usize)))
                }
                _ => Ok(None),
            }
        } else {
            Ok(None)
        }
    }
}
