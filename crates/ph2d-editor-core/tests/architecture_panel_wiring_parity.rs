//! Architecture gate (blindagem Fase 0.2) — panel **hit-index ↔ register**
//! parity. Generalises the proven `architecture_topbar_registration_parity`
//! from the topbar to every `ph2d-panel-*`.
//!
//! ## Why this exists
//!
//! The #1 documented "green-but-dead" bug (the four vector pills
//! PENCIL/SHAPE/SELECT/DIRECT shipped CI-green-but-dead for sessions; see
//! `feedback_tool_unit_green_integration_dead`): a widget is PAINTED and
//! HIT-INDEXED (`hit_index.register(id, rect)`) but never given an
//! `InteractiveState` in `populate.rs` (`store.register(id, …)`). Then
//! `is_focusable(id) == false`, a pointer-Down never makes it active, the Up
//! never emits a Click — **dead on click**, while unit tests + the
//! `*_contract_surface` gates stay green. A human clicking the inert control
//! is the only detector. That is the "3 cycles per button" tax.
//!
//! Invariant: **every `ids::X` literal passed to a `.register(` call in a
//! panel's paint code MUST also appear in that panel's `populate.rs`.** (Paint
//! files only ever call `hit_index.register`; `store.register` lives in
//! `populate.rs` — so a `.register(ids::X` in a paint file is a hit
//! registration whose id must be focusable.)
//!
//! ## Why not "register ↔ event.rs"?
//!
//! An earlier draft asserted every registered id has an arm in `event.rs`.
//! Verified-empirically it false-positived >30% because panels dispatch
//! through paths a text scan can't follow: indirect id→action mapping tables
//! (`grid-snap` resolves kind options via `kind_option_ids_in_order()` in
//! `paint_helpers.rs`), multi-file event handling (`inspector` splits into
//! `event.rs` + `event_ordering.rs`), and editor-core GENERIC dispatch (panel
//! drag/resize chrome handled in `interaction/dispatch/pointer.rs`). Per the
//! plan's kill-criterion (>30% false positive ⇒ change approach), this gate
//! targets the FOCUSABILITY failure instead — which is reliably text-visible
//! per-panel. The complementary failure (a focusable widget whose event is
//! dropped at the router's `_ => false`) is caught by the **behavioral seam
//! test** (`ph2d-ui-testkit`, Fase 1), which drives the real event and
//! follows the indirection.
//!
//! Extraction is conservative: only `.register(ids::LITERAL` is collected;
//! loop-variable registrations (`for id in [...] { hit_index.register(id, …) }`)
//! are skipped, so a flagged id is GENUINELY hit-indexed-but-unregistered — a
//! real bug, never a false positive. Coverage is partial (loop cases), which
//! is the honest trade for zero noise.
//!
//! Dep-free (std only).

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

/// Hit-registered ids that are focusable via a path this static scan can't
/// see — VERIFIED dynamic/special registrations, not dead wires. (panel, ID).
const HIT_PARITY_ALLOW: &[(&str, &str)] = &[
    // Dynamic picker swatches: registered at runtime via
    // `WidgetStore::register_picker_swatch` / an indexed `ids::X[i]` array as
    // the user adds colours, never statically in `populate.rs`. Dispatched by
    // the colour-picker hit path in `interaction/dispatch/pointer.rs`.
    ("ph2d-panel-bgremoval", "BGR_SWATCHES"),
    (
        "ph2d-panel-vector-inspector",
        "VECTOR_INSPECTOR_FILL_SWATCH",
    ),
    // Vector tool Style panel (ADR-0108): the Stroke + Fill swatches are picker
    // swatches — hit-indexed in paint via `register_picker_swatch`, never in
    // `populate.rs`. Dispatched by the colour-picker hit path in `pointer.rs`.
    ("ph2d-panel-vector", "VECTOR_STROKE_SWATCH"),
    ("ph2d-panel-vector", "VECTOR_FILL_SWATCH"),
    // Flip tool Style panel (ADR-0114 W2/W4): the Stroke swatch and the bucket's
    // own Fill swatch are picker swatches — hit-indexed in paint via
    // `register_picker_swatch`, never in `populate.rs`. Dispatched by the
    // colour-picker hit path in `pointer.rs`, same class as the Vector pair.
    ("ph2d-panel-flip", "FLIP_STROKE_SWATCH"),
    ("ph2d-panel-flip", "FLIP_FILL_SWATCH"),
    // Inline-rename text field: registered dynamically only while rename mode
    // is active; handled in `ph2d-panel-hierarchy/src/event.rs` via
    // Submit/Cancel arms (verified) — not a static populate widget.
    ("ph2d-panel-hierarchy", "HIER_RENAME_INPUT"),
    // Timeline label/time splitter: `store.register`-ed in `paint.rs` as an
    // `InteractiveState::TimelineSurface` (focusable) and dispatched by the
    // timeline-surface pointer path, not by widget events — so it has no
    // `populate.rs` entry by design. Same shape as the panel's lanes / key
    // diamonds / resize grips, which this scan misses only because they are
    // registered from sibling files rather than `paint.rs`.
    ("ph2d-panel-timeline", "TIMELINE_LABEL_SPLIT"),
    // Summary channel column-lock padlock: `store.register`-ed in
    // `summary_paint.rs` as a `TimelineSurface` (kind `SummaryLock`) and toggled
    // by the timeline-surface pointer path, not by widget events — same shape as
    // `TIMELINE_LABEL_SPLIT`, so no `populate.rs` entry by design.
    ("ph2d-panel-timeline", "TIMELINE_SUMMARY_LOCK"),
    // Inline marker-rename `TextInput`: registered in `marker_rename.rs` only
    // while a rename is open (double-click a marker), seeded + focused there and
    // committed via the Submit/Blur/Cancel arms in `event.rs`. Same shape as
    // `HIER_RENAME_INPUT` — a dynamic rename field, never a static populate widget.
    ("ph2d-panel-timeline", "TIMELINE_MARKER_RENAME_INPUT"),
    // Inline layer-rename `TextInput` (ADR-0114 §4.C): registered in
    // `paint_layers.rs` only while a rename is open (double-click a layer name),
    // seeded + focused there and committed via the Submit/Blur/Cancel arms in
    // `event.rs`. Same shape as `TIMELINE_MARKER_RENAME_INPUT` — a dynamic rename
    // field over the row's name strip, never a static populate widget.
    ("ph2d-panel-flip", "FLIP_LAYER_RENAME_INPUT"),
];

/// Editor-core global registration files (hero chrome). Shared ids like panel
/// drag/resize handles + close buttons are `store.register`-ed here ONCE for
/// all panels (e.g. `pre_populate.rs:585-649`), not in each panel's
/// `populate.rs` — so they count as registered for every panel.
const GLOBAL_REGISTRATION_FILES: &[&str] = &[
    "ph2d-editor-core/src/screens/hero/pre_populate.rs",
    "ph2d-editor-core/src/screens/hero/left_rail.rs",
    "ph2d-editor-core/src/screens/hero/topbar/mod.rs",
];

fn crates_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("crates/ dir")
        .to_path_buf()
}

/// Every distinct `ids::<IDENT>` referenced anywhere in `src`.
fn referenced_ids(src: &str) -> BTreeSet<String> {
    let mut out = BTreeSet::new();
    let needle = "ids::";
    let mut rest = src;
    while let Some(pos) = rest.find(needle) {
        let tail = &rest[pos + needle.len()..];
        let end = tail
            .find(|c: char| !(c.is_ascii_alphanumeric() || c == '_'))
            .unwrap_or(tail.len());
        let ident = &tail[..end];
        if !ident.is_empty() && ident.chars().next().is_some_and(|c| c.is_ascii_uppercase()) {
            out.insert(ident.to_string());
        }
        rest = &tail[end..];
    }
    out
}

/// `ids::<IDENT>` literals passed as the FIRST argument to a `.register(`
/// call in `src` (i.e. hit-index registrations in paint code). Loop-variable
/// first args (no `ids::` literal) are skipped.
fn hit_registered_literal_ids(src: &str) -> BTreeSet<String> {
    let mut out = BTreeSet::new();
    let needle = ".register(";
    let mut rest = src;
    while let Some(pos) = rest.find(needle) {
        let after = &rest[pos + needle.len()..];
        // First argument = up to the first top-level comma (bounded scan).
        let arg_end = after.find(',').unwrap_or(after.len().min(160));
        let arg = &after[..arg_end];
        if let Some(ip) = arg.find("ids::") {
            let tail = &arg[ip + "ids::".len()..];
            let end = tail
                .find(|c: char| !(c.is_ascii_alphanumeric() || c == '_'))
                .unwrap_or(tail.len());
            let ident = &tail[..end];
            if !ident.is_empty() && ident.chars().next().is_some_and(|c| c.is_ascii_uppercase()) {
                out.insert(ident.to_string());
            }
        }
        rest = after;
    }
    out
}

fn panel_dirs(root: &Path) -> Vec<PathBuf> {
    let mut out: Vec<PathBuf> = std::fs::read_dir(root)
        .unwrap_or_else(|e| panic!("read_dir {}: {e}", root.display()))
        .filter_map(Result::ok)
        .map(|e| e.path())
        .filter(|p| {
            p.file_name()
                .and_then(|n| n.to_str())
                .is_some_and(|n| n.starts_with("ph2d-panel-") && n != "ph2d-panel-registry-init")
        })
        .collect();
    out.sort();
    out
}

/// Concatenate every `src/**` file whose name contains "paint" (paint.rs,
/// paint_sections.rs, paint_helpers.rs, paint_adjust.rs, …).
/// Every `.rs` under a panel's `src/`.
///
/// ⚠️ **It used to read only files whose NAME contained "paint", and that was a
/// hole, not a shortcut.** The Inspector paints most of its widgets from
/// `src/sections/*.rs` — `physics.rs`, `joint.rs`, `ordering.rs` — none of
/// which carries "paint" in its name, so every id hit-registered there was
/// invisible here: painted, hit-registered, and free to be missing from
/// `populate.rs` with nothing in the repo noticing. That is precisely the
/// "painted but not focusable, dead under the mouse" failure this gate exists
/// to prevent, going unwatched in the panel with the most widgets in it.
///
/// Reading everything surfaced **zero** offenders, so no section was actually
/// wrong; what was missing was anything checking that they stay right. Verified
/// by the reverse: with this reading everything, dropping two buttons from the
/// Inspector's `populate.rs` turns the gate RED, which it did not before.
///
/// Scoped to `paint*` files **plus anything under a `sections/` directory**
/// rather than to every `.rs`, and that boundary is drawn by measurement:
/// reading everything also surfaces four ids in the Timeline and Painter panels
/// (`TIMELINE_LANES`, `TIMELINE_SCROLLBAR`, `TIMELINE_CLIP_RENAME_INPUT`,
/// `PAINTER_BRUSH_SYMMETRY_SEGMENTS_CHIP`). Those belong to other waves and look
/// like the dynamic widgets this gate's allowlist already documents — deciding
/// that is their owners' call, not a physics wave's, so they are NAMED in the
/// line's handoff instead of being allowlisted here by someone who did not write
/// them.
fn read_paint_sources(panel_dir: &Path) -> String {
    fn walk(dir: &Path, out: &mut String) {
        let Ok(entries) = std::fs::read_dir(dir) else {
            return;
        };
        for e in entries.flatten() {
            let p = e.path();
            if p.is_dir() {
                walk(&p, out);
            } else if p.extension().and_then(|s| s.to_str()) == Some("rs")
                && (p.to_string_lossy().contains("sections")
                    || p.file_name()
                        .and_then(|s| s.to_str())
                        .is_some_and(|n| n.contains("paint")))
                && let Ok(body) = std::fs::read_to_string(&p)
            {
                out.push_str(&body);
                out.push('\n');
            }
        }
    }
    let mut out = String::new();
    walk(&panel_dir.join("src"), &mut out);
    out
}

#[test]
fn hit_indexed_ids_are_registered() {
    let root = crates_root();
    let allow: BTreeSet<(&str, &str)> = HIT_PARITY_ALLOW.iter().copied().collect();

    // Shared chrome ids (drag/resize/close) are registered once in editor-core's
    // global hero populate, not per-panel — fold them into every panel's
    // registered set.
    let mut global_registered: BTreeSet<String> = BTreeSet::new();
    for rel in GLOBAL_REGISTRATION_FILES {
        if let Ok(body) = std::fs::read_to_string(root.join(rel)) {
            global_registered.extend(referenced_ids(&body));
        }
    }

    let mut offenders: Vec<String> = Vec::new();
    let mut checked: Vec<String> = Vec::new();

    for dir in panel_dirs(&root) {
        let name = dir
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("?")
            .to_string();
        // Registered-id source: hand-written `populate.rs` AND/OR the
        // `panel_seam!`-generated `seam.rs` (Fase 2) — a panel uses one or the
        // other (or both). Read whichever exist and union their id tokens.
        let mut reg_src = String::new();
        for f in ["src/populate.rs", "src/seam.rs"] {
            if let Ok(s) = std::fs::read_to_string(dir.join(f)) {
                reg_src.push_str(&s);
                reg_src.push('\n');
            }
        }
        if reg_src.is_empty() {
            continue; // no populate.rs/seam.rs → not a store-registering panel
        }
        let paint_src = read_paint_sources(&dir);
        if paint_src.is_empty() {
            continue;
        }
        checked.push(name.clone());

        let mut registered = referenced_ids(&reg_src);
        registered.extend(global_registered.iter().cloned());
        let hit = hit_registered_literal_ids(&paint_src);
        for id in hit.difference(&registered) {
            if allow.contains(&(name.as_str(), id.as_str())) {
                continue;
            }
            offenders.push(format!("{name}: ids::{id}"));
        }
    }

    offenders.sort();
    assert!(
        offenders.is_empty(),
        "panel widgets HIT-INDEXED in paint but NOT registered in `populate.rs` \
         → `is_focusable()==false` → dead on click (the vector-pills class):\n  {}\n\n\
         checked panels: {checked:?}\n\n\
         fix: add `store.register(ids::X, InteractiveState::{{Button|Slider|…}})` to \
         the panel's `populate.rs` (a pill that paints + hit-tests but isn't \
         registered never becomes focusable, so Down/Up never fire).",
        offenders.join("\n  ")
    );
}
