//! Live snapshot bridging the Audio Editor panel and the shell's editor bridge.
//! Thread-local (UI + shell both run on the main thread), mirroring the Audio
//! Mixer panel's channels.
//!
//! Two directions:
//! - **panel → shell intents** (Play/Pause · Stop · Load · Export): one-shot
//!   flags the bridge *drains* each frame via the `take_*` getters, so a click
//!   fires exactly one engine action. Loop is a persistent flag.
//! - **shell → panel display** (playing · position · duration · loaded · name):
//!   the bridge publishes the live transport state for the readout + buttons.

use std::cell::{Cell, RefCell};

use crate::{AudioEditCmd, MAX_FX_PARAMS};

thread_local! {
    // Effects rack (W3 block 3a) — panel → shell: which effect is selected and
    // where its sliders sit (normalized 0..1; the shell owns the real ranges).
    static FX_KIND: Cell<usize> = const { Cell::new(0) };
    static FX_NORMS: Cell<[f32; MAX_FX_PARAMS]> = const { Cell::new([0.0; MAX_FX_PARAMS]) };
    // Effects rack — shell → panel: what to paint.
    static FX_KIND_COUNT: Cell<usize> = const { Cell::new(0) };
    static FX_KIND_NAME: RefCell<String> = const { RefCell::new(String::new()) };
    static FX_PARAM_VIEWS: RefCell<Vec<(String, String)>> = const { RefCell::new(Vec::new()) };
    static FX_DEFAULTS: Cell<[f32; MAX_FX_PARAMS]> = const { Cell::new([0.0; MAX_FX_PARAMS]) };
    /// Kind whose defaults were last loaded into the sliders. Mirror of the name
    /// box's sync guard: on a kind change the paint step re-seeds the sliders once
    /// instead of fighting the user's drag every frame.
    static FX_SYNCED_KIND: Cell<Option<usize>> = const { Cell::new(None) };

    // Panel → shell one-shot intents (drained by the bridge).
    static PLAY_PAUSE_REQ: Cell<bool> = const { Cell::new(false) };
    static STOP_REQ: Cell<bool> = const { Cell::new(false) };
    static LOAD_REQ: Cell<bool> = const { Cell::new(false) };
    static EXPORT_REQ: Cell<bool> = const { Cell::new(false) };
    static EDIT_CMD: Cell<Option<AudioEditCmd>> = const { Cell::new(None) };
    // Shell → panel: undo/redo availability (dims the buttons).
    static CAN_UNDO: Cell<bool> = const { Cell::new(false) };
    static CAN_REDO: Cell<bool> = const { Cell::new(false) };
    // Shell → panel: whether a waveform selection exists (enables range ops).
    static HAS_SELECTION: Cell<bool> = const { Cell::new(false) };
    // Panel → shell persistent.
    static LOOPING: Cell<bool> = const { Cell::new(false) };
    // Shell → panel display.
    static PLAYING: Cell<bool> = const { Cell::new(false) };
    static POSITION_SECS: Cell<f64> = const { Cell::new(0.0) };
    static DURATION_SECS: Cell<f64> = const { Cell::new(0.0) };
    static LOADED: Cell<bool> = const { Cell::new(false) };
    static CLIP_NAME: RefCell<String> = const { RefCell::new(String::new()) };
    /// Last name the paint step pushed into the name TextInput — so the box is
    /// re-synced only when a NEW clip loads, not every frame (which would fight
    /// user edits). Mirror of the Inspector name box's `last_entity` guard.
    static LAST_SYNCED_NAME: RefCell<Option<String>> = const { RefCell::new(None) };
}

/// Set + take a one-shot flag (returns the previous value, resets to `false`).
fn take(flag: &'static std::thread::LocalKey<Cell<bool>>) -> bool {
    flag.with(|c| c.replace(false))
}

/// Panel: request a play/pause toggle (the shell flips the preview transport).
pub(crate) fn request_play_pause() {
    PLAY_PAUSE_REQ.with(|c| c.set(true));
}

/// Shell: take the pending play/pause request (one-shot).
pub fn take_play_pause() -> bool {
    take(&PLAY_PAUSE_REQ)
}

/// Panel: request Stop (rewind to start + stop the preview).
pub(crate) fn request_stop() {
    STOP_REQ.with(|c| c.set(true));
}

/// Shell: take the pending Stop request (one-shot).
pub fn take_stop() -> bool {
    take(&STOP_REQ)
}

/// Panel: request Load (the shell opens a file picker + decodes).
pub(crate) fn request_load() {
    LOAD_REQ.with(|c| c.set(true));
}

/// Shell: take the pending Load request (one-shot).
pub fn take_load() -> bool {
    take(&LOAD_REQ)
}

/// Panel: request Export (the shell writes the current clip to WAV).
pub(crate) fn request_export() {
    EXPORT_REQ.with(|c| c.set(true));
}

/// Shell: take the pending Export request (one-shot).
pub fn take_export() -> bool {
    take(&EXPORT_REQ)
}

/// Panel: arm an edit command (overwrites any un-drained one — one click/frame).
pub(crate) fn request_edit(cmd: AudioEditCmd) {
    EDIT_CMD.with(|c| c.set(Some(cmd)));
}

/// Shell: take the pending edit command (one-shot).
pub fn take_edit_cmd() -> Option<AudioEditCmd> {
    EDIT_CMD.with(|c| c.take())
}

/// Shell → panel: whether undo/redo are available (dims the buttons).
pub fn set_can_undo(v: bool) {
    CAN_UNDO.with(|c| c.set(v));
}

pub(crate) fn can_undo() -> bool {
    CAN_UNDO.with(Cell::get)
}

pub fn set_can_redo(v: bool) {
    CAN_REDO.with(|c| c.set(v));
}

pub(crate) fn can_redo() -> bool {
    CAN_REDO.with(Cell::get)
}

// ---- Effects rack (W3 block 3a) ----

/// Panel: step the effect selector by `delta`, wrapping. No-op until the shell
/// has published how many kinds there are.
pub(crate) fn cycle_fx_kind(delta: isize) {
    let count = FX_KIND_COUNT.with(Cell::get);
    if count == 0 {
        return;
    }
    FX_KIND.with(|c| {
        let next = (c.get() as isize + delta).rem_euclid(count as isize);
        c.set(next as usize);
    });
}

/// Panel → shell: the selected effect's index into the shell's `FX_KINDS`.
pub fn fx_kind() -> usize {
    FX_KIND.with(Cell::get)
}

/// Panel → shell: every parameter slider's normalized 0..1 position.
pub fn fx_norms() -> [f32; MAX_FX_PARAMS] {
    FX_NORMS.with(Cell::get)
}

/// Panel: record slider `i`'s new normalized position.
pub(crate) fn set_fx_norm(i: usize, v: f32) {
    FX_NORMS.with(|c| {
        let mut n = c.get();
        if let Some(slot) = n.get_mut(i) {
            *slot = v.clamp(0.0, 1.0);
            c.set(n);
        }
    });
}

/// Shell → panel: how many effect kinds the selector cycles.
pub fn set_fx_kind_count(n: usize) {
    FX_KIND_COUNT.with(|c| c.set(n));
}

/// Shell → panel: the selected effect's display name.
pub fn set_fx_kind_name(name: &str) {
    FX_KIND_NAME.with(|c| {
        let mut s = c.borrow_mut();
        s.clear();
        s.push_str(name);
    });
}

pub(crate) fn fx_kind_name() -> String {
    FX_KIND_NAME.with(|c| c.borrow().clone())
}

/// Shell → panel: `(label, formatted value)` per parameter of the selected
/// effect. Length = that effect's parameter count; the panel hides the rest.
pub fn set_fx_param_views(views: &[(String, String)]) {
    FX_PARAM_VIEWS.with(|c| {
        let mut v = c.borrow_mut();
        v.clear();
        v.extend_from_slice(views);
    });
}

pub(crate) fn fx_param_views() -> Vec<(String, String)> {
    FX_PARAM_VIEWS.with(|c| c.borrow().clone())
}

/// Shell → panel: the normalized slider positions of the selected kind's preset.
pub fn set_fx_defaults(defaults: [f32; MAX_FX_PARAMS]) {
    FX_DEFAULTS.with(|c| c.set(defaults));
}

/// The defaults to seed the sliders with, iff the selected kind changed since the
/// last [`mark_fx_synced`]. `None` while the kind is unchanged, so a user's drag
/// isn't overwritten every frame.
pub(crate) fn fx_defaults_need_sync() -> Option<[f32; MAX_FX_PARAMS]> {
    let kind = FX_KIND.with(Cell::get);
    (FX_SYNCED_KIND.with(Cell::get) != Some(kind)).then(|| FX_DEFAULTS.with(Cell::get))
}

/// Record that the sliders now hold the selected kind's defaults.
pub(crate) fn mark_fx_synced() {
    FX_SYNCED_KIND.with(|c| c.set(Some(FX_KIND.with(Cell::get))));
}

/// Shell → panel: whether a waveform selection exists (enables the range ops).
pub fn set_has_selection(v: bool) {
    HAS_SELECTION.with(|c| c.set(v));
}

pub(crate) fn has_selection() -> bool {
    HAS_SELECTION.with(Cell::get)
}

/// Panel: flip the loop flag; returns the new value.
pub(crate) fn toggle_looping() -> bool {
    LOOPING.with(|c| {
        let next = !c.get();
        c.set(next);
        next
    })
}

/// Panel → shell: whether looping is enabled.
pub fn looping() -> bool {
    LOOPING.with(Cell::get)
}

/// Shell → panel: whether the preview is currently sounding.
pub fn set_playing(v: bool) {
    PLAYING.with(|c| c.set(v));
}

pub(crate) fn playing() -> bool {
    PLAYING.with(Cell::get)
}

/// Shell → panel: the preview's current playback position, in seconds.
pub fn set_position_secs(v: f64) {
    POSITION_SECS.with(|c| c.set(v));
}

pub(crate) fn position_secs() -> f64 {
    POSITION_SECS.with(Cell::get)
}

/// Shell → panel: the loaded clip's total duration, in seconds.
pub fn set_duration_secs(v: f64) {
    DURATION_SECS.with(|c| c.set(v));
}

pub(crate) fn duration_secs() -> f64 {
    DURATION_SECS.with(Cell::get)
}

/// Shell → panel: whether a clip is loaded (enables Export + transport).
pub fn set_loaded(v: bool) {
    LOADED.with(|c| c.set(v));
}

pub(crate) fn loaded() -> bool {
    LOADED.with(Cell::get)
}

/// Shell → panel: the loaded clip's display name (shown in the readout).
pub fn set_clip_name(name: &str) {
    CLIP_NAME.with(|c| {
        let mut s = c.borrow_mut();
        s.clear();
        s.push_str(name);
    });
}

/// The clip name if it differs from what was last pushed into the name box —
/// i.e. a new clip loaded. Returns `None` when already in sync. Does NOT update
/// the last-synced marker (call [`mark_name_synced`] once the box is updated),
/// so if the box is skipped this frame (user editing) it re-syncs next frame.
pub(crate) fn clip_name_needs_sync() -> Option<String> {
    CLIP_NAME.with(|c| {
        let cur = c.borrow();
        LAST_SYNCED_NAME.with(|l| {
            if l.borrow().as_deref() != Some(cur.as_str()) {
                Some(cur.clone())
            } else {
                None
            }
        })
    })
}

/// Record that the name box now reflects the current clip name.
pub(crate) fn mark_name_synced() {
    CLIP_NAME.with(|c| {
        let cur = c.borrow().clone();
        LAST_SYNCED_NAME.with(|l| *l.borrow_mut() = Some(cur));
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn intents_are_one_shot() {
        request_play_pause();
        assert!(take_play_pause(), "first take sees the request");
        assert!(!take_play_pause(), "second take is cleared");
        request_load();
        request_export();
        request_stop();
        assert!(take_load() && take_export() && take_stop());
        assert!(!take_load() && !take_export() && !take_stop());
    }

    #[test]
    fn loop_toggles_and_display_round_trips() {
        assert!(!looping());
        assert!(toggle_looping());
        assert!(looping());
        assert!(!toggle_looping());

        set_position_secs(1.5);
        set_duration_secs(3.0);
        set_playing(true);
        set_loaded(true);
        assert_eq!(position_secs(), 1.5);
        assert_eq!(duration_secs(), 3.0);
        assert!(playing() && loaded());

        set_clip_name("kick.wav");
        // A fresh name reads as "needs sync" (nothing pushed to the box yet).
        assert_eq!(clip_name_needs_sync().as_deref(), Some("kick.wav"));
        mark_name_synced();
        assert_eq!(clip_name_needs_sync(), None, "in sync after marking");
    }
}
