use std::io::{Result as IoResult, stdout};

use clap::Parser;
use doodles::common::CommonArgs;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about=None)]
pub struct Args {
    #[clap(flatten)]
    common: CommonArgs,
}

fn main() -> IoResult<()> {
    let args = Args::parse();

    let mut stdout = stdout();

    Ok(())
}
