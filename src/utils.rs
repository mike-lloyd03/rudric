use anyhow::Result;
use dialoguer::{theme::ColorfulTheme, Password};

pub fn set_password(prompt: &str) -> Result<String> {
    Ok(Password::with_theme(&ColorfulTheme::default())
        .with_prompt(prompt)
        .with_confirmation("Confirm password", "Passwords do not match")
        .interact()?)
}
