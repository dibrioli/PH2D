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

mod fx;
pub use fx::*;

use std::cell::{Cell, RefCell};

use crate::AudioEditCmd;

thread_local! {
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
    /// Whether the clipboard holds anything — a Paste button that lights up with nothing to paste
    /// is a button that lies.
    static HAS_CLIPBOARD: Cell<bool> = const { Cell::new(false) };
    /// Panel → shell: write one file per piece (needs a folder, so the shell drives it).
    static EXPORT_PIECES_REQ: Cell<bool> = const { Cell::new(false) };
    /// Panel -> shell: write every shipping target. Like the pieces, these become FILES, so
    /// the shell has to pick a folder -- the panel never touches the filesystem.
    static EXPORT_SET_REQ: Cell<bool> = const { Cell::new(false) };
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

/// Shell → panel: whether a waveform selection exists (enables the range ops).
pub fn set_has_selection(v: bool) {
    HAS_SELECTION.with(|c| c.set(v));
}

pub(crate) fn has_selection() -> bool {
    HAS_SELECTION.with(Cell::get)
}

/// Shell → panel: whether the clipboard holds audio (enables Paste).
pub fn set_has_clipboard(v: bool) {
    HAS_CLIPBOARD.with(|c| c.set(v));
}

pub(crate) fn has_clipboard() -> bool {
    HAS_CLIPBOARD.with(Cell::get)
}

/// Panel: ask the shell to write one file per piece. The shell drives it because the pieces become
/// FILES, and picking where they land is a dialog — the panel never touches the filesystem.
///
/// Splitting itself is an ordinary [`AudioEditCmd`](crate::AudioEditCmd) and needs none of this:
/// it changes the document, which is what the panel is for. The two used to be one button, and
/// that is why "Split at Markers" wrote eight files to disk.
pub(crate) fn request_export_pieces() {
    EXPORT_PIECES_REQ.with(|c| c.set(true));
}

/// Shell: drain the export-pieces request.
pub(crate) fn request_export_set() {
    EXPORT_SET_REQ.with(|c| c.set(true));
}

/// Shell: did the user ask for the whole platform set? Consumes the request.
pub fn take_export_set() -> bool {
    take(&EXPORT_SET_REQ)
}

pub fn take_export_pieces() -> bool {
    take(&EXPORT_PIECES_REQ)
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
