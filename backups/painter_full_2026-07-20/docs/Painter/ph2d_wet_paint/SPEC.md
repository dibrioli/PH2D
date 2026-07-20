# PH2D Wet Paint — Behavioral Specification (v1.0)

> **What this is.** The complete behavioral specification for **PH2D Wet Paint**, a standalone
> watercolor painting app (HTML + vanilla JS, no dependencies, no build step). It describes a
> shallow-water / capillary fluid simulation with a two-layer pigment model, paper-tooth
> granulation, gravity drips, wet-on-wet bleeding, a porous-bristle brush engine, and the standard
> tool set.
>
> **Clean-room rule (non-negotiable).** This document is the implementer's **only** source. Do NOT
> consult any other watercolor/painting codebase, any other folder under `docs/Painter/`, any
> project memory notes about watercolor apps, or any external painting app's source. Choose your
> own architecture, module decomposition, and identifier names — descriptive English names
> throughout. Where this spec gives a constant, treat it as a *calibration value* with the stated
> semantics; where it gives math, treat it as *the required behavior*, and write your own code for
> it.
>
> The physics family is classical published work: Jos Stam's stable fluids (SIGGRAPH 1999),
> Curtis et al. "Computer-Generated Watercolor" (SIGGRAPH 1997), Kubelka–Munk pigment theory
> (1931/1948). This spec composes those ideas into one concrete, calibrated behavior.

---

## 0. Product overview

- Canvas: **900×450** paintable pixels, one simulation cell per pixel.
- The user paints with a round/flat/fan brush that deposits **suspended pigment** and **water**.
  While the pointer is down the fluid sim is paused (paint lands, nothing flows); on release the
  sim runs: water levels out, follows the paper grain, bleeds on wet areas, drips under gravity
  (tilt), and dries — suspended pigment settles into a **fixed** layer with darkened edges
  (the classic watercolor rim).
- Tools: paint, erase, smear, blend, wet, dry, blow. Canvas-wide actions: wet canvas, dry canvas,
  fast dry, show wet, clear. Up to 4 paint layers, per-layer undo/redo, PNG export, EN/PT chrome,
  a live tuning panel exposing ~39 calibration knobs.
- Everything is deterministic: no `Math.random()` anywhere in the engine (seeded RNGs and integer
  hashes only), so identical input scripts reproduce identical states.

## 1. Architecture requirements

1. **ES modules, no build step.** Runs from any static file server (`python3 -m http.server`).
2. **DOM-free engine.** Every simulation module (grid, solver, brush stamp, stroke, deposit,
   tools, paper, tuning registry) must import cleanly in Node (`node --input-type=module`) with no
   `document`/`window` references. Only the app shell (entry point, UI wiring, render-to-canvas,
   export) may touch the DOM.
3. **A Node smoke test** (`test/smoke.mjs`, run as `node test/smoke.mjs` from the app folder) that
   implements §18 and exits nonzero on failure.
4. Module decomposition, file names, class/function names: **your choice**. Keep files readable
   (< ~600 lines each). Comment the *physics semantics*, in your own words.

## 2. Data model

The paintable canvas is W×H (900×450). All per-cell arrays are **padded**: stride `S = W + 2`,
`rows = H + 2`, one border cell on every side, index `i = x + y·S`. Interior cells are
x ∈ [1..W], y ∈ [1..H]. **Brushes may only touch x ∈ [2..W−1], y ∈ [2..H−1]** — the outermost
interior ring (x or y equal to 1 or W/H) is a *drain* the boundary pass wipes every frame (§6.7),
which is what lets a drip run off the sheet edge instead of pooling there.

Per-cell state (Float32 arrays unless noted):

| field | meaning | range |
|---|---|---|
| water film | free water depth | 0..waterCap (default cap 8) |
| suspended mass | wet pigment mass, moves with the flow | 0..3000 scale (soft cap, see §10) |
| suspended color R,G,B | color of the suspended pigment | 0..255 floats (never quantize) |
| settled mass | pigment dried onto the paper, does not move | same scale |
| settled color R,G,B | color of the settled pigment | 0..255 floats |
| persistent velocity x,y | carried frame to frame; **gravity accumulates here** | clamped ±1 |
| transient flow x,y | rebuilt from the persistent field each frame; what mass actually moves along | clamped ±1 |
| wetness | "the sheet is damp here" byte, quantized to n/255 | 0..1 |
| paper height | the tooth, baked from the tile (§4) — **bake the padding ring too** | 0..1 |
| active mask (Uint8) | 0 = skip, 1/2 = solver processes this cell | |
| bloom budget (Uint8) | per-cell counter for the optional backrun extension (§17) | |

Plus: an active bounding box (the solver iterates only inside it), a `hasFluid` flag, the shared
opacity table (§3), and the paper tile index/preset.

**Two pigment layers is the core model:** the brush deposits SUSPENDED; drying transfers
SUSPENDED→SETTLED (§6.2); re-wetting lifts a little SETTLED→SUSPENDED. On-screen opacity of each
layer is read from the saturating opacity table.

**Two velocity fields is the core trick** (why drips run): gravity lands in the *persistent*
field; the absorbency brake (§6.3) is applied only while rebuilding the *transient* field, and
only on 1 frame in 4 — on the other 3 frames the persistent velocity passes through unbraked, so
a drip keeps advancing instead of being braked to a stall.

## 3. Opacity model

Pigment coverage is NOT linear in mass. Precompute `alpha[m]` for integer m in [0..3000]:

```
alpha[0] = 0
alpha[m] = 0.002 + 0.998 · alpha[m−1]        (i.e. alpha(m) = 1 − 0.998^m)
alpha[3000] = 1                              (force the last entry to exactly 1)
```

Every consumer clamps its (truncated-integer) index into the table; mass ≥ 3000 therefore reads
fully opaque. Half-opacity sits near mass ≈ 346. This saturating curve is the single reason paint
reads as absorbed watercolor instead of acrylic: a light wash (mass 100–400) has alpha 0.18–0.55
and the paper shows through.

## 4. Paper model

A **512×512** height tile in [0,1], tiled across the canvas with integer wrap
(`tile[(y & 511)·512 + (x & 511)]`, nearest texel, no interpolation), baked into the full padded
canvas array (including the pad ring — border cells must read a real tooth or edge capillary
diverges). Double-click cycles among 4 sheet indices; three **presets** select the tile
statistics. Deterministic per (index, preset): seed the RNG from them.

**Target statistics** (acceptance-tested, §18.6). These match scans of real watercolor sheets:

| preset | mean | std | mean |∇| (central difference) | grain direction |
|---|---|---|---|---|
| cold press (default) | 0.543 | 0.150 | ≈ 0.121 | isotropic-ish |
| rough | 0.517 | 0.192 | ≈ 0.144 | strongly vertical |
| hot press | 0.497 | 0.106 | ≈ 0.098 | isotropic-ish |

**Required structure** (what the capillary term needs): the relief is a fine **quasi-periodic
felt** — narrow grooves ~1–2 px wide, broken into short segments (~5–15 px), densely packed
(~5–7 px apart), with one dominant orientation; ≥ ~95% of the height variance below 8 px
wavelength; the gradient stays high after a 3 px blur (sustained slopes, not speckle). Long
smooth valleys or large-scale basins are a defect: they make drip fronts descend as fused wedges
instead of fingering into rivulets.

**Suggested recipe** (any construction hitting the stats + structure is acceptable):
1. Sum three seamless-tiling components: a faint large-scale undulation (e.g. a 6×6 bilinear
   value-noise lattice, amplitude ~0.02), a fine tooth (e.g. a 256×256 lattice, amplitude
   ~0.2–0.4 by preset), and a per-pixel integer-hash sparkle (amplitude ~0.06–0.09).
2. Stamp **thousands of short fiber strokes** (cold ~9000, rough ~11000, hot ~5000): each walks
   5–13 steps of ~0.7 px along `grainAngle ± jitter` (rough: vertical, jitter ±0.25 rad; others
   near-isotropic), width ~0.4–1.1 px half-width, mostly *grooves* (negative) with ~38% bright
   ridges, soft round cross-section, additive with wrap.
3. **High-pass**: subtract ~0.55 × an 8-px wrap-around box blur (separable), to push ≥95% of the
   variance below 8 px.
4. **Contrast-stretch** to the preset's target std around the target mean; clamp to [0,1] (the
   clamp mints the sparse 0/1 tails real scans have).

Live paper knobs (§16): contrast (× std target), fibre count (× strokes), groove depth (× stroke
depth). Rebaking happens on slider release. A separate **render-only** "paper visibility" control
(§13) fades the tooth in the appearance without touching this array.

## 5. Simulation loop & cadences

Fixed timestep at **40 Hz** (accumulator over `requestAnimationFrame`; clamp the accumulator to
5 steps so a background tab drops frames instead of fast-forwarding). Pointer input is **decimated
to the sim clock**: record the latest pointer position; feed the stroke engine exactly one
`move()` per 40 Hz step while the pointer is down. (Stroke speed is measured in px/frame of this
clock; feeding per pointer-event or per rAF changes the pressure model and over-deposits.)

Per sim step, with a frame counter `n` (increment first) — **the sim is paused while the pointer
is down** (deposition still happens through the stroke path), **except** when the active stroke
is the `blow` tool, which needs the sim live; drips/bleeding run on release:

| cadence | pass |
|---|---|
| every 2nd frame | rebuild the active region (§6.1) — if no fluid remains, idle |
| every `dryEvery` frames | drying/settle/re-wet (§6.2) |
| every 4th frame | flow-field build **with brake** (§6.3) |
| all other frames | cheap velocity smoothing, **no brake** (§6.4) + optional diffusion (§17) |
| every frame | advection + gravity injection (§6.5), then boundaries (§6.7) |
| every 3rd frame | pressure projection (§6.6), then velocity boundaries again |

Adaptive drying cadence from the previous advection's max |velocity component| `vmax`:
- calm (`vmax < 0.5`): `dryEvery = 6`, `evapScale = 0.001`, `rewetBase = 0.0001`
- flowing: `dryEvery = 3`, `evapScale = 0.00025`, `rewetBase = 0.000025`

## 6. Solver passes

All passes iterate the active bounding box only. Below, `L/R/U/D` are the 4-neighbours,
`film` = water at the cell, `susp`/`sett` = the two pigment masses, `wet` = the wetness byte,
`paper` = paper height, `A(m)` = opacity table lookup (truncate m to int, clamp to table).

### 6.1 Active-region rebuild

Two passes over the (previous) padded bbox; also derives the fresh bbox.

- First clear the active mask over the previous bbox padded by ±2 (clamped to [1..W]/[1..H]).
- **Pass 1 — wet cells:** scan x ∈ [max(2, bx0) .. min(W−1, bx1)], y ∈ [max(3, by0) ..
  min(H−2, by1)] (one row/col *inside* the brushable area: the drain ring only ever activates via
  the skirt below). Where `film[i−1] + film[i] + film[i+1] > 0`, set active = 1 on those three
  cells. If nothing fired: empty the bbox, clear the mask, `hasFluid = false`, return.
- **Pass 2 — the skirt:** scan top→down over y ∈ [pass1.y0−2 .. pass1.y1+2] (clamped to [1..H]),
  x over the ±2-padded x-range. Where the vertical triple `active[i−S] + active[i] + active[i+S]`
  sums to **exactly 1**: set `active[i−S] = 2`, `active[i] = 2`, and `active[i+S] = 2` if it was
  0 — and record the fire position (x, y) into the fresh bbox (the fire row itself, not y±1).
  Note the 2s written by earlier rows **count** in later sums; that interaction is load-bearing
  (an isolated front gets a full ±2 skirt and can run 1 cell/frame; a train of stripes with small
  gaps starves its own skirt and waits for the follower to merge — this keeps a wide front from
  decomposing into permanent horizontal bands).
- New bbox = fire extent padded ±5, clamped to [1..W]/[1..H]; `hasFluid = true`.

### 6.2 Drying, settling, re-wetting

For every bbox cell (do NOT filter by the active mask — dry every cell that has water or
suspended pigment; skip only `film ≤ 0 && susp ≤ 0`):

1. **Edge factor** `e` (drives edge darkening): if `film > 0` and `sett < 1000`, count the 3×3
   neighbours whose suspended mass > 10; `e = (count == 9) ? 1 : count/9`. Else `e = 1`.
   (Once a cell has settled ≥ 1000 the edge boost stops — saturated washes stop over-darkening.)
2. **Evaporate:** `newFilm = film · retention − evapBase · ((1−e) · edgeEvap + baseEvap)`, with
   `retention = 0.995`, `edgeEvap = 50`, `baseEvap = 2`, and `evapBase = evapScale` (from §5's
   cadence), or `1000` for one pass when a force-dry action is pending. If `newFilm < 0.0001`,
   set 0. `lost = 1 − clamp01(newFilm/film)`.
3. **Settle:** move `dm = susp · lost` from suspended to settled. The settled **color** updates by
   *opacity compositing* (not mass weighting): with `aSett = A(sett)`, `aIn = A(dm)`; if
   `aSett > 0`: `u = aSett · (1 − aIn)`, `inv = 1/(u + aIn)`,
   `settColor = (u · settColor + suspColor · aIn) · inv`; else `settColor = suspColor`.
   Then `sett += dm; susp −= dm` (clamp ≥ 0).
4. **Re-wet:** if `film > 0 && sett > 0`: the lift fraction grows with the *excess* water over
   what the suspended pigment already occupies:
   `excess = max(0, film − A(susp))`,
   `b = rewetBase · (1 + excess · 50)` (clamped to [0,1]), `lift = sett · b`.
   Suspended color updates by opacity compositing symmetric to step 3, with the incoming
   weight `A(sett) · b`. Then `susp += lift; sett −= lift`.

### 6.3 Flow-field build (every 4th frame — the braked frame)

For each active cell, start from the persistent velocity `(ex, ey)` and:

1. **Leveling** (water flows thick→thin): per axis add
   `clamp( (film[L] − film[R]) · 0.5 , ±0.2 )` to ex (resp. U/D for ey).
2. **Capillary** (only where `susp + sett < 2000` — thin paint): a *steepest-descent* pull toward
   the lower paper. Per axis (x shown; y symmetric with U/D):
   let `pl, pc, pr` = paper at L, center, R. The descent value is
   - if `pl > pc`: `(pc < pr ? pl − pr : pc − pr)`
   - else:        `(pc < pr ? pl − pc : pl − pr)`
   add `descent · 0.2` to ex. (This asymmetric form picks the drop across the cell in the
   direction of the steeper fall; it is what channels water into grain-following rivulets.)
3. **Viscosity:** if `film > 0.2`: `ex = 0.2·ex + 0.2·(vx[L]+vx[R]+vx[U]+vx[D])` (persistent
   field neighbours), same for ey.
4. **Wetness stamp:** if `film > 3`, write the wetness byte from the paper:
   `wet = min(1, 2 − 2·paper)` **quantized to byte resolution** (`(v·255 | 0)/255`). Valleys
   (low paper) read wetter, which keeps flow running along established wet channels.
5. **Look-ahead absorbency brake:** probe `look = 4` px downstream:
   `s = look / (√(ex²+ey²) + 0.01)`; probe index `p = i + trunc(ex·s) + trunc(ey·s)·S`
   (truncate toward zero; **linear index arithmetic**, no 2-D clamp). If `p` is inside the array:
   `brake = clamp( film[p] + 3·wet[p] − bias , 0.05 , 1 )` with `bias = 1.5`; multiply ex, ey by
   it. A probe past the array end skips the brake entirely (flow reaching the sheet edge runs
   off it); a probe landing on the pad reads film 0 / wet ≈ 0 and stalls naturally.
6. Optional extensions hook here (fingering pre-brake, backrun post-brake — §17).
7. Write the transient flow = `(clamp(ex, ±1), clamp(ey, ±1))`.

### 6.4 Velocity smoothing (the other 3 frames — never braked)

For each active cell: if `film > 0.05`, transient flow = `clamp(0.2·v_self + 0.2·Σ v_4neighbours, ±1)`
per axis (persistent field in, transient out); else transient = clamp(persistent). No leveling,
no capillary, no brake — whatever gravity the persistent field carries passes straight through.

### 6.5 Advection + gravity (every frame)

Semi-Lagrangian **conservative gather** along the transient flow. For each bbox cell:

- Non-active cells: zero their persistent velocity, continue.
- `(ux, uy)` = transient flow here; update the running max |component| (drives §5 cadence).
- Back-trace source `(sx, sy) = (x − ux, y − uy)`. If the source leaves [1..W]×[1..H]: **keep the
  old persistent velocity and move nothing** (a drip that reaches the sheet edge keeps its
  momentum). Otherwise compute the 4 bilinear corners and weights `w00..w11`.
- **Persistent velocity** = bilinear sample of the *transient flow* at the source, **then add
  gravity**: `v += gravityVector · film[i]` (unbraked, into the persistent field).
- **Mass gather:** pull `p_k = susp[corner_k] · w_k` and `c_k = film[corner_k] · w_k` from each
  corner. If total pulled pigment < 0.00001 treat as 0. Incoming color = mass-weighted mean of
  the corners' suspended colors using the **pre-clamp** weights. Subtract each `p_k` at its
  corner, clamping so a corner never goes negative (reduce that `p_k` by the shortfall). Add the
  (clamped) total here. **The destination's suspended color is REPLACED by the incoming mean**
  whenever any mass arrives (a fast rivulet that delivers even a little mass takes the cell's
  color — this is what makes color fronts move; blending with the resident mass freezes color
  where heavy pigment sits). No cap on the destination mass or water — fronts pile up (that is
  the rim/backrun raw material; only the brush deposit is soft-capped, §10). Water moves the
  same way (subtract at corners with clamping, add here).
- After the loop: apply the full boundary pass (§6.7).

Gravity: the tilt control provides a vector; magnitude default 0.005 (tilt boots ON, pointing
straight down). The `blow` tool also injects into the persistent field (§11).

### 6.6 Pressure projection (every 3rd frame)

One cheap Jacobi relaxation toward incompressibility (kills piling-up, leaves the uniform
downward drift intact so drips survive):

1. Divergence scratch: `d = vx[L] − vx[R] + vy[U] − vy[D]` per active cell.
2. Pressure scratch: `p = (d + 0.25·(d[L]+d[R]+d[U]+d[D])) · 0.25` (one relaxation).
3. Subtract the gradient, under-relaxed: `vx −= 0.5·(p[R] − p[L]) · 0.7`, same for vy — then a
   guard: if the cell's back-trace `x − vx` (resp. `y − vy`) leaves [1..W]/[1..H], zero that
   component. (The two scratch fields may reuse the transient-flow arrays; they are rebuilt next
   frame.)
4. Re-apply the velocity boundaries (§6.7, velocity part only).

### 6.7 Boundaries — the drain

The band is **two cells wide on every side**: the pad ring {0, W+1}×{0, H+1} plus the first
interior ring {1, W}/{1, H}. Every frame (after advection):

- Water and suspended mass on the band: **zeroed** (mass that advected onto the drain is deleted —
  the drip ran off the paper).
- Wetness on the band: the faintest non-zero byte value, `1/255`.
- Velocities on the band: tangential component zeroed on both rings first, then the normal
  component written **outward** at ±0.1 (left band −0.1, right band +0.1, top −0.1, bottom +0.1;
  writing normal after tangential leaves the corners carrying the normal value). This soft
  outward bias is what keeps the solver stable at the edge.

## 7. Brush stamp

The stamp at a canvas pixel = `radialFalloff(d, hardness) × bristleTexture(dxPx, dyPx)`, where
`d` = normalized distance from the dab center (0..1; reject ≥ 1 using squared distance before any
sqrt) and `(dxPx, dyPx)` = the pixel offset from the dab center in *canvas pixels*.

**Radial falloff:** `a = (1 − d) · hardness`; result = `a ≥ 1 ? 1 : smoothstep(a)` (with
`smoothstep(t) = t²(3 − 2t)`). Hardness for tools is a fixed 4.8; the paint dab passes its own
speed-driven value (§9) — a slow, heavy dab is a hard flat-top, a fast flick a soft puff.

**Bristle texture — porosity is load-bearing.** A 128×128 tile, built once per knob change,
deterministic seed:
- A near-zero *felt* floor between the hairs: uniform noise scaled so the felt's mean equals the
  `felt` knob (default **0.01**).
- ~**950** bristle *tips*: random centers, radius `(0.4 + rng·0.7) · tipSize`, strength
  `(0.2 + rng·0.25) · tipStrength`, soft round cross-section (smoothstep), **max-blended** onto
  the felt.
- Calibration target: texture **mean ≈ 0.02**, with only ~6% of texels above 0.04 — i.e. ~94% of
  the footprint deposits nothing. This porosity is what lets a painted blob stay a sieve of wet
  channels so the solver nucleates discrete rivulets at its edge and the reservoir keeps its
  pigment; a uniform film deposits a slab that drips as one fused curtain.
- **Deposit-integral calibration (binding — the mid/high band of the tip distribution).** The
  quantiles above under-constrain what the paper gate integrates. Define the single-dab integral
  at a tile position (cx, cy) as `Σ ( min(1, falloff(d, 9) · tex(cx+dx, cy+dy) · 2.415) − 0.3 )⁺`
  over the 25×25-texel disc (radius 12). Requirements:
  - at the tile origin: **≈ 4.0 (±15%)**, with **13–23 texels** clearing the gate;
  - **uniformity (binding):** ALL tip populations are placed uniformly at random over the whole
    tile — no deliberate placement, no reserved regions, no dependence on the origin. Assert it
    behaviorally: over ≥ 8 well-spread positions (e.g. a 3×3 grid of the tile), the integral's
    mean is **4.0 ± 25%** and its minimum is **≥ 1.5**. A brush of ANY radius must find the same
    strong-tip density anywhere in its footprint; concentrating the strong band near or away
    from particular rows/columns produces radius-dependent, one-sided strokes and is a defect.
  Tune the tip strength/size/count distribution (e.g. peak strengths reaching ~0.45 and
  cross-sections wide enough to hold the 0.12–0.45 band) until this and the mean/porosity
  targets hold simultaneously.
- Sampled nearest-texel with wrap, **anchored to the dab center in canvas px** (1 texel = 1 px at
  every brush size, so grain has constant physical size and the tips streak with the stroke).

**Brush shapes:** round = plain disc. flat/fan = an ellipse in the stroke frame: rotate the
normalized offset by the stroke direction, divide the along/across components by
(flat: 1 / 0.35 · r, fan: 1.4 / 0.5 · r), reject where the normalized radius ≥ 1; texture coords
are the rotated px offsets. With no stroke direction yet, fall back to the disc.

## 8. Stroke engine

Feed exactly one pointer sample per 40 Hz sim step while the pointer is down.

**Synthetic pressure** `t` (there is no pen pressure; slow = heavy):
- Starts at **0** on press (a stroke fades in over ~0.25 s instead of landing as a blob).
- Each frame: `t' = 11 − min(8, (t + speedPrevFrame)/2)`, then slew-limit the change to
  ±0.5/frame, floor at 0. `speedPrevFrame` = the *previous* frame's pointer speed in px/frame
  (the recurrence reads speed one frame late). Steady range works out to [3, 7.33]: holding
  still builds toward 7.33; a fast drag drops toward 3.
- **Release tail:** after pointer-up keep ticking at 40 Hz: `t = (t − 0.5) · 0.6` per frame,
  emitting geometry while the cursor is still moving (last hop ≥ 1 px) and `t > 0.01` — the
  stroke lifts off with a fading tail. Then flush the spline and drop the trail remainder (§10).
- Geometry only advances when the frame's hop ≥ 0.5 px (pressure still ticks while holding).

**Geometry:** a Catmull-Rom spline through the frame samples (standard uniform form; first/last
control points duplicated). Walk each segment in `max(2, ceil(chordLength))` micro-steps,
accumulate arc length, and emit a dab every `spacing` px (default **2**) regardless of speed.
Each dab's pressure `b` = lerp between the segment endpoints' `t` values by the micro-step
fraction. On stroke end, duplicate the last point once to flush the spline tail.

A per-frame-segment callback reports the chord length *before* that segment's dabs (the trail
engine sizes its window with it, §10).

## 9. Dab parameters

From the UI sliders and the dab's synthetic pressure `b` (computed once per dab):

- `pressureBase = 0.2 + 0.7 · pressureSlider` (∈ [0.2, 0.9] — even minimum pressure deposits).
- `intensity = clamp(b · 0.5 · pressureBase, 0, 3) · intensityKnob` (knob applied post-clamp).
- `waterUnit = waterSlider · 0.35`;
  `waterAmount = 2 · waterUnit² / (pressureBase + 0.01) · waterGainKnob` — **pressure-inverse**:
  a light touch is wetter, a heavy slow stroke drier-but-denser.
- `hardness = min( clamp(0.5·b, 1, 3), r/2 )²` — slow ⇒ 9 (crisp flat-top), fast ⇒ ~2 (soft
  puff); small brushes are always soft.
- Dry-brush gate (only feeds the optional §17 extension):
  `dry = clamp01(1 − waterAmount/1.5) · clamp01(((11 − b) − 4)/4)`.
- Brush radius from the size slider: `r = 2 + slider · 33` px (range [2, 35]).

## 10. The paint trail (two-step deposit)

Painting does NOT stamp each dab straight into the canvas. Dabs accumulate into a **stroke-local
trail buffer**, and the trail lands on the canvas once per *window* — a continuous footprint
instead of a chain of beads, and the place where the wet-brush feel lives (tip memory, paint
drag, opacity-composite color).

**Trail state (per stroke):** pigment-mass trail + water trail (square Float32 buffers sized to
cover the max radius 35 plus the max window drift ≈ 4·spacing, i.e. half-size
`ceil(35 + 4·6) + 2`), a brush-tip **color buffer** of the same footprint (R,G,B), the window
anchor (rounded canvas position of the window's first dab), the previous anchor (drag source),
a dab counter, the window size C, and the trail's touched-extent bbox.

**Window sizing:** at each frame segment, `C = min(cap, floor(chordLen / spacing))` — cap **2**
for paint, **4** for blend. A slow stroke transfers every dab (C = 0); a fast one accumulates its
segment and lands it in one piece.

**Stroke start:** clear the trails; fill the tip buffer with the picked color; both anchors = the
press position.

**Accumulate (each dab, paint mode):** for each stamp pixel (respecting the brushable area
[2..W−1]×[2..H−1] and the trail bounds):
- `stamp = min(1, falloff · texture · intensity)`; skip ≤ 0.
- **Paper gate:** `tooth = (susp + sett < pigCap 2000) ? paper : 0.45`;
  `deposit = stamp − (1 − tooth) · gate` (gate default **0.6**); skip ≤ 0. On tooth peaks the
  subtraction ≈ 0 so peaks always take pigment even at minimum pressure; valleys reject it —
  that per-pixel pass/reject IS the granulation.
- **Wetness seed:** OVERWRITE `wet = min(1, 2 − 2·tooth)` (overwrite, not max — repainting can
  dry the byte back down).
- Trail pigment += `deposit · pigmentGain 600` (× the optional per-pigment density/drybrush/
  wet-softening modifiers, §17); trail water += `deposit · waterAmount`. Track the trail bbox.

**Accumulate (blend mode):** a *saturating* mask instead: `m ← m·(1 − e) + e` with
`e = (stamp_clamped − (1 − tooth)·gate) · blendStrength 0.8` — scrubbing the same spot builds
toward a total mix. No water, no tip, no wetness writes.

**Transfer (lands when the dab counter exceeds C; also rolls the window):**
1. **Tip self-cleaning:** every tip texel lerps toward the stroke's base color by
   `tipClean 0.001`.
2. **Tip pickup** (the dirty brush): for each window cell, canvas coverage
   `fe = A(sett)·0.5 + A(susp)`; if 0 skip (blank paper keeps the tip color). Canvas color =
   the coverage-weighted mix of settled+suspended colors. Keep factor
   `k = tipRetain + (1 − tipRetain)·(1 − min(1, fe))` with `tipRetain = 0.995`; tip = tip·k +
   canvas·(1−k). Reads the PRE-deposit canvas. Painting through a wet pool drags that color
   along; the tip cleans itself back over ~hundreds of transfers.
3. **Cap shedding means:** window means of trail pigment and water over cells with pigment > 0,
   each × **1.05**.
4. **Land the window** (per cell with trail pigment v > 0, `inA = A(v)`):
   - Resident suspended mass > 0: color = **opacity composite** of tip over resident:
     `e0 = A(old)`, `e9 = e0·(1 − inA)`, `norm = 1/(e9 + inA)`;
     `suspColor = (suspColor·e9 + tip·inA) · norm`. Mass: `old < 3000 ? old + v :
     old + v − shedMean` (a **soft cap** — hovers at the cap instead of clamping); floor 0. If
     the settled mass exceeds 3000 it sheds the same mean.
   - Virgin cell: color = tip, mass = v, **and water is added twice** (fresh paper drinks its
     first wetting deeper): add the trail water here once, then fall through to the general add.
   - Water: `film < waterCap ? film + trailWater : film + trailWater − waterShedMean`; then keep
     a **trace film** floor of 0.00001 on every landed cell (the wet map must know the stroke
     passed).
   - **Paint drag:** also pull from the cell at the same window offset relative to the *previous*
     anchor (if inside the brushable area): masses and water lerp toward the source by
     `drag 0.16`; colors are copied from the source only where the destination had none; the
     wetness byte is dragged with **integer byte weights** — `(src·w1 + dst·w2) / 256` where
     `w1 = trunc(drag·255)`, `w2 = trunc((1−drag)·255)` — the weights sum to slightly less than
     256 on purpose, so each transfer leaks ~0.8% of the wetness (a float lerp never decays it).
5. Roll: previous anchor ← this anchor; clear the trail and its bbox.

Blend-mode transfer instead hands the saturating mask window to the blend apply (§11) and rolls
the same way. **Stroke end: the remainder window is dropped** (the release tail already emitted
the fade-out; landing the remainder would double the ending).

## 11. Tools (per dab; stamp = `min(1, falloff(d, 4.8) · texture · min(3, intensity))`)

Tools dispatch per dab (no trail), except blend which routes through the trail. All respect the
brushable area and expand the active bbox.

- **erase** — paper-gated multiplicative removal: `tooth` as in §10 (cap 2000 → 0.45);
  `g = stamp − (1 − tooth)·gate`; skip ≤ 0, clamp ≤ 1. `k = 1 − eraseForce 0.4 · eraseSlider · g`;
  skip if k ≤ 0 (never wipe). Suspended: `susp ·= k`, water `·= k`, snap susp < 1 → 0. Settled:
  `sett ·= k`, snap < 1 → 0. (Stale color bytes may remain; mass 0 hides them.)
- **wet** — drip-feed water: `film += stamp · 2 · (waterUnit · 0.2)² · waterGain`, capped at
  `waterCap`; **set** `wet = min(1, 2 − 2·paper)`. Gentle: one pass leaves a thin film; it
  accumulates over repeated passes.
- **dry** — `film ·= (0.999 − stamp · dryForce 0.32)`, floor **0.001** (never exactly zero);
  `wet = 0` — the dry tool SEALS the paper (stops future bleed there).
- **blow** — needs the sim live. Displacement since the previous dab, clamped **asymmetrically**
  per axis to [−0.6, +0.3] (weak, slightly anti-forward). Per stamp pixel: skip the whole pixel
  if the *unclamped, rounded* drag source offset falls outside [1..W]×[1..H]; weight
  `e = stamp · blowForce 0.2`; if `film > 0`: persistent velocity += clampedDisplacement · e and
  re-wet `wet = min(1, 2 − 2·paper)`. Independently, the wetness byte is **dragged** from the
  source offset (`wet ← wetSrc·blowForce + wet·(1−blowForce)`) wherever the stamp touches, even
  on dry cells — use a read-before-write scratch. Right-button drag = blow regardless of the
  selected tool.
- **smear** — drag suspended pigment + water from the real previous-dab offset (reach grows with
  speed). Skip the stroke's first dab (no displacement). **Snapshot** the affected region (+10 px
  pad) before writing — sampling live arrays compounds the drag within one dab; clamp the sample
  offset to the pad. Per pixel: `u = min(1, stamp · smearForce 0.8)`; source = **mass-weighted**
  bilinear sample of the snapshot's suspended mass/color (weighting each corner by its mass —
  an unweighted sample near an edge drags the color toward black). If source mass > 0: color
  composites by coverage (`inC = A(srcMass)·u`, `res = A(dst)·(1−inC)`, weight = inC/(inC+res)),
  mass lerps by u. Water lerps unconditionally (plain bilinear). **Only the suspended layer
  moves** — the settled layer is deliberately untouched (matching real smudge behavior; do not
  "fix" this).
- **blend** (via the trail's saturating mask window): compute window averages over mask-active
  cells only — suspended mass & mass-weighted color, settled mass & color, water, wetness. Then
  per mask cell (`a = clamp01(mask)`):
  - Suspended pass (gated on window suspended coverage > 0): incoming weight
    `inC = A(avgSuspMass)·a`, resident `res = A(susp)·(1 − inC)`; color lerps by `inC/(inC+res)`;
    `susp` and `film` lerp toward the window averages by `a`.
  - Settled pass (gated on window settled coverage > 0): same shape for settled color/mass — dry
    paint re-mixes — and the **wetness byte relaxes toward the window average** in this pass.

## 12. Canvas-wide actions

- **wet canvas** — raise the wetness byte to `min(1, 2 − 2·paper)` via **max** over the whole
  interior. Injects NO water, touches no bbox: the sim stays idle (this is why it never freezes
  the app), but subsequent strokes bleed everywhere and show-wet reads damp.
- **dry canvas** — one-shot O(area): settle all suspended mass into settled with the same
  opacity-composite as §6.2 step 3; zero water, both velocities, wetness; empty the bbox.
- **fast dry** — loop (guard ≤ 40): halve the water in the bbox, run the §6.2 pass with the
  force-dry flag, rebuild the active region; stop when no fluid remains. (No advection — it
  can't stall; edge darkening still forms because settle runs.)
- **clear** — zero all dynamic state (paper untouched).

## 13. Rendering

Composite the interior into an RGBA image (cell (x,y) → pixel (x−1, y−1); alpha 255). Support a
**dirty-rect**: render only a sub-rectangle (the caller tracks the changed region ±1 and requests
exactly that; appearance-knob changes request a full repaint).

Per pixel, with `pap = 0.5 + (paper − 0.5) · paperVisibility` (a render-only master; physics
never sees it):

1. **Base** (the sheet): `base = 255 + (pap·30 − 30)` in all channels (paper-tinted near-white).
2. If total mass > 0:
   - **Granulation offset** `v = (pap·100 − 40) · grainVisibility`, faded out linearly as total
     mass goes 1000 → 3000 (thick paint hides the tooth).
   - **Emboss**: signed first-derivative of total mass,
     `emb = clamp( ((massR − massL)·0.5 + (massD − massU)·1.0) · 0.008 · embossKnob, ±40 )`;
     `v += emb`. (Signed gradient — one side lightens, the other darkens: a soft bevel, not an
     outline. Vertical weighted 2× the horizontal.)
   - **Alpha-over, settled then suspended**: `aS = A(settMass)`, then
     `c = c·(1−aS) + (settColor + v)·aS`; same with the suspended layer on top. (The tooth
     offset `v` is added INTO the pigment color channels — grain shows *through* the color.)
   - Optional Kubelka–Munk glaze compositing replaces the two alpha-overs when enabled (§14).
3. **Show-wet overlay** (toggle): a realistic damp look, not a flat tint. Wetness signal
   `wetSig = max( smoothstep(min(1, film/2.5)) for film > 0.02 , wetnessByte )`. If > 0.01:
   darken cool — multiply R by `1 − 2.4·0.04·wetSig`, G by `1 − 1.6·0.04·wetSig`, B by
   `1 − 0.8·0.04·wetSig` (a subtle blue-gray cast). Plus a meniscus glint on the film's rim:
   gradient `(gx, gy)` of the film; if |∇| > 0.01, `rim = min(1, |∇|·0.7)`,
   `shine = 18.3 · rim² · (0.5 + 0.5·(∇̂ · lightDir))` added to all channels (light from
   top-left).
4. Optional neutral-by-default extras (§17): rake light, valley granulation, wet sheen, grain
   dither, edge tint.
5. Clamp to bytes.

**Layers:** the bottom layer renders through the full path above (paper base included). Each
higher visible layer over-composites *pigment only* (no second paper base; same granulation
offset + emboss math, alphas scaled by the layer's opacity). Single-layer must use the plain
path.

## 14. Color

- **UI color:** HSV wheel (angle = hue, radius = saturation) + brightness bar; standard HSV→RGB
  and back (for palette picks moving the wheel).
- **sRGB ↔ linear:** standard EOTF (`c ≤ 0.04045 ? c/12.92 : ((c+0.055)/1.055)^2.4` and inverse).
- **Kubelka–Munk subtractive mixing** (opt-in checkbox; classical single-constant K–M): per
  linear-reflectance channel, `KS(R) = (1−R)²/(2R)` (floor R at 1/255); mixtures are linear in
  K/S: `KS = (1−w)·KS(dst) + w·KS(src)`; invert `R = 1 + KS − √(KS² + 2KS)`. Work entirely in
  float (0..255 float channels → linear → mix → back), **never quantizing to bytes** (cells
  re-mix thousands of times; a byte round-trip drifts washes toward black). When the checkbox is
  ON, route these color blends through K–M: trail transfer composite, advection incoming color,
  settle, re-wet, smear, blend. Default OFF ⇒ the plain composites above.
- **K–M glaze layering** (opt-in): stack the pigment layers by reflectance instead of alpha-over.
  Per channel with layer coverage a: `Rt = a·Rtop`, `Tt = 1 − a`,
  `R = Rt + Tt²·Rbot / (1 − Rt·Rbot)` (guard the denominator; clamp [0,1]). Energy-bounded:
  a=1 → Rtop, a=0 → Rbot. Paper → settled → suspended, in linear light.

## 15. Layers, history, export, shortcuts

- **Layers:** up to 4. Each layer = a full independent field grid sharing the paper. Only the
  active layer simulates. Per-layer opacity slider + visibility toggle + select; add/remove.
- **History:** per-layer undo/redo ring, depth 6. Snapshot = every dynamic per-cell buffer + bbox
  + hasFluid + paper index (paper array rebuilt on restore if the index changed). Capture BEFORE
  each mutation (stroke start, canvas-wide action). Ctrl+Z / Ctrl+Shift+Z / Ctrl+Y.
- **Export PNG:** (a) the on-screen composite (with paper), or (b) pigment-only over transparent:
  alpha = combined coverage `aS + aF·(1−aS)`, RGB = settled-then-suspended straight-alpha
  compositing. Checkbox chooses; file names `ph2d-wet-paint.png` / `ph2d-wet-paint-pigment.png`.
- **Shortcuts:** `p e s n w d b` → paint/erase/smear/blend/wet/dry/blow; `[` `]` → brush size
  ∓0.03; guarded when focus is in an input.

## 16. UI

Dark, minimal, professional. Layout: **left panel** (color wheel + brightness + current swatch +
an 8-swatch named-pigment palette; brush/pressure/water/erase sliders; shape buttons round/flat/
fan + "wet brush"/"dry brush" preset buttons (pressure/water = 0.35/0.85 and 0.85/0.2); tool
buttons; tilt toggle + a **tilt dial** — a round pad with a polar grid (8 rings × 12 spokes),
dragging the knob sets the gravity direction+magnitude snapped to the grid, and implicitly turns
tilt on; paper preset buttons; layer add/remove + list with per-layer visibility/opacity). The
left panel is **resizable** — a drag handle on its right edge sets its width (mirroring the right
panel's resizer: pointer-capture drag, clamped to a sensible min/max, the center reflows), and its
width persists across the session. **Center:** the canvas, wheel-zoom (0.25×–8×, zoom toward cursor), middle-drag pan, a reset pill
showing the current %. **Bottom bar:** undo/redo · wet canvas/dry canvas/fast dry/show wet ·
clear/save png (+"paper" checkbox) · panel toggles + EN/PT. **Right:** a collapsible, resizable
**tuning panel**, open by default.

Boot defaults: tool paint, brush 0.7 (radius 25), pressure 0.7, water 1.0, erase 0.4, color
rgb(50, 140, 210), tilt **ON** pointing straight down, paper cold-press sheet 0, language **EN**
(toggle EN/PT; app chrome translated via a small string table; knob tooltips stay PT-BR).

**Tuning registry** — one module owning every live calibration knob: key, group, PT/EN label,
slider range (a comfortable suggestion), default, and an optional rebuild kind ("brush" = rebuild
the stamp texture, "paper" = re-bake the sheet on release, "render" = repaint). A free numeric
input beside each slider accepts values outside the slider range (NaN falls back to the default);
per-knob reset buttons (red-tinted when off-default) and per-group resets. The engine reads knobs
at runtime. The full set with defaults:

| group | knob | default | notes |
|---|---|---|---|
| paint | pigment per dab | 600 | §10 gain |
| paint | paper gate | 0.6 | §10 |
| paint | felt (pores) | 0.01 | §7, rebuild brush |
| paint | bristle count | 950 | §7, rebuild brush |
| paint | drag | 0.16 | §10 |
| paint | pickup | 0.005 | = 1 − tipRetain |
| paint | intensity | 1 | §9 post-clamp |
| paint | bristle strength | 1 | §7, rebuild brush |
| paint | bristle size | 1 | §7, rebuild brush |
| paint | spacing | 2 | §8 |
| paint | tip clean | 0.001 | §10 |
| paint | blend force | 0.8 | §10 blend |
| paint | gate saturation | 2000 | §10 pigCap |
| water | water per dab | 1 | §9 multiplier |
| water | water cap | 8 | §2 |
| water | evaporation | 1 | §6.2 multiplier |
| water | re-wet | 1 | §6.2 multiplier |
| water | retention | 0.995 | §6.2 |
| water | edge darkening | 50 | §6.2 edgeEvap |
| water | base evaporation | 2 | §6.2 |
| physics | leveling | 0.5 | §6.3 |
| physics | capillary | 0.2 | §6.3 |
| physics | brake | 1.5 | §6.3 bias |
| physics | gravity | 0.005 | §6.5 |
| physics | level clamp | 0.2 | §6.3 |
| physics | viscosity | 0.2 | §6.3 threshold |
| physics | max velocity | 1 | CFL clamp |
| physics | projection | 0.7 | §6.6 |
| physics | brake reach | 4 | §6.3 look |
| physics | capillary gate | 2000 | §6.3 |
| tools | eraser | 0.4 | §11 |
| tools | dryer | 0.32 | §11 |
| tools | blow | 0.2 | §11 |
| tools | smear | 0.8 | §11 |
| paper | contrast | 1 | §4, rebuild paper |
| paper | fibres | 1 | §4, rebuild paper |
| paper | grooves | 1 | §4, rebuild paper |
| paper | visual grain | 1 | §13, repaint |
| paper | emboss | 1 | §13, repaint |

Plus the render-only **paper appearance** master (eye toggle + 0–1.5 slider) in the paper group
header (§13 paperVisibility).

**Knob documentation:** every knob gets a rich hover tooltip (PT-BR) explaining what it does
physically and a concrete recipe ("washed & smooth: raise felt, lower gate, lower evaporation…"),
plus colored correlation letters tagging which of four subsystems it touches (paint deposition /
water / flow physics / render) at what strength. Author this content fresh from this spec's
semantics.

**Chrome documentation (left panel + bottom bar) — same treatment:** every user-facing control
outside the tuning panel ALSO gets a rich PT-BR hover tooltip through the same floating-tooltip
system: the color wheel + brightness bar, the pigment palette, each of the four brush sliders
(size / pressure / water / erase), the shape buttons and wet/dry presets, each tool button, the
tilt toggle and tilt dial, the paper preset buttons, the layer add/remove buttons, and every
bottom-bar action (undo/redo, wet/dry canvas, fast dry, show wet, clear, save + paper checkbox,
panel toggles, EN/PT). Each tooltip: emoji + title + a short physical explanation of what the
control does in the simulation + a concrete usage tip where meaningful (e.g. water slider: "wet
watercolor that flows: slider at max; dry gouache look: 0.2–0.4 — with the porous deposit,
scrub to soak the sheet"). The four brush sliders additionally carry the colored
subsystem-correlation letters, exactly like tuning rows. Author all content fresh from this
spec's semantics.

**Named pigments** (color-only presets; 8 swatches):
Hansa Yellow (250,205,40) · Pyrrole Red (220,45,40) · Quinacridone Rose (220,55,120) ·
Phthalo Blue (10,70,150) · Ultramarine (45,70,165) · Viridian (20,120,100) ·
Burnt Sienna (150,75,40) · Payne's Grey (55,65,85).

**Experimental panel** (collapsible): two visible checkboxes — pigment mixing (K–M) and glaze
layering (K–M). The §17 extensions ship implemented but with neutral defaults and no visible
sliders (leave the wiring commented/documented for future re-enablement).

## 17. Gated extensions (implement; ALL default-neutral — booting untouched must behave §5–§13)

- **Diffusion** (flow, smoothing frames): Fickian spread of suspended pigment through a still wet
  film. For each active cell with film > 0.1, symmetric flux to +x/+y neighbours (each edge once,
  mass-conserving): `flux = rate · (susp_here − susp_there)`, `rate = knob · min(1, film/1.5)`;
  color rides mass-weighted. Default knob 0 ⇒ pass skipped.
- **Backrun / bloom** (flow build, post-brake): where this cell is much wetter than a neighbour
  holding settled pigment (`filmGap > 0.8 ± 0.2·hashJitter(x,y)`), shove the flow toward that
  neighbour (`± knob · min(filmGap, 1.5) · 0.5` on the axis) and lift 10% of the neighbour's
  settled mass into its suspension (mass-weighted color). Per-cell budget: at most 6 blooming
  build-frames per fresh front (Uint8 counter; recharges when the front condition lapses or on
  dry/clear) — prevents sloshing from pumping thin lines. Deterministic integer-hash jitter
  crenellates the rim. Default 0.
- **Fingering** (flow build, pre-brake): when gravity is on, at the drip's leading edge (wet
  here, dry one gravity-step downstream) add a transverse sinusoidal velocity ripple with
  wavelength `max(4, W / rivuletCount 15)`; push down at the peaks (`amp · min(film, 2)`) plus a
  slight transverse component (×0.3) — the brake then self-selects rivulet columns. Default
  amp 0.
- **Physical granulation** (dry pass): bias the settled amount toward paper valleys:
  `dm ×= clamp(1 + knob·0.6·(0.5 − paper)·2, 0.3, 1.7)`. Default 0.
- **Staining** (dry pass): a bidirectional lift multiplier on re-wet — 0.5 neutral; toward 0
  lifts up to 8×, toward 1 lifts down to 0 (permanent). Default 0.5.
- **Dry-brush** (deposit): raise the gate subtraction by `dabDryness · knob · 0.6` (broken color
  on the tooth peaks). **Wet-edge softening** (deposit): on already-wet paper scale the dab's rim
  deposit by `1 − softness·(1 − falloff)`, softness = `min(1, film/3) · knob`. Defaults 0.
- **Render extras:** rake light over the paper gradient (clamp ±12, default direction top-left,
  angle knob); valley granulation (settled reads denser in troughs, `alpha(m·(1 + knob·3·
  max(0, 0.5 − pap)))`); wet sheen (specular on film 0.05→2.5 window, keyed on the paper
  gradient · light); static per-pixel grain dither (integer hash, ±0.5·knob); edge tint
  (bidirectional: mass-gradient window 100→1400 smoothstepped, ±130 max darken/lighten, slider
  0.5 = neutral). All default 0 / 0.5-neutral.

## 18. Acceptance tests (Node, engine-only; all with knobs at defaults)

1. **Pressure fade-in:** drive a 260-px straight stroke at 4 px/frame. First dab's pressure = 0;
   the mean of dabs 2–6 < 70% of the steady mean (last 15 minus tail 5); max pressure ≤ 7.34.
2. **Porous deposit:** on that stroke's band (radius 12), < 85% of band cells hold pigment; total
   mass > 0; no NaN in mass/color/water arrays.
3. **Wet & dry tools:** one wet pass over a line leaves a thin film (0 < film < 0.2) and a
   wetness byte > 0 at the line's center; 30 more passes accumulate to > 5× the first film. One
   dry pass then zeroes the wetness and leaves film ≥ 0.001.
4. **Blow:** on a pre-wetted patch (film 3, wetness 0.8), a blow stroke produces max |velocity_x|
   in (0, 0.31); no NaN in wetness.
5. **Advection semantics:** (a) a heavy cell (mass 2500) receiving flow from a light colored
   neighbour (mass 300) REPLACES its suspended color with the incoming one after one advect;
   (b) with two full cells (film 8) and flow into one of them, the destination's film exceeds
   the cap 8 (no cap during advection).
6. **Paper statistics:** per preset, |mean − target| < 0.02, |std − target| < 0.04, mean |∇|
   within 18% of target (targets in §4).
7. **Blend re-mixes dry paint:** paint a settled patch (mass 900) then blend-stroke across its
   edge: settled mass beyond the original patch boundary grows by > 100; no NaN.
8. **Tuning registry:** ≥ 39 knobs; group reset restores all defaults; free numeric input
   accepts out-of-range values; NaN input falls back to the default.
9. **Drip + stability:** a 20×20 wet block (film 8, mass 800, wetness 1) near the top of an
   80×300 grid, gravity (0, 1.5): after 240 sim steps the wet front (film > 0.1) has descended
   at least 40 rows below the block's initial bottom edge, and with gravity (0,0) it stays
   within 8 rows. 120 further steps: no NaN anywhere.
10. **Neutrality:** with all §17 knobs at their defaults, a scripted stroke + 60 sim steps
    produce identical field state whether the extension code paths are compiled in or bypassed
    (assert via checksum of the mass/water arrays computed with extensions force-skipped vs
    normal).
11. **Stroke budget at TWO radii (binding calibration):** on a 300×100 grid with default knobs
    and sliders (pressure 0.7, water 1.0), a straight 260-px stroke driven at 4 px/frame (no
    release tail, sim paused) must land, per radius:
    - **r = 12:** suspended mass **235 000 ± 12%**, water **245 ± 12%**, band coverage
      (fraction of cells with pigment in the ±r band around the path) **0.20–0.27**;
    - **r = 25 (the boot radius):** suspended mass **890 000 ± 15%**, water **835 ± 15%**, band
      coverage **0.33–0.45**.
    The §7 deposit integral (including its uniformity clause) must also hold. (These pin the
    perceived strength of a default stroke at both a mid and the boot size.)
12. **Lane structure (binding — the bristly look must be two-sided and radius-independent):**
    for the §18.11 strokes, sum the suspended mass per row over the central 220 columns of the
    ±r band. Strokes deposit in bristle LANES (streaky rows are correct — a uniform filled band
    is wrong), but the lanes must spread over the whole band on both sides of the path:
    - the upper half-band (rows above the path) carries **35–65%** of the off-path mass, at
      both radii;
    - rows exceeding 5% of the max row: **≥ 7 of 25** at r = 12, **≥ 20 of 51** at r = 25;
    - the longest run of consecutive rows below 2% of the max row: **≤ 6** at r = 12, **≤ 12**
      at r = 25.
    (Note: a dried stroke's row profile is deposit-lane-dominated; wash-boundary darkening is
    §6.2's edge factor and needs no separate stroke-level assertion.)

## 19. Deliverables & definition of done

```
docs/Painter/ph2d_wet_paint/
  index.html          (title: "PH2D Wet Paint")
  css/…               (your design)
  js/…                (your architecture; engine modules DOM-free)
  test/smoke.mjs      (§18; exit code 0)
  README.md           (your own words: what it is, how to run, module map)
```

Done when: `node test/smoke.mjs` passes; the app runs from a static server; painting, all seven
tools, tilt drips, wet/dry canvas, show wet, layers, undo/redo, export, EN/PT, and the tuning
panel all function; and the boot look is: pressure-faded strokes of porous bristly paint that
granulate on the tooth, bleed on wet paper, and — with tilt — run into thin grain-following
rivulets with darkened drying edges.
