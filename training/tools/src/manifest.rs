//! A manifest of evaluation sets: what each holds, the SHA-256 of its files,
//! and what any two of them share.
//!
//! Re-run it to check that a set was not touched, and to audit leakage: two
//! sets that must be apart (an evaluation set and what a model trained on)
//! share no commit, repository or target, and show how many commits come
//! from the same owner (`apache`, `microsoft`): conventions travel inside an
//! owner, so a shared owner is a softer leak than a shared repository.
//!
//! Dates are not in the metadata, so a split by time cannot be audited here.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use pionus::models::sha256_hex;
use serde_json::{Value, json};

use crate::pytext::read_lines;

struct Set {
    name: String,
    meta: Vec<Value>,
    files: Value,
}

fn owner(repo: &str) -> &str {
    repo.split('/').next().unwrap_or(repo)
}

fn read(dir: &Path) -> Result<Set> {
    let mut files = serde_json::Map::new();
    for name in ["inputs.jsonl", "meta.jsonl"] {
        let bytes = std::fs::read(dir.join(name))
            .with_context(|| format!("cannot read {name} of {}", dir.display()))?;
        files.insert(
            name.trim_end_matches(".jsonl").to_owned(),
            json!({"lines": bytes.iter().filter(|b| **b == b'\n').count(), "sha256": sha256_hex(&bytes)}),
        );
    }
    let meta = read_lines(&dir.join("meta.jsonl"))?
        .iter()
        .map(|l| serde_json::from_str(l).context("invalid meta line"))
        .collect::<Result<_>>()?;
    let name = dir.file_name().map_or_else(
        || dir.display().to_string(),
        |n| n.to_string_lossy().into_owned(),
    );
    Ok(Set {
        name,
        meta,
        files: Value::Object(files),
    })
}

/// The values of `key` over the set, as strings.
fn values<'a>(set: &'a Set, key: &str) -> Vec<&'a str> {
    set.meta.iter().filter_map(|m| m[key].as_str()).collect()
}

fn summary(set: &Set) -> Value {
    let repos = values(set, "repo");
    let distinct: BTreeSet<&str> = repos.iter().copied().collect();
    let owners: BTreeSet<&str> = repos.iter().map(|r| owner(r)).collect();
    let targets = values(set, "target");
    let top = distinct
        .iter()
        .map(|r| repos.iter().filter(|x| *x == r).count())
        .max()
        .unwrap_or(0);
    let mut kinds = serde_json::Map::new();
    for kind in set
        .meta
        .iter()
        .filter_map(|m| m["set"].as_str())
        .collect::<BTreeSet<_>>()
    {
        let n = set.meta.iter().filter(|m| m["set"] == kind).count();
        kinds.insert(kind.to_owned(), json!(n));
    }
    json!({
        "n": set.meta.len(),
        "repos": distinct.len(),
        "owners": owners.len(),
        "largest_repo_share": crate::score::round3(top as f64 / set.meta.len().max(1) as f64),
        "duplicate_targets": targets.len() - targets.iter().collect::<BTreeSet<_>>().len(),
        "sets": kinds,
        "files": set.files,
    })
}

fn shared(a: &Set, b: &Set) -> Value {
    let of = |s: &Set, key: &str| -> BTreeSet<String> {
        values(s, key).into_iter().map(str::to_owned).collect()
    };
    let owners = |s: &Set| -> BTreeSet<String> {
        values(s, "repo")
            .into_iter()
            .map(|r| owner(r).to_owned())
            .collect()
    };
    let same_owner = owners(a)
        .intersection(&owners(b))
        .cloned()
        .collect::<BTreeSet<_>>();
    let commits = values(a, "repo")
        .into_iter()
        .filter(|r| same_owner.contains(owner(r)))
        .count();
    json!({
        "a": a.name,
        "b": b.name,
        "hashes": of(a, "hash").intersection(&of(b, "hash")).count(),
        "repos": of(a, "repo").intersection(&of(b, "repo")).count(),
        "targets": of(a, "target").intersection(&of(b, "target")).count(),
        "owners": same_owner.len(),
        "a_commits_from_shared_owners": commits,
    })
}

/// The manifest of `sets`, and what each of them shares with each of
/// `against`; a pair of the same name is left out.
pub fn manifest(sets: &[PathBuf], against: &[PathBuf]) -> Result<Value> {
    let sets: Vec<Set> = sets.iter().map(|d| read(d)).collect::<Result<_>>()?;
    let against: Vec<Set> = against.iter().map(|d| read(d)).collect::<Result<_>>()?;
    let mut described = serde_json::Map::new();
    for set in sets.iter().chain(&against) {
        described
            .entry(set.name.clone())
            .or_insert_with(|| summary(set));
    }
    let overlap: Vec<Value> = sets
        .iter()
        .flat_map(|a| {
            against
                .iter()
                .filter(move |b| b.name != a.name)
                .map(move |b| shared(a, b))
        })
        .collect();
    Ok(json!({"sets": described, "overlap": overlap}))
}

#[cfg(test)]
#[path = "manifest_tests.rs"]
mod tests;
