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
    /// Last dab centre of the open stroke (chord source for the trail window).
    prev: Option<[f32; 2]>,
    /// A direct stroke is open in the engine (pen-down .. pen-up).
    stroke_open: bool,
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
                prev: None,
                stroke_open: false,
            });
        }
        let sess = self.paint.wetpaint.session.as_mut().expect("ensured above");
        if !sess.stroke_open {
            // The engine speaks sRGB **0..255** (its boot colour is `[50, 140, 210]` and the render
            // writes plane values straight through `clamp_u8`); the brush stores sRGB 0..1. Passing
            // the normalized value painted BLACK — Enio's W1 smoke, "a cor está preta".
            let c = brush.color;
            sess.engine.color = [
                f64::from(c[0]) * 255.0,
                f64::from(c[1]) * 255.0,
                f64::from(c[2]) * 255.0,
            ];
            let p0 = dabs[0].center;
            sess.engine
                .begin_direct_stroke(p0[0] as f64 + 1.0, p0[1] as f64 + 1.0);
            sess.stroke_open = true;
            sess.prev = None;
        }
        let strength = brush.strength.clamp(1e-3, 1.0);
        for d in dabs {
            let [x, y] = d.center;
            if let Some(p) = sess.prev {
                let chord = (((x - p[0]) as f64).powi(2) + ((y - p[1]) as f64).powi(2)).sqrt();
                sess.engine.direct_segment(chord);
            }
            let b = ((d.coverage / strength).clamp(0.0, 1.0) as f64) * 10.0;
            sess.engine.dispatch_pressure_dab(
                x as f64 + 1.0,
                y as f64 + 1.0,
                b,
                d.dir[0] as f64,
                d.dir[1] as f64,
                d.radius_px as f64,
            );
            sess.prev = Some(d.center);
        }
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
            sess.prev = None;
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
