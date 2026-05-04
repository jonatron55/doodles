// Copyright (c) 2025 Jonathon Burnham Cobb
// Licensed under the MIT-0 license.

use std::{
    backtrace::{Backtrace, BacktraceStatus},
    io::{Error as IoError, ErrorKind as IoErrorKind, Result as IoResult, stderr, stdout},
    panic,
    process::exit,
    thread,
    time::Duration,
};

use clap::Parser;
use crossterm::{
    cursor::{Hide, MoveTo, Show},
    event::{self, Event, KeyCode},
    execute,
    style::{Attribute, Attributes, Color, ContentStyle, PrintStyledContent},
    terminal::{Clear, ClearType, DisableLineWrap, EnableLineWrap, EnterAlternateScreen, LeaveAlternateScreen},
};

use crate::vec::{UVec2, uvec2};

#[derive(Parser, Debug)]
pub struct CommonArgs {
    /// Number of iterations to run.
    ///
    /// If absent, the program will run indefinitely until interrupted.
    #[arg(short = 'n', long = "iter", alias = "iterations")]
    pub max_iterations: Option<usize>,

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
    #[arg(short = 'w', long, default_value_t = 32, conflicts_with = "interactive")]
    wait: u64,
}

/// Result of a wait operation.
#[derive(Debug, PartialEq, Eq)]
pub enum WaitResult {
    /// The wait completed and the animation should continue.
    Continue,

    /// A key was pressed and the current animation should end and the next should begin.
    Next,

    /// The screen was resized and the animation should adjust accordingly.
    Resize(UVec2),

    /// The user requested to exit the program. The animation should halt.
    Exit,
}

/// Style used for error messages.
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

/// Set up the terminal for alternate screen rendering.
///
/// This will register a Ctrl+C handler that will clean up the terminal state
/// and exit the program when triggered.
///
/// On a graceful exit, the caller should invoke [`cleanup_term`] to restore the terminal state.
pub fn setup_term() -> IoResult<()> {
    execute!(
        stdout(),
        EnterAlternateScreen,
        Hide,
        DisableLineWrap,
        Clear(ClearType::All)
    )?;

    ctrlc::set_handler(move || {
        cleanup_term().unwrap();
        exit(1);
    })
    .map_err(|err| IoError::new(IoErrorKind::Other, format!("Failed to set Ctrl+C handler: {}", err)))?;

    panic::set_hook(Box::new(|panic_info| {
        let _ = cleanup_term();

        if let Some(name) = thread::current().name() {
            eprint!("Thread '{name}' ");
        } else {
            let id = thread::current().id();
            eprint!("Thread {id:?} ");
        }

        eprintln!("{panic_info}");

        let backtrace = Backtrace::capture();
        if backtrace.status() == BacktraceStatus::Captured {
            eprintln!("{backtrace}");
        }
    }));

    Ok(())
}

/// Restore the terminal to its original state after alternate screen rendering.
///
/// This undoes the effects of [`setup_term`].
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
        doodle::term::error(&format!($($arg)*));
    };
}

/// Prints a formatted warning message to the standard error output and exits
/// the program with a non-zero exit code.
#[macro_export]
macro_rules! abort {
    ($($arg:tt)*) => {
        doodle::term::error(&format!($($arg)*));
        std::process::exit(1);
    };
}

impl CommonArgs {
    /// Wait for either a delay to elapse or a keypress event, depending on the arguments.
    pub fn wait(&self) -> IoResult<WaitResult> {
        loop {
            let result = if self.interactive {
                Some(self.handle_event()?)
            } else if let Ok(true) = event::poll(Duration::from_millis(self.wait)) {
                Some(self.handle_event()?)
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

    fn handle_event(&self) -> IoResult<WaitResult> {
        if let Ok(event) = event::read() {
            match event {
                Event::Key(ev) if ev.is_press() && !ev.is_repeat() => match ev.code {
                    KeyCode::Esc | KeyCode::Char('q') => Ok(WaitResult::Exit),
                    KeyCode::Right | KeyCode::Tab => Ok(WaitResult::Next),
                    _ => Ok(WaitResult::Continue),
                },
                Event::Resize(width, height) => Ok(WaitResult::Resize(uvec2(width as usize, height as usize))),
                _ => Ok(WaitResult::Continue),
            }
        } else {
            Ok(WaitResult::Continue)
        }
    }
}
