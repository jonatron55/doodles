use std::{env, io::Result as IoResult};

use winresource::WindowsResource;

fn main() -> IoResult<()> {
    if env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        let mut res = WindowsResource::new();
        res.set_icon("../assets/doodle.ico");
        res.compile()?;
    }
    Ok(())
}
