//! Locks the full parse of every corpus name, field by field.
//!
//! The other corpus tests only assert that a name does not panic and yields some
//! title, which a grammar change can satisfy while silently reclassifying half the
//! corpus. This one fails on any change at all, so widening the grammar means
//! reading the diff and accepting it deliberately:
//!
//! ```text
//! UPDATE_SNAPSHOT=1 cargo test --test snapshot
//! git diff tests/snapshot.expected.txt
//! ```

use std::fmt::Write as _;
use std::panic::{catch_unwind, AssertUnwindSafe};

const CORPORA: &[(&str, &str)] = &[
    ("anime", include_str!("anime.txt")),
    ("movies", include_str!("movies.txt")),
    ("nyaa", include_str!("nyaa.txt")),
];

const EXPECTED: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/snapshot.expected.txt");

fn render() -> String {
    let hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(|_| {}));
    let mut out = String::new();
    for (label, body) in CORPORA {
        for line in body.lines().filter(|l| !l.trim().is_empty()) {
            let rendered = match catch_unwind(AssertUnwindSafe(|| sacha::parse(line))) {
                Ok(Ok(result)) => serde_json::to_string(&result).expect("ParseResult serializes"),
                Ok(Err(error)) => format!("ERROR {error}"),
                Err(_) => "PANIC".to_string(),
            };
            writeln!(out, "{label}\t{line}\t{rendered}").expect("string write");
        }
    }
    std::panic::set_hook(hook);
    out
}

#[test]
fn corpus_parse_is_unchanged() {
    let actual = render();

    if std::env::var_os("UPDATE_SNAPSHOT").is_some() {
        std::fs::write(EXPECTED, &actual).expect("write snapshot");
        return;
    }

    let expected = std::fs::read_to_string(EXPECTED).unwrap_or_default();
    if expected == actual {
        return;
    }

    let expected_lines: Vec<&str> = expected.lines().collect();
    let actual_lines: Vec<&str> = actual.lines().collect();
    let mut changed = Vec::new();
    for index in 0..expected_lines.len().max(actual_lines.len()) {
        let before = expected_lines.get(index).copied().unwrap_or("<missing>");
        let after = actual_lines.get(index).copied().unwrap_or("<missing>");
        if before != after {
            changed.push(format!("  line {}\n    before: {before}\n    after:  {after}", index + 1));
        }
    }

    // the whole corpus in the failure output is unreadable, and the first few are enough to tell an intended widening from a regression
    let shown = changed.iter().take(15).cloned().collect::<Vec<_>>().join("\n");
    panic!(
        "{} of {} corpus parses changed:\n{shown}\n\nIf every change is intended: UPDATE_SNAPSHOT=1 cargo test --test snapshot",
        changed.len(),
        expected_lines.len().max(actual_lines.len()),
    );
}
