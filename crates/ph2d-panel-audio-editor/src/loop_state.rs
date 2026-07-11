//! Asset-prep panel state for the Audio Editor (W6): **loop points** + **batch LUFS**.
//!
//! The panel owns only intents + display state; the shell owns the `EditClip` loop
//! region, the click-free crossfade, the `smpl` export, and the folder batch. Loop:
//! one persistent control (the crossfade slider position) and two one-shot intents
//! (Set from selection · Clear); the loop plays via the transport's Loop toggle + Play
//! (no Audition control). Batch LUFS: one one-shot intent the bridge turns into a
//! folder picker.
//!
//! Thread-local, like `snapshot`/`presets` — the panel and the shell bridge both run
//! on the main thread.

use std::cell::Cell;

thread_local! {
    /// Panel → shell: the loop crossfade slider position, normalized `0..1`. The
    /// shell maps it onto a real crossfade length in ms.
    static XFADE_NORM: Cell<f32> = const { Cell::new(DEFAULT_XFADE_NORM) };
    /// Shell → panel: the loop region as `(start_secs, end_secs)` for the readout,
    /// or `None` when no loop is set. Doubles as the has-loop signal.
    static LOOP_SPAN: Cell<Option<(f64, f64)>> = const { Cell::new(None) };
    /// Panel → shell one-shots (drained by the bridge each frame).
    static SET_LOOP_REQ: Cell<bool> = const { Cell::new(false) };
    static CLEAR_LOOP_REQ: Cell<bool> = const { Cell::new(false) };
    /// Panel → shell: normalize a whole folder of audio to a target loudness.
    static BATCH_LUFS_REQ: Cell<bool> = const { Cell::new(false) };
    /// Panel → shell: flip the non-destructive force-mono output toggle (one-shot).
    static TOGGLE_MONO_REQ: Cell<bool> = const { Cell::new(false) };
    /// Shell → panel: whether force-mono is currently engaged (lights the toggle).
    static MONO_ON: Cell<bool> = const { Cell::new(false) };
    /// Panel → shell one-shots: add a marker at the playhead / delete the nearest.
    static ADD_MARKER_REQ: Cell<bool> = const { Cell::new(false) };
    static DEL_MARKER_REQ: Cell<bool> = const { Cell::new(false) };
    /// Shell → panel: how many markers exist (for the readout + Del enablement).
    static MARKER_COUNT: Cell<usize> = const { Cell::new(0) };
}

/// Panel: arm "add a cue marker at the playhead".
pub(crate) fn request_add_marker() {
    ADD_MARKER_REQ.with(|c| c.set(true));
}

/// Shell: take the pending add-marker request (one-shot).
pub fn take_add_marker() -> bool {
    ADD_MARKER_REQ.with(|c| c.replace(false))
}

/// Panel: arm "delete the marker nearest the playhead".
pub(crate) fn request_del_marker() {
    DEL_MARKER_REQ.with(|c| c.set(true));
}

/// Shell: take the pending delete-marker request (one-shot).
pub fn take_del_marker() -> bool {
    DEL_MARKER_REQ.with(|c| c.replace(false))
}

/// Shell → panel: publish the marker count.
pub fn set_marker_count(n: usize) {
    MARKER_COUNT.with(|c| c.set(n));
}

/// How many markers exist (readout + Del enablement).
pub(crate) fn marker_count() -> usize {
    MARKER_COUNT.with(Cell::get)
}

/// Panel: arm "flip the force-mono output toggle".
pub(crate) fn request_toggle_mono() {
    TOGGLE_MONO_REQ.with(|c| c.set(true));
}

/// Shell: take the pending mono-toggle request (one-shot).
pub fn take_toggle_mono() -> bool {
    TOGGLE_MONO_REQ.with(|c| c.replace(false))
}

/// Shell → panel: publish whether force-mono is engaged.
pub fn set_mono_on(on: bool) {
    MONO_ON.with(|c| c.set(on));
}

/// Whether force-mono is engaged (for the toggle's lit state).
pub(crate) fn mono_on() -> bool {
    MONO_ON.with(Cell::get)
}

/// Panel: arm "batch-normalize a folder to a target LUFS".
pub(crate) fn request_batch_lufs() {
    BATCH_LUFS_REQ.with(|c| c.set(true));
}

/// Shell: take the pending batch-LUFS request (one-shot; the bridge opens a folder
/// picker).
pub fn take_batch_lufs() -> bool {
    BATCH_LUFS_REQ.with(|c| c.replace(false))
}

/// Default crossfade slider position (≈ the shell's mid-length crossfade).
pub(crate) const DEFAULT_XFADE_NORM: f32 = 0.3; // LITERAL-PX-OK: normalized 0..1 slider default

/// Panel: record the crossfade slider position.
pub(crate) fn set_xfade_norm(v: f32) {
    XFADE_NORM.with(|c| c.set(v.clamp(0.0, 1.0)));
}

/// Panel → shell: the crossfade slider position (`0..1`).
pub fn xfade_norm() -> f32 {
    XFADE_NORM.with(Cell::get)
}

/// Shell → panel: publish the loop region as `(start_secs, end_secs)` (or `None`).
pub fn set_loop_span(span: Option<(f64, f64)>) {
    LOOP_SPAN.with(|c| c.set(span));
}

/// The loop region as `(start_secs, end_secs)`, if any (for the readout).
pub(crate) fn loop_span() -> Option<(f64, f64)> {
    LOOP_SPAN.with(Cell::get)
}

/// Whether a loop region is set (drives the Clear/Snap/Audition enablement).
pub(crate) fn has_loop() -> bool {
    LOOP_SPAN.with(|c| c.get().is_some())
}

/// Panel: arm "set the loop region from the current selection".
pub(crate) fn request_set_loop() {
    SET_LOOP_REQ.with(|c| c.set(true));
}

/// Shell: take the pending set-loop request (one-shot).
pub fn take_set_loop() -> bool {
    SET_LOOP_REQ.with(|c| c.replace(false))
}

/// Panel: arm "clear the loop region".
pub(crate) fn request_clear_loop() {
    CLEAR_LOOP_REQ.with(|c| c.set(true));
}

/// Shell: take the pending clear-loop request (one-shot).
pub fn take_clear_loop() -> bool {
    CLEAR_LOOP_REQ.with(|c| c.replace(false))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn span_drives_has_loop_and_the_readout() {
        set_loop_span(None);
        assert!(!has_loop());
        assert_eq!(loop_span(), None);
        set_loop_span(Some((1.25, 3.5)));
        assert!(has_loop());
        assert_eq!(loop_span(), Some((1.25, 3.5)));
    }

    #[test]
    fn intents_are_one_shot() {
        request_set_loop();
        assert!(take_set_loop());
        assert!(!take_set_loop());
        request_clear_loop();
        assert!(take_clear_loop());
        assert!(!take_clear_loop());
    }

    #[test]
    fn xfade_norm_clamps() {
        set_xfade_norm(1.4);
        assert_eq!(xfade_norm(), 1.0);
        set_xfade_norm(-0.2);
        assert_eq!(xfade_norm(), 0.0);
    }

    #[test]
    fn batch_lufs_intent_is_one_shot() {
        request_batch_lufs();
        assert!(take_batch_lufs());
        assert!(!take_batch_lufs());
    }
}
