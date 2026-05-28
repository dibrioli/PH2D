//! Regression guard for the 2026-05-27 bug "typing 0.2 in the Grow
//! chip commits a totally different number (saw 0.6)".
//!
//! Root cause: `commit_number_buffer` / `apply_number_stepper_if_hit`
//! / `update_drag_value` / drag-scrub / continuous-hold all mirrored
//! the chip's value into the slider as if both lived in the same
//! `0..1` storage. Chips painted via `display_override` (Grow ±1,
//! Min Px integer count, Upscale "×N", etc.) actually display in a
//! different space, so the round-trip diverged on every interaction:
//!
//! - Slider at storage 0.5 → display "+0.00" (signed).
//! - User clicks chip, types `0.2`, Enter.
//! - Old path: parsed 0.2 → slider.value = 0.2.clamp(0..1) = 0.2.
//!   Next frame painter recomputes signed = (0.2-0.5)*2 = -0.6 →
//!   chip shows "-0.60". User typed 0.2 and saw something completely
//!   unrelated.
//!
//! Fix: [`WidgetStore::link_slider_number_mapped`] registers an affine
//! projection `display = storage*scale + offset`. Every chip↔slider
//! mirror path inverse-projects on the way to the slider and
//! forward-projects on the way back. Identity (`scale=1, offset=0`) =
//! the pre-fix path, so the existing call sites stay correct.

use bumpalo::Bump;
use ph2d_a11y::NodeId;
use ph2d_editor_core::interaction::dispatch::keymap::{KEY_BACKSPACE, KEY_ENTER};
use ph2d_editor_core::interaction::{
    HitIndex, InteractiveState, WidgetStore, dispatch_key, dispatch_pointer, dispatch_text_input,
};
use ph2d_editor_core::widget::{SliderOrientation, SliderState, TextInputState};
use ph2d_editor_core::zones::Rect;
use ph2d_host::{KeyEvent, KeyKind, Modifiers, PointerEvent, PointerKind, PointerSource};

fn pointer(kind: PointerKind, x: f32, y: f32) -> PointerEvent {
    PointerEvent {
        x,
        y,
        pressure: 1.0,
        kind,
        source: PointerSource::Mouse,
        button: ph2d_host::PointerButton::Primary,
        timestamp_ns: 0,
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

/// Build a slider+chip pair seeded to `slider_value` / `chip_display`,
/// linked via [`link_slider_number_mapped`] with `(scale, offset)`.
/// Chip starts Focused with caret at buffer end, matching the live
/// "user just clicked the chip" state so subsequent dispatched
/// keystrokes mutate the buffer.
fn build_pair(scale: f32, offset: f32, slider_value: f32, chip_display: f64) -> WidgetStore {
    let mut store = WidgetStore::with_capacity(8);
    store.register(
        NodeId(1),
        InteractiveState::Slider {
            state: SliderState::Normal,
            value: slider_value,
            orientation: SliderOrientation::Horizontal,
        },
    );
    let buffer = format!("{chip_display:.3}");
    let buffer_len = buffer.len();
    store.register(
        NodeId(2),
        InteractiveState::NumberInput {
            state: TextInputState::Focused,
            value: chip_display,
            buffer,
            caret: buffer_len,
            last_committed: chip_display,
            selection_anchor: None,
        },
    );
    store.set_focus(Some(NodeId(2)));
    store.link_slider_number_mapped(NodeId(1), NodeId(2), scale, offset);
    store
}

#[test]
fn identity_mapping_is_default_and_passthrough() {
    // No mapping registered → linked_slider_mapping returns identity.
    let mut store = WidgetStore::with_capacity(4);
    store.register(
        NodeId(1),
        InteractiveState::Slider {
            state: SliderState::Normal,
            value: 0.5,
            orientation: SliderOrientation::Horizontal,
        },
    );
    store.register(
        NodeId(2),
        InteractiveState::NumberInput {
            state: TextInputState::Normal,
            value: 0.5,
            buffer: "0.5".into(),
            caret: 3,
            last_committed: 0.5,
            selection_anchor: None,
        },
    );
    store.link_slider_number(NodeId(1), NodeId(2));
    assert_eq!(store.linked_slider_mapping(NodeId(2)), (1.0, 0.0));
}

#[test]
fn mapped_link_keyboard_commit_inverse_projects_to_slider() {
    // Grow mapping: display = storage*2 - 1, so storage = (display+1)/2.
    // Slider starts at 0.5 (display 0.0). User types "0.2" + Enter.
    // Expect: slider.value = (0.2 + 1) / 2 = 0.6.
    let mut store = build_pair(2.0, -1.0, 0.5, 0.0);
    store.set_focus(Some(NodeId(2)));
    let arena = Bump::new();
    // Backspace × 5 to clear "0.000" buffer, then type "0.2".
    for _ in 0..5 {
        let _ = dispatch_key(&mut store, key(KEY_BACKSPACE), &arena);
    }
    for ch in ['0', '.', '2'] {
        let _ = dispatch_text_input(&mut store, ch, &arena);
    }
    let _ = dispatch_key(&mut store, key(KEY_ENTER), &arena);
    let (_, slider_v) = store.slider(NodeId(1)).expect("slider");
    assert!(
        (slider_v - 0.6).abs() < 1e-5,
        "mapped commit must inverse-project: slider expected 0.6, got {slider_v}"
    );
    // Chip's value retains the display-space input — exactly what the
    // user typed. The next paint's `display_override` would recompute
    // (storage 0.6 → display 0.2) and match.
    let (_, chip_v, _, _, _) = store.number_input(NodeId(2)).expect("chip");
    assert!((chip_v - 0.2).abs() < 1e-5, "chip kept display value: {chip_v}");
}

#[test]
fn mapped_link_keyboard_commit_handles_negative_display() {
    // Same Grow mapping; user types "-0.5" which projects to slider 0.25.
    let mut store = build_pair(2.0, -1.0, 0.5, 0.0);
    store.set_focus(Some(NodeId(2)));
    let arena = Bump::new();
    for _ in 0..5 {
        let _ = dispatch_key(&mut store, key(KEY_BACKSPACE), &arena);
    }
    for ch in ['-', '0', '.', '5'] {
        let _ = dispatch_text_input(&mut store, ch, &arena);
    }
    let _ = dispatch_key(&mut store, key(KEY_ENTER), &arena);
    let (_, slider_v) = store.slider(NodeId(1)).expect("slider");
    assert!(
        (slider_v - 0.25).abs() < 1e-5,
        "negative display must round-trip: slider expected 0.25, got {slider_v}"
    );
}

#[test]
fn mapped_link_integer_count_round_trip() {
    // Min Px mapping: count = storage*255 + 1, scale=255, offset=1.
    // User types "50" → storage = (50-1)/255 = 49/255 ≈ 0.192156...
    let mut store = build_pair(255.0, 1.0, 0.0, 1.0);
    store.set_focus(Some(NodeId(2)));
    let arena = Bump::new();
    // Clear buffer "1.000" then type "50".
    for _ in 0..5 {
        let _ = dispatch_key(&mut store, key(KEY_BACKSPACE), &arena);
    }
    for ch in ['5', '0'] {
        let _ = dispatch_text_input(&mut store, ch, &arena);
    }
    let _ = dispatch_key(&mut store, key(KEY_ENTER), &arena);
    let (_, slider_v) = store.slider(NodeId(1)).expect("slider");
    let expected = 49.0 / 255.0;
    assert!(
        (slider_v - expected).abs() < 1e-5,
        "integer-count mapping: expected slider {expected}, got {slider_v}"
    );
}

#[test]
fn mapped_link_keyboard_commit_clamps_at_display_bounds() {
    // Grow mapping; user types out-of-range "+5.0" → should clamp to
    // slider 1.0 (= display 1.0). The pre-fix path clamped at the
    // wrong bounds (0..1 in display space) so positive grow was
    // unreachable AND the chip retained the bogus typed value.
    let mut store = build_pair(2.0, -1.0, 0.5, 0.0);
    store.set_focus(Some(NodeId(2)));
    let arena = Bump::new();
    for _ in 0..5 {
        let _ = dispatch_key(&mut store, key(KEY_BACKSPACE), &arena);
    }
    for ch in ['5', '.', '0'] {
        let _ = dispatch_text_input(&mut store, ch, &arena);
    }
    let _ = dispatch_key(&mut store, key(KEY_ENTER), &arena);
    let (_, slider_v) = store.slider(NodeId(1)).expect("slider");
    assert!(
        (slider_v - 1.0).abs() < 1e-5,
        "over-bound commit must clamp slider at 1.0, got {slider_v}"
    );
}

#[test]
fn slider_drag_forward_projects_to_chip_display() {
    // Grow mapping: dragging the slider to position 0.5 must surface
    // as display 0.0 on the chip (centre of ±1 readout), not 0.5.
    let mut store = build_pair(2.0, -1.0, 0.0, -1.0);
    let mut hits = HitIndex::new();
    hits.register(NodeId(1), Rect::new(0.0, 0.0, 100.0, 30.0));
    let arena = Bump::new();
    let _ = dispatch_pointer(
        &mut store,
        &hits,
        pointer(PointerKind::Down, 50.0, 15.0),
        &arena,
    );
    let (_, chip_v, _, _, _) = store.number_input(NodeId(2)).expect("chip");
    assert!(
        chip_v.abs() < 1e-5,
        "slider drag at storage 0.5 must surface chip display 0.0, got {chip_v}"
    );
}

#[test]
fn link_slider_number_mapped_with_identity_args_equals_legacy() {
    // `link_slider_number_mapped(slider, chip, 1.0, 0.0)` should be
    // semantically equivalent to `link_slider_number(slider, chip)` —
    // the mapping table stays empty and the helper returns identity.
    let mut store = WidgetStore::with_capacity(4);
    store.register(
        NodeId(1),
        InteractiveState::Slider {
            state: SliderState::Normal,
            value: 0.5,
            orientation: SliderOrientation::Horizontal,
        },
    );
    store.register(
        NodeId(2),
        InteractiveState::NumberInput {
            state: TextInputState::Normal,
            value: 0.5,
            buffer: "0.5".into(),
            caret: 3,
            last_committed: 0.5,
            selection_anchor: None,
        },
    );
    store.link_slider_number_mapped(NodeId(1), NodeId(2), 1.0, 0.0);
    assert_eq!(store.linked_slider_mapping(NodeId(2)), (1.0, 0.0));
    assert_eq!(store.linked_slider(NodeId(2)), Some(NodeId(1)));
    assert_eq!(store.linked_number(NodeId(1)), Some(NodeId(2)));
}
