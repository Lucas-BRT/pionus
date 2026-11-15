//! What the diff states without ambiguity, to check a reply against.
//!
//! The only family so far is a number that changes in place: a removed line
//! and the added line next to it, equal once their numbers are blanked. A
//! reply that says `to 30` where 30 was the old value, `from 60` where 60 is
//! the new one, or that a one-line change goes the other way, contradicts
//! the diff. Anything else (no such pair, a number written as a percentage,
//! a change that is not alone) is unknown and never rejects a reply.

use super::FileDiff;

#[derive(Debug, Clone, PartialEq)]
struct Change {
    old: f64,
    new: f64,
    old_text: String,
    new_text: String,
}

/// The numbers of `line` as (start, end) byte ranges: digits with an optional
/// fraction, and a leading `-` that is a sign, not a hyphen or a minus.
fn numbers(line: &str) -> Vec<(usize, usize)> {
    let bytes = line.as_bytes();
    let mut found = Vec::new();
    let mut i = 0;
    while i < bytes.len() {
        if !bytes[i].is_ascii_digit() {
            i += 1;
            continue;
        }
        let mut start = i;
        let word_before =
            start > 0 && (bytes[start - 1].is_ascii_alphanumeric() || bytes[start - 1] == b'_');
        let mut end = i;
        while end < bytes.len() && bytes[end].is_ascii_digit() {
            end += 1;
        }
        if end + 1 < bytes.len() && bytes[end] == b'.' && bytes[end + 1].is_ascii_digit() {
            end += 1;
            while end < bytes.len() && bytes[end].is_ascii_digit() {
                end += 1;
            }
        }
        if start > 0 && bytes[start - 1] == b'-' {
            let before = start.checked_sub(2).map(|p| bytes[p]);
            if before.is_none_or(|b| !(b.is_ascii_alphanumeric() || b == b')' || b == b'_')) {
                start -= 1;
            }
        }
        // A digit inside a name (`v2`, `utf8`) is part of the name.
        if !word_before {
            found.push((start, end));
        }
        i = end;
    }
    found
}

fn blanked(line: &str) -> String {
    let mut out = String::new();
    let mut at = 0;
    for (start, end) in numbers(line) {
        out.push_str(&line[at..start]);
        out.push('#');
        at = end;
    }
    out.push_str(&line[at..]);
    out
}

/// The numbers that changed in place, in the order of the diff.
fn changes(hunks: &str) -> Vec<Change> {
    let (mut removed, mut added): (Vec<&str>, Vec<&str>) = (Vec::new(), Vec::new());
    let mut found = Vec::new();
    let mut pair_up = |removed: &mut Vec<&str>, added: &mut Vec<&str>| {
        for (a, b) in removed.iter().zip(added.iter()) {
            let (x, y) = (numbers(a), numbers(b));
            if x.len() != y.len() || blanked(a) != blanked(b) {
                continue;
            }
            for (p, q) in x.iter().zip(&y) {
                let (old_text, new_text) = (&a[p.0..p.1], &b[q.0..q.1]);
                if let (Ok(old), Ok(new)) = (old_text.parse::<f64>(), new_text.parse::<f64>())
                    && old != new
                {
                    found.push(Change {
                        old,
                        new,
                        old_text: old_text.to_owned(),
                        new_text: new_text.to_owned(),
                    });
                }
            }
        }
        removed.clear();
        added.clear();
    };
    for line in hunks.lines() {
        if line.starts_with("---") || line.starts_with("+++") {
            continue;
        }
        if let Some(rest) = line.strip_prefix('-') {
            if !added.is_empty() {
                pair_up(&mut removed, &mut added);
            }
            removed.push(rest);
        } else if let Some(rest) = line.strip_prefix('+') {
            added.push(rest);
        } else {
            pair_up(&mut removed, &mut added);
        }
    }
    pair_up(&mut removed, &mut added);
    found
}

/// The description of a subject, lowercase, without its `type(scope):` header.
fn description(subject: &str) -> String {
    let text = match subject.split_once(": ") {
        Some((head, rest)) if !head.contains(' ') => rest,
        _ => subject,
    };
    text.to_lowercase()
}

/// The number written after each `word` in `text`, if it is a plain one: not
/// a percentage, whose scale the diff may write differently (`0.25`).
fn after(text: &str, word: &str) -> Vec<f64> {
    let tokens: Vec<&str> = text.split_whitespace().collect();
    tokens
        .windows(2)
        .filter(|w| w[0] == word)
        .filter_map(|w| {
            let token = w[1].trim_end_matches([',', ';', ')', '.']);
            token.parse::<f64>().ok().filter(|_| !w[1].contains('%'))
        })
        .collect()
}

const UP: [&str; 5] = ["increase", "increases", "raise", "raises", "higher"];
const DOWN: [&str; 6] = [
    "decrease",
    "decreases",
    "lower",
    "lowers",
    "reduce",
    "reduces",
];

fn mentions(text: &str, words: &[&str]) -> bool {
    text.split(|c: char| !c.is_alphanumeric())
        .any(|w| words.contains(&w))
}

/// How a reply contradicts the numbers that changed in `hunks`, if it does.
pub fn contradiction_in(subject: &str, hunks: &str) -> Option<String> {
    let changes = changes(hunks);
    if changes.is_empty() {
        return None;
    }
    let text = description(subject);
    let says = |c: &Change| format!("the diff changes {} to {}", c.old_text, c.new_text);
    for n in after(&text, "to") {
        if let Some(c) = changes.iter().find(|c| c.old == n)
            && !changes.iter().any(|c| c.new == n)
        {
            return Some(format!("{}, not to {}", says(c), c.old_text));
        }
    }
    for n in after(&text, "from") {
        if let Some(c) = changes.iter().find(|c| c.new == n)
            && !changes.iter().any(|c| c.old == n)
        {
            return Some(format!("{}, not from {}", says(c), c.new_text));
        }
    }
    // A rise or fall is checked only on a diff that is that one change: with
    // anything else in it, the verb may be about something else.
    let lines = hunks
        .lines()
        .filter(|l| {
            (l.starts_with('+') || l.starts_with('-'))
                && !l.starts_with("+++")
                && !l.starts_with("---")
        })
        .count();
    if let [c] = changes.as_slice()
        && lines == 2
    {
        if mentions(&text, &UP) && c.new < c.old {
            return Some(format!("{}, which is a decrease", says(c)));
        }
        if mentions(&text, &DOWN) && c.new > c.old {
            return Some(format!("{}, which is an increase", says(c)));
        }
    }
    None
}

/// [`contradiction_in`] over the hunks of `files`.
pub fn contradiction(subject: &str, files: &[FileDiff]) -> Option<String> {
    let hunks: Vec<&str> = files.iter().map(|f| f.hunks.as_str()).collect();
    contradiction_in(subject, &hunks.join("\n"))
}

#[cfg(test)]
#[path = "facts_tests.rs"]
mod tests;
