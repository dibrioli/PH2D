//! Regression guard for the 2026-05-27 bug "duplicating a sprite
//! highlights both rows on a single-entity selection".
//!
//! Root cause: the Hierarchy painter's old fallback
//! `entity.selected = entity.name == selection_label` flagged every
//! homonym of the primary selection as selected. The shell already
//! pre-marks `HierarchyEntity.selected` from the gizmo selection set
//! (which is keyed by `entity_bits`, NOT by name), so the fallback was
//! redundant for live mode and actively wrong for any scene with
//! same-named entities (e.g. duplicates pre-uniqueness, multi-import
//! of the same asset, hand-renames the user happened to type).
//!
//! Defense in depth lives elsewhere:
//! - `shells/desktop/src/name_unique.rs` enforces uniqueness at
//!   creation time (duplicate, add-child, import, rename) so the
//!   panel never sees a collision in the first place.
//! - `shells/desktop/src/render_loop/snapshots.rs::publish` pre-marks
//!   `entity.selected` from `gizmo.iter_selected()` (identity-driven).
//!
//! This test traps the regression at its narrowest point: a grep on
//! `paint.rs`. Cheap to run, impossible to circumvent by accident.

const PAINT_SRC: &str = include_str!("../src/paint.rs");

#[test]
fn paint_does_not_compare_entry_name_to_selection_label() {
    // The exact line we removed — never let it come back without
    // updating this assertion AND adding a real-paint regression test.
    assert!(
        !PAINT_SRC.contains("entity.name == sel_label"),
        "Hierarchy paint regressed: selection display compares name \
         to selection_label again. This re-introduces the duplicate-\
         row bug (2026-05-27). Use entity_bits via the bridge instead."
    );
    // Broader pattern catch — any `entity.name == ...` involving the
    // selection should also fail this test.
    let suspicious = PAINT_SRC
        .lines()
        .filter(|l| l.contains("entity.name") && l.contains("=="))
        .filter(|l| !l.trim_start().starts_with("//"))
        .count();
    assert_eq!(
        suspicious, 0,
        "Hierarchy paint contains a new `entity.name == ...` comparison; \
         selection display must remain identity-driven."
    );
}

#[test]
fn paint_does_not_take_selection_label_param() {
    // The old fallback's only consumer was `selection_label`. With the
    // fallback gone, the param itself is gone — guard against silent
    // re-introduction (which would carry the temptation to wire a new
    // label-keyed comparison).
    assert!(
        !PAINT_SRC.contains("selection_label: Option<&str>"),
        "paint_hierarchy_body re-grew a `selection_label` param. The \
         hierarchy display is identity-driven; if you need the label \
         for header/title sync, do it in the shell's snapshots::publish \
         where the bridge already resolves the primary entity."
    );
}
