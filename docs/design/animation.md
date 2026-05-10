# PH2D · Motion

Motion is restrained, purposeful, and respects `prefers-reduced-motion` (see `accessibility.md`). All motion lives in tokens — no per-component magic numbers.

---

## Easing tokens

| Token | cubic-bezier | Use |
|---|---|---|
| `ease.linear` | `linear` | Progress bars, scrubber, anything time-coupled. |
| `ease.standard` | `cubic-bezier(.2, 0, 0, 1)` | Default for everything that doesn't need character. |
| `ease.accelerate` | `cubic-bezier(.3, 0, 1, 1)` | Things leaving the screen. |
| `ease.decelerate` | `cubic-bezier(0, 0, .2, 1)` | Things arriving on screen. |
| `ease.emphasized` | `cubic-bezier(.2, 0, 0, 1.1)` | Slight overshoot for high-attention motion (toast, modal). |
| `ease.spring-soft` | spring(stiffness 320, damping 28, mass 1) | Panels, drag-snaps. |
| `ease.spring-snappy` | spring(stiffness 480, damping 22) | Toggle thumbs, segmented selectors. |

---

## Duration tiers

| Token | ms | Use |
|---|---|---|
| `motion.instant` | 0 | Reduced-motion fallback baseline. |
| `motion.flash` | 80 | Press-down, ripple suppression, focus ring appear. |
| `motion.fast` | 140 | Hover transitions, tooltip fade, simple state shifts. |
| `motion.default` | 220 | Modal in, panel slide, tab indicator. |
| `motion.slow` | 360 | Hierarchy reorder, large layout shifts. |
| `motion.deliberate` | 520 | First-run tour, intentional reveal animations. |

Implementation guidance: never use a duration outside this scale. Composing animations from a small fixed set produces a recognizable rhythm across the editor.

---

## Stagger patterns

Use stagger to convey **relationship**, never decoration.

| Pattern | Delay | Use |
|---|---|---|
| List cascade | 16 ms × index, capped at 8 items (rest reveal together) | Asset grid, search results, menu open. |
| Symmetric burst | 24 ms × |n − center| | QuickMenu radial reveal. |
| Path follow | 40 ms × hop along path | Hot-reload pulse along entity tree. |

Stagger is suppressed under reduced motion.

---

## Spring physics — when to reach for them

- Panel snap-to-edge (after drag release)
- Toggle / Switch knob slide
- Segmented selector pill movement
- Hierarchy drop confirmation (parent flashes + settles)
- Modal in/out — only with `spring-soft` and `emphasized` ease

Avoid springs on: focus rings, tooltip, validation flashes, error toasts (assertive context expects predictable timing).

---

## Animation choreography (canonical)

### Entry

| Element | Choreography |
|---|---|
| Modal | backdrop fade 200 ms · dialog scale 0.96→1 + opacity 220 ms (`ease.emphasized`) |
| Floating panel | scale 0.95→1 + opacity 200 ms from anchor (`ease.decelerate`) |
| Toast | translateY 12 → 0 + opacity 240 ms (`ease.spring-soft`) |
| Dropdown | scale 0.96→1 + opacity 160 ms · items stagger 12 ms (`ease.decelerate`) |
| QuickMenu | radial: each slot rotateY 30→0 + opacity, 24 ms × |i−center| (symmetric burst, `ease.spring-snappy`) |
| Tooltip | opacity 100 ms (`ease.fast` linear) |

### Exit

All exits are 70% of their entry duration, `ease.accelerate`, no stagger.

### State-shift

| Element | Choreography |
|---|---|
| Hover (button, row) | bg + border crossfade 120 ms |
| Press | scale 1.0→0.96 80 ms, then back on release 120 ms |
| Toggle flip | knob slide 180 ms `spring-snappy`, bg crossfade 160 ms |
| Tab change | indicator slide 180 ms `spring-soft`, content crossfade 140 ms |
| Tree expand | height ease 220 ms · child stagger 16 ms cap 8 |
| Slider drag | thumb scale 1→1.15 120 ms; bubble fade 100 ms |

---

## Loading

- Skeleton: opacity oscillates 0.5 ↔ 1 over 1.6 s (sine).
- Spinner: 1.0 s rotation (linear).
- Indeterminate progress bar: 1.4 s ping-pong (cubic ease in-out).
- Determinate progress bar: width transitions per delta with `ease.linear` 200 ms.

Reduced motion fallback for all three: pulse opacity 0.6 ↔ 1 every 800 ms (no movement).

---

## Performance budget

- All transitions ≤ 60 fps on iPad mini (M1) and base Mac.
- No simultaneous spring on > 8 elements (use stagger cap).
- Avoid animating layout-affecting properties on > 50 elements at once — use transform + opacity exclusively for cascades.
- Hot-reload pulse animation is GPU only (transform + filter, no width/height).

---

## Reduced motion

When `prefers-reduced-motion: reduce`:

- Replace all spring/elastic with `ease.linear`.
- Cap any motion at `motion.fast` (140 ms).
- Drop slide / scale entry — keep opacity only.
- Remove all stagger; reveal as a group.
- Hot-reload pulse and other "delight" motion is suppressed entirely.
- Spinners replaced with stepped 8-tick frame at 800 ms.
