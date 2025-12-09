// Copyright (c) 2025 Jonathon Burnham Cobb
// Licensed under the MIT-0 license.

use std::{env, io::Result as IoResult, process::Command};

use clap::Parser;
use rand::seq::IndexedRandom;

struct Doodle {
    name: &'static str,
    args: &'static [&'static str],
}

const DOODLES: [Doodle; 4] = [
    Doodle {
        name: "sorty",
        args: &["--wait=0", "--iter=1"],
    },
    Doodle {
        name: "conway",
        args: &["--wait=48", "--iter=1"],
    },
    Doodle {
        name: "digirain",
        args: &["--wait=64", "--iter=384"],
    },
    Doodle {
        name: "maze",
        args: &["--wait=32", "--iter=1"],
    },
];

/// Plays random doodles.
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about=None)]
struct Args {
    /// Number of iterations to run.
    ///
    /// If absent, the program will run indefinitely until interrupted.
    #[arg(short = 'n', long)]
    pub iter: Option<usize>,
}

fn main() -> IoResult<()> {
    let args = Args::parse();
    let path = env::current_exe()?;
    let path = path.parent().unwrap();

    let mut iteration = 0;
    let mut rand = rand::rng();

    loop {
        if let Some(max_iterations) = args.iter
            && iteration >= max_iterations
        {
            break;
        }
        iteration += 1;

        let doodle = DOODLES.choose(&mut rand).unwrap();
        let doodle_path = path.join(doodle.name);

        let mut cmd = Command::new(doodle_path);
        cmd.args(doodle.args);

        let mut proc = cmd.spawn()?;
        if !proc.wait()?.success() {
            break;
        }
    }

    Ok(())
}
