use anyhow::{bail, Context, Result};
use std::fs;

use mktemp::Temp;

pub fn edit_text(input: &[u8]) -> Result<Vec<u8>> {
    let file = Temp::new_file()?;

    fs::write(file.as_path(), input).expect("failed to write file");
    let editor = get_editor()?;

    std::process::Command::new(editor)
        .arg(file.as_path())
        .status()
        .context("Failed to edit file")?;

    let new_contents = fs::read(file.as_path())?;
    Ok(new_contents)
}

/// Gets the user's prefered text editor from `VISUAL` or `EDITOR` variables. Returns an error
/// if neither variables are defined.
pub fn get_editor() -> Result<String> {
    if let Ok(r) = std::env::var("VISUAL") {
        return Ok(r);
    }
    if let Ok(r) = std::env::var("EDITOR") {
        return Ok(r);
    }
    bail!("Could not determine preferred editor. Define VISUAL or EDITOR in the environment and try again.")
}
