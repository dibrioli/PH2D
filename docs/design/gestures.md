# PH2D · Gestures

Touch and pencil mapping for the iPad client. Mouse + keyboard parity is in `interactions.md`. All gestures are remappable from **Preferences → Gestures**, except the iOS-canon ones marked **(fixed)**.

---

## Single-input

| Gesture | Action | Notes |
|---|---|---|
| 1-finger tap | Select entity under finger; deselect if hits canvas | Tap-through on locked layers honored. |
| 1-finger drag (canvas empty) | Pan canvas | Inertial release. |
| 1-finger drag (on entity) | Move entity along Translate gizmo plane | Snap when grid is on. |
| 1-finger long-press (canvas) | Eyedropper — sample tile / sprite under finger | Persists while held; releases sample to swatch. |
| 1-finger long-press (chrome) | Tooltip / context menu | Tooltip if button is informative; menu if actionable. |
| Pencil tap | Same as 1-finger tap | Pencil never pans canvas. |
| Pencil long-press | Open Component editor for entity under tip | Skips Inspector entirely. |
| Pencil double-tap | Cycle Modify button (programmable: Eraser ↔ Select, Pan ↔ Zoom-fit, etc.) | Apple Pencil only. |

---

## Two-finger

| Gesture | Action | Notes |
|---|---|---|
| 2-finger tap | Undo **(fixed)** | iOS canon — never remappable. |
| 2-finger pinch | Zoom + rotate canvas | Rotation snaps to 0/90/180/270° within 5° threshold. |
| 2-finger drag | Pan + zoom + rotate combined | Continuous gesture. |
| 2-finger long-press | Open QuickMenu radial | Default 6 slots: Run, Pause, Step, Reset, Hot-reload, Export. |

---

## Three-finger

| Gesture | Action | Notes |
|---|---|---|
| 3-finger tap | Redo **(fixed)** | iOS canon. |
| 3-finger swipe down | Cut / Copy / Paste menu | On selected entities. |
| 3-finger swipe left | Hide all panels (Hierarchy + Inspector) — keep tool rail | Toggleable. |
| 3-finger swipe right | Restore panels. | |
| 3-finger swipe up | Open Console | |
| 3-finger scrub left/right | Clear current selection / scratch undo | |
| 3-finger pinch | Group selected entities into a parent | |

---

## Four-finger

| Gesture | Action | Notes |
|---|---|---|
| 4-finger tap | Toggle Zen mode **(fixed)** | Hides all chrome. |
| 4-finger swipe up | Show command palette (⌘K equivalent) | |
| 4-finger swipe down | Reveal Asset Library full | |

---

## Edges

| Gesture | Action |
|---|---|
| Edge swipe from left | Open Hierarchy from edge (if hidden). |
| Edge swipe from right | Open Inspector from edge. |
| Edge swipe from bottom | Reveal Console. |
| Edge swipe from top | Show Project menu (left) or Build modal (right). |

---

## ColorDrop & DragDrop

| Gesture | Action |
|---|---|
| Drag asset thumb onto canvas | Instance prefab at drop position. |
| Hold (≥ 350 ms) before release | Show snap candidates: grid · pixel · parent · sibling. |
| Drag asset thumb onto Hierarchy row | Parent the new instance under that row. |
| Drag color swatch onto entity sprite | Apply tint (if Sprite component) or fill (if Solid). |
| Drag color swatch + hold | Show Threshold slider — adjust similarity radius before commit. |

---

## QuickMenu radial (long-press, 2-finger)

- Six slots arranged in a ring 110 px from center.
- Pointer/finger position highlights wedge; release to fire.
- Drag past outer ring (180 px) to cancel.
- Hold ≥ 1.2 s to enter "edit mode" — drag in different actions from a tray.

---

## QuickShape + QuickLine equivalents

| Gesture | Action |
|---|---|
| Place + hold (≥ 350 ms after tap) | Snap-align entity to grid / pixel / parent / sibling. |
| Place + drag without releasing | Live preview with continuously evaluated snap. |
| Drag tile + line motion | Quantize to a straight axis (X or Y depending on dominant axis). |

---

## Customization

Preferences → Gestures lets the user:
- Rebind any non-fixed gesture to any editor command.
- Adjust hold-to-trigger thresholds (250 ms – 800 ms).
- Disable specific gestures globally.
- Toggle inertia for canvas pan and zoom.
- Adjust scroll-and-zoom velocity (touch + trackpad).
- Switch left-handed orientation (mirror sidebar to right edge).

Bindings persist per user profile and sync via iCloud / Drive when enabled.
