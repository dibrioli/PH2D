//! **Focus selects all: typing into a numeric chip REPLACES its readout.**
//!
//! The 2026-07-23 Dur(s) bug: clicking the chip focused it and seeded the
//! buffer with the formatted value ("2") with a COLLAPSED selection, so typing
//! "2" appended — parse 22 — and the panel authored a 22 s duration whose veil
//! sat off-screen and whose clamp pinned nothing. The keyboard-commit tests'
//! `Backspace × 5 to clear the buffer, then type` dance was the same trap,
//! confessed in-repo. The fix is the Blender/AE number-field model: the click
//! that FOCUSES selects all (typing replaces), a second click places the caret
//! (surgical edits keep working).
//!
//! These gates drive the REAL gesture — `dispatch_pointer` on an unfocused
//! chip's rect, then `dispatch_text_input`, then Enter — not a store built
//! pre-focused, which is exactly the fixture shape that kept every prior gate
//! green over the bug.

use bumpalo::Bump;
use ph2d_a11y::NodeId;
use ph2d_editor_core::interaction::dispatch::keymap::KEY_ENTER;
use ph2d_editor_core::interaction::{
    HitIndex, InteractiveState, WidgetStore, dispatch_key, dispatch_pointer, dispatch_text_input,
};
use ph2d_editor_core::widget::TextInputState;
use ph2d_editor_core::zones::Rect;
use ph2d_host::{KeyEvent, KeyKind, Modifiers, PointerEvent, PointerKind, PointerSource};

fn pointer(kind: PointerKind, x: f32, y: f32, timestamp_ns: u128) -> PointerEvent {
    PointerEvent {
        x,
        y,
        pressure: 1.0,
        kind,
        source: PointerSource::Mouse,
        button: ph2d_host::PointerButton::Primary,
        timestamp_ns,
    }
}

fn key(kc: u32) -> KeyEvent {
    KeyEvent {
        keycode: kc,
        modifiers: Modifiers {
            shift: false,
            ctrl: false,
            alt: false,
            meta: false,
        },
        kind: KeyKind::Down,
        timestamp_ns: 0,
    }
}

const CHIP: NodeId = NodeId(2);
/// The chip's rect. Clicks land on the LEFT body — the right column is the
/// stepper zone, whose click is a value bump, not a focus gesture.
const RECT: Rect = Rect {
    x: 0.0,
    y: 0.0,
    w: 100.0,
    h: 24.0,
};

/// An UNFOCUSED chip showing `value`, hit-indexed at [`RECT`] — the state the
/// artist actually starts from (every prior fixture began pre-focused).
fn unfocused_chip(value: f64) -> (WidgetStore, HitIndex) {
    let mut store = WidgetStore::with_capacity(4);
    let buffer = format!("{value}");
    let len = buffer.len();
    store.register(
        CHIP,
        InteractiveState::NumberInput {
            state: TextInputState::Normal,
            value,
            buffer,
            caret: len,
            last_committed: value,
            selection_anchor: None,
        },
    );
    let mut hits = HitIndex::default();
    hits.register(CHIP, RECT);
    (store, hits)
}

/// One click on the chip's body. `at_ns` spaces successive clicks a second
/// apart ON PURPOSE — two Downs at the same spot within the double-click
/// window read as a double-click (select-all), which is another gesture.
fn click_body(store: &mut WidgetStore, hits: &HitIndex, arena: &Bump, at_ns: u128) {
    let _ = dispatch_pointer(
        store,
        hits,
        pointer(PointerKind::Down, 30.0, 12.0, at_ns),
        arena,
    );
    let _ = dispatch_pointer(
        store,
        hits,
        pointer(PointerKind::Up, 30.0, 12.0, at_ns),
        arena,
    );
}

const SECOND: u128 = 1_000_000_000;

/// The reported gesture, end to end: click the chip that shows "2", type "2",
/// Enter. The committed value must be what the artist TYPED — 2, never 22.
#[test]
fn clicking_a_chip_then_typing_replaces_the_readout() {
    let (mut store, hits) = unfocused_chip(2.0);
    // The Dur chip is commit-always flagged (typing the shown value must still
    // author) — this gate wears the same flag so the same-digit commit FIRES.
    store.set_number_commit_always(CHIP);
    let arena = Bump::new();
    click_body(&mut store, &hits, &arena, 0);
    let _ = dispatch_text_input(&mut store, '2', &arena);
    let events = dispatch_key(&mut store, key(KEY_ENTER), &arena);
    let (_, v, _, _, _) = store.number_input(CHIP).expect("chip");
    assert!(
        (v - 2.0).abs() < 1e-9,
        "typing '2' into a chip showing '2' must commit 2.0 (replace), \
         got {v} (append)"
    );
    assert!(
        events.iter().any(|e| matches!(
            e,
            ph2d_editor_core::interaction::WidgetEvent::ValueChanged(id) if *id == CHIP
        )),
        "the commit still fires ValueChanged, got {events:?}"
    );
}

/// A DIFFERENT typed value replaces too (the general case, not just the
/// same-digit coincidence).
#[test]
fn the_first_keystroke_after_focus_replaces_not_appends() {
    let (mut store, hits) = unfocused_chip(2.0);
    let arena = Bump::new();
    click_body(&mut store, &hits, &arena, 0);
    let _ = dispatch_text_input(&mut store, '5', &arena);
    let _ = dispatch_key(&mut store, key(KEY_ENTER), &arena);
    let (_, v, _, _, _) = store.number_input(CHIP).expect("chip");
    assert!(
        (v - 5.0).abs() < 1e-9,
        "typing '5' must commit 5.0, got {v} — the focus click must select all"
    );
}

/// The other half of the Blender model: a SECOND click on the already-focused
/// chip places the caret (collapses the selection), so typing INSERTS — the
/// surgical edit path is not lost to the fix.
#[test]
fn a_second_click_places_the_caret_for_surgical_edits() {
    let (mut store, hits) = unfocused_chip(2.0);
    let arena = Bump::new();
    click_body(&mut store, &hits, &arena, 0); // focus + select all
    click_body(&mut store, &hits, &arena, 2 * SECOND); // caret place, selection collapsed
    let _ = dispatch_text_input(&mut store, '5', &arena);
    let (_, _, buffer, _, _) = store.number_input(CHIP).expect("chip");
    assert!(
        buffer.contains('2') && buffer.contains('5'),
        "after a second click typing must INSERT (buffer keeps the 2), got {buffer:?}"
    );
}
