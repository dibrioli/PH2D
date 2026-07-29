//! **The shell publishes the scene selection into the timeline snapshot.**
//!
//! ⚠️ An arch-gate because no unit test can reach this: the fill lives inside
//! `render_loop`, which needs a window and a GPU, and the panel-side gates
//! (`expression_ui_seam::the_card_follows_the_scene_selection`) set `selected_entity` by
//! hand — so they stay green over a shell that never fills it, and the Expression card
//! would follow nothing in the product while every panel gate passed.
//!
//! Two things are asserted, and the second is the one that rots:
//!
//! * the field is filled **from the same `selected_now`** the timeline path already
//!   computes for `selection_jumps_to_keys` — not from a second query, which would be a
//!   second answer to *"who is selected?"*;
//! * it is filled **BEFORE** the snapshot is handed to the panel, because the panel reads
//!   a CLONE (`set_current_timeline(Some(self.timeline_view.clone()))`) and a fill after
//!   the clone lands in a struct nobody looks at.

const MOD_RS: &str = include_str!("../src/render_loop/mod.rs");

#[test]
fn the_selection_is_published_from_the_one_selection_the_frame_already_knows() {
    let at = MOD_RS
        .find("self.timeline_view.selected_entity =")
        .expect("the shell must fill `TimelineViewSnapshot::selected_entity`");
    let line_end = MOD_RS[at..].find(';').expect("a statement") + at;
    let stmt = &MOD_RS[at..line_end];
    assert!(
        stmt.contains("selected_now"),
        "it must come from the frame's own `selected_now`, not a second query: {stmt}"
    );
}

#[test]
fn the_selection_is_published_before_the_panel_gets_the_snapshot() {
    let fill = MOD_RS
        .find("self.timeline_view.selected_entity =")
        .expect("the fill exists");
    // The publish that hands the panel a CLONE of the snapshot.
    let publish = MOD_RS
        .find("set_current_timeline(Some(self.timeline_view.clone()))")
        .expect("the shell must hand the panel the snapshot");
    assert!(
        fill < publish,
        "the fill must precede the clone — after it, the panel reads a snapshot whose \
         selection is stale by one frame at best and never set at worst"
    );
}

/// Positive control: the scanner is reading the real file and can find the neighbouring
/// projection it models itself on. Without this, renaming `mod.rs` leaves both gates
/// vacuously green.
#[test]
fn the_scanner_reads_the_real_render_loop() {
    assert!(
        MOD_RS.contains("self.timeline_view.position_is_path ="),
        "the sibling projection (ADR-0141) must be in the file this gate scans — it is the \
         precedent the selection fill copies"
    );
}
