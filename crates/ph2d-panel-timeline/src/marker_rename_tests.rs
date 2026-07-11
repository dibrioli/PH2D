//! Unit tests for [`super`] (`marker_rename.rs`) — commit / cancel of the inline
//! marker-rename field. Extracted to a sibling module (`#[path]`) so the source
//! stays under the 600-LOC panel cap.

use super::*;
use crate::state::MarkerRename;

/// A store whose rename field holds `text`.
fn store_with(text: &str) -> WidgetStore {
    let mut store = WidgetStore::with_capacity(0);
    store.register(
        ids::TIMELINE_MARKER_RENAME_INPUT,
        InteractiveState::TextInput {
            state: TextInputState::Focused,
            text: text.to_string(),
            caret: 0,
            selection_anchor: None,
        },
    );
    store
}

fn renaming(index: usize) -> TimelinePanelState {
    TimelinePanelState {
        marker_rename: Some(MarkerRename {
            index,
            opened: true,
        }),
        ..TimelinePanelState::default()
    }
}

#[test]
fn commit_pushes_a_rename_with_the_trimmed_buffer_and_closes() {
    let _ = state::drain_intents();
    let store = store_with("  Chorus  ");
    let mut st = renaming(2);
    commit(&mut st, &store);
    assert_eq!(
        state::drain_intents(),
        vec![TimelineIntent::RenameMarker {
            index: 2,
            label: "Chorus".to_string(),
        }],
        "commit trims and renames the armed marker index"
    );
    assert!(st.marker_rename.is_none(), "the field closes after commit");
}

#[test]
fn commit_with_a_blank_buffer_keeps_the_old_name() {
    let _ = state::drain_intents();
    let store = store_with("   ");
    let mut st = renaming(0);
    commit(&mut st, &store);
    assert!(
        state::drain_intents().is_empty(),
        "an empty/whitespace label raises no rename"
    );
    assert!(st.marker_rename.is_none(), "but the field still closes");
}

#[test]
fn commit_is_idempotent_across_the_submit_then_blur_pair() {
    let _ = state::drain_intents();
    let store = store_with("Bridge");
    let mut st = renaming(1);
    commit(&mut st, &store); // Enter → Submit
    let first = state::drain_intents();
    commit(&mut st, &store); // …then the trailing Blur
    assert_eq!(first.len(), 1, "the Submit renamed once");
    assert!(
        state::drain_intents().is_empty(),
        "the take() guard makes the trailing Blur a no-op"
    );
}

#[test]
fn cancel_drops_the_field_without_renaming() {
    let _ = state::drain_intents();
    let mut st = renaming(3);
    cancel(&mut st);
    assert!(st.marker_rename.is_none());
    assert!(
        state::drain_intents().is_empty(),
        "Esc abandons the edit — no rename intent"
    );
}
