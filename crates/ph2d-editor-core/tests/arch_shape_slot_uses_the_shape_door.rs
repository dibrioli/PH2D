//! **The Shape slot must go through the Shape door** (`texture::shape_basis`, never `texture::dab_basis`).
//!
//! The Shape's **Flow** mapping lays its pattern in the STROKE's frame, which needs two per-dab facts the
//! Grain never wants: the dab's `arc_len` and the stroke's nominal radius (`ShapeFrame`). They used to
//! ride in through an optional builder (`with_arc_len`), and **five** Shape routes — the relief pass,
//! sculpt, smear, watercolor and blur/clone — simply never called it. The failure is silent and total:
//! `arc_len` stays `0` on every dab, so Flow degrades into exactly the per-stamp phase reset it exists to
//! remove, the pattern breaks up on curves, and the Follow dropdown still reads "Flow".
//!
//! Making the frame a *parameter of a dedicated door* means a route cannot forget it — but only if new
//! Shape routes use that door. That is what this gate pins. Enumerating the call sites by hand is the
//! shape of bug this repo keeps re-finding ([[feedback_a_condition_that_enumerates_its_readers_rots]]);
//! a gate over the source is the version that does not rot.
//!
//! Allowlist: a trailing `// SHAPE-DOOR-OK: <reason>` on the call line.

use std::fs;
use std::path::{Path, PathBuf};

/// Everything between `dab_basis(` and the first top-level `,` — the settings argument.
fn first_arg(src: &str, open: usize) -> String {
    let mut depth = 0usize;
    let mut out = String::new();
    for ch in src[open..].chars() {
        match ch {
            '(' | '[' | '{' => depth += 1,
            ')' | ']' | '}' => {
                if depth == 0 {
                    break;
                }
                depth -= 1;
            }
            ',' if depth == 0 => break,
            _ => {}
        }
        out.push(ch);
        if out.len() > 400 {
            break;
        }
    }
    out
}

fn scan(dir: &Path, hits: &mut Vec<String>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for e in entries.flatten() {
        let p = e.path();
        if p.is_dir() {
            if p.file_name().is_some_and(|n| n == "target") {
                continue;
            }
            scan(&p, hits);
        } else if p.extension().is_some_and(|x| x == "rs") {
            // This gate's own file carries the offence as a string literal (the positive control).
            if p.file_name()
                .is_some_and(|n| n == "arch_shape_slot_uses_the_shape_door.rs")
            {
                continue;
            }
            let Ok(src) = fs::read_to_string(&p) else {
                continue;
            };
            let mut at = 0usize;
            while let Some(rel) = src[at..].find("dab_basis(") {
                let idx = at + rel;
                at = idx + "dab_basis(".len();
                // The definition itself (`fn dab_basis(`) — the token must be IMMEDIATELY preceded by
                // `fn`, not merely near it: `let a = dab_basis(..)` inside `fn f()` has "fn " in its
                // preceding window, which made the first version of this scanner report nothing at all.
                if src[..idx].trim_end().ends_with("fn") {
                    continue;
                }
                let arg = first_arg(&src, at);
                if !arg.contains(".shape") {
                    continue;
                }
                let line_no = src[..idx].matches('\n').count() + 1;
                let line = src.lines().nth(line_no - 1).unwrap_or("");
                if line.contains("SHAPE-DOOR-OK") {
                    continue;
                }
                hits.push(format!(
                    "{}:{line_no}: the Shape slot must use `texture::shape_basis` (it takes the \
                     `ShapeFrame` that Flow needs), not `dab_basis` — arg was `{}`",
                    p.display(),
                    arg.trim()
                ));
            }
        }
    }
}

#[test]
fn the_shape_slot_goes_through_the_shape_door() {
    let ws = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("workspace root")
        .to_path_buf();
    let mut hits = Vec::new();
    for sub in ["crates", "shells"] {
        scan(&ws.join(sub), &mut hits);
    }
    assert!(
        hits.is_empty(),
        "Shape slots bypassing the Shape door:\n{}",
        hits.join("\n")
    );
}

/// A **positive control** for the search above: a negative grep that finds nothing proves nothing unless
/// it can be shown to find something ([[feedback_a_negative_search_needs_a_positive_control]]). This feeds
/// the scanner a synthetic file containing the exact offence and requires it to be reported — and requires
/// a Grain call in the same file NOT to be.
#[test]
fn the_scanner_finds_a_planted_offence_and_spares_the_grain() {
    let dir = std::env::temp_dir().join("ph2d_shape_door_gate_probe");
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).expect("probe dir");
    fs::write(
        dir.join("planted.rs"),
        "fn f() {\n    let a = dab_basis(&spec.shape, rng, dims, fp);\n\
         \x20   let b = dab_basis(&spec.texture, rng, dims, fp);\n\
         \x20   let c = dab_basis(&spec.shape, rng, dims, fp); // SHAPE-DOOR-OK: allowlisted\n}\n",
    )
    .expect("write probe");
    let mut hits = Vec::new();
    scan(&dir, &mut hits);
    let _ = fs::remove_dir_all(&dir);
    assert_eq!(
        hits.len(),
        1,
        "the scanner must flag the planted Shape call, spare the Grain one and honour the \
         allowlist; got:\n{}",
        hits.join("\n")
    );
    assert!(hits[0].contains("planted.rs:2"), "wrong line: {}", hits[0]);
}
