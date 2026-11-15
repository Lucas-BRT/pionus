use std::fmt::Write as _;

use super::annotate;
use super::{CompactDiff, FileDiff, FileKind, MIN_BODY_LINES, estimate_tokens};
use crate::git::FileChange;

/// Fits `files` into `budget` tokens: the stat first, then hunks.
///
/// The stat is capped at half the budget; files beyond it are summarised in
/// one line. Hunks are shared by priority tier: each tier splits what is left
/// evenly, files needing less than their share give the rest back, and
/// whatever the tier does not use goes to the next one. A share too small to
/// show any of a file's lines is raised to the least that does, while the
/// budget lasts; the files left without it appear in the stat only.
///
/// The only way to exceed `budget` is a budget too small for the stat's
/// summary line itself, which is always kept.
pub fn compact(files: &[FileDiff], budget: usize) -> CompactDiff {
    let lines: Vec<String> = files.iter().map(stat_line).collect();
    let useful: Vec<bool> = files.iter().map(FileDiff::has_useful_hunks).collect();
    compact_with(files, &lines, &useful, budget)
}

/// Like [`compact`], with the annotated file list of [`annotate`], which
/// needs each file's status from `staged`; files whose change is only
/// whitespace keep their line in the list, saying which, and leave the
/// diff, unless the change is code reformatted.
pub fn compact_annotated(files: &[FileDiff], staged: &[FileChange], budget: usize) -> CompactDiff {
    let lines = annotate::stat_lines(files, staged);
    let useful: Vec<bool> = files
        .iter()
        .map(|f| {
            f.has_useful_hunks()
                && match annotate::change_of(f) {
                    annotate::Change::Whitespace(spacing) => spacing.shows_in_diff(),
                    _ => true,
                }
        })
        .collect();
    compact_with(files, &lines, &useful, budget)
}

/// [`compact`] with the stat's `lines` and which files' hunks are `useful`.
fn compact_with(
    files: &[FileDiff],
    lines: &[String],
    useful: &[bool],
    budget: usize,
) -> CompactDiff {
    let stat = render_lines(lines, budget / 2);
    let mut remaining = budget.saturating_sub(estimate_tokens(&stat));

    // (kind, tokens needed, index): sorting puts each tier's smallest
    // files first, so their leftovers flow to the larger ones.
    let mut order: Vec<(FileKind, usize, usize)> = files
        .iter()
        .enumerate()
        .filter(|&(i, _)| useful[i])
        .map(|(i, f)| (FileKind::of(f), estimate_tokens(&f.section()), i))
        .collect();
    order.sort();

    let mut allotted = vec![0usize; files.len()];
    for tier in order.chunk_by(|a, b| a.0 == b.0) {
        for (left, &(_, need, i)) in (1..=tier.len()).rev().zip(tier) {
            let least = files[i].min_tokens();
            let share = need.min(remaining / left);
            let grant = if share >= least {
                share
            } else if remaining >= least {
                least
            } else {
                0
            };
            allotted[i] = grant;
            remaining -= grant;
        }
    }

    let mut patch = String::new();
    let mut truncated = 0;
    for (i, (file, &tokens)) in files.iter().zip(&allotted).enumerate() {
        if tokens == 0 {
            truncated += usize::from(useful[i]);
            continue;
        }
        let (text, cut) = fit_file(file, tokens);
        truncated += usize::from(cut);
        patch.push_str(&text);
    }
    CompactDiff {
        stat,
        patch,
        truncated,
    }
}

#[cfg(test)]
pub(super) fn render_stat(files: &[FileDiff], budget: usize) -> String {
    let lines: Vec<String> = files.iter().map(stat_line).collect();
    render_lines(&lines, budget)
}

fn render_lines(lines: &[String], budget: usize) -> String {
    let total: usize = lines.iter().map(|l| estimate_tokens(l)).sum();
    if total <= budget {
        return lines.concat();
    }
    // Not everything fits: keep room for the summary of what is left out.
    let mut stat = String::new();
    let mut used = 0;
    for (shown, line) in lines.iter().enumerate() {
        let summary = format!("... and {} more files\n", lines.len() - shown);
        if used + estimate_tokens(line) + estimate_tokens(&summary) > budget {
            stat.push_str(&summary);
            break;
        }
        stat.push_str(line);
        used += estimate_tokens(line);
    }
    stat
}

pub(super) fn stat_line(file: &FileDiff) -> String {
    let path = match &file.old_path {
        Some(old) => format!("{old} => {}", file.path),
        None => file.path.clone(),
    };
    match file.lines {
        Some((added, deleted)) => format!("{path} | +{added} -{deleted}\n"),
        None => format!("{path} | binary\n"),
    }
}

/// The file's whole hunks that fit in `tokens`, with a marker for the others;
/// `None` when it has a single hunk or none fits whole.
fn fit_hunks(file: &FileDiff, tokens: usize) -> Option<String> {
    let mut hunks: Vec<String> = Vec::new();
    for line in file.hunks.lines() {
        if line.starts_with("@@") || hunks.is_empty() {
            hunks.push(String::new());
        }
        let hunk = hunks.last_mut()?;
        hunk.push_str(line);
        hunk.push('\n');
    }
    if hunks.len() < 2 {
        return None;
    }
    let costs: Vec<usize> = hunks.iter().map(|h| estimate_tokens(h)).collect();
    let mut order: Vec<usize> = (0..hunks.len()).collect();
    order.sort_by_key(|&i| (costs[i], i));
    let header = format!("--- {}\n", file.path);
    let marker = estimate_tokens(&format!("[... {} more hunks]\n", hunks.len()));
    let mut used = estimate_tokens(&header) + marker;
    let mut kept = vec![false; hunks.len()];
    for i in order {
        if used + costs[i] > tokens {
            break;
        }
        kept[i] = true;
        used += costs[i];
    }
    let shown = kept.iter().filter(|k| **k).count();
    if shown == 0 || shown == hunks.len() {
        return None;
    }
    // What is left of the budget shows the start of the first hunk left out,
    // with its own marker: more evidence, and never half a hunk as a whole.
    let first_left_out = kept.iter().position(|k| !k);
    let partial = first_left_out.and_then(|i| {
        cut_lines(&hunks[i], tokens.saturating_sub(used), MIN_BODY_LINES).map(|cut| (i, cut))
    });
    let mut text = header;
    for (i, hunk) in hunks.iter().enumerate() {
        match &partial {
            Some((at, cut)) if *at == i => text.push_str(cut),
            _ if kept[i] => text.push_str(hunk),
            _ => {}
        }
    }
    let omitted = hunks.len() - shown - usize::from(partial.is_some());
    if omitted > 0 {
        let _ = writeln!(text, "[... {omitted} more hunks]");
    }
    Some(text)
}

/// The file's `--- path` line and hunk headers, as opposed to its lines.
pub(super) fn is_header(line: &str) -> bool {
    line.starts_with("--- ") || line.starts_with("@@")
}

/// Keeps whole hunks of the file's section while they fit in `tokens`, the
/// smallest first and in the file's order, so a hunk at the end is not lost to
/// a large one before it and no change is cut in half; a file of one hunk, or
/// of none that fits whole, keeps whole lines while they fit, and nothing at
/// all if no line besides the headers fits.
pub(super) fn fit_file(file: &FileDiff, tokens: usize) -> (String, bool) {
    let full = file.section();
    if estimate_tokens(&full) <= tokens {
        return (full, false);
    }
    if let Some(text) = fit_hunks(file, tokens) {
        return (text, true);
    }
    (cut_lines(&full, tokens, 1).unwrap_or_default(), true)
}

/// Whole lines of `text` while they fit in `tokens`, ending with a marker for
/// the rest; `None` when fewer than `least` of them are lines besides the
/// headers.
fn cut_lines(text: &str, tokens: usize, least: usize) -> Option<String> {
    let lines: Vec<&str> = text.lines().collect();
    let mut kept = String::new();
    // Sum of per-line estimates: an upper bound of the estimate of `kept`.
    let mut used = 0;
    let mut body = 0;
    for (n, line) in lines.iter().enumerate() {
        let marker = format!("[... {} more lines]\n", lines.len() - n);
        let cost = estimate_tokens(line) + 1;
        if used + cost + estimate_tokens(&marker) > tokens {
            if body < least {
                return None;
            }
            kept.push_str(&marker);
            break;
        }
        let _ = writeln!(kept, "{line}");
        used += cost;
        body += usize::from(!is_header(line));
    }
    Some(kept)
}
