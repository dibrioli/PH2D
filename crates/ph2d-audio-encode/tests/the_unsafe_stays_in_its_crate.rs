//! **ADR-0116 §4.5, as a gate.** The whole justification for `ph2d-audio-opus` existing as a
//! separate crate is that `ph2d-audio-encode` keeps its `#![forbid(unsafe_code)]` — the `unsafe`
//! of the transpiled libopus ABI is contained one crate over, behind a safe API.
//!
//! A guarantee is worth exactly as much as it is checked. A future edit that reaches for the raw
//! ABI directly ("just here, just this once") would dissolve the reason the crate was split, and
//! it would compile the moment someone deleted a line from `lib.rs`. So: read the source, and
//! make sure the line is still there and that nobody wrote an `unsafe` block under it.

use std::fs;
use std::path::{Path, PathBuf};

#[test]
fn ph2d_audio_encode_still_forbids_unsafe() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let lib = fs::read_to_string(root.join("lib.rs")).expect("read lib.rs");
    assert!(
        lib.starts_with("#![forbid(unsafe_code)]"),
        "the `#![forbid(unsafe_code)]` is gone from ph2d-audio-encode. It is the ONLY reason \
         ph2d-audio-opus is a separate crate (ADR-0116): the unsafe libopus ABI is contained \
         there so that it is absent here. Restore it, or the split has no purpose."
    );

    // …and no `unsafe` snuck in anyway. The attribute alone would catch it — but an edit that
    // drops the attribute AND adds unsafe would sail past a check that only looked for the line.
    for file in walk(&root) {
        let src = fs::read_to_string(&file).expect("read source");
        for (i, line) in src.lines().enumerate() {
            let t = line.trim_start();
            if t.starts_with("//") {
                continue;
            }
            assert!(
                !t.contains("unsafe "),
                "{}:{}: an `unsafe` in ph2d-audio-encode. It belongs in ph2d-audio-opus, which \
                 exists precisely to hold it.",
                file.display(),
                i + 1
            );
        }
    }
}

fn walk(dir: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    let Ok(rd) = fs::read_dir(dir) else {
        return out;
    };
    for e in rd.flatten() {
        let p = e.path();
        if p.is_dir() {
            out.extend(walk(&p));
        } else if p.extension().and_then(|s| s.to_str()) == Some("rs") {
            out.push(p);
        }
    }
    out
}
