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
//! ## Authoring vs deposit (doc 21 — deposit-at-commit)
//!
//! The re-stamping methods (DragDot / Anchored / the shape editors) AUTHOR
//! through the normal flat pipeline (their batches are un-owned — see
//! [`PainterTool::wet_owns_the_dabs`]) and the fluid receives the FINAL dab
//! list exactly once, at commit (pen-up / Enter / Apply), through the
//! commit door (`wetpaint_commit`). The impasto height pass never runs in
//! this mode (relief pinned to a footprint would outlive paint that flows
//! away — gated in `stamp_dabs_inner`). A live deposit still only happens
//! inside a real incremental gesture: re-stamp previews piling paint into a
//! non-idempotent fluid while the artist just LOOKS (the I2 disease) is the
//! exact failure the ownership split + the route's belt refuse.

use super::*;
use ph2d_wet_paint::painter::{Dirty, Engine};

/// 40 Hz fixed-step (SPEC §5); at most 5 steps per frame, backlog dropped.
const WET_STEP_S: f32 = 1.0 / 40.0;
const WET_MAX_STEPS: usize = 5;

pub(crate) struct WetPaintState {
    /// The Wet Paint **checkbox** — the authored, persistent ARM (Enio
    /// 2026-07-21, the Watercolor/Impasto pattern): while `true`, the
    /// `"brush"` wire resolves to [`PaintMode::WetPaint`], so leaving to the
    /// eraser / selection / any tool and coming back returns to the FLUID
    /// instead of the plain digital brush. One fact, mode-independent, not a
    /// `BrushSpec` field (per-slot copies of one truth would disagree).
    /// Entering the mode by ANY door arms it — a checkbox that reads OFF
    /// while the paint is wet would be the lying radio this file refuses.
    pub(super) armed: bool,
    /// The live session; `None` until the first Wet Paint dab lands.
    pub(super) session: Option<WetSession>,
    /// A live freehand paint GESTURE is open (`paint_begin` .. pen-up). This —
    /// not `paint.stroke` — is the deposit gate: the lifecycle `mem::take`s
    /// the stroke while stamping, and the shape editors' per-frame re-stamps
    /// never run `paint_begin` at all, so the flag is exactly "dabs are the
    /// artist's hand, once".
    pub(super) live_gesture: bool,
    /// The authored knob values (W3) — survive the session, the mode and the
    /// tool round-trips; the section reset restores [`WetKnobs::default`].
    pub(super) knobs: WetKnobs,
    /// Doc 21 (deposit-at-commit): the commit-deposit STASH — the exact batch
    /// the last flat authoring preview painted (`stamp_drag_preview`'s own
    /// parameter, untiled/mirrored; the dispatcher re-tiles at deposit
    /// exactly as it did at preview). The deposit IS the preview by
    /// construction — nothing is re-derived at commit. Transient, never
    /// serialized; cleared by pen-down, cancel, mode-leave and teardown.
    pub(super) pending_deposit: Vec<Dab>,
    /// Doc 21: TRUE only across the single `stamp_dabs` replay inside
    /// `wetpaint_commit_deposit` — the third key on the wet arm's gate. A
    /// leak of this flag while a preview re-stamp runs is I2 resurrected
    /// (every refill becomes a fluid deposit); it is written in exactly two
    /// adjacent statements around that one call, nowhere else.
    pub(super) deposit_pass: bool,
    /// The wet TOOL (doc 22): which of the model's tools the brush wire
    /// drives while armed. Paint by default; Erase is NOT here — it is the
    /// rail eraser's other view.
    pub(super) tool: WetTool,
    /// The tilt DIAL (doc 22): on/off + ring (0..8, magnitude `ring/4`) +
    /// spoke (0..11, 30° steps). Boot = the model's boot = the engine's
    /// boot: ON, straight down, half radius (gate-pinned no-op reconcile).
    pub(super) tilt_on: bool,
    pub(super) tilt_ring: u8,
    pub(super) tilt_spoke: u8,
    /// Show-wet overlay (display-only; NEVER baked — the end-session door
    /// recomposites clean first).
    pub(super) show_wet: bool,
    /// Paper checkbox (doc 22 §2.8): the tooth becomes visually part of the
    /// painting (granulation + emboss into the pigment colours). Baked on
    /// purpose — it is part of the painting, unlike the show-wet veil.
    pub(super) paper_visual: bool,
    /// Experimental: K–M pigment mixing (`sim.km_mixing`).
    pub(super) km_mixing: bool,
    /// Experimental: K–M glaze stacking in the composite.
    pub(super) km_glaze: bool,
    /// The Tuning side panel's visibility (the basic section's checkbox).
    pub(super) tuning_open: bool,
}

impl Default for WetPaintState {
    fn default() -> Self {
        Self {
            armed: false,
            session: None,
            live_gesture: false,
            knobs: WetKnobs::default(),
            pending_deposit: Vec::new(),
            deposit_pass: false,
            tool: WetTool::default(),
            tilt_on: true,
            tilt_ring: 4,
            tilt_spoke: 3,
            show_wet: false,
            paper_visual: false,
            km_mixing: false,
            km_glaze: false,
            tuning_open: false,
        }
    }
}

/// Everything the session pushes into the ENGINE (doc 22) — compared as one
/// value per batch/tick; the boot constant equals the engine's own boot (gate-pinned).
#[derive(Clone, Copy, PartialEq)]
pub(super) struct WetEngineFacts {
    pub(super) knobs: WetKnobs,
    pub(super) tilt: (bool, u8, u8),
    pub(super) km_mixing: bool,
}

impl WetEngineFacts {
    /// The session's `applied` init — the engine's REAL boot (`WetKnobs::ENGINE_BOOT` == `Engine::new`),
    /// NOT the tool's product default (`WetKnobs::DEFAULT`). The first reconcile then sees the authored
    /// knobs differ from this baseline and pushes them; collapsing the two would skip the product values.
    pub(super) const BOOT: Self = Self {
        knobs: WetKnobs::ENGINE_BOOT,
        tilt: (true, 4, 3),
        km_mixing: false,
    };
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
    /// `pub(super)` so the doc-21 commit door can re-arm it after an OWNED
    /// authoring write (`wetpaint_commit::wetpaint_rearm_after_own_write`).
    pub(super) canvas: Arc<Vec<u8>>,
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
    /// What the engine's PAPER plane was last seeded from: `None` = the
    /// engine's own preset tile (session birth / paper disarmed); `Some` =
    /// the artist's Paper slot, keyed by everything the seed reads. The
    /// session **reconciles** against this every dab batch — paper is
    /// substrate under live water, but the SLOT is authored state, and the
    /// session spans strokes: seeding only at session birth left every later
    /// adjustment (Brightness/Contrast, any knob, the arm itself) silently
    /// ignored until the bake (Enio 2026-07-21, "os ajustes ... não estão
    /// sendo levados em consideração").
    paper_key: Option<PaperKey>,
    /// The [`WetEngineFacts`] last pushed into the engine — starts at the
    /// boot constant (the engine boots with exactly it, gate-pinned) and the
    /// reconcile pushes the authored values when they differ. Same law as
    /// `paper_key`: the facts are authored state, the session is not.
    applied: WetEngineFacts,
}

/// Everything the paper seed reads — the reconcile re-seeds exactly when one
/// of these moves, and an unchanged key costs one small struct compare per
/// batch. The image rides by its monotonic VERSION, never the `Arc` address
/// (an address survives a content swap and a content survives an address
/// swap — the ADR-0124 cache-identity trap).
#[derive(Clone, Copy, PartialEq)]
struct PaperKey {
    tex: ph2d_painter_brush::TextureSettings,
    image_version: u64,
    period: [f32; 2],
}

impl WetSession {
    /// Push the authored FACTS into the engine when they moved (W3, grown by
    /// doc 22). ONE door for the stamp AND the tick — Dry Speed and Gravity
    /// act on water already sitting on the canvas, so a knob must land while
    /// the sim runs, not only at the next stroke. Unchanged facts cost one
    /// struct compare. The tuning knobs go through `Engine::set_knob` (the
    /// door that reacts to what a knob invalidates: bristle rebuild lazy,
    /// paper re-bake, repaint) — and the paper re-bake only fires while the
    /// ENGINE's own tile is the tooth source (an armed artist Paper slot
    /// overrides it; its three physical knobs hide in the panel then).
    fn reconcile_facts(&mut self, f: WetEngineFacts) {
        use ph2d_wet_paint::tuning::KNOB_DEFS;
        if self.applied == f {
            return;
        }
        let a = self.applied;
        if f.knobs.water != a.knobs.water {
            self.engine.sliders.water = f.knobs.water;
        }
        if f.knobs.erase != a.knobs.erase {
            self.engine.sliders.erase = f.knobs.erase;
        }
        let moved = KNOB_DEFS.iter().zip(&f.knobs.knobs).zip(&a.knobs.knobs);
        for ((def, new), old) in moved {
            if new != old {
                self.engine.set_knob(def.knob, *new);
            }
        }
        if f.tilt != a.tilt {
            let (on, ring, spoke) = f.tilt;
            let d = ph2d_wet_paint::sim::tilt_dir_for_spoke(spoke);
            self.engine.sim.tilt_on = on;
            self.engine.sim.tilt_dir_x = d[0];
            self.engine.sim.tilt_dir_y = d[1];
            self.engine.sim.tilt_scale = f64::from(ring) / 4.0;
        }
        if f.km_mixing != a.km_mixing {
            self.engine.sim.km_mixing = f.km_mixing;
        }
        if self.paper_key.is_none() && self.engine.paper_dirty() {
            self.engine.rebake_paper();
        }
        self.applied = f;
    }
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
    /// Wet Paint owns (doc 21): the INCREMENTAL methods' live batches and the
    /// commit door's `deposit_pass` replay — nothing else. A non-incremental
    /// AUTHORING batch (DragDot / Anchored / the shape editors' re-stamps) is
    /// deliberately UN-owned: it falls through the completely normal flat
    /// pipeline (snapshot/restore wrapper + colour routes), which is what
    /// makes the flat preview — and hands it Selection/protection/alpha-lock
    /// byte-identically to Paint mode. The eraser with no live session (W2.6)
    /// also falls through (it erases the BAKED canvas, which is what is
    /// visibly there). Non-wet modes: the mode conjunct short-circuits first —
    /// not one instruction changes (gate G0a).
    pub(super) fn wet_owns_the_dabs(&self) -> bool {
        matches!(self.paint.paint_mode, PaintMode::WetPaint)
            && (self.paint.brush.stroke_method.is_incremental() || self.paint.wetpaint.deposit_pass)
            && (!self.paint.eraser || self.paint.wetpaint.session.is_some())
    }

    /// The Wet Paint dab route ([`Self::stamp_dabs_inner`] arm). The dab list
    /// is already mirrored (Symmetry) and replicated (Tiling) — the engine
    /// sees exactly what the colour routes would have seen.
    pub(super) fn stamp_dabs_wetpaint(&mut self, dabs: &[Dab], brush: &BrushSpec) {
        let (w, h) = self.source_size;
        // Two doors in, one belt (doc 21): a LIVE incremental gesture (the
        // W1/W2 path — dabs are the artist's hand, once) or the commit
        // door's one-shot `deposit_pass` replay. Everything else is refused
        // — authoring batches are un-owned and can no longer even reach
        // here (routing sends them flat); this gate is the belt against a
        // routing regression, and it must stay a WALL, not a debug_assert.
        let live = self.paint.wetpaint.live_gesture && brush.stroke_method.is_incremental();
        if dabs.is_empty() || w == 0 || h == 0 || !(live || self.paint.wetpaint.deposit_pass) {
            return;
        }
        self.wetpaint_guard();
        // The wet ERASER (W2.6) erases the FLUID and only the fluid: with no
        // live session there is nothing wet, and the routing predicate
        // (`wet_owns_the_dabs`) already sent those dabs to the normal
        // eraser — this early-out only covers the guard killing the session
        // between the route decision and here (the batch then does nothing).
        let erasing = self.paint.eraser;
        if self.paint.wetpaint.session.is_none() {
            if erasing {
                return;
            }
            self.ensure_wet_session();
        }
        // Take the session out so the prep below can borrow `self.paint`'s
        // Shape state alongside it (disjoint in fact, not in the borrow
        // checker's eyes); restored before the composite.
        let mut taken = self.paint.wetpaint.session.take().expect("ensured above");
        let sess = &mut taken;
        // ── The artist's PAPER drives the engine's tooth (W2.7) ────────────
        // RECONCILED per batch, not seeded once: paper is substrate under
        // live water, but the SLOT is authored state and the session spans
        // strokes — a key of everything the seed reads decides. Key moved =
        // re-seed; slot disarmed = the engine re-bakes its own preset
        // (`rebake_paper`); unchanged = one small struct compare. The law is
        // the painter's own canvas-fixed paper sampling (the NEUTRAL door
        // `sample_tiled_rot_wrapped`, the same law the watercolor substrate
        // reads — one paper, never a second system). Known v1 gap: the
        // bitmap Size seam-snap under sprite Tiling (`snap_slot_size`) is
        // not applied — procedurals wrap exactly; a bitmap paper may seam.
        let period = [
            if self.paint.tiling[0] { w as f32 } else { 0.0 },
            if self.paint.tiling[1] { h as f32 } else { 0.0 },
        ];
        let want = brush.paper.is_active().then_some(PaperKey {
            tex: brush.paper,
            image_version: self.paint.paper_image_version,
            period,
        });
        if sess.paper_key != want {
            if want.is_some() {
                let paper_tex = brush.paper;
                let paper_img = self.paint.paper_image.as_ref().map(|i| i.as_mask());
                let rot = ph2d_painter_brush::texture::angle_basis(paper_tex.angle_deg);
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
            } else {
                sess.engine.rebake_paper();
            }
            sess.paper_key = want;
        }
        // ── The authored facts — same reconcile law as the paper. ──────────
        sess.reconcile_facts(self.wet_facts());
        // The engine-side TOOL (doc 22): gives the lane doors their
        // `TrailMode` (begin picks Blend by it) and the sim its pause law
        // (`sim_should_run` keeps running under a Blow stroke, the model's
        // one exception). The ERASER always pauses — the engine erases with
        // its own tool untouched, exactly as before.
        let wet_tool = self.paint.wetpaint.tool;
        sess.engine.tool = if erasing {
            ph2d_wet_paint::painter::Tool::Paint
        } else {
            wet_tool.engine()
        };
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
            // The eraser and the DIRECT wet tools (Wet/Dry/Blow/Smear) are
            // LANE-LESS (per-dab grid ops, no trail) — open one direct
            // stroke so the sim gets its stroke gate as usual (paused under
            // the gesture; the Blow exception rides `engine.tool`).
            if (erasing || !wet_tool.uses_lanes())
                && let Some(d0) = dabs.first()
            {
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
            // where it lands. The direct wet tools keep the geometric lane
            // MATCHING (it is what carries each symmetry/tiling copy's own
            // previous dab centre — the smear/blow displacement source) but
            // never touch the engine's lane trails.
            let mut lane_born = false;
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
                        if wet_tool.uses_lanes() {
                            sess.engine.direct_segment(i, f64::from(best.sqrt()));
                        }
                        i
                    }
                    None => {
                        let i = sess.lanes.len();
                        lane_born = true;
                        if wet_tool.uses_lanes() {
                            // The DAB's colour, not the brush's: Randomize
                            // is already resolved per dab by the stroke
                            // engine (W2.2).
                            sess.engine.color = ink(d.color);
                            sess.engine.begin_direct_stroke(
                                i,
                                f64::from(x) + 1.0,
                                f64::from(y) + 1.0,
                            );
                        }
                        sess.lanes.push(Lane {
                            pos: d.center,
                            ink: d.color,
                        });
                        i
                    }
                }
            };
            // The lane's PREVIOUS centre (before this dab updates it) — the
            // direct tools' displacement source; a born lane has none, and
            // the eraser has no lanes at all.
            let lane_prev: Option<[f32; 2]> = (!erasing && !lane_born).then(|| sess.lanes[li].pos);
            // Per-dab fresh ink (Randomize): reload the lane's trail — a
            // brush dipped in new paint (see `Trail::set_base_color`).
            if !erasing && wet_tool == WetTool::Paint && d.color != sess.lanes[li].ink {
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
            match wet_tool {
                WetTool::Paint => sess.engine.dispatch_pressure_dab_lane(
                    li,
                    f64::from(x) + 1.0,
                    f64::from(y) + 1.0,
                    b,
                    f64::from(d.dir[0]),
                    f64::from(d.dir[1]),
                    f64::from(d.radius_px),
                    Some(&mut sil),
                    grain_arg,
                ),
                // Blend keeps the engine's own fixed-hardness stamp (the
                // model's tools ignore the brush silhouette on purpose).
                WetTool::Blend => sess.engine.dispatch_pressure_dab_lane_blend(
                    li,
                    f64::from(x) + 1.0,
                    f64::from(y) + 1.0,
                    b,
                    f64::from(d.dir[0]),
                    f64::from(d.dir[1]),
                    f64::from(d.radius_px),
                ),
                // Direct tools: per-dab grid ops; the displacement source is
                // THIS lane's previous centre (host-tracked per copy).
                WetTool::Wet | WetTool::Dry | WetTool::Blow | WetTool::Smear => {
                    sess.engine.dispatch_pressure_dab_tool(
                        wet_tool.engine(),
                        f64::from(x) + 1.0,
                        f64::from(y) + 1.0,
                        b,
                        f64::from(d.dir[0]),
                        f64::from(d.dir[1]),
                        f64::from(d.radius_px),
                        lane_prev.map(|p| [f64::from(p[0]) + 1.0, f64::from(p[1]) + 1.0]),
                    );
                }
            }
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
        // Doc 21 §F layer 2: the deposit flag must never survive into a tick.
        debug_assert!(!self.paint.wetpaint.deposit_pass);
        if self.paint.wetpaint.session.is_none() {
            return;
        }
        self.wetpaint_guard();
        // Doc 21 law D: the water FREEZES while the artist authors a flat
        // preview (flow AND drying) — zero steps, zero composites, zero knob
        // reconciles; a composite would copy `sess.base` verbatim where the
        // pigment is empty and ERASE the preview inside its dirty rect (a
        // torn state). The guard above still ran: foreign swaps (undo, layer
        // switch) kill the session even mid-authoring. The hold is DERIVED
        // (`drag_preview` is the flat re-stamp record in this mode) — no
        // state, no release door; commit/cancel drop the record and the
        // next tick simply resumes.
        if self.wet_authoring_hold() {
            return;
        }
        let facts = self.wet_facts();
        let Some(sess) = self.paint.wetpaint.session.as_mut() else {
            return;
        };
        // Facts land while the water SITS too (Dry Speed / Gravity / the
        // tilt are sim forces): the tick reconciles with the stamp's door.
        sess.reconcile_facts(facts);
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

    /// The canvas-identity guard (module doc): a foreign `canvas_rgba` swap
    /// ends the session before anything composites over restored pixels.
    fn wetpaint_guard(&mut self) {
        if let Some(sess) = &self.paint.wetpaint.session
            && !Arc::ptr_eq(&sess.canvas, &self.canvas_rgba)
        {
            self.paint.wetpaint.session = None;
        }
    }
}

mod authored_actions; // canvas actions + session birth + facts — child file (LOC cap)
mod composite; // the composite half (visual terms + veil) — child file (LOC cap)

#[cfg(test)]
mod tests; // the W1/W2 gates — child file (workspace file-LOC cap)
#[cfg(test)]
mod tests_doc22; // the doc-22 gates (tuning/tilt/tools/actions/flags)
