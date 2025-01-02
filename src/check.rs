use crate::fetch::LicenseReport;
use anyhow::{bail, Result};

/// Splits a license expression like "(MIT OR Apache-2.0)" into multiple tokens, e.g. ["MIT", "Apache-2.0"].
/// We do a naive replacement of " OR ", " AND ", etc. with '|', then split on '|'.
fn parse_license_expression(license_str: &str) -> Vec<String> {
    let normalized = license_str
        .replace(" OR ", "|")
        .replace(" or ", "|")
        .replace(" AND ", "|")
        .replace(" and ", "|");

    normalized
        .split('|')
        .map(|s| s.trim_matches(|c: char| c.is_whitespace() || c == '(' || c == ')'))
        .filter(|s| !s.is_empty())
        .map(String::from)
        .collect()
}

/// Expand each user-supplied `--deny` or `--allow` string the same way, so
/// `--deny "MIT OR Apache-2.0"` becomes `["MIT", "Apache-2.0"]` in the final list.
pub fn expand_user_input(license_list: &[String]) -> Vec<String> {
    let mut expanded = Vec::new();
    for item in license_list {
        let subparts = parse_license_expression(item);
        expanded.extend(subparts);
    }
    expanded
}

/// Checks each crate's license(s) against the expanded deny/allow lists.
/// - If ANY sub-license is in `deny`, that's a violation.
/// - If `allow` is non-empty, ALL sub-licenses must be in `allow` or it's a violation.
///
/// We accumulate *all* violations, then fail at the end if any are found.
pub fn check_licenses(reports: &[LicenseReport], deny: &[String], allow: &[String]) -> Result<()> {
    let mut violations = Vec::new();

    for r in reports {
        let sub_licenses = parse_license_expression(&r.license);

        // DENY: if any sub-license is in deny => violation
        if !deny.is_empty() {
            for lic in &sub_licenses {
                if deny.contains(lic) {
                    violations.push(format!(
                        "Crate '{}': sub-license '{}' is in the deny list.",
                        r.crate_name, lic
                    ));
                }
            }
        }

        // ALLOW: if not empty, all sub-licenses must appear
        if !allow.is_empty() {
            for lic in &sub_licenses {
                if !allow.contains(lic) {
                    violations.push(format!(
                        "Crate '{}': sub-license '{}' is NOT in the allow list.",
                        r.crate_name, lic
                    ));
                }
            }
        }
    }

    if !violations.is_empty() {
        let msg = violations.join("\n");
        // Combine them in a single error
        bail!("License check found these violations:\n{}", msg);
    }

    Ok(())
}
