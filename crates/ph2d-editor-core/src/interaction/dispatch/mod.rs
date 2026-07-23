//! Pointer + key dispatchers — translate raw shell events into
//! [`super::WidgetStore`] state mutations and [`super::WidgetEvent`]
//! emissions.
//!
//! Every dispatcher takes a `&'frame bumpalo::Bump` and returns a
//! `&'frame [WidgetEvent]` slice — the arena is the frame-local
//! event allocator. Caller drains the slice in the same frame and
//! resets the arena before the next frame.
//!
//! Phase A wires Button + Toggle. Slider/RadioGroup/Checkbox arrive
//! in Phase B; TextInput/NumberInput/Combobox in Phase C;
//! TreeView/ContextMenu/ColorPicker/Modal/Tabs in Phase D.

mod blender;
pub mod clipboard;
mod curve;
mod focus;
pub mod hierarchy;
mod hover;
pub mod key;
pub mod keymap;
mod number_input;
mod pointer;
mod pointer_down;
mod pointer_down_menus;
mod pointer_move;
mod pointer_up;
pub mod scroll;
pub mod text_input;
mod text_ops;
pub mod tick;

use blender::{apply_blender_channel_value, derive_blender_channel_value};
pub use clipboard::apply_clipboard_paste;
pub use key::{dispatch_key, graph_key_for};
pub use keymap::{
    KEY_ARROW_DOWN, KEY_ARROW_LEFT, KEY_ARROW_RIGHT, KEY_ARROW_UP, KEY_BACKSPACE, KEY_DELETE,
    KEY_ENTER, KEY_ESCAPE, KEY_F2, KEY_KEY_A, KEY_KEY_C, KEY_KEY_D, KEY_KEY_F, KEY_KEY_G,
    KEY_KEY_K, KEY_KEY_P, KEY_KEY_V, KEY_KEY_X, KEY_SPACE, KEY_TAB,
};
pub use pointer::{dispatch_pointer, dispatch_pointer_with_text};
pub use scroll::dispatch_wheel;
pub use text_input::dispatch_text_input;
pub use tick::dispatch_tick;

use super::{InteractiveState, WidgetEvent, WidgetStore};
use crate::zones::Rect;
use bumpalo::collections::Vec as BumpVec;

/// Apply a new chip display-value (e.g. from Enter commit, stepper
/// click, continuous hold). Writes the chip's `value`+`buffer`, then
/// inverse-projects through the registered mapping to write the linked
/// slider's clamped `0..1` storage, then RE-SYNCS the chip's
/// `value`+`buffer` from the clamped slider so that out-of-range typed
/// input (e.g. `"999"` in an upscale chip bounded to ×16) snaps the
/// visible buffer back to the clamped display.
///
/// Single source of truth for the chip↔slider mirror; the 4 dispatch
/// sites (commit / stepper / tick / drag-scrub) all call this so the
/// re-sync invariant can't drift between paths.
///
/// Returns `(final_chip_value, slider_was_clamped)`. Callers use the
/// returned value to drive downstream events (`WidgetEvent::ValueChanged`,
/// `last_committed` updates, etc.) without re-reading the store.
pub(super) fn apply_chip_value_with_mirror(
    store: &mut WidgetStore,
    chip_id: ph2d_a11y::NodeId,
    new_display: f64,
) -> (f64, bool) {
    // NaN / inf guard — degenerate input from upstream arithmetic
    // (e.g. tick step over a corrupted value, or a non-finite mapping)
    // shouldn't persist NaN into chip/slider state.
    if !new_display.is_finite() {
        return (new_display, false);
    }
    // Integer-domain snap (audit finding #3, 2026-05-28): chips
    // registered via `link_slider_number_mapped_integer` (BgRemoval
    // Min Px, Color-Eq Tile Grid / Posterize Dither Grain) round the
    // typed display before writing so the chip's persisted value
    // matches the painter's `display_override`. No-op for continuous-
    // domain chips (membership defaults to false).
    let new_display = if store.linked_slider_snap_integer(chip_id) {
        new_display.round()
    } else {
        new_display
    };
    if let Some(InteractiveState::NumberInput { value, buffer, .. }) = store.get_mut(chip_id) {
        *value = new_display;
        *buffer = super::format_number(new_display);
    } else {
        return (new_display, false);
    }
    let Some(slider_id) = store.linked_slider(chip_id) else {
        return (new_display, false);
    };
    let (scale, offset) = store.linked_slider_mapping(chip_id);
    // Runtime guard for degenerate mapping (release-build counterpart
    // of the `debug_assert!` in `link_slider_number_mapped`). A buggy
    // caller registering scale ≈ 0 would otherwise produce ±inf / NaN
    // storage. Skip the mirror; chip's raw value already written above.
    if scale.abs() <= f32::EPSILON {
        return (new_display, false);
    }
    let storage_raw = ((new_display as f32) - offset) / scale;
    let storage_clamped = storage_raw.clamp(0.0, 1.0); // CLAMP-OK: literal bounds 0,1
    // Compare clamped detection in DISPLAY-space — storage-space
    // EPSILON (~1.19e-7) is too tight near the bounds when scale is
    // large (a 1-ULP-out-of-range display value for padding scale=1024
    // produces a ~1e-10 storage diff that misses EPSILON, leaving the
    // chip out of sync). Display-space tolerance scales with the
    // mapping range.
    let display_eps = f32::EPSILON * scale.abs().max(1.0);
    let display_drift = ((storage_clamped - storage_raw) * scale).abs();
    let was_clamped = display_drift > display_eps;
    if let Some(InteractiveState::Slider { value, .. }) = store.get_mut(slider_id) {
        *value = storage_clamped;
    }
    if was_clamped {
        // Re-project the clamped storage into display and rewrite the
        // chip — keeps `value` + `buffer` in lockstep with the slider
        // thumb. `last_committed` is NOT touched here: drag-scrub uses
        // this same helper but must preserve the pre-drag anchor for
        // Esc rollback (audit fix #2 CRITICAL); the other 3 callers
        // (commit_number_buffer, apply_number_stepper_if_hit,
        // dispatch_tick) set `last_committed = final_v` themselves
        // after the helper returns.
        let resync_display = (storage_clamped * scale + offset) as f64;
        if let Some(InteractiveState::NumberInput { value, buffer, .. }) = store.get_mut(chip_id) {
            *value = resync_display;
            *buffer = super::format_number(resync_display);
        }
        return (resync_display, true);
    }
    (new_display, false)
}

pub(super) fn init_number_buffer(store: &mut WidgetStore, id: ph2d_a11y::NodeId) {
    // BlenderColorPicker channel chip: seed `value` from the parent
    // picker's current channel value (the chip's stored `value` is
    // stale; the painter renders the live derived value every
    // frame, but on focus the buffer needs to start from the
    // visible value, not the stale stored one).
    if let Some((parent, idx)) = store.blender_channel_chip(id) {
        let derived = derive_blender_channel_value(store, parent, idx);
        if let Some(InteractiveState::NumberInput { value, .. }) = store.get_mut(id) {
            *value = derived;
        }
    }
    if let Some(InteractiveState::NumberInput {
        state,
        value,
        buffer,
        caret,
        last_committed,
        selection_anchor,
    }) = store.get_mut(id)
    {
        *state = crate::widget::TextInputState::Focused;
        buffer.clear();
        use std::fmt::Write;
        let _ = write!(buffer, "{}", super::format_number(*value));
        *caret = buffer.len();
        *last_committed = *value;
        // **Focus selects ALL** — a numeric chip is a readout the artist
        // SUBSTITUTES: the first keystroke must replace the formatted value,
        // not append to it. With the anchor collapsed here, clicking a chip
        // showing "2" and typing "2" parsed "22" — the Dur(s) chip authored a
        // 22 s duration whose veil sat off-screen and whose clamp pinned
        // nothing (Enio, 2026-07-23; the keyboard-commit tests' `Backspace × 5`
        // dance was this same trap, confessed). Blender/AE convention: first
        // click selects all, a second click places the caret (the Down that
        // FOCUSED skips `place_text_caret` — see `pointer_down.rs`); Tab-focus
        // lands here too and gets the same replace-on-type.
        *selection_anchor = Some(0);
    }
}

/// True iff `id` belongs to a section header that supports the
/// right-click → SectionOutline context menu. Used by the right-click
/// dispatcher to decide whether to open that menu vs the create-note
/// menu.
///
/// Two source-of-truth arrays:
/// - [`crate::ids::SECTION_IDS`] — 10 Widget
///   Gallery (showcase) section headers.
/// - [`crate::ids::LIVE_SECTION_IDS`] — 6 live
///   Inspector section headers (Name / Visibility / Transform /
///   Render Source / Color & Tint / Sprite Sheet). Restored in Wave 4.1 so the section outline
///   affordance reaches the canonical Inspector, not just the demo
///   gallery.
///
/// Wave 2 PR 11.3 migrated NodeIds from numeric ranges to FNV-1a
/// hashes; this function used to test `350..=359` which became dead
/// after the migration (every hash falls outside that range), silently
/// breaking the affordance until Wave 4.1's audit caught it.
pub(super) fn is_section_header_id(id: ph2d_a11y::NodeId) -> bool {
    crate::ids::SECTION_IDS.contains(&id) || crate::ids::LIVE_SECTION_IDS.contains(&id)
}

/// Reset the focused visual state of a text-editing widget at `id`
/// to its `Normal` variant. Used on every blur path (Down handler,
/// `cycle_focus`, ESC, hex commit) so the painter stops drawing the
/// caret + focus border once the widget loses focus. Combobox uses
/// its own `ComboboxState` enum so it gets a separate match arm.
pub(super) fn reset_focused_visual_state(store: &mut WidgetStore, id: ph2d_a11y::NodeId) {
    match store.get_mut(id) {
        Some(InteractiveState::NumberInput { state, .. })
        | Some(InteractiveState::TextInput { state, .. }) => {
            *state = crate::widget::TextInputState::Normal;
        }
        Some(InteractiveState::Combobox { state, .. }) => {
            *state = crate::widget::ComboboxState::Normal;
        }
        _ => {}
    }
}

/// Set `selection_anchor = Some(0)` and `caret = text.len()` on the
/// focused TextInput / NumberInput widget at `id`. Triggered by
/// double-click and by Cmd/Ctrl+A. No-op for any other widget kind.
pub(super) fn select_all_in_text_widget(store: &mut WidgetStore, id: ph2d_a11y::NodeId) {
    match store.get_mut(id) {
        Some(InteractiveState::TextInput {
            text,
            caret,
            selection_anchor,
            ..
        }) => {
            *selection_anchor = Some(0);
            *caret = text.len();
        }
        Some(InteractiveState::NumberInput {
            buffer,
            caret,
            selection_anchor,
            ..
        }) => {
            *selection_anchor = Some(0);
            *caret = buffer.len();
        }
        Some(InteractiveState::Combobox {
            query,
            caret,
            selection_anchor,
            ..
        }) => {
            *selection_anchor = Some(0);
            *caret = query.len();
        }
        _ => {}
    }
}

/// On focus departure (Blur, Tab away, Enter commit) from a
/// NumberInput, parse `buffer.trim()`. On success → update `value` +
/// `last_committed` and emit `ValueChanged`. On failure → revert the
/// buffer to the formatted `last_committed`. After committing,
/// mirrors the new value into a linked Slider (clamped to [0..1]).
pub(super) fn commit_number_buffer<'a>(
    store: &mut WidgetStore,
    id: ph2d_a11y::NodeId,
    events: &mut BumpVec<'a, WidgetEvent>,
    force: bool,
) {
    // Phase 1 — parse the buffer. On parse failure, revert to
    // last_committed; on success, capture the parsed display value
    // (without writing yet; the mirror helper does the write so the
    // re-sync invariant lives in one place).
    let (parsed_value, prev_value) = {
        let Some(InteractiveState::NumberInput {
            value,
            buffer,
            caret,
            last_committed,
            ..
        }) = store.get_mut(id)
        else {
            return;
        };
        let prev = *value;
        match buffer.trim().parse::<f64>() {
            Ok(parsed) if parsed.is_finite() => (Some(parsed), prev),
            _ => {
                buffer.clear();
                use std::fmt::Write;
                let _ = write!(buffer, "{}", super::format_number(*last_committed));
                *value = *last_committed;
                *caret = buffer.len();
                (None, prev)
            }
        }
    };
    let mut new_value: Option<f64> = None;
    if let Some(parsed) = parsed_value
        && (parsed - prev_value).abs() > f64::EPSILON
    {
        // Mirror helper writes chip, slider (inverse-projected, clamped),
        // and re-syncs the chip if the slider clamped. Returns the
        // final chip value so we can park it in `last_committed` +
        // caret without re-reading the store.
        let (final_v, _was_clamped) = apply_chip_value_with_mirror(store, id, parsed);
        new_value = Some(final_v);
        if let Some(InteractiveState::NumberInput {
            caret,
            last_committed,
            buffer,
            ..
        }) = store.get_mut(id)
        {
            *last_committed = final_v;
            *caret = buffer.len();
        }
        events.push(WidgetEvent::ValueChanged(id));
        // The slider's stored value changed (or clamped) — emit so
        // downstream consumers re-render. Guard against a stale link
        // (slider id registered but the InteractiveState was replaced):
        // only emit when the slider variant is actually present, which
        // matches the helper's actual-write condition above.
        if let Some(slider_id) = store.linked_slider(id)
            && matches!(store.get(slider_id), Some(InteractiveState::Slider { .. }))
        {
            events.push(WidgetEvent::ValueChanged(slider_id));
        }
    } else if parsed_value.is_some() {
        // No-change commit (typed same value). Still normalize the
        // buffer so trailing whitespace etc. doesn't survive.
        if let Some(InteractiveState::NumberInput {
            value,
            buffer,
            caret,
            ..
        }) = store.get_mut(id)
        {
            buffer.clear();
            use std::fmt::Write;
            let _ = write!(buffer, "{}", super::format_number(*value));
            *caret = buffer.len();
        }
        // **An opt-in chip commits an UNCHANGED value on explicit ENTER**
        // (`force`), because its displayed number is a DERIVED readout and
        // typing that same number is a real DERIVED -> AUTHORED transition
        // downstream (the timeline Dur(s) box; Enio, 2026-07-23). Only on
        // `force` (Enter), never on a mere blur, and only for a flagged id —
        // every other chip keeps the delta-gate above.
        if force && store.number_commit_always(id) {
            events.push(WidgetEvent::ValueChanged(id));
        }
    }
    // BlenderColorPicker channel chip: write the parsed value back
    // into the parent picker's RGBA / HSVA dimension at `idx`.
    if let Some(v) = new_value
        && let Some((parent, idx)) = store.blender_channel_chip(id)
    {
        apply_blender_channel_value(store, parent, idx, v as f32);
        events.push(WidgetEvent::ValueChanged(parent));
    }
}

/// Restore a NumberInput's buffer to its last committed value
/// without emitting any event. Used by Escape.
pub(super) fn revert_number_buffer(store: &mut WidgetStore, id: ph2d_a11y::NodeId) {
    if let Some(InteractiveState::NumberInput {
        value,
        buffer,
        caret,
        last_committed,
        ..
    }) = store.get_mut(id)
    {
        *value = *last_committed;
        buffer.clear();
        use std::fmt::Write;
        let _ = write!(buffer, "{}", super::format_number(*last_committed));
        *caret = buffer.len();
    }
}

/// Parse the hex `TextInput` buffer at `id` and apply the resulting
/// color to the linked parent BlenderPicker (via
/// [`WidgetStore::link_blender_hex`]). Whether the parse succeeds or
/// not, the buffer is normalised to the canonical `#RRGGBBAA` form
/// of the parent's resulting value, so the painter always shows a
/// consistent string after commit. No-op if `id` is not a TextInput
/// or has no linked parent.
pub(super) fn commit_hex_buffer<'a>(
    store: &mut WidgetStore,
    id: ph2d_a11y::NodeId,
    events: &mut BumpVec<'a, WidgetEvent>,
) {
    let Some(parent) = store.blender_hex_parent(id) else {
        return;
    };
    let buf_owned: String = match store.get(id) {
        Some(InteractiveState::TextInput { text, .. }) => text.clone(),
        _ => return,
    };
    if let Some(color) = crate::widget::parse_hex(&buf_owned) {
        store.set_blender_value(parent, color);
        events.push(WidgetEvent::ValueChanged(parent));
    }
    write_hex_canonical(store, id);
}

/// Rewrite the hex `TextInput` buffer at `id` with the canonical
/// `#RRGGBBAA` form of the linked parent BlenderPicker's current
/// value. Used by both commit (after parse + apply) and revert
/// (ESC) so the visible text always matches the parent state.
pub(super) fn write_hex_canonical(store: &mut WidgetStore, id: ph2d_a11y::NodeId) {
    let Some(parent) = store.blender_hex_parent(id) else {
        return;
    };
    let Some((cv, ..)) = store.blender_picker(parent) else {
        return;
    };
    let [r, g, b, a] = cv.rgba;
    if let Some(InteractiveState::TextInput { text, caret, .. }) = store.get_mut(id) {
        text.clear();
        use std::fmt::Write;
        let _ = write!(text, "#{r:02X}{g:02X}{b:02X}{a:02X}");
        *caret = text.len();
    }
}

#[cfg(test)]
mod tests;
