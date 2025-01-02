use anyhow::{Context, Result};
use std::collections::HashSet;
use std::fs::File;
use std::io::Read;
use toml::Value;

#[derive(Debug, Clone)]
pub struct Dep {
    pub name: String,
    pub version_req: String,
}

pub fn parse_cargo_toml(
    path: &str,
    include_dev: bool,
    include_build: bool,
    skip_optional: bool,
) -> Result<Vec<Dep>> {
    let mut file = File::open(path).context("Failed to open Cargo.toml")?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    let toml_val: Value = toml::from_str(&contents)?;

    let mut all_deps = Vec::new();

    if let Some(table) = toml_val.get("dependencies").and_then(|v| v.as_table()) {
        all_deps.extend(parse_deps_table(table, skip_optional)?);
    }
    if include_dev {
        if let Some(table) = toml_val.get("dev-dependencies").and_then(|v| v.as_table()) {
            all_deps.extend(parse_deps_table(table, skip_optional)?);
        }
    }
    if include_build {
        if let Some(table) = toml_val
            .get("build-dependencies")
            .and_then(|v| v.as_table())
        {
            all_deps.extend(parse_deps_table(table, skip_optional)?);
        }
    }

    let mut seen = HashSet::new();
    let mut unique = Vec::new();
    for d in all_deps {
        let key = (d.name.clone(), d.version_req.clone());
        if !seen.contains(&key) {
            seen.insert(key);
            unique.push(d);
        }
    }

    Ok(unique)
}

fn parse_deps_table(
    table: &toml::map::Map<String, Value>,
    skip_optional: bool,
) -> Result<Vec<Dep>> {
    let mut deps = Vec::new();

    for (name, item) in table.iter() {
        match item {
            Value::String(ver_req) => {
                deps.push(Dep {
                    name: name.clone(),
                    version_req: ver_req.clone(),
                });
            }
            Value::Table(tbl) => {
                if skip_optional {
                    if let Some(Value::Boolean(true)) = tbl.get("optional") {
                        continue;
                    }
                }
                let version_req = if let Some(Value::String(v)) = tbl.get("version") {
                    v.clone()
                } else {
                    "unspecified".to_string()
                };
                deps.push(Dep {
                    name: name.clone(),
                    version_req,
                });
            }
            _ => {
                deps.push(Dep {
                    name: name.clone(),
                    version_req: "unspecified".to_string(),
                });
            }
        }
    }

    Ok(deps)
}
