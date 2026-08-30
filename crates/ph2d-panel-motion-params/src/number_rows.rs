//! The two numeric param rows that are NOT sliders (Motion Nodes M1.P2):
//!
//! - **Angle** — the app-standard number box with a `deg` unit chip
//!   ([`NumericInputWithUnit`]): type `90deg`, drag-scrub, stepper. Always shown
//!   in degrees whatever the param stores (turns / radians); the bridge hands the
//!   row a `deg_to_native` factor so the emitted value is node-native.
//! - **Seed** — a whole-number box plus a **re-roll** button. A seed has no
//!   magnitude, so it is never a slider: the artist wants *another* seed, not a
//!   *bigger* one. The re-roll is deterministic ([`next_seed`]), so a document
//!   re-opens identical and nothing depends on a wall clock.
//!
//! Both reuse the foundational widgets (`paint_number_input_with_buffer` +
//! `paint_icon_button`) rather than improvising chrome, and both mirror the doc
//! value into the store when unfocused — the same sync the Inspector's Transform
//! number boxes use.

use ph2d_a11y::NodeId;
use ph2d_editor_core::IconId;
use ph2d_editor_core::interaction::{HitIndex, InteractiveState, WidgetStore, format_number};
use ph2d_editor_core::paint::resolve;
use ph2d_editor_core::text_elide::paint_text_elided;
use ph2d_editor_core::widget::showcase::read_number_input;
use ph2d_editor_core::widget::{
    DEFAULT_LABEL_W, IconButtonStyle, IconGlyph, NumberInput, NumericInputWithUnit,
    SliderOrientation, SliderState, TextInputState, Unit, paint_icon_button,
    paint_number_input_with_buffer, paint_numeric_input_with_unit,
};
use ph2d_editor_core::zones::Rect;
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken, ROW_H_PX, Spacing, Theme, TypeToken};
use ph2d_vector::VectorScene;

/// Decimals shown in the Angle box — whole degrees (the Inspector's angle
/// convention; the `.`-less buffer also picks the coarse drag step).
pub(crate) const ANGLE_DECIMALS: usize = 0;
/// Decimals shown in the Seed box — a seed is a whole number.
pub(crate) const SEED_DECIMALS: usize = 0;

/// Format a value for a number box: whole when `decimals == 0`, else fixed.
fn fmt_val(v: f64, decimals: usize) -> String {
    if decimals == 0 {
        format!("{}", v.round() as i64)
    } else {
        format!("{v:.decimals$}")
    }
}

/// Register (if absent) + mirror the live doc `value` into the store's
/// `NumberInput` when unfocused — the app-standard per-frame sync. A focused box
/// owns its buffer (the user is typing), so it is left alone.
pub(crate) fn mirror_number(store: &mut WidgetStore, id: NodeId, value: f64, decimals: usize) {
    let text = fmt_val(value, decimals);
    let _ = store.register_if_absent(
        id,
        InteractiveState::NumberInput {
            state: TextInputState::Normal,
            value,
            buffer: text.clone(),
            caret: text.len(),
            last_committed: value,
            selection_anchor: None,
        },
    );
    if store.focus_id() != Some(id)
        && let Some(InteractiveState::NumberInput {
            value: v,
            buffer,
            caret,
            last_committed,
            ..
        }) = store.get_mut(id)
    {
        *v = value;
        buffer.clear();
        buffer.push_str(&text);
        *caret = buffer.len();
        *last_committed = value;
    }
}

/// **Mirror the doc value into a pooled SLIDER without touching its state.**
///
/// The scalar row's two halves are seeded every frame from the document, and the
/// seed owns the **value** while the *dispatch* owns the **state** — the same
/// division [`mirror_number`] above already keeps, and the same one the toggle
/// branch of `seed_rows` states in prose.
///
/// Re-registering the whole widget here is what broke Motion's hover: the pointer
/// pass writes `Hovered` and the paint that follows erased it, so a row lit under
/// the mouse and went out again before anything was drawn (measured, one paint:
/// `Hovered -> Normal`, on BOTH halves of the row). Patching in place cannot lose
/// it — there is no field left to remember to copy.
pub(crate) fn mirror_slider(store: &mut WidgetStore, id: NodeId, track: f32) {
    let _ = store.register_if_absent(
        id,
        InteractiveState::Slider {
            state: SliderState::Normal,
            value: track,
            orientation: SliderOrientation::Horizontal,
        },
    );
    if let Some(InteractiveState::Slider { value, .. }) = store.get_mut(id) {
        *value = track;
    }
}

/// The chip half of [`mirror_slider`] — every field the old wholesale re-register
/// wrote, **except** `state`. Distinct from [`mirror_number`] because the scalar
/// chip formats with the shared [`format_number`] (no per-row decimals) and parks
/// the caret at 0; focusing it runs `init_number_buffer`, which selects all.
pub(crate) fn mirror_chip(store: &mut WidgetStore, id: NodeId, v: f64) {
    let text = format_number(v);
    let _ = store.register_if_absent(
        id,
        InteractiveState::NumberInput {
            state: TextInputState::Normal,
            value: v,
            buffer: text.clone(),
            caret: 0,
            last_committed: v,
            selection_anchor: None,
        },
    );
    if let Some(InteractiveState::NumberInput {
        value,
        buffer,
        caret,
        last_committed,
        selection_anchor,
        ..
    }) = store.get_mut(id)
    {
        *value = v;
        buffer.clear();
        buffer.push_str(&text);
        *caret = 0;
        *last_committed = v;
        *selection_anchor = None;
    }
}

/// The number box's current committed value (what a `ValueChanged` reports —
/// a typed commit or the end of a drag-scrub).
pub(crate) fn number_value(store: &WidgetStore, id: NodeId) -> f64 {
    read_number_input(store, id).1
}

/// `true` while a pooled number box is focused (the user is typing into it) — the
/// undo bracket holds open, like a slider drag.
pub(crate) fn number_is_typing(store: &WidgetStore, id: NodeId) -> bool {
    matches!(
        store.get(id),
        Some(InteractiveState::NumberInput {
            state: TextInputState::Focused,
            ..
        })
    )
}

/// A left-aligned, vertically-centred row label clipped to the label column.
fn row_label(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    label: &str,
    rect: Rect,
    theme: Theme,
) {
    let font = TypeToken::Base.px();
    paint_text_elided(
        text_system,
        scene,
        label,
        rect.x,
        rect.y + (ROW_H_PX - font) * 0.5,
        font,
        DEFAULT_LABEL_W,
        resolve(ColorToken::Text1, theme),
    );
}

/// `label | [ number ][deg]` — the Angle row. The unit chip is inert; only the
/// editable field is registered for hit-testing. Returns the height consumed.
#[allow(clippy::too_many_arguments)]
pub(crate) fn paint_angle_row(
    rect: Rect,
    label: &str,
    id: NodeId,
    step: f64,
    store: &WidgetStore,
    hit_index: &mut HitIndex,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
) -> f32 {
    row_label(scene, text_system, label, rect, theme);
    let fx = rect.x + DEFAULT_LABEL_W + Spacing::Sm.px();
    let host = Rect::new(fx, rect.y, (rect.x + rect.w - fx).max(0.0), ROW_H_PX);

    let (state, value, buf, caret, anchor) = read_number_input(store, id);
    let buf = buf.to_string();
    let widget = NumericInputWithUnit::new(
        NumberInput::new(id, "", value)
            .step(step)
            .visual((state, store.hover_live(id))),
        Unit::Degrees,
    );
    paint_numeric_input_with_unit(
        &widget,
        Some(&buf),
        caret,
        anchor,
        host,
        scene,
        text_system,
        theme,
    );
    hit_index.register(id, widget.input_rect(host));
    ROW_H_PX
}

/// `label | [ number ] [re-roll]` — the Seed row. The button is a circular-arrow
/// icon (regenerate), sized to the row. Returns the height consumed.
#[allow(clippy::too_many_arguments)]
pub(crate) fn paint_seed_row(
    rect: Rect,
    label: &str,
    id: NodeId,
    reroll_id: NodeId,
    store: &WidgetStore,
    hit_index: &mut HitIndex,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
) -> f32 {
    row_label(scene, text_system, label, rect, theme);
    let gap = Spacing::Sm.px();
    let fx = rect.x + DEFAULT_LABEL_W + gap;
    let btn_w = ROW_H_PX; // square button, row-height (the Inspector's icon-button slot)
    let field_w = (rect.x + rect.w - fx - btn_w - gap).max(0.0);

    let (state, value, buf, caret, anchor) = read_number_input(store, id);
    let buf = buf.to_string();
    let field = Rect::new(fx, rect.y, field_w, ROW_H_PX);
    let input = NumberInput::new(id, "", value)
        .step(1.0)
        .visual((state, store.hover_live(id)));
    paint_number_input_with_buffer(
        &input,
        Some(&buf),
        caret,
        anchor,
        field,
        scene,
        text_system,
        theme,
    );
    hit_index.register(id, field);

    let brect = Rect::new(fx + field_w + gap, rect.y, btn_w, ROW_H_PX);
    let bstate = store.button_visual(reroll_id);
    paint_icon_button(
        brect,
        IconGlyph::Builtin(IconId::Rotate),
        IconButtonStyle::Plain,
        bstate,
        scene,
        theme,
    );
    hit_index.register(reroll_id, brect);
    ROW_H_PX
}

/// splitmix64 — one round of the finalizer, the same mixer the brush jitter uses.
fn splitmix64(x: u64) -> u64 {
    let mut z = x.wrapping_add(0x9E37_79B9_7F4A_7C15);
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}

/// The next seed after `current`, folded into `[min, max]`. **Deterministic on
/// purpose**: re-roll walks a reproducible chain (no RNG, no wall clock), so a
/// saved document re-opens identical and a replay stays byte-stable. Guaranteed
/// to differ from `current` whenever the range holds more than one value.
pub(crate) fn next_seed(current: f64, min: f64, max: f64) -> f64 {
    let span = (max - min).max(0.0) as u64 + 1;
    let fold = |h: u64| min + (h % span) as f64;
    let base = current.max(0.0) as u64;
    let first = fold(splitmix64(base));
    if first != current || span == 1 {
        return first;
    }
    // A fixed point: advance once more so the button always visibly changes it.
    fold(splitmix64(splitmix64(base)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fmt_val_is_whole_for_zero_decimals() {
        assert_eq!(fmt_val(89.6, 0), "90");
        assert_eq!(fmt_val(-0.4, 0), "0");
        assert_eq!(fmt_val(1.234, 2), "1.23");
    }

    #[test]
    fn next_seed_stays_in_range_and_is_deterministic() {
        for s in 0..64 {
            let v = next_seed(f64::from(s), 0.0, 100.0);
            assert!((0.0..=100.0).contains(&v), "seed {s} -> {v} out of range");
            assert_eq!(v, next_seed(f64::from(s), 0.0, 100.0), "deterministic");
        }
    }

    #[test]
    fn next_seed_always_changes_the_value() {
        // The whole point of the button: it must never be a no-op.
        for s in 0..200 {
            let v = next_seed(f64::from(s), 0.0, 100.0);
            assert_ne!(v, f64::from(s), "re-roll of {s} returned itself");
        }
    }

    #[test]
    fn next_seed_honours_a_degenerate_range() {
        // A single-value range can only ever return that value (no infinite loop).
        assert_eq!(next_seed(5.0, 5.0, 5.0), 5.0);
    }

    #[test]
    fn next_seed_clamps_a_negative_current() {
        let v = next_seed(-9.0, 0.0, 10.0);
        assert!((0.0..=10.0).contains(&v));
    }
}
