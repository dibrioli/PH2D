//! Arch gate: `paint_icon_path` — the manifest-`BezPath` glyph draw — may
//! only be called from the canonical icon-button painter
//! (`widget/icon_button.rs`) and its own definition in `paint.rs` (where
//! `paint_icon` calls it internally). Every manifest/SVG icon button must
//! render through the single source of truth `paint_icon_button`, never
//! through hand-rolled `fill_rounded_rect` + `stroke` + `paint_icon_path`
//! chrome (the TopBar Image Tools pills used to). A new chrome surface
//! that reaches for `paint_icon_path` directly fails this test in CI.

use std::path::{Path, PathBuf};

#[test]
fn paint_icon_path_only_in_canonical_icon_button() {
    let src = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    // Exact relative paths allowed to call `paint_icon_path`:
    //  - `paint.rs`            — defines it + `paint_icon` calls it.
    //  - `widget/icon_button.rs` — the canonical painter.
    let allowed = [
        PathBuf::from("paint.rs"),
        PathBuf::from("widget").join("icon_button.rs"),
    ];
    let mut offenders = Vec::new();
    visit(&src, &mut |path, contents| {
        let Ok(rel) = path.strip_prefix(&src) else {
            return;
        };
        if allowed.iter().any(|a| a == rel) {
            return;
        }
        for (i, line) in contents.lines().enumerate() {
            if line.trim_start().starts_with("//") {
                continue;
            }
            if line.contains("paint_icon_path(") {
                offenders.push(format!("{}:{}: {}", rel.display(), i + 1, line.trim()));
            }
        }
    });
    assert!(
        offenders.is_empty(),
        "`paint_icon_path` must only be called from the canonical icon \
         button (widget/icon_button.rs). Route manifest/SVG icon buttons \
         through `paint_icon_button(.., IconGlyph::Path(..), ..)`. \
         Offending call sites:\n{}",
        offenders.join("\n"),
    );
}

fn visit(dir: &Path, f: &mut impl FnMut(&Path, &str)) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            visit(&path, f);
        } else if path.extension().is_some_and(|e| e == "rs")
            && let Ok(contents) = std::fs::read_to_string(&path)
        {
            f(&path, &contents);
        }
    }
}
