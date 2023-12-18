use std::{fs, path::Path};

use anyhow::{bail, Result};
use regex::Regex;

use crate::types::secret::Secret;

use super::app::App;

#[derive(Debug)]
pub struct Renv {
    pub variables: Vec<Variable>,
}

pub enum OutputShell {
    Fish,
}

#[derive(Debug)]
pub struct Variable {
    pub name: String,
    pub value: String,
}

impl Variable {
    pub fn from_string(s: &str) -> Result<Self> {
        let re_var = Regex::new(r"^(?P<var_name>[\w]+)=(?P<var_value>[^=]*)$")?;
        if let Some(captures) = re_var.captures(s) {
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
    let re = Regex::new(r"\{\{(\w+)\}\}")?;
    for m in re.find_iter(s) {
        let secret_name = m.as_str().trim_start_matches('{').trim_end_matches('}');
        let secret = Secret::get(&app.db, secret_name).await?;
        let clear_text = secret.to_cleartext(&app.derived_key)?;
        new_s = new_s.replace(m.as_str(), clear_text.value.trim());
    }

    Ok(new_s.to_string())
}

impl Renv {
    // Loads the given `path` and parses it's contents for variable names and secret names. Secret
    // names will be replaced with their secret values.
    //
    // For now, this will only accept variable definitions in the form:
    // VARIABLE_NAME={{secret_name}}
    //
    // In the future, this should accept defintions like:
    // VARIABLE_NAME=non-secret text
    // VARIABLE_NAME=prefix_{{secret_name}}
    pub async fn load(app: &App, path: &Path) -> Result<Self> {
        let contents = fs::read_to_string(path)?;
        let lines: Vec<String> = contents.lines().map(|l| l.trim().to_string()).collect();

        let mut variables = vec![];

        for (i, line) in lines.iter().enumerate() {
            let var = match Variable::from_string(line) {
                Ok(mut v) => {
                    v.value = replace_template_vars(app, &v.value).await?;
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
}
