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
use crate::stack::{LaneMode, StripId, StripLoop};
use crate::state::{SelectedKey, TimelineState};

/// One key as the panel sees it (a dope-sheet diamond).
#[derive(Debug, Clone, PartialEq)]
pub struct KeyView {
    /// Stable key id (for hit-testing + selection).
    pub id: KeyId,
    /// Key time in seconds.
    pub t_seconds: f64,
    /// The key's value, projected to a scalar for the graph editor. Every
    /// [`PropKind`] is a `Float`; a non-scalar `AnimValue` (unreachable in v1)
    /// projects to `0.0` and draws as a flat curve.
    pub value: f32,
    /// Outgoing interpolation (drives the diamond glyph / graph handles).
    pub interp: Interp,
    /// Whether this key is in the current selection.
    pub selected: bool,
    /// Whether this key roves (its time is derived, not authored) — drawn as a
    /// small dot instead of the full diamond/anchor, AE-style.
    pub roving: bool,
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
    /// `true` when the bound entity is dead. Currently always `false` in a
    /// published snapshot: `rebuild` hides missing bindings entirely (their rows
    /// leave the panel and return if the binding heals by name). Kept for a
    /// future partial-missing display.
    pub missing: bool,
    /// The row's keys, in time order.
    pub keys: Vec<KeyView>,
}

/// One clip strip, as the panel draws it: a named rectangle.
///
/// It carries `clip_name` already resolved rather than an index for the panel to
/// look up — the panel is a pure function of this snapshot, and a strip whose
/// clip vanished between rebuild and paint would otherwise index into thin air.
#[derive(Debug, Clone, PartialEq)]
pub struct StripView {
    /// Stable identity — what a drag and a context menu hold on to.
    pub id: StripId,
    /// The clip it plays.
    pub clip_name: String,
    /// Its span on the timeline, in seconds.
    pub t_start: f64,
    /// Exclusive end.
    pub t_end: f64,
    /// The blend window at the start — the OVERLAP with the strip before it where
    /// there is one, the authored ease where there is not. The panel draws the
    /// crossfade from this, and the evaluator weights by the same number, so what
    /// is drawn and what is heard cannot drift apart.
    pub blend_in: f64,
    /// Mirror of `blend_in`, at the end.
    pub blend_out: f64,
    /// Whether its source loops, ping-pongs, or plays once.
    pub loop_mode: StripLoop,
    /// Its playback rate. The panel writes it on the strip whenever it is not
    /// `1.0` — a retimed strip that looks exactly like a real-time one is a strip
    /// whose speed the animator will rediscover by being surprised.
    pub speed: f64,
}

/// One lane of the clip stack.
#[derive(Debug, Clone, PartialEq)]
pub struct LaneView {
    /// Author-visible name.
    pub name: String,
    /// A muted lane is REMOVED from the blend — not merely turned down.
    pub muted: bool,
    /// Its influence over the stack below it.
    pub weight: f64,
    /// How it enters that stack.
    pub mode: LaneMode,
    /// Its strips, ordered by start time.
    pub strips: Vec<StripView>,
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
    /// The loop plays back and forth instead of wrapping. Only meaningful with a
    /// `loop_range`; the two transport toggles read
    /// `loop_range.is_some() && !ping_pong` and `loop_range.is_some() && ping_pong`,
    /// which is what makes them look mutually exclusive — because they ARE.
    pub loop_ping_pong: bool,
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
    /// Performing (record-during-play) armed.
    pub performing: bool,
    /// The clip stack, bottom lane first. **Empty is the norm**: a document with
    /// no stack is one whose active clip drives the scene, and the panel paints no
    /// lane rows at all.
    pub lanes: Vec<LaneView>,
    /// Every clip's author-visible name, in document order — the selector's list.
    /// Reused across rebuilds (clear + push).
    pub clips: Vec<String>,
    /// Index into [`Self::clips`] of the clip being edited.
    pub active_clip: usize,
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
        self.loop_ping_pong = doc.active_ping_pong();
        self.duration_seconds = doc.active_clip().duration().to_seconds();
        self.auto_key = state.flags.auto_key;
        self.frame_snap = state.flags.frame_snap;
        self.performing = state.flags.performing;

        // Clips (reuse buffer): names only — the panel never touches the curves of
        // a clip it is not showing.
        self.clips.clear();
        for c in doc.clips() {
            self.clips.push(c.name.clone());
        }
        self.active_clip = doc.active_index();

        // The clip stack. The blend windows are asked of the LANE, never recomputed
        // here — so the crossfade the panel DRAWS is literally the one the
        // evaluator WEIGHTS by, and the two cannot drift apart.
        self.lanes.clear();
        for lane in doc.stack() {
            let mut strips = Vec::with_capacity(lane.strips.len());
            for (i, st) in lane.strips.iter().enumerate() {
                strips.push(StripView {
                    id: st.id,
                    clip_name: doc
                        .clips()
                        .get(st.clip as usize)
                        .map_or_else(String::new, |c| c.name.clone()),
                    t_start: st.t_start,
                    t_end: st.t_end,
                    blend_in: lane.blend_in(i),
                    blend_out: lane.blend_out(i),
                    loop_mode: st.loop_mode,
                    speed: st.speed,
                });
            }
            self.lanes.push(LaneView {
                name: lane.name.clone(),
                muted: lane.muted,
                weight: lane.weight,
                mode: lane.mode,
                strips,
            });
        }

        // Markers (reuse buffer).
        self.markers.clear();
        for m in doc.markers() {
            self.markers.push((m.t.to_seconds(), m.label.clone()));
        }

        // Track rows: one per LIVE binding, in binding order — a missing binding
        // (its object was deleted) paints no row; the data stays dormant in the
        // document and the row returns if the binding heals (an undo brings the
        // object back under the same name — `refresh_and_heal_bindings`). Reuse
        // the outer Vec and each row's `keys` Vec across rebuilds.
        let clip = doc.active_clip();
        let mut rows = 0;
        for b in doc.bindings() {
            if b.missing {
                continue;
            }
            if self.tracks.len() == rows {
                self.tracks.push(TrackView {
                    target: AnimTarget::new(0),
                    prop: PropKind::TranslationX,
                    entity: 0,
                    missing: false,
                    keys: Vec::new(),
                });
            }
            let row = &mut self.tracks[rows];
            rows += 1;
            row.target = b.target;
            row.prop = b.prop;
            row.entity = b.entity;
            row.missing = b.missing;
            row.keys.clear();
            if let Some(track) = clip.track(b.target) {
                for ((k, &id), &roving) in track.keys().iter().zip(track.ids()).zip(track.roving())
                {
                    row.keys.push(KeyView {
                        id,
                        t_seconds: k.t.to_seconds(),
                        value: match k.value {
                            ph2d_anim::AnimValue::Float(v) => v,
                            _ => 0.0,
                        },
                        interp: k.interp,
                        selected: state.selection.contains(SelectedKey {
                            target: b.target,
                            key: id,
                        }),
                        roving,
                    });
                }
            }
        }
        self.tracks.truncate(rows);
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

    #[test]
    fn a_missing_binding_paints_no_row() {
        // Deleting an object must take its rows off the panel this frame (the
        // data stays dormant in the document; healing brings the row back).
        let mut st = TimelineState::new();
        let mut ph = Playhead::new(1.0 / 60.0);
        for entity in [1u64, 2] {
            apply_intent(
                &mut st,
                &mut ph,
                Ix::AddKey {
                    entity,
                    prop: PropKind::TranslationX,
                    t: s(0.0),
                    value: AnimValue::Float(0.0),
                    interp: Interp::Linear,
                },
            );
        }
        let mut snap = TimelineViewSnapshot::default();
        snap.rebuild(&st, &ph);
        assert_eq!(snap.tracks.len(), 2, "both objects alive: two rows");

        // Entity 1's object dies (the apply pass flags it).
        st.doc.bindings_mut()[0].missing = true;
        snap.rebuild(&st, &ph);
        assert_eq!(snap.tracks.len(), 1, "the dead object's row is gone");
        assert_eq!(snap.tracks[0].entity, 2, "the live row survived");
        assert_eq!(
            st.doc.bindings().len(),
            2,
            "the document keeps the dormant binding — hidden, not dropped"
        );

        // It heals (the object came back) → the row returns.
        st.doc.bindings_mut()[0].missing = false;
        snap.rebuild(&st, &ph);
        assert_eq!(snap.tracks.len(), 2, "healed: the row is back");
    }
}
