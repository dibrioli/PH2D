//! The stroke engine — turns a pointer path into dabs per the Blender "Stroke" panel.
//!
//! Behavioural reference (clean-room, no code copied): Blender `paint_stroke.cc`. The per-raw-sample
//! pipeline is **input-samples box-average** ([`crate::sampler`]) → **stabilize spring** → per-
//! [`StrokeMethod`] emission → per-dab **dash gate** + **jitter** + pressure **dynamics** + **space-
//! attenuation**. `Space` resamples the path at `spacing × diameter`; the per-event methods (`Dots`/
//! `DragDot`/`Airbrush`) emit one dab per sample. The *interactive* methods (`Anchored`/`Line`/`Arc`)
//! are tool/shell-owned; the engine exposes [`Stroke::fill_segment`] + [`Stroke::tick`] as primitives.

use crate::dynamics::Dynamics;
use crate::line_kind::LineKind;
use crate::sampler::InputSampler;
use crate::spec::{BrushSpec, MAX_BRUSH_RADIUS_PX};
use crate::stroke_method::{JitterUnit, StrokeMethod};

/// One input sample from the pointer device, in image-space pixels.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct StrokePoint {
    /// Position in image pixels.
    pub pos: [f32; 2],
    /// Pen pressure in `[0, 1]`; `1.0` for devices without pressure (mouse).
    pub pressure: f32,
}

/// One dab to stamp: where, how big, and how opaque. The falloff profile / blend come from the
/// [`BrushSpec`]; the pressure-varying parts plus the per-dab randomized colour / rotation live here.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Dab {
    /// Centre in image pixels.
    pub center: [f32; 2],
    /// Radius in pixels (brush radius × pressure size-scale × [`BrushSpec::jitter_scale`]).
    pub radius_px: f32,
    /// Per-dab opacity in `[0, 1]` (brush strength × pressure coverage-scale × space-attenuation).
    pub coverage: f32,
    /// This dab's paint colour. Equal to [`BrushSpec::color`] unless **Randomize Color** is on, in
    /// which case it is the per-dab HSV-jittered colour (see [`crate::jitter`]).
    pub color: [f32; 3],
    /// Extra per-dab texture rotation as a unit vector (identity `[1, 0]` = none); set by **Jitter
    /// Rotate**. The stamp composes it into the texture frame; it has no effect without a texture.
    pub rotation: [f32; 2],
    /// The **stroke heading** at this dab, as a unit vector (`[0, 0]` = no heading yet, e.g. the stroke's
    /// first dab). The length-weighted EMA of the path tangent ([`crate::heading::advance`]), driven once
    /// per flattened path chord so the smoothing length is honoured **in arc length** — which is what the
    /// EMA's own contract promises and what its caller was not doing (it fed the within-chord remainder,
    /// so the filter ran with a step up to 16× too small and lagged by up to 52°, reported as *"o Rake não
    /// gira"*, Enio 2026-07-19; the cure is the caller, not the estimator — a raw one-spacing secant tracks
    /// beautifully and is 3-5× noisier dab to dab, which a repeating pattern shows as interlacing).
    /// [`crate::BrushSpec::dab_rotor`] turns the dab frame onto it when a slot follows the stroke; it is
    /// IGNORED otherwise, so a non-following brush is unaffected by it.
    pub dir: [f32; 2],
    /// **Arc-length** at this dab's centre: the cumulative distance travelled along the stroke path from
    /// the pen-down (in image pixels), monotonic and continuous across segments. The Shape **Flow** mapping
    /// uses it as the *along-the-stroke* coordinate so the silhouette pattern's phase stays continuous from
    /// dab to dab (parallel lines that follow the curve) instead of resetting per stamp. It is IGNORED
    /// unless the Shape's Flow is on, so a non-Flow brush is unaffected by it.
    pub arc_len: f32,
    /// The stroke's **nominal dab radius** (px) — [`BrushSpec::clamped_radius`] at pen-down, constant for
    /// the whole stroke, unlike [`Self::radius_px`] which breathes with pressure and Jitter Scale.
    ///
    /// It is the unit the Shape **Flow** mapping divides its along-the-stroke coordinate by, and it has to
    /// be stroke-constant or the promise of Flow dies: the numerator is the pixel's absolute position along
    /// the path, so a per-dab divisor re-phases the entire accumulated history by `arc_len · Δ(1/r)` every
    /// time the pressure wobbles. It rides the DAB because the engine is the only thing that can know it
    /// and cannot get it wrong — the same reason [`Self::arc_len`] does.
    pub stroke_radius_px: f32,
}

/// Incremental stroke state. Feed it pointer samples; it emits dabs per the brush's stroke method.
///
/// Usage: [`Stroke::begin`] on pointer-down, [`Stroke::extend`] on every move. Both fill a
/// caller-provided `Vec<Dab>` (cleared first) so a hot pointer loop allocates nothing per call.
#[derive(Clone, Debug)]
pub struct Stroke {
    spec: BrushSpec,
    dynamics: Dynamics,
    /// Last point the path was walked to — the spacing segment start AND the stabilize spring
    /// anchor (Blender's `last_mouse_position`).
    last_pos: [f32; 2],
    /// **Grid Stamp** — a última célula carimbada, para o carimbo ser *uma vez por célula* em vez de
    /// uma vez por evento: o ponteiro treme dentro de uma célula e não pode re-depositar por isso.
    /// `None` até o primeiro carimbo do gesto.
    last_cell: Option<[i32; 2]>,
    last_pressure: f32,
    /// Distance travelled since the last emitted dab (carried across segments).
    accum: f32,
    /// **Jitter Spacing** multiplier for the next gap (`1.0` = even); re-drawn after each dab, carried
    /// across segments like `accum`.
    spacing_mult: f32,
    /// State for the dep-free jitter RNG (splitmix64).
    rng: u64,
    started: bool,
    /// Input-samples box-average ring (front of the pipeline).
    sampler: InputSampler,
    /// Monotonic emitted-dab-slot counter that drives the dash pattern.
    tot_samples: u32,
    /// Cached "Adjust Strength for Spacing" multiplier (recomputed on every spec change).
    overlap: f32,
    /// Airbrush time accumulator in seconds, consumed by [`Stroke::tick`].
    airbrush_accum_s: f32,
    /// A velocidade do gesto em px/s **medida no tique** — a grandeza que [`Stroke::speed_px_s`]
    /// publica. A lei e o porquê vivem em [`mod@self::speed`], a porta única dela.
    speed_px_s: f32,
    /// O `arc_len` no tique anterior — o outro extremo do `Δarco` que [`mod@self::speed`] divide.
    speed_arc_mark: f32,
    /// A velocidade que o ARREMESSO deste dab cavalga: ela caminha até a medida acima pela rampa,
    /// e é PRIVADA de propósito — a grandeza que descreve o gesto é a medida, não a rampeada.
    throw_speed_px_s: f32,
    /// O comprimento de arco sobre o qual a rampa caminha até o alvo — o arco do quadro medido.
    speed_ramp_len: f32,
    /// O `arc_len` do dab anterior, para a rampa saber quanto de caminho passou entre dois dabs.
    speed_dab_arc: f32,
    /// **A MEMÓRIA do traço** — a vizinhança que o Sketchy costura: `[x, y, arco]`, os dois
    /// primeiros em px de canvas e o terceiro o [`Dab::arc_len`] daquele centro.
    ///
    /// ⚠️ **O arco não é enfeite: é o que separa os dois modos de Magnetify.** *Perto no canvas* e
    /// *perto no percurso* são perguntas diferentes, e o Krita chama a primeira de Magnetify ligado
    /// (dois trechos que se aproximaram) e a segunda de desligado (a porção ativa do traço). Sem o
    /// arco guardado ao lado do ponto, a segunda é inexprimível.
    ///
    /// Nasce vazia em cada [`Stroke::begin`], que é o que impede um fio de ligar dois TRAÇOS.
    /// Só é populada com o Sketchy armado: um `Vec` que cresce em todo traço do app seria memória
    /// paga por quem nunca escolheu o tipo.
    sketchy_pts: Vec<[f32; 3]>,
    /// Os fios costurados desde o último [`Stroke::take_threads`].
    threads: Vec<sketchy::Thread>,
    /// ⚠️ **Fluxo de RNG PRÓPRIO do Sketchy.** Sortear os fios do `rng` do traço adiantaria o mesmo
    /// fluxo que o Jitter (posição, escala, rotação, cor) consome — e então **ligar o Sketchy mudaria
    /// o jitter de um pincel**, em silêncio. Dois consumidores, dois fluxos.
    sketchy_rng: u64,
    /// **A MIRA do arremesso** (vetor unitário; `[0,0]` = ainda não estabelecida) — o heading suavizado
    /// por uma janela PRÓPRIA, muito mais longa que a do Rake. A lei e o porquê vivem em
    /// [`mod@self::speed`]; o resumo é que o arremesso é uma ALAVANCA e uma alavanca amplifica tremor.
    throw_dir: [f32; 2],
    /// Onde a TINTA do dab anterior caiu (posição arremessada, pré-jitter) e o arco dele — o começo
    /// do segmento que [`mod@self::speed`] preenche. `None` = não há dab anterior a ligar.
    fill_from: Option<([f32; 2], f32)>,
    /// Onde a tinta do dab ATUAL caiu; vira o `fill_from` do próximo. Um par, não dois campos, porque
    /// as duas metades são a mesma resposta (*onde e quando esta tinta caiu*).
    last_thrown: Option<([f32; 2], f32)>,
    /// The point *before* `last_pos` — the trailing neighbour the Catmull-Rom smoother needs for the
    /// centred tangent at the start of each segment. `None` until the second move after
    /// [`Stroke::begin`] (the first segment has no trailing neighbour, so it starts straight).
    prev_prev: Option<[f32; 2]>,
    /// The stabilizer's lazy-mouse filtered position (lags the cursor by the stabilizer intensity).
    /// The path is built from this, not the raw cursor — that is what regularises a shaky hand.
    stab_pos: [f32; 2],
    /// The most recent *raw* (un-stabilized) sample, so [`Stroke::finish`] can catch the lagged
    /// stroke up to the real release point on pointer-up.
    last_raw_pos: [f32; 2],
    last_raw_pressure: f32,
    /// The smoothed stroke **heading** (unit vector; `[0, 0]` = none yet) stamped on every emitted dab as
    /// `Dab::dir`. Updated by the length-weighted EMA in [`Self::walk_space`] (where the path tangent is
    /// clean) and reset in [`Self::begin`]. Drives the texture **Rake**; see [`crate::heading`].
    heading: [f32; 2],
    /// **Rake warm-up** state (see [`mod@self::warmup`]): opening dabs held in `warm_buf` until the cursor
    /// travels `warm_dist` ≥ [`crate::heading::warmup_len`] (from `warm_from`), then released at the settled
    /// heading. `warming` gates it — `false` ⇒ dabs flow straight through (non-Rake / ineligible method).
    warm_buf: Vec<Dab>,
    warm_dist: f32,
    warm_from: [f32; 2],
    warming: bool,
    /// Cumulative arc-length travelled to `last_pos` (image px), stamped on each [`Dab::arc_len`] and used
    /// by the Shape **Flow** mapping as the along-the-stroke coordinate. Reset in [`Self::begin`]; the shape
    /// editors build a fresh `Stroke` per re-stamp, so it restarts from `0` there on its own.
    arc_len: f32,
    /// **Whether this stroke has a head for [`crate::taper`] to shape** — see [`mod@ends`]. Set by
    /// whoever knows the geometry: the closed-loop fills declare that their first dab lands on an
    /// arbitrary seam, and everything else keeps what the method started with.
    taper_head: bool,
}

/// Smallest lazy-mouse blend factor, reached at stabilizer `1.0` (heaviest smoothing / most lag).
/// At stabilizer `0.0` the factor is `1.0` (the filtered point IS the cursor — no stabilization).
const STABILIZER_MIN_BLEND: f32 = 0.08;
/// When settling on a pause, converge at least this fast per tick — so even the heaviest stabilizer
/// catches up to the cursor in a fraction of a second, not the ~1 s its in-stroke blend would take.
const SETTLE_BLEND_FLOOR: f32 = 0.3;
/// Sub-pixel distance at which the stabilizer counts as caught up to the cursor (stops settling).
const SETTLE_EPS_PX: f32 = 0.5;
/// Max airbrush dabs a single [`Stroke::tick`] may emit, so a stalled frame's oversized `dt`
/// can't dump a retroactive flood (Blender's async timer never catches up after a hitch).
const MAX_AIRBRUSH_DABS_PER_TICK: u32 = 8;

impl Stroke {
    /// Create a stroke for `spec`/`dynamics`. `seed` seeds the jitter RNG (use a per-stroke
    /// counter; never a global/thread RNG — keeps replays reproducible, HR-5).
    #[must_use]
    pub fn new(spec: BrushSpec, dynamics: Dynamics, seed: u64) -> Self {
        Self {
            spec,
            dynamics,
            last_pos: [0.0, 0.0],
            last_cell: None,
            last_pressure: 1.0,
            accum: 0.0,
            spacing_mult: 1.0,
            rng: seed ^ 0x9E37_79B9_7F4A_7C15,
            started: false,
            sampler: InputSampler::new(spec.input_samples),
            tot_samples: 0,
            overlap: spec.space_overlap_factor(),
            airbrush_accum_s: 0.0,
            speed_px_s: 0.0,
            speed_arc_mark: 0.0,
            throw_speed_px_s: 0.0,
            speed_ramp_len: 0.0,
            speed_dab_arc: 0.0,
            sketchy_pts: Vec::new(),
            threads: Vec::new(),
            sketchy_rng: seed ^ 0x2545_F491_4F6C_DD1D,
            throw_dir: [0.0, 0.0],
            fill_from: None,
            last_thrown: None,
            prev_prev: None,
            stab_pos: [0.0, 0.0],
            last_raw_pos: [0.0, 0.0],
            last_raw_pressure: 1.0,
            heading: [0.0, 0.0],
            warm_buf: Vec::new(),
            warm_dist: 0.0,
            warm_from: [0.0, 0.0],
            warming: false,
            arc_len: 0.0,
            taper_head: ends::method_starts_with_head(spec.stroke_method),
        }
    }

    /// Update the live brush parameters mid-stroke (e.g. the artist drags a slider). Recomputes the
    /// cached space-attenuation factor and re-clamps the input-samples window.
    pub fn set_spec(&mut self, spec: BrushSpec) {
        self.spec = spec;
        self.overlap = spec.space_overlap_factor();
        self.sampler.set_window(spec.input_samples);
    }

    /// Begin the stroke at `p`. For the continuous methods this emits the first dab at the down
    /// point; the interactive methods (Anchored/Line/Arc) only record the anchor (they paint on
    /// finalise via [`Stroke::fill_segment`]).
    pub fn begin(&mut self, p: StrokePoint, out: &mut Vec<Dab>) {
        out.clear();
        self.last_pos = p.pos;
        self.last_pressure = p.pressure;
        self.accum = 0.0;
        self.spacing_mult = 1.0;
        self.tot_samples = 0;
        self.airbrush_accum_s = 0.0;
        self.speed_px_s = 0.0; // traço novo: a mão ainda não andou, então não há inércia a arremessar
        self.speed_arc_mark = 0.0;
        self.throw_speed_px_s = 0.0;
        self.speed_ramp_len = 0.0;
        self.speed_dab_arc = 0.0;
        self.sketchy_pts.clear(); // traço novo: nenhum fio liga ao traço anterior
        self.threads.clear();
        self.throw_dir = [0.0, 0.0];
        self.fill_from = None;
        self.last_thrown = None; // traço novo: nenhuma tinta anterior a que ligar a primeira
        self.prev_prev = None;
        self.stab_pos = p.pos;
        self.last_raw_pos = p.pos;
        self.last_raw_pressure = p.pressure;
        self.heading = [0.0, 0.0]; // fresh stroke → heading re-aims from the first travel (Rake from Angle)
        self.arc_len = 0.0; // fresh stroke → the Flow along-coordinate starts at the pen-down
        self.last_cell = None; // Grid Stamp: gesto novo, nenhuma célula carimbada ainda
        // Rake warm-up (see [`mod@self::warmup`]): hold the opening dabs until travel defines the heading.
        self.warm_buf.clear();
        self.warm_dist = 0.0;
        self.warm_from = p.pos;
        // Every reader of `Dab::dir` must be in this list, or its opening dabs get `[0, 0]` and it silently
        // degrades. `needs_heading` is how the third one (the Sculpt Chisel) says so — see `BrushSpec`.
        // The fourth is the deposit's PUSH (the bow-wave bank aims by the heading): derived from the
        // spec itself rather than enumerated by a caller, so it cannot rot the way this list once did.
        self.warming = self.spec.stroke_method.rake_warmup_eligible()
            && (self.spec.texture.rake
                || self.spec.shape.rake
                || self.spec.shape.flow
                || self.spec.needs_heading
                || self.spec.effective_impasto_push() > 0.0);
        self.sampler.reset(p);
        self.started = true;
        if self.spec.stroke_method == StrokeMethod::GridStamp {
            // O pen-down carimba a célula sob o cursor: um clique tem de pintar uma célula, senão o
            // método só funcionaria arrastando.
            self.stamp_cell(self.spec.grid_cell_at(p.pos), out);
        } else if self.spec.stroke_method.emits_on_begin() {
            let pr = self.method_pressure(p.pressure);
            let dab = self.dab_at(p.pos, pr, self.method_overlap(), self.arc_len);
            self.emit(dab, out);
            self.tot_samples = self.tot_samples.wrapping_add(1);
        }
        self.warmup_gate(out);
    }

    /// Extend the stroke to the raw sample `raw`: average it into the input-sample window, run it
    /// through the stabilizer, then emit dabs per the stroke method. No-op until [`Stroke::begin`].
    pub fn extend(&mut self, raw: StrokePoint, out: &mut Vec<Dab>) {
        out.clear();
        if !self.started {
            return;
        }
        let avg = self.sampler.push_average(raw);
        self.last_raw_pos = avg.pos;
        self.last_raw_pressure = avg.pressure;
        if self.warming {
            self.warm_dist += dist(self.warm_from, avg.pos); // accumulate travel toward the warm-up release
            self.warm_from = avg.pos;
        }
        match self.spec.stroke_method {
            // The stabilizer (smooth-stroke) applies to Space + the per-event Dots/Airbrush, like
            // Blender. Space additionally resamples through the spline; Dots/Airbrush place one
            // dab per event at the filtered position. Drag Dot below stays raw (exact placement).
            StrokeMethod::Space => {
                let target = self.stabilize(avg);
                self.walk_smoothed(target, out);
            }
            StrokeMethod::Dots => {
                let target = self.stabilize(avg);
                self.advance_heading_to(target.pos); // Rake heading from the inter-dab travel (no spline)
                self.arc_len += dist(self.last_pos, target.pos); // per-event: arc grows by the inter-dab travel
                self.emit_single(
                    target.pos,
                    self.method_pressure(target.pressure),
                    self.arc_len,
                    out,
                );
                self.advance_anchor(target);
            }
            // Airbrush deposits dabs ONLY on the timer ([`Stroke::tick`]), never on motion (Blender
            // `paint_stroke.cc`). A move tracks the (stabilized) cursor for the next tick + advances the
            // Rake heading; holding still keeps depositing at one spot (the characteristic spray).
            StrokeMethod::Airbrush => {
                let target = self.stabilize(avg);
                self.advance_heading_to(target.pos);
                self.arc_len += dist(self.last_pos, target.pos); // track the cursor so the next tick's dab flows
                self.advance_anchor(target);
            }
            // Drag Dot: one dab/move at the cursor; the tool restores the previous so only one follows (commits last on up).
            StrokeMethod::DragDot => {
                self.arc_len += dist(self.last_pos, avg.pos);
                self.emit_single(avg.pos, 1.0, self.arc_len, out);
                self.advance_anchor(avg);
            }
            // Anchored: a single stamp pinned at the press point (`last_pos`, never advanced) whose
            // RADIUS is the drag distance from it (Blender `anchored_size = |cursor − initial|`; edge-
            // to-edge centres on the midpoint with half the radius, spanning anchor→cursor). The tool
            // routes this dab through the restore+re-stamp preview (like Drag Dot) so it commits on up.
            StrokeMethod::Anchored => {
                let anchor = self.last_pos;
                let dx = avg.pos[0] - anchor[0];
                let dy = avg.pos[1] - anchor[1];
                let dist = (dx * dx + dy * dy).sqrt();
                let (center, radius) = if self.spec.edge_to_edge {
                    ([anchor[0] + dx * 0.5, anchor[1] + dy * 0.5], dist * 0.5)
                } else {
                    (anchor, dist)
                };
                // Anchored never walks the spline, so it has no EMA heading; its Rake direction is the
                // drag vector (anchor → cursor). `[0, 0]` before any drag → `dab_basis` falls back to Angle.
                let dir = if dist > f32::EPSILON {
                    [dx / dist, dy / dist]
                } else {
                    [0.0, 0.0]
                };
                let dab = self.anchored_dab(center, radius, dir);
                crate::symmetry::push_symmetric(out, dab, &self.spec.symmetry);
            }
            // Line / Arc / Ellipse / Polygon / Free Hand are tool/shell-driven shape editors
            // (`docs/Painter/`), filled via `Stroke::fill_*_preview` (Line = `fill_polyline_preview`); a
            // bare `extend` (no editor) only tracks the anchor.
            StrokeMethod::Line
            | StrokeMethod::Arc
            | StrokeMethod::Ellipse
            | StrokeMethod::Polygon
            | StrokeMethod::FreeHand => {
                self.advance_anchor(avg);
            }
            // **Grid Stamp** — o percurso desde a amostra anterior é percorrido em passos de meia
            // célula e cada célula NOVA recebe um carimbo no seu centro. ⚠️ O passo é meia célula (o
            // menor lado) porque assim duas amostras consecutivas caem na mesma célula ou numa
            // vizinha: nenhuma célula ATRAVESSADA é pulada num arrasto rápido. Um caminho que corta
            // exatamente uma quina não carimba as duas células adjacentes — ele não entra nelas, e
            // carimbar mesmo assim seria a ferramenta pintando onde a mão não passou.
            StrokeMethod::GridStamp => {
                self.walk_grid_cells(self.last_pos, avg.pos, out);
                self.advance_anchor(avg);
            }
        }
        self.warmup_gate(out);
    }

    /// Fill the straight segment `a → b` with spaced dabs (Blender LINE / CURVE-segment finalise).
    /// Pressure is constant (Blender fills lines/curves at pressure `1.0`). Appends to `out` (the
    /// caller clears) and continues the dash counter so a multi-segment curve dashes continuously.
    pub fn fill_segment(&mut self, a: [f32; 2], b: [f32; 2], pressure: f32, out: &mut Vec<Dab>) {
        self.last_pos = a;
        self.last_pressure = pressure;
        self.accum = 0.0;
        if self.spec.dash_on(self.tot_samples) {
            let pr = self.method_pressure(pressure);
            let d = self.dab_at(a, pr, self.method_overlap(), self.arc_len);
            self.emit(d, out);
        }
        self.tot_samples = self.tot_samples.wrapping_add(1);
        self.walk_space(StrokePoint { pos: b, pressure }, out);
    }

    /// Close the stroke on pointer-up. With the stabilizer on, the painted path lags the cursor, so
    /// flush the lag: walk the spline from the last filtered point up to the true release point so
    /// the stroke ends exactly where the pen lifted (Space only — the per-event methods stamp at the
    /// cursor each event and have nothing pending). Then clear the smoother state.
    pub fn finish(&mut self, out: &mut Vec<Dab>) {
        out.clear();
        if self.started && self.spec.stroke_method == StrokeMethod::Space {
            self.walk_smoothed(
                StrokePoint {
                    pos: self.last_raw_pos,
                    pressure: self.last_raw_pressure,
                },
                out,
            );
        }
        // Ended still warming (e.g. a tap): flush the held dabs + tail at whatever heading settled
        // (`[0, 0]` ⇒ the rest Angle, right for a directionless tap).
        if self.warming {
            self.warm_buf.append(out);
            self.release_warmup(out);
        }
        self.prev_prev = None;
    }

    /// Advance the airbrush timer by `dt` seconds, emitting a dab at the current cursor every
    /// `rate` seconds (Blender airbrush TIMER). No-op unless the method is [`StrokeMethod::Airbrush`].
    /// The tool drives this from its tick while the button is held and the cursor is parked.
    pub fn tick(&mut self, dt: f32, out: &mut Vec<Dab>) {
        out.clear();
        if !self.started {
            return;
        }
        // ⚠️ A velocidade do gesto é medida ANTES do desvio abaixo, e a ordem é a feature — o
        // porquê está no [`mod@self::speed`], junto da lei.
        self.note_tick_speed(dt);
        if self.spec.stroke_method != StrokeMethod::Airbrush {
            return;
        }
        let rate = self.spec.airbrush_rate_s.max(1e-3);
        self.airbrush_accum_s += dt.max(0.0);
        let mut emitted = 0u32;
        while self.airbrush_accum_s >= rate {
            self.airbrush_accum_s -= rate;
            let pr = self.method_pressure(self.last_pressure);
            let d = self.dab_at(self.last_pos, pr, 1.0, self.arc_len); // per-event: no spacing attenuation
            self.emit(d, out);
            self.tot_samples = self.tot_samples.wrapping_add(1);
            emitted += 1;
            // Stall guard: a frame-driven tick can hand us a huge `dt` after a hitch (GC/resize/
            // breakpoint). Blender's async timer wouldn't fire retroactively, so cap the burst and
            // drop the backlog rather than dump a flood of overlapping dabs at one spot.
            if emitted >= MAX_AIRBRUSH_DABS_PER_TICK {
                self.airbrush_accum_s = 0.0;
                break;
            }
        }
    }

    // ── internals ───────────────────────────────────────────────────────────────────

    /// Space spacing walk from `last_pos` → `target`, emitting a dab every `spacing × diameter` of
    /// arc length and carrying the residual distance across calls (`accum`).
    fn walk_space(&mut self, target: StrokePoint, out: &mut Vec<Dab>) {
        let from = self.last_pos;
        let to = target.pos;
        let seg = dist(from, to);
        if seg <= f32::EPSILON {
            self.last_pressure = target.pressure;
            return;
        }
        let base_step = self.spec.dab_spacing_px();
        let dir = [(to[0] - from[0]) / seg, (to[1] - from[1]) / seg];
        // Smoothing length for the heading EMA, from the brush diameter (Krita's Fade is brush-relative).
        let smooth_len = crate::heading::smooth_len(2.0 * self.spec.clamped_radius());
        let overlap = self.method_overlap();
        // Arc-length at `from` (= current cumulative to `last_pos`); each dab stamps `base_arc + traveled`,
        // and the segment advances the accumulator by its full length so the along-stroke coordinate is
        // continuous across walk_space calls (the Catmull-Rom flattener calls this once per short chord).
        let base_arc = self.arc_len;
        let mut traveled = 0.0;
        // Path length of this chord already folded into the heading EMA. The filter is length-weighted
        // (`α = Δs/(Δs + L)`) and its contract is that behaviour is independent of how the path was
        // chopped into steps — so it must be fed the travel that ACTUALLY elapsed, including the travel of
        // chords that emit no dab at all. Feeding it the within-chord remainder instead (the bug until
        // 2026-07-19) ran it with a step up to 16× too small, i.e. an effective smoothing length of ~240 px
        // where 12 was intended, which is where the 52° of Rake lag came from.
        let mut advanced = 0.0;
        loop {
            // **Jitter Spacing** scales each gap by the carried multiplier (`1.0` = even); a break does
            // not redraw (no wasted draw), and the next-gap draw is gated so an off brush is unchanged.
            // ⚠️ The gap is `spacing × the diameter that will actually be stamped here`, not the nominal
            // one. Spacing is a RATIO — "how much of a dab's width until the next" — so holding the gap
            // fixed while the taper shrinks the dab makes the ratio blow up and the tip comes out as a
            // row of separate DOTS (Enio 2026-08-08, with the picture). Scaling the step by the same
            // width factor the dab gets keeps the overlap constant all the way to the point.
            //
            // The `max(1.0)` floor is the engine's own, not a new number: it already bounded the walk,
            // and it is what stops the loop from stalling as the factor approaches zero at a sharp tip.
            let w = self.taper_width_at(base_arc + traveled);
            let step = (base_step * w * self.spacing_mult).max(1.0);
            // A step that shrank below what is already banked (a taper ramp, or a live spacing edit)
            // must not walk BACKWARDS: emit here, and the reset of `accum` below carries the walk on.
            let to_next = (step - self.accum).max(0.0);
            if traveled + to_next > seg {
                break;
            }
            traveled += to_next;
            let f = traveled / seg;
            let pos = [from[0] + dir[0] * traveled, from[1] + dir[1] * traveled];
            let pressure = lerp(self.last_pressure, target.pressure, f);
            // Fold in exactly the path travelled since the heading was last advanced, so the dab is
            // stamped with the heading as of ITS position and the smoothing length is honoured in arc
            // length, independent of dab density and of the flattener's chord size.
            self.heading =
                crate::heading::advance(self.heading, dir, traveled - advanced, smooth_len);
            advanced = traveled;
            if self.spec.dash_on(self.tot_samples) {
                let d = self.dab_at(pos, pressure, overlap, base_arc + traveled);
                self.emit(d, out);
            }
            self.tot_samples = self.tot_samples.wrapping_add(1);
            self.accum = 0.0;
            self.spacing_mult = if self.spec.jitter_spacing > 0.0 {
                crate::jitter::spacing_mult(&mut self.rng, self.spec.jitter_spacing)
            } else {
                1.0
            };
        }
        self.accum += seg - traveled;
        // The rest of the chord still happened — fold it in, so a chord that emits NO dab (the common
        // case: the Catmull-Rom flattener chops at ~3 px while the spacing can be tens of px) still steers
        // the heading. Without this the EMA only ever saw the sub-chord slivers around the dabs it did emit.
        self.heading = crate::heading::advance(self.heading, dir, seg - advanced, smooth_len);
        self.arc_len = base_arc + seg; // arc at `to` = arc at `from` + the full chord length
        self.last_pos = to;
        self.last_pressure = target.pressure;
    }

    /// Freehand smoother for the `Space` method — a **Catmull-Rom spline** through the input points.
    /// Each `extend` paints the segment from the last point `a` to the new point `p`, so the stroke
    /// follows the cursor in real time (no held-back tail), and the spline interpolates *through*
    /// every sample with a smooth, continuous tangent, so sparse / coalesced input reads as a clean
    /// curve instead of the connected straight facets the old scheme produced.
    ///
    /// The segment `a → p` is a cubic Hermite with the Catmull-Rom tangents: at `a` the centripetal
    /// tangent `(p − prev_prev)/2` (smooth join with the previous segment), at `p` the causal chord
    /// `p − a`. The first segment after [`Stroke::begin`] has no `prev_prev` (start tangent = chord,
    /// straight). The cubic is flattened into short chords fed through [`Stroke::walk_space`] (which owns
    /// spacing / dash / jitter / attenuation). Collinear input keeps straight strokes straight.
    fn walk_smoothed(&mut self, p: StrokePoint, out: &mut Vec<Dab>) {
        let a = self.last_pos;
        let a_pr = self.last_pressure;
        let b = p.pos;
        let seg = dist(a, b);
        if seg <= f32::EPSILON {
            self.last_pressure = p.pressure;
            return;
        }
        // Catmull-Rom tangents (at `a`: centred on its neighbours `prev_prev` and `b`; the first
        // segment uses the chord. at `b`: the causal chord), scaled by the stabilizer intensity `w`:
        // `w = 0` ⇒ zero tangents ⇒ the Hermite is the straight chord `a→b` (raw, faceted path);
        // `w → 1` ⇒ full curvature between samples. So the one knob ramps from raw to smooth.
        let w = self.spec.stabilizer.clamp(0.0, 1.0);
        let m_a = match self.prev_prev {
            Some(pp) => [(b[0] - pp[0]) * 0.5 * w, (b[1] - pp[1]) * 0.5 * w],
            None => [(b[0] - a[0]) * w, (b[1] - a[1]) * w],
        };
        let m_b = [(b[0] - a[0]) * w, (b[1] - a[1]) * w];
        // Flatten the Hermite `a → b` into short chords (denser than the dab spacing so the curve
        // never facets), each chained through `walk_space`.
        let n = ((seg / 3.0).ceil() as usize).clamp(1, 96);
        for i in 1..=n {
            let t = i as f32 / n as f32;
            self.walk_space(
                StrokePoint {
                    pos: hermite(a, m_a, b, m_b, t),
                    pressure: lerp(a_pr, p.pressure, t),
                },
                out,
            );
        }
        // This segment's start point becomes the next segment's `prev_prev` neighbour.
        self.prev_prev = Some(a);
    }

    /// Emit one dab at `pos`/`pressure` (the per-event methods). Per-event dabs carry no
    /// space-attenuation (that normalises *dense spacing*, which these don't have).
    fn emit_single(&mut self, pos: [f32; 2], pressure: f32, arc: f32, out: &mut Vec<Dab>) {
        let d = self.dab_at(pos, pressure, 1.0, arc);
        self.emit(d, out);
        self.tot_samples = self.tot_samples.wrapping_add(1);
    }

    /// Build the Anchored stamp's single dab at `center` with the drag-defined `radius_px`. No
    /// jitter (Blender disables it for Anchored, `paint_stroke_use_jitter`), full pressure
    /// (coverage = strength); the falloff / hardness / blend / colour come from the spec at stamp
    /// time, like every dab — so the anchored stamp wears the brush profile, just sized by the drag.
    fn anchored_dab(&self, center: [f32; 2], radius_px: f32, dir: [f32; 2]) -> Dab {
        Dab {
            center,
            radius_px: radius_px.clamp(0.0, MAX_BRUSH_RADIUS_PX),
            coverage: self.spec.strength.clamp(0.0, 1.0),
            // Anchored opts out of all per-dab jitter (like position jitter): a deliberate stamp.
            color: self.spec.color,
            rotation: [1.0, 0.0],
            // The Rake heading is the drag direction (anchor → cursor), set by the caller.
            dir,
            // Anchored is a single pinned stamp — there is no travel, so the Flow along-coordinate is 0.
            arc_len: 0.0,
            stroke_radius_px: self.spec.clamped_radius(),
        }
    }

    /// Move the spacing/spring anchor to `target` without resetting the spacing residual (used by
    /// the per-event + interactive methods, which don't accumulate distance).
    fn advance_anchor(&mut self, target: StrokePoint) {
        self.last_pos = target.pos;
        self.last_pressure = target.pressure;
    }

    /// Pressure to use given the method (DragDot/Anchored/Line force full pressure).
    fn method_pressure(&self, pressure: f32) -> f32 {
        if self.spec.stroke_method.forces_full_pressure() {
            1.0
        } else {
            pressure
        }
    }

    /// Space-attenuation multiplier for the current method — applied to the spaced fills
    /// (Space/Line/Arc), `1.0` for the per-event methods.
    fn method_overlap(&self) -> f32 {
        match self.spec.stroke_method {
            StrokeMethod::Space
            | StrokeMethod::Line
            | StrokeMethod::Arc
            | StrokeMethod::Ellipse
            | StrokeMethod::Polygon => self.overlap,
            _ => 1.0,
        }
    }

    /// **How wide the taper is at `arc`** — the one place that question is answered, for BOTH of the
    /// things that depend on it: how big the dab is, and how far the next one goes.
    ///
    /// `1.0` when the taper is off, when the mark has no head, or past the window — exactly `1.0`, so
    /// every arithmetic that multiplies by it is byte-identical to the arithmetic that shipped before the
    /// taper existed.
    fn taper_width_at(&self, arc: f32) -> f32 {
        if !self.spec.taper.is_active() || !self.taper_head {
            return 1.0;
        }
        self.spec.taper.width(arc, 2.0 * self.spec.clamped_radius())
    }

    /// The live **stroke heading** (unit vector; `[0, 0]` = not established yet), i.e. the direction the
    /// next dab will be stamped with. Exposed so the brush-cursor ring can show the orientation the dab
    /// will actually use when a slot follows the stroke — the ring reads the ENGINE's heading rather than
    /// re-deriving one from the cursor, so it cannot point somewhere the paint does not.
    #[must_use]
    pub fn heading(&self) -> [f32; 2] {
        self.heading
    }
}

mod curve;
/// The Arc stroke method’s fill + the shared Catmull-Rom flattener (child module, as the others below).
/// **Como UM dab é construído** a partir de uma posição do caminho: dinâmica de pressão, atenuação
/// de espaçamento, o jitter por-dab e o taper. Um estágio nomeado do pipeline do cabeçalho.
mod dab_build;
/// A MEMÓRIA do traço e os fios do `Sketchy` (plano 38 W3).
pub mod sketchy;
/// A INÉRCIA do gesto: a velocidade por tique e o arremesso do `Speed Shapes` (plano 38 W2).
mod speed;
pub use curve::flatten_catmull_rom;
/// The Ellipse stroke method's perimeter generator + ellipse fill (same child-module rationale).
mod ellipse;
pub use ellipse::ellipse_perimeter;
/// The Polygon stroke method's regular-N-gon perimeter + fill (same child-module rationale).
mod polygon;
pub use polygon::{POLY_MAX_SIDES, POLY_MIN_SIDES, polygon_perimeter};
/// Whether a stroke has a **head** for the taper to shape (`method_starts_with_head`), and the record
/// of the two things that were built there and removed: the withheld tail and the far end itself.
mod ends;
/// A aritmética de CAMINHO partilhada (`dist` / `lerp` / `hermite`) — três consumidores
/// (o caminhador, o achatador de Catmull-Rom e o estabilizador) e nenhum deles é o dono do fato.
mod geom;
/// **Grid Stamp** emission (`walk_grid_cells` / `stamp_cell` / `grid_dab`) — split out for the
/// LOC-cap reason, and by SUBJECT: how a stroke that snaps to a lattice emits its dabs is a
/// different question from how a free stroke walks a path.
mod grid;
/// The lazy-mouse stabilizer (`stabilize`/`settle`/`lazy_mouse_step`) — split out for the LOC-cap reason.
pub mod stabilize;
use geom::{dist, hermite, lerp};
/// The texture-Rake **warm-up** (`warmup_gate`/`release_warmup`) deferring a stroke's opening dabs.
mod warmup;

#[cfg(test)]
mod tests;
