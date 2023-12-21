use anyhow::{bail, Context, Result};
use std::fs;

use tempfile::NamedTempFile;

/// Opens the provided text in the user's preferred editor
pub fn edit_text(input: &[u8], filename_prefix: Option<&str>) -> Result<Vec<u8>> {
    let file = match filename_prefix {
        Some(prefix) => NamedTempFile::with_prefix(format!("{prefix}-")),
        None => NamedTempFile::new(),
    }?;

    fs::write(file.path(), input).expect("failed to write file");
    let editor = get_editor()?;

    std::process::Command::new(editor)
        .arg(file.path())
        .status()
        .context("Failed to edit file")?;

    let new_contents = fs::read(file.path())?;
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
