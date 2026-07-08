//! [`TimelineViewSnapshot`] — the read-only projection the timeline panel
//! paints, and [`TimelineViewSnapshot::rebuild`], which refills it from the
//! [`TimelineState`] + [`Playhead`].
//!
//! The bridge owns one snapshot and `rebuild`s it **into reused buffers** (no
//! per-frame allocation once warm — HR-3), only when the document or selection
//! actually changed. Paused + unchanged ⇒ no rebuild ⇒ no allocation (W1.T9).
//! The panel never reaches into the document; it reads this (mirrors
//! `GraphViewSnapshot`).

use ph2d_anim::{AnimTarget, Interp, KeyId};
use ph2d_core::Playhead;

use crate::prop::PropKind;
use crate::state::{SelectedKey, TimelineState};

/// One key as the panel sees it (a dope-sheet diamond).
#[derive(Debug, Clone, PartialEq)]
pub struct KeyView {
    /// Stable key id (for hit-testing + selection).
    pub id: KeyId,
    /// Key time in seconds.
    pub t_seconds: f64,
    /// Outgoing interpolation (drives the diamond glyph / graph handles).
    pub interp: Interp,
    /// Whether this key is in the current selection.
    pub selected: bool,
}

/// One track row as the panel sees it.
#[derive(Debug, Clone, PartialEq)]
pub struct TrackView {
    /// The track's opaque target.
    pub target: AnimTarget,
    /// The bound property (the panel resolves its label via `i18n_suffix`).
    pub prop: PropKind,
    /// The bound entity bits (for the row's object name lookup).
    pub entity: u64,
    /// `true` when the bound entity is dead → the row shows a "missing" badge.
    pub missing: bool,
    /// The row's keys, in time order.
    pub keys: Vec<KeyView>,
}

/// The whole panel view for one frame.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct TimelineViewSnapshot {
    /// Display frame rate (ruler + frame readouts).
    pub fps: f64,
    /// Playhead position in seconds.
    pub time_seconds: f64,
    /// Playhead frame index at `fps`.
    pub frame: i64,
    /// Whether the transport is playing.
    pub playing: bool,
    /// Loop range `[start, end)` in seconds, if set.
    pub loop_range: Option<(f64, f64)>,
    /// Active-clip duration in seconds.
    pub duration_seconds: f64,
    /// Track rows (active clip).
    pub tracks: Vec<TrackView>,
    /// Markers as `(seconds, label)`.
    pub markers: Vec<(f64, String)>,
    /// Auto-key armed.
    pub auto_key: bool,
    /// Frame snapping on.
    pub frame_snap: bool,
}

impl TimelineViewSnapshot {
    /// Refill this snapshot from `state` + `playhead`, reusing the existing
    /// `tracks`/`markers`/`keys` buffers (clear + push, no fresh `Vec`s once the
    /// capacities are warm).
    pub fn rebuild(&mut self, state: &TimelineState, playhead: &Playhead) {
        let doc = &state.doc;
        self.fps = doc.fps_display;
        self.time_seconds = playhead.time();
        self.frame = playhead.frame(doc.fps_display);
        self.playing = playhead.is_playing();
        self.loop_range = playhead.loop_range();
        self.duration_seconds = doc.active_clip().duration().to_seconds();
        self.auto_key = state.flags.auto_key;
        self.frame_snap = state.flags.frame_snap;

        // Markers (reuse buffer).
        self.markers.clear();
        for m in doc.markers() {
            self.markers.push((m.t.to_seconds(), m.label.clone()));
        }

        // Track rows: one per binding, in binding order. Reuse the outer Vec and
        // each row's `keys` Vec across rebuilds.
        let clip = doc.active_clip();
        let n = doc.bindings().len();
        if self.tracks.len() > n {
            self.tracks.truncate(n);
        }
        while self.tracks.len() < n {
            self.tracks.push(TrackView {
                target: AnimTarget::new(0),
                prop: PropKind::TranslationX,
                entity: 0,
                missing: false,
                keys: Vec::new(),
            });
        }
        for (row, b) in self.tracks.iter_mut().zip(doc.bindings()) {
            row.target = b.target;
            row.prop = b.prop;
            row.entity = b.entity;
            row.missing = b.missing;
            row.keys.clear();
            if let Some(track) = clip.track(b.target) {
                for (k, &id) in track.keys().iter().zip(track.ids()) {
                    row.keys.push(KeyView {
                        id,
                        t_seconds: k.t.to_seconds(),
                        interp: k.interp,
                        selected: state.selection.contains(SelectedKey {
                            target: b.target,
                            key: id,
                        }),
                    });
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::TimelineIntent as Ix;
    use crate::{PropKind, apply_intent};
    use ph2d_anim::{AnimValue, RationalTime};

    fn s(t: f64) -> RationalTime {
        RationalTime::from_seconds(t)
    }

    #[test]
    fn snapshot_projects_tracks_keys_selection_and_transport() {
        let mut st = TimelineState::new();
        let mut ph = Playhead::new(1.0 / 60.0);
        apply_intent(
            &mut st,
            &mut ph,
            Ix::AddKey {
                entity: 1,
                prop: PropKind::TranslationX,
                t: s(0.0),
                value: AnimValue::Float(0.0),
                interp: Interp::Linear,
            },
        );
        apply_intent(&mut st, &mut ph, Ix::Pause);

        let mut snap = TimelineViewSnapshot::default();
        snap.rebuild(&st, &ph);
        assert_eq!(snap.tracks.len(), 1);
        assert_eq!(snap.tracks[0].prop, PropKind::TranslationX);
        assert_eq!(snap.tracks[0].keys.len(), 1);
        assert!(snap.tracks[0].keys[0].selected, "new key is selected");
        assert!(!snap.playing);

        // Rebuilding into the same snapshot reuses the buffers (no growth).
        let cap = snap.tracks[0].keys.capacity();
        snap.rebuild(&st, &ph);
        assert_eq!(snap.tracks[0].keys.capacity(), cap, "key buffer reused");
    }
}
