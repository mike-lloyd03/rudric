use anyhow::{Error, Result};
use dialoguer::{theme::ColorfulTheme, Confirm, Input, Password};

/// Prompts the user to confirm an action
pub fn confirm(prompt: &str, default: bool) -> Result<bool> {
    Confirm::with_theme(&ColorfulTheme::default())
        .with_prompt(prompt)
        .wait_for_newline(true)
        .default(default)
        .interact()
        .map_err(Error::msg)
}

/// Prompts the user to set a master password
pub fn set_password(prompt: &str) -> Result<String> {
    Password::with_theme(&ColorfulTheme::default())
        .with_prompt(prompt)
        .with_confirmation("Confirm password", "Passwords do not match")
        .interact()
        .map_err(Error::msg)
}

/// Reads the user's password
pub fn read_password() -> Result<String> {
    Password::with_theme(&ColorfulTheme::default())
        .with_prompt("Enter master password")
        .report(false)
        .interact()
        .map_err(Error::msg)
}

/// Prompts the user for input
pub fn input(prompt: &str) -> Result<String> {
    Input::with_theme(&ColorfulTheme::default())
        .with_prompt(prompt)
        .interact()
        .map_err(Error::msg)
}
