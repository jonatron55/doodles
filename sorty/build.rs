use std::{env, io::Result as IoResult};

use winresource::WindowsResource;

fn main() -> IoResult<()> {
    if env::var("CARGO_CFG_TARGET_OS") == Ok("windows".into()) {
        let mut res = WindowsResource::new();
        res.set_icon("../assets/sorty.ico");
        res.compile()?;
    }
    Ok(())
}
