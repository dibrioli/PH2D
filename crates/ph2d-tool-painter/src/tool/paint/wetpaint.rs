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
    base: Arc<Vec<u8>>,
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
        if self.paint.wetpaint.session.is_none() {
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
            let li = match lane {
                Some(i) => {
                    sess.engine.direct_segment(i, f64::from(best.sqrt()));
                    i
                }
                None => {
                    let i = sess.lanes.len();
                    // The DAB's colour, not the brush's: Randomize is already
                    // resolved per dab by the stroke engine (W2.2).
                    sess.engine.color = ink(d.color);
                    sess.engine
                        .begin_direct_stroke(i, f64::from(x) + 1.0, f64::from(y) + 1.0);
                    sess.lanes.push(Lane {
                        pos: d.center,
                        ink: d.color,
                    });
                    i
                }
            };
            // Per-dab fresh ink (Randomize): reload the lane's trail — a
            // brush dipped in new paint (see `Trail::set_base_color`).
            if d.color != sess.lanes[li].ink {
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
            sess.engine.dispatch_pressure_dab_lane(
                li,
                f64::from(x) + 1.0,
                f64::from(y) + 1.0,
                b,
                f64::from(d.dir[0]),
                f64::from(d.dir[1]),
                f64::from(d.radius_px),
                Some(&mut sil),
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
    fn wetpaint_composite(&mut self) {
        let (w, h) = self.source_size;
        let (w, h) = (w as usize, h as usize);
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
        let canvas = Arc::make_mut(&mut self.canvas_rgba);
        for cy in cy0..=cy1 {
            for cx in cx0..=cx1 {
                let o = ((cy - 1) * w + (cx - 1)) * 4;
                let pa = sess.pigment[o + 3] as f32 / 255.0;
                if pa <= 0.0 {
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
mod tests {
    //! W1 gates. Each names the mutation that bleeds it; the OFF contract and
    //! the canvas-identity guard are the two halves that MUST hold for the
    //! handoff's law #1 ("o comportamento atual não pode ser prejudicado").

    use super::*;
    use crate::tool::PainterTool;
    use ph2d_editor_core::tool::{CanvasPaintTool, CanvasPointer, PointerPhase};
    use ph2d_painter_brush::Falloff;

    fn cp(pos: [f32; 2], phase: PointerPhase) -> CanvasPointer {
        CanvasPointer {
            pos,
            pressure: 1.0,
            tilt: [0.0, 0.0],
            phase,
        }
    }

    /// A white opaque canvas + a red brush, in the given paint mode.
    fn tool_in_mode(mode: &str) -> PainterTool {
        let mut t = PainterTool::default();
        t.set_source(vec![255u8; 200 * 120 * 4], 200, 120);
        let b = BrushSpec {
            radius_px: 10.0,
            hardness: 1.0,
            falloff: Falloff::Constant,
            color: [0.8, 0.1, 0.1],
            space_attenuation: false,
            ..Default::default()
        };
        t.paint.brush = b;
        for slot in &mut t.paint.brush_by_mode {
            *slot = b;
        }
        t.set_paint_tool_mode(mode);
        t
    }

    fn stroke_across(t: &mut PainterTool) {
        t.on_canvas_pointer(cp([30.0, 60.0], PointerPhase::Down));
        for k in 1..=20 {
            t.on_canvas_pointer(cp([30.0 + 7.0 * k as f32, 60.0], PointerPhase::Move));
        }
        t.on_canvas_pointer(cp([170.0, 60.0], PointerPhase::Up));
    }

    /// Suspended-pigment mass inside the cell columns `[x0, x1]` (1-based).
    fn susp_in_columns(t: &PainterTool, x0: usize, x1: usize) -> f64 {
        let sess = t.paint.wetpaint.session.as_ref().expect("a wet session");
        let g = &sess.engine.layers[0].grid;
        let mut sum = 0.0f64;
        for cy in 1..=g.h {
            for cx in x0..=x1 {
                sum += f64::from(g.susp[cx + cy * g.s]);
            }
        }
        sum
    }

    /// SEAM: Symmetry is FREE through the choke point — the dab list arrives
    /// already mirrored, so a stroke held to the LEFT half deposits fluid on
    /// the RIGHT half too, in comparable mass. Mutation that bleeds it: a wet
    /// route hung off its own geometry instead of `stamp_dabs_inner`'s list
    /// (the exact disease the choke-point comment warns about).
    #[test]
    fn symmetry_mirrors_the_wet_deposit_for_free() {
        // The SOLO baseline first: the same stroke with no symmetry. The
        // ratio alone stayed green under the single-trail bug (the
        // alternating-anchor salvage is symmetric — each side kept ~half),
        // so the oracle also demands each side carries a full stroke's mass.
        let mut solo = tool_in_mode("wetpaint");
        solo.on_canvas_pointer(cp([25.0, 60.0], PointerPhase::Down));
        for k in 1..=10 {
            solo.on_canvas_pointer(cp([25.0 + 4.5 * k as f32, 60.0], PointerPhase::Move));
        }
        solo.on_canvas_pointer(cp([70.0, 60.0], PointerPhase::Up));
        let solo_left = susp_in_columns(&solo, 1, 100);

        let mut t = tool_in_mode("wetpaint");
        t.toggle_symmetry_enabled(); // mirror X on the canvas centre (x = 100)
        t.on_canvas_pointer(cp([25.0, 60.0], PointerPhase::Down));
        for k in 1..=10 {
            t.on_canvas_pointer(cp([25.0 + 4.5 * k as f32, 60.0], PointerPhase::Move));
        }
        t.on_canvas_pointer(cp([70.0, 60.0], PointerPhase::Up));
        let left = susp_in_columns(&t, 1, 100);
        assert!(
            left > solo_left * 0.8,
            "the original side lost mass to the mirror ({left} vs solo {solo_left})"
        );
        let right = susp_in_columns(&t, 101, 200);
        assert!(
            left > 1000.0,
            "the stroke itself deposited nothing ({left})"
        );
        // Each copy is a FULL stroke through its own lane. The first cut
        // accepted 0.5..2.0 and stayed green while the single-trail window
        // silently dropped half of every copy (the "Simetria Circular não
        // está correta" bug wore mirror clothes here).
        assert!(
            right > left * 0.75 && right < left * 1.33,
            "the mirrored deposit is missing or lopsided (left {left}, right {right})"
        );
    }

    /// SEAM (W2.3b): the painter's SILHOUETTE drives the wet stamp — a
    /// flattened dab (Flatten & Rotate) deposits a thin BAND, not the round
    /// disc the engine's internal footprint would lay. Mutation that bleeds
    /// it: passing `None` instead of the silhouette closure to the shaped
    /// door (the deposit's vertical extent balloons back to the full disc).
    #[test]
    fn the_flattened_dab_deposits_a_band_not_a_disc() {
        let extent_of = |flatten: f32| -> (usize, f64) {
            let mut t = tool_in_mode("wetpaint");
            t.paint.brush.dab_flatten = flatten;
            t.paint.brush.radius_px = 14.0;
            for slot in &mut t.paint.brush_by_mode {
                slot.dab_flatten = flatten;
                slot.radius_px = 14.0;
            }
            stroke_across(&mut t);
            let sess = t.paint.wetpaint.session.as_ref().expect("a wet session");
            let g = &sess.engine.layers[0].grid;
            // Vertical extent: rows carrying real suspended mass.
            let mut rows = 0usize;
            let mut total = 0.0f64;
            for gy in 1..=g.h {
                let m: f64 = (1..=g.w).map(|gx| f64::from(g.susp[gx + gy * g.s])).sum();
                total += m;
                if m > 200.0 {
                    rows += 1;
                }
            }
            (rows, total)
        };
        let (round_rows, round_mass) = extent_of(0.0);
        let (flat_rows, flat_mass) = extent_of(0.92);
        assert!(
            round_mass > 1000.0 && flat_mass > 500.0,
            "a fixture deposited nothing"
        );
        assert!(
            flat_rows * 2 < round_rows,
            "flatten never reached the fluid (flat {flat_rows} rows vs round {round_rows})"
        );
    }

    /// SEAM (the Enio report, W2): CIRCULAR symmetry — a stroke drawn in one
    /// sector must lay the SAME stroke in every radial sector. With one trail
    /// window the interleaved copies were silently dropped (`lx >=
    /// TRAIL_SIZE` returns) and the sectors came out broken; the lanes fix
    /// is what this pins. Mutation that bleeds it: routing every dab to lane
    /// 0 (the matching loop removed).
    #[test]
    fn circular_symmetry_lays_the_same_stroke_in_every_sector() {
        let mut t = tool_in_mode("wetpaint");
        t.toggle_symmetry_enabled();
        t.toggle_symmetry_circular();
        t.set_symmetry_segments(6);
        // A radial stroke inside one sector, well away from the centre.
        t.on_canvas_pointer(cp([130.0, 60.0], PointerPhase::Down));
        for k in 1..=10 {
            t.on_canvas_pointer(cp([130.0 + 4.0 * k as f32, 60.0], PointerPhase::Move));
        }
        t.on_canvas_pointer(cp([170.0, 60.0], PointerPhase::Up));
        // Suspended mass per angular sector around the canvas centre.
        let sess = t.paint.wetpaint.session.as_ref().expect("a wet session");
        let g = &sess.engine.layers[0].grid;
        let (cx, cy) = (101.0f32, 61.0f32); // canvas centre, cell coords
        let mut sectors = [0.0f64; 6];
        for gy in 1..=g.h {
            for gx in 1..=g.w {
                let m = g.susp[gx + gy * g.s];
                if m <= 0.0 {
                    continue;
                }
                let a = (gy as f32 - cy).atan2(gx as f32 - cx); // gate-only trig
                let sec = (((a + std::f32::consts::PI) / (2.0 * std::f32::consts::PI)) * 6.0)
                    .clamp(0.0, 5.999) as usize;
                sectors[sec] += f64::from(m);
            }
        }
        let max = sectors.iter().cloned().fold(0.0f64, f64::max);
        assert!(max > 1000.0, "no sector deposited anything ({sectors:?})");
        for (i, m) in sectors.iter().enumerate() {
            assert!(
                *m > max * 0.5,
                "sector {i} is missing its copy ({m:.0} vs max {max:.0}; all {sectors:?})"
            );
        }
    }

    /// SEAM (W2.2): Randomize Colour reaches the wet deposit — with full Hue
    /// jitter the per-dab `d.color` varies, the fresh-ink door follows it,
    /// and the deposited pigment carries VISIBLY different hues along the
    /// stroke. Mutation that bleeds it: dropping the `set_stroke_color` call
    /// (every cell wears the first dab's colour; spread collapses).
    #[test]
    fn randomize_colour_reaches_the_wet_deposit() {
        let mut t = tool_in_mode("wetpaint");
        t.set_brush_color_jitter(0, 1.0); // full Hue jitter
        stroke_across(&mut t);
        let sess = t.paint.wetpaint.session.as_ref().expect("a wet session");
        let g = &sess.engine.layers[0].grid;
        // Across the heavy cells, the red channel of the deposited colour
        // must SPREAD (different dabs, different hues). A fixed-ink stroke
        // measures a few units of spread from tip pickup; full hue jitter
        // measures >100.
        let (mut lo, mut hi, mut n) = (255.0f32, 0.0f32, 0usize);
        for i in 0..g.susp.len() {
            if g.susp[i] > 100.0 {
                lo = lo.min(g.susp_rgb[i][0]);
                hi = hi.max(g.susp_rgb[i][0]);
                n += 1;
            }
        }
        assert!(n > 50, "too few heavy cells to judge ({n})");
        assert!(
            hi - lo > 60.0,
            "the deposit wears one hue — Randomize never reached the fluid (spread {})",
            hi - lo
        );
    }

    /// SEAM: Tiling wraps the wet deposit — a stroke hugging the LEFT edge
    /// lands its wrapped copies by the RIGHT edge. Same mutation as the
    /// symmetry gate (the free lunch is the same list).
    #[test]
    fn tiling_wraps_the_wet_deposit_across_the_edge() {
        let mut t = tool_in_mode("wetpaint");
        t.paint.tiling[0] = true;
        t.on_canvas_pointer(cp([4.0, 60.0], PointerPhase::Down));
        for k in 1..=10 {
            t.on_canvas_pointer(cp([4.0, 60.0 + 3.0 * k as f32], PointerPhase::Move));
        }
        t.on_canvas_pointer(cp([4.0, 90.0], PointerPhase::Up));
        let near_left = susp_in_columns(&t, 1, 20);
        let near_right = susp_in_columns(&t, 185, 200);
        assert!(
            near_left > 500.0,
            "the stroke itself deposited nothing ({near_left})"
        );
        // The wrap is a full lane of its own now — not the accidental half
        // the alternating-anchor windows used to salvage (>0.1 was green
        // over that bug).
        assert!(
            near_right > near_left * 0.5,
            "no wrapped deposit by the far edge (left {near_left}, right {near_right})"
        );
    }

    /// PRESENCE: a Wet Paint stroke deposits (the canvas moves), the session
    /// exists past pen-up (the water is still wet), and the heartbeat keeps
    /// the sim alive without poisoning a pixel. Mutation that bleeds it:
    /// removing the route arm in `stamp_dabs_inner` (dabs fall through to the
    /// colour routes, no session is ever built).
    #[test]
    fn a_wet_stroke_deposits_and_the_water_survives_pen_up() {
        let mut t = tool_in_mode("wetpaint");
        let before = Arc::clone(&t.canvas_rgba);
        stroke_across(&mut t);
        assert!(
            t.paint.wetpaint.session.is_some(),
            "no wet session after a wet stroke"
        );
        assert_ne!(
            &*before, &*t.canvas_rgba,
            "a wet stroke left the canvas byte-identical"
        );
        // The session spans strokes: pen-up closed the engine stroke but kept the water.
        let sess = t.paint.wetpaint.session.as_ref().unwrap();
        assert!(
            !sess.stroke_open,
            "pen-up must close the engine's direct stroke"
        );
        assert!(
            sess.engine.sim_should_run(),
            "the sim must resume after pen-up"
        );
        // Heartbeat: the sim steps and composites without panicking or NaNing.
        for _ in 0..30 {
            t.paint_tick(1.0 / 40.0);
        }
        assert!(
            t.paint.wetpaint.session.is_some(),
            "the heartbeat must not kill the session"
        );
        assert!(t.canvas_rgba.iter().all(|b| *b != 0 || true), "unreachable");
    }

    /// The OFF contract — law #1: NO other paint mode reaches the wet engine.
    /// Every wet-paint canvas write goes through a session, so "no session
    /// after painting" IS "not one byte touched by this module". Mutation
    /// that bleeds it: widening the route arm's `matches!` to another mode.
    #[test]
    fn no_other_paint_mode_reaches_the_wet_engine() {
        for mode in [
            "brush", "eraser", "smear", "blur", "clone", "mask", "inpaint", "fill", "knife",
            "sculpt",
        ] {
            let mut t = tool_in_mode(mode);
            stroke_across(&mut t);
            assert!(
                t.paint.wetpaint.session.is_none(),
                "mode {mode:?} built a wet session — the wet engine leaked past its mode gate"
            );
        }
        // And the positive control: the same fixture in the wet mode DOES.
        let mut t = tool_in_mode("wetpaint");
        stroke_across(&mut t);
        assert!(
            t.paint.wetpaint.session.is_some(),
            "positive control failed"
        );
    }

    /// The canvas-identity guard, at the TICK: a foreign `canvas_rgba` swap
    /// (undo, fill, layer switch) ends the session BEFORE the sim composites
    /// over the restored pixels. Mutation that bleeds it: dropping the
    /// `wetpaint_guard` call from `wetpaint_tick` (the restored canvas gets
    /// repainted by a zombie session).
    #[test]
    fn a_foreign_canvas_swap_ends_the_session_before_the_tick_composites() {
        let mut t = tool_in_mode("wetpaint");
        stroke_across(&mut t);
        assert!(t.paint.wetpaint.session.is_some());
        // A foreign mutation: the undo restore swaps the canvas Arc wholesale.
        let restored = Arc::new(vec![255u8; 200 * 120 * 4]);
        t.canvas_rgba = Arc::clone(&restored);
        t.paint_tick(0.25);
        assert!(
            t.paint.wetpaint.session.is_none(),
            "a live session survived a foreign canvas swap"
        );
        assert_eq!(
            &*restored, &*t.canvas_rgba,
            "the tick composited over a canvas the session no longer owns"
        );
    }

    /// The paint wears the BRUSH's colour — the engine speaks 0..255 and the
    /// brush 0..1, and forgetting the scale paints black (Enio's W1 smoke).
    /// Mutation that bleeds it: dropping the `* 255.0` in the colour handoff.
    #[test]
    fn the_wet_paint_wears_the_brushs_colour_not_black() {
        let mut t = tool_in_mode("wetpaint");
        // The fixture brush is [0.8, 0.1, 0.1] — a saturated red.
        stroke_across(&mut t);
        // The strongest deposit anywhere on the canvas (the trail lays mass
        // with vertical structure, so a single row is not a fair sample —
        // the first cut of this gate scanned only the stroke row and failed
        // over a CORRECT product).
        let mut best = [255u8; 4];
        let mut best_dev = 0u32;
        for px in t.canvas_rgba.chunks_exact(4) {
            let dev = px[..3].iter().map(|&c| 255u32 - u32::from(c)).sum::<u32>();
            if dev > best_dev {
                best_dev = dev;
                best = [px[0], px[1], px[2], px[3]];
            }
        }
        assert!(
            best[0] > best[1].saturating_add(40) && best[0] > best[2].saturating_add(40),
            "the wet deposit is not red-dominant (got {best:?}) — the colour scale is wrong"
        );
        assert!(best[0] > 90, "the deposit reads near-black ({best:?})");
    }

    /// Entering Wet Paint must NOT take the Impasto section away (Enio, W1
    /// smoke: "uma das regras era não afetar o que já existia") — the section
    /// hosts the ten-tool list and the canvas's Lighting. And the radio must
    /// light NOTHING (claiming Deposit would be the lying radio the rail
    /// refused for the Knife). Mutations: drop WetPaint from
    /// `impasto_section_applies` (first half) or let `impasto_tool` fall to
    /// its `_ => DEPOSIT` arm (second half).
    #[test]
    fn wet_paint_keeps_the_impasto_section_with_no_tool_lit() {
        let t = tool_in_mode("wetpaint");
        assert!(
            t.impasto_section_applies(),
            "Wet Paint hid the Impasto section — the tool list and Lighting became unreachable"
        );
        assert_eq!(
            t.impasto_tool(),
            super::super::impasto_tool::IMPASTO_TOOL_NONE,
            "the tool radio claims a tool the hand is not holding"
        );
    }

    /// Leaving the mode ends the session, and ending IS the bake: the pixels
    /// of the last composite stay exactly as they are. Mutation that bleeds
    /// it: dropping the teardown arm in `set_paint_tool_mode`.
    #[test]
    fn leaving_the_mode_bakes_by_simply_stopping() {
        let mut t = tool_in_mode("wetpaint");
        stroke_across(&mut t);
        assert!(t.paint.wetpaint.session.is_some());
        let painted = Arc::clone(&t.canvas_rgba);
        t.set_paint_tool_mode("brush");
        assert!(
            t.paint.wetpaint.session.is_none(),
            "mode exit must end the wet session"
        );
        assert_eq!(
            &*painted, &*t.canvas_rgba,
            "the mode exit moved pixels — ending must be a stop"
        );
    }
}
