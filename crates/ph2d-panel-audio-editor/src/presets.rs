//! Chain-preset state for the Audio Editor panel (W3 — presets).
//!
//! Two kinds of preset, two mechanisms:
//! - **Factory presets** are browsed in-panel with a `◀ name ▶` selector (mirror of
//!   the effect selector) and applied to the chain by the shell, which owns the
//!   curated table. The panel holds only the selected index + the names the shell
//!   published, plus a one-shot "apply this one" intent.
//! - **User presets** are files: Save / Load arm one-shot intents the bridge drains
//!   into a native file dialog, then serializes / parses the chain shell-side (only
//!   the shell knows the effect-name ↔ kind mapping).
//!
//! Thread-local, like `snapshot` — the panel and the shell bridge both run on the
//! main thread.

use std::cell::{Cell, RefCell};

thread_local! {
    /// Which factory preset the selector shows. Clamped to the published count.
    static PRESET_SEL: Cell<usize> = const { Cell::new(0) };
    /// Shell → panel: the factory preset names, in table order.
    static PRESET_NAMES: RefCell<Vec<String>> = const { RefCell::new(Vec::new()) };
    /// Panel → shell one-shots (drained by the bridge each frame).
    static APPLY_PRESET_REQ: Cell<bool> = const { Cell::new(false) };
    static SAVE_PRESET_REQ: Cell<bool> = const { Cell::new(false) };
    static LOAD_PRESET_REQ: Cell<bool> = const { Cell::new(false) };
}

/// Shell → panel: publish the factory preset names (fixes how far the selector
/// cycles). Clamps the selection if the table shrank.
pub fn set_preset_names(names: &[&str]) {
    PRESET_NAMES.with(|c| {
        let mut v = c.borrow_mut();
        v.clear();
        v.extend(names.iter().map(|n| (*n).to_string()));
        let count = v.len();
        PRESET_SEL.with(|s| s.set(s.get().min(count.saturating_sub(1))));
    });
}

/// How many factory presets the selector cycles.
pub(crate) fn preset_count() -> usize {
    PRESET_NAMES.with(|c| c.borrow().len())
}

/// The selected factory preset's index (clamped to the table).
pub fn preset_sel() -> usize {
    PRESET_SEL
        .with(Cell::get)
        .min(preset_count().saturating_sub(1))
}

/// The selected factory preset's display name (empty if the table is empty).
pub(crate) fn preset_name() -> String {
    PRESET_NAMES.with(|c| c.borrow().get(preset_sel()).cloned().unwrap_or_default())
}

/// Panel: step the factory-preset selector, wrapping. No-op until the shell has
/// published the table. Does **not** apply anything — browsing is free.
pub(crate) fn cycle_preset(delta: isize) {
    let count = preset_count();
    if count == 0 {
        return;
    }
    PRESET_SEL.with(|c| {
        let next = (c.get() as isize + delta).rem_euclid(count as isize);
        c.set(next as usize);
    });
}

/// Panel: arm "load the selected factory preset into the chain".
pub(crate) fn request_apply_preset() {
    APPLY_PRESET_REQ.with(|c| c.set(true));
}

/// Shell: take the pending apply-preset request (one-shot).
pub fn take_apply_preset() -> bool {
    APPLY_PRESET_REQ.with(|c| c.replace(false))
}

/// Panel: arm "save the current chain to a user preset file".
pub(crate) fn request_save_preset() {
    SAVE_PRESET_REQ.with(|c| c.set(true));
}

/// Shell: take the pending save-preset request (one-shot).
pub fn take_save_preset() -> bool {
    SAVE_PRESET_REQ.with(|c| c.replace(false))
}

/// Panel: arm "load a user preset file into the chain".
pub(crate) fn request_load_preset() {
    LOAD_PRESET_REQ.with(|c| c.set(true));
}

/// Shell: take the pending load-preset request (one-shot).
pub fn take_load_preset() -> bool {
    LOAD_PRESET_REQ.with(|c| c.replace(false))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn selector_cycles_and_wraps_over_the_published_table() {
        set_preset_names(&["A", "B", "C"]);
        assert_eq!(preset_sel(), 0);
        assert_eq!(preset_name(), "A");
        cycle_preset(1);
        assert_eq!(preset_sel(), 1);
        cycle_preset(-1);
        cycle_preset(-1);
        assert_eq!(preset_sel(), 2, "Prev at 0 wraps to the last");
    }

    #[test]
    fn a_shrinking_table_clamps_the_selection() {
        set_preset_names(&["A", "B", "C", "D"]);
        cycle_preset(1);
        cycle_preset(1);
        cycle_preset(1);
        assert_eq!(preset_sel(), 3);
        set_preset_names(&["A", "B"]);
        assert_eq!(preset_sel(), 1, "selection followed the table down");
    }

    #[test]
    fn the_intents_are_one_shot() {
        request_apply_preset();
        assert!(take_apply_preset());
        assert!(!take_apply_preset());
        request_save_preset();
        request_load_preset();
        assert!(take_save_preset() && take_load_preset());
        assert!(!take_save_preset() && !take_load_preset());
    }
}
