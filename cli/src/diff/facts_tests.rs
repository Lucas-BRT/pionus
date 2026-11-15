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

#[test]
fn right_paraphrases_pass_and_wrong_ones_are_caught() {
    // 30 -> 60: what a person or a model may write about it.
    for right in [
        "fix: raise the timeout to 60",
        "fix: increase TIMEOUT from 30 to 60",
        "chore: bump timeout from 30 to 60",
        "fix: timeout is now 60 (was 30)",
        "fix(net): set TIMEOUT to 60",
        "fix: double the timeout",
        "fix: allow slow disks, timeout 30 -> 60",
        "fix: update the timeout",
        "fix: raise the timeout to 60 seconds",
        "fix: a longer timeout, up to 60.",
    ] {
        assert_eq!(contradiction_in(right, TIMEOUT), None, "{right}");
    }
    for wrong in [
        "fix: lower the timeout to 30",
        "fix: reduce the timeout from 60 to 30",
        "fix: decrease timeout to 30",
        "fix: shorten nothing, set the timeout to 30",
        "fix: the timeout goes from 60",
        "fix: increase the timeout to 30",
    ] {
        assert!(contradiction_in(wrong, TIMEOUT).is_some(), "{wrong}");
    }
}

#[test]
fn an_ambiguous_change_never_rejects_a_reply() {
    // Two numbers in one line, or a number that is not what the reply names:
    // not a plain pair, so the reply is not judged on it.
    let two = "@@ -1 +1 @@\n-size = (30, 40)\n+size = (60, 40)\n";
    assert_eq!(contradiction_in("fix: set the width to 60", two), None);
    assert!(contradiction_in("fix: set the width to 30", two).is_some());
    // Several files changing the same number to different values: either
    // direction is in the diff, and "to N" is only wrong when no change ends there.
    let both = "@@ -1 +1 @@\n-a = 30\n+a = 60\n@@ -9 +9 @@\n-b = 10\n+b = 30\n";
    assert_eq!(contradiction_in("fix: set b to 30", both), None);
    assert_eq!(contradiction_in("fix: lower the limits", both), None);
    // A removed line with nothing added is not a changed number.
    let gone = "@@ -1,2 +1 @@\n-const OLD: u64 = 30;\n const KEEP: u64 = 1;\n";
    assert_eq!(contradiction_in("fix: set it to 30", gone), None);
}
