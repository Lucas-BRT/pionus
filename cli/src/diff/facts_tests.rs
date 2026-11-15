use super::*;

const TIMEOUT: &str = "@@ -1 +1 @@\n-const TIMEOUT: u64 = 30;\n+const TIMEOUT: u64 = 60;\n";

#[test]
fn a_number_moved_the_wrong_way_is_a_contradiction_with_its_reason() {
    assert_eq!(
        contradiction_in("fix: decrease timeout to 30", TIMEOUT).unwrap(),
        "the diff changes 30 to 60, not to 30"
    );
    assert!(contradiction_in("fix: raise timeout from 60", TIMEOUT).is_some());
    assert!(
        contradiction_in("fix: lower the timeout", TIMEOUT)
            .unwrap()
            .ends_with("which is an increase")
    );
    assert!(contradiction_in("fix: increase timeout to 60", TIMEOUT).is_none());
    assert!(contradiction_in("fix: double the timeout from 30 to 60", TIMEOUT).is_none());
    assert!(contradiction_in("fix: reduce timeout", "@@ -1 +1 @@\n-x = 5\n+x = 3\n").is_none());
}

#[test]
fn what_is_not_a_plain_number_in_place_is_unknown() {
    // No pair, a renamed identifier, a percentage and a name with a digit.
    assert!(contradiction_in("fix: decrease timeout to 30", "").is_none());
    assert!(
        contradiction_in(
            "fix: set a to 30",
            "@@ -1 +1 @@\n-let a = 30;\n+let b = 60;\n"
        )
        .is_none()
    );
    let fee = "@@ -1 +1 @@\n-fee = 0.12\n+fee = 0.15\n";
    assert!(contradiction_in("fix: cut the fee to 12%", fee).is_none());
    assert!(contradiction_in("fix: lower the fee to 0.12", fee).is_some());
    let names = "@@ -1 +1 @@\n-use v2::api;\n+use v3::api;\n";
    assert!(contradiction_in("fix: use v3, not v2, to 2", names).is_none());
}

#[test]
fn a_rise_or_fall_word_needs_a_diff_that_is_that_one_change() {
    // Another change in the diff: "drop" or "reduce" may be about it.
    let mixed = "@@ -1,3 +1,2 @@\n-version = 1\n+version = 2\n-legacy()\n";
    assert!(contradiction_in("refactor: reduce the legacy calls", mixed).is_none());
    let two = "@@ -1,2 +1,2 @@\n-a = 1\n-b = 9\n+a = 5\n+b = 2\n";
    assert!(contradiction_in("fix: lower the limits", two).is_none());
    // But an explicit wrong number is still caught.
    assert!(contradiction_in("fix: set a to 1", two).is_some());
}

#[test]
fn a_signed_number_is_one_number_and_a_hyphen_is_not_a_sign() {
    assert_eq!(numbers("x = -5"), [(4, 6)]);
    assert_eq!(numbers("step-5"), [(5, 6)]);
    assert_eq!(numbers("a-1.5-b"), [(2, 5)]);
    assert!(numbers("utf8 v2").is_empty());
}

fn file(path: &str, hunks: &str) -> FileDiff {
    FileDiff {
        path: path.into(),
        old_path: None,
        lines: Some((1, 1)),
        hunks: hunks.into(),
    }
}

#[test]
fn the_numbers_that_changed_are_listed_per_file_and_versions_are_not() {
    let files = [
        file("src/config.rs", TIMEOUT),
        file(
            "Cargo.toml",
            "@@ -1 +1 @@\n-version = \"0.14.0\"\n+version = \"0.15.0\"\n",
        ),
        file("Cargo.lock", "@@ -1 +1 @@\n-x = 1\n+x = 2\n"),
        file("src/same.rs", "@@ -1 +1 @@\n-a = 1\n+a = 1\n"),
    ];
    assert_eq!(
        described(&files).unwrap(),
        "Numbers that changed: 30 -> 60 (src/config.rs).\n"
    );
    assert_eq!(described(&files[1..]), None);
    // At most four, each once.
    let many: String = (1..=6)
        .map(|n| format!("@@ -{n} +{n} @@\n-v{n} = {n}\n+v{n} = {}\n", n + 10))
        .collect();
    let line = described(&[file("a.rs", &many)]).unwrap();
    assert_eq!(line.matches(" -> ").count(), 4);
}
