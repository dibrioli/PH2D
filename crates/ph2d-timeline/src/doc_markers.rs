//! Marker methods on [`TimelineDoc`], split from `doc.rs` to keep it under the LOC cap
//! ([ADR-0143] grew the marker cluster past 700). A child module of `doc`, so it reaches
//! the private `markers` field; the split is by responsibility (markers), not allowlist.
//!
//! [ADR-0143]: ../../../docs/architecture/decisions/0143-timeline-signals-a-marker-emits-a-decoupled-event-not-a-call.md

use super::{Marker, TimelineDoc};
use ph2d_anim::RationalTime;

impl TimelineDoc {
    /// All markers (sorted by insertion; the panel sorts for display).
    #[must_use]
    pub fn markers(&self) -> &[Marker] {
        &self.markers
    }

    /// Add a marker; returns its index.
    pub fn add_marker(&mut self, t: RationalTime, label: impl Into<String>) -> usize {
        self.markers.push(Marker {
            t,
            label: label.into(),
            signal: None,
        });
        self.markers.len() - 1
    }

    /// Remove the marker at `index`, returning `true` if it existed.
    pub fn remove_marker(&mut self, index: usize) -> bool {
        if index < self.markers.len() {
            self.markers.remove(index);
            true
        } else {
            false
        }
    }

    /// Move the marker at `index` to `t` (storage order is preserved, so the
    /// index stays valid across a drag). Returns `true` if it existed.
    pub fn move_marker(&mut self, index: usize, t: RationalTime) -> bool {
        match self.markers.get_mut(index) {
            Some(m) => {
                m.t = t;
                true
            }
            None => false,
        }
    }

    /// Relabel the marker at `index`. Returns `true` if it existed. Marker labels
    /// are user content, not HR-15 UI strings.
    pub fn set_marker_label(&mut self, index: usize, label: impl Into<String>) -> bool {
        match self.markers.get_mut(index) {
            Some(m) => {
                m.label = label.into();
                true
            }
            None => false,
        }
    }

    /// Set (or clear, with `None`) the signal the marker at `index` emits when the
    /// play crosses it ([ADR-0143]). Returns `true` if it existed. An empty or
    /// whitespace-only name **clears** the signal — a signal without a name is not a
    /// contract anyone can match, so it must not read as "has a signal".
    ///
    /// [ADR-0143]: ../../../docs/architecture/decisions/0143-timeline-signals-a-marker-emits-a-decoupled-event-not-a-call.md
    pub fn set_marker_signal(&mut self, index: usize, signal: Option<String>) -> bool {
        match self.markers.get_mut(index) {
            Some(m) => {
                m.signal = signal.filter(|s| !s.trim().is_empty());
                true
            }
            None => false,
        }
    }
}
