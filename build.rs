use std::{
    env, fs,
    io::{Error as IoError, ErrorKind as IoErrorKind, Result as IoResult},
    path::PathBuf,
};

// Descriptions are copied from the --help output of each binary, so they should be kept in sync with those.
const BINS: [(&'static str, &'static str); 6] = [
    ("conway", "Conway's Game of Life simulator and renderer."),
    ("digirain", "Digital rain terminal animation."),
    ("doodle", "Plays random doodles."),
    ("maze", "Generates and solves mazes."),
    ("ripples", ""),
    ("sorty", "Visualizes different sorting algorithms."),
];

fn gen_rc(bin: &str, description: &str, manifest_dir: &PathBuf) -> String {
    let ico_path = manifest_dir.join("assets").join(format!("{bin}.ico"));
    println!("cargo:rerun-if-changed={}", ico_path.display());

    format!(
        r#"
#include <winver.h>
#include <ntverp.h>

VS_VERSION_INFO VERSIONINFO
    FILEVERSION     {comma_version},0
    PRODUCTVERSION  {comma_version},0
    FILEFLAGSMASK   VS_FFI_FILEFLAGSMASK
    FILEFLAGS       0
    FILEOS          VOS_NT
    FILETYPE        VFT_APP
BEGIN
    BLOCK "StringFileInfo"
    BEGIN
        BLOCK "040904B0"
        BEGIN
            VALUE "CompanyName",      "{authors}\0"
            VALUE "FileDescription",  "{description}\0"
            VALUE "FileVersion",      "{dot_version}.0\0"
            VALUE "InternalName",     "{bin}.exe\0"
            VALUE "LegalCopyright",   "Copyright (c) 2025 Jonathon Burnham Cobb. Licensed under the MIT-0 license.\0"
            VALUE "OriginalFilename", "{bin}.exe\0"
            VALUE "ProductName",      "{pkg_name}\0"
            VALUE "ProductVersion",	  "{dot_version}.0\0"
        END
    END

    BLOCK "VarFileInfo"
    BEGIN
        VALUE "Translation", 0x0409, 0x04B0
    END
END

1 ICON "{ico_path}"
"#,
        dot_version = env::var("CARGO_PKG_VERSION").unwrap(),
        comma_version = env::var("CARGO_PKG_VERSION").unwrap().replace('.', ","),
        authors = env::var("CARGO_PKG_AUTHORS").unwrap(),
        description = description,
        pkg_name = env::var("CARGO_PKG_DESCRIPTION").unwrap(),
        bin = bin,
        ico_path = ico_path.display().to_string().replace('\\', "\\\\")
    )
}

fn main() -> IoResult<()> {
    let target = env::var("TARGET").unwrap();

    if env::var("CARGO_CFG_TARGET_OS").unwrap() != "windows" || !target.contains("msvc") {
        return Ok(());
    }

    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());

    let rc = cc::windows_registry::find_tool(&target, "rc.exe")
        .expect("Could not find rc.exe — is the Windows SDK installed?");

    for (bin, description) in BINS {
        let rc_path = out_dir.join(format!("{bin}.rc"));
        fs::write(&rc_path, gen_rc(bin, description, &manifest_dir))?;

        // Compile the .rc file into a .res file using rc.exe.
        let res_path = out_dir.join(format!("{bin}.res"));
        let status = rc.to_command().args(["/nologo", "/fo"]).arg(&res_path).arg(&rc_path).status()?;

        if !status.success() {
            return Err(IoError::new(IoErrorKind::Other, format!("rc.exe failed for {bin}")));
        }

        // Tell Cargo to pass the .res file to the linker for this specific binary.
        println!("cargo:rustc-link-arg-bin={}={}", bin, res_path.display());
    }

    Ok(())
}
