//! **The warp SESSION** — the frozen source and the cumulative displacement that two tools share.
//!
//! A warp session is four frozen planes (pixels + `heights` + `covers` + `mats`) and one cumulative
//! displacement map. Every render is
//!
//! ```text
//! out[p] = sample(frozen_plane, p − disp[p])
//! ```
//!
//! — a **single resample from a pristine baseline**, which is what keeps a long gesture sharp (nothing
//! ever re-reads its own output) and what makes transport a **sum over the dab list instead of a
//! product**. The second property is why this type has two owners now.
//!
//! ## Two tools, one session type, one door
//!
//! [`super::DeformState`] (Reshape: Push · Twist · Pinch · Wrinkle · Fold · Reconstruct) was the first
//! owner. The **Smear** is the second: it used to lift-and-blend pixels per dab, which decays
//! geometrically over a stroke and delivered a one-texel filament instead of a trail with mass — see
//! `ph2d_painter_brush::smear_field` for the measurement and the law.
//!
//! They share the **type** and the render door ([`super::relief`]'s `warp_render_relief`), never an
//! instance's lifetime: `PaintMode` is exclusive, so at most one tool has a live session at a time, and
//! each owns its own lifecycle —
//!
//! * **Deform** opens at `warp_begin` and closes at Apply / Reset / temperament switch: one session spans
//!   many strokes, which is what lets Reconstruct un-warp work from several strokes ago.
//! * **Smear** opens at pen-down and closes at pen-up: one session per stroke. It has no Apply or Reset
//!   to close it, and a stroke's result must become the next stroke's baseline (the knife edits the layer
//!   in place, as it always has).
//!
//! **The invariant that keeps those two rules from meeting:** leaving a session-owning mode ends the
//! session. That is enforced in `set_paint_tool_mode` and pinned by a gate — without it, a `disp` from
//! one tool would be read as the other's, and the frozen `pre` would describe a canvas that no longer
//! exists.
//!
//! `affect_relief` sits here rather than on either tool because it answers a question about the session
//! ("does the body ride this displacement?"), and the door that reads it is shared. It is a knob, not
//! document state, so — unlike the other seven — it is not captured by undo.

use super::super::Region;
use crate::tool::PainterTool;
use std::sync::Arc;

/// The frozen baselines + cumulative displacement of one warp gesture. See the module docs for who owns
/// an instance and when.
pub(crate) struct WarpSession {
    /// Pristine pixels at session start — Reconstruct fades back to this, Reset restores it wholesale,
    /// and every render resamples it exactly once.
    pub(crate) pre: Arc<Vec<u8>>,
    /// **Affect Relief**: does the body ride this displacement? Paint is a substance — what moves takes
    /// its thickness with it (doc 18 §5's exception). Off = warp the colour only. A knob, not document
    /// state, so it is not snapshotted.
    pub(crate) affect_relief: bool,
    /// The impasto height plane frozen at session start, beside [`Self::pre`]. Frozen for EVERY session on
    /// a relief-bearing layer whatever the toggle says: the artist can flip it mid-session, and Reset must
    /// give the body back unconditionally — asking the CURRENT toggle what to restore asks the wrong
    /// question (the sculpt session learned this rule first). Empty on a layer with no relief.
    pub(crate) pre_h: Arc<Vec<f32>>,
    /// The frozen paint coverage (see [`Self::pre_h`]).
    pub(crate) pre_cover: Arc<Vec<u8>>,
    /// The frozen per-pixel material (see [`Self::pre_h`]).
    pub(crate) pre_mats: Arc<Vec<ph2d_painter_brush::material::MaterialBytes>>,
    /// The layer the frozen planes describe. Relief lives PER LAYER (unlike `pre`, which shadows the
    /// active canvas), so a render must refuse to write planes captured from a layer that is no longer
    /// active — entity bits are recycled, and the wrong layer must not wear the old layer's body.
    pub(crate) relief_layer: Option<crate::tool::RtLayerId>,
    /// A session is live (`pre` captured).
    pub(crate) active: bool,
    /// **Session cumulative displacement** (`[dx, dy]` px per texel) — the source of truth. Contributors
    /// ADD to it (Deform's fields, the Smear's dabs); **Reconstruct REDUCES it** toward zero (pixels slide
    /// BACK to where they started — a real un-warp, not a cross-fade); Reset zeroes it. Sized `w·h`,
    /// allocated at session start, empty when idle. `Arc`-shared so the undo snapshot captures it cheaply
    /// (CoW).
    pub(crate) disp: Arc<Vec<[f32; 2]>>,
    /// How far the BODY rides along `disp`, as a fraction of how far the pixels ride (`1.0` = together).
    ///
    /// There is still exactly ONE displacement map — this is not a second transport, it is the single
    /// door's per-consumer scale, and that distinction is the whole architecture. Deform leaves it at
    /// `1.0` and gates the body with the **Affect Relief** checkbox instead; the Smear sets it from the
    /// **Plow** slider, whose shipped meaning is precisely *"how much of the paint's body does the knife
    /// take with it"*. `1.0` (the default since `e1fa546b`) is the coherent case Enio asked for — pigment
    /// and thickness moving as one substance; `0.0` is the old knife that spread colour and left the body
    /// behind, still available to anyone who wants it.
    pub(crate) relief_disp_scale: f32,
}

impl Default for WarpSession {
    fn default() -> Self {
        Self {
            pre: Arc::new(Vec::new()),
            affect_relief: true, // substance by default: warping paint moves its body too
            pre_h: Arc::new(Vec::new()),
            pre_cover: Arc::new(Vec::new()),
            pre_mats: Arc::new(Vec::new()),
            relief_layer: None,
            active: false,
            disp: Arc::new(Vec::new()),
            relief_disp_scale: 1.0,
        }
    }
}

impl PainterTool {
    /// Start a warp session if none is live: snapshot the current pixels as the pristine `pre` (a cheap
    /// `Arc` clone — the first write copies-on-write, so `pre` stays frozen at session start) and allocate
    /// a zeroed displacement map. No-op if a session is already live, so a multi-stroke owner (Deform)
    /// accumulates into one map.
    ///
    /// Returns `true` when a session is live afterwards — a caller that cannot proceed without one (the
    /// Smear route) checks it rather than assuming, because an unsized canvas produces no session.
    pub(crate) fn ensure_warp_session(&mut self) -> bool {
        let (w, h) = self.source_size;
        let n = (w as usize) * (h as usize);
        if n == 0 {
            return false;
        }
        if !self.paint.warp.active || self.paint.warp.pre.len() != n * 4 {
            self.paint.warp.pre = Arc::clone(&self.canvas_rgba);
            self.paint.warp.disp = Arc::new(vec![[0.0, 0.0]; n]);
            // The impasto planes freeze BESIDE the pixels — all of them, at the same instant, whatever the
            // Affect Relief toggle currently says (three refcount bumps; a layer with no relief pays
            // nothing). A session that froze the pixels at stroke 1 and the body at stroke 40 would
            // reconstruct a canvas that never existed.
            let active_layer = self.layers.active();
            let planes = active_layer.and_then(|l| {
                let hgt = self.heights.get(&l).filter(|v| v.len() == n)?;
                Some((
                    Arc::clone(hgt),
                    self.covers.get(&l).filter(|v| v.len() == n).cloned(),
                    self.mats.get(&l).filter(|v| v.len() == n).cloned(),
                ))
            });
            if let Some((hgt, cov, mat)) = planes {
                self.paint.warp.pre_h = hgt;
                self.paint.warp.pre_cover = cov.unwrap_or_default();
                self.paint.warp.pre_mats = mat.unwrap_or_default();
                self.paint.warp.relief_layer = active_layer;
            } else {
                self.paint.warp.pre_h = Arc::new(Vec::new());
                self.paint.warp.pre_cover = Arc::new(Vec::new());
                self.paint.warp.pre_mats = Arc::new(Vec::new());
                self.paint.warp.relief_layer = None;
            }
            self.paint.warp.active = true;
        }
        self.paint.warp.pre.len() == n * 4 && self.paint.warp.disp.len() == n
    }

    /// End the warp session and free its buffers. The displacement is NOT in the undo snapshot's
    /// restore path, so a restore must drop it rather than re-apply a stale warp.
    ///
    /// `affect_relief` deliberately survives: it is a knob the artist set, not session state.
    pub(crate) fn end_warp_session(&mut self) {
        self.paint.warp.active = false;
        self.paint.warp.pre = Arc::new(Vec::new());
        self.paint.warp.disp = Arc::new(Vec::new());
        self.paint.warp.pre_h = Arc::new(Vec::new());
        self.paint.warp.pre_cover = Arc::new(Vec::new());
        self.paint.warp.pre_mats = Arc::new(Vec::new());
        self.paint.warp.relief_layer = None;
    }

    /// Snapshot the warp session for the undo model (the buffers are `tool::paint`-private, so the general
    /// `snapshot_model` reaches them through here — mirror of `sculpt_for_snapshot`). The frozen relief
    /// planes ride it for the same reason `disp` does (Enio 2026-07-04): undoing a stroke must leave
    /// Reconstruct able to un-warp the REST — of the body as much as of the colour.
    pub(crate) fn warp_for_snapshot(&self) -> crate::undo::WarpSnap {
        crate::undo::WarpSnap {
            disp: Arc::clone(&self.paint.warp.disp),
            pre: Arc::clone(&self.paint.warp.pre),
            pre_h: Arc::clone(&self.paint.warp.pre_h),
            pre_cover: Arc::clone(&self.paint.warp.pre_cover),
            pre_mats: Arc::clone(&self.paint.warp.pre_mats),
            relief_layer: self.paint.warp.relief_layer,
            active: self.paint.warp.active,
        }
    }

    /// Reinstate the warp session from an undo model — rolls the warp back in lock-step with the pixels so
    /// Reconstruct stays usable after an undo.
    pub(crate) fn restore_warp(&mut self, s: crate::undo::WarpSnap) {
        self.paint.warp.disp = s.disp;
        self.paint.warp.pre = s.pre;
        self.paint.warp.pre_h = s.pre_h;
        self.paint.warp.pre_cover = s.pre_cover;
        self.paint.warp.pre_mats = s.pre_mats;
        self.paint.warp.relief_layer = s.relief_layer;
        self.paint.warp.active = s.active;
    }

    /// Re-render `bbox` of the CANVAS from the session's frozen pixels through the total displacement, and
    /// carry the body through the same map. The colour half of the render both owners share; the body half
    /// is `warp_render_relief`, the one door.
    ///
    /// With `disp = 0` everywhere each `dst` resolves to itself → **byte-identical**.
    pub(crate) fn warp_render_from_session(&mut self, bbox: Region) {
        let (w, h) = self.source_size;
        let n = (w as usize) * (h as usize);
        if self.paint.warp.pre.len() != n * 4 || self.paint.warp.disp.len() != n {
            return;
        }
        let src = Arc::clone(&self.paint.warp.pre);
        let disp = Arc::clone(&self.paint.warp.disp);
        let buf = super::super::plane_fork::fork_canvas(
            &mut self.canvas_rgba,
            &self.undo.write_state,
            self.source_size.0,
        );
        for ry in 0..bbox.h {
            let dy = bbox.y + ry;
            for rx in 0..bbox.w {
                let dx = bbox.x + rx;
                let gi = (dy * w + dx) as usize;
                let d = disp[gi];
                let px =
                    super::apply::bilinear_clamped(&src, w, h, dx as f32 - d[0], dy as f32 - d[1]);
                let b = gi * 4;
                buf[b..b + 4].copy_from_slice(&px);
            }
        }
        self.warp_render_relief(bbox);
    }
}
