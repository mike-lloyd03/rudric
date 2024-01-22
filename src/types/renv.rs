use std::{fs, path::Path};

use anyhow::{anyhow, bail, Result};
use regex::Regex;

use crate::types::secret::Secret;

use super::{app::App, secret::SECRET_NOT_FOUND, shell_type::ShellType};

#[derive(Debug)]
pub struct Renv {
    pub variables: Vec<Variable>,
}

#[derive(Debug)]
pub struct Variable {
    pub name: String,
    pub value: String,
}

impl Variable {
    pub fn from_string(s: &str) -> Result<Self> {
        let re = Regex::new(r"^(?P<var_name>[\w]+)=(?P<var_value>[^=]*)$")?;

        if let Some(captures) = re.captures(s) {
            if let Some(var_name) = captures.name("var_name") {
                if let Some(var_value) = captures.name("var_value") {
                    return Ok(Variable {
                        name: var_name.as_str().to_string(),
                        value: var_value.as_str().to_string(),
                    });
                }
                bail!("Failed to get variable value");
            } else {
                bail!("Failed to get variable name");
            }
        }
        bail!("Failed to parse variable from string")
    }
}

async fn replace_template_vars(app: &App, s: &str) -> Result<String> {
    let mut new_s = s.to_string();
    let re = Regex::new(r"\{\{([^}]+)}}")?;

    for capture in re.captures_iter(s) {
        let match_str = capture.get(0).unwrap();

        let secret_name = capture
            .get(1)
            .ok_or(anyhow!("Failed to get regex match"))?
            .as_str();

        let secret = match Secret::get(&app.db, secret_name).await {
            Ok(s) => s,
            Err(e) => {
                if e.to_string().contains(SECRET_NOT_FOUND) {
                    bail!("Secret '{secret_name}' not found")
                } else {
                    bail!(e)
                }
            }
        };
        let clear_text = secret.to_cleartext(&app.master_key)?;
        new_s = new_s.replace(match_str.as_str(), clear_text.value.trim());
    }

    Ok(new_s.to_string())
}

impl Renv {
    // Loads the given `path` and parses it's contents for variable names and secret names. Secret
    // names will be replaced with their secret values.
    pub async fn load(app: &App, path: &Path) -> Result<Self> {
        let contents = fs::read_to_string(path)?;
        let lines: Vec<String> = contents.lines().map(|l| l.trim().to_string()).collect();

        let mut variables = vec![];

        for (i, line) in lines.iter().enumerate() {
            // Skip commented and empty lines
            if line.trim_start().starts_with('#') || line.trim().is_empty() {
                continue;
            }

            let var = match Variable::from_string(line) {
                Ok(mut v) => {
                    v.value = match replace_template_vars(app, &v.value).await {
                        Ok(s) => s,
                        Err(e) => {
                            eprintln!("{e}");
                            continue;
                        }
                    };
                    v
                }
                Err(e) => {
                    eprintln!("Error parsing line {}: {}", i + 1, e);
                    continue;
                }
            };
            variables.push(var);
        }

        Ok(Self { variables })
    }

    pub fn to_shell(&self, shell_type: ShellType) -> String {
        let mut output = String::new();
        for v in &self.variables {
            let line = match shell_type {
                ShellType::Fish => {
                    format! {"set -x {} '{}'\n", v.name, v.value}
                }
                ShellType::Bash | ShellType::Zsh => {
                    format! {"export {}='{}'\n", v.name, v.value}
                }
                ShellType::Nu => {
                    format! {"$env.{} = '{}'\n", v.name, v.value}
                }
            };

            output += &line;
        }

        output
    }
}
