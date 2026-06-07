# ADR-0077 — Brush engine physics overhaul (pressure, wet/burnt edges, dab AA, taper)

- **Status:** Accepted — pressure (D1), wet/burnt edges + intensity slider (D2/D3), pixel-relative dab AA (D4), live start taper + UI (D5) all landed; sub-pixel dab positioning (D4), end-taper re-render + One-Euro (D5) deferred with recipes below
- **Date:** 2026-06-07
- **Scope:** `ph2d-painter-brush` (scheduler, cpu_render, settle), `ph2d-tool-painter` (lifecycle), `ph2d-panel-brush-studio`
- **Supersedes nothing; amends** ADR-0044 (brush GPU contract) at the *semantics* level only — the frozen `Stamp` ABI, `RenderingMode`, and sub-caps are untouched (every change below rides existing fields/flags).

## Context

The Painter brush engine had three documented failures (HANDOFF_painter_brush_overhaul_impl §0):

1. **Wet/Burnt Edges shipped twice-wrong** — a per-dab rim, then a coverage-feather rim, both producing a **hard, artificial, low-resolution outline** around the stroke, nothing like real watercolor/charcoal.
2. **Falloff twice-wrong** (already corrected before this ADR — ink-depletion model).
3. **Pressure recorded but inert** — `Stamp.pressure` was stored but never modulated size or opacity; `PencilParams.pressure_curve` / `pressure_targets` were dead.

The root cause of (1) was **modelling the wrong physics**: edge darkening was treated as a *contour filter on the stroke silhouette* instead of *pigment transport to the drying-water boundary*. This ADR records the researched, physically-grounded replacements.

### Sources (researched 2026-06-07; cited at the call sites)

- **Watercolor / edge darkening:** Curtis, Anderson, Seims, Fleischer, Salesin, *"Computer-Generated Watercolor"*, SIGGRAPH 1997 (the canonical 3-layer fluid + `FlowOutward: p ← p − η·(1 − blur(M))·M` operator); patent US6198489B1; WPI transcription <https://davis.wpi.edu/~matt/courses/watercolor/fluid.html>. Real-time approximations: DiVerdi et al. *"Painting with Polygons"* (I3D 2012, grow-then-bake), Adobe "Edge effect" patent US7777745B2 (distance × low-freq Perlin noise), Montesdeoca/Bousseau real-time watercolor (`out = color^(1+strength·diff)`), Adobe Fresco / Procreate Wet Mix.
- **Dry media (burnt edges):** Sousa & Buchanan, *"Observational Models of Graphite Pencil Materials"*, CGF 2000 — tone = `f(pressure, paper-tooth)`, peaks catch pigment first, granulated edges from pressure taper.
- **Pressure / dab AA / taper:** Procreate Handbook (Brush Studio Pressure/Taper); Van Verth, *"Drawing an Antialiased Ellipse"*, GDC 2015 (gradient-normalized coverage `d ≈ f/‖∇f‖`, what Skia uses); Inigo Quilez *2D distance functions* + *premultiplied alpha*; Casiez et al. *"1€ Filter"*, CHI 2012.
- **Pigment (audited, already sound):** spectral.js / Kubelka-Munk single-constant `K/S = (1−R)²/2R`; Sochorová & Jamriška *Mixbox* (SIGGRAPH Asia 2021) — our clean-room spectral K-M (`pigment_mix.rs`) matches this approach; validated by the blue+yellow→green / cyan+red→neutral battery. **No change needed.**

## Decision

### D1 — Pressure → size + opacity (LANDED)

The `StampScheduler` shapes raw pen pressure through `brush.pencil.pressure_curve` (8-pt piecewise-linear, `pencil::eval_curve8`) and routes it to size and/or opacity per `pressure_targets` (default `Size | Opacity`). Pressure scales the **effective diameter before spacing is derived**, so a thinning tail also tightens its dab spacing ("dabs per actual radius", MyPaint) — no dotted line. Opacity folds into the per-stamp deposit. Mouse/touch report constant 1.0 → identity curve → **zero regression**. Deterministic (pure arithmetic, HR-5). Tests: `pressure_modulates_size_and_opacity_monotonically`, `low_pressure_tail_keeps_dabs_dense_no_dotted_line`, etc.

### D2 — Wet edges via pigment-transport settle pass (LANDED — the headline fix)

**Not a contour filter.** On stroke settle (pen-up — the Procreate/DiVerdi grow-then-bake lifecycle), `cpu_render::apply_wash_settle` computes the **inner-edge shoulder of the continuous coverage field**:

```
rim = clamp(COV − blur_K(COV), 0, 1)          // discrete Curtis (1 − blur(M))·M
rim *= (noise_floor + (1−noise_floor)·FBM(x·f, y·f))   // low-freq noise breaks the rim up
c   = c^(1 + strength·rim)                     // multiplicative darken in linear light
```

This darkens the **wet-region boundary** (soft, sub-pixel, 0 in the interior and outside), noise-broken so it never reads as an outline, applied multiplicatively so it deepens value *and* saturation (pigment moved to the rim, lighter centre). Wash-mode only — the `wash_coverage` buffer **is** the wet-region field. Live preview shows interior-only wet paint (Phase A); the rim settles in on pen-up (Phase B). Tests: `wet_edge_darkens_the_rim_not_the_centre`, `wet_edges_darken_the_stroke_rim_on_pen_up` (end-to-end), `wet_edges_off_leaves_a_flat_wash` (control).

### D3 — Burnt edges via dry-media tooth grain (LANDED)

Same inner-edge transport band, but `EdgeStyle::Burnt` is made **clearly distinct** from the wet rim (otherwise, on the same colour, both just darkened and looked identical): a **bidirectional** tooth modulation over a **wider band** — coarse paper-tooth grain where peaks *darken* (`e>1`, charcoal catches) and valleys *lighten* (`e<1`, bare paper shows through), vs the wet rim which only ever darkens smoothly. That two-sided speckle over a broad edge zone reads as granular dry media, not a wet dark line. Tests: `burnt_edge_makes_a_grainier_rim_than_wet` (Laplacian high-pass); visually confirmed (rough charcoal edge vs smooth watercolor bloom).

Both D2/D3 are exposed as live toggles in the Brush Studio Rendering section (the previously-dormant `PAINTER_STUDIO_WET_EDGES` / `_BURNT_EDGES` ids), plus a shared **Edge Intensity** slider (`RenderingParams.edge_intensity`, default 0.6, `BrushParam::EdgeIntensity`) that scales the active settle strength and is shown only when an edge mode is on. The settle works in BOTH wash and build-up (`accumulate`) modes — the per-pixel coverage buffer is a side output of the build-up render (`apply_stamps_buildup`), gated on edges being on (zero overhead otherwise). Tests: `wet_edges_work_in_accumulate_buildup_mode`, `edge_intensity_scales_the_rim_darkness`.

### D4 — Pixel-relative SDF dab AA (LANDED) + sub-pixel positioning (optional refinement)

The dab kernels (`library::shape_*`) used to antialias over a **radius-relative** band (`[0.85R, R]`): sub-pixel on small dabs (aliased — the "serrilhado") and over-soft on large dabs. **Landed:** analytic coverage in pixel units — `SHAPE_AA_PX = 1.5`, `aa = SHAPE_AA_PX / footprint`, `coverage = clamp((0.5 − r)/aa + 0.5)` for round/square, Van Verth gradient-normalized `clamp(0.5 − (f/‖∇f‖)/aa)` for the oval (round_soft keeps its `(1−d²)²` profile). Mirrored CPU (`library.rs` + `cpu_render`) and WGSL (`stamp.wgsl`), with `cpu_shader_shape_kernels_textual_parity` updated to pin the new strings. Result: a clean ~1.5px edge at every brush size; verified visually (a 7px brush magnified is smooth, no stairstep). The dab still rasterizes on the integer-snapped footprint grid; **sub-pixel positioning** (rasterize over the canvas bbox from the fractional stamp centre) is a further optional refinement — deferred because it churns ~47 pinned positional assertions for a marginal gain now that the 1.5px AA masks the ≤0.5px snap. Recipe:

```
// aa = SHAPE_AA_PX / footprint  (SHAPE_AA_PX ≈ 1.5px, the transition width in canvas px)
round_hard : r = len(uv−0.5);                 cov = clamp((0.5 − r)/aa + 0.5, 0, 1)
square_hard: d = max(|uv−0.5|);               cov = clamp((0.5 − d)/aa + 0.5, 0, 1)
oval_hard  : ex=(u−.5)/.5; ey=(v−.5)/.25; f=ex²+ey²−1; g=‖(2ex/.5, 2ey/.25)‖;
             cov = clamp(0.5 − (f/g)/aa, 0, 1)        // Van Verth / Taubin
round_soft : unchanged (its softness is the brush, not AA)
// + iterate canvas pixels in [centre ± size/2 + 1px], local = (px+0.5) − centre_frac
//   so a slow diagonal stroke stops stair-stepping (research: never snap the centre).
```

Preview filtering is already `Smooth`/bilinear (`ImageFilterMode::default()`), so the jaggedness is the raster, which the recipe fixes.

### D5 — Live start taper (LANDED) + end-taper re-render & One-Euro (deferred)

**Landed:** live **start taper** — `scheduler::start_taper_factors` ramps dab size + opacity up over the first `L_start = taper_length_start · 12 · diameter` of arc length (smoothstep), so the stroke enters from a clean point independent of pressure. Default (`taper_length_start == 0`) is exact passthrough. Exposed as a **"Taper"** slider in the Brush Studio Stroke Path section (`BrushParam::TaperLength`, slider 0..1 → `taper_length_start` 0..0.5; the default pointed/faint tip makes it a clean entry). Tests: `start_taper_grows_size_and_opacity_from_the_tip`, `no_taper_is_exact_passthrough`; visually confirmed (pointed entry → full width at constant pressure). **Deferred:** **end taper** needs the stroke end → a pen-up re-render (re-walk the recorded samples on `end_stroke` with a symmetric tail ramp; the wash/coverage buffer already exists to re-composite). **One-Euro filter** (Casiez 2012, adaptive cutoff) should replace the fixed EMA+moving-average stabilizer — needs per-sample `dt` threaded into the scheduler.

### D6 — Rendering modes honoured in the wash path (LANDED)

Previously the 6 rendering modes (`apply_rendering_mode`) applied **only** in the build-up path; the default **wash** composite (and the pigment K-M composite) ignored them, so the Brush Studio "Mode" cycler was inert on the default brush. Fixed: the wash now composites via the brush's rendering mode — **Linear wash** runs the full `apply_rendering_mode` (all six distinct; `UniformGlaze` reproduces the old over-lerp byte-for-byte, so the default `round_hard` brush is unchanged), and **pigment wash** keeps its subtractive K-M colour but applies a per-mode coverage curve (`wash_glaze_coverage`: Light = thinner, Intense/Heavy = more solid; UniformGlaze = identity) so the glaze modes still vary the wash transparency. Test: `wash_honors_rendering_mode_option_a`; visually confirmed (the 6 modes are distinct in the default wash). Note: in the *pigment* wash the two Blending modes ≈ UniformGlaze (their distinction is colour mixing, which K-M already does) — full six-way distinction needs Linear pigment, same as build-up.

### D7 — Global canvas paper tooth (LANDED — the #1 "pro vs digital" enchant)

Two independent pro-comparison audits (watercolor vs Rebelle/Fresco/Procreate; dab/stroke vs Procreate/PS/Krita) converged on the **same** root cause for "looks digital": **our default stroke is a flat solid-colour ribbon — the procedural-grain system shipped disabled (`GrainSource::None`)** — whereas every pro brush has paper tooth showing through *on every stroke*, pressure-modulated (Procreate Grain Texturized+Depth; Photoshop Texture Depth/Min-Depth; watercolor granulation = pigment × paper tooth). Both audits ranked this the highest payoff-per-cost, and it is also the watercolor report's cheapest 60–70% ("granulation in the wash body, no fluid sim").

Landed: `cpu_render::paper_tooth_factor` — a static **world-space** paper-height field (so it is consistent across strokes and does NOT crawl with the brush) with a **pressure-aware tooth threshold**: a light touch only deposits on the paper's crests (the valleys show through as bare paper), firmer pressure fills the valleys. Applied to every dab in BOTH the wash and build-up paths. Controlled by `PainterParams.paper_grain` (0..1, `#[serde(default)]`). **Default revised to 0.0 (OFF)** — the original 0.4-ON applied the tooth to *every* stroke including when the grain selector read "none", which the user flagged as unexpected grain; the tooth is now an explicit opt-in via a **"Paper" slider** in the Brush Studio Rendering section (`PAINTER_STUDIO_PAPER_SLIDER` → `BrushParam::Paper` → `params.paper_grain`; 0 = crisp ink, 1 = heavy paper). No Brush-ABI change (it's a tool/canvas param). Tests: `paper_tooth_textures_stroke_and_modulates_by_pressure`, `paper_slider_routes_to_params_paper_grain`; visually confirmed.

### D8 — Sub-pixel dab positioning (LANDED)

The dab shape was sampled on the integer footprint grid (`(i+0.5)/footprint`), so a slow diagonal stroke snapped coverage a whole pixel at a time → the "serrilhado" wobble the user saw. Fixed in both `cpu_render` loops + the WGSL `cs_stamp`: the world write coord is computed first (consecutive `floor(pos + i − c + 0.5)`), then the shape is sampled from each pixel centre's offset to the dab's TRUE fractional centre (`uc = (world − position) / size_px`), so coverage shifts smoothly sub-pixel. `aa` now derives from `1/size_px` (not the ceil'd `footprint`), bit-identical CPU↔shader — locked by the `cpu_shader_rotation_pipeline_textual_parity` gate (updated) + naga parse/validate. The flip-invariance gate was relaxed to ±1/255 (an integer position with an even footprint samples a slightly asymmetric uv — *more* correct than the old footprint-centred sampling, which was a half-pixel off — so flip is visually but not bit-exactly invariant under rotation).

### D9 — Watercolor v1.5 dry-down: K–M thickness rim + granulation + outward bleed (LANDED)

The pen-up `apply_wash_settle` is upgraded from a gamma `c^e` rim to physically-grounded Kubelka–Munk, with three effects (all CPU-only, in the one settle pass — no live-path, WGSL, or frozen-mix change). Grounded in three cited research passes (Curtis SIGGRAPH 1997; Montesdeoca/MNPR; Bousseau; ScienceDirect K–M; citations inline in `settle.rs`):

- **K–M thickness rim** (replaces the gamma): the rim deposits extra pigment *optical depth* `t` at the receding boundary and recomputes via the finite-thickness K–M `R(t) = (1 − Rg(a−b·coth(b·t)))/(a − Rg + b·coth(b·t))` toward the pigment **masstone** (`wash_color`). Darker AND more saturated with the correct hue shift, **bounded by the masstone** (no black-crush). Consequence: edge darkening shows only where the body is a glaze (`opacity < 1` / coverage dip) — the physical watercolor case; an opaque fill correctly can't go past R∞. (Two opacity-1.0 rim tests updated to transparent washes.)
- **Granulation** (mass-conserving): pigment sediments into the paper-tooth **valleys** (deposit toward masstone) and lifts off the **crests** (saturating lerp toward the backdrop — removing pigment, the inverse of a deposit, so NOT a `km_deposit`). Strength rides the **Paper** slider (granulation *is* pigment settling into the tooth). Gives a flat wash a mottled granular texture with no net darkening.
- **Outward bleed** (the bloom past the silhouette): a second pass over the bbox grown by `2.6·rim_px`; the blurred-coverage "front" is **domain-warped** by 3-octave fBM into irregular tendrils, gated by a `smoothstep` band (fades to exactly 0 — no outer ring), and deposited as a THIN K–M glaze of the stroke's mean pigment over the backdrop (correct over-composite for opaque paper *and* a transparent layer). Driven by the wet-edge `strength`.

Build-up mode (no `wash_color` buffer) and `EdgeStyle::Burnt` (dry media) keep the gamma path. Tests: `km_deposit_endpoints_and_monotone`, `km_rim_darkens_and_saturates_toward_masstone`, `granulation_mottles_a_flat_glaze`, `outward_bleed_deposits_a_fringe_past_the_silhouette`, `settle_is_deterministic` (HR-5); visually confirmed (ultramarine wash: dark rim + valley sediment + feathered bloom). Follow-ups: two-octave granulation for coarser clumping; per-pigment granulation constants; the live gated-diffusion solver (v2).

### D10 — Velocity dynamics + One-Euro motion filtering (LANDED)

Both wire **dormant** `Brush` sub-struct fields that shipped unimplemented; the default brush (all 0) is byte-identical to before (gates + existing smoothing tests untouched). All pure arithmetic → HR-5 deterministic; no new `det_random` axes.

- **One-Euro motion filtering** (Casiez/Roussel/Fekete, CHI 2012) — wires `stabilization.motion_filtering_amount` (→ min cutoff, more smoothing) + `motion_filtering_expression` (→ β, speed-responsiveness; the field's "reinject expression" intent). Added as a third stage in `smooth_input_position` (stabilization MA → One-Euro → streamline lag), per-axis with `dt = 1` (no timestamps). Adaptive low-pass: low speed → low cutoff (kills tremor), high speed → high cutoff (no lag — keeps expressive strokes crisp), which the fixed EMA/MA cannot. `MF_MIN_CUTOFF = 0.22` (the adaptive β recovers fast strokes, so a low floor is safe). Tests: `motion_filtering_damps_slow_tremor`, `motion_filtering_expression_keeps_fast_strokes_responsive`, `motion_filtering_is_deterministic`.
- **Velocity dynamics** — wires `dynamics.speed_{size,opacity,spacing}` (∈[-1,1]). A smoothed stroke speed (EMA of per-advance segment length) → `vfactor ∈[0,1]` (`SPEED_REF_PX = 24`) scales size/opacity/spacing by `±SPEED_DYN_SWING` (0.6). E.g. `speed_size = −0.85` gives a calligraphic taper (fast → thin). Test: `speed_size_grows_the_dab_on_fast_strokes` (+ default speed-invariance).

Visually confirmed (`visual_smoke_velocity_and_smoothing`): calligraphic velocity taper + One-Euro stripping fine tremor while keeping the gesture. **Exposed in the Brush Studio:** Motion Filt + Motion Expr in the Stroke Path section (unipolar), Speed→Size/Opac/Space in the Dynamics section (bipolar −100..+100%, centre = off; `BrushParam` maps slider `v·2−1`). To stay under the HR-18 panel-file LOC cap, the Stroke Path / Shape / Dynamics rows were converted to data-table loops, and the pre-existing `painter_bridge.rs` overage (629 LOC, unrelated debt from `69446c2`) was decomposed — its two downcast queries moved to `painter_bridge_queries.rs` (added to the no-downcast allowlist). Test: `motion_filter_and_speed_sliders_route_to_brush`. Follow-up: end-taper pen-up re-render (D5).

### D11 — Watercolor v2: wet-on-wet diffusion solver CORE (LANDED; live integration = open decision)

The deterministic physics core of live watercolor is landed as `ph2d_painter_brush::diffusion` — a low-resolution **gated diffusion-advection** field (Curtis SIGGRAPH 1997 with the Navier-Stokes momentum solver replaced by gated diffusion — the real-time-feasible, replayable form per Van Laerhoven CAVW 2005 + the TAMU GPU-watercolor thesis; researched, cited inline). `DiffusionGrid` holds water + pigment + a static paper-height map; `step()` does: wet-gate `smoothstep(w_lo,w_hi,water)·permeability(height)` → conservative divergence-form gated Laplacian (blooms) → gated upwind advection along `flow = −β·∇height − λ·∇water` (downhill paper-channeling + the Curtis FlowOutward wet→dry push = edge darkening + backruns) → evaporation (drying freezes pigment); `splat()` re-wets + deposits (paint-into-wet re-blooms). Pure arithmetic, CFL-stable (`D·dt ≤ 0.2`). Tests: mass-conservation, determinism (HR-5), wet-blooms-vs-dry-stays-crisp (the gate), 200-step stability, drying-freezes. Visually confirmed (`visual_smoke_watercolor_v2_diffusion`): a stroke crisp on dry paper, blooming where the paper is wet, over a 0/14/40-step sequence.

**Decision (Enio, 2026-06-07): full live W15 (option c).** Landed incrementally:
- **W15.1 — `Tool::on_tick` contract hook (LANDED).** Frozen `Tool` cap 10→11 ([ADR-0040-amendment-2](0040-tool-as-isolated-feature-crate.md)): `fn on_tick(&mut self, dt_ms) {}` (default no-op → the 8 satellite tools are unchanged). The desktop shell calls `active_tool.on_tick(frame_ms)` 1×/frame with the real frame delta. Gate updated; CLAUDE.md §6 = `Tool=11`.
- **W15.2 — `PainterTool` live CPU diffusion (LANDED).** `wet_field: Option<DiffusionGrid>` allocated at `begin_stroke` when `brush.rendering.fluid_enabled`; dabs **splat into the grid** (re-wet + pigment) instead of the canvas; `queue_pointer` + `on_tick` step the diffusion + composite it (bilinear upsample + density→alpha over the pre-stroke backdrop) into the canvas, so the wash blooms wet-on-wet AS you paint and keeps evolving + drying after pen-up. The field is dropped once the wettest cell falls below the gate (the wash "sets"). Low-res grid (canvas/4); full-canvas CPU composite (bare-paper pixels short-circuit). `fluid_enabled` default false ⇒ zero behaviour change (all suites green). Tests: `fluid_brush_blooms_live_and_dries_on_tick`, `non_fluid_brush_allocates_no_wet_field`; visual `visual_smoke_watercolor_v2_live` (the stroke blooms + dries over idle ticks).
- **W15.3 — GPU port + device tiers + budget + det-fallback (pending).** The ADR-0049 `ph2d-painter-fluid` crate: WGSL compute solver, `fluid_capable()` gating, bbox-scoped upload, CPU 256² det-fallback for replay. The CPU path above is the reference + low-tier fallback.

Build order (engineering): the core (D11) → W15.1 contract → W15.2 CPU live → W15.3 GPU. Original options recorded for history: 
- **(a) Pen-up "deep diffusion bleed"** — run the solver to partial convergence in `end_stroke` (no contract change; a richer replacement for the v1.5 one-pass outward bleed — real backruns + channeled flow). Ships now, safe, but the bleed appears on pen-up, not while painting.
- **(b) Live-while-painting** — step the solver inside `queue_pointer` (an existing hook, no contract change) so the wash blooms as you move; hot-path perf must be validated (256² grid × few substeps ≈ a few M ops/event). Does not evolve while the pen is held still.
- **(c) Full live (ADR-0049 W15)** — a per-frame `Tool::on_tick` hook (FROZEN `Tool` contract change → Coord + ADR amendment) + GPU port + device tiers + det-fallback, the true "paint stays wet" Fresco behavior. The largest, the ADR-0049-reserved vision.

`Brush.rendering.fluid_enabled` + `Stamp.wet_amount` + `FLAG_FLUID_SAMPLE` (ADR-0049 reserved-but-dormant) are the hooks any of these would light up.

### D12 — Watercolor v2 live: UI toggle + post-pen-up backdrop fix + lifecycle safety (LANDED 2026-06-07)

Makes the W15.2 live diffusion actually reachable + actually working.

- **Fluid toggle (Brush Studio → Rendering).** New `PAINTER_STUDIO_FLUID` checkbox, routed through the **uncapped** `BrushParam::Fluid` (bool via `>= 0.5`, mirroring Wet/Burnt Edges) — deliberately **not** a new `PainterUiEdit` variant, since that surface is frozen at ≤24 (ADR-0043 §2.3). Writes `brush.rendering.fluid_enabled`. Test `fluid_toggle_via_brush_studio_checkbox`.
- **Dead-feature fix (the headline claim of W15.2 was inert).** "Keeps blooming after pen-up" did **not** work: `end_stroke` consumes the composite backdrop (`pending_pre_stroke`, `take()`n by the undo stack), so `composite_wet_field` lost its backdrop and the post-pen-up `on_tick`s no-op'd — the canvas froze at the last in-stroke frame. `visual_smoke_watercolor_v2_live` couldn't catch it: it dumps a PPM with **no assertion**, and its three bands (+0 / +18 / +45 ticks) were measured byte-identical. **Fix:** a dedicated `wet_backdrop: Option<Vec<u8>>` snapshot taken at `begin_stroke`, separate from the undo pre-image so it outlives the stroke; `composite_wet_field` reads it; dropped in lock-step with `wet_field`. Now guarded by `fluid_wash_keeps_blooming_after_pen_up` (asserts the canvas evolves across post-pen-up ticks — fails on the frozen build).
- **Lifecycle safety (v1 edge case, now reachable via the toggle).** `undo` / `redo` / `set_source` drop `wet_field` + `wet_backdrop`, so a still-blooming wash can't re-composite onto a stroke that was backed out or a canvas that was replaced. Test `fluid_wet_field_dropped_on_undo_and_set_source`.
- **K–M subtractive composite.** The live composite blended the wash over the backdrop with a *linear* `over` — a wash glazed over existing colour came out a muddy average (yellow over blue → grey). Swapped to the frozen `pigment_mix::mix_prepared` (Kubelka–Munk, the same engine the stamp wash uses): the glaze now mixes subtractively (yellow over blue → green). Measured green-dominant `[81,138,67]` for a yellow wash over saturated blue — test `fluid_composite_mixes_subtractively_km`.
- **Edge quality (Enio report: "low resolution at the edges").** The wash edge was visibly blocky/stair-stepped at zoom — the cost of a **bilinear** readout of the 1/`WET_FIELD_SCALE` field. Three changes, measured (release, 1024², full-width stroke = worst case):
  - **Bicubic (Catmull-Rom) upsample** of the pigment field (`sample_pigment_bicubic`) — C1 interpolation dissolves the grid facets into a continuous falloff. Bicubic alone wasn't enough; the grid was genuinely too coarse.
  - **`WET_FIELD_SCALE` 4 → 2** — the real crispness lever (half the block size). Bloom distance per step halves in canvas px (tuning is tighter, looks controlled); the diffusion constants were left as-is.
- **Colour-independent coverage (Enio report: yellow/magenta opaque + "cobrem tudo", blue/red correct).** Coverage was `alpha = 1 − exp(−dens·K)` with `dens = Σ(linear pigment mass)`. Since the grid stores colour×amount, `dens = amount · Σcolour` — i.e. luminance-weighted: yellow's `Σcolour ≈ 1.4` vs blue's `≈ 0.53`, so for the same pigment load yellow saturated to fully opaque while blue/red stayed a proper translucent wash. Fix: normalise by the stroke colour's linear sum to recover the colour-independent `amount`, then `alpha = 1 − exp(−amount · K)` with `K = 1.06` anchored to the (already-correct) blue/red look. Now identical loads give identical opacity across hues (measured: blue/red/yellow/magenta centre alpha all `93`), so bright washes are translucent and overlaps blend instead of burying. Tests `fluid_coverage_is_color_independent` + gated `visual_smoke_fluid_color_swatches`.
- **Straight-alpha glaze (Enio report: "bordas escuras" on a new top layer).** The composite blended the pigment over the backdrop's RGB, so on a transparent layer — backdrop `(0,0,0,0)` — partial-coverage edge pixels mixed toward BLACK and showed a dark fringe once the layer composited below (measured: a coral edge pixel `[60,0,0,124]`). K-M subtractive mixing is physical only over opaque *paint*, so the composite now blends between a porter-duff straight "over" (pigment colour at the edge, `out_a = a + back_a(1−a)`) and the K-M colour, **by the backdrop's own alpha**: transparent edge → pure pigment (no fringe), opaque paint → full subtractive mix. Edge pixel now `[255,179,140,18]` (clean coral); the green-over-blue K-M case is unchanged (`[133,174,69]`). Tests `fluid_no_dark_fringe_on_transparent_layer` + gated `visual_smoke_fluid_on_transparent_layer` (over a grey|white split).
  - **Composite perf, now that those raised the cost:** the 16-tap bicubic ran on *every* canvas pixel before the `dens` short-circuit, and `prepare_pigment` (spectral) ran per-pixel. Fix: **`prepare_pigment` amortised once** per composite (a stroke is one colour, so the chromaticity is constant — exactly what the API hoists), and the composite **scoped to the wet bbox** (`wet_pigment_bbox`, unioned with the previous frame so dried cells reset to the backdrop). Net: 46 ms → **12.8 ms** per frame worst-case (well under 16 ms); typical strokes far less. Big-canvas (≥2048²) perf is bounded by the full-grid `step()` (not the bbox-scoped composite) → the W15.3 GPU port / active-region stepping. Repro/inspect via gated `visual_smoke_fluid_edge_quality`.
- **Contract surface untouched** — `BrushParam` is uncapped; `PainterUiEdit` / `Brush` / `Stamp` counts unchanged (`architecture_painter_contract_surface` + `architecture_tool_contract_surface` green).

## Consequences

- **ABI/contract untouched** — every change rides existing `Brush`/`Stamp`/flag fields; the `architecture_painter_contract_surface` gate is unaffected.
- **Determinism (HR-5)** preserved — pressure curve eval, the settle blur, and the FBM rim noise are pure arithmetic seeded by the stroke seed; no new `det_random` axes.
- **Performance** — the settle is a one-shot separable box-blur over the stroke bbox on pen-up (not the hot path).
- **Spec** — `01_brush_engine.md` §1.3.6 (wet_edges/burnt_edges) should cross-reference this ADR's transport model over the old "accumulate ink at edges" prose.
- **Follow-ups:** D4 (dab AA, highest-value remaining), D5 (taper UI + One-Euro), and a Brush Studio **Pressure/Taper/Stabilization** section to expose D1/D5.
