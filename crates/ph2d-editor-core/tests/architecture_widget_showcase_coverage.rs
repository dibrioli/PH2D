//! Wave 9 Eixo B.2 — every primitive widget under `src/widget/` must
//! appear in the Widget Gallery showcase (or carry an explicit opt-out).
//!
//! The showcase is the canonical visual reference that peripheral
//! agents copy section layout + state-machine wiring from. A widget
//! that exists but isn't shown there is invisible to humans browsing
//! the editor AND to agents discovering primitives via the codebase.
//!
//! Heuristic: a widget file `src/widget/<slug>.rs` is "shown" iff
//! some file under `src/widget/showcase/` references it via a type
//! import path (`Button`, `Checkbox`, etc., as parsed from the
//! `pub use` re-export in `src/widget/mod.rs`).
//!
//! Failure modes this catches:
//! - New widget added without showcase entry.
//! - Showcase section deleted that previously covered a widget.
//!
//! Both are review gates — to fix, either add a showcase section or
//! add an explicit `WIDGET_OPT_OUT` entry with a written reason.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

/// Widgets that legitimately don't appear in the Widget Gallery.
/// Each entry: (file slug under `widget/`, one-line reason).
const WIDGET_OPT_OUT: &[(&str, &str)] = &[
    (
        "blender_color_picker",
        "chrome-internal: full picker is painted out-of-band atop every panel",
    ),
    (
        "color_picker",
        "chrome-internal: legacy picker, replaced by blender_color_picker for the live UI",
    ),
    (
        "context_menu",
        "chrome-internal: floating overlay invoked via right-click everywhere",
    ),
    (
        "divider",
        "primitive too small to merit a dedicated showcase section; used inline by other widgets",
    ),
    (
        "modal",
        "chrome-internal: app-wide overlay invoked from menus, no idle visual",
    ),
    (
        "panel_chrome",
        "primitive: layout helpers for every floating panel, not a standalone widget",
    ),
    (
        "pill_group",
        "compound: covered by the topbar Image Tools pill cluster on every paint",
    ),
    (
        "popover",
        "chrome-internal: anchored popover invoked from scenes / settings menus",
    ),
    (
        "scrollbar",
        "primitive: painted by every scrollable panel automatically, no idle visual",
    ),
    (
        "slider_with_chip",
        "compound: superset of slider; showcase covers the underlying slider primitive",
    ),
    (
        "status_bar",
        "chrome-internal: bottom HUD primitive painted by paint_bottom_hud",
    ),
    (
        "tool_rail",
        "chrome-internal: left rail painter; covered visually by the running editor",
    ),
    (
        "tooltip",
        "chrome-internal: hover overlay rendered on top of every widget",
    ),
];

fn editor_core_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn widget_dir() -> PathBuf {
    editor_core_root().join("src/widget")
}

fn list_widget_files() -> Vec<String> {
    let mut out = Vec::new();
    for entry in std::fs::read_dir(widget_dir()).expect("widget/ dir readable") {
        let entry = entry.expect("dir entry");
        let path = entry.path();
        if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("rs") {
            let stem = path.file_stem().unwrap().to_string_lossy().into_owned();
            if stem == "mod" {
                continue;
            }
            out.push(stem);
        } else if path.is_dir() {
            // A sub-directory counts as a widget when it contains a
            // `mod.rs` — e.g. `blender_color_picker/mod.rs`. The
            // `showcase/` directory is also a subdir but isn't a
            // widget primitive; skip it explicitly.
            let dirname = path.file_name().unwrap().to_string_lossy().into_owned();
            if dirname == "showcase" {
                continue;
            }
            if path.join("mod.rs").is_file() {
                out.push(dirname);
            }
        }
    }
    out.sort();
    out
}

fn read_all_showcase_text() -> String {
    fn walk(dir: &Path, out: &mut String) {
        for entry in std::fs::read_dir(dir).expect("showcase dir readable") {
            let entry = entry.expect("dir entry");
            let path = entry.path();
            if path.is_dir() {
                walk(&path, out);
            } else if path.extension().and_then(|s| s.to_str()) == Some("rs") {
                out.push_str(&std::fs::read_to_string(&path).expect("showcase file readable"));
                out.push('\n');
            }
        }
    }
    let mut blob = String::new();
    walk(&widget_dir().join("showcase"), &mut blob);
    blob
}

/// Map a widget file slug to the type identifiers exported by it.
/// Reads `src/widget/mod.rs` and gathers `pub use <slug>::{...}`
/// entries. Falls back to the CamelCase form of the slug.
fn slug_to_exported_idents(slug: &str) -> Vec<String> {
    let mod_rs = std::fs::read_to_string(widget_dir().join("mod.rs")).expect("widget/mod.rs");
    let needle = format!("pub use {slug}::{{");
    if let Some(start) = mod_rs.find(&needle) {
        let after = &mod_rs[start + needle.len()..];
        if let Some(end) = after.find('}') {
            return after[..end]
                .split(',')
                .map(|s| s.trim().trim_start_matches("paint_").to_string())
                .filter(|s| !s.is_empty() && s.chars().next().is_some_and(|c| c.is_uppercase()))
                .collect();
        }
    }
    let needle2 = format!("pub use {slug}::");
    if let Some(start) = mod_rs.find(&needle2) {
        let after = &mod_rs[start + needle2.len()..];
        if let Some(end) = after.find(';') {
            let item = after[..end].trim();
            if item.chars().next().is_some_and(|c| c.is_uppercase()) {
                return vec![item.to_string()];
            }
        }
    }
    // Fallback — CamelCase of slug.
    vec![
        slug.split('_')
            .map(|w| {
                let mut c = w.chars();
                match c.next() {
                    Some(first) => first.to_uppercase().chain(c).collect(),
                    None => String::new(),
                }
            })
            .collect::<String>(),
    ]
}

#[test]
fn every_widget_is_shown_or_explicitly_opted_out() {
    let widgets = list_widget_files();
    let opt_out: BTreeSet<&str> = WIDGET_OPT_OUT.iter().map(|(s, _)| *s).collect();
    let showcase_blob = read_all_showcase_text();
    let mut missing: Vec<String> = Vec::new();
    for widget in &widgets {
        if opt_out.contains(widget.as_str()) {
            continue;
        }
        let idents = slug_to_exported_idents(widget);
        let referenced = idents.iter().any(|ident| showcase_blob.contains(ident));
        if !referenced {
            missing.push(format!("{widget} (idents tried: {idents:?})"));
        }
    }
    assert!(
        missing.is_empty(),
        "the following widgets are not referenced anywhere under \
         src/widget/showcase/ and have no WIDGET_OPT_OUT entry:\n  {}\n\
         fix: either add a showcase section that uses the widget, or \
         add a WIDGET_OPT_OUT entry with a written reason.",
        missing.join("\n  ")
    );
}

#[test]
fn opt_out_entries_match_real_files() {
    let widgets: BTreeSet<String> = list_widget_files().into_iter().collect();
    let mut stale: Vec<&str> = Vec::new();
    for (slug, _) in WIDGET_OPT_OUT {
        if !widgets.contains(*slug) {
            stale.push(slug);
        }
    }
    assert!(
        stale.is_empty(),
        "WIDGET_OPT_OUT entries reference non-existent widget files: {stale:?}. \
         fix: remove the stale entries."
    );
}
