//! [`TimelineClipboard`] — the dope-sheet key clipboard (copy / cut / paste).
//!
//! Panel state: **not** part of the undoable document and never serialized (a
//! paste is the undo step, not the copy). Keys are stored **relative to the
//! earliest copied key**, so pasting lands the whole group at the playhead while
//! preserving its internal timing — copy three keys 0/0.5/2.0 s apart, scrub
//! anywhere, paste, and you get the same rhythm starting there.
//!
//! A sibling module of `history`/`state` (append-only extension point), so other
//! lines can grow `TimelineState` without colliding here.

use ph2d_anim::{AnimTarget, AnimValue, Interp};

/// One key on the clipboard: which track it came from, its offset from the copy
/// anchor (the earliest copied key), and the value/interp to re-create it.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ClipboardKey {
    /// The track this key belongs to.
    pub target: AnimTarget,
    /// Seconds after the earliest copied key (`>= 0`).
    pub offset_seconds: f64,
    /// The key's value.
    pub value: AnimValue,
    /// The key's outgoing interpolation.
    pub interp: Interp,
}

/// The copied keys, time-rebased to the earliest one.
#[derive(Debug, Default, Clone, PartialEq)]
pub struct TimelineClipboard {
    keys: Vec<ClipboardKey>,
}

impl TimelineClipboard {
    /// An empty clipboard.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Whether nothing has been copied.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.keys.is_empty()
    }

    /// How many keys are on the clipboard.
    #[must_use]
    pub fn len(&self) -> usize {
        self.keys.len()
    }

    /// The copied keys, offsets relative to the earliest.
    #[must_use]
    pub fn keys(&self) -> &[ClipboardKey] {
        &self.keys
    }

    /// Forget the copied keys.
    pub fn clear(&mut self) {
        self.keys.clear();
    }

    /// Replace the contents from `(target, absolute_seconds, value, interp)`
    /// tuples, re-basing every time so the earliest key sits at offset `0`.
    /// An empty input clears the clipboard.
    pub fn set_from_absolute(&mut self, keys: &[(AnimTarget, f64, AnimValue, Interp)]) {
        self.keys.clear();
        let Some(anchor) = keys
            .iter()
            .map(|(_, t, _, _)| *t)
            .fold(None, |acc: Option<f64>, t| {
                Some(acc.map_or(t, |a| a.min(t)))
            })
        else {
            return;
        };
        self.keys
            .extend(keys.iter().map(|(target, t, value, interp)| ClipboardKey {
                target: *target,
                offset_seconds: t - anchor,
                value: *value,
                interp: *interp,
            }));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn t(n: u64) -> AnimTarget {
        AnimTarget::new(n)
    }

    #[test]
    fn set_from_absolute_rebases_to_the_earliest_key() {
        let mut c = TimelineClipboard::new();
        c.set_from_absolute(&[
            (t(1), 2.0, AnimValue::Float(20.0), Interp::Linear),
            (t(1), 0.5, AnimValue::Float(5.0), Interp::Hold),
            (t(2), 1.0, AnimValue::Float(10.0), Interp::Linear),
        ]);
        assert_eq!(c.len(), 3);
        // Earliest (0.5 s) becomes the anchor; the rest keep their spacing.
        let offsets: Vec<f64> = c.keys().iter().map(|k| k.offset_seconds).collect();
        assert_eq!(offsets, vec![1.5, 0.0, 0.5]);
        // Identity + payload survive.
        assert_eq!(c.keys()[2].target, t(2));
        assert_eq!(c.keys()[1].interp, Interp::Hold);
    }

    #[test]
    fn empty_input_clears_the_clipboard() {
        let mut c = TimelineClipboard::new();
        c.set_from_absolute(&[(t(1), 1.0, AnimValue::Float(1.0), Interp::Linear)]);
        assert!(!c.is_empty());
        c.set_from_absolute(&[]);
        assert!(c.is_empty(), "an empty copy clears, never panics");
    }
}
