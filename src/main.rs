//! A tool that:
//! - Parses only [dependencies], [dev-dependencies], and [build-dependencies] from Cargo.toml
//! - Optionally skips "optional" dependencies
//! - Uses semver to find the newest crates.io version satisfying "0.12", etc.
//! - Fetches license info from crates.io for direct dependencies only
//! - Outputs Markdown or JSON
//! - Provides a CLI (generate, list, version) via clap

use anyhow::{Context, Result};
use clap::{Parser, Subcommand, ValueEnum};
use reqwest::blocking::Client;
use semver::{Version, VersionReq};
use serde::Deserialize;
use serde::Serialize;
use serde_json::json;
use std::collections::HashSet;
use std::fs::File;
use std::io::{Read, Write};
use toml::Value;

/// CLI definition
#[derive(Debug, Parser)]
#[command(
    author,
    version,
    about = "A tool to report licenses of direct dependencies from Cargo.toml."
)]
struct Cli {
    /// Subcommands
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Generate a license report (Markdown or JSON)
    Generate {
        /// Output format
        #[arg(long, value_enum, default_value = "md")]
        format: OutputFormat,

        /// Include dev-dependencies?
        #[arg(long, default_value_t = false)]
        dev: bool,

        /// Include build-dependencies?
        #[arg(long, default_value_t = false)]
        build: bool,

        /// Skip optional dependencies? (default = false => include them)
        #[arg(long)]
        skip_optional: bool,
    },

    /// Just list direct dependencies (by name & version constraint) from Cargo.toml
    List {
        #[arg(long, default_value_t = false)]
        dev: bool,
        #[arg(long, default_value_t = false)]
        build: bool,
        #[arg(long, default_value_t = false)]
        skip_optional: bool,
    },

    /// Show version of this tool and exit
    Version,
}

/// Output format for the Generate subcommand
#[derive(Debug, Clone, ValueEnum)]
enum OutputFormat {
    Md,
    Json,
}

/// We’ll store a dependency’s name & semver constraint
#[derive(Debug)]
struct Dep {
    name: String,
    version_req: String,
}

/// We fetch license data from crates.io’s `versions` endpoint
/// after we figure out which exact version to use (best match).
#[derive(Debug, Deserialize)]
struct VersionsResponse {
    versions: Vec<CratesIoVersion>,
}

/// A single version entry from crates.io
#[derive(Debug, Deserialize)]
struct CratesIoVersion {
    num: String,
    license: Option<String>,
}

/// Final info we display in the report
#[derive(Debug, Clone, Serialize)]
struct LicenseReport {
    crate_name: String,
    matched_version: String,
    license: String,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Command::Generate {
            format,
            dev,
            build,
            skip_optional,
        } => {
            let deps = parse_cargo_toml("Cargo.toml", dev, build, skip_optional)?;
            let client = Client::builder()
                .user_agent("cargo-licenses/0.3.0 (only direct dependencies)")
                .build()
                .context("Failed to build reqwest client")?;

            // Build a license report
            let results = build_license_report(&deps, &client)?;

            // Output
            match format {
                OutputFormat::Md => write_markdown(&results)?,
                OutputFormat::Json => write_json(&results)?,
            }
        }

        Command::List {
            dev,
            build,
            skip_optional,
        } => {
            let deps = parse_cargo_toml("Cargo.toml", dev, build, skip_optional)?;
            for d in deps {
                println!("{} = \"{}\"", d.name, d.version_req);
            }
        }

        Command::Version => {
            println!(
                "cargo-licenses (direct only) version {}",
                env!("CARGO_PKG_VERSION")
            );
        }
    }

    Ok(())
}

/// Parse Cargo.toml, extracting direct dependencies from
/// [dependencies], [dev-dependencies], and [build-dependencies] if requested.
/// If `skip_optional` is true, skip dependencies with `optional = true`.
fn parse_cargo_toml(
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

    // --- parse main [dependencies] ---
    if let Some(table) = toml_val.get("dependencies").and_then(|v| v.as_table()) {
        let deps = parse_deps_table(table, skip_optional)?;
        all_deps.extend(deps);
    }

    // --- parse [dev-dependencies] if user wants ---
    if include_dev {
        if let Some(table) = toml_val.get("dev-dependencies").and_then(|v| v.as_table()) {
            let deps = parse_deps_table(table, skip_optional)?;
            all_deps.extend(deps);
        }
    }

    // --- parse [build-dependencies] if user wants ---
    if include_build {
        if let Some(table) = toml_val
            .get("build-dependencies")
            .and_then(|v| v.as_table())
        {
            let deps = parse_deps_table(table, skip_optional)?;
            all_deps.extend(deps);
        }
    }

    // Remove duplicates if they appear in multiple sections
    // We'll store them in a set keyed by (name, version_req).
    let mut unique_set = HashSet::new();
    let mut unique_vec = Vec::new();
    for d in all_deps {
        let key = (d.name.clone(), d.version_req.clone());
        if !unique_set.contains(&key) {
            unique_set.insert(key);
            unique_vec.push(d);
        }
    }

    Ok(unique_vec)
}

/// Parse a table (dependencies, dev-dependencies, etc.) into a list of Dep {name, version_req}.
/// If `skip_optional` is true, we skip any that has `optional=true`.
fn parse_deps_table(
    table: &toml::map::Map<String, Value>,
    skip_optional: bool,
) -> Result<Vec<Dep>> {
    let mut deps = Vec::new();
    for (name, item) in table.iter() {
        // item could be "0.12" or { version="0.12", optional=true } or ...
        match item {
            Value::String(ver_req) => {
                deps.push(Dep {
                    name: name.clone(),
                    version_req: ver_req.clone(),
                });
            }
            Value::Table(tbl) => {
                // e.g. reqwest = { version="0.12", optional=true }
                // or openssl = { git="https://github.com/...", branch="..." }
                // We'll read "version" if present, else fallback to "unspecified"
                // If skip_optional && tbl["optional"]==true, skip it.
                if skip_optional {
                    if let Some(Value::Boolean(true)) = tbl.get("optional") {
                        // skip
                        continue;
                    }
                }
                let version_req = if let Some(Value::String(v)) = tbl.get("version") {
                    v.clone()
                } else {
                    // no version specified
                    "unspecified".to_string()
                };
                deps.push(Dep {
                    name: name.clone(),
                    version_req,
                });
            }
            _ => {
                // Could be an array or something else. We'll just store "unspecified".
                deps.push(Dep {
                    name: name.clone(),
                    version_req: "unspecified".to_string(),
                });
            }
        }
    }
    Ok(deps)
}

/// Build a license report for each direct dependency from Cargo.toml,
/// using crates.io's "GET /crates/<name>/versions" and semver matching
/// to find the newest version that satisfies the user's version_req.
fn build_license_report(deps: &[Dep], client: &Client) -> Result<Vec<LicenseReport>> {
    let mut reports = Vec::new();
    for dep in deps {
        let (matched_ver, license) = match fetch_best_match(client, &dep.name, &dep.version_req) {
            Ok((ver, lic)) => (ver, lic),
            Err(e) => {
                // If we fail, record the error as license
                let msg = format!("Failed: {}", e);
                reports.push(LicenseReport {
                    crate_name: dep.name.clone(),
                    matched_version: "unknown".into(),
                    license: msg,
                });
                continue;
            }
        };

        reports.push(LicenseReport {
            crate_name: dep.name.clone(),
            matched_version: matched_ver,
            license: license.unwrap_or_else(|| "No license listed".to_string()),
        });
    }

    Ok(reports)
}

/// 1) Query crates.io: GET /api/v1/crates/<crate>/versions
/// 2) Parse the versions array
/// 3) Use semver to find the newest version that satisfies <dep.version_req>.
fn fetch_best_match(
    client: &Client,
    crate_name: &str,
    constraint: &str,
) -> Result<(String, Option<String>)> {
    let url = format!("https://crates.io/api/v1/crates/{}/versions", crate_name);
    let resp = client
        .get(&url)
        .send()
        .with_context(|| format!("Failed to fetch crates.io for crate={}", crate_name))?;

    if !resp.status().is_success() {
        anyhow::bail!("crates.io returned status {}", resp.status());
    }

    let ver_resp: VersionsResponse = resp
        .json()
        .with_context(|| format!("Failed to parse JSON for crate={}", crate_name))?;

    // Parse the user's version_req. If "unspecified", we treat as ">=0"
    let version_req_str = if constraint == "unspecified" {
        ">=0".to_string()
    } else {
        constraint.to_string()
    };

    let req = VersionReq::parse(&version_req_str).with_context(|| {
        format!(
            "Failed to parse semver constraint='{}' for crate={}",
            constraint, crate_name
        )
    })?;

    // Filter crates.io versions to those that match the constraint
    let mut matches = ver_resp
        .versions
        .iter()
        .filter_map(|cv| {
            if let Ok(parsed) = Version::parse(&cv.num) {
                if req.matches(&parsed) {
                    return Some((parsed, cv.license.clone()));
                }
            }
            None
        })
        .collect::<Vec<_>>();

    // Sort descending so the newest is first
    matches.sort_by(|a, b| b.0.cmp(&a.0));

    if let Some((ver, lic)) = matches.first() {
        // Return the first (highest) match
        Ok((ver.to_string(), lic.clone()))
    } else {
        // If none matched, maybe the user gave "1.0" but crate has no 1.0. We'll fail
        anyhow::bail!(
            "No versions on crates.io matched the constraint={}",
            constraint
        );
    }
}

/// Write a Markdown file
fn write_markdown(reports: &[LicenseReport]) -> Result<()> {
    let path = ".license_report.md";
    let mut file = File::create(path)?;
    writeln!(file, "# License Report")?;
    writeln!(
        file,
        "This report lists direct dependencies (only from Cargo.toml) and their matched licenses.\n"
    )?;

    for r in reports {
        writeln!(
            file,
            "- **{}** (version: `{}`) → *{}*",
            r.crate_name, r.matched_version, r.license
        )?;
    }

    println!("Generated Markdown: {}", path);
    Ok(())
}

/// Write a JSON file
fn write_json(reports: &[LicenseReport]) -> Result<()> {
    let path = ".license_report.json";
    let mut file = File::create(path)?;
    let json_val = json!(reports);
    writeln!(file, "{}", serde_json::to_string_pretty(&json_val)?)?;

    println!("Generated JSON: {}", path);
    Ok(())
}
