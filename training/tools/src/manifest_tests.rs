use super::*;

fn set(dir: &Path, name: &str, rows: &[(&str, &str, &str)]) -> PathBuf {
    let path = dir.join(name);
    std::fs::create_dir_all(&path).unwrap();
    let meta: String = rows
        .iter()
        .map(|(repo, hash, target)| {
            format!(
                "{}\n",
                json!({"repo": repo, "hash": hash, "target": target, "set": "free"})
            )
        })
        .collect();
    std::fs::write(path.join("meta.jsonl"), meta).unwrap();
    std::fs::write(path.join("inputs.jsonl"), "{}\n".repeat(rows.len())).unwrap();
    path
}

#[test]
fn a_set_is_described_and_hashed() {
    let dir = tempfile::tempdir().unwrap();
    let a = set(
        dir.path(),
        "a",
        &[("o/x", "1", "t"), ("o/x", "2", "t"), ("p/y", "3", "u")],
    );
    let report = manifest(&[a], &[]).unwrap();
    let a = &report["sets"]["a"];
    assert_eq!(
        (a["n"].as_u64(), a["repos"].as_u64(), a["owners"].as_u64()),
        (Some(3), Some(2), Some(2))
    );
    assert_eq!(a["duplicate_targets"], 1);
    assert_eq!(a["largest_repo_share"], 0.667);
    assert_eq!(a["files"]["inputs"]["sha256"], sha256_hex(b"{}\n{}\n{}\n"));
    assert_eq!(a["files"]["meta"]["lines"], 3);
}

#[test]
fn what_two_sets_share_is_counted_by_commit_repository_target_and_owner() {
    let dir = tempfile::tempdir().unwrap();
    let eval = set(
        dir.path(),
        "eval",
        &[
            ("apache/a", "1", "t1"),
            ("apache/b", "2", "t2"),
            ("z/z", "3", "t3"),
        ],
    );
    let train = set(
        dir.path(),
        "train",
        &[
            ("apache/a", "9", "t9"),
            ("apache/c", "1", "t1"),
            ("q/q", "8", "t8"),
        ],
    );
    let same = eval.clone();
    let report = manifest(&[eval], &[train, same]).unwrap();
    let rows = report["overlap"].as_array().unwrap();
    assert_eq!(rows.len(), 1, "a set is not compared with itself");
    let row = &rows[0];
    assert_eq!(
        (
            row["hashes"].as_u64(),
            row["repos"].as_u64(),
            row["targets"].as_u64()
        ),
        (Some(1), Some(1), Some(1))
    );
    // `apache` is shared: two of the eval commits come from it.
    assert_eq!(
        (
            row["owners"].as_u64(),
            row["a_commits_from_shared_owners"].as_u64()
        ),
        (Some(1), Some(2))
    );
}

#[test]
fn a_missing_file_is_named() {
    let dir = tempfile::tempdir().unwrap();
    let error = manifest(&[dir.path().join("none")], &[]).unwrap_err();
    assert!(error.to_string().contains("inputs.jsonl"));
}
