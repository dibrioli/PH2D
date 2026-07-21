//! **Wet Paint** ([`PaintMode::WetPaint`]) — the `ph2d-wet-paint` fluid engine
//! as a paint mode (ADR-0134). The MODE is the master switch: no `BrushSpec`
//! flag exists on purpose (two doors to one question diverge — the Knife
//! precedent), so "off" means "any other mode", and the OFF contract is that
//! no other mode ever constructs a session (gated below).
//!
//! ## The session model (the watercolor wet-session, generalized)
//!
//! A session freezes the canvas (`base`, a shared `Arc` — O(1)) and owns an
//! engine grid sized to it; every composite re-renders `pigment OVER base`
//! into `canvas_rgba` for exactly the engine's dirty rect. The session spans
//! STROKES — the water stays live between pen-ups, which is the module's
//! whole point — and it is **display-state, not document-state**: the pixels
//! the artist sees are always IN `canvas_rgba`, so ending a session is the
//! bake (nothing to write), and the per-stroke undo capture at pen-down
//! already holds the look. The engine grid therefore stays OUT of
//! `ModelSnapshot` — a `GridSnapshot` per undo step would be ~14 f32 planes
//! per canvas (~235 MB at 2048², the ADR-0117 disease).
//!
//! What guards that stance is the **canvas-identity guard** (the watercolor's
//! `wet_session_canvas`, made eager): any foreign mutation — undo, layer
//! switch, fill, resize, another tool — swaps the `canvas_rgba` Arc, and the
//! next wet-paint touch (dab OR tick) sees `Arc::ptr_eq` fail and ends the
//! session. The tick half is load-bearing: the sim composites WITHOUT a
//! pen-down, so a lazy at-pen-down check would let a live session repaint
//! over a canvas the undo just restored.
//!
//! ## Coordinates and dab mapping
//!
//! Engine cells are 1-based with a pad ring: canvas pixel `(0..W-1)` maps to
//! cell `(1..W)` (the reference app's `view.toCell`: `x = px + 1`). Dabs go
//! through the engine's OWN §9 parameter mapping (`dispatch_pressure_dab`)
//! with exactly two host substitutions: pressure = `coverage / strength`
//! (the dab's real pressure response, rescaled to §8's ~0..10 range) and
//! radius = the dab's real `radius_px`. Colour is taken at stroke start
//! (per-dab Randomize colour is a W2 seam).
//!
//! ## What deliberately does NOT run in this mode (W1)
//!
//! The impasto height pass and every colour route — the engine owns the
//! deposit, and relief for paint that then FLOWS AWAY would be wrong twice.
//! And a deposit only happens inside a live freehand stroke: the shape
//! editors re-stamp their whole geometry every preview frame, which against
//! a non-idempotent fluid deposit would pile paint while the artist just
//! LOOKS (the I2 disease) — their integration is a W2 design, not a default.

use super::*;
use ph2d_wet_paint::painter::{Dirty, Engine};
use ph2d_wet_paint::render::{RenderLayer, render_pigment_only_region};

/// 40 Hz fixed-step (SPEC §5); at most 5 steps per frame, backlog dropped.
const WET_STEP_S: f32 = 1.0 / 40.0;
const WET_MAX_STEPS: usize = 5;

#[derive(Default)]
pub(crate) struct WetPaintState {
    /// The live session; `None` until the first Wet Paint dab lands.
    pub(super) session: Option<WetSession>,
    /// A live freehand paint GESTURE is open (`paint_begin` .. pen-up). This —
    /// not `paint.stroke` — is the deposit gate: the lifecycle `mem::take`s
    /// the stroke while stamping, and the shape editors' per-frame re-stamps
    /// never run `paint_begin` at all, so the flag is exactly "dabs are the
    /// artist's hand, once".
    pub(super) live_gesture: bool,
}

pub(super) struct WetSession {
    /// The engine (grid + sim + tuning), sized to the layer canvas.
    pub(super) engine: Engine,
    /// The canvas frozen at session start — every composite renders over THIS.
    /// `pub(super)` so [`PainterTool::wet_splat_gates`] can serve it as the
    /// alpha-lock's α reference (the frozen base the composite reads).
    pub(super) base: Arc<Vec<u8>>,
    /// The exact `canvas_rgba` Arc OUR last composite produced (or the one the
    /// session started from). A mismatch = foreign mutation = session over.
    canvas: Arc<Vec<u8>>,
    /// Session-persistent pigment scratch (`w*h*4`); the region render fully
    /// overwrites the rect it is asked for, so stale bytes outside are inert.
    pigment: Vec<u8>,
    /// Fixed-step accumulator for [`PainterTool::wetpaint_tick`].
    acc: f32,
    /// Per-LANE stroke state (one lane per symmetry copy / tile offset,
    /// matched geometrically): last dab centre (the chord source) + the last
    /// fresh-ink colour sent (Randomize change detector). Cleared per stroke.
    lanes: Vec<Lane>,
    /// A direct stroke is open in the engine (pen-down .. pen-up).
    stroke_open: bool,
}

/// One replication lane of the open stroke — see [`WetSession::lanes`].
struct Lane {
    pos: [f32; 2],
    ink: [f32; 3],
}

impl PainterTool {
    /// Whether the WET module owns this batch of dabs — the ONE routing
    /// question, asked by BOTH `stamp_dabs` (to bypass the snapshot/restore
    /// wrapper, which would kill the session — see W2.5) and
    /// `stamp_dabs_inner` (to enter the wet arm). Two copies of this
    /// condition would diverge, and the diverged half is a canvas gate that
    /// either leaks or kills the water.
    ///
    /// Wet Paint owns everything in its mode EXCEPT the eraser with no live
    /// session (W2.6): the wet eraser erases the FLUID, and with no session
    /// there is nothing wet — those dabs fall through to the normal eraser
    /// and erase the BAKED canvas instead (which is what is visibly there).
    pub(super) fn wet_owns_the_dabs(&self) -> bool {
        matches!(self.paint.paint_mode, PaintMode::WetPaint)
            && (!self.paint.eraser || self.paint.wetpaint.session.is_some())
    }

    /// The Wet Paint dab route ([`Self::stamp_dabs_inner`] arm). The dab list
    /// is already mirrored (Symmetry) and replicated (Tiling) — the engine
    /// sees exactly what the colour routes would have seen.
    pub(super) fn stamp_dabs_wetpaint(&mut self, dabs: &[Dab], brush: &BrushSpec) {
        let (w, h) = self.source_size;
        // Deposits only inside a live gesture (module doc, I2), and only for
        // the CUMULATIVE stroke methods — DragDot/Anchored/Line re-stamp a
        // moving shape every preview frame, which against a non-idempotent
        // fluid deposit would pile paint while the artist just looks (W2).
        let cumulative = matches!(
            brush.stroke_method,
            StrokeMethod::Dots | StrokeMethod::Airbrush | StrokeMethod::Space
        );
        if dabs.is_empty() || w == 0 || h == 0 || !self.paint.wetpaint.live_gesture || !cumulative {
            return;
        }
        self.wetpaint_guard();
        // The wet ERASER (W2.6) erases the FLUID and only the fluid: with no
        // live session there is nothing wet, and the routing predicate
        // (`wet_owns_the_dabs`) already sent those dabs to the normal
        // eraser — this early-out only covers the guard killing the session
        // between the route decision and here (the batch then does nothing).
        let erasing = self.paint.eraser;
        let fresh = self.paint.wetpaint.session.is_none();
        if fresh {
            if erasing {
                return;
            }
            self.paint.wetpaint.session = Some(WetSession {
                engine: Engine::new(w as usize, h as usize),
                base: Arc::clone(&self.canvas_rgba),
                canvas: Arc::clone(&self.canvas_rgba),
                pigment: vec![0u8; w as usize * h as usize * 4],
                acc: 0.0,
                lanes: Vec::new(),
                stroke_open: false,
            });
        }
        // Take the session out so the prep below can borrow `self.paint`'s
        // Shape state alongside it (disjoint in fact, not in the borrow
        // checker's eyes); restored before the composite.
        let mut taken = self.paint.wetpaint.session.take().expect("ensured above");
        let sess = &mut taken;
        // ── The artist's PAPER drives the engine's tooth (W2.7) ────────────
        // Seeded ONCE, at session birth: paper is SUBSTRATE — it does not
        // change under live water; a new session reads the current slot. The
        // law is the painter's own canvas-fixed paper sampling (the NEUTRAL
        // door `sample_tiled_rot_wrapped`, the same law the watercolor
        // substrate reads — one paper, never a second system). No slot armed
        // = the port's preset tile, byte-identical. Known v1 gap: the bitmap
        // Size seam-snap under sprite Tiling (`snap_slot_size`) is not
        // applied — procedurals wrap exactly; a bitmap paper may seam.
        if fresh && brush.paper.is_active() {
            let paper_tex = brush.paper;
            let paper_img = self.paint.paper_image.as_ref().map(|i| i.as_mask());
            let rot = ph2d_painter_brush::texture::angle_basis(paper_tex.angle_deg);
            let period = [
                if self.paint.tiling[0] { w as f32 } else { 0.0 },
                if self.paint.tiling[1] { h as f32 } else { 0.0 },
            ];
            sess.engine.seed_paper_with(&mut |px, py| {
                f64::from(ph2d_painter_brush::texture::sample_tiled_rot_wrapped(
                    &paper_tex,
                    px,
                    py,
                    paper_img.as_ref(),
                    rot,
                    period,
                ))
            });
        }
        // ── The dab's SILHOUETTE — the painter's, not the engine's ─────────
        // `silhouette_at` is the single source of the dab's shape (falloff ×
        // Shape image/procedural × flatten/rotate footprint); the engine's
        // internal falloff/footprint step aside per dab via the shaped door,
        // and the bristle texture stays as the fluid's default grain (W2.3b).
        let shape_image = self.paint.shape_image.as_ref().map(|i| i.as_mask());
        let shape_ramp_lut = (self.paint.shape_color_ramp_enabled
            && self.paint.shape_color_ramp_bw)
            .then_some(self.paint.shape_ramp_lut.as_slice());
        let shape_active = brush.shape_silhouette_active(shape_image.is_some());
        // The artist's GRAIN replaces the engine's bristle texture (W2.4).
        // The bristle IS the fluid's default grain, so an armed Grain slot
        // takes its place outright — two textures multiplying would
        // double-darken, and everywhere else in the app the Grain is THE
        // texture. The per-pixel law is `dab::grain_at`, the same single
        // door the colour route and the impasto height kernel call.
        let grain_image = self.paint.texture_image.as_ref().map(|i| i.as_mask());
        let grain_active = brush.texture.is_active();
        let groups = self.paint.dab_groups.clone();
        let mut dab_rng = super::tiling::DabRng::new(self.paint.tex_rng);
        let canvas_wh = [w as f32, h as f32];
        // The engine speaks sRGB **0..255** (its boot colour is `[50, 140, 210]` and the render
        // writes plane values straight through `clamp_u8`); the dab stores sRGB 0..1. Passing the
        // normalized value painted BLACK — Enio's W1 smoke, "a cor está preta".
        let ink = |c: [f32; 3]| {
            [
                f64::from(c[0]) * 255.0,
                f64::from(c[1]) * 255.0,
                f64::from(c[2]) * 255.0,
            ]
        };
        if !sess.stroke_open {
            sess.stroke_open = true;
            sess.lanes.clear();
            // The eraser is LANE-LESS (a direct grid op, no trail) — open one
            // direct stroke so the sim pauses under the gesture as usual.
            if erasing && let Some(d0) = dabs.first() {
                sess.engine.begin_direct_stroke(
                    0,
                    f64::from(d0.center[0]) + 1.0,
                    f64::from(d0.center[1]) + 1.0,
                );
            }
        }
        let strength = brush.strength.clamp(1e-3, 1.0);
        for (didx, d) in dabs.iter().enumerate() {
            let [x, y] = d.center;
            // LANE matching, geometric: the dab belongs to the lane whose
            // last position is within its own radius (consecutive dabs of one
            // copy are a spacing apart; other copies are far). A dab with no
            // lane in reach BEGINS one — a symmetry copy at stroke start, or
            // a Tiling wrap born mid-stroke at the sprite edge. Near a radial
            // centre the copies converge and may swap lanes; there their
            // positions coincide, so a swap deposits the same paint.
            // The ERASER skips the lanes entirely — every copy just erases
            // where it lands.
            let li = if erasing {
                0
            } else {
                let thr = d.radius_px.max(4.0);
                let mut best = thr * thr;
                let mut lane = None;
                for (i, l) in sess.lanes.iter().enumerate() {
                    let (ddx, ddy) = (x - l.pos[0], y - l.pos[1]);
                    let d2 = ddx * ddx + ddy * ddy;
                    if d2 <= best {
                        best = d2;
                        lane = Some(i);
                    }
                }
                match lane {
                    Some(i) => {
                        sess.engine.direct_segment(i, f64::from(best.sqrt()));
                        i
                    }
                    None => {
                        let i = sess.lanes.len();
                        // The DAB's colour, not the brush's: Randomize is
                        // already resolved per dab by the stroke engine
                        // (W2.2).
                        sess.engine.color = ink(d.color);
                        sess.engine
                            .begin_direct_stroke(i, f64::from(x) + 1.0, f64::from(y) + 1.0);
                        sess.lanes.push(Lane {
                            pos: d.center,
                            ink: d.color,
                        });
                        i
                    }
                }
            };
            // Per-dab fresh ink (Randomize): reload the lane's trail — a
            // brush dipped in new paint (see `Trail::set_base_color`).
            if !erasing && d.color != sess.lanes[li].ink {
                sess.engine.set_stroke_color(li, ink(d.color));
                sess.lanes[li].ink = d.color;
            }
            let b = ((d.coverage / strength).clamp(0.0, 1.0) as f64) * 10.0;
            // Per-dab silhouette closure — the impasto walk's exact recipe
            // (spec at the dab's radius → rotor → footprint → Shape basis in
            // the stroke frame), evaluated per engine cell (cell − 1 = px).
            let spec = BrushSpec {
                radius_px: d.radius_px,
                ..*brush
            };
            let rotor = spec.dab_rotor(d);
            let fp = spec.dab_footprint(rotor);
            let dab_index = didx;
            let tex_rng = dab_rng.enter(&groups, dab_index);
            let shape_basis = shape_active.then(|| {
                ph2d_painter_brush::texture::shape_basis(
                    &spec.shape,
                    &mut *tex_rng,
                    canvas_wh,
                    fp,
                    ph2d_painter_brush::texture::ShapeFrame::Stroke {
                        arc_len: d.arc_len,
                        unit_px: d.stroke_radius_px,
                    },
                )
            });
            let shape_input = shape_basis
                .as_ref()
                .map(|sb| ph2d_painter_brush::ShapeInput {
                    basis: sb,
                    image: shape_image.as_ref(),
                    ramp_lut: shape_ramp_lut,
                });
            // Grain basis AFTER the Shape basis, from the same RNG stream —
            // the colour route's resolve order (Shape before Grain), so the
            // random draws land where every other route puts them.
            let grain_basis = grain_active.then(|| {
                ph2d_painter_brush::texture::dab_basis(&spec.texture, &mut *tex_rng, canvas_wh, fp)
            });
            let inv_r = 1.0 / d.radius_px.max(0.01);
            let mut sil = |cx: i32, cy: i32| -> f64 {
                let px = i64::from(cx) - 1;
                let py = i64::from(cy) - 1;
                let ddx = (px as f32 + 0.5) - d.center[0];
                let ddy = (py as f32 + 0.5) - d.center[1];
                let t = fp.falloff_t(ddx * inv_r, ddy * inv_r);
                f64::from(ph2d_painter_brush::dab::silhouette_at(
                    &spec,
                    shape_input,
                    t,
                    px,
                    py,
                    d.center,
                    d.radius_px,
                ))
            };
            // Per-dab grain closure (armed slot only): replaces the bristle
            // sample inside the shaped stamp, cell − 1 = px like `sil`.
            let spec_ref = &spec;
            let gimg = grain_image.as_ref();
            let mut grain = grain_basis.as_ref().map(|gb| {
                move |cx: i32, cy: i32| -> f64 {
                    let px = i64::from(cx) - 1;
                    let py = i64::from(cy) - 1;
                    f64::from(ph2d_painter_brush::dab::grain_at(
                        spec_ref,
                        gb,
                        gimg,
                        px,
                        py,
                        d.center,
                        d.radius_px,
                    ))
                }
            });
            let grain_arg = grain.as_mut().map(|g| g as &mut dyn FnMut(i32, i32) -> f64);
            if erasing {
                // W2.6: the same §9 mapping, routed to the engine's ERASER —
                // silhouette and grain shape the erase exactly as they shape
                // the deposit.
                sess.engine.dispatch_pressure_dab_erase(
                    f64::from(x) + 1.0,
                    f64::from(y) + 1.0,
                    b,
                    f64::from(d.dir[0]),
                    f64::from(d.dir[1]),
                    f64::from(d.radius_px),
                    Some(&mut sil),
                    grain_arg,
                );
                continue;
            }
            sess.engine.dispatch_pressure_dab_lane(
                li,
                f64::from(x) + 1.0,
                f64::from(y) + 1.0,
                b,
                f64::from(d.dir[0]),
                f64::from(d.dir[1]),
                f64::from(d.radius_px),
                Some(&mut sil),
                grain_arg,
            );
            sess.lanes[li].pos = d.center;
        }
        self.paint.wetpaint.session = Some(taken);
        self.wetpaint_composite();
    }

    /// Pen-up: close the engine's direct stroke (the sim resumes). Called from
    /// `paint_end`; the session itself stays alive — the water is still wet.
    pub(super) fn wetpaint_stroke_end(&mut self) {
        self.paint.wetpaint.live_gesture = false;
        if let Some(sess) = self.paint.wetpaint.session.as_mut()
            && sess.stroke_open
        {
            sess.engine.end_direct_stroke();
            sess.stroke_open = false;
            sess.lanes.clear();
        }
    }

    /// Per-frame heartbeat: run the 40 Hz sim (paused while a stroke is down —
    /// the engine's own gate) and composite whatever it moved. No session = a
    /// true no-op (the OFF contract: not one byte is looked at).
    pub(super) fn wetpaint_tick(&mut self, dt_s: f32) {
        if self.paint.wetpaint.session.is_none() {
            return;
        }
        self.wetpaint_guard();
        let Some(sess) = self.paint.wetpaint.session.as_mut() else {
            return;
        };
        sess.acc += dt_s;
        let mut steps = 0;
        while sess.acc >= WET_STEP_S && steps < WET_MAX_STEPS {
            sess.acc -= WET_STEP_S;
            steps += 1;
            if sess.engine.sim_should_run() {
                sess.engine.step_simulation();
            }
        }
        // Clamp semantics: a stall never owes a burst of catch-up steps.
        sess.acc = sess.acc.min(WET_STEP_S);
        if steps > 0 {
            self.wetpaint_composite();
        }
    }

    /// End the session (mode switch / explicit teardown). The last composite
    /// is already in `canvas_rgba`, so ending IS the bake — the water just
    /// stops moving.
    pub(super) fn wetpaint_end_session(&mut self) {
        self.paint.wetpaint.session = None;
    }

    /// The canvas-identity guard (module doc): a foreign `canvas_rgba` swap
    /// ends the session before anything composites over restored pixels.
    fn wetpaint_guard(&mut self) {
        if let Some(sess) = &self.paint.wetpaint.session
            && !Arc::ptr_eq(&sess.canvas, &self.canvas_rgba)
        {
            self.paint.wetpaint.session = None;
        }
    }

    /// Composite the engine's dirty rect: pigment (straight alpha) OVER the
    /// frozen base, written into `canvas_rgba`, preview marked.
    ///
    /// Selection / protection / alpha-lock (W2.5) are enforced HERE, at the
    /// single place the module writes the canvas: the fluid flows wherever
    /// the SIM takes it, and the gated texels keep-lerp back to the frozen
    /// base (`splat_keep` — the watercolor composite's exact law, a hard
    /// guarantee independent of where the water reached). Alpha-lock pins
    /// the layer's α to the frozen base (colour deposits into existing
    /// paint; the silhouette never moves). All gates off = byte-identical.
    fn wetpaint_composite(&mut self) {
        let (w, h) = self.source_size;
        let (w, h) = (w as usize, h as usize);
        let (gate_sel, gate_prot, gate_alock) = self.wet_splat_gates();
        let Some(sess) = self.paint.wetpaint.session.as_mut() else {
            return;
        };
        // Engine dirty (1-based cells, inclusive) → clamped cell rect.
        let (cx0, cy0, cx1, cy1) = match sess.engine.take_dirty() {
            Dirty::Clean => return,
            Dirty::Full => (1, 1, w, h),
            Dirty::Rect { x0, y0, x1, y1 } => (
                (x0.max(1) as usize).min(w),
                (y0.max(1) as usize).min(h),
                (x1.max(1) as usize).min(w),
                (y1.max(1) as usize).min(h),
            ),
        };
        if cx1 < cx0 || cy1 < cy0 || sess.base.len() != w * h * 4 {
            return;
        }
        let layers: Vec<RenderLayer<'_>> = sess
            .engine
            .layers
            .iter()
            .map(|l| RenderLayer {
                grid: &l.grid,
                opacity: l.opacity,
                visible: l.visible,
            })
            .collect();
        render_pigment_only_region(&layers, cx0, cy0, cx1, cy1, &mut sess.pigment);
        drop(layers);
        // Straight-alpha OVER the frozen base, cell (cx,cy) → pixel (cx-1,cy-1).
        let gsel: Option<&[u8]> = gate_sel.as_deref().map(Vec::as_slice);
        let gprot: Option<&[u8]> = gate_prot.as_deref().map(Vec::as_slice);
        let gate_on = gsel.is_some() || gprot.is_some();
        let alock_on = gate_alock.is_some();
        let canvas = Arc::make_mut(&mut self.canvas_rgba);
        for cy in cy0..=cy1 {
            for cx in cx0..=cx1 {
                let o = ((cy - 1) * w + (cx - 1)) * 4;
                let pa = sess.pigment[o + 3] as f32 / 255.0;
                if pa <= 0.0 {
                    // Base verbatim — the gates keep-lerp TOWARD the base, so
                    // they are exact no-ops on this branch.
                    canvas[o..o + 4].copy_from_slice(&sess.base[o..o + 4]);
                    continue;
                }
                let ba = sess.base[o + 3] as f32 / 255.0;
                let oa = pa + ba * (1.0 - pa);
                for ch in 0..3 {
                    let pc = sess.pigment[o + ch] as f32;
                    let bc = sess.base[o + ch] as f32;
                    canvas[o + ch] = ((pc * pa + bc * ba * (1.0 - pa)) / oa).round() as u8;
                }
                canvas[o + 3] = (oa * 255.0).round() as u8;
                if gate_on {
                    let keep = super::watercolor_accum::splat_keep(gsel, gprot, None, o / 4);
                    if keep < 1.0 {
                        for c in 0..4 {
                            let painted = f32::from(canvas[o + c]);
                            let orig = f32::from(sess.base[o + c]);
                            canvas[o + c] = (painted * keep + orig * (1.0 - keep))
                                .round()
                                .clamp(0.0, 255.0)
                                as u8;
                        }
                    }
                }
                if alock_on {
                    // Alpha-lock: the α reference IS `sess.base` (the gate's
                    // buffer, served by `wet_splat_gates`' wet-session arm).
                    canvas[o + 3] = sess.base[o + 3];
                }
            }
        }
        // Re-arm the guard with the Arc our own make_mut may have re-seated.
        sess.canvas = Arc::clone(&self.canvas_rgba);
        let region = Region {
            x: (cx0 - 1) as u32,
            y: (cy0 - 1) as u32,
            w: (cx1 - cx0 + 1) as u32,
            h: (cy1 - cy0 + 1) as u32,
        };
        self.mark_dirty(region);
    }
}

#[cfg(test)]
mod tests; // the W1/W2 gates — child file (workspace file-LOC cap)
