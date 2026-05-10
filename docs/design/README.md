# PH2D Design System

Design source-of-truth for **PH2D — Power House Game Engine**, a 2D game editor in Rust + Vello + parley + AccessKit.

> **Audience.** This package is consumed by the Vello implementation agent. Mockups are HTML/CSS so they can be diffed pixel-to-pixel against Vello renders. Tokens are JSON for codegen to Rust. Icons are SVG so paths convert directly. Specs are Markdown so they version cleanly.

---

## Layout

```
/
├── tokens.json                    # 4 themes · typography · radius · spacing · motion · z
├── component-library.html         # every widget × every state × theme switcher
├── icons/                         # 65 SVG glyphs (24 vb · 1.5pt · currentColor)
│   ├── index.html                   ← icon catalog (browse + copy paths)
│   └── *.svg                        ← individual glyphs (kebab-case names)
├── screens/                       # 17 full-viewport mockups (iPad 12.9 landscape)
│   ├── 01-welcome.html
│   ├── 02-editor-main.html          ← hero · default state
│   ├── 03-editor-place-tool.html
│   ├── 04-editor-select-tool.html
│   ├── 05-asset-browser.html
│   ├── 06-hierarchy-panel.html
│   ├── 07-inspector.html
│   ├── 08-color-picker.html
│   ├── 09-component-editor.html
│   ├── 10-script-editor.html
│   ├── 11-console.html
│   ├── 12-quickmenu.html
│   ├── 13-zen-mode.html
│   ├── 14-play-mode.html
│   ├── 15-build-export.html
│   ├── 16-prefs.html
│   └── 17-search-global.html
├── tweaks-panel.jsx               # in-page Tweaks (theme · accent · density · radius · sidebar side)
├── interactions.md                # widget input handling + keyboard shortcuts
├── gestures.md                    # iPad / pencil gesture map (touch-first)
├── animation.md                   # easing tokens · duration tiers · choreography
├── accessibility.md               # WCAG AA contrast · AccessKit roles · focus order · SR labels
└── README.md                      # this file
```

---

## Themes

Four themes ship in `tokens.json`. forge-sdf is the default and is the most thoroughly tuned.

| Key | Mood | Accent | Surface | Layout |
|---|---|---|---|---|
| `forge-sdf` | dark · technical · default | OKLCH(.74 .16 340) — magenta | bg-0 `oklch(.14 .005 280)` | floating |
| `paint-studio` | dark · canvas-first | OKLCH(.78 .14 205) — cyan | bg-0 `oklch(.13 .003 240)` | floating |
| `sunstone` | light · warm | OKLCH(.62 .19 55) — orange | bg-0 `oklch(.97 .005 80)` | floating |
| `blueprint` | light · CAD | OKLCH(.55 .18 250) — blue | bg-0 `oklch(.96 .003 230)` | sidebar |

Use the **Tweaks** toggle on any `screens/` page to switch themes live and adjust accent · density · radius · sidebar side. Tweaks state persists per file via the Edit Mode protocol.

---

## Conventions

### Color
- All color is **OKLCH** at the source. Hex / sRGB only emitted at the codegen layer.
- Accent is reserved for active state and keyboard focus. Not used for decoration.
- Semantic channels: `success` (green), `warn` (amber), `danger` (red), `info` (cool).

### Spacing
- **8 px base** scale: `xxs 2 · xs 4 · sm 6 · md 8 · lg 12 · xl 16 · 2xl 24 · 3xl 32 · 4xl 48`.
- Density tier sets row height: compact 22 · cozy 26 · comfortable 32.
- Section gap is fixed at `14 px` — overrides require explicit justification in code review.

### Radius
- Tier (`sharp · soft · round`): all radii scale together; never one-off.
- Default tier ("soft"): `xs 4 · sm 6 · md 8 · lg 12 · xl 16 · full 9999`.

### Typography
- Sans: **Inter** (variable), with system stack fallback.
- Mono: **JetBrains Mono** with `ui-monospace` fallback.
- Sizes: `xs 11 · sm 13 · base 14 · md 16 · lg 20 · xl 28` · weights 400/500/600/700.
- Mono is used for all **values, IDs, technical metadata** (paths, FPS, hex strings).
- Letter-spacing: positive (`.04em`–`.12em`) for uppercase mono labels; default for sans.

### Iconography
- 24 × 24 viewBox, **1.5 pt stroke**, no fills (except color swatches).
- `currentColor` always — color comes from the surrounding context.
- Lucide-derived (ISC license) — see `icons/index.html` for the full set.

### Naming
- Files: `kebab-case.{html,svg,json}`.
- Tokens: `category.subcategory.variant` in JSON; CSS vars are `--category-name`.
- Components: `PascalCase` widget name as it appears in the library.

---

## How to use this with Vello

1. **Tokens** — `tokens.json` is the codegen input. Generate `ph2d-tokens` Rust crate from it: each theme becomes a module, each token a const.
2. **Icons** — convert SVG paths to Vello `Path` builders. Stroke + width + currentColor map to Vello stroke styles. The icon set is the **complete** chrome glyph palette — do not introduce new icons without adding to `icons/` first.
3. **Mockups** — for each screen in `screens/`, the Vello agent compares its render side-by-side with the HTML mockup at the iPad 12.9 viewport (1366 × 1024). Pixel-precise alignment is the bar; small density variations are acceptable but should match the comp's `row-h` and section-gap.
4. **Specs** — `interactions.md`, `gestures.md`, `animation.md`, `accessibility.md` are the contract for behavior. The HTML mockups are static — they show *what*, not *how*. The specs cover *how*.

---

## Pending / known gaps

- Themes other than `forge-sdf` are tuned for tokens but have not been pixel-reviewed across all 17 screens. Treat their look as "validated palette, mockup pending."
- Aspect ratios for iPad 11" (1194 × 834) and Mac 16:10 (1440 × 900) are not yet shipped. The forge-sdf tokens are responsive; layout reflow rules live in `interactions.md`.
- A first-run tour is described in `interactions.md` but not visualized.
- Localization: no RTL pass yet. Layouts use logical properties (start/end) where they exist; strings are extracted in copy review.

---

## Contributing

When updating this design system:

1. **Add or change a token** → edit `tokens.json`, then run the screen viewer with the Tweaks panel to confirm no regression in any theme.
2. **Add an icon** → drop SVG into `icons/`, register in `icons/index.html` under the right group, and re-export inline into any screen that uses it.
3. **Change a widget** → update `component-library.html` first (catalog source of truth), then propagate to every screen that uses it.
4. **Change a flow or interaction** → update `interactions.md` (and `gestures.md` if touch is involved) before changing a screen — the spec is the authoritative description.
5. **Change motion** → update `animation.md` and `tokens.json` motion section together.

For new screens beyond §9, add an entry to the `screens/` table in this README and follow the same naming convention.

---

## License

Tokens, icons, and CSS code in this directory are MIT-licensed for use within the PH2D project. Inter and JetBrains Mono are SIL Open Font License. Lucide icons (the source set this library is derived from) are ISC-licensed.
