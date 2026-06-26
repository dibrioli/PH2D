# PH2D · Interactions

Per-widget specification of input handling, keyboard shortcuts, animations, and edge states. Implementation reference for the Vello + AccessKit build.

---

## Conventions

- **Animation duration** uses the `motion.duration` token tier — see `animation.md`.
- **Easing** is `ease.standard` unless noted otherwise.
- **`⌘`** = Cmd on Mac, **`Ctrl`** elsewhere — written as `⌘/Ctrl` when both apply.

---

## IconButton

| Channel | Behavior |
|---|---|
| Pointer hover | Background → `bg-3`, icon → `text-1`, border tint shifts to accent at 25% alpha. Transition 120 ms. |
| Pointer press | Scale to 0.96, background → accent-soft. Released within target → activate. |
| Pointer leave during press | Cancel — no activation. |
| Touch tap | Activate immediately on `pointerup` if start and end within target. |
| Touch long-press (550 ms) | If button has tooltip, show tooltip near button. If button has secondary action, fire it. |
| Keyboard | Tab to focus → focus ring (accent, 2 px). Space / Enter activates. |
| Disabled | Opacity 0.4, no hover, no pointer events, `aria-disabled=true`. |
| Loading | Replace icon with spinner; disable interaction; preserve label for SR. |

Edge: a button that owns a popover (e.g. theme switcher) shows a tiny chevron-down adornment at 60% alpha.

---

## TextButton

Same as IconButton plus:

- **Primary variant** uses accent fill; pressed shifts to accent-strong. Caret/chevron icons inside text follow text color.
- **Destructive variant** (`danger`) shows an outline confirmation on first click — second click within 2 s commits. Always keyboard-confirmable via Enter twice.
- Icon-then-label layout uses 6 px gap; label-then-icon used only for "next-step" affordance.

---

## Slider (horizontal & vertical)

| Channel | Behavior |
|---|---|
| Pointer drag | 1:1 follow on track; thumb scales 1.0 → 1.15 while dragging. |
| Click track | Jump to clicked position. ⇧-click to jump in stepped increments. |
| Wheel over slider | Adjust by one step. Shift-wheel → 10× step. |
| Touch drag | Same as pointer; thumb grows 1.25× and a value bubble appears 32 pt above the thumb to clear the finger. |
| Keyboard | ← / → (or ↓ / ↑ for vertical) = step. PageUp / PageDown = 10× step. Home/End = min/max. |
| Expression input | Activated by tapping the value cell. Accepts `2*pi/3`, `90deg`, `1.5+0.2`. Errors inline-tooltip; preserve last valid on blur. |
| Two-handle (range) | Dragging anywhere between handles drags the range as a whole. |
| Disabled | Track 50% alpha, thumb hidden. |

Animation: thumb scale 120 ms ease-out; value bubble fade 100 ms.

---

## Toggle / Switch

- Pointer/touch: tap to flip.
- Keyboard: Space toggles; Enter activates the surrounding form (no-op standalone).
- Animation: knob slides 180 ms with overshoot of 4% (spring 0.7 damping). Background color crossfades 160 ms.
- Indeterminate state (third position) shown only when programmatically set by parent semantics (e.g. group of children with mixed state).

---

## Checkbox

- States: Unchecked, Checked, Indeterminate, Disabled, Focused.
- Indeterminate auto-applies to a parent when its children are mixed; clicking it sets all children to checked.
- Animation: Check glyph draws over 140 ms (stroke-dasharray); background fill fades 100 ms.
- Edge: in dense lists, hit-zone extends to the whole row; clicking row label toggles.

---

## RadioGroup (segmented + vertical)

- Arrow keys move selection (with wrap on segmented; clamp on vertical).
- Animation: active pill slides between options 220 ms, spring 0.75 damping.
- Edge: if disabled while focused, focus moves to next enabled option.

---

## ColorSwatch + Color Picker

| Channel | Behavior |
|---|---|
| Tap swatch | Open Color Picker modal anchored to swatch. |
| Long-press swatch | Show "Replace" / "Reset to default" context menu. |
| Drag swatch onto canvas | ColorDrop — flood fill of nearest matching region (Threshold via held modifier). |
| Picker — Disc tab | Outer ring = hue; inner disc = sat/value. Pencil / cursor controls reticle position. |
| Picker — Classic tab | Square + 3 sliders (HSB), with hex input. |
| Picker — Harmony tab | 5 algorithms: Complementary, Split, Analogous, Triadic, Tetradic. |
| Picker — Value tab | Precise sliders + hex + OKLCH inputs. |
| Picker — Palettes tab | Custom collections; tap to apply, long-press to edit. |
| Eyedropper | Tap-and-hold canvas with picker open. |
| Dismiss | Tap outside, Esc, or close button. |

Animation: picker enters from anchor with 200 ms scale 0.95→1.0 + opacity, exit 140 ms.

---

## Dropdown / Select

- Click → menu opens below; if not enough room, opens above.
- Keyboard: Down arrow opens; Type-ahead jumps to matching item; Esc closes.
- Animation: open scale 0.96 → 1.0 + opacity, 160 ms ease-out from trigger; menu items stagger 12 ms.
- Edge: > 12 items adds a sticky search header; > 30 adds virtualization.

## Combobox

- Same as Dropdown plus inline editable input that filters items as user types.
- Submit on Enter creates a new value if "Allow new" is set; otherwise selects best match.

---

## TextInput / TextArea

- Focus draws focus ring at 2 px, accent color.
- Selection follows system look (accent at 30% alpha).
- TextArea: ⌘⏎ submits, ⏎ inserts newline.
- Validation: error appears below in danger color with icon; field gets danger-tinted border.
- Loading (e.g. fetching auto-complete): dim input, show inline spinner at right edge.

---

## NumberInput

- Steppers (▲▼) nudge by `step`; for a **bounded** box (one with a registered range) the result is clamped to `[min, max]`.
- Drag-scrub: hold the pointer over the value and drag — **horizontal = fast, vertical = precise, Shift = super-precise**. The dispatch axis-locks on the dominant direction at the threshold crossing.
- **Register the range (REQUIRED for bounded boxes):** call `WidgetStore::set_number_range(id, min, max, step)` where you paint the box. The scrub then maps the cursor displacement PROPORTIONALLY to `[min, max]` (full range over `DRAG_RANGE_PX_H` horizontal / `DRAG_RANGE_PX_V` vertical) **and clamps** — without it, a small-range box (e.g. Offset `±1`) races past 100 on a few pixels and the stepper jumps by a buffer-inferred step (1.0). Unbounded boxes (e.g. pixel position) register no range and keep the legacy step-based rate. Reference impl: `ph2d-panel-painter-layers::number_field`.
- Wheel scrolls value (only when focused, to avoid hijacking page scroll).
- Empty + blur snaps to last valid (or min).

---

## Vector3Editor

- Three NumberInputs in a row, labelled X / Y / Z (axis-tinted: red / green / blue tabs).
- ⌘ on a single field forces uniform editing across X/Y/Z.
- Tab cycles X→Y→Z; Shift-Tab reverses.

---

## Toast

| Severity | Politeness | Duration | Color |
|---|---|---|---|
| Info | polite | 3 s | text-1 over bg-2 |
| Success | polite | 3 s | success accent |
| Warning | polite | 5 s | warn accent |
| Error | assertive | sticky until dismissed | danger accent |

- Stack from bottom-right (configurable). Max 4 visible — older queue.
- Hover anywhere on toast pauses dismiss timer.
- Animation: slide in 240 ms with overshoot 6%; fade out 180 ms.

---

## Tooltip

- Mouse hover after 500 ms idle on a target.
- Touch: long-press (550 ms) — tooltip appears as floating chip until release.
- Disappears on pointerleave, scroll, or keyboard navigation away.
- Animation: 100 ms fade-in.

---

## ContextMenu

- Right-click (mouse) or two-finger tap (trackpad) or long-press (touch).
- Animation: appear 140 ms ease-out; submenu opens after 200 ms hover or click.
- Keyboard: Apps key opens; arrows navigate; Enter activates; Esc closes.

---

## Modal / Dialog

- Block all background interaction; backdrop fades 200 ms.
- Focus traps inside; first focusable receives focus on open.
- Esc closes (unless declared non-dismissable).
- Animation: scale 0.96 → 1.0 + opacity 220 ms; exit 160 ms.
- Stack: only one modal at a time; new requests replace with a fade transition.

---

## FloatingPanel

- Open animation: scale 0.95 → 1.0 + opacity from anchor, 200 ms ease-out.
- Drag handle (pill at top) — 1 px = 1 px move, edge-snaps within 8 px of viewport edges.
- Pin button toggles "remain on switch" — otherwise auto-closes on tool change.
- Tap outside dismisses (configurable per panel).

---

## Tabs

- Horizontal: click or arrow keys.
- Animation: indicator slides 180 ms spring; content cross-fades 140 ms.
- Overflow: chevron buttons appear; mousewheel scrolls; ⌘1…⌘9 jump to nth tab.

---

## TreeView (Hierarchy)

| Channel | Behavior |
|---|---|
| Click row | Select; ⌘-click toggles add; ⇧-click ranges. |
| Right-arrow | Expand or move into first child. |
| Left-arrow | Collapse or move to parent. |
| Drag | Reorder; drop indicator shows insertion line; drop into = becomes child if hovered ≥ 250 ms. |
| Double-click name | Rename inline. |
| Visibility eye | Toggle render; ⌥-click solos. |
| Lock | Toggle edit-lock. |
| Right-click | Context menu (Rename / Duplicate / Delete / Group / Unparent / Convert to Prefab / …). |

---

## ListItem (asset / layer rows)

- Click selects; double-click opens detail.
- Touch: swipe right reveals quick actions (Lock, Duplicate, Delete); swipe left adds to multi-selection.
- Drag: thumb drag onto canvas = ColorDrop equivalent (instance prefab).

---

## Loading & empty states

- **Loading** — skeleton rows pulse at 0.5 Hz. After 2 s, replace with explicit spinner + text "Loading…".
- **Empty** — illustration placeholder + 1-line title + 1-line action.
- **Error** — danger icon + cause + retry CTA + "Report" link.
- **Offline** — yellow banner above chrome; queued actions persist locally.

---

## Editor flows

### Place entity

1. Select a prefab in Asset Library or press `V` for Place tool.
2. Cursor becomes a ghost preview that snaps to grid (`G` toggles snap, hold `⌥` disables).
3. Click on canvas to instance.
4. Hold last click (≥ 350 ms) to enter "place-and-hold" with snap candidates highlighted.
5. `Esc` exits Place tool.

### Component edit

1. Select an entity → Inspector populates.
2. Click "Add component" → Component Editor modal opens.
3. Choose a category from the left rail; configure on the right.
4. `⌘⏎` = Apply & close · `⌘⇧⏎` = Apply (keep open) · Esc = Revert & close.

### Run / Pause / Step

1. `⌘⏎` toggles play. Tools dim; HUD shifts to runtime stats; canvas frame ring turns red.
2. `Space` (when playing) toggles pause.
3. `.` (period) steps one frame forward; `,` steps back if available.
4. `⌘.` stops and resets to scene origin.

### Hot-reload script

1. Save Luau file (⌘S) in Script Editor or external editor.
2. Engine recompiles in background; banner shows "Hot-reloading…".
3. On success — toast "Reloaded {script}" (polite).
4. On failure — Console banner pops with line:col link; entity ticks suspended.

### Build

1. ⌘B opens Build modal.
2. Choose target platform, optimization tier, asset budget.
3. Estimated size + warnings render live.
4. "Build" runs in background; modal collapses to HUD-pinned progress chip; final notification with Reveal/Run buttons.
