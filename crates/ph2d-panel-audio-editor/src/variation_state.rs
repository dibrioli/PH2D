//! Variation-container panel state for the Audio Editor (W6 asset-prep).
//!
//! A variation container groups several clips and, on each trigger, plays **one**
//! chosen by a strategy (Random / Sequence / Shuffle), with per-play pitch/gain
//! jitter and per-entry weights (the Wwise Random/Sequence Container). The panel is
//! UI-only: it owns the selected row, the two jitter slider positions, and a set of
//! one-shot intents; the **shell** owns the [`ph2d_audio_edit::VariationSet`], the
//! decoded clips, the picker, and the audition through the preview voice.
//!
//! Thread-local, like `loop_state`/`snapshot` — the panel and the shell bridge both
//! run on the main thread. Kept in its own file (not `snapshot.rs`, which is at the
//! panel LOC cap, nor `loop_state.rs`) so the section can grow without pressure.

use std::cell::{Cell, RefCell};

thread_local! {
    /// Shell → panel: the row labels (name + formatted weight), in list order. Its
    /// length is the variation count.
    static NAMES: RefCell<Vec<String>> = const { RefCell::new(Vec::new()) };
    /// Shell → panel: the current strategy's display name (for the `◀ name ▶`
    /// selector readout).
    static STRATEGY: RefCell<String> = RefCell::new(String::from("Shuffle"));
    /// Panel-owned: the selected row (Remove / Weight act on it). The shell reads it
    /// to resolve the target.
    static SEL: Cell<usize> = const { Cell::new(0) };

    /// Panel → shell one-shots (drained by the bridge each frame).
    static ADD_REQ: Cell<bool> = const { Cell::new(false) };
    static ADD_FOLDER_REQ: Cell<bool> = const { Cell::new(false) };
    static REMOVE_REQ: Cell<bool> = const { Cell::new(false) };
    static PLAY_REQ: Cell<bool> = const { Cell::new(false) };
    static SAVE_REQ: Cell<bool> = const { Cell::new(false) };
    static LOAD_REQ: Cell<bool> = const { Cell::new(false) };
    /// Panel → shell: net strategy-cycle steps since the last drain (signed).
    static STRATEGY_STEP: Cell<i32> = const { Cell::new(0) };
    /// Panel → shell: net weight-doubling steps for the selected entry (each step
    /// halves/doubles the weight).
    static WEIGHT_STEP: Cell<i32> = const { Cell::new(0) };

    /// Panel → shell: the pitch-jitter slider position `0..1` (the shell maps it to
    /// `± semitones`).
    static PITCH_NORM: Cell<f32> = const { Cell::new(0.0) };
    /// Panel → shell: the gain-jitter slider position `0..1` (`± dB`).
    static GAIN_NORM: Cell<f32> = const { Cell::new(0.0) };
}

/// Default jitter slider position — neutral (no jitter). Variety comes from the
/// distinct clips; jitter is opt-in.
pub(crate) const DEFAULT_JITTER_NORM: f32 = 0.0; // LITERAL-PX-OK: normalized 0..1 slider default

// --- Shell → panel publishers -----------------------------------------------------

/// Shell → panel: publish the row labels (name + weight), in list order.
pub fn set_variation_names(names: &[String]) {
    NAMES.with(|c| {
        let mut v = c.borrow_mut();
        v.clear();
        v.extend_from_slice(names);
    });
}

/// Shell → panel: publish the current strategy's display name.
pub fn set_strategy_name(name: &str) {
    STRATEGY.with(|c| *c.borrow_mut() = name.to_string());
}

/// The row labels (for painting the list).
pub(crate) fn names() -> Vec<String> {
    NAMES.with(|c| c.borrow().clone())
}

/// How many variations exist (readout + Remove/Play enablement).
pub(crate) fn count() -> usize {
    NAMES.with(|c| c.borrow().len())
}

/// The current strategy's display name (selector readout).
pub(crate) fn strategy_name() -> String {
    STRATEGY.with(|c| c.borrow().clone())
}

// --- Selection --------------------------------------------------------------------

/// Panel: select row `i` (clamped to the current count).
pub(crate) fn select(i: usize) {
    let n = count();
    SEL.with(|c| c.set(if n == 0 { 0 } else { i.min(n - 1) }));
}

/// Panel → shell: the selected row index (clamped to the count).
pub fn variation_sel() -> usize {
    let n = count();
    SEL.with(|c| {
        let s = c.get();
        if n == 0 { 0 } else { s.min(n - 1) }
    })
}

// --- One-shot intents -------------------------------------------------------------

/// Panel: arm "add a variation" (the bridge opens a file picker + decodes).
pub(crate) fn request_add() {
    ADD_REQ.with(|c| c.set(true));
}
/// Shell: take the pending add request (one-shot).
pub fn take_add_variation() -> bool {
    ADD_REQ.with(|c| c.replace(false))
}

/// Panel: arm "add every clip in a folder" (the bridge opens a folder picker).
pub(crate) fn request_add_folder() {
    ADD_FOLDER_REQ.with(|c| c.set(true));
}
/// Shell: take the pending add-folder request (one-shot).
pub fn take_add_variation_folder() -> bool {
    ADD_FOLDER_REQ.with(|c| c.replace(false))
}

/// Panel: arm "remove the selected variation".
pub(crate) fn request_remove() {
    REMOVE_REQ.with(|c| c.set(true));
}
/// Shell: take the pending remove request (one-shot).
pub fn take_remove_variation() -> bool {
    REMOVE_REQ.with(|c| c.replace(false))
}

/// Panel: arm "audition the next variation" (pick + jitter + preview).
pub(crate) fn request_play() {
    PLAY_REQ.with(|c| c.set(true));
}
/// Shell: take the pending play request (one-shot).
pub fn take_play_variation() -> bool {
    PLAY_REQ.with(|c| c.replace(false))
}

/// Panel: arm "save the set to a manifest file".
pub(crate) fn request_save() {
    SAVE_REQ.with(|c| c.set(true));
}
/// Shell: take the pending save request (one-shot).
pub fn take_save_variation_set() -> bool {
    SAVE_REQ.with(|c| c.replace(false))
}

/// Panel: arm "load a set from a manifest file".
pub(crate) fn request_load() {
    LOAD_REQ.with(|c| c.set(true));
}
/// Shell: take the pending load request (one-shot).
pub fn take_load_variation_set() -> bool {
    LOAD_REQ.with(|c| c.replace(false))
}

/// Panel: cycle the strategy by `delta` (accumulates until drained).
pub(crate) fn cycle_strategy(delta: i32) {
    STRATEGY_STEP.with(|c| c.set(c.get() + delta));
}
/// Shell: take the net strategy-cycle steps (0 = none), resetting the accumulator.
pub fn take_strategy_step() -> i32 {
    STRATEGY_STEP.with(|c| c.replace(0))
}

/// Panel: bump the selected entry's weight by `steps` doublings (± halves/doubles).
pub(crate) fn bump_weight(steps: i32) {
    WEIGHT_STEP.with(|c| c.set(c.get() + steps));
}
/// Shell: take the net weight-doubling steps (0 = none), resetting the accumulator.
pub fn take_weight_step() -> i32 {
    WEIGHT_STEP.with(|c| c.replace(0))
}

// --- Jitter sliders (persistent, read every frame) --------------------------------

/// Panel: record the pitch-jitter slider position.
pub(crate) fn set_pitch_norm(v: f32) {
    PITCH_NORM.with(|c| c.set(v.clamp(0.0, 1.0)));
}
/// Panel → shell: the pitch-jitter slider position `0..1`.
pub fn pitch_jitter_norm() -> f32 {
    PITCH_NORM.with(Cell::get)
}
/// Panel: record the gain-jitter slider position.
pub(crate) fn set_gain_norm(v: f32) {
    GAIN_NORM.with(|c| c.set(v.clamp(0.0, 1.0)));
}
/// Panel → shell: the gain-jitter slider position `0..1`.
pub fn gain_jitter_norm() -> f32 {
    GAIN_NORM.with(Cell::get)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn names_drive_count_and_strategy_round_trips() {
        set_variation_names(&["a ×1.0".into(), "b ×2.0".into()]);
        assert_eq!(count(), 2);
        assert_eq!(names().len(), 2);
        set_strategy_name("Sequence");
        assert_eq!(strategy_name(), "Sequence");
    }

    #[test]
    fn selection_clamps_to_the_count() {
        set_variation_names(&["a".into(), "b".into()]);
        select(5);
        assert_eq!(variation_sel(), 1, "select must clamp to the last row");
        set_variation_names(&[]);
        assert_eq!(variation_sel(), 0, "empty set selects 0");
    }

    #[test]
    fn intents_are_one_shot() {
        request_add();
        assert!(take_add_variation());
        assert!(!take_add_variation());
        request_play();
        assert!(take_play_variation());
        assert!(!take_play_variation());
    }

    #[test]
    fn step_accumulators_net_and_reset() {
        let _ = take_strategy_step();
        cycle_strategy(1);
        cycle_strategy(1);
        cycle_strategy(-1);
        assert_eq!(take_strategy_step(), 1);
        assert_eq!(take_strategy_step(), 0);

        let _ = take_weight_step();
        bump_weight(1);
        bump_weight(1);
        assert_eq!(take_weight_step(), 2);
        assert_eq!(take_weight_step(), 0);
    }

    #[test]
    fn jitter_norms_clamp() {
        set_pitch_norm(1.4);
        assert_eq!(pitch_jitter_norm(), 1.0);
        set_gain_norm(-0.3);
        assert_eq!(gain_jitter_norm(), 0.0);
    }
}
