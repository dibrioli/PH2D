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

thread_local! {
    // Panel → shell one-shot intents (drained by the bridge).
    static PLAY_PAUSE_REQ: Cell<bool> = const { Cell::new(false) };
    static STOP_REQ: Cell<bool> = const { Cell::new(false) };
    static LOAD_REQ: Cell<bool> = const { Cell::new(false) };
    static EXPORT_REQ: Cell<bool> = const { Cell::new(false) };
    // Panel → shell persistent.
    static LOOPING: Cell<bool> = const { Cell::new(false) };
    // Shell → panel display.
    static PLAYING: Cell<bool> = const { Cell::new(false) };
    static POSITION_SECS: Cell<f64> = const { Cell::new(0.0) };
    static DURATION_SECS: Cell<f64> = const { Cell::new(0.0) };
    static LOADED: Cell<bool> = const { Cell::new(false) };
    static CLIP_NAME: RefCell<String> = const { RefCell::new(String::new()) };
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

/// Read the clip name without cloning (calls `f` with the borrowed string).
pub(crate) fn with_clip_name<R>(f: impl FnOnce(&str) -> R) -> R {
    CLIP_NAME.with(|c| f(&c.borrow()))
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
        assert_eq!(with_clip_name(|s| s.to_string()), "kick.wav");
    }
}
