# UI bugs — lições aprendidas

Padrões que quebraram a UI do PH2D durante o sprint do BlenderColorPicker
(M13). Cada item: **sintoma** / **causa** / **fix** / **onde olhar no
código**. Se for tocar num painter ou no dispatch, releia esta lista
antes — todos os erros aqui já voltaram a aparecer pelo menos uma vez
durante o desenvolvimento.

---

## 1. Hit-testing

### 1.1 Container "engole" os filhos clicáveis

- **Sintoma**: clicar nos sub-controles (wheel, sliders, swatches) não
  faz nada; sempre acerta o pai (BlenderPicker).
- **Causa**: `HitIndex::hit()` percorre `back-to-front` e devolve o
  primeiro rect que contém o ponto. Registrar o `parent_id` com o
  rect inteiro **depois** dos filhos sombra todo mundo.
- **Fix**: **NÃO registre o pai** quando ele só interage via filhos.
  Registre só os sub-controles; eventos viram mutações no pai via
  apply (`apply_blender_hit` → mutate parent state).
- **Código**: [`widget/blender_color_picker/paint.rs`
  `paint_blender_color_picker_with_store` — comentário "Intentionally
  NOT registering `parent_id`"](../../crates/ph2d-editor/src/widget/blender_color_picker/paint.rs).

### 1.2 Hit-rect maior que o controle visível

- **Sintoma**: drag do channel slider mapeia errado (offset perto da
  borda); chip numérico não recebe foco.
- **Causa**: registrava o **row inteiro** (label + track + chip) como
  hit do slider. A normalização `(px - rect.x) / rect.w` ficava
  enviesada e o chip ficava sem rect próprio.
- **Fix**: registre **só o track** como hit do slider; chip tem seu
  próprio NodeId + rect. `paint_slider_with_chip` faz isso por padrão.
- **Código**: [`widget/slider_with_chip.rs`](../../crates/ph2d-editor/src/widget/slider_with_chip.rs).

### 1.3 Slots estáticos vs dados dinâmicos

- **Sintoma**: usuário adiciona um swatch novo e ele não responde ao
  click (left ou right).
- **Causa**: hit-index slots eram pré-registrados (ex.: 12 swatches);
  além disso, sem hit-rect → invisível pro pointer.
- **Fix**: pré-aloque slots com folga (24-27) e cape o `push` no
  store + esconda o "+" quando atinge o teto.
- **Código**: [`screens/hero/ids.rs` `BLENDER_SWATCH_0..26`](../../crates/ph2d-editor/src/screens/hero/ids.rs)
  + [`interaction/state.rs` `blender_palette_push` cap=27](../../crates/ph2d-editor/src/interaction/state.rs).

---

## 2. Pointer dispatch

### 2.1 Move re-aplicando ação one-shot

- **Sintoma**: clique em "+ swatch" adiciona N cópias da mesma cor;
  click no Eyedropper trigger múltiplo.
- **Causa**: `apply_blender_hit` no Move handler re-aplicava em
  TODOS os BlenderHit kinds. Para os "drag" (Wheel/ValueSlider/
  ChannelSlider) é correto — para botões / toggles / swatches /
  drag-handle / eyedropper é catastrófico.
- **Fix**: gate explícito por kind no Move handler; só Wheel,
  ValueSlider e ChannelSlider repetem.
- **Código**: [`interaction/dispatch.rs` `PointerKind::Move` —
  `let drag_apply = matches!`](../../crates/ph2d-editor/src/interaction/dispatch.rs).

### 2.2 Click fora do widget focado não comita

- **Sintoma**: usuário digita no hex, clica no canvas → buffer fica
  pendurado, picker.value não muda.
- **Causa**: Down handler só comita prev_focus quando o clique novo
  acerta um widget focusável. Click "no nada" deixava o estado
  pendurado.
- **Fix**: comita prev_focus sempre que `new_focus != Some(old)`,
  inclusive quando `new_focus = None`.
- **Código**: [`interaction/dispatch.rs` `PointerKind::Down` —
  comentário "blur+commit … even when click landed in dead space"](../../crates/ph2d-editor/src/interaction/dispatch.rs).

### 2.3 Right-click não distinguido de left

- **Sintoma**: right-click em swatch da paleta não deletava.
- **Causa**: `PointerEvent` não carregava o botão (host descartava
  o `winit::MouseButton`).
- **Fix**: `PointerButton::{Primary, Secondary, Middle}` no
  `ph2d-host`; shell mapeia winit; dispatch lê `event.button` e
  ramifica (PaletteSwatch + Secondary → remove).
- **Código**: [`crates/ph2d-host/src/events.rs`
  `PointerButton`](../../crates/ph2d-host/src/events.rs).

---

## 3. Painters & rounded chrome

### 3.1 Fill sharp dentro de stroke rounded

- **Sintoma**: "quinas minúsculas" aparecendo nos cantos
  (hue strip, color preview, swatches translúcidos).
- **Causa**: `scene.fill(..., &KurboRect)` é fill **retangular**;
  o stroke ao redor é arredondado. Os pixels do fill estouram os
  cantos do stroke.
- **Fix**: use `kurbo::RoundedRect` no path do fill (mesmo radius
  que o stroke). Para padrões compostos (checker), inset os
  sub-rects pelo radius.
- **Código**: [`widget/blender_color_picker/value_slider.rs` —
  `RoundedRect` no fill da hue strip](../../crates/ph2d-editor/src/widget/blender_color_picker/value_slider.rs);
  [`widget/color_swatch.rs` `paint_checker(rect, corner_radius)`
  inset](../../crates/ph2d-editor/src/widget/color_swatch.rs);
  [`widget/blender_color_picker/paint.rs` `paint_color_preview`
  checker inset](../../crates/ph2d-editor/src/widget/blender_color_picker/paint.rs).

### 3.2 Cursor / thumb extrapolando o rect

- **Sintoma**: anel do cursor SV ou thumb da hue strip "vaza" os
  cantos (ear artifact) quando o valor está em 0 ou 1.
- **Causa**: posição calculada como `rect.x + s * rect.w` sem
  considerar o raio/largura do desenho. No extremo, metade do
  ornamento fica fora do rect.
- **Fix**: clamp `inwards` por `(radius + outer_stroke_half)`.
- **Código**: [`widget/blender_color_picker/wheel.rs` — clamp do
  cursor](../../crates/ph2d-editor/src/widget/blender_color_picker/wheel.rs);
  [`value_slider.rs` thumb_x clamped + altura igual ao rect](../../crates/ph2d-editor/src/widget/blender_color_picker/value_slider.rs).

### 3.3 Caret/seleção com largura aproximada

- **Sintoma**: caret aparece em cima da letra (não entre); seleção
  por drag fica deslocada.
- **Causa**: aproximação `font_size * 0.55` por char para fontes
  proporcionais.
- **Fix**: medir o prefixo via `text_system.layout(prefix,
  font_size, INF).width()` quando o painter pode rodar (caret,
  seleção rect). No dispatch (que não tem `text_system`), a
  aproximação é tolerável pra drag-select.
- **Código**: [`widget/blender_color_picker/hex_field.rs` caret
  measurement real](../../crates/ph2d-editor/src/widget/blender_color_picker/hex_field.rs);
  [`widget/slider_with_chip.rs` `paint_number_chip` mede
  via `text_system`](../../crates/ph2d-editor/src/widget/slider_with_chip.rs).

### 3.4 Label rendered como pill ativo

- **Sintoma**: labels do channel slider (Red/Green/Blue/Alpha)
  pareciam "botões selecionados".
- **Causa**: fundo `AccentPress` + texto `AccentFg` num label que
  não é interativo.
- **Fix**: texto plano, `Text2`, sem fundo.
- **Código**: [`widget/slider_with_chip.rs` `paint_text_centered`
  do label](../../crates/ph2d-editor/src/widget/slider_with_chip.rs).

---

## 4. Color spaces

### 4.1 HSV round-trip perde H+S em V→0

- **Sintoma**: ao escolher cores escuras (V<0.05), cursor SV pula,
  thumb da hue strip volta pro vermelho.
- **Causa**: RGBA→HSV em pixels com value=0 retorna H=0, S=0
  (indeterminados pela definição). Quantização em alpha-byte
  amplifica.
- **Fix**: `hsv_h` e `hsv_s` retidos no `BlenderPicker` state;
  todos os apply paths atualizam-nos quando recebem uma escolha
  canônica; painters leem **retidos**, não rgba_to_hsv.
- **Código**: [`interaction/state.rs` `BlenderPicker { hsv_h,
  hsv_s }`](../../crates/ph2d-editor/src/interaction/state.rs);
  [`set_blender_value_with_hsv`](../../crates/ph2d-editor/src/interaction/state.rs).

### 4.2 Hue strip wrap right→left

- **Sintoma**: drag o thumb pra direita até o fim, ele pula pro
  início.
- **Causa**: `hsv_to_rgba8` faz `rem_euclid(6.0)` (h=1.0 vira
  red); na próxima leitura via `rgba_to_hsv`, h volta a 0.
- **Fix**: thumb lê do retido `hsv_h` (clamp [0,1] **sem**
  rem_euclid). H=1.0 stays at right edge mesmo internamente
  produzindo red.
- **Código**: [`interaction/state.rs` `set_blender_value_with_hsv`
  comentário sobre clamp em vez de rem_euclid](../../crates/ph2d-editor/src/interaction/state.rs).

### 4.3 Eyedropper aplicava sRGB encoding extra

- **Sintoma**: cor capturada saía mais clara que a exibida.
- **Causa**: A textura intermediate é `Rgba8Unorm` (linear no
  format), mas Vello escreve **bytes sRGB-encoded** direto (peniko
  `Color` é sRGB). Aplicar `linear_to_srgb` no readback dobra a
  encoding.
- **Fix**: ler bytes crus, sem conversão; eles já casam com
  `ColorValue::from_rgba8`.
- **Código**: [`crates/ph2d-render/src/vello_pass.rs` `read_pixel`
  + comentário "Note on color space"](../../crates/ph2d-render/src/vello_pass.rs).

---

## 5. Text editing

### 5.1 Buffer não sincroniza com source-of-truth

- **Sintoma**: hex field mostra valor antigo depois de o disco
  mudar a cor.
- **Causa**: painter usava `buffer.unwrap_or(fallback_hex)`. Quando
  não focado, o buffer "stale" do TextInput vencia o fallback
  derivado.
- **Fix**: `display = match buffer { Some(b) if focused => b, _ =>
  fallback_hex }`. **Buffer só é fonte enquanto focado**; senão
  derivar do estado canônico.
- **Código**: [`widget/blender_color_picker/hex_field.rs`
  `paint_hex_field_with_state`](../../crates/ph2d-editor/src/widget/blender_color_picker/hex_field.rs).

### 5.2 Enter / blur sem commit

- **Sintoma**: usuário digita no hex / chip e a cor não muda
  mesmo após Enter ou Tab.
- **Causa**: handler do KEY_ENTER caía em `apply_click` genérico;
  blur via Down não chamava commit.
- **Fix**: handler dedicado pro hex (`commit_hex_buffer`) e pro
  number chip (`commit_number_buffer` + linkagem aos pais via
  `link_blender_channel`); chamados em Enter, Tab (cycle_focus) e
  Down quando prev_focus muda.
- **Código**: [`interaction/dispatch.rs` `commit_hex_buffer`,
  `commit_number_buffer`](../../crates/ph2d-editor/src/interaction/dispatch.rs).

### 5.3 Selection-anchor para "select all" / drag-to-select

- **Sintoma**: duplo-click não selecionava tudo; drag não estendia
  seleção; Cmd+A inexistente.
- **Causa**: `TextInput`/`NumberInput`/`Combobox` só tinham `caret`
  como cursor — nenhuma noção de range.
- **Fix**: `selection_anchor: Option<usize>` em todos três; helpers
  `select_all_in_text_widget`, `delete_selection_if_any`,
  `collapse_selection`; double-click via timestamp delta no
  `WidgetStore::record_pointer_down`; Cmd/Ctrl+A com `KEY_KEY_A`.
- **Código**: [`interaction/state.rs` `selection_anchor`
  fields](../../crates/ph2d-editor/src/interaction/state.rs);
  [`interaction/dispatch.rs` helpers](../../crates/ph2d-editor/src/interaction/dispatch.rs).

---

## 6. Visual / chrome consistency

### 6.1 Inspector slider dual-row vs picker single-row

- **Sintoma**: slider do Inspector parecia "mais alto" que o do
  picker.
- **Causa**: Inspector pintava `head_rect` (dot + label) +
  `body_rect` (slider+chip) — duas linhas. Picker pintava tudo
  numa linha só.
- **Fix**: `paint_slider_with_chip` é a referência canônica única;
  Inspector e picker chamam a mesma função em single-row layout.
- **Código**: [`widget/slider_with_chip.rs`](../../crates/ph2d-editor/src/widget/slider_with_chip.rs)
  + [`screens/hero/inspector.rs paint_inspector_field`](../../crates/ph2d-editor/src/screens/hero/inspector.rs)
  (Slider/LinkedSlider em single-row, outros kinds em dual-row).

### 6.2 Pílula segmented "Active" muito sutil

- **Sintoma**: Linear/Perceptual e RGB/HSV pareciam iguais
  (selected vs unselected).
- **Causa**: pill ativo usava `AccentSoft` (alpha 0.16) sobre Bg2
  → quase invisível em temas escuros.
- **Fix**: pill ativo usa `Accent` (saturado) + `AccentFg` no texto.
- **Código**: [`widget/radio_group.rs paint_segmented`](../../crates/ph2d-editor/src/widget/radio_group.rs).

### 6.3 Tabs de paleta única ("Default")

- **Sintoma**: pílula "Default" ocupava espaço sem informação.
- **Causa**: paleta single-mode mas painter ainda renderizava o
  Tabs.
- **Fix**: skipa a faixa de tabs quando `palettes.len() == 1`.
- **Código**: [`widget/blender_color_picker/palette.rs`
  comentário "Single-palette picker: skip the palette-name tabs"](../../crates/ph2d-editor/src/widget/blender_color_picker/palette.rs).

---

## 7. Estado persistido vs ephemeral

### 7.1 Painel movível precisa de offset retido

- **Sintoma**: ao soltar o drag, painel volta pra posição inicial.
- **Causa**: posição era recalculada cada frame a partir do
  `layout`. Drag só atualizava o frame atual.
- **Fix**: side-table `panel_offset: BTreeMap<NodeId, (f32, f32)>`
  no `WidgetStore`, populado por dispatch durante drag (`Down →
  begin_blender_drag`, `Move → set_blender_picker_offset`,
  `Up → end_blender_drag`); painter aplica o offset ao base_rect
  com clamp pro viewport.
- **Código**: [`interaction/state.rs blender_picker_offset
  + blender_drag_anchor`](../../crates/ph2d-editor/src/interaction/state.rs).
  Mecanismo é panel-agnostic (NodeId é a chave), reusado pelo
  showcase.

---

## 8. Pegadinhas de painter pra futuras revisões

- **Vello + peniko `Color`**: textura intermediate é `Rgba8Unorm`
  (rotulada linear) mas armazena bytes **sRGB-encoded**. Não
  converta de novo.
- **`HitIndex::hit()` walks back-to-front**: o último registrado
  ganha em sobreposições. Registre sub-controles ANTES do
  container; ou não registre o container.
- **`f32::clamp(min, max)` panica se `min > max`**: ao calcular
  bounds dinâmicos (e.g. `viewport.h - panel.h`), trate o caso
  degenerado: `clamp(min.min(max), min.max(max))` ou caminho
  alternativo.
- **`rem_euclid` em valores normalizados [0,1]**: faz wrap de 1.0
  → 0.0. Quando o usuário precisa de "extremo direito", use
  clamp em vez de rem_euclid.
- **`format!()` per-frame**: bumpalo arena evita o heap-alloc do
  evento, mas chamadas de `format!`/`String::push_str` no painter
  ainda alocam. Para hot-path use `paint_text_centered` com slices
  ou builders sobre BumpString.

---

## 9. Auditoria pós-picker (loop M13.x)

Padrões aplicados em massa quando varremos o catálogo inteiro depois
do BlenderColorPicker estabilizar. Cada item generaliza algo que já
estava implícito nos §1–§8.

### 9.1 Helpers de hit (`close_rect`, `entry_rect`, `row_rect`, `chevron_rect`)

- **Sintoma**: consumidores recalculavam a geometria interna dos
  widgets para registrar sub-hits (ex.: close X de uma `Tag`,
  chevron de uma linha de `TreeView`, item de um `ContextMenu`). Era
  fácil errar paddings e descolar do que o painter desenhava.
- **Fix**: cada widget composto expõe métodos públicos que devolvem
  os rects de cada sub-elemento — fonte única usada por painter e
  hit-test. Padrão é `widget.<elemento>_rect(host[, index])`.
- **Código**: [`widget/tag.rs Tag::close_rect`](../../crates/ph2d-editor/src/widget/tag.rs);
  [`widget/context_menu.rs ContextMenu::entry_rect`](../../crates/ph2d-editor/src/widget/context_menu.rs);
  [`widget/tree_view.rs TreeView::row_rect / chevron_rect`](../../crates/ph2d-editor/src/widget/tree_view.rs).

### 9.2 Body-clip helpers em surface containers

- **Sintoma**: conteúdo pintado dentro de um `Modal`/`Card`/`Popover`
  vazava pelos cantos arredondados (mesma família de bug do §3.1,
  mas em escala de container e não de glifo).
- **Fix**: cada container expõe `push_<container>_body_clip` /
  `pop_<container>_body_clip` que empilham um `scene.push_clip`
  pelo `body_rect`. Consumidor chama em par.
- **Código**: [`widget/modal.rs push_modal_body_clip`](../../crates/ph2d-editor/src/widget/modal.rs);
  [`widget/card.rs push_card_body_clip`](../../crates/ph2d-editor/src/widget/card.rs);
  [`widget/popover.rs push_popover_clip`](../../crates/ph2d-editor/src/widget/popover.rs).

### 9.3 Medição real em TODO painter com `text_system`

- **Sintoma**: rótulos secundários (key shortcut no `ListItem`,
  segmentos do `StatusBar`) ficavam apertados/sobrescritos. Mesmo
  bug que o §3.3 mas em widgets que escapavam da regra.
- **Fix**: substituir QUALQUER `chars().count() * <const>` por
  `text_system.layout(s, font_size, INF).width()`. Único caso
  aceito: estimativas pré-text-system (ex.: `preferred_width`
  chamado pela fase de layout, antes da paint pass).
- **Código**: [`widget/list_item.rs paint_list_item value_w`](../../crates/ph2d-editor/src/widget/list_item.rs);
  [`widget/status_bar.rs paint_status_bar widths`](../../crates/ph2d-editor/src/widget/status_bar.rs).

### 9.4 ProgressBar: raio do fill clamped à largura

- **Sintoma**: valores baixos (0–10 %) exibiam stadium com caps
  iguais ao trilho. Visualmente o fill parecia maior que o valor.
- **Fix**: `fill_radius = radius.min(fill_w * 0.5)`. Quando o fill
  é mais estreito que o diâmetro do cap, o cap encolhe junto.
- **Código**: [`widget/progress_bar.rs`](../../crates/ph2d-editor/src/widget/progress_bar.rs).

### 9.5 Checkbox indeterminate: glifo certo

- **Sintoma**: o estado "mixed" pintava um `+` (Plus) — lia como
  "adicionar" e não como "alguns filhos selecionados".
- **Fix**: adicionado `IconId::Minus` (path único `M5 12h14`); o
  pintor usa Minus para Indeterminate.
- **Código**: [`icons.rs IconId::Minus`](../../crates/ph2d-editor/src/icons.rs);
  [`widget/checkbox.rs paint_checkbox`](../../crates/ph2d-editor/src/widget/checkbox.rs).

### 9.6 Toggle disabled deve preservar on/off

- **Sintoma**: `Toggle` desabilitado virava cinza tanto em on quanto
  em off — usuário não conseguia ler o valor.
- **Fix**: disabled+on → `AccentSoft`; disabled+off → `Border`. Estado
  on continua legível mesmo travado.
- **Código**: [`widget/toggle.rs paint_toggle`](../../crates/ph2d-editor/src/widget/toggle.rs).

### 9.7 Tabs hover overlay

- **Sintoma**: tabs não-selecionadas não davam feedback sob o cursor.
- **Fix**: `paint_tabs_with_hover(hovered: Option<usize>, …)` desenha
  um `Bg2` rounded por baixo da tab quando hovered e não-selecionada.
  `paint_tabs` continua sendo o entry point estático.
- **Código**: [`widget/tabs.rs paint_tabs_with_hover`](../../crates/ph2d-editor/src/widget/tabs.rs).

### 9.8 Tooltip registry genérico

- **Sintoma**: tooltips estavam hardcoded em `tooltip_for(id)` num
  match enorme; cada widget novo exigia editar topbar.rs.
- **Fix**: `WidgetStore::set_tooltip(id, text)` + `tooltip_for(id)`
  como side-table. `paint_hover_tooltip` lê do store. Qualquer
  painter pode registrar tooltip via uma linha.
- **Código**: [`interaction/state.rs tooltips`](../../crates/ph2d-editor/src/interaction/state.rs);
  [`screens/hero/topbar.rs paint_hover_tooltip`](../../crates/ph2d-editor/src/screens/hero/topbar.rs).

### 9.9 TextArea multi-line caret/selection

- **Sintoma**: `TextArea` não mostrava caret nem seleção quando
  focado — invisível pro usuário.
- **Fix**: `paint_text_area_with_state(caret, selection_anchor, …)`
  divide `value` por `\n`, mede prefixo da linha do caret com
  `text_system.layout`, e desenha caret + AccentSoft por linha. Seleção
  multi-linha estende até a margem direita nas linhas intermediárias
  (convenção de editor).
- **Código**: [`widget/text_area.rs paint_text_area_with_state`](../../crates/ph2d-editor/src/widget/text_area.rs).

### 9.10 Click→byte multi-line aware (TextArea) e snap por linha

- **Sintoma**: clicar na linha 2 do `TextArea` posicionava o caret
  na linha 1 no X equivalente; clicar à direita do texto em uma
  linha pulava para o fim do buffer (em vez do fim da LINHA).
- **Causa**: `byte_offset_from_click_x` ignorava o `y` do clique e
  fazia `.min(text.len())` no buffer inteiro.
- **Fix**: `byte_offset_from_click_xy` detecta multi-linha pela
  presença de `\n`, deriva o `line_idx` do `click_y - text_start_y`
  / `line_h`, e clampa o offset local em `[0, line.len()]` —
  clicar à direita de uma linha curta vai pro fim DA LINHA, não
  do buffer.
- **Código**: [`interaction/dispatch.rs byte_offset_from_click_xy`](../../crates/ph2d-editor/src/interaction/dispatch.rs).

### 9.12 Dropdown aberto: lista flutuante real

- **Sintoma 1**: a lista do `Dropdown` aberto era pintada como linhas
  individuais com fill `BgElev` — em temas onde o painel host
  também é `BgElev`, a lista "sumia" no fundo.
- **Sintoma 2**: a primeira linha da lista tocava a borda inferior
  do chip; sem gap, viravam uma surface só.
- **Sintoma 3**: o tooltip-pill da hover-tooltip registry caía em
  cima da primeira opção (mesma geometria que o pill esperaria).
- **Fix**: `Dropdown::popover_rect(chip)` retorna um painel
  flutuante completo (BgElev + Border + radius Md), 4 px abaixo do
  chip. `option_rect` agora inset dentro desse painel. `paint_dropdown`
  pinta o painel, e a tooltip-pintora (`paint_hover_tooltip`) faz
  early-out quando o hot widget é um `Dropdown { open: true, .. }`
  ou `Combobox { open: true, .. }`.
- **Código**: [`widget/dropdown.rs Dropdown::popover_rect`](../../crates/ph2d-editor/src/widget/dropdown.rs);
  [`screens/hero/topbar.rs paint_hover_tooltip` early-out](../../crates/ph2d-editor/src/screens/hero/topbar.rs).

### 9.13 SectionHeader v2: chevron-only + UPPERCASE + colapso real

- **Sintoma**: header pintava um ponto rosa decorativo + label `Xs`
  em `Text2` minúsculo — visualmente fraco e sem affordance de
  "clique para minimizar".
- **Fix**: removido o ponto. Chevron sempre desenhado (ChevronDown
  aberto, ChevronRight colapsado). Label vira `to_uppercase()` em
  `TypeToken::Sm` com `Text1`. Estado de colapso vive em
  `WidgetStore::collapsed` (side-table genérica); `inspector::apply_event`
  toggleia em Click e cada section painter early-outs quando
  `!open`.
- **Código**: [`widget/section_header.rs paint_section_header`](../../crates/ph2d-editor/src/widget/section_header.rs);
  [`interaction/state.rs collapsed` side-table](../../crates/ph2d-editor/src/interaction/state.rs);
  [`screens/hero/inspector.rs apply_event + paint_collapsible_header`](../../crates/ph2d-editor/src/screens/hero/inspector.rs).

### 9.14 RadioGroup/Tabs/TreeView mantendo seleção entre frames

- **Sintoma**: clicar Mid no segmented RadioGroup voltava pra Low
  imediatamente; tabs sample não trocava conteúdo; Group TreeView
  não colapsava.
- **Causa**: cada widget era modelado como N `Button`s. O default
  `set_widget_released` no Up handler reverte cada botão pra
  `Hovered`/`Normal` — nenhum continua `Pressed`. O painter lia o
  índice ativo via `Button::Pressed`, então achava que nada estava
  selecionado e caía no índice 0.
- **Fix**: `inspector::apply_event` intercepta o `Click` do grupo e
  chama `pin_button_selection(selected, group)` que força o
  clicado em `Pressed` e os irmãos em `Normal`. Mesmo padrão usado
  para tabs e folhas do TreeView. Para o root do TreeView, click
  toggleia `is_collapsed(ROOT)`; a paint passa a chamar
  `tree.expand(ROOT)` somente quando não colapsado.
- **Código**: [`screens/hero/inspector.rs apply_event + pin_button_selection`](../../crates/ph2d-editor/src/screens/hero/inspector.rs).

### 9.16 Click→caret com medição real (pixel-perfect)

- **Sintoma**: clicar no meio de uma palavra colocava o caret 1-2
  caracteres antes/depois — em texto multi-linha pior porque o
  erro acumulava por linha.
- **Causa**: `byte_offset_from_click_x` usava `font_size *
  APPROX_ADVANCE_RATIO` para mapear pixel→char (heurística para
  fonts proporcionais).
- **Fix**: `dispatch_pointer_with_text(text_system: Some(...))`
  threadando o `TextSystem` desde o shell. `nearest_byte_on_line`
  itera os char boundaries da linha, mede cada prefixo com
  `text_system.layout(prefix).width()`, e devolve o byte cuja
  posição pixel está mais próxima do clique. Pixel-perfect. Testes
  caem na variante antiga (sem TS) para evitar 50ms por test.
- **Código**: [`interaction/dispatch.rs nearest_byte_on_line`](../../crates/ph2d-editor/src/interaction/dispatch.rs);
  [`screens/hero.rs HeroScreen::handle_pointer_with_text`](../../crates/ph2d-editor/src/screens/hero.rs);
  [`shells/desktop/src/main.rs forward_pointer_to_hero`](../../shells/desktop/src/main.rs).

### 9.17 Dropdown popover em segunda passada + geometria apertada

- **Sintoma 1**: lista do Dropdown sumia atrás das seções pintadas
  depois dela na mesma frame.
- **Sintoma 2**: itens visualmente "deslocados para baixo" dentro
  do popover (padding interno excessivo).
- **Fix**: `paint_dropdown` virou um wrapper sobre `paint_dropdown_chip`
  + `paint_dropdown_popover`. Inspector chama o chip in-place e
  stasha o rect num thread-local; ao FIM de `paint_inspector`
  (depois de TODAS as seções) chama `paint_dropdown_popover` —
  z-order correto sem precisar de cena multi-layer. Padding
  interno reduzido para 2 px (era 6 px).
- **Código**: [`widget/dropdown.rs paint_dropdown_chip + paint_dropdown_popover`](../../crates/ph2d-editor/src/widget/dropdown.rs);
  [`screens/hero/inspector.rs PENDING_DROPDOWN_CHIP`](../../crates/ph2d-editor/src/screens/hero/inspector.rs).

### 9.18 NumberInput stepper handlers no dispatch

- **Sintoma**: as setinhas ▲▼ no chip do NumberInput eram
  decorativas — clicar não fazia nada.
- **Fix**: `apply_number_stepper_if_hit` no Down handler detecta
  click dentro de `NumberInput::up_rect/down_rect` (geometria do
  próprio widget) e aplica +/- step. Step heurístico: `0.01`
  quando buffer contém `.` (fractional), `1.0` caso contrário.
  Espelha pro slider linkado se houver. Emite `ValueChanged`.
- **Código**: [`interaction/dispatch.rs apply_number_stepper_if_hit`](../../crates/ph2d-editor/src/interaction/dispatch.rs).

### 9.19 Glifos macOS Cmd/Return → tofu

- **Sintoma**: tooltips e o ListItem sample mostravam `□S` / `□O`
  em vez de `⌘S` / `⌘O`.
- **Causa**: o font stack default do parley (system sans-serif)
  não cobre o Unicode Technical Symbols block (U+2300..U+23FF) —
  glifos U+2318 (⌘) e U+21B5 (↵) caem em tofu.
- **Fix**: substituir por ASCII (`Cmd+S`, `Cmd+Enter`, `Cmd+O`).
  O dígito-de-meio U+00B7 (·) está em Latin-1 Supplement, é
  seguro manter.
- **Código**: [`screens/hero/topbar.rs populate tooltips`](../../crates/ph2d-editor/src/screens/hero/topbar.rs);
  [`screens/hero/inspector.rs ListItem value`](../../crates/ph2d-editor/src/screens/hero/inspector.rs).

### 9.20 Inspector scroll clamp + separador colorido por seção

- **Sintoma**: scroll do wheel era ilimitado — passava o último
  elemento por centenas de pixels.
- **Fix**: `paint_inspector` publica `last_inspector_content_h`
  num thread-local; `paint_hero_screen` clamps `panel_scroll` ao
  `content_h - visible_h` antes da próxima frame de paint
  (one-frame stale OK — invisível).
- **Sintoma 2**: seções sem divisor visual ficavam coladas
  visualmente.
- **Fix**: `paint_section_separator` desenha um `Accent`-tinted
  pill de 2 px alto, inset 16 px horizontal, ao fim de cada
  section block.
- **Código**: [`screens/hero/inspector.rs paint_section_separator + LAST_CONTENT_H`](../../crates/ph2d-editor/src/screens/hero/inspector.rs).

### 9.15 Vector3Editor com buffer vivo

- **Sintoma**: clicar nos chips X/Y/Z focava (border accent) mas
  digitar não aparecia na caixa.
- **Causa**: o sample chamava o estático `paint_vector3_editor`,
  que passa `[None; 3]` para os buffers → o `paint_number_input_with_buffer`
  sempre exibe `input.value` em vez do buffer em edição.
- **Fix**: o Inspector lê o estado completo (state/value/buffer/caret/anchor)
  de cada chip via `read_number_input` e chama
  `paint_vector3_editor_with_state` com `Some(buffer)` por eixo.
  Mesma fix do TextArea (§9.10) aplicada a NumberInput composto.
- **Código**: [`screens/hero/inspector.rs paint_vector_section`](../../crates/ph2d-editor/src/screens/hero/inspector.rs).

### 9.11 Combobox click X-offset corrigido + clear-✕ central

- **Sintoma 1**: caret do `Combobox` "escorregava" ~24 px à direita
  do ponto clicado.
- **Causa**: `byte_offset_from_click_x` usava `rect.x + 12` como
  `text_start_x` para Combobox, mas o painter desenha o texto após
  o ícone de busca + gap (`pad_x + icon_size + Spacing::Md`).
- **Sintoma 2**: campo de busca não tinha como esvaziar a query
  sem teclado (Cmd+A → Backspace).
- **Fix**: `Combobox::clear_button_rect(host)` exposto pelo widget
  (única fonte da geometria do ✕). Painter desenha o `IconId::Close`
  quando `!query.is_empty()`; dispatch detecta Down dentro do rect
  via `clear_combobox_if_button_hit` e zera query + caret + anchor
  emitindo `TextChanged`.
- **Código**: [`widget/combobox.rs Combobox::clear_button_rect`](../../crates/ph2d-editor/src/widget/combobox.rs);
  [`interaction/dispatch.rs clear_combobox_if_button_hit`](../../crates/ph2d-editor/src/interaction/dispatch.rs).

---

## 10. Auditoria pós-M13 (sprint 2026-05-09 → 2026-05-11)

Segunda rodada de polish — input pipeline maduro, paineis flutuantes,
DnD parenting, clipboard e snapshot de referência. Cada item segue o
mesmo formato (sintoma / causa / fix / código).

### 10.1 Caret não acompanha espaço final (parley trim)

- **Sintoma**: ao apertar SPACE em `TextInput`/`TextArea`/`Combobox`/
  `NumberInput`/hex/note, a letra entra no buffer mas o caret fica
  parado visualmente; o efeito repete a cada espaço.
- **Causa**: `parley::Layout::width()` corta whitespace à direita.
  Todo painter de caret/seleção media o prefixo via `text_system
  .layout(prefix, font_size, INF).width()` — para prefixos terminando
  em espaço, a medição era idêntica ao prefixo sem o espaço.
- **Fix**: novo helper `TextSystem::prefix_width(prefix, font_size)`
  em [`ph2d-text/src/system.rs`](../../crates/ph2d-text/src/system.rs)
  que mede `prefix + sentinel('|')`, mede só o sentinel, e subtrai.
  Branch fast-path quando `!ends_with(' '|'\t')`. Aplicado em todos
  os painters de caret/seleção (text_input, text_area, combobox,
  number_input, slider_with_chip, hex_field, inspector note painters)
  e em `nearest_byte_on_line` no dispatch.
- **Lição**: NUNCA chame `.layout(...).width()` direto pra medir
  prefixos em texto editável. Use sempre `prefix_width`.

### 10.2 Letras acentuadas PT-BR (é, á, ç) não aparecem no macOS

- **Sintoma**: dead-key sequences (`'` + `a` = `á`) não inserem nada
  no buffer.
- **Causa**: macOS NSTextInputClient engole o keystroke dead-key e
  emite o glifo composto via `Ime::Commit`. O shell escutava só
  `WindowEvent::KeyboardInput` (cuja `text` chega vazia pro dead-key).
- **Fix**: shell habilita IME no construtor da janela
  (`window.set_ime_allowed(true)`) e adiciona um arm
  `WindowEvent::Ime(Ime::Commit(text))` que itera `text.chars()` e
  chama `forward_text_to_hero` por char não-controle. `Preedit`
  ignorado por ora (sem caret preedit italic ainda).
- **Código**: [`shells/desktop/src/main.rs`](../../shells/desktop/src/main.rs).

### 10.3 Painéis flutuantes — auto-resize ao arrastar para baixo

- **Sintoma**: arrastando Inspector ou Hierarchy para baixo, o painel
  vazava o viewport e o conteúdo no rodapé ficava cortado.
- **Causa**: o clamp original em `paint_hero_screen` só ajustava x/y,
  mantendo `base.h` invariável.
- **Fix**: `clamp_panel` (em `screens/hero.rs`) auto-encolhe `h` quando
  `natural_bottom > max_bottom = viewport.bottom - 8`. Floor de
  `MIN_H = 120`. Quando o usuário arrasta de volta pra cima, h volta
  ao `base.h`. O clamp também é escrito de volta no
  `blender_picker_offset` para que próximos drag-begins capturem o
  valor visível.
- **Código**: [`screens/hero.rs paint_hero_screen` — `clamp_panel`](../../crates/ph2d-editor/src/screens/hero.rs).

### 10.4 "Saltos discretos" no drag (rubber band)

- **Sintoma**: arrastando um painel pra fora do clamp, depois voltando,
  o usuário precisa "puxar" um trecho invisível antes do painel
  responder.
- **Causa**: modelo de drag cumulativo. O anchor guardava
  `cursor_at_down + offset_at_down`; cada Move computava
  `offset = anchor_off + (cursor_now − cursor_down)`. Sem clamp no
  store, o offset crescia além do clamp visível.
- **Fix**: modelo INCREMENTAL. O anchor passa a guardar apenas o
  `last_cursor`; cada Move computa
  `delta = cursor − last_cursor`, aplica em `cur_off` (que já está
  clamped pelo paint anterior), e re-ancora. Reverso de direção
  move imediatamente. Novo método
  `WidgetStore::update_blender_drag_cursor`.
- **Código**: [`interaction/dispatch.rs PointerKind::Move`](../../crates/ph2d-editor/src/interaction/dispatch.rs);
  [`interaction/state.rs update_blender_drag_cursor`](../../crates/ph2d-editor/src/interaction/state.rs).

### 10.5 Resize handles manuais nos painéis

- **Sintoma**: usuário só conseguia mover painéis; faltava redimensionar.
- **Solução**: novo `BlenderHitKind::ResizeHandle` + side-tables
  `panel_resize_delta: BTreeMap<NodeId, (f32, f32)>` e
  `panel_resize_anchor: Option<(NodeId, f32, f32)>`. Down em hit zone
  16×16 no canto inferior-direito chama `begin_panel_resize`; Move
  aplica incremental delta (mesma lição do §10.4); Up chama
  `end_panel_resize`. `clamp_panel` em `paint_hero_screen` consome
  o delta antes do auto-resize. Constraints:
  `MIN_W = 220`, `MIN_H = 120`, `max_w = viewport.w * 0.7`.
- **Código**: [`interaction/state.rs panel_resize_*`](../../crates/ph2d-editor/src/interaction/state.rs);
  [`screens/hero.rs clamp_panel`](../../crates/ph2d-editor/src/screens/hero.rs).

### 10.6 Hierarchy DnD — drop-inside (parenting) + indentação

- **Sintoma**: drag de uma entity em cima de outra só reordenava;
  faltava criar relações pai-filho.
- **Solução**: side-table `hierarchy_parent: BTreeMap<NodeId, NodeId>`.
  Cycle detection em `hierarchy_set_parent` (self-parent ou
  descendente como ancestor → rejeitado). `hierarchy_depth_of`
  com cap=32 (defense in depth). O dispatch resolve o drop em 3
  bandas verticais (30/40/30) via `enum HierDrop {Before, Inside, End}`:
  top=sibling-acima, middle=child, bottom=cai pra próxima row. Painter
  indenta cada row por `depth × 16px`; indicator mostra linha 2px no
  topo (sibling) OU stroke de Accent ao redor da row (inside).
- **Código**: [`interaction/dispatch.rs find_hierarchy_drop + HierDrop`](../../crates/ph2d-editor/src/interaction/dispatch.rs);
  [`interaction/state.rs hierarchy_set_parent + hierarchy_depth_of`](../../crates/ph2d-editor/src/interaction/state.rs);
  [`screens/hero/hierarchy.rs paint_hierarchy`](../../crates/ph2d-editor/src/screens/hero/hierarchy.rs).

### 10.7 DnD ignorando cursor X → grandchild acidental

- **Sintoma**: usuário queria fazer X virar filho de root Y, mas o
  TAB de indentação ficou tão grande que parecia neto.
- **Causa**: `find_hierarchy_drop` testava só `cursor_y` vs row rect.
  Cursor à esquerda de uma row já indentada (depth ≥ 1) ainda casava
  como `Inside(esa row)` se o y caísse na faixa — `dragged` virava
  child da indentada (depth+1) em vez de sibling no root.
- **Fix**: cursor_x deve estar dentro do `[rect.x, rect.x + rect.w]`
  para `Inside`/`Before`; se y bate mas x está à esquerda do indent,
  retorna `Before(row)` com a row's parent como new parent
  (sibling-at-root quando a row é root). `HierarchyDragState` ganhou
  `cursor_x` (atualizado no Move) pra que o indicator do painter
  espelhe exatamente o resolve do dispatch.
- **Lição**: drop targets em listas com indentação SEMPRE precisam
  considerar X. Sem isso, drops ambíguos viram bug de profundidade.

### 10.8 radius_scale finalmente consumido

- **Sintoma**: menu de tema oferece Sharp / Default / Round mas a
  escolha não mudava nada visualmente.
- **Causa**: `WidgetStore::radius_scale` armazenava o valor mas
  nenhum painter consultava. Threadar via cada painter (200+ call
  sites) seria caro.
- **Fix**: thread-local em [`paint.rs`](../../crates/ph2d-editor/src/paint.rs);
  `fill_rounded_rect` / `stroke_rounded_rect` multiplicam por ele
  internamente. `paint_hero_screen` chama `set_radius_scale` no
  início de cada frame. `Radius::Full` (999) é PRESERVADO exato
  para que pills/circles continuem perfeitos mesmo com scale < 1.

### 10.9 Cmd+C / Cmd+V / Cmd+X clipboard

- **Solução**: side-tables `pending_clipboard_copy: Option<String>` e
  `pending_clipboard_paste: Option<NodeId>` no store. Dispatch:
  Cmd+C extrai a seleção do focused text widget e empurra; Cmd+X
  igual + delete + emit TextChanged; Cmd+V grava request com o id
  focado. Shell drena via `take_clipboard_*` após handle_key e
  bridgeia ao `arboard::Clipboard` (uma instância no `AppGfx`).
  Função pública `apply_clipboard_paste(store, id, text)` insere no
  caret (replacing selection); filtra non-numeric chars para
  `NumberInput` para que paste não desync o buffer.
- **Dep**: `arboard = "3"` em `shells/desktop`. As transitive deps
  `clipboard-win` e `error-code` são BSL-1.0 — `deny.toml` ganha
  per-crate exceptions explícitas (preferido a global allow).

### 10.10 Note painters não alinhavam com o contrato do dispatch

- **Sintoma**: drag-to-select em notas selecionava bytes errados;
  duplo-clique em multi-line não pintava seleção alguma.
- **Causa**: `paint_note_editable_line` / `_multiline` desenhavam
  texto a partir de `rect.x` / `rect.y`. Mas
  `byte_offset_from_click_xy` no dispatch assume convenção
  `TextInput`: `text_start_x = rect.x + 12`,
  `text_start_y = rect.y + 8` (multi-line). Painter e dispatch viam
  origens diferentes → caret/seleção em byte errado.
- **Fix**: novas constantes `NOTE_TEXT_PAD_X = 12`,
  `NOTE_TEXT_PAD_Y = 8` em `inspector.rs`. Painters realinhados.
  Multi-line agora pinta seleção (uma caixa por linha visível).
  Altura do body recalculada para acomodar o inset top.
- **Lição**: hard rule — qualquer painter de campo editável DEVE
  usar a mesma origem que `byte_offset_from_click_xy`. Se não estiver
  no contrato padrão, ou alinha visual com o contrato OU publica
  origem em side-table.

### 10.11 Duplo-clique "select all" encolhe na soltura

- **Sintoma**: DC seleciona tudo, depois deseleciona a parte à direita
  do cursor.
- **Causa**: depois de `select_all_in_text_widget` (anchor=0,
  caret=len), o Move event do mouse-jitter na soltura re-entrava no
  branch text drag-to-select e setava `caret = clicked_byte` com
  anchor=0 → seleção shrunk para 0..clicked_byte.
- **Fix**: depois do DC, `store.set_active_rect(None)`. Move skipa
  o block porque `if let Some(rect) = store.active_rect()` falha.
  Up ainda usa `active_id` pra cleanup. **Guardado a text widgets**
  apenas — limpar pra qualquer widget quebrou teste de Dropdown
  toggle (DC fechava em vez de toggle).

### 10.12 Scroll bloqueando drag handle

- **Sintoma**: depois de rolar o Inspector, a pílula de drag no topo
  não responde mais; só de voltar o scroll ao começo o drag volta.
- **Causa**: `HitIndex::hit()` faz reverse-walk (last-registered
  vence). A drag pill era registrada PRIMEIRO; depois os widgets do
  body. Com scroll, a primeira row do body subia e sua hit zone caía
  na faixa y da drag pill — sombreando-a.
- **Fix**: re-registrar drag handle + resize handle DEPOIS do
  `pop_layer` do body, em ambos Inspector e Hierarchy. Chrome sempre
  vence body.
- **Lição**: hit zones de chrome (drag, resize, close buttons) DEVEM
  ser as ÚLTIMAS registradas no `HitIndex` do seu container. Senão
  qualquer scroll/redimensionamento pode sombrear.

### 10.13 Dot do canto coberto por widget do body

- **Sintoma**: o accent dot no canto inferior-direito aparece em
  Hierarchy (sem widget no canto) mas some em Inspector (ListItem /
  TreeView na última posição).
- **Causa**: `paint_panel_surface` pintava o dot ANTES do body. Body
  desenha por cima dele.
- **Fix**: split em duas funções em `style.rs` —
  `paint_panel_surface` (rounded fill + border + drag pill,
  chamado ANTES) e `paint_panel_corner_dot` (só o dot, chamado
  DEPOIS do `pop_layer`). Ambos os painters de painel consomem as
  duas, na ordem certa.
- **Lição**: chrome com z-order acima do conteúdo NUNCA pode ser
  pintado antes do conteúdo num single-pass painter. Split.

### 10.14 Centralização de chrome de painel

- **Solução**: helpers canônicos em `screens/hero/style.rs`:
  - `paint_panel_surface(rect, scene, theme)` — base chrome
  - `paint_panel_corner_dot(rect, scene, theme)` — corner dot
  - `panel_drag_handle_rect(panel) -> Rect` — top center 80×14
  - `panel_resize_handle_rect(panel) -> Rect` — bottom-right 16×16
  - `const PANEL_RESIZE_HANDLE_SIZE: f32 = 16.0`
- Qualquer painel novo só precisa: 1) chamar `paint_panel_surface`,
  2) registrar suas hit zones com os 2 helpers, 3) chamar
  `paint_panel_corner_dot` depois do conteúdo. Geometria, cor e
  tamanho são iguais por construção.

### 10.15 Workaround: drift de bindgen no Windows CI (CRLF/LF)

- **Sintoma**: `ph2d-bindgen --check` falhou só no Windows CI; os
  outros runners aprovavam. `committed != generated` em
  `runtime/luau/ph2d.d.luau` e `runtime/mcp/schema.json`.
- **Causa**: sem `.gitattributes` no projeto, o git checkout no
  Windows aplica `core.autocrlf=true` reescrevendo LF → CRLF.
  Bindgen gera LF puro. `read_to_string() != fresh_string` byte-wise
  mesmo com conteúdo semanticamente idêntico.
- **Fix**: `drift_detected` normaliza `\r\n` → `\n` em ambos os
  lados antes de comparar. HR-10 (paridade gerado↔commitado) é
  semântica, não byte-level.
- **Código**: [`tools/ph2d-bindgen/src/main.rs drift_detected`](../../tools/ph2d-bindgen/src/main.rs).

### 10.16 Workaround: comentários PT-BR em .rs flagrados pelo typos

- **Sintoma**: CI typos detectou `componente`, `classe`, `limite` em
  .rs files como typos en-US (`componente` → `components` etc.).
- **Causa**: SKILL §10.1 manda comentários em inglês; `.typos.toml`
  exclui markdown PT-BR mas não `.rs`.
- **Fix**: reescrever os fragmentos PT-BR em inglês preservando a
  attribution do feedback do usuário. NÃO adicionar `.rs` ao excludes
  do typos — o problema é o comentário PT-BR em lugar errado, não a
  ferramenta.

### 10.17 Widget Gallery embutida (substitui o snapshot aposentado em 2026-05-14)

- **Histórico**: até 2026-05-13 existia um snapshot `screens/hero_ref/`
  (cópia verbatim de `screens/hero/`) acessível via
  `reference.command` + feature `reference-snapshot`. O snapshot
  drifted, o duplo binário causou confusão, e o painel
  Inspector / Hierarchy / TopBar do snapshot ficaram desatualizados.
- **Substituto**: o showcase de 10 seções vive agora como painel
  flutuante **Widget Gallery** dentro do app principal — paint via
  [`screens::hero::inspector::paint_showcase_body`], toggle pela
  paleta no TopBar (`ids::TOPBAR_WIDGET_GALLERY`). Drag, resize,
  scroll, sticky notes e section outline funcionam dentro do app
  real, sem rebuild de feature.
- **Launcher único**: `./play.command`. O `reference.command`,
  `scripts/run-shell-reference.sh`, a feature `reference-snapshot`,
  `screens/hero_ref/` e `screens/hero_ref.rs` foram removidos.
  Histórico preservado via
  `git log -- crates/ph2d-editor/src/screens/hero_ref/`.
