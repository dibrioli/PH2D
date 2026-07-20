# PH2D Wet Paint

A standalone watercolor painting app — plain HTML + vanilla ES modules, no
dependencies, no build step. It simulates a shallow-water / capillary fluid on
a procedural paper sheet with a two-layer pigment model (suspended pigment
that moves with the flow, settled pigment stuck to the paper), a porous
bristle brush, synthetic stroke pressure, gravity drips under a tilt dial,
wet-on-wet bleeding, and the classic darkened drying rim. Implemented
clean-room from `SPEC.md` (the only source); the physics family is classical
published work (Stam's stable fluids, Curtis et al. watercolor, Kubelka–Munk
pigment theory).

## Run

```
cd docs/Painter/ph2d_wet_paint
python3 -m http.server 8000        # any static file server works
# open http://localhost:8000
```

Acceptance tests (Node ≥ 18, engine-only, DOM-free):

```
node test/smoke.mjs                # spec §18: exits 0 when all checks pass
node test/shell-boot-check.mjs     # auxiliary: boots the UI shell under a DOM shim
```

## Using it

Boot defaults: paint tool, round brush 25 px, pressure 0.7, water 1.0, blue,
tilt **on** pointing straight down, cold-press paper, EN chrome, tuning panel
open. While the pointer is down the fluid is frozen (paint just lands); on
release water levels out, follows the paper grain, bleeds where the sheet is
damp, drips under tilt, and dries with darkened edges. Right-button drag =
blow, always. Double-click the canvas to cycle among 4 sheets of the current
paper preset. Shortcuts: `p e s n w d b` select tools, `[` `]` resize the
brush, `Ctrl+Z / Ctrl+Shift+Z / Ctrl+Y` undo/redo. Standing water is what
drips: flood an area with several wet passes (or the wet tool), tilt, and
watch it finger into rivulets; the drying rate, rim strength, gravity and
~39 other calibration knobs are live in the tuning panel (PT-BR tooltips
explain each one physically). The experimental panel toggles Kubelka–Munk
subtractive pigment mixing and K–M glaze layer compositing.

## Module map

```
index.html                 layout skeleton (all controls are built by JS)
css/style.css              the dark chrome
js/engine/                 DOM-free: everything imports cleanly in Node
  rng.js                   seeded stream RNG + stateless integer hashes (no Math.random)
  opacity.js               saturating pigment-opacity table alpha(m) = 1 - 0.998^m
  colorops.js              HSV/RGB, sRGB<->linear, Kubelka-Munk mixing + glaze
  grid.js                  padded per-cell field arrays, bbox, canvas-wide actions, snapshots
  paper.js                 procedural 512x512 paper tile (3 presets, target statistics)
  brush.js                 porous bristle texture (fine sieve + core blobs, deposit-integral
                           calibrated per spec §7) + radial falloff + shaped stamp iterator
  solver.js                active-region rebuild, flow build (brake), smoothing, conservative
                           gather advection + gravity, projection, drain boundaries,
                           diffusion/backrun/fingering extensions
  drying.js                evaporate/settle/re-wet pass (edge darkening), fast dry
  sim.js                   40 Hz orchestrator: pass cadences + adaptive drying cadence
  stroke.js                synthetic pressure recurrence + Catmull-Rom dab emission
  trail.js                 two-step paint deposit (tip pickup/self-clean, soft caps,
                           paint drag) + blend's saturating mask window
  tools.js                 erase / wet / dry / blow / smear (per-dab)
  painter.js               facade: dab parameters, tool dispatch, layers, undo/redo,
                           canvas actions, dirty-rect bookkeeping
  render.js                RGBA compositor (granulation, signed emboss, show-wet,
                           layers, K-M glaze, pigment-only export buffer)
  tuning.js                the live knob registry (39 visible + hidden neutral extensions)
js/app/                    the only DOM-aware code
  main.js                  boot, fixed-step loop, pointer wiring, bottom bar, export, shortcuts
  view.js                  canvas blit (dirty-rect), wheel zoom, middle-drag pan
  ui-left.js               color wheel/palette, sliders, shapes, tools, tilt dial, paper, layers
  ui-tuning.js             tuning panel (sliders + free numeric inputs + resets + tooltips),
                           experimental panel, panel resize
  i18n.js                  EN/PT chrome strings
  tooltip.js               shared floating rich-tooltip system (viewport-flipping placement)
  knobdocs.js              PT-BR knob tooltips with subsystem correlation letters
  chromedocs.js            PT-BR rich tooltips for every chrome control (left panel + bottom bar)
test/
  smoke.mjs                spec §18 acceptance tests (1-12) + perf sanity (exit code 0 = green)
  shell-boot-check.mjs     auxiliary DOM-shim boot of the full shell
  texture-calib.mjs        calibration reporter (§7 integral + uniformity, §18.11/12 at both radii)
  texture-sweep.mjs        seed/param search harness used to calibrate the bristle tile
```

## Notes

- Everything is deterministic: identical input scripts reproduce identical
  states (seeded RNGs and integer hashes only).
- The §17 gated extensions (pigment diffusion, backrun blooms, drip fingering,
  physical granulation, staining, dry-brush, render extras) are implemented
  and neutral by default; their knobs live in the tuning registry's `hidden`
  group (`js/engine/tuning.js`) — move an entry to a visible group to expose
  a slider for it. Acceptance test 10 asserts bit-identical fields with the
  extension code paths compiled in vs bypassed.
