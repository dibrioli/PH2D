//! [`MotionPath`] — the **trajectory** a `PropKind::Position` binding follows.
//!
//! The spatial half of [ADR-0141]. The temporal half is the ordinary scalar
//! [`crate::Track`] the binding already had, whose value is **distance along this
//! path** — so the graph editor, weighted tangents, the speed graph and roving all
//! keep working on a Position track without knowing a path exists.
//!
//! # The anchors ARE the keys
//!
//! Anchor `i` of this path is key `i` of the track, exactly as in After Effects: a
//! Position keyframe is a point on the motion path, and there is no way to have one
//! without the other. What the key *stores* is [`MotionPath::arclen_at`] — a
//! **derived** number, not an authored one.
//!
//! That is the whole reason [`MotionPath`] owns its own arc-length table and the
//! reason every mutation goes through [`MotionPath::edit`]: moving one anchor moves
//! the distance of every anchor **after** it, so the geometry and the numbers the
//! track holds have to be rewritten in the SAME operation. Two doors here would let
//! a path and its own track disagree about where the object is
//! ([[feedback_derived_coordinate_seed_must_match_sample]]).
//!
//! # No cap on anchors, deliberately
//!
//! A path has exactly as many anchors as its track has keys, and keys have no cap.
//! A `MAX_ANCHORS` would be a limit on the number of *keyframes*, invented here,
//! measured nowhere.
//!
//! [ADR-0141]: ../../../docs/architecture/decisions/0141-timeline-position-is-one-2d-channel-and-separate-axes-are-a-mode.md

use ph2d_arclen::{Cubic, arclen_to, inv_arclen, point_at, tangent_at};
use serde::{Deserialize, Serialize};

/// How many uniform samples per segment [`MotionPath::project`] takes before it
/// refines. Distance-squared to a cubic is a degree-6 polynomial and can have
/// several local minima, so the coarse pass exists to pick the right basin — the
/// refinement below only sharpens the one it lands in.
const PROJECT_SAMPLES: usize = 16;

/// Refinement steps of the ternary search inside the bracket the coarse pass chose.
/// Each step keeps 2/3 of the interval, so 24 steps shrink a `1/16` bracket by
/// `(2/3)^24 ≈ 1e-4`, i.e. well under a screen pixel on any path an artist draws.
const PROJECT_REFINE: usize = 24;

/// **The three handle types an anchor's tangents can carry** — the
/// Corner/Smooth/Symmetric trio of Inkscape/Illustrator, mirrored from the vector
/// module's `ph2d_vec_scene::VertexKind` (the repo has one answer for "what are a
/// bezier point's handle modes"). It decides how the OPPOSITE handle reacts when the
/// artist drags one, and what [`MotionPath::retype_anchor`] rebuilds on a convert.
///
/// ⚠️ **Orthogonal to [`PathAnchor::auto`].** An `auto` anchor derives BOTH handles
/// from its neighbours every time they move (After Effects' Auto Bezier), so its kind
/// is dormant until the artist shapes it — at which point `auto` clears and the kind
/// takes over the drag. There is no "Auto" variant here on purpose: `auto` is the
/// birth default a fresh key wears, and picking any of these three is the deliberate
/// act that ends it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum TangentKind {
    /// A cusp: the two handles are **independent** — dragging one leaves the other
    /// exactly where it was, so the curve can kink at this anchor.
    Corner,
    /// Colinear tangents (the passage through the anchor stays smooth), but the
    /// opposite handle keeps its **own length** — After Effects' *Continuous Bezier*.
    /// The default a shaped anchor wears, and the byte-identical behaviour this door
    /// had before the kinds existed.
    #[default]
    Smooth,
    /// Colinear **and equal length**: dragging one handle mirrors the other through
    /// the anchor. The most constrained of the three.
    Symmetric,
}

/// One authored point the object passes through, with the two spatial handles that
/// shape the curve on either side of it.
///
/// Handles are **relative to the anchor**, the same convention as
/// `ph2d_vec_scene::VecVertex` — the repo has one answer for "what is a bezier
/// vertex" and this is it. Coordinates are `f32` because they are world positions
/// that end up in a `Transform`: storing `f64` here would claim precision the
/// consumer throws away.
#[derive(Debug, Clone, Copy, PartialEq, Default, Serialize, Deserialize)]
pub struct PathAnchor {
    /// Where the object is when its track's value equals this anchor's distance.
    pub anchor: [f32; 2],
    /// Incoming handle, relative to `anchor`.
    pub in_handle: [f32; 2],
    /// Outgoing handle, relative to `anchor`.
    pub out_handle: [f32; 2],
    /// `true` while the handles are still whatever [`MotionPath::auto_smooth`] gave
    /// them — i.e. the artist has not shaped this corner by hand.
    ///
    /// It is what lets a NEW key re-smooth its neighbours without destroying work:
    /// re-smoothing an anchor someone shaped would throw that shape away, and never
    /// re-smoothing leaves a visible kink on a curve that was smooth a moment ago.
    /// After Effects makes the same distinction (Auto Bezier vs a keyframe you
    /// touched), and it cannot be *derived* — a hand-shaped anchor can coincidentally
    /// sit on the auto handles.
    pub auto: bool,
    /// Which of the three [`TangentKind`]s this anchor's handles obey when shaped.
    /// Dormant while `auto` (the handles are re-derived from neighbours regardless);
    /// it governs the drag and the convert once the artist has touched the anchor.
    pub kind: TangentKind,
}

impl PathAnchor {
    /// An anchor with both handles collapsed onto it — a corner. The Auto Bezier
    /// smoothing an artist expects on a fresh key is [`MotionPath::auto_smooth`],
    /// applied by whoever authors the anchor.
    #[must_use]
    pub const fn corner(anchor: [f32; 2]) -> Self {
        Self {
            anchor,
            in_handle: [0.0, 0.0],
            out_handle: [0.0, 0.0],
            auto: false,
            kind: TangentKind::Corner,
        }
    }
}

/// The trajectory: a chain of cubic segments through authored anchors, plus the
/// arc-length table that turns "distance travelled" into a point on it.
///
/// **Serialised as its anchors alone.** The table is derived, and a derived number
/// in a file is a second copy that can disagree with the first; `serde(from/into)`
/// routes every load through the one constructor that rebuilds it, so the table
/// cannot be stale and cannot be forged.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(from = "Vec<PathAnchor>", into = "Vec<PathAnchor>")]
pub struct MotionPath {
    anchors: Vec<PathAnchor>,
    /// `starts[i]` = distance from the path's origin to `anchors[i]`. Length always
    /// equals `anchors.len()`; `starts[0]` is `0.0` and the last entry is the total.
    starts: Vec<f64>,
}

impl From<Vec<PathAnchor>> for MotionPath {
    fn from(anchors: Vec<PathAnchor>) -> Self {
        Self::new(anchors)
    }
}

impl From<MotionPath> for Vec<PathAnchor> {
    fn from(p: MotionPath) -> Self {
        p.anchors
    }
}

impl MotionPath {
    /// A path through `anchors`, with its arc-length table built.
    #[must_use]
    pub fn new(anchors: Vec<PathAnchor>) -> Self {
        let mut p = Self {
            anchors,
            starts: Vec::new(),
        };
        p.rebuild();
        p
    }

    /// The authored anchors. There is no mutable counterpart on purpose — see
    /// [`MotionPath::edit`].
    #[must_use]
    pub fn anchors(&self) -> &[PathAnchor] {
        &self.anchors
    }

    /// How many anchors (= how many keys the track carries).
    #[must_use]
    pub fn len(&self) -> usize {
        self.anchors.len()
    }

    /// `true` when there is no trajectory at all.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.anchors.is_empty()
    }

    /// Total length of the trajectory, in world units. `0.0` for an empty path and
    /// for a path whose anchors all coincide.
    #[must_use]
    pub fn length(&self) -> f64 {
        self.starts.last().copied().unwrap_or(0.0)
    }

    /// **The number key `i` must hold**: the distance from the origin of the path to
    /// anchor `i`.
    #[must_use]
    pub fn arclen_at(&self, i: usize) -> Option<f64> {
        self.starts.get(i).copied()
    }

    /// **THE DOOR.** Every mutation of the anchor list runs through here, and the
    /// arc-length table is rebuilt before the caller gets control back — so a path
    /// is never observable in a state where its geometry and its distances
    /// disagree.
    ///
    /// The closure returns whatever the caller needs; the rebuild is not optional
    /// and not the caller's to remember. A gate holds this file to exactly one
    /// mutable borrow of `anchors`, which is this one.
    pub fn edit<R>(&mut self, f: impl FnOnce(&mut Vec<PathAnchor>) -> R) -> R {
        let r = f(&mut self.anchors);
        self.rebuild();
        r
    }

    /// **Re-derive the Auto Bezier handles of every anchor still marked `auto`.**
    ///
    /// An auto handle is a function of the two NEIGHBOURS, so inserting, removing or
    /// moving an anchor invalidates the handles on either side of it. Anchors the
    /// artist shaped ([`PathAnchor::auto`] `false`) are left exactly as they are — the
    /// whole point of the flag.
    ///
    /// Runs through [`MotionPath::edit`], so the arc-length table follows.
    pub fn resmooth_auto(&mut self) {
        self.edit(|anchors| {
            let pts: Vec<[f32; 2]> = anchors.iter().map(|a| a.anchor).collect();
            for (i, a) in anchors.iter_mut().enumerate() {
                if !a.auto {
                    continue;
                }
                *a = Self::auto_smooth(
                    (i > 0).then(|| pts[i - 1]),
                    pts[i],
                    (i + 1 < pts.len()).then(|| pts[i + 1]),
                );
            }
        });
    }

    /// Move anchor `i` (or reshape its handles). Returns `false` if there is no
    /// such anchor.
    pub fn set_anchor(&mut self, i: usize, a: PathAnchor) -> bool {
        self.edit(|anchors| match anchors.get_mut(i) {
            Some(slot) => {
                *slot = a;
                true
            }
            None => false,
        })
    }

    /// **Change anchor `i`'s [`TangentKind`], reshaping its handles to obey it** — the
    /// convert that a right-click on the anchor drives, mirroring the vector module's
    /// `ph2d_vec_scene::retype_vertex`.
    ///
    /// - `Corner` keeps the handles exactly where they are (they only become
    ///   *independent* from here on — nothing moves on screen);
    /// - `Smooth`/`Symmetric` make both handles colinear along the current tangent
    ///   (`out − in`, normalised). `Smooth` keeps each handle's own length; `Symmetric`
    ///   gives both the **average** of the two lengths.
    /// - A straight cusp (both handles collapsed onto the anchor) has no tangent of its
    ///   own, so it borrows the [`MotionPath::auto_smooth`] direction from its
    ///   neighbours — the same ⅓-of-the-gap Auto Bezier a fresh key is born with.
    ///
    /// ⚠️ **Marks the anchor shaped** (`auto = false`): choosing a type is a deliberate
    /// act, and leaving it `auto` would let the next neighbour edit re-derive the very
    /// handles the artist just asked for. Returns `false` if there is no anchor `i`.
    /// Runs through [`MotionPath::edit`], so the arc-length table follows.
    pub fn retype_anchor(&mut self, i: usize, kind: TangentKind) -> bool {
        let n = self.anchors.len();
        if i >= n {
            return false;
        }
        let prev = (i > 0).then(|| self.anchors[i - 1].anchor);
        let next = (i + 1 < n).then(|| self.anchors[i + 1].anchor);
        self.edit(|anchors| {
            let at = anchors[i].anchor;
            let (mut in_rel, mut out_rel) = (anchors[i].in_handle, anchors[i].out_handle);
            let a = &mut anchors[i];
            a.auto = false;
            a.kind = kind;
            if kind == TangentKind::Corner {
                // The handles stay put; only the constraint changes.
                return;
            }
            // A straight cusp has no tangent to align to → take the neighbours' one.
            let li = in_rel[0].hypot(in_rel[1]);
            let lo = out_rel[0].hypot(out_rel[1]);
            if li <= f32::EPSILON && lo <= f32::EPSILON {
                let s = Self::auto_smooth(prev, at, next);
                in_rel = s.in_handle;
                out_rel = s.out_handle;
            }
            // Colinear along `out − in`; empty tangent (both handles equal) leaves them.
            let d = [out_rel[0] - in_rel[0], out_rel[1] - in_rel[1]];
            let dn = d[0].hypot(d[1]);
            if dn <= f32::EPSILON {
                return;
            }
            let tan = [d[0] / dn, d[1] / dn];
            let (li, lo) = (in_rel[0].hypot(in_rel[1]), out_rel[0].hypot(out_rel[1]));
            let (ti, to) = if kind == TangentKind::Symmetric {
                let m = 0.5 * (li + lo);
                (m, m)
            } else {
                (li, lo)
            };
            a.in_handle = [-tan[0] * ti, -tan[1] * ti];
            a.out_handle = [tan[0] * to, tan[1] * to];
        });
        true
    }

    /// Insert an anchor at `i` (clamped to the end).
    pub fn insert_anchor(&mut self, i: usize, a: PathAnchor) {
        self.edit(|anchors| anchors.insert(i.min(anchors.len()), a));
    }

    /// **Insert an anchor at arc distance `d`, PRESERVING the curve's shape** — the "add
    /// point" of a vector editor, by a de Casteljau split. The segment containing `d` is
    /// cut into two cubics that together reproduce the original, so the curve does not move
    /// when it gains the point.
    ///
    /// The new anchor is born SHAPED (with the split handles), and its two neighbours have
    /// their adjacent handle adjusted (the split requires it) and are marked shaped too —
    /// otherwise a later [`MotionPath::resmooth_auto`] would re-derive them and undo the
    /// split. Returns the new anchor's index, or `None` on a path of fewer than two anchors.
    /// Runs through [`MotionPath::edit`], so the arc-length table follows.
    pub fn split_segment_at(&mut self, d: f64) -> Option<usize> {
        if self.anchors.len() < 2 {
            return None;
        }
        let d = d.clamp(0.0, self.length());
        let i = self.segment_index(d);
        let c = self.segment(i);
        let u = inv_arclen(&c, d - self.starts[i]);
        // de Casteljau at `u`: the split point S and the six control points that make the
        // two sub-cubics equal the original. All in the arc engine's `f64`.
        let lerp =
            |a: [f64; 2], b: [f64; 2], t: f64| [a[0] + (b[0] - a[0]) * t, a[1] + (b[1] - a[1]) * t];
        let (p0, p1, p2, p3) = (c[0], c[1], c[2], c[3]);
        let q0 = lerp(p0, p1, u);
        let q1 = lerp(p1, p2, u);
        let q2 = lerp(p2, p3, u);
        let r0 = lerp(q0, q1, u);
        let r1 = lerp(q1, q2, u);
        let s = lerp(r0, r1, u);
        let rel = |a: [f64; 2], b: [f64; 2]| [(a[0] - b[0]) as f32, (a[1] - b[1]) as f32];
        let new_anchor = PathAnchor {
            anchor: [s[0] as f32, s[1] as f32],
            in_handle: rel(r0, s),
            out_handle: rel(r1, s),
            auto: false,
            kind: TangentKind::Smooth,
        };
        let out_i = rel(q0, p0);
        let in_next = rel(q2, p3);
        self.edit(|anchors| {
            anchors[i].out_handle = out_i;
            anchors[i].auto = false;
            anchors[i + 1].in_handle = in_next;
            anchors[i + 1].auto = false;
            anchors.insert(i + 1, new_anchor);
        });
        Some(i + 1)
    }

    /// Which segment contains arc distance `d` — the index of the anchor it starts at.
    #[must_use]
    pub fn segment_of(&self, d: f64) -> usize {
        self.segment_index(d.clamp(0.0, self.length()))
    }

    /// Remove anchor `i`, returning it.
    pub fn remove_anchor(&mut self, i: usize) -> Option<PathAnchor> {
        self.edit(|anchors| (i < anchors.len()).then(|| anchors.remove(i)))
    }

    /// **Where the object is after travelling `s` along the path**, and which way it
    /// is heading.
    ///
    /// `s` outside `0..=length` saturates at the ends: a track key past the end of
    /// the trajectory means "at the end", not an error. `None` only when there is no
    /// path at all.
    #[must_use]
    pub fn at(&self, s: f64) -> Option<PathSample> {
        let first = *self.anchors.first()?;
        if self.anchors.len() == 1 {
            // A one-anchor path is a point. It has a position and no direction —
            // and `None` here is the same answer a cusp gives, for the same reason.
            return Some(PathSample {
                point: first.anchor,
                tangent: None,
            });
        }
        let s = s.clamp(0.0, self.length());
        let i = self.segment_index(s);
        let c = self.segment(i);
        let t = inv_arclen(&c, s - self.starts[i]);
        let p = point_at(&c, t);
        Some(PathSample {
            point: [p[0] as f32, p[1] as f32],
            tangent: tangent_at(&c, t).map(|d| [d[0] as f32, d[1] as f32]),
        })
    }

    /// **How far along the path the point nearest `p` sits** — the inverse of
    /// [`MotionPath::at`] for a point that is not necessarily on the curve.
    ///
    /// This is what a Position binding's `rest` means (the pose the object was left
    /// in, expressed in the track's own units), and what a click on the trajectory
    /// resolves to.
    ///
    /// Coarse sample per segment, then a ternary search inside the bracket that won
    /// — distance to a cubic is not unimodal, so the coarse pass picks the basin and
    /// only the refinement assumes smoothness inside it.
    #[must_use]
    pub fn project(&self, p: [f32; 2]) -> Option<f64> {
        if self.anchors.is_empty() {
            return None;
        }
        if self.anchors.len() == 1 {
            return Some(0.0);
        }
        let q = [f64::from(p[0]), f64::from(p[1])];
        let mut best = (f64::INFINITY, 0usize, 0.0f64);
        for i in 0..self.anchors.len() - 1 {
            let c = self.segment(i);
            let (mut bt, mut bd) = (0.0f64, f64::INFINITY);
            for k in 0..=PROJECT_SAMPLES {
                let t = k as f64 / PROJECT_SAMPLES as f64;
                let d = dist2(point_at(&c, t), q);
                if d < bd {
                    bd = d;
                    bt = t;
                }
            }
            let h = 1.0 / PROJECT_SAMPLES as f64;
            let (mut lo, mut hi) = ((bt - h).max(0.0), (bt + h).min(1.0));
            for _ in 0..PROJECT_REFINE {
                let m1 = lo + (hi - lo) / 3.0;
                let m2 = hi - (hi - lo) / 3.0;
                if dist2(point_at(&c, m1), q) < dist2(point_at(&c, m2), q) {
                    hi = m2;
                } else {
                    lo = m1;
                }
            }
            let t = 0.5 * (lo + hi);
            let d = dist2(point_at(&c, t), q);
            if d < best.0 {
                best = (d, i, t);
            }
        }
        Some(self.starts[best.1] + arclen_to(&self.segment(best.1), best.2))
    }

    /// **Auto Bezier** — the handle pair a smooth anchor gets when the artist has not
    /// shaped it, and the default a fresh Position key is born with (ADR-0141 §4,
    /// After Effects' default for a spatial keyframe).
    ///
    /// The handle direction is the chord between the neighbours (Catmull-Rom's
    /// tangent) and its length is a third of the neighbouring gap, which is what makes
    /// the curve pass through the anchors without the overshoot a full-length handle
    /// produces. An end anchor has one neighbour, so it aims at it.
    #[must_use]
    pub fn auto_smooth(prev: Option<[f32; 2]>, at: [f32; 2], next: Option<[f32; 2]>) -> PathAnchor {
        let (a, b) = (prev.unwrap_or(at), next.unwrap_or(at));
        let dir = [b[0] - a[0], b[1] - a[1]];
        let n = (dir[0] * dir[0] + dir[1] * dir[1]).sqrt();
        if n <= f32::EPSILON {
            // Sem vizinhos em que se apoiar (a primeira âncora de um caminho, ou dois
            // pontos coincidentes) as alças colapsam — mas a âncora continua **auto**.
            // ⚠️ Devolver [`PathAnchor::corner`] aqui a marcaria como MOLDADA, e
            // [`MotionPath::resmooth_auto`] passaria a saltá-la para sempre: o caminho
            // que o K constrói nasceria todo em quinas vivas, sem nada dizendo por quê.
            return PathAnchor {
                anchor: at,
                in_handle: [0.0, 0.0],
                out_handle: [0.0, 0.0],
                auto: true,
                kind: TangentKind::Smooth,
            };
        }
        let u = [dir[0] / n, dir[1] / n];
        let reach = |q: [f32; 2]| {
            let d = [q[0] - at[0], q[1] - at[1]];
            (d[0] * d[0] + d[1] * d[1]).sqrt() / 3.0
        };
        let (ri, ro) = (reach(prev.unwrap_or(at)), reach(next.unwrap_or(at)));
        PathAnchor {
            anchor: at,
            in_handle: [-u[0] * ri, -u[1] * ri],
            out_handle: [u[0] * ro, u[1] * ro],
            auto: true,
            kind: TangentKind::Smooth,
        }
    }

    /// The cubic between anchors `i` and `i + 1`, in the arc engine's `f64`.
    fn segment(&self, i: usize) -> Cubic {
        let a = self.anchors[i];
        let b = self.anchors[i + 1];
        let pt = |p: [f32; 2]| [f64::from(p[0]), f64::from(p[1])];
        let off = |p: [f32; 2], h: [f32; 2]| [f64::from(p[0] + h[0]), f64::from(p[1] + h[1])];
        [
            pt(a.anchor),
            off(a.anchor, a.out_handle),
            off(b.anchor, b.in_handle),
            pt(b.anchor),
        ]
    }

    /// Which segment distance `s` falls in, clamped to a real segment.
    fn segment_index(&self, s: f64) -> usize {
        let last = self.anchors.len() - 2;
        self.starts
            .partition_point(|&x| x <= s)
            .saturating_sub(1)
            .min(last)
    }

    /// Recompute the arc-length table from the anchors. Private and unconditional:
    /// [`MotionPath::edit`] is the only caller.
    fn rebuild(&mut self) {
        self.starts.clear();
        self.starts.reserve(self.anchors.len());
        let mut acc = 0.0;
        for i in 0..self.anchors.len() {
            self.starts.push(acc);
            if i + 1 < self.anchors.len() {
                acc += ph2d_arclen::arclen(&self.segment(i));
            }
        }
    }
}

/// Where the object is, and which way it faces, at one distance along a path.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PathSample {
    /// The world position to write.
    pub point: [f32; 2],
    /// The unit tangent, or `None` at a cusp — where there is no direction, and
    /// inventing one is what produces the spike nobody can trace (the contract
    /// `ph2d_arclen::tangent_at` already states, honoured rather than re-decided).
    pub tangent: Option<[f32; 2]>,
}

fn dist2(a: [f64; 2], b: [f64; 2]) -> f64 {
    let (dx, dy) = (a[0] - b[0], a[1] - b[1]);
    dx * dx + dy * dy
}

#[cfg(test)]
#[path = "path_tests.rs"]
mod tests;
