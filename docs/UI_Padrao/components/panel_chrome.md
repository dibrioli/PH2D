# Panel chrome — título, drag, resize

**Status:** ativo desde 2026-05-24.
**Aplica a:** todo painel flutuante (`ph2d-panel-*` + showcase). Sem exceção.
**Fontes vivas:** [Widget Gallery](../../../crates/ph2d-panel-widget-gallery/), [Inspector](../../../crates/ph2d-panel-inspector/), [Hierarchy](../../../crates/ph2d-panel-hierarchy/) — todas devem se comportar igual ao descrito aqui.
**Implementação canônica:** [`crates/ph2d-editor-core/src/widget/panel_chrome.rs`](../../../crates/ph2d-editor-core/src/widget/panel_chrome.rs).
**Gates:** [`architecture_panel_loc_cap`](../../../crates/ph2d-editor-core/tests/architecture_panel_loc_cap.rs).

---

## Anatomia

```
┌─────────────────────────────────────────┐ ← painel.y (rounded BgElev, 16 px radius)
│  TÍTULO                          [ X ]  │
│  subtítulo opcional · meta              │ ← faixa de título (drag area, full-width)
│─────────────────────────────────────────│
│                                         │
│  conteúdo do painel                     │ ← body (scroll vertical)
│                                         │
│                                         │
│ ◇                                    ◇ │ ← cantos resize (BL + BR)
└─────────────────────────────────────────┘
```

## Título

- **Fonte:** `TypeToken::Lg`. Cor `ColorToken::Text1`. Pintado via [`paint_panel_title`](../../../crates/ph2d-editor-core/src/widget/panel_chrome.rs).
- **Baseline:** `PANEL_TITLE_BASELINE = 18 px` do topo do painel. Constante global — todos os painéis alinham na mesma linha.
- **Subtítulo opcional:** mesma fonte do Inspector (`TypeToken::Sm` em `Text3`), abaixo do título com gap `Spacing::Xs`. Pode conter meta (ex: dimensão `"0.400 × 0.400 m"`).
- **MAIÚSCULAS:** título do PAINEL fica em case normal ("Inspector"). MAIÚSCULAS é regra de [`section_header`](section_header.md), NÃO do título do painel.

## Drag area (barra de título arrastável)

**Regra:** a faixa do título inteira é a hit-zone de drag. NÃO use uma pill estreita centralizada.

- Helper: [`panel_drag_handle_rect(panel, header_h, right_reserve)`](../../../crates/ph2d-editor-core/src/widget/panel_chrome.rs).
- `header_h`: `PANEL_HEADER_H_DEFAULT = 56 px` — cobre título + subtítulo, **pára antes** de qualquer widget interativo (ex: barra de busca da Hierarchy em y=62).
- `right_reserve`: reserva à direita pros ícones de header:
  - `PANEL_HEADER_CLOSE_RESERVE = 40 px` — para painéis com apenas o X (Widget Gallery, Grid Snap).
  - `PANEL_HEADER_ADD_RESERVE = 80 px` — para painéis com X + Add (Hierarchy).
  - `0.0` — para painéis sem ícones (Inspector).
- **Pré-2026-05-24:** pill 80×14 centralizada — muito estreita pra grab confiável.

### Click-block atrás do título (z-order pattern)

Body com scroll pode rolar widgets atrás da faixa de título. Sem ação, esses widgets continuam clicáveis (paint clipado ≠ hit clipado). Solução padrão:

1. Início do paint: `hit_index.register(DRAG_HANDLE, drag_handle_rect)` — registra pra dispatch funcionar quando o body ainda não cresceu.
2. Body paint registra seus widgets (potencialmente atrás do header).
3. **Fim do paint: re-registra `DRAG_HANDLE` no mesmo rect** — empurra pra topo do z-order via last-registered-wins do [`HitIndex`](../../../crates/ph2d-editor-core/src/interaction/hit.rs). Drag ganha sobre o widget scrollado.
4. Ícones do header (X, Add) ficam FORA de `drag_handle_rect` (via `right_reserve`) — clicáveis sem ser sombreados pela drag area.

Todo painel canon segue esse padrão. Esquecer o re-register de fim-de-frame = click-through bug.

## Resize handles (BR + BL)

**Regra:** todo painel é redimensionável dos DOIS cantos inferiores. Não só BR.

- BR: `panel_resize_handle_rect(panel)` (existente).
- BL: `panel_resize_handle_rect_bl(panel)` (novo 2026-05-24, mirror exato).
- Tamanho: `PANEL_RESIZE_HANDLE_SIZE_PX = 16 px` (square).
- Visual: corner dot `ColorToken::Text2`, ⌀ `Spacing::Xs`, inset `7 px` — pintado APÓS o body com [`paint_panel_corner_dot`](../../../crates/ph2d-editor-core/src/widget/panel_chrome.rs) (BR) + [`paint_panel_corner_dot_bl`](../../../crates/ph2d-editor-core/src/widget/panel_chrome.rs) (BL).

### Dispatch

- BR: `BlenderHitKind::ResizeHandle` → `begin_panel_resize` → Move acumula `(dw, dh)` em `panel_resize_delta`.
- BL: `BlenderHitKind::ResizeHandleBl` → `begin_panel_resize_bl` → Move aplica `(−dx, +dx, +dy)`:
  - `panel_resize_delta.w -= dx` (cursor pra direita → largura encolhe)
  - `panel_resize_delta.h += dy`
  - `blender_picker_offset.x += dx` (offset do painel desloca pra direita)
  - **Invariante:** borda DIREITA fica parada. Só a esquerda + bottom se movem.

Ambos terminam com `end_panel_resize()` (compartilhado).

### IDs por painel

Cada painel canon declara DUAS NodeIds resize (BR + BL):

| Painel | BR | BL |
|---|---|---|
| Inspector | `INSP_RESIZE_HANDLE` | `INSP_RESIZE_HANDLE_BL` |
| Hierarchy | `HIER_RESIZE_HANDLE` | `HIER_RESIZE_HANDLE_BL` |
| Widget Gallery | `GAL_RESIZE_HANDLE` | `GAL_RESIZE_HANDLE_BL` |
| Grid Snap | `GS_RESIZE_HANDLE` | `GS_RESIZE_HANDLE_BL` |

Cada uma registrada em [`pre_populate.rs`](../../../crates/ph2d-editor-core/src/screens/hero/pre_populate.rs) (Inspector + Hierarchy) ou no `populate()` do crate-painel (Gallery + Grid Snap) como `InteractiveState::BlenderHit { parent: ..._PANEL, kind: ... }`.

## Surface

[`paint_panel_surface`](../../../crates/ph2d-editor-core/src/widget/panel_chrome.rs):
- Fill: `ColorToken::PanelBg` (BgElev hue/L com ~0.92 alpha — efeito glass).
- Stroke: 1 px `ColorToken::Border`.
- Radius: `PANEL_RADIUS_PX = 16 px`.
- **NÃO pinta drag pill.** Pré-2026-05-24 pintava um traço 36×4 no topo central — removido quando o drag virou full-width (o título passou a ser o cue visual, igual macOS/Windows).

## Checklist quando adicionar painel novo

- [ ] `paint_panel_surface(rect, scene, theme)` antes do body.
- [ ] `panel_drag_handle_rect(rect, PANEL_HEADER_H_DEFAULT, reserve)` registrado pro `*_DRAG_HANDLE` no INÍCIO do paint.
- [ ] `panel_resize_handle_rect(rect)` registrado pro `*_RESIZE_HANDLE`.
- [ ] `panel_resize_handle_rect_bl(rect)` registrado pro `*_RESIZE_HANDLE_BL` (obrigatório).
- [ ] Body paint dentro de `scene.push_clip` + body widgets registram seus rects.
- [ ] FIM do paint: `paint_panel_corner_dot` + `paint_panel_corner_dot_bl` + re-registra drag + os 2 resize handles (z-order on top).
- [ ] Novo painel declara 2 NodeIds resize (BR + BL) + popula em `BlenderHit { parent, kind: ResizeHandle | ResizeHandleBl }`.
- [ ] Ícones no header (X, Add) ficam dentro do `right_reserve` — não overlam com `drag_handle_rect`.

## Layout adaptativo (canon 2026-05-24)

Painéis encolhem (resize do user, dock estreito). Os widgets internos
NÃO devem encolher abaixo dos limites que quebram leitura. Padrões
canônicos pra adaptar:

### Number-input rows: label-acima quando estreito

Quando uma linha não cabe em `label LEFT + tag + chip + tag + chip`
sem reduzir o chip abaixo de [`NUMBER_INPUT_MIN_W_PX = 96`](../../../crates/ph2d-editor-core/src/widget/number_input.rs):

- O label sobe pra linha de cima (sozinho, full-width).
- Os chips ficam na linha de baixo, dividindo `rect.w` igualmente,
  cada um ≥ MIN_W_PX.

Referência viva: [`crates/ph2d-panel-inspector/src/sections.rs::paint_transform_section`](../../../crates/ph2d-panel-inspector/src/sections.rs) (closure `paint_row`).

### Segmented button rows: drop 1-by-1 pra linha abaixo

Quando uma linha de botões segmentados (Atlas / Individual / Hand-packed
etc.) não cabe na largura disponível: **NÃO** quebre o label dentro
do botão ("Hand-\npacked" — anti-pattern). Use
[`paint_segmented_group_adaptive`](../../../crates/ph2d-editor-core/src/widget/panel_chrome.rs).

Comportamento:

1. Mede a largura natural de cada label (text + padding).
2. Se a soma > rect.w, desce o ÚLTIMO botão pra linha abaixo (full-width).
3. Repete se necessário (próximo botão desce; cada um ocupa sua própria row).
4. Retorna a altura total usada. Caller avança `cur_y` pelo retorno.

Demote-order: do fim pra começo (preserva os botões "primários" da esquerda na top row).

## Anti-padrões

- **Pill estreita 80×14 no topo** → user não consegue grab confiável. Use full-width.
- **Drag area cobrindo `right_reserve = 0` quando há ícones** → ícones sombreados pelo drag. Sempre reserve.
- **Esquecer re-register de fim-de-frame** → widget scrollado pra trás do título fica clicável (click-through bug).
- **Só BR resize** → metade da affordance perdida. Sempre BL + BR.
- **Hard-code `header_h`** → calcule baseado em `PANEL_HEADER_H_DEFAULT` (ou justifique no review se for diferente).
