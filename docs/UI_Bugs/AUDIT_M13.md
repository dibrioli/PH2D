# Auditoria de consistência — M13

Snapshot do estado da padronização central da UI ao final do loop
autônomo M13. Confere se a stack passa todos pelos pontos canônicos
(tokens, painters, side-tables) ao invés de literais espalhados.

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
- `radius_scale` → preset de quina (menu de theme)
- `blender_picker_offset`, `blender_drag_anchor`, `eyedropper_pending`,
  `blender_palettes`, `hex_to_blender_parent`, `blender_channel_chip` →
  estado retido do BlenderColorPicker

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

## Clippy

14 warnings → 10 (4 corrigidas neste round: `iter().any` →
`contains` em SECTION_IDS / RADIO_GROUP_IDS / TAB_GROUP_IDS /
TREE_LEAF_IDS). As restantes são:
- 2× "too many arguments" (painters complexos do Inspector — aceito
  por enquanto, todos `#[allow(clippy::too_many_arguments)]`)
- doc list indentation, identical if blocks, loop counter, derefed
  type — cosmético, mantenho.

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

- `radius_scale` está no store mas painters ainda não consultam
  → preset não tem efeito visual ainda.
- Painéis ainda não móveis (Inspector + Hierarchy fixos). Drag
  handle reutilizável do BlenderColorPicker precisa virar
  `panel_offset` genérico.
- Hierarchy não tem drag-and-drop de entidades.
- Clipboard (Cmd+C / Cmd+V) não implementado — exige crate
  `arboard` ou equivalente.

## Próximos passos do loop

1. Painéis móveis (proximo commit).
2. Hierarchy DnD (depois).
3. `radius_scale` consumido pelos painters principais.
