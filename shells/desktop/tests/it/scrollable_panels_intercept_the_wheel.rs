//! Arch-gate: every panel that SCROLLS also intercepts the **mouse wheel** — and nobody has to
//! remember to make it so.
//!
//! ## The bug this exists to prevent
//!
//! Until 2026-09-29 making a panel scrollable took four edits, and the fourth — naming the panel
//! in a hand-written `inside(ID) ||` chain in `cursor_over_hero_panel` — was the only one that did
//! not fail loud. Forget it and the panel still compiled, still painted its bar, the bar still
//! dragged, and the wheel silently zoomed the camera underneath. It cost the Audio Mixer, the Asset
//! Browser, the Model3D panel, the Widget Lab, the Tags panel — and the Skeleton panel was still
//! missing on the day the chain died (rolagem única, W3,
//! `docs/UI_New_and_Simple/spec/04_a_rolagem_unica.md`).
//!
//! ## What is asserted now
//!
//! 1. `cursor_over_hero_panel` **derives** from the published panel table (`panel_at`) and carries
//!    no hand list — the list coming back is the regression.
//! 2. Every panel crate that goes through the scroll door (`scroll_area::open`/`open_with`) also
//!    **publishes its rect** (`set_panel_rect`) — the rect is what `panel_at` reads, so a panel that
//!    scrolls without publishing it would scroll by bar and zoom by wheel, the old bug in a new place.
//!
//! Both halves are source scans; the behaviour (`panel_at` honouring the stacking order) is gated
//! in `ph2d-editor-core` (`the_panel_on_top_owns_the_point_where_two_panels_overlap`).

use std::fs;
use std::path::{Path, PathBuf};

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
}

fn read(path: &Path) -> String {
    fs::read_to_string(path).unwrap_or_else(|e| panic!("cannot read {}: {e}", path.display()))
}

/// Code only: `//` comments are cut, so the prose that EXPLAINS the old chain (and names
/// `inside(`) cannot satisfy or trip the scan.
fn code_of(src: &str) -> String {
    src.lines()
        .map(|l| l.split("//").next().unwrap_or(""))
        .collect::<Vec<_>>()
        .join("\n")
}

fn wheel_fn_body() -> String {
    let src = read(&repo_root().join("shells/desktop/src/forwarding.rs"));
    let start = src
        .find("pub fn cursor_over_hero_panel")
        .expect("cursor_over_hero_panel vanished — this gate is stale");
    let body = &src[start..];
    let end = body.find("\n}").expect("unterminated fn");
    code_of(&body[..end])
}

#[test]
fn the_wheel_asks_the_published_panel_table_and_keeps_no_list() {
    let body = wheel_fn_body();
    assert!(
        // ⚠️ Pela porta FÍSICA desde 2026-10-02 (`HeroScreen::chrome_panel_at`, a escala da UI).
        body.contains(".chrome_panel_at("),
        "cursor_over_hero_panel no longer asks `chrome_panel_at` — the wheel must derive from the rects \
         the panels publish, never from a list someone has to remember:\n{body}"
    );
    assert!(
        !body.contains("inside(") && !body.contains("panel_rect("),
        "cursor_over_hero_panel checks panels one by one again — that hand list is exactly what \
         left the Skeleton panel's wheel zooming the camera. Ask `panel_at` instead:\n{body}"
    );
}

/// Every `crates/ph2d-panel-*` source file tree that opens the scroll door must publish its rect.
#[test]
fn every_panel_that_scrolls_publishes_the_rect_the_wheel_reads() {
    let crates = repo_root().join("crates");
    let mut scrolling = 0usize;
    let mut missing = Vec::new();
    for entry in fs::read_dir(&crates).expect("crates/ readable") {
        let dir = entry.expect("dir entry").path();
        let name = dir.file_name().unwrap().to_string_lossy().into_owned();
        if !name.starts_with("ph2d-panel-") {
            continue;
        }
        let mut code = String::new();
        collect(&dir.join("src"), &mut code);
        let opens = code.contains("scroll_area::open(") || code.contains("scroll_area::open_with(");
        if !opens {
            continue;
        }
        scrolling += 1;
        if !code.contains("set_panel_rect(") {
            missing.push(name);
        }
    }
    // Population floor: the migration of 2026-09-29 put ~25 panel crates through the door. A scan
    // that finds almost none has broken (the door renamed, the tree moved), not passed.
    assert!(
        scrolling >= 20,
        "only {scrolling} panel crates open the scroll door — the scan broke, not the wiring"
    );
    assert!(
        missing.is_empty(),
        "these panels scroll through the door but never publish their rect, so the wheel over them \
         zooms the camera: {missing:?}. Fix: `store.set_panel_rect(<PANEL>, rect)` while visible and \
         `clear_panel_rect` when closed."
    );
}

fn collect(dir: &Path, out: &mut String) {
    let Ok(rd) = fs::read_dir(dir) else { return };
    for entry in rd {
        let path = entry.expect("dir entry").path();
        if path.is_dir() {
            collect(&path, out);
        } else if path.extension().and_then(|s| s.to_str()) == Some("rs") {
            let name = path.file_name().unwrap().to_string_lossy();
            if name.ends_with("_tests.rs") || name == "tests.rs" {
                continue;
            }
            out.push_str(&code_of(&read(&path)));
            out.push('\n');
        }
    }
}
