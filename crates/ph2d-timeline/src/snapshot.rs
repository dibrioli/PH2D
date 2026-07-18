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
    /// The clip — or container — it plays.
    pub clip_name: String,
    /// `Some(container index)` when this strip plays a CONTAINER (ADR-0133).
    ///
    /// The panel needs it to know whether "Enter Container" means anything here, and it is
    /// carried resolved rather than as a `StripSource` so the panel never has to know the
    /// document's types — the same reason `clip_name` is a name and not an index.
    pub container: Option<usize>,
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
    /// The OUTWARD fade-in ("lead-in"), in seconds — the fade that lives in the gap
    /// BEFORE the strip (the travel to this clip's start pose). The panel draws its
    /// wedge and grip to the LEFT of the strip's box, and it is mutually exclusive with
    /// `blend_in` (the fade-in grip is on one side of the start edge or the other).
    pub lead_in: f64,
    /// The window at this edge is defined by a NEIGHBOUR (the overlap), not by the
    /// strip's own `ease_in`/`ease_out` — so the ease handle there is **read-only**:
    /// the overlap wins, and the way to change the fade is to move the strips.
    ///
    /// This is Unity's rule, and shipping the flag in the snapshot (instead of letting
    /// the panel work it out from the strips it can see) is what keeps *one* answer to
    /// "whose window is this": [`crate::ClipLane::neighbour_reach_in`].
    pub ease_locked_in: bool,
    /// Mirror of `ease_locked_in`, at the end.
    pub ease_locked_out: bool,
    /// Whether its source loops, ping-pongs, or plays once.
    pub loop_mode: StripLoop,
    /// Its playback rate. The panel writes it on the strip whenever it is not
    /// `1.0` — a retimed strip that looks exactly like a real-time one is a strip
    /// whose speed the animator will rediscover by being surprised.
    pub speed: f64,
    /// **What each corner's last edit did**, in signed seconds — see
    /// [`crate::ClipStrip::marks`] and index it with [`crate::mark_index`]. The panel
    /// paints the span between the edge and `edge + mark` in that corner's colour, so
    /// a trim or a stretch stays legible long after the pointer let go.
    pub marks: [f64; 4],
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
    /// The active clip's **effective length** in seconds — its authored duration OR its last
    /// keyframe, whichever is later ([`crate::TimelineDoc::end_seconds`]).
    ///
    /// **NOT `duration()`.** A hand-keyed clip has an authored duration of `0` (keys never extend
    /// it), so `duration()` would report a 5-second animation as 0. This is the length a NEW STRIP
    /// is sized to, and `add_strip` sizes the strip's SLICE to the same `end_seconds`: read the
    /// two through different doors and a strip is born with `span ≠ slice`, i.e. a `speed` the
    /// artist never asked for — a 5 s clip crammed into a 1 s box at 5×. Transport's go-to-end
    /// reads `end_seconds` for the very same reason; this field is the panel's copy of that answer.
    pub clip_length_seconds: f64,
    /// **Where the KEYS ruler's playhead stands**, in the active clip's own time.
    ///
    /// The keys in [`Self::tracks`] are stamped in the clip's time and the strips in
    /// [`Self::lanes`] in the timeline's, so **one ruler cannot serve both**.
    ///
    /// In the **Keys** view ([`Self::keys_mode`]) this is the INDEPENDENT clip
    /// playhead the animator scrubs — the AE precomp model, so
    /// [`Self::time_seconds`] equals it (the whole transport reflects the clip
    /// clock while you edit its keys). Outside it, it is the derived answer to "is
    /// the active clip playing here" ([`crate::clip_playhead`]) — informational,
    /// and `None` when the clip is not under the timeline playhead exactly once.
    pub clip_time: Option<f64>,
    /// **The panel is showing the Keys view, driven by an independent clip
    /// playhead.** Set when the active tab is Keys: the scene solos the active clip
    /// ([`crate::apply_active_clip`]), the ruler scrubs the clip clock, and K keys
    /// there. It is a mirror of the panel's tab (a frame may lag on a switch), and
    /// it is what makes [`Self::time_seconds`] a CLIP time rather than a timeline
    /// one.
    pub keys_mode: bool,
    /// Track rows, in the **active clip's** own time — see [`Self::clip_time`].
    pub tracks: Vec<TrackView>,
    /// Markers as `(seconds, label)`.
    pub markers: Vec<(f64, String)>,
    /// Auto-key armed.
    pub auto_key: bool,
    /// Frame snapping on.
    pub frame_snap: bool,
    /// Performing (record-during-play) armed.
    pub performing: bool,
    /// The rigid simulation is armed on the transport (ADR-0131).
    pub simulate_physics: bool,
    /// The clip stack, bottom lane first. **Empty is the norm**: a document with
    /// no stack is one whose active clip drives the scene, and the panel paints no
    /// lane rows at all.
    pub lanes: Vec<LaneView>,
    /// Every clip's author-visible name, in document order — the selector's list.
    /// Reused across rebuilds (clear + push).
    pub clips: Vec<String>,
    /// Every container's name, index-aligned with `TimelineDoc::containers()` (ADR-0133).
    pub containers: Vec<String>,
    /// **Where the OPEN container's own playhead stands**, in its own seconds — `None` at the
    /// scene root (there is no second clock) or when the container is not playing exactly once
    /// here ([`crate::container_playhead`]).
    ///
    /// The panel's lanes are laid out in this clock when a container is open, so the ruler
    /// must mark it and not [`Self::time_seconds`] — and the two are shown TOGETHER, which is
    /// the only mitigation for "two clocks on one screen" that any product in the research
    /// actually shipped (After Effects stacks a second ruler above the first when Time
    /// Remapping is on, with a marker tying them).
    pub host_time: Option<f64>,
    /// **How the open container's own seconds relate to the timeline's** — the same fact as
    /// [`Self::host_time`], as a RELATION rather than a sample of it (`crate::nest_map`).
    ///
    /// The panel needs it in both directions: the ruler reads it to label the timeline's
    /// seconds over the interior's axis, and a scrub of that ruler writes through it, because
    /// the drag has to arrive as a time on the TIMELINE — the only playhead there is. `None`
    /// where the instance is not a bijection (not playing exactly once, or wrapping), and the
    /// ruler then does not scrub at all rather than seeking to a second it invented.
    pub host_map: Option<crate::ContainerMap>,
    /// **Where the animator IS** — the breadcrumb, outermost first.
    ///
    /// Empty means the document's own stack (the trail still paints its root: "Scene"). Each
    /// entry is `(container index, name)`, and clicking one pops back OUT to it — the
    /// Animate/Harmony gesture, which the research found is the 2D lineage (After Effects
    /// opens a new tab instead, and its users rebuild the lost context by hand).
    pub crumbs: Vec<(usize, String)>,
    /// Index into [`Self::clips`] of the clip being edited.
    pub active_clip: usize,
}

impl TimelineViewSnapshot {
    /// **Is a clip stack driving the scene?** The projection of
    /// `TimelineDoc::stack().is_empty()` — [`Self::lanes`] is filled from that very
    /// slice, so the two are the same fact and cannot disagree.
    ///
    /// It is the whole difference between one clock and two: without a stack the
    /// active clip IS the timeline, so a panel that asks this is really asking
    /// "does [`Self::clip_time`] mean something other than [`Self::time_seconds`]".
    #[must_use]
    pub fn stacked(&self) -> bool {
        !self.lanes.is_empty()
    }

    /// Refill this snapshot from `state` + the ACTIVE `playhead`, reusing the
    /// existing `tracks`/`markers`/`keys` buffers (clear + push, no fresh `Vec`s
    /// once the capacities are warm).
    ///
    /// # `playhead` is the ACTIVE clock, and `keys_mode` says which one it is
    ///
    /// In the Keys view the shell passes the **clip** playhead (an independent
    /// clip-time scrub) and `keys_mode = true`; everything derived here —
    /// `time_seconds`, `frame`, `playing`, `loop` — is then in clip time, which is
    /// what makes the whole transport reflect the clip you are editing (the AE
    /// precomp model). In the Arrange view it is the timeline playhead.
    ///
    /// # Why `&mut state`
    ///
    /// Only the Arrange path needs it: the derived [`Self::clip_time`] (is the active
    /// clip playing here?) lives in the stack's scratch, so this **primes it** rather
    /// than trust a caller to — a view publisher is the worst place for a hidden
    /// ordering contract ([[feedback_a_view_publisher_must_not_require_a_primed_cache]]).
    /// In Keys mode there is no stack in view, so the prime is skipped and the clip
    /// clock IS the (clip) playhead.
    pub fn rebuild(&mut self, state: &mut TimelineState, playhead: &Playhead, keys_mode: bool) {
        if !keys_mode && !state.doc.stack().is_empty() {
            state.doc.prime_stack(playhead.time());
        }
        let doc = &state.doc;
        self.fps = doc.fps_display;
        self.time_seconds = playhead.time();
        self.frame = playhead.frame(doc.fps_display);
        self.playing = playhead.is_playing();
        // The loop the panel DRAWS + braces come from the doc's view-appropriate
        // pair, not from the playhead: the Keys tab shows the clip-clock loop, the
        // Arrange tab the timeline loop, and they are independent (Enio, 2026-07-16).
        // The doc is the truth; the playhead is merely the live copy that wraps
        // playback, and reading display straight from the doc is what lets a tab
        // switch show the right braces before any sync has run.
        self.loop_range = doc.active_loop_for(keys_mode);
        self.loop_ping_pong = doc.active_ping_pong_for(keys_mode);
        self.clip_length_seconds = doc.end_seconds();
        self.keys_mode = keys_mode;
        // The KEYS ruler's playhead. In Keys mode the active playhead already IS the
        // clip clock, so publish it verbatim — the transport is the clip's. Outside
        // it, ask `clip_playhead` where the active clip is playing on the timeline
        // (the derived, informational answer; the same door K reads on the Arrange
        // side), which is `None` when it plays zero or two times.
        self.clip_time = if keys_mode {
            Some(playhead.time())
        } else {
            crate::clip_playhead(doc, playhead.time()).ok()
        };
        self.auto_key = state.flags.auto_key;
        self.frame_snap = state.flags.frame_snap;
        self.performing = state.flags.performing;
        self.simulate_physics = state.flags.simulate_physics;

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
        // **The lanes are the OPEN host's** (ADR-0133 §5): inside a container the panel shows
        // its interior, and the edits it raises are routed to the same host by
        // `TimelineState::edit_host`. One answer to "which stack am I looking at", read here
        // and honoured there — two would drift the first time somebody entered a container.
        self.containers.clear();
        self.containers
            .extend(doc.containers().iter().map(|c| c.name.clone()));
        self.crumbs.clear();
        self.host_time = None;
        self.host_map = None;
        if let crate::StackHost::Container(c) = state.edit_host
            && let Some(named) = doc.containers().get(c)
        {
            self.crumbs.push((c, named.name.clone()));
            // The scratch was primed above (the `!keys_mode && stacked` branch), which is the
            // same precondition `clip_playhead` rides.
            self.host_time = crate::container_playhead(doc, c, playhead.time()).ok();
            self.host_map = crate::container_map(doc, c);
        }
        let host_lanes: &[crate::ClipLane] = doc.host_stack(state.edit_host).unwrap_or(&[]);
        self.lanes.clear();
        for lane in host_lanes {
            let mut strips = Vec::with_capacity(lane.strips.len());
            for (i, st) in lane.strips.iter().enumerate() {
                strips.push(StripView {
                    id: st.id,
                    // The strip's source names itself, whichever list it lives in — a
                    // container strip that painted a blank label would read as a corrupt
                    // clip strip rather than as what it is (ADR-0133).
                    container: st.source.container_index().map(usize::from),
                    clip_name: match st.source {
                        crate::StripSource::Clip(i) => doc
                            .clips()
                            .get(i as usize)
                            .map_or_else(String::new, |c| c.name.clone()),
                        crate::StripSource::Container(i) => doc
                            .containers()
                            .get(i as usize)
                            .map_or_else(String::new, |c| c.name.clone()),
                    },
                    t_start: st.t_start,
                    t_end: st.t_end,
                    blend_in: lane.blend_in(i),
                    blend_out: lane.blend_out(i),
                    lead_in: st.lead_in,
                    marks: st.marks,
                    ease_locked_in: lane.neighbour_reach_in(i) > 0.0,
                    ease_locked_out: lane.neighbour_reach_out(i) > 0.0,
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
        snap.rebuild(&mut st, &ph, false);
        assert_eq!(snap.tracks.len(), 1);
        assert_eq!(snap.tracks[0].prop, PropKind::TranslationX);
        assert_eq!(snap.tracks[0].keys.len(), 1);
        assert!(snap.tracks[0].keys[0].selected, "new key is selected");
        assert!(!snap.playing);

        // Rebuilding into the same snapshot reuses the buffers (no growth).
        let cap = snap.tracks[0].keys.capacity();
        snap.rebuild(&mut st, &ph, false);
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
        snap.rebuild(&mut st, &ph, false);
        assert_eq!(snap.tracks.len(), 2, "both objects alive: two rows");

        // Entity 1's object dies (the apply pass flags it).
        st.doc.bindings_mut()[0].missing = true;
        snap.rebuild(&mut st, &ph, false);
        assert_eq!(snap.tracks.len(), 1, "the dead object's row is gone");
        assert_eq!(snap.tracks[0].entity, 2, "the live row survived");
        assert_eq!(
            st.doc.bindings().len(),
            2,
            "the document keeps the dormant binding — hidden, not dropped"
        );

        // It heals (the object came back) → the row returns.
        st.doc.bindings_mut()[0].missing = false;
        snap.rebuild(&mut st, &ph, false);
        assert_eq!(snap.tracks.len(), 2, "healed: the row is back");
    }

    /// **The snapshot shows each VIEW its own loop.** With a different loop parked in
    /// each pair, rebuilding in `keys_mode` publishes the Keys loop; rebuilding in
    /// Arrange publishes the timeline loop — the braces the panel draws follow the tab
    /// (Enio, 2026-07-16). Read from the DOC, not the playhead, so a tab switch shows
    /// the right loop before any sync runs.
    #[test]
    fn the_snapshot_publishes_the_views_own_loop() {
        let mut st = TimelineState::new();
        let ph = Playhead::new(1.0 / 60.0);
        st.doc.set_active_loop_for(false, Some((0.0, 2.0))); // Arrange
        st.doc.set_active_ping_pong_for(false, false);
        st.doc.set_active_loop_for(true, Some((1.5, 4.0))); // Keys
        st.doc.set_active_ping_pong_for(true, true);

        let mut snap = TimelineViewSnapshot::default();
        snap.rebuild(&mut st, &ph, true);
        assert_eq!(
            snap.loop_range,
            Some((1.5, 4.0)),
            "Keys tab shows the clip loop"
        );
        assert!(snap.loop_ping_pong, "and its ping-pong");

        snap.rebuild(&mut st, &ph, false);
        assert_eq!(
            snap.loop_range,
            Some((0.0, 2.0)),
            "Arrange tab shows the timeline loop"
        );
        assert!(!snap.loop_ping_pong, "which wraps");
    }
}
