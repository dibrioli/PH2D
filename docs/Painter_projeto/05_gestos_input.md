# 05 — Gestos, QuickMenu, Atalhos de teclado

> A essência da UX Procreate vive aqui. Cada gesto desliga uma sequência inteira de menus convencionais. **Replicar bem é mais importante que adicionar — a tentação de "1 botão a mais por se acaso" é exatamente o que tira o sabor.**

## 5.1 Princípios

1. **Multi-touch gestos cobrem 80% das ações frequentes.** Undo, redo, zoom, rotate, zen, copy/paste, eyedropper, QuickShape — todos sem botão dedicado.
2. **Atalhos de teclado em desktop = primeira classe.** O que iPad faz com 2-finger tap, desktop faz com `Ctrl+Z` simples. Não criar UI button só porque desktop não tem multi-touch.
3. **Gestos preservados quando possível em desktop trackpad.** Multi-finger trackpad gestures (MacBook, Linux libinput) suportados pelo mesmo gesture recognizer.
4. **Customização em Gesture Controls panel.** Defaults excelentes; usuária power user pode reconfigurar quase tudo. Mas a UI padrão **nunca pede** customização.

## 5.2 Canvas gestures (default, não-configuráveis)

Tabela canônica. Implementados em `ph2d-painter-brush::gestures::canvas`.

| Gesto | Ação | Plataformas |
|-------|------|-------------|
| **1-finger drag** | Paint/Smudge/Erase (tool ativa) | Touch + Pencil |
| **1-finger drag dentro da sidebar** | Move sidebar verticalmente | Touch + Pencil |
| **2-finger tap** | Undo | Touch |
| **2-finger tap-and-hold** | Undo rapid (10 steps/s) | Touch |
| **3-finger tap** | Redo | Touch |
| **3-finger tap-and-hold** | Redo rapid (10 steps/s) | Touch |
| **3-finger swipe down** | Open Copy/Paste menu (Cut/Copy/Copy All/Paste/Duplicate) | Touch |
| **3-finger scrub horizontal** | Clear active layer | Touch |
| **Pinch (2-finger)** | Zoom in/out | Touch + Trackpad |
| **Pinch + twist** | Rotate canvas | Touch + Trackpad |
| **Quick pinch (rapid)** | Snap to Fit Screen | Touch + Trackpad |
| **4-finger tap** | Toggle Full Screen (Zen Mode) | Touch + Trackpad |
| **Touch & Hold (~800ms)** | Eyedropper loupe | Touch + Pencil |
| **Draw + Hold at end (~500ms)** | QuickShape | Touch + Pencil |

### 5.2.1 Sensibilidade dos gestos

Configurável em Painter Preferences → Gestures:
- `Tap timing` (max ms entre tap-up e tap-down em multi-tap) — default 250ms.
- `Hold-and-drag delay` (ms para Hold-and-drag iniciar) — default 500ms.
- `Eyedropper delay` (ms para loupe aparecer) — default 800ms; range 0–1500ms.
- `QuickShape hold delay` (ms) — default 500ms.

### 5.2.2 Pencil-aware vs Touch-aware

Quando `pencil_only_mode` é ativo (Gesture Controls → Pencil Only Mode toggle), **touch é desabilitado para paint** mas gestos multi-touch continuam funcionando. Útil quando o usuário descansa a mão e usa apenas Pencil para drawing.

Diferenciação:
- Pencil (com pressão > 0): hit-test = paint/smudge/erase tool.
- Touch (sem pressão de Pencil): hit-test = gesture recognizer.

## 5.3 Layer panel gestures

Listados em [02_layers.md](02_layers.md) §2.4. Resumo:

| Gesto | Ação |
|-------|------|
| Tap | Select layer |
| Swipe right | Secondary select (multi-select) |
| Long press | Properties popover |
| 2-finger tap | Opacity slider inline |
| 2-finger pinch | Adjust opacity (continuous) |
| 2-finger swipe right | Alpha Lock toggle |
| 2-finger hold | Select content (gera selection) |
| Pinch two layers | Merge Down |
| Drag | Reorder |

## 5.4 Sidebar gestures (no slider de size/opacity)

A sidebar Procreate é programável. Layout default:

```
   ┌───┐
   │ ▲ │  ← size slider (vertical, top half)
   │ ║ │
   │ ║ │
   │ ▼ │
   ├───┤
   │ ◯ │  ← Modifier square (programmable button)
   ├───┤
   │ ▲ │  ← opacity slider (vertical, bottom half)
   │ ║ │
   │ ║ │
   │ ▼ │
   ├───┤
   │ ↺ │  ← Undo (quick tap)
   │ ↻ │  ← Redo (quick tap)
   └───┘
```

Gestures dentro da sidebar:
- **Tap-and-drag no slider** = continuous adjustment (size ou opacity).
- **Tap rápido** no slider = nada (deve drag para mudar; previne tap acidental).
- **Tap no Modifier square** = ativa a ação configurada (default: Color → secondary swatch toggle).
- **Long press no Modifier square** = abre menu de configuração rápida.
- **Drag vertical do sidebar inteiro** (segurar fora dos sliders) = move sidebar para reposicionar.
- **Pinch-zoom no sidebar inteiro** (raro, mas suportado) = escala sidebar (default 1×; 0.7×–1.5× range).

### 5.4.1 Modifier Square — ações configuráveis

Default = "sample color" (eyedropper while held + touch). Outras opções em Gesture Controls:

- Eyedropper (sample-while-held)
- Erase mode (toggle while held — release volta pra paint)
- QuickMenu (radial menu)
- Active brush switch (toggle entre 2 brushes pre-pickados)
- Speed Tap (alternativa rápida pra ação custom)
- Color History toggle

## 5.5 QuickMenu (menu radial)

### 5.5.1 Layout

6 slots dispostos em estrela:

```
         [ 1 ]
       /       \
   [ 6 ]       [ 2 ]
     │           │
   [ 5 ]       [ 3 ]
       \       /
         [ 4 ]
```

Cada slot ocupa ~60° do disco; tap em um slot executa a ação configurada. Drag mantém-tocando-no-canvas até soltar no slot desejado → ativa essa ação.

### 5.5.2 Múltiplos menus

Até **4 QuickMenus salváveis** simultaneamente. Switch entre eles via:
- Swipe right/left dentro do menu radial (passa pro próximo/anterior).
- Pinch out → mostra grade dos 4 menus para escolha 1-tap.

Cada menu pode ter um label customizado (e.g., "Inks", "Paint Setup", "Quick Edits").

### 5.5.3 Ações disponíveis para slots

Praticamente toda ação do Painter:
- Brush switch (qualquer brush da library)
- Blend mode switch
- Layer operation (new, duplicate, merge down, clear)
- Adjustment trigger (HSB, Curves, etc.)
- Custom toggle (Drawing Assist, Reference, Animation Assist)
- Tool switch (Brush/Smudge/Erase/Move/Select)
- Quick action (Undo, Redo, Fill Selection, Clear)
- Custom Luau action (HR-10: usuária scripta uma macro Luau, slot executa)

### 5.5.4 Default activation

QuickMenu não está ativo por default — usuária precisa configurar gesto em Gesture Controls. Sugestão default no Setup wizard: **tap-and-hold com Modifier Square** ou **Apple Pencil double-tap**.

## 5.6 Gesture Controls panel

Painel full-config de gestos. Acesso: Actions → Prefs → Gesture Controls.

### 5.6.1 Layout

Lista de **ações** (rows verticais), cada uma com opções de assignment:

```
┌─────────────────────────────────────────────────┐
│ Smudge                                          │
│   ┌─ Touch ─┬─ Pencil ─┬─ Modifier (▢) ─┐       │
│   │   ☐    │    ☑     │      ☐         │       │
│   └────────┴──────────┴────────────────┘       │
│   Tap / Hold / Touch & Hold     [▼ Hold]        │
│   Delay: [200ms ──●──]                          │
│   Multi-touch:  1 / 2 / 3 / 4   [● 1 ]          │
├─────────────────────────────────────────────────┤
│ Erase                                           │
│ (similar layout)                                │
├─────────────────────────────────────────────────┤
│ ColorDrop                                       │
│ Eyedropper                                      │
│ QuickMenu                                       │
│ QuickShape                                      │
│ QuickLine (subset of QuickShape)                │
│ Speed Tap                                       │
│ Full Screen                                     │
│ Clear Layer                                     │
├─────────────────────────────────────────────────┤
│ Apple Pencil 2 double-tap:                      │
│   ● Switch tool/previous tool                    │
│   ◯ Switch tool/eraser                          │
│   ◯ Show color palette                          │
│   ◯ Off                                         │
├─────────────────────────────────────────────────┤
│ Apple Pencil Pro squeeze:                       │
│   (similar)                                     │
├─────────────────────────────────────────────────┤
│ Apple Pencil Pro barrel roll:                   │
│   (per-brush; see Brush Studio)                 │
└─────────────────────────────────────────────────┘
```

### 5.6.2 Conflict validation

Não é possível atribuir o mesmo gesto a duas ações. Tentativa de criar conflict mostra warning *"Touch+Hold já está atribuído a Eyedropper. Desativar lá primeiro?"* com botão one-click pra desativar.

### 5.6.3 Reset to defaults

Botão "Reset to factory defaults" no topo do panel.

### 5.6.4 Per-device profile

Em desktop com tablet pen, gesture configs incluem opções para:
- **Wacom ExpressKeys** (até 8 botões customizáveis no tablet).
- **Wacom Touch Ring** (rotational input).

Detectados dinamicamente via `gilrs` + custom HID — appearance é greyed em devices sem.

## 5.7 Atalhos de teclado (desktop)

Lista canônica. Configurável em Painter Preferences → Keyboard Shortcuts.

### 5.7.1 Tools

| Tecla | Tool |
|-------|------|
| `B` | Brush |
| `S` | Smudge |
| `E` | Eraser |
| `G` | Move (drag canvas content) |
| `V` | Move tool (alias for G, Photoshop muscle memory) |
| `M` | Selection (Marquee — Rectangle default) |
| `W` | Magic Wand (Automatic Selection) |
| `L` | Lasso (Freehand Selection) |
| `T` | Transform |
| `I` | Eyedropper |
| `H` | Pan (alt to space-drag) |
| `Z` | Zoom (alt to scroll wheel) |

Modal switching: tap tecla = switch tool. Hold tecla = temporary switch (release = restore previous tool — Blender muscle memory).

### 5.7.2 Brush parameters

| Tecla | Ação |
|-------|------|
| `[` | Decrease brush size (10%) |
| `]` | Increase brush size (10%) |
| `Shift+[` | Decrease size (1%) |
| `Shift+]` | Increase size (1%) |
| `{` | Decrease opacity (10%) |
| `}` | Increase opacity (10%) |
| `0`...`9` | Set opacity (0=0%, 1=10%, ..., 9=90%, 00=100% double-press) |

### 5.7.3 Canvas

| Tecla | Ação |
|-------|------|
| `Ctrl+Z` (`Cmd+Z` mac) | Undo |
| `Ctrl+Shift+Z` / `Ctrl+Y` | Redo |
| `Ctrl+S` | Save |
| `Ctrl+E` | Merge Down |
| `Ctrl+Shift+E` | Flatten Visible |
| `Ctrl+J` | Duplicate layer |
| `Ctrl+N` | New canvas |
| `Ctrl+O` | Open |
| `Ctrl+W` | Close canvas |
| `Ctrl++` / `Ctrl+=` | Zoom in |
| `Ctrl+-` | Zoom out |
| `Ctrl+0` | Fit to screen |
| `Ctrl+1` | 100% zoom |
| `Ctrl+Alt+0` | Full screen (Zen) |
| `Space+drag` | Pan |
| `Space+drag` + `Alt` | Rotate |
| `R` | Reference Companion toggle |
| `Tab` | Zen Mode toggle |
| `Shift+G` | Drawing Assist toggle |
| `Ctrl+'` | Drawing Guide visibility toggle |

### 5.7.4 Color

| Tecla | Ação |
|-------|------|
| `X` | Swap primary ↔ secondary color |
| `D` | Reset to default (primary=black, secondary=white) |
| `C` | Open color picker |
| `Alt+drag` (no canvas while brushing) | Eyedropper transient |

### 5.7.5 Layers

| Tecla | Ação |
|-------|------|
| `Ctrl+Shift+N` | New layer |
| `Ctrl+G` | Group selected |
| `Ctrl+Shift+G` | Ungroup |
| `Ctrl+/` | Lock/unlock |
| `Ctrl+,` | Alpha lock toggle |
| `Ctrl+Alt+G` | Clipping mask toggle |
| `Alt+Click no thumb` | Select content (selection from layer) |

### 5.7.6 Customização

Painter Preferences → Keyboard Shortcuts:
- Lista todas as ações + atalhos atuais.
- Click em uma row → "Press new shortcut..." → registra.
- Conflict detection: warning se a tecla já está em uso.
- Import/export keymap como `.ph2d-keymap` (postcard).
- 3 keymap presets: **PH2D default** (acima), **Photoshop-like** (B for brush, Ctrl+Backspace fill, etc), **Krita-like**.

## 5.8 Apple Pencil specifics

### 5.8.1 Double-tap (Apple Pencil 2 / Pro)

Configurável em Painter Preferences → Apple Pencil (NÃO em Gesture Controls — Apple-specific):

- **Switch between Current Tool and Previous Tool** (default)
- **Switch between Current Tool and Eraser**
- **Show Color Palette**
- **Off**

### 5.8.2 Squeeze (Apple Pencil Pro)

Configurável similar:
- **Open QuickMenu** (default — útil pra reproduzir Procreate flow)
- **Switch tool/previous**
- **Eyedropper hold**
- **Show Color Palette**
- **Off**

Squeeze tem haptic feedback automático (Apple-controlled; respeitado).

### 5.8.3 Barrel Roll (Apple Pencil Pro)

**Per-brush** (não global). Configurado em Brush Studio → Apple Pencil section (§01 §1.3.10). Modula size/opacity/bleed conforme a rotação física do barrel.

## 5.9 Multi-platform behavior

| Gesto / Atalho | Desktop | iPad/iOS | Android | Web |
|----------------|---------|----------|---------|-----|
| 2-finger tap (undo) | Trackpad | Touch | Touch | Touch + Pointer |
| 3-finger swipe | Trackpad | Touch | Touch | Limited (browser pode capturar) |
| 4-finger tap (zen) | Trackpad (rare; Tab key alt) | Touch | Touch | Limited |
| Pinch zoom | Trackpad / Scroll Ctrl+Wheel | Touch | Touch | Pointer Events |
| Pencil double-tap | N/A (alt: hardware Bamboo Pen?) | Pencil 2/Pro | S Pen button | N/A |
| Pencil squeeze | N/A | Pencil Pro | N/A | N/A |
| Keyboard `Ctrl+Z` | Yes | iPad hardware keyboard | Android keyboard | Yes |

**Graceful degradation:** features faltantes em uma plataforma ficam **disabled na UI com tooltip explicativo**, não escondidas. Razão: HR-1 + UX consistency.

## 5.10 Gates de teste

| Gate | Crate | Valida |
|------|-------|--------|
| `gestures_2finger_tap_undo` | `ph2d-painter-brush` | 2-finger tap dispara `EditorAction::Undo` no Painter |
| `gestures_4finger_zen_toggle` | idem | 4-finger tap toggles `ZenMode` |
| `gestures_pencil_only_touch_disabled` | idem | Pencil-only mode on + finger touch no canvas → ignored para paint, multi-finger gestos ainda funcionam |
| `quickmenu_radial_hit_test` | idem | 60° wedges; tap em cada produz slot correto |
| `gestures_conflict_validation` | idem | Tentar atribuir Touch+Hold a 2 ações falha |
| `keyboard_shortcut_brush_size` | idem | `[` decreases size 10%; `Shift+[` decreases 1%; clamp em min |
| `keyboard_shortcut_swap_colors` | idem | `X` swap primary/secondary |
| `apple_pencil_double_tap_switch` | idem | (mocked) double-tap → switch tool conforme config |
| `gesture_controls_reset_to_defaults` | idem | Reset volta todas as assignments ao default state |
| `keymap_preset_photoshop_like` | idem | Switch para keymap "Photoshop-like" muda atalhos corretamente |

**Continua em:** [06_selection_transform_adjustments.md](06_selection_transform_adjustments.md) — selection, transform, adjustments.
