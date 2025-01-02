use anyhow::Result;
use serde_json::json;
use std::fs::File;
use std::io::Write;

use crate::fetch::LicenseReport;

pub fn write_markdown(results: &[LicenseReport]) -> Result<()> {
    let path = ".license_report.md";
    let mut file = File::create(path)?;

    writeln!(file, "# License Report")?;
    writeln!(
        file,
        "This report lists direct dependencies (async) and their matched licenses.\n"
    )?;

    for r in results {
        writeln!(
            file,
            "- **{}** (version: `{}`) → *{}*",
            r.crate_name, r.matched_version, r.license
        )?;
    }

    println!("Generated Markdown: {path}");
    Ok(())
}

pub fn write_json(results: &[LicenseReport]) -> Result<()> {
    let path = ".license_report.json";
    let mut file = File::create(path)?;

    let json_val = json!(results);
    writeln!(file, "{}", serde_json::to_string_pretty(&json_val)?)?;

    println!("Generated JSON: {path}");
    Ok(())
}
