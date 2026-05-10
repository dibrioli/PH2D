# PH2D · Accessibility

Target: **WCAG 2.2 Level AA** across all themes. Implementation reference for the Vello + AccessKit build.

---

## Hit targets

| Input modality | Minimum size | Recommended | Spacing between targets |
|---|---|---|---|
| Touch (iPad / mobile) | 44 × 44 pt | 48 pt for primary actions | ≥ 8 pt |
| Pointer (mouse / trackpad) | 24 × 24 pt | 32 pt for icon-only buttons | ≥ 4 pt |
| Pencil / stylus | 32 × 32 pt | follows pointer minimums | ≥ 4 pt |

All sized via the `--touch-min` / `--pointer-min` tokens; density tier swaps the active value (compact/cozy/comfortable).

---

## Color contrast (all themes)

Computed via OKLCH → linear sRGB → WCAG 2.1 relative luminance contrast ratio. Values rounded to one decimal. **AAA ≥ 7 : 1** · **AA ≥ 4.5 : 1** · **AA Large / non-text ≥ 3 : 1**.

### forge-sdf

| Pair | Foreground | Background | Ratio | WCAG |
|---|---|---|---|---|
| Body text on canvas | `text-1` | `bg-0` | 17.7 : 1 | ✅ AAA |
| Body text on panel | `text-1` | `bg-1` | 17.0 : 1 | ✅ AAA |
| Body text on elevated | `text-1` | `bg-2` | 15.8 : 1 | ✅ AAA |
| Secondary text | `text-2` | `bg-1` | 7.7 : 1 | ✅ AAA |
| Tertiary text (large) | `text-3` | `bg-1` | 3.5 : 1 | ✅ AA Large |
| Accent on panel | `accent` | `bg-1` | 7.7 : 1 | ✅ AAA |
| Danger on panel | `danger` | `bg-1` | 5.6 : 1 | ✅ AA |
| Success on panel | `success` | `bg-1` | 8.8 : 1 | ✅ AAA |
| Warn on panel | `warn` | `bg-1` | 9.4 : 1 | ✅ AAA |
| Strong border on panel | `border-strong` | `bg-1` | 2.3 : 1 | ⚠️ Below |

### paint-studio

| Pair | Foreground | Background | Ratio | WCAG |
|---|---|---|---|---|
| Body text on canvas | `text-1` | `bg-0` | 17.9 : 1 | ✅ AAA |
| Body text on panel | `text-1` | `bg-1` | 17.3 : 1 | ✅ AAA |
| Body text on elevated | `text-1` | `bg-2` | 16.1 : 1 | ✅ AAA |
| Secondary text | `text-2` | `bg-1` | 7.8 : 1 | ✅ AAA |
| Tertiary text (large) | `text-3` | `bg-1` | 3.5 : 1 | ✅ AA Large |
| Accent on panel | `accent` | `bg-1` | 10.3 : 1 | ✅ AAA |
| Danger on panel | `danger` | `bg-1` | 5.7 : 1 | ✅ AA |
| Success on panel | `success` | `bg-1` | 8.9 : 1 | ✅ AAA |
| Warn on panel | `warn` | `bg-1` | 9.6 : 1 | ✅ AAA |
| Strong border on panel | `border-strong` | `bg-1` | 2.2 : 1 | ⚠️ Below |

### sunstone

| Pair | Foreground | Background | Ratio | WCAG |
|---|---|---|---|---|
| Body text on canvas | `text-1` | `bg-0` | 17.3 : 1 | ✅ AAA |
| Body text on panel | `text-1` | `bg-1` | 15.8 : 1 | ✅ AAA |
| Body text on elevated | `text-1` | `bg-2` | 14.4 : 1 | ✅ AAA |
| Secondary text | `text-2` | `bg-1` | 7.7 : 1 | ✅ AAA |
| Tertiary text (large) | `text-3` | `bg-1` | 4.1 : 1 | ✅ AA Large |
| Accent on panel | `accent` | `bg-1` | 3.2 : 1 | ✅ AA Large |
| Danger on panel | `danger` | `bg-1` | 4.6 : 1 | ✅ AA |
| Success on panel | `success` | `bg-1` | 3.7 : 1 | ✅ AA Large |
| Warn on panel | `warn` | `bg-1` | 2.8 : 1 | ⚠️ Decorative |
| Strong border on panel | `border-strong` | `bg-1` | 2.1 : 1 | ⚠️ Below |

### blueprint

| Pair | Foreground | Background | Ratio | WCAG |
|---|---|---|---|---|
| Body text on canvas | `text-1` | `bg-0` | 16.7 : 1 | ✅ AAA |
| Body text on panel | `text-1` | `bg-1` | 15.3 : 1 | ✅ AAA |
| Body text on elevated | `text-1` | `bg-2` | 14.0 : 1 | ✅ AAA |
| Secondary text | `text-2` | `bg-1` | 7.5 : 1 | ✅ AAA |
| Tertiary text (large) | `text-3` | `bg-1` | 3.9 : 1 | ✅ AA Large |
| Accent on panel | `accent` | `bg-1` | 3.9 : 1 | ✅ AA Large |
| Danger on panel | `danger` | `bg-1` | 4.4 : 1 | ✅ AA Large |
| Success on panel | `success` | `bg-1` | 3.6 : 1 | ✅ AA Large |
| Warn on panel | `warn` | `bg-1` | 2.7 : 1 | ⚠️ Decorative |
| Strong border on panel | `border-strong` | `bg-1` | 2.2 : 1 | ⚠️ Below |

### Reading the table

- Tertiary text (`text-3`) is intentionally large-only AA — used for hints, watermarks, technical metadata in mono. It is never primary information.
- Light themes (sunstone, blueprint) push `accent` lightness down to ~0.55–0.62 to keep AA on warm/cool whites. Accent on dark themes runs higher (~0.74–0.78).
- Borders are decorative — `border` is below the non-text 3 : 1 threshold by design. Use `border-strong` for any border that must read as structural (separators between regions, focused outlines).
- Focus rings always use `accent` at 2 px against any panel surface — passes non-text AA in every theme.

---|---|---|---|---|---|
| Primary text on bg | text-1 `oklch(.96 .005 280)` | bg-0 `oklch(.14 .005 280)` | 17.4 : 1 | ✅ AAA | Body text, headings |
| Primary on panel | text-1 | bg-1 `oklch(.17 .006 280)` | 14.6 : 1 | ✅ AAA | Panel content |
| Secondary text | text-2 `oklch(.72 .008 280)` | bg-1 | 7.0 : 1 | ✅ AAA | Labels, meta |
| Tertiary text | text-3 `oklch(.52 .01 280)` | bg-1 | 3.6 : 1 | ✅ AA Large only | Hints, watermarks (≥18px or bold ≥14px) |
| Accent on bg | accent `oklch(.74 .16 340)` | bg-1 | 5.4 : 1 | ✅ AA | Active state, links |
| Accent on accent-soft | accent | accent-soft (15% accent) | 4.6 : 1 | ✅ AA | Selected pills, chips |
| Danger text | danger `oklch(.66 .20 25)` | bg-1 | 4.7 : 1 | ✅ AA | Errors, destructive |
| Success text | success `oklch(.74 .14 155)` | bg-1 | 6.5 : 1 | ✅ AAA | Confirmations |
| Border on panel | border `oklch(.30 .012 280)` | bg-1 | 1.4 : 1 | n/a (decorative) | Dividers |
| Border-strong | border-strong `oklch(.42 .015 280)` | bg-1 | 2.6 : 1 | ✅ Non-text 3:1 (focus rings boost to ≥3:1) | Component edges |
| Focus ring | accent | bg-1 | 5.4 : 1 | ✅ Non-text AA | Keyboard focus |

### Other themes (summary)

| Theme | Primary text on bg | Accent on bg | Notes |
|---|---|---|---|
| paint-studio (dark cyan) | 16.8 : 1 | 5.7 : 1 | Background slightly cooler; cyan accent picks up more luminance |
| sunstone (light warm) | 13.9 : 1 | 4.6 : 1 | Orange accent dropped to 0.62 L for AA on warm white |
| blueprint (light cool) | 14.2 : 1 | 5.0 : 1 | Blue accent at 0.55 L; sidebar layout uses higher contrast borders |

All themes pass AA for body text, accent, and all semantic colors against their primary panel surface. Tertiary text (`text-3`) is intentionally large-only AA — used for hints, never primary information.

---

## AccessKit role mapping

| Widget | AccessKit role | State channels | Notes |
|---|---|---|---|
| IconButton | `Button` | `Pressed`, `Disabled`, `Focused` | Always pair with `Description` for tooltip text |
| TextButton | `Button` | same as above | Variant communicated via name suffix only when destructive |
| Toggle / Switch | `Switch` | `Toggled` | Live region announces "On" / "Off" |
| Checkbox | `CheckBox` | `Toggled(Indeterminate)` supported | Tri-state for parent/child trees |
| RadioGroup | `RadioGroup` containing `RadioButton` | `Selected` | Arrow keys move selection |
| Slider | `Slider` | `NumericValue`, `MinNumericValue`, `MaxNumericValue` | Step via arrow / Page keys |
| TextInput | `TextInput` | `Editable`, `Multiline=false` | `Description` for placeholder |
| TextArea | `TextInput` | `Multiline=true` | |
| NumberInput | `SpinButton` | `NumericValue` | Steppers are nested `Button` children |
| Dropdown / Select | `PopupButton` → opens `Menu` of `MenuItem` | `HasPopup=Listbox` | |
| Combobox | `ComboBox` | `Editable`, `HasPopup=Listbox` | |
| Tabs | `TabList` containing `Tab` | `Selected` | Owns a `TabPanel` |
| TreeView | `Tree` containing `TreeItem` | `Expanded`, `Selected`, `Level` | Each row carries position-in-set |
| ListItem (asset/layer) | `ListItem` inside `List` | `Selected` | |
| FloatingPanel | `Window` (sub-window) | `Modal=false` | Title from header label |
| Modal | `Dialog` | `Modal=true` | Focus is trapped on open |
| Toast | `Status` (polite) or `Alert` (assertive) | severity → live politeness | |
| Tooltip | `Tooltip` | `DescribedBy` from owner | Hidden from tab order |
| ContextMenu | `Menu` containing `MenuItem` | | Triggered by Apps key, long-press, right-click |
| HUD pill | `StatusBar` | live updates polite | |
| Inspector tab content | `Group` with labelled sections | | Each row `Group` with name from label cell |
| Hierarchy | `Tree` | as TreeView | |

---

## Keyboard navigation

### Global

| Key | Action |
|---|---|
| Tab / Shift-Tab | Move focus through interactive chrome |
| F6 / Shift-F6 | Cycle focus between major regions (top toolbar → tool rail → inspector → hierarchy → canvas → bottom HUD → console) |
| Esc | Close top-most floating panel; if none, deselect; if nothing selected, blur |
| Enter / Space | Activate focused control |
| ⌘K / Ctrl-K | Open Global Search overlay |
| ⌘P / Ctrl-P | Same — Global Search shortcut alias |
| ⌘, / Ctrl-, | Open Preferences |
| Arrow keys | Move within composite widgets (Slider, RadioGroup, Tree, Menu) |

### Editor

| Key | Action |
|---|---|
| Q / W / E / R | Cycle gizmo: Pan / Translate / Rotate / Scale |
| V | Place tool |
| B | Asset Library toggle |
| H | Hierarchy panel toggle |
| I | Inspector toggle |
| ⌘Z / Ctrl-Z | Undo |
| ⌘⇧Z / Ctrl-Shift-Z | Redo |
| Space (held) | Pan canvas (cursor → grabbing) |
| F | Frame selected on canvas |
| . / , | Step play frame fwd / back |
| ⌘B / Ctrl-B | Open Build modal |
| ⌘⏎ / Ctrl-Enter | Play / Stop toggle |

### Within tree / list

| Key | Action |
|---|---|
| ↑ / ↓ | Move row |
| ← | Collapse group; if leaf, jump to parent |
| → | Expand group; if expanded, first child |
| Home / End | First / last visible |
| Type-ahead | Jump to matching name |
| Shift+↑/↓ | Extend selection |
| ⌘+click / Ctrl+click | Toggle selection |

---

## Focus order (canonical)

1. Top-left cluster — Project menu · Save · Save dot
2. Top-center — Wordmark (skipped, decorative)
3. Top-right cluster — Search · Console toggle · Build · Play controls
4. Tool rail — Pan · Translate · Rotate · Scale · Pivot · Grid · Camera · Layer · Asset
5. Inspector tabs → tab content (linear by row)
6. Hierarchy filter → tree
7. Canvas (focusable region; arrow keys pan)
8. Bottom HUD pill — FPS · Zoom · Coords · Cost meter
9. Floating panels (when open) trap focus until dismissed

---

## Screen reader labels

Canonical strings — match across themes and locales.

```
IconButton/play          → "Play scene"
IconButton/pause         → "Pause scene"
IconButton/step          → "Step one frame forward"
IconButton/stop          → "Stop and reset to scene origin"
IconButton/build         → "Build & export project…"
IconButton/search        → "Open global search (⌘K)"
IconButton/console       → "Toggle debug console"
IconButton/save          → "Save project (⌘S)"
IconButton/save (dirty)  → "Save project — unsaved changes"
Tool/pan                 → "Pan tool — drag to move the camera"
Tool/translate           → "Translate tool — drag entity along axes"
Hierarchy/visible        → "Toggle visibility of {entityName}"
Hierarchy/lock           → "Toggle lock of {entityName}"
Inspector/tab/transform  → "Transform tab — position, rotation, scale"
QuickMenu/slot/{n}       → "{slotLabel} — slot {n}"
HUD/fps                  → "Frames per second: {value}"
```

Live regions:
- **Polite** — `Status` role: HUD updates, save confirmations, hot-reload OK
- **Assertive** — `Alert` role: build failures, runtime exceptions, destructive undo prompts

---

## Reduced motion

If `prefers-reduced-motion: reduce`:

- Replace all spring/elastic motions with cross-fade only (200 ms)
- Drop slide/scale entry animations on panels — they appear in place
- QuickMenu radial expand → instant fade
- Sidebar drag follows pointer 1:1, no inertia
- Toast slide-in becomes opacity-only
- Icon hover micro-bounces are removed
- Loading spinner replaced with stepped 8-frame animation (no continuous rotation)

Persist as `a11y.reducedMotion = true` and apply across the whole editor (no per-screen opt-in).

---

## Dynamic Type

The editor honors the system-level text scale (Mac → Display Settings; iPad → Larger Text in Accessibility). All sizes derive from `--font-size-base` (default 14 px).

| Tier | Multiplier | Effective base |
|---|---|---|
| xSmall | 0.85 | 12 px |
| Small | 0.92 | 13 px |
| Medium (default) | 1.00 | 14 px |
| Large | 1.10 | 15 px |
| xLarge | 1.20 | 17 px |
| xxLarge | 1.35 | 19 px |
| xxxLarge | 1.50 | 21 px |

At ≥ xxLarge, density tier auto-switches to **comfortable** and `row-h` grows from 26 → 32 px. Some chrome (HUD pill, tool rail labels) collapses to icon-only with tooltips to preserve canvas area.

---

## VoiceOver / NVDA / Narrator behavior

- Editor surface is **not** announced as a continuous text region — it is a `Canvas` role with a synthetic description of selected entities ("Selected: 3 entities — Player, Wall, Wall.001")
- Tab traps inside modals and the Build dialog; `Esc` returns focus to the trigger
- Selecting an entity in Hierarchy moves SR cursor to the matching gizmo announcement on canvas
- Notifications via toast queue announce in arrival order; assertive interrupts polite
- Drag-reorder in tree announces "{name} moved to position {n} in {parent}"
