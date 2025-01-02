use anyhow::{anyhow, bail, Context, Result};
use futures::stream::FuturesUnordered;
use futures::StreamExt;
use reqwest::Client;
use semver::{Version, VersionReq};
use serde::{Deserialize, Serialize};

use crate::parse::Dep;

/// Data from crates.io: GET /crates/&lt;crate&gt;/versions
#[derive(Debug, Deserialize)]
struct VersionsResponse {
    versions: Vec<CratesIoVersion>,
}

#[derive(Debug, Deserialize)]
struct CratesIoVersion {
    num: String,
    license: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct LicenseReport {
    pub crate_name: String,
    pub matched_version: String,
    pub license: String,
}

pub async fn build_license_report(deps: &[Dep], client: &Client) -> Result<Vec<LicenseReport>> {
    let mut futures = FuturesUnordered::new();

    for dep in deps.iter().cloned() {
        let c = client.clone();
        futures.push(async move {
            match fetch_best_match(&c, &dep.name, &dep.version_req).await {
                Ok((matched_ver, license)) => LicenseReport {
                    crate_name: dep.name,
                    matched_version: matched_ver,
                    license: license.unwrap_or_else(|| "No license listed".to_string()),
                },
                Err(e) => LicenseReport {
                    crate_name: dep.name,
                    matched_version: "unknown".into(),
                    license: format!("Failed: {e}"),
                },
            }
        });
    }

    let mut results = Vec::new();
    while let Some(report) = futures.next().await {
        results.push(report);
    }

    Ok(results)
}

async fn fetch_best_match(
    client: &Client,
    crate_name: &str,
    constraint: &str,
) -> Result<(String, Option<String>)> {
    let url = format!("https://crates.io/api/v1/crates/{crate_name}/versions");
    let resp = client
        .get(&url)
        .send()
        .await
        .with_context(|| format!("Failed to fetch crates.io for crate={crate_name}"))?;

    if !resp.status().is_success() {
        bail!("crates.io returned status {}", resp.status());
    }

    let ver_resp: VersionsResponse = resp
        .json()
        .await
        .with_context(|| format!("Failed to parse JSON for crate={crate_name}"))?;

    let req_str = if constraint == "unspecified" {
        ">=0"
    } else {
        constraint
    };
    let req = VersionReq::parse(req_str)
        .map_err(|e| anyhow!("Semver parse error: {e} (constraint={constraint})"))?;

    let mut matches = ver_resp
        .versions
        .into_iter()
        .filter_map(|cv| {
            if let Ok(v) = Version::parse(&cv.num) {
                if req.matches(&v) {
                    return Some((v, cv.license));
                }
            }
            None
        })
        .collect::<Vec<_>>();

    matches.sort_by(|a, b| b.0.cmp(&a.0));

    if let Some((v, lic)) = matches.first() {
        Ok((v.to_string(), lic.clone()))
    } else {
        bail!("No crates.io versions matched constraint={constraint}");
    }
}
