use anyhow::Result;
use clap::Parser;
use tokio::main;

mod check;
mod cli;
mod fetch;
mod parse;
mod report;

use check::{check_licenses, expand_user_input};
use cli::{Cli, Command, OutputFormat};
use fetch::build_license_report;
use parse::parse_cargo_toml;
use report::{write_json, write_markdown};

#[main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Command::Generate {
            format,
            dev,
            build,
            skip_optional,
        } => {
            let deps = parse_cargo_toml("Cargo.toml", dev, build, skip_optional)?;
            let client = reqwest::Client::builder()
                .user_agent("cargo-licenses/0.2.1 (async direct deps)")
                .build()?;

            let results = build_license_report(&deps, &client).await?;
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

        Command::Check {
            deny,
            allow,
            dev,
            build,
            skip_optional,
        } => {
            // 1) Parse direct dependencies
            let deps = parse_cargo_toml("Cargo.toml", dev, build, skip_optional)?;

            // 2) Build async client
            let client = reqwest::Client::builder()
                .user_agent("cargo-licenses/0.2.1 (async check mode)")
                .build()?;

            // 3) Fetch license reports
            let results = build_license_report(&deps, &client).await?;

            // 4) Expand user input so "MIT OR Apache-2.0" => ["MIT","Apache-2.0"], etc.
            let deny_expanded = expand_user_input(&deny);
            let allow_expanded = expand_user_input(&allow);

            // 5) Check them
            match check_licenses(&results, &deny_expanded, &allow_expanded) {
                Ok(_) => {
                    println!("All licenses are acceptable under the given policy.");
                }
                Err(e) => {
                    eprintln!("License check failed: {}", e);
                    std::process::exit(1);
                }
            }
        }

        Command::Version => {
            println!(
                "cargo-licenses (async) version {}",
                env!("CARGO_PKG_VERSION")
            );
        }
    }

    Ok(())
}
