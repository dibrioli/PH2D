//! Motion Nodes document model (M0.T7).
//!
//! [`MotionDoc`] is the persistable document of the Motion Nodes module: the
//! frozen node [`Graph`] plus **UI-only** decoration ([`Backdrop`] group regions)
//! and `base_z` (the z-order base the sink columns offset from). Runtime state
//! (the persistent `Cook`, the `NodeRegistry`, the transport) lives in the shell's
//! `MotionState`, **not** here — this crate is pure model, no render/UI deps.
//!
//! ## Serialization — textual `v1` + `[backdrop]`
//!
//! Reuses the node graph's canonical textual form ([`ph2d_nodegraph::format`],
//! ADR-0032 §6): line-oriented, diffable, deterministic. The document appends one
//! `[backdrop]` section — sibling of `[layout]`, **UI-only** (never affects the
//! cook) — carrying `base_z` and the backdrops:
//!
//! ```text
//! v1
//! n <id> <type>
//! e <from> <fp> <to> <tp> <fwd|pre>
//! p <id> <param> <value>
//! [layout]
//! l <id> <x> <y>
//! [backdrop]
//! z <base_z>
//! b <id> <x> <y> <w> <h> <color> <title...>
//! w <to_node> <to_port> <x0> <y0> <x1> <y1> ...
//! ```
//!
//! The section is really *the UI-only section* — it carries the backdrops **and** the wire
//! waypoints (`w`, doc 44), both of which are decoration that never reaches the cook. Its
//! name is historical; splitting it would break every document already on disk for nothing.
//!
//! A graph-only text (no `[backdrop]` section) still loads — `base_z` defaults to
//! 0 and there are no backdrops — so the format is backward-compatible with a bare
//! `ph2d_nodegraph::format::to_text` dump.
//!
//! ## Undo — snapshot history
//!
//! [`MotionHistory`] snapshots the whole `MotionDoc` (mold of
//! `ph2d-vec-edit::History`): cheap because the doc is `Clone` (a `Graph` clone +
//! a small `Vec` + a `u32`). `begin` at gesture start, `commit_if_changed` at the
//! end (no-op if nothing changed), or `push_undo` for an atomic op.

mod waypoint;
pub use waypoint::Waypoints;

use ph2d_nodegraph::format::ParseError;
use ph2d_nodegraph::graph::Graph;
use std::fmt::Write as _;

/// A translucent group region drawn **behind** a set of nodes (Cavalry/Blender
/// frame-node analog). Pure decoration — never part of the cook. `color` indexes
/// the `graph-backdrop-1..8` tokens (0-based: `0` → `graph-backdrop-1`); values
/// out of range clamp at paint time, not here.
#[derive(Clone, Debug, PartialEq)]
pub struct Backdrop {
    /// Stable id, unique within the document (serialized; drives selection).
    pub id: u32,
    /// Top-left corner in graph space.
    pub x: f32,
    pub y: f32,
    /// Extent in graph space.
    pub w: f32,
    pub h: f32,
    /// Tint index into `graph-backdrop-1..8` (0..=7).
    pub color: u8,
    /// Free-text label. May contain spaces (serialized as the trailing field of
    /// the `b` record); leading/trailing whitespace is trimmed on load.
    pub title: String,
}

/// The Motion Nodes document: the frozen node graph + UI-only decoration.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct MotionDoc {
    /// The node graph (semantic — the only part that cooks).
    pub graph: Graph,
    /// UI-only group backdrops (see [`Backdrop`]).
    pub backdrops: Vec<Backdrop>,
    /// UI-only wire routing points (see [`Waypoints`]) — keyed by the input each wire
    /// lands on. Like the backdrops: decoration, never part of the cook.
    pub waypoints: Vec<Waypoints>,
    /// Base z-order the sink columns offset from (`base_z + quantized(z)`).
    pub base_z: u32,
}

impl MotionDoc {
    /// An empty document (empty graph, no backdrops, `base_z = 0`).
    pub fn new() -> Self {
        Self::default()
    }

    /// Serialize to the canonical textual form (deterministic: the graph section
    /// is sorted by `ph2d_nodegraph::format::to_text`; backdrops are emitted in
    /// ascending id order). Byte-identical for equal documents.
    pub fn to_text(&self) -> String {
        let mut out = ph2d_nodegraph::format::to_text(&self.graph);
        out.push_str("[backdrop]\n");
        // Doc-level scalar first, then backdrops sorted by id for a stable diff.
        let _ = writeln!(out, "z {}", self.base_z);
        let mut sorted: Vec<&Backdrop> = self.backdrops.iter().collect();
        sorted.sort_by_key(|b| b.id);
        for b in sorted {
            // Single-space separated; `title` is the trailing free-text field, so
            // titles with interior single spaces round-trip exactly.
            let _ = writeln!(
                out,
                "b {} {} {} {} {} {} {}",
                b.id, b.x, b.y, b.w, b.h, b.color, b.title
            );
        }
        // The wire routing points — the section's other UI-only citizen (doc 44).
        waypoint::emit(&mut out, &self.waypoints);
        out
    }

    /// Parse the canonical textual form. The graph section is delegated to
    /// [`ph2d_nodegraph::format::from_text`] (re-validated — a cyclic/corrupt file
    /// is rejected); the `[backdrop]` section is parsed here. A text with no
    /// `[backdrop]` section loads as a graph with `base_z = 0` and no backdrops.
    pub fn from_text(text: &str) -> Result<Self, ParseError> {
        // The section header always starts a line (`\n[backdrop]`), so anchoring on
        // the newline avoids a stray match inside a (whitespace-free) type name.
        let (graph_part, backdrop_part) = match text.split_once("\n[backdrop]") {
            Some((g, b)) => (g, b),
            None => (text, ""),
        };
        let graph = ph2d_nodegraph::format::from_text(graph_part)?;

        let mut base_z = 0u32;
        let mut backdrops: Vec<Backdrop> = Vec::new();
        let mut waypoints: Vec<Waypoints> = Vec::new();
        let mut seen = std::collections::BTreeSet::new();
        for line in backdrop_part
            .lines()
            .map(str::trim)
            .filter(|l| !l.is_empty())
        {
            // A wire's routing points (doc 44). Malformed → rejected, never half-read.
            if line.starts_with("w ") {
                waypoints
                    .push(waypoint::parse(line).ok_or_else(|| ParseError::BadLine(line.into()))?);
                continue;
            }
            // `b` carries a trailing free-text title, so split into at most 8 fields
            // on single spaces (to_text guarantees single-space separators). Other
            // records use whitespace-collapsing splits.
            if line.starts_with("b ") {
                let parts: Vec<&str> = line.splitn(8, ' ').collect();
                if parts.len() < 7 {
                    return Err(ParseError::BadLine(line.into()));
                }
                let id: u32 = field(&parts, 1, line)?;
                if !seen.insert(id) {
                    return Err(ParseError::BadLine(line.into())); // duplicate backdrop id
                }
                let x: f32 = finite(field(&parts, 2, line)?, line)?;
                let y: f32 = finite(field(&parts, 3, line)?, line)?;
                let w: f32 = finite(field(&parts, 4, line)?, line)?;
                let h: f32 = finite(field(&parts, 5, line)?, line)?;
                let color: u8 = field(&parts, 6, line)?;
                let title = parts
                    .get(7)
                    .map(|s| s.trim().to_string())
                    .unwrap_or_default();
                backdrops.push(Backdrop {
                    id,
                    x,
                    y,
                    w,
                    h,
                    color,
                    title,
                });
            } else {
                let mut tok = line.split_whitespace();
                match tok.next() {
                    Some("z") => {
                        base_z = tok
                            .next()
                            .and_then(|s| s.parse().ok())
                            .ok_or_else(|| ParseError::BadLine(line.into()))?;
                        if tok.next().is_some() {
                            return Err(ParseError::BadLine(line.into()));
                        }
                    }
                    // The section header line itself, if it survived the split
                    // (it does not, but tolerate a hand-edited leading marker).
                    Some("[backdrop]") => {}
                    _ => return Err(ParseError::BadLine(line.into())),
                }
            }
        }
        Ok(Self {
            graph,
            backdrops,
            waypoints,
            base_z,
        })
    }
}

/// Parse the `idx`-th whitespace field of a pre-split `b` record.
fn field<T: std::str::FromStr>(parts: &[&str], idx: usize, line: &str) -> Result<T, ParseError> {
    parts
        .get(idx)
        .and_then(|s| s.parse().ok())
        .ok_or_else(|| ParseError::BadLine(line.into()))
}

/// Reject non-finite geometry (`nan`/`inf` parse as `f32` but are never a
/// legitimate authored backdrop rect).
fn finite(v: f32, line: &str) -> Result<f32, ParseError> {
    if v.is_finite() {
        Ok(v)
    } else {
        Err(ParseError::BadLine(line.into()))
    }
}

/// Cap on undo steps (bounded memory; snapshots are cheap but not infinite).
/// Mirrors `ph2d-vec-edit::History::HISTORY_CAP`.
const HISTORY_CAP: usize = 256;

/// Undo/redo of the Motion Nodes document by **whole-document snapshot** (mold of
/// `ph2d-vec-edit::History`). Cheap: [`MotionDoc`] is `Clone`. Usage: `begin` at
/// the start of an interaction (Down), `commit_if_changed` at the end (Up) → only
/// becomes a step if the doc actually changed; or `push_undo(pre)` directly for an
/// atomic op (add/delete/connect) that already knows it mutated.
#[derive(Default)]
pub struct MotionHistory {
    undo: Vec<MotionDoc>,
    redo: Vec<MotionDoc>,
    pending: Option<MotionDoc>,
}

impl MotionHistory {
    pub fn new() -> Self {
        Self::default()
    }

    /// Tentative snapshot of the CURRENT state, before an interaction that may
    /// mutate. A second `begin` before `commit` replaces the pending snapshot.
    pub fn begin(&mut self, doc: &MotionDoc) {
        self.pending = Some(doc.clone());
    }

    /// Close the interaction: if the doc differs from the `begin` snapshot it
    /// becomes an undo step (and clears redo). If nothing changed, the snapshot is
    /// discarded (does not pollute the history).
    pub fn commit_if_changed(&mut self, doc: &MotionDoc) {
        if let Some(pre) = self.pending.take()
            && &pre != doc
        {
            self.push_undo(pre);
        }
    }

    /// Push a pre-state straight onto the undo stack (atomic op that already knows
    /// it mutated). Clears redo.
    pub fn push_undo(&mut self, pre: MotionDoc) {
        if self.undo.len() >= HISTORY_CAP {
            self.undo.remove(0);
        }
        self.undo.push(pre);
        self.redo.clear();
    }

    pub fn can_undo(&self) -> bool {
        !self.undo.is_empty()
    }

    pub fn can_redo(&self) -> bool {
        !self.redo.is_empty()
    }

    /// Undo: return the previous state; push `current` onto redo.
    pub fn undo(&mut self, current: &MotionDoc) -> Option<MotionDoc> {
        let prev = self.undo.pop()?;
        self.redo.push(current.clone());
        Some(prev)
    }

    /// Redo: return the next state; push `current` back onto undo.
    pub fn redo(&mut self, current: &MotionDoc) -> Option<MotionDoc> {
        let next = self.redo.pop()?;
        self.undo.push(current.clone());
        Some(next)
    }
}

/// Playback transport for the Motion Nodes timeline (Motion Nodes M0.T8).
///
/// **Not** part of the persisted [`MotionDoc`] — this is runtime playback state.
/// The playhead is derived from the integer `tick` (`tick × fixed_dt`), never
/// from accumulated wall-clock float, so motion time is deterministic (HR-5):
/// the shell advances `tick` by the number of fixed steps its `FixedStep`
/// produced this frame (the accumulator lives in `FixedStep`, not here).
#[derive(Copy, Clone, Debug, PartialEq, Eq, Default)]
pub struct MotionTransport {
    /// Current fixed tick (playhead = `tick × fixed_dt`). Monotonic during
    /// playback except at a loop wrap or an explicit scrub.
    pub tick: u64,
    /// Whether playback is advancing.
    pub playing: bool,
    /// Optional `[lo, hi)` loop range in ticks. When set and `tick` reaches
    /// `hi`, playback wraps back into the range.
    pub loop_range: Option<(u64, u64)>,
}

impl MotionTransport {
    pub fn new() -> Self {
        Self::default()
    }

    /// Playhead in seconds for a given fixed timestep (e.g. `1.0/60.0`).
    pub fn playhead(&self, fixed_dt: f64) -> f64 {
        self.tick as f64 * fixed_dt
    }

    pub fn play(&mut self) {
        self.playing = true;
    }

    pub fn pause(&mut self) {
        self.playing = false;
    }

    /// Flip play/pause; returns the new `playing` state.
    pub fn toggle(&mut self) -> bool {
        self.playing = !self.playing;
        self.playing
    }

    /// Jump to an explicit tick (scrub). Does not change `playing`.
    pub fn scrub_to(&mut self, tick: u64) {
        self.tick = tick;
    }

    /// Reset to tick 0 (does not change `playing`).
    pub fn reset(&mut self) {
        self.tick = 0;
    }

    /// Advance by `steps` fixed ticks **iff playing**. Wraps within
    /// [`Self::loop_range`] when set. No-op when paused or `steps == 0`.
    pub fn advance(&mut self, steps: u64) {
        if !self.playing || steps == 0 {
            return;
        }
        self.tick = self.tick.saturating_add(steps);
        if let Some((lo, hi)) = self.loop_range
            && hi > lo
            && self.tick >= hi
        {
            let span = hi - lo;
            self.tick = lo + (self.tick - lo) % span;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ph2d_nodegraph::graph::{Edge, Pos};

    fn sample() -> MotionDoc {
        let mut g = Graph::new();
        let grid = g.add_node("motion.grid");
        let clone = g.add_node("motion.clone");
        g.connect(Edge {
            from: (grid, 0),
            to: (clone, 0),
            delayed: false,
        })
        .unwrap();
        g.connect(Edge {
            from: (clone, 0),
            to: (clone, 1),
            delayed: true,
        })
        .unwrap();
        g.set_pos(grid, Pos { x: 10.0, y: 20.0 });
        g.set_pos(clone, Pos { x: 120.0, y: 20.0 });
        g.set_param(grid, "rows", 4.0);
        MotionDoc {
            graph: g,
            waypoints: vec![],
            backdrops: vec![
                Backdrop {
                    id: 1,
                    x: -20.0,
                    y: -10.0,
                    w: 300.0,
                    h: 160.0,
                    color: 2,
                    title: "Source Cluster".to_string(),
                },
                Backdrop {
                    id: 0,
                    x: 5.0,
                    y: 5.0,
                    w: 40.0,
                    h: 40.0,
                    color: 0,
                    title: String::new(),
                },
            ],
            base_z: 7,
        }
    }

    #[test]
    fn round_trip_preserves_graph_backdrops_and_base_z() {
        let doc = sample();
        let text = doc.to_text();
        let back = MotionDoc::from_text(&text).unwrap();
        // Whole-document equality (graph derives Clone/Debug/PartialEq now).
        // Backdrops compare set-wise: to_text sorts by id, so `back` is id-sorted.
        let mut expect = doc.clone();
        expect.backdrops.sort_by_key(|b| b.id);
        assert_eq!(expect, back);
    }

    #[test]
    fn backdrop_section_is_below_layout() {
        let text = sample().to_text();
        let layout = text.find("[layout]").expect("layout section");
        let backdrop = text.find("[backdrop]").expect("backdrop section");
        assert!(layout < backdrop);
    }

    #[test]
    fn title_with_spaces_round_trips_and_empty_title_ok() {
        let doc = sample();
        let back = MotionDoc::from_text(&doc.to_text()).unwrap();
        let by_id = |id: u32| back.backdrops.iter().find(|b| b.id == id).unwrap();
        assert_eq!(by_id(1).title, "Source Cluster");
        assert_eq!(by_id(0).title, "");
    }

    #[test]
    fn graph_only_text_loads_with_defaults() {
        // A bare graph dump (no [backdrop] section) still loads.
        let mut g = Graph::new();
        g.add_node("motion.grid");
        let text = ph2d_nodegraph::format::to_text(&g);
        let doc = MotionDoc::from_text(&text).unwrap();
        assert_eq!(doc.base_z, 0);
        assert!(doc.backdrops.is_empty());
        assert_eq!(doc.graph, g);
    }

    #[test]
    fn serialization_is_deterministic() {
        let doc = sample();
        assert_eq!(
            doc.to_text(),
            MotionDoc::from_text(&doc.to_text()).unwrap().to_text()
        );
    }

    #[test]
    fn duplicate_backdrop_id_is_rejected() {
        let text = "v1\n[layout]\n[backdrop]\nz 0\nb 0 0 0 1 1 0 A\nb 0 0 0 1 1 0 B\n";
        assert!(matches!(
            MotionDoc::from_text(text),
            Err(ParseError::BadLine(_))
        ));
    }

    #[test]
    fn non_finite_backdrop_rect_is_rejected() {
        let text = "v1\n[layout]\n[backdrop]\nb 0 nan 0 1 1 0 A\n";
        assert!(matches!(
            MotionDoc::from_text(text),
            Err(ParseError::BadLine(_))
        ));
    }

    #[test]
    fn history_undo_redo_cycle() {
        let mut hist = MotionHistory::new();
        let v0 = MotionDoc::new();
        let mut v1 = v0.clone();
        v1.base_z = 3;

        hist.begin(&v0);
        hist.commit_if_changed(&v1); // v0 -> v1: a real change
        assert!(hist.can_undo());
        assert!(!hist.can_redo());

        let restored = hist.undo(&v1).unwrap();
        assert_eq!(restored, v0);
        assert!(hist.can_redo());

        let redone = hist.redo(&v0).unwrap();
        assert_eq!(redone, v1);
    }

    #[test]
    fn commit_without_change_is_no_op() {
        let mut hist = MotionHistory::new();
        let v = sample();
        hist.begin(&v);
        hist.commit_if_changed(&v); // identical -> no step
        assert!(!hist.can_undo());
    }

    #[test]
    fn transport_playhead_is_deterministic_from_tick() {
        let mut t = MotionTransport::new();
        t.tick = 120;
        assert!((t.playhead(1.0 / 60.0) - 2.0).abs() < 1e-9); // 120 ticks @ 60Hz = 2s
    }

    #[test]
    fn transport_advances_only_when_playing() {
        let mut t = MotionTransport::new();
        t.advance(5); // paused
        assert_eq!(t.tick, 0);
        t.play();
        t.advance(5);
        assert_eq!(t.tick, 5);
        t.pause();
        t.advance(5);
        assert_eq!(t.tick, 5);
    }

    #[test]
    fn transport_loop_range_wraps() {
        let mut t = MotionTransport {
            tick: 8,
            playing: true,
            loop_range: Some((4, 10)), // span 6
        };
        t.advance(4); // 8 -> 12 -> wrap to 4 + (12-4)%6 = 4 + 2 = 6
        assert_eq!(t.tick, 6);
    }

    #[test]
    fn transport_scrub_and_reset() {
        let mut t = MotionTransport::new();
        t.play();
        t.scrub_to(42);
        assert_eq!(t.tick, 42);
        assert!(t.playing); // scrub does not change playing
        t.reset();
        assert_eq!(t.tick, 0);
    }
}
