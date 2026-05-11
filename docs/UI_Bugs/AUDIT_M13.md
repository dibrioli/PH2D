# Auditoria de consistência — M13

Snapshot do estado da padronização central da UI ao final do loop
autônomo M13. Confere se a stack passa todos pelos pontos canônicos
(tokens, painters, side-tables) ao invés de literais espalhados.

**Última revisão:** 2026-05-11 (pós-sprint UI polish round 2). Vide
`README.md §10` para bugs/fixes desta segunda rodada.

## Tokens

- **Cores**: todo painter chama `resolve(ColorToken::*, theme)` ou
  `ColorToken::resolve(theme)` → `Color::from_oklch`. Exceções
  intencionais e documentadas:
  - Note title/body: `#212121` hardcoded para garantir contraste
    em qualquer tema sobre o bg highlighter (texto claro de
    temas escuros ficaria invisível). Cf. `paint_one_note`.
  - Highlighter palette (5 cores): `context_menu_overlay::HIGHLIGHTER_RGBA`
    é a fonte única — outlines de seção, bg de notas, swatches do
    menu de cores. Compartilhada.
  - Eyedropper de pixel: bytes vindos da GPU passam sem conversão
    (vello escreve sRGB-encoded em Rgba8Unorm). Documentado em
    `docs/UI_Bugs/README.md` §4.3.
- **Tipografia**: `TypeToken::Xs/Sm/Base/Md/Lg/Xl` em todos os
  painters. `byte_offset_from_click_xy` no dispatch reusa as
  mesmas constantes para garantir caret pixel-perfect (lição
  §9.16 do UI_Bugs).
- **Espaços**: `Spacing::Xs..Xl` em todos os paddings de chrome.
  Literais aparecem só em geometrias de shapes (caret 1.5 px,
  selection rect 1 px de radius — não fazem sentido como token).
- **Raios**: `Radius::Xs..Full` em chrome (botões, painéis,
  modais). Literais pequenos (`0.5`, `0.75`, `1.5`) só em shapes
  finos (dividers, caret) onde não há equivalente semântico.

## Side-tables centrais no `WidgetStore`

Toda mutação de UI passa por uma das side-tables abaixo (sem
estado espalhado em `static`/`thread_local` exceto para
publish-from-paint, com fall-back para o store quando possível):

- `panel_scroll` + `panel_content_h` + `panel_visible_h` →
  scroll + clamp + scrollbar
- `panel_rects` → wheel dispatch pega panel certo
- `collapsed` → headers colapsáveis (Inspector, TreeView)
- `tooltips` → registro genérico (§9.8)
- `context_menu` + `last_context_menu` → menus de contexto
- `section_outline_color` → outline colorido de seção
- `notes_per_panel` → notas por painel
- `picker_target` + `widget_colors` → picker como singleton
- `scrollbar_drag` → drag do scrollbar
- `radius_scale` → preset de quina (menu de theme) — **consumido via
  thread-local em `paint::fill_rounded_rect` / `stroke_rounded_rect`
  (§10.8)**
- `blender_picker_offset`, `blender_drag_anchor`, `eyedropper_pending`,
  `blender_palettes`, `hex_to_blender_parent`, `blender_channel_chip` →
  estado retido do BlenderColorPicker (`blender_*` infra reusada
  por Inspector/Hierarchy panel drag/resize via
  `BlenderHitKind::DragHandle / ResizeHandle`)
- `panel_resize_delta` + `panel_resize_anchor` → resize manual de
  painéis flutuantes (§10.5)
- `hierarchy_parent` → tree-style hierarchy via DnD drop-inside (§10.6),
  com cycle detection e cap de depth=32
- `pending_clipboard_copy` + `pending_clipboard_paste` → bridge
  Cmd+C/V/X ↔ `arboard` (§10.9)

Estado efêmero do dispatch (drag em curso, double-click window)
também vive no store via `last_down_id`, `last_down_at_ns`,
`active_id`, `active_rect`, `focus_id`, `hot_id`.

## Eventos canônicos

`WidgetEvent` cobre Click / Toggled / ValueChanged / TextChanged /
Focus / Blur / SelectionChanged / EyedropperPick. Single point of
truth — `apply_event` chain (Hero → Topbar → LeftRail → Hierarchy →
Inspector) decide quem consome.

## Geometria centralizada

Hit-rects calculados por helpers no widget (single source of
truth painter↔dispatch):

- `Tag::close_rect`, `ContextMenu::entry_rect`,
  `TreeView::row_rect / chevron_rect`, `NumberInput::up_rect /
  down_rect`, `Combobox::clear_button_rect`,
  `Dropdown::popover_rect`, `Modal::close_rect`, `Card::body_rect /
  header_rect / footer_rect`, `Popover::anchor_below`,
  `Vector3Editor::field_rects`, `PillGroup::child_rect`,
  `Scrollbar::track_rect / thumb_rect`,
  `SectionHeader::color_circle_hit_rect`.
- **Chrome de painel** (§10.14):
  `style::panel_drag_handle_rect(panel)`,
  `style::panel_resize_handle_rect(panel)`,
  `style::paint_panel_surface(rect, scene, theme)`,
  `style::paint_panel_corner_dot(rect, scene, theme)` —
  Inspector/Hierarchy consomem; novos painéis seguem.

## Hit-zone priority em scrollable panels (§10.12)

Lição confirmada por bug real: hit zones de **chrome** (drag pill,
resize gripper, close buttons) DEVEM ser registradas POR ÚLTIMO no
`HitIndex` do seu container. `HitIndex::hit()` faz reverse-walk;
sem isso, qualquer scroll/redimensionamento que mova widgets do body
pode sombrear o chrome. Em paint_inspector + paint_hierarchy, o
registro de chrome acontece DEPOIS do `pop_layer` do body.

## Contrato painter ↔ dispatch em campos editáveis (§10.10)

Hard rule: o painter de QUALQUER campo editável (TextInput,
TextArea, NumberInput, Combobox, hex_field, note title/body) DEVE
desenhar texto/caret/seleção a partir da mesma origem que
`byte_offset_from_click_xy` no dispatch:
- single-line: `text_start_x = rect.x + 12`, `text_start_y = rect.y`
- multi-line: `text_start_x = rect.x + 12`, `text_start_y = rect.y + 8`,
  `line_h = font_size + 4`
- hex_field: `text_start_x = rect.x + 8 + 36` (label "Hex" prefix)
- Combobox: `text_start_x = rect.x + 12 + icon_size + 8`
Se um novo painter precisar de origem custom, **publique-a em
side-table** e leia no dispatch. Senão click→caret e drag-select
quebram silenciosamente.

## Clippy

**Estado atual (2026-05-11):** 0 warnings. Round 2 fechou os 10 que
ficaram pendentes do round 1 (todos cosméticos — needless_option_
as_deref via `ts.take()`, collapsible_if no scrollbar, doc list
indentation, manual_range_contains, if_same_then_else em tabs,
explicit_counter_loop em text_area, collapsible_match no shell IME,
3 too_many_arguments via `#[allow]` nos painters do Inspector/notes).
Vide §10.16 (typos) e §10.15 (drift bindgen Windows) pra fixes do
CI.

## O que está realmente padronizado

- Right-click: sempre abre `ContextMenuRequest` no store; o overlay
  pinta + dispatch fecha em outside-click; mesmo caminho para
  CreateNote / SectionOutline / NoteBackground / ThemeSelector.
- Color picker: 1 picker flutuante global. `picker_target` decide
  o alvo (seção color circle, swatch, …). `widget_colors[id]` é
  a fonte da cor.
- Scroll: `panel_scroll` única side-table; `Scrollbar` widget
  central; `dispatch_wheel` clampa via `panel_content_h` /
  `panel_visible_h`.
- Tooltips: `set_tooltip(id, text)` central; painter único
  (`paint_hover_tooltip`).
- Text widgets: `byte_offset_from_click_xy` no dispatch usa o
  mesmo `TypeToken::Base.px()` que os painters; pixel-perfect.

## O que ainda não é centralizado (saída)

Round 1 listou 4 gaps; round 2 fechou todos:

- ✅ `radius_scale` consumido — thread-local em `paint.rs` (§10.8)
- ✅ Painéis Inspector + Hierarchy móveis E redimensionáveis
  (§10.3-10.5). Drag handle do BlenderColorPicker virou
  infra de painel genérico via `BlenderHitKind::DragHandle` +
  `ResizeHandle`.
- ✅ Hierarchy DnD funcional, incluindo drop-inside parenting
  (§10.6) e drop-target x-aware (§10.7).
- ✅ Cmd+C/V/X clipboard via `arboard` (§10.9).

Gaps remanescentes (não bloqueantes):
- Sem preedit visível (italic caret) durante IME composing — só
  Commit. Vide §10.2.
- DnD reparenting não persiste em scene save/load (M13 ainda não
  toca scene-graph).
- Hierarchy não tem fold/unfold de subtrees ainda.

## Snapshot congelado

`screens/hero_ref/` é uma cópia verbatim de `screens/hero/` no
fim deste round, ativada via cargo feature `reference-snapshot`.
Launcher: `reference.command`. Permite A/B visual contra o working
hero enquanto a iteração continua. Vide §10.17.

## Próximos passos do loop

UI core funcional CONCLUÍDO. Próxima fase depende do projeto-piloto
que vai exercitar o editor com cena real (M14+ em
[`docs/plans/2026-05-post-spike.md`](../plans/2026-05-post-spike.md)).
Gaps prováveis a serem expostos pelo piloto:
- Scene-graph persistente conectado ao Hierarchy (DnD reparent →
  scene save).
- Inspector multi-select (atualmente single-select via
  `HeroSelection`).
- Undo/redo do toolkit do editor (separado do undo do scripting).
