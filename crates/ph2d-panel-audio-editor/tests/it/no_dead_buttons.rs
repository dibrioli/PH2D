//! **Every button the panel paints must be REGISTERED — or it is dead.**
//!
//! ## The hole this closes
//!
//! `seam.rs` proves that a click *on an id* arms the right command. It does that by handing
//! `apply_panel_event` a `WidgetEvent::Click(id)` directly — which is the right way to test the
//! event mapping, and **completely blind to `populate.rs`**.
//!
//! In the real application a click does not arrive as an id. It arrives as a *position*, and the
//! `WidgetStore` — filled by `populate` — is what turns one into the other. A button that is
//! painted, hit-indexed, and mapped in `event.rs` but **missing from `populate`** is therefore
//! dead: it highlights, it looks alive, and clicking it does nothing.
//!
//! Found by mutation (2026-07-12): deleting `AEDIT_PASTE` from `populate.rs` left every existing
//! seam gate **green**. The project has a memory about this exact failure
//! (`feedback_panel_populate_register`) — and no gate for it. Now it has one.

use ph2d_a11y::NodeId;
use ph2d_editor_core::interaction::WidgetStore;
use ph2d_editor_core::panel::Panel;
use ph2d_panel_audio_editor::*;

/// Every id a user can click. Adding a button and forgetting this list is the one way to slip past
/// the gate — so the list is the checklist, and it is short on purpose.
const CLICKABLE: &[(NodeId, &str)] = &[
    // Transport.
    (AEDIT_PLAY, "Play"),
    (AEDIT_STOP, "Stop"),
    (AEDIT_LOAD, "Load"),
    // Whole-clip ops.
    (AEDIT_UNDO, "Undo"),
    (AEDIT_REDO, "Redo"),
    (AEDIT_NORMALIZE, "Normalize"),
    (AEDIT_NORM_LUFS, "Normalize LUFS"),
    (AEDIT_REVERSE, "Reverse"),
    (AEDIT_DC, "Remove DC"),
    (AEDIT_INVERT, "Invert"),
    (AEDIT_MONO, "Force Mono"),
    (AEDIT_GAIN_UP, "Gain +"),
    (AEDIT_GAIN_DOWN, "Gain -"),
    // The Edit toolbar: the three tools, then the structure.
    (AEDIT_TOOL_SELECT, "Select tool"),
    (AEDIT_TOOL_MOVE, "Move tool"),
    (AEDIT_TOOL_SCALE, "Scale tool"),
    (AEDIT_SPLIT_PLAYHEAD, "Split"),
    (AEDIT_CUTS_CLEAR, "Clear Cuts"),
    // Range ops + the clipboard (W2).
    (AEDIT_TRIM, "Trim"),
    (AEDIT_CUT, "Cut"),
    (AEDIT_COPY, "Copy"),
    (AEDIT_PASTE, "Paste"),
    (AEDIT_SILENCE, "Silence"),
    (AEDIT_FADE_IN, "Fade In"),
    (AEDIT_FADE_OUT, "Fade Out"),
    // Loop + markers.
    (AEDIT_LOOP_SET, "Set Loop"),
    (AEDIT_LOOP_CLEAR, "Clear Loop"),
    (AEDIT_LOOP_BAKE, "Crossfade Loop"),
    (AEDIT_MARK_ADD, "Add Marker"),
    (AEDIT_MARK_DEL, "Delete Marker"),
    (AEDIT_SPLIT, "Split at Markers"),
    // Rack.
    (AEDIT_FX_APPLY, "Apply Effect"),
    (AEDIT_FX_CANCEL, "Cancel Effect"),
    // Delivery.
    (AEDIT_EXPORT, "Export"),
    (AEDIT_EXPORT_PIECES, "Export Pieces"),
    (AEDIT_EXPORT_SET, "Export Set"),
    (AEDIT_BATCH_LUFS, "Batch LUFS"),
];

#[test]
fn every_clickable_button_is_registered_in_the_widget_store() {
    let mut store = WidgetStore::default();
    AudioEditorPanel::populate(&mut store);

    let mut dead: Vec<&str> = Vec::new();
    for (id, label) in CLICKABLE {
        if store.get(*id).is_none() {
            dead.push(label);
        }
    }
    assert!(
        dead.is_empty(),
        "these buttons are painted but NOT registered in `populate.rs` — in the app they highlight \
         and do nothing: {dead:?}"
    );

    // The gate must be reading something. An empty store would satisfy nothing above while proving
    // nothing at all — the failure mode of every list-based check.
    assert!(
        store.get(AEDIT_PLAY).is_some(),
        "populate registered no widgets — this gate is not testing what it thinks it is"
    );
}
