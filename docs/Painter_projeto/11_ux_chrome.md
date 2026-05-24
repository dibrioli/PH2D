# 11 — UX chrome, layout, Zen Mode, multi-plataforma

> O chrome do Painter integra-se ao layout 4-zonas existente do PH2D ([`ph2d-editor-core`](../../crates/ph2d-editor-core/), ADR-0023). Mas o "sabor" Procreate exige adaptações: quando Painter ativo, ele assume **takeover do canvas** com sua própria chrome minimalista.

## 11.1 Princípios do chrome

1. **Canvas-first**: canvas central ocupa 100% da área visível em zen. Chrome em volta é finíssimo (~48px top bar, ~64px sidebar) e desaparece quando solicitado.
2. **Chrome esmaece durante stroke**: quando user pinta, sidebar e top bar fade para 40% opacity (subtle); cursor outline e HUD mantêm-se visíveis. Implementação: time-based animation `chrome_alpha = 1.0 - (1.0 - 0.4) * smooth_step(time_since_pen_down, 0, 200ms)`.
3. **"No menu walls"**: hierarquia ≤ 2 níveis. Quase nenhum dialog modal. Mudanças aplicam imediatamente (Live preview onde possível).
4. **Gestos > botões**: undo (2-finger), pan (2-finger drag), zoom (pinch) — sem botão correspondente na UI. Atalho de teclado é o equivalente desktop.
5. **Settings escondidos atrás de Actions** (wrench icon, top-left). Frequente fica gestural / sidebar / popover rápido. Profundidade existe (Brush Studio, Gesture Controls, Preferences) mas nunca empurrada.
6. **HUD discreto durante stroke**: sliders mostram número absoluto flutuante perto do slider quando ajustados. Some 1s após release.

## 11.2 Integração com o layout 4-zonas (ADR-0023)

Layout existente do PH2D editor:

```
┌──────────────────────────────────────────────┐
│ TopLeft EDIT (zone 1)  │  TopRight CREATE   │   ← pequenas zonas de chrome
├──────────────────────────────────────────────┤
│                                              │
│                                              │
│              CENTER (canvas, zone 4)         │
│                                              │
│                                              │
├──────────────────────────────────────────────┤
│        Sidebar modulators (zone 3)           │
└──────────────────────────────────────────────┘
```

Quando Painter ativo (usuária clicou "Painter" pill em Image Tools):

```
┌──────────────────────────────────────────────────────────────┐
│ [Gallery] [Wrench] [Wand] [S] [Arrow]      [Brush][Smudge]   │
│                                       [Eraser][Layers][●]    │  ← Painter top bar (full-width)
├──────────────────────────────────────────────────────────────┤
│┌─┐                                                            │
││▲│                                                            │
││ │                                                            │
││ │            CANVAS (Painter takeover)                       │
││ │                                                            │
││▼│                                                            │
│├─┤                                                            │
││▢│  ← modifier square                                         │
│├─┤                                                            │
││▲│                                                            │
││ │  ← opacity slider                                          │
││ │                                                            │
││▼│                                                            │
│├─┤                                                            │
││↺│ ↻│  ← undo/redo                                            │
│└─┘                                                            │
└──────────────────────────────────────────────────────────────┘
```

**Decisão estrutural (2026-05-23, fechada em [README §11](README.md) #3):** Painter substitui completamente a chrome PH2D do canvas (TopBar Edit/Create + Sidebar modulators) **enquanto ativo**. Sair do takeover: trocar tool ou clicar Gallery (top bar #1).

**Razão da decisão:**
- O sabor Procreate exige sidebar + top bar Procreate-style; tentar coexistir com chrome PH2D normal dilui o sabor.
- Painter é workhorse de horas (vs BgRemoval/Padding que são one-shot/lean — esses justificam panel docado).
- Trade-off aceito: inconsistência com BgRemoval/Padding (panel-docado-style). Painter é nível de ferramenta diferente; chrome diferente é apropriado.

**Mecânica do takeover:**
- `ActivateTool("painter")` no `EditorAction` registra `painter_active = true` no `HeroScreen`.
- Chrome `dispatch_all` (vide ADR-0040 chrome handlers em [`crates/ph2d-editor-core/src/screens/hero/chrome/`](../../crates/ph2d-editor-core/src/screens/hero/chrome/)) lê esse estado e **suprime** renderização das pills/sidebar fora do escopo Painter.
- Painter top bar + sidebar próprios renderizam por cima.
- Reverter no `CancelActiveTool` ou `ActivateTool(other)` — `painter_active = false`, chrome PH2D normal retoma.
- **Esc com stroke em vôo** → confirm popup (*"Descartar stroke em curso?"*). UX safety: descartar trabalho via Esc é erro caro.
- **Esc sem stroke em vôo** → fecha popovers (Brush Library, Color Panel, etc.); 2º Esc consecutivo = sai do Painter para chrome normal.

## 11.3 Top bar layout (Painter active)

10 pills, ordenados esquerda → direita conforme [README.md](README.md) §7 e [06 §6.5](06_selection_transform_adjustments.md):

**Editing Tools (left side, 5):**
1. **Gallery** ⌂ — volta para gallery
2. **Actions** ⚙ — wrench, abre menu
3. **Adjustments** ✨ — wand
4. **Selection** ▢ — S
5. **Transform** ⤧ — arrow

**Painting Tools (right side, 5):**
6. **Brushes** 🖌 — paint brush icon
7. **Smudge** 👆 — finger
8. **Eraser** ⌫ — eraser
9. **Layers** ☰ — square stack
10. **Color** ● — filled circle, **active color thumb**

Cada pill é um button widget consumindo tokens. Highlight do "active" tool com border accent.

Implementação concreta:
- Painter PanelEvent: `tool_select` enum dispatched.
- Pills via `paint_pill` widget (existe em ph2d-editor-core).
- A11y: cada pill `Role::Button` + `label` via i18n.

### 11.3.1 Color thumb (slot #10)

Sempre **visível**, sempre **drag-source** para ColorDrop. Tap → abre Color Panel popover (5 modos §03).

Long-press → "Set as secondary" menu.

## 11.4 Sidebar Procreate-style (left default)

Layout descrito em [05_gestos_input.md](05_gestos_input.md) §5.4.

### 11.4.1 Posicionamento

- **Default**: lateral esquerda do canvas, vertical.
- **Flip via Painter Preferences**: lateral direita (canhotos).
- **Drag vertical**: usuária pode reposicionar o sidebar inteiro arrastando fora dos sliders (move up/down dentro da margem do canvas).
- **Width**: 48px em iPad / desktop; 56px em compact android (touch targets maiores).

### 11.4.2 Elementos (de cima para baixo)

1. **Size slider** (vertical bar com handle). Top half.
2. **Modifier square** ▢ — programmable.
3. **Opacity slider** (vertical bar). Bottom half.
4. **Undo** ↺ + **Redo** ↻ buttons (size-shared, dois em row no fim do sidebar).

### 11.4.3 Visual feedback

Durante drag no size slider: número em px aparece flutuante à direita do slider, com fundo arredondado neutral.

```
   ┌─┐
   │▲│
   │║│   ┌────┐
   │●│ ──│ 64 │   ← floating HUD; appears during drag, fades 1s after release
   │║│   │ px │
   │║│   └────┘
   │▼│
   └─┘
```

## 11.5 HUD durante stroke

Esmaecimento + indicadores visuais:

1. **Sidebar + top bar**: alpha → 40% smooth (200ms).
2. **Cursor outline**: visible (shape + grain preview).
3. **Active color thumb badge**: pequeno no canto inferior-direito do canvas (fixed, sempre visible).
4. **Size indicator** (opcional): se size estiver fora do default, mostra "N px" próximo ao cursor por 500ms quando começa novo stroke.

Após release, fade back to full opacity em 400ms (chrome "respira" visualmente conforme intensidade da pintura).

## 11.6 Zen Mode (Full Screen)

Acesso:
- 4-finger tap (touch)
- `Tab` (keyboard)
- Actions menu → Zen Mode

Quando ativo:
- Toda chrome desaparece (top bar, sidebar, modal panels).
- Canvas ocupa 100% da área.
- Pintura continua funcionando.
- Multi-touch gestures permanecem ativos (incluindo 4-finger tap para sair).
- Atalhos de teclado funcionam (`Ctrl+Z` etc).

Sair: 4-finger tap ou `Tab`. Fade in da chrome.

Status persistido per-session (não persiste por canvas — toggle é UI-level).

## 11.7 Brush Library (popover overlay)

Acesso: tap em Brushes pill (#6) ou atalho `B+B` (double-tap B) ou pinch-in no canvas com Paint tool ativo (Procreate gesture).

Layout overlay:

```
┌────────────────────────────────────────────────────┐
│ Search [______________]              [ ✕ Close ]   │
├──────────────┬─────────────────────────────────────┤
│ Categories   │ Brushes                             │
├──────────────┤                                     │
│ Recents (8)  │ ┌────────────────────────────────┐ │
│              │ │ Pencil 2B                      │ │
│ Pencils      │ ├────────────────────────────────┤ │
│ Inks         │ │ Pencil Charcoal                │ │
│ Markers      │ ├────────────────────────────────┤ │
│ Paints       │ │ ...                            │ │
│ Watercolors  │ │                                │ │
│ Airbrushes   │ │                                │ │
│              │ │                                │ │
│ Imported     │ │                                │ │
│ Custom       │ │                                │ │
└──────────────┴─────────────────────────────────────┘
```

- Pull-down no search bar = filter.
- Long-press num brush thumb = open Brush Studio.
- Drag brush = reorder.
- Categorias collapsable.

## 11.8 Layer panel (popover overlay)

Acesso: tap em Layers pill (#9) ou atalho `F7` (Photoshop muscle memory).

Layout vertical scroll, layers no thumb stack:

```
┌──────────────────────────────────┐
│ Layers (12/75)             [+] [⋯] │  ← count + add + menu
├──────────────────────────────────┤
│ [thumb] Layer 1            ✓     │  ← visibility checkbox
│ [thumb] Layer 2     Opacity 80%  │
│ [thumb]   ↳ Mask                 │  ← nested
│ [thumb] Group A         ▼ (3)    │  ← collapsable
│ ...                              │
└──────────────────────────────────┘
```

- Tap thumb = select primary
- Swipe right = secondary multi-select
- Long-press = layer properties popover (vide [02 §2.3](02_layers.md))

## 11.9 Color panel (popover overlay)

Acesso: tap em Color thumb (#10) ou atalho `C`.

Layout: vide [03_color.md](03_color.md). 5 abas no top.

```
┌────────────────────────────────┐
│ Disc │ Classic │ Harmony │ ... │
├────────────────────────────────┤
│                                │
│        [Color picker]          │
│                                │
├────────────────────────────────┤
│ Color history (10 most recent) │
│  ● ● ● ● ● ● ● ● ● ●           │
├────────────────────────────────┤
│ Default palette (if set):      │
│  ▣ ▣ ▣ ▣ ▣ ▣ ▣ ▣ ▣ ▣ ▣ ▣        │
└────────────────────────────────┘
```

## 11.10 Adjustments panel (popover overlay)

Acesso: tap em Adjustments pill (#3).

Layout: lista vertical de adjustments + toggle Layer/Pencil. Selecting one applies tool mode + entra em modo adjustment.

```
┌──────────────────────────────────┐
│ Adjustments                      │
├──────────────────────────────────┤
│  Hue, Saturation, Brightness  >  │
│  Color Balance                >  │
│  Curves                       >  │
│  Gradient Map                 >  │
│  Gaussian Blur                >  │
│  Motion Blur                  >  │
│  Noise                        >  │
│  Sharpen                      >  │
│  Bloom                        >  │
│  Liquify                      >  │
│  Clone                        >  │
│  Recolor                      >  │
└──────────────────────────────────┘
```

## 11.11 Actions menu (popover overlay)

Acesso: tap em Actions pill (#2) ou atalho `M`-then-`A` (chord). 6 sub-menus:

```
┌──────────────────────────────────┐
│ Add                              │  ← Files, Photo, Camera, Text
│ Canvas                           │  ← Crop, Flip, Rotate, AnimAssist, Reference, Guides
│ Share                            │  ← Export menu
│ Video                            │  ← Time-lapse export, recording control
│ Preferences                      │  ← Open Painter Preferences
│ Help                             │  ← Docs, Tutorials, About
└──────────────────────────────────┘
```

## 11.12 Painter Preferences

Acesso: Actions → Preferences (popover full-screen overlay, Esc to close).

Tabs verticais:

```
┌──────────┬──────────────────────────────────────┐
│ General  │ Painter Preferences > Apple Pencil   │
│ Apple    │                                      │
│  Pencil  │ Pressure curve:                      │
│ Gestures │   [XY graph 8 control points]        │
│ Keyboard │                                      │
│ Color    │ Tilt curve:                          │
│ Brushes  │   [XY graph]                         │
│ Advanced │                                      │
│          │ Double-tap action:                   │
│          │   ● Switch tool/previous             │
│          │   ◯ ...                              │
│          │                                      │
│          │ Squeeze (Pencil Pro):                │
│          │   ...                                │
└──────────┴──────────────────────────────────────┘
```

Cada tab tem seções configuráveis. Mudanças persistidas globalmente (não per-canvas — preferences são per-installation).

## 11.13 Toast feedback

Para feedback discreto: usar [`ToastQueue`](../../crates/ph2d-editor-core/src/toast.rs) existente do editor (ADR-0023). Painter dispara toasts em:

- Save complete: *"Saved as my-canvas.ph2d-painter"*
- Export complete: *"Exported PNG to ~/Desktop/canvas.png"*
- Tool changed: *"Brush · Pencil 2B"* (low-key, 1.5s duration)
- Warning: *"Layer limit reached. Try Flatten Visible."* (accent color)
- Thermal throttle: *"Performance reduced (thermal)"*

Strings via i18n (HR-15).

## 11.14 Multi-plataforma — adaptações

### 11.14.1 Desktop

- **Menu bar nativa** (macOS / Windows / GNOME): integra File/Edit/View/Window/Help. Painter actions expostos.
- **Window controls**: title bar nativa com canvas name + dirty indicator (`* My Canvas`).
- **Tablet config dialog**: detecta Wacom/Huion/XP-Pen e expõe configurações ExpressKeys + Touch Ring.
- **Keyboard primary**: layout otimizado para keyboard + tablet pen (not just touch).
- **Tooltips**: hover de mouse mostra tooltip em pills/buttons (after 500ms hover).

### 11.14.2 iPad / iOS

- **Sem menu bar** (iPadOS native style); Actions wrench cobre tudo.
- **Status bar** integrada (battery, time, notifications visible se não em Zen).
- **Slide-over / Split-view** suportado (canvas redimensiona).
- **Stage Manager** (iPadOS 16+) suportado — Painter como window.
- **Pencil-first**: keyboard otimizações secundárias.

### 11.14.3 Android

- **Sem menu bar** (Android material style); Actions wrench cobre tudo.
- **System back gesture**: Esc-equivalent (sai de popovers; em canvas, abre confirm dialog para Gallery).
- **S Pen primary**: barrel button mapeado configurable.
- **Status bar adaptive** (esconde em Zen).

### 11.14.4 Web

- **Browser chrome** consome top da viewport; Painter top bar abaixo.
- **No menu bar nativa**.
- **Keyboard limited** (browser captura alguns atalhos como `Ctrl+W`, `Ctrl+T`).
- **Pointer Events** for stylus support.

## 11.15 Acessibilidade (HR-12)

Toda chrome do Painter emite `accesskit::Node`:

- Top bar pills: `Role::Button` + label i18n.
- Sidebar sliders: `Role::Slider` + value text + min/max + orientation Vertical.
- Modifier square: `Role::Button` + descriptive label da ação configurada.
- Layer panel: `Role::Tree` com `Role::TreeItem` por layer.
- Color disc: custom `Role::Slider2D` (não-standard ARIA; custom annotation).
- Brush library: `Role::ListBox` com `Role::ListBoxItem` por brush.
- Brush Studio sliders: `Role::Slider` cada.

**Live regions** para feedback:
- Stroke commit: silent (não polui screen reader durante painting).
- Layer changes: announced ("Layer 3 selected").
- Tool changes: announced ("Brush tool active").
- Color changes: announced quando intentional (palette tap; eyedropper sample tem opt-out via prefs).

**Reduced motion** respected: chrome fade animations disabled; HUD mostra/oculta abruptly.

**Color contrast**: tokens já WCAG AA (vide ph2d-tokens); Painter herda.

## 11.16 Tokens consumidos

Painter chrome usa apenas `ph2d-tokens`. Lista:

| Color | Onde |
|-------|------|
| `ColorToken::Bg0` | Canvas vazio background |
| `ColorToken::Bg1` | Top bar + sidebar bg |
| `ColorToken::Bg2` | Popovers bg |
| `ColorToken::Bg3` | Drawing guides (alpha 30%) |
| `ColorToken::Text1` | Pill labels active state |
| `ColorToken::Text2` | Pill labels inactive |
| `ColorToken::Text3` | Tooltips, hints |
| `ColorToken::Accent` | Active pill highlight, sliders thumbs |
| `ColorToken::AccentMuted` | Subtle borders |
| `ColorToken::Danger` | Warnings, destructive confirmations |
| `ColorToken::Selection` | Marching ants secondary color |

| Spacing | Onde |
|---------|------|
| `Spacing::Xxs` | Internal padding within icons |
| `Spacing::Xs` | Between pills in top bar |
| `Spacing::Sm` | Padding within popovers |
| `Spacing::Md` | Sidebar slider gap |
| `Spacing::Lg` | Popover margins |

| Radius | Onde |
|--------|------|
| `Radius::Xs` | Tag-like badges |
| `Radius::Sm` | Buttons, pills |
| `Radius::Md` | Popovers, modals |

| Typography | Onde |
|------------|------|
| `TypeToken::Body` | Layer names, pill labels |
| `TypeToken::Caption` | HUD numbers, tooltips |
| `TypeToken::Heading` | Popover titles |
| `TypeToken::Mono` | Hex code in Value picker |

Tudo via `ColorToken::Bg1.resolve(theme)`, `Spacing::Md.px()`, etc. Zero hex, zero `f32` literal de UI fora do allowlist (HR-15 + gates `no_literal_color` + `no_magic_numeric`).

## 11.17 Gates de teste

| Gate | Crate | Valida |
|------|-------|--------|
| `painter_chrome_no_literal_color` | `ph2d-panel-painter` + `ph2d-tool-painter` | Zero hex em `paint_*` calls (HR-15 enforced via `no_literal_color`) |
| `painter_chrome_a11y_nodes` | idem | Cada top-bar pill, slider, layer thumb emite `Node` (HR-12) |
| `painter_chrome_widget_loc_cap` | idem | Widgets ≤ 500 LOC each |
| `painter_zen_mode_hides_chrome` | idem | 4-finger tap simula → chrome alpha → 0 |
| `painter_hud_during_stroke_fades` | idem | Stroke active → chrome alpha → 0.4 dentro de 250ms |
| `painter_sidebar_flip_lr` | idem | Toggle "flip sidebar" → sidebar renderiza à direita |
| `painter_toast_queue_emit` | idem | Save event dispatched → toast aparece com label i18n correta |
| `painter_takeover_vs_pannel_mode` | idem | Modo takeover (canvas full) vs modo panel (chrome PH2D mantido) — toggle funcional |

**Continua em:** [12_fora_de_escopo.md](12_fora_de_escopo.md) — não-objetivos explícitos.
