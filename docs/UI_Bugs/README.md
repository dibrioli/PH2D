# UI bugs — lições aprendidas

Padrões que quebraram a UI do PH2D durante o sprint do BlenderColorPicker
(M13). Cada item: **sintoma** / **causa** / **fix** / **onde olhar no
código**. Se for tocar num painter ou no dispatch, releia esta lista
antes — todos os erros aqui já voltaram a aparecer pelo menos uma vez
durante o desenvolvimento.

---

## 1. Hit-testing

### 1.1 Container "engole" os filhos clicáveis


**Gate:** gate-deferred: documented incident; no class-level gate identified yet
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


**Gate:** gate-deferred: documented incident; no class-level gate identified yet
- **Sintoma**: drag do channel slider mapeia errado (offset perto da
  borda); chip numérico não recebe foco.
- **Causa**: registrava o **row inteiro** (label + track + chip) como
  hit do slider. A normalização `(px - rect.x) / rect.w` ficava
  enviesada e o chip ficava sem rect próprio.
- **Fix**: registre **só o track** como hit do slider; chip tem seu
  próprio NodeId + rect. `paint_slider_with_chip` faz isso por padrão.
- **Código**: [`widget/slider_with_chip.rs`](../../crates/ph2d-editor/src/widget/slider_with_chip.rs).

### 1.3 Slots estáticos vs dados dinâmicos


**Gate:** gate-deferred: documented incident; no class-level gate identified yet
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


**Gate:** gate-deferred: documented incident; no class-level gate identified yet
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


**Gate:** gate-deferred: documented incident; no class-level gate identified yet
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


**Gate:** gate-deferred: documented incident; no class-level gate identified yet
- **Sintoma**: right-click em swatch da paleta não deletava.
- **Causa**: `PointerEvent` não carregava o botão (host descartava
  o `winit::MouseButton`).
- **Fix**: `PointerButton::{Primary, Secondary, Middle}` no
  `ph2d-host`; shell mapeia winit; dispatch lê `event.button` e
  ramifica (PaletteSwatch + Secondary → remove).
- **Código**: [`crates/ph2d-host/src/events.rs`
  `PointerButton`](../../crates/ph2d-host/src/events.rs).

### 2.4 Item de menu de contexto não dispara — faltou registrar no populate


**Gate:** `simple_row_context_menu_items_are_populate_registered`
([`ph2d-editor-core/src/screens/hero/tests.rs`](../../crates/ph2d-editor-core/src/screens/hero/tests.rs))
— afirma que toda linha de menu chrome-driven tem entrada no populate.
- **Sintoma**: o menu de contexto abre, mas clicar num item **não faz nada** —
  nenhum evento gerado (o handle Vector/Auto do falloff do Painter custou várias
  horas, 2026-06-21). Logs decisivos: no Down do item `dispatch produced 0
  events`, no Up `dispatch produced 0 events` (Click nunca emitido).
- **Causa**: a linha do menu estava só em `hit_index.register` (no paint do
  overlay), mas **faltava a entrada `Plain`** em
  `pre_populate.rs::populate_global_context_menu`. Sem ela a linha não é
  `is_focusable` → o **Down nunca arma `active`/`active_rect`** → o Up não emite
  `Click` → o handler do chrome nunca roda. Toda linha de menu simples precisa
  dessa entrada; as irmãs `CTX_MENU_POINT_TYPE_*` (VectorPointType) já estavam lá.
  É a mesma classe das pegadinhas de populate (§10 "Esquecido no populate").
- **Red herring (NÃO repita)**: parece "o menu fecha no Down + o repaint contínuo
  do Painter des-registra o item antes do Up". **Não é.** O Up resolve o widget
  ativo pelo **snapshot `active_rect`** tirado no Down (`pointer_up.rs` ~209-217),
  não pelo hit-index vivo — por isso TODO menu (hierarquia/inspector/tema)
  sobrevive a fechar-no-Down + repaint. Mexer no `pointer_down`/`pointer_up`
  global pra "consertar" quebra os menus do app inteiro (tentativa revertida em
  `d72873af`, CLAUDE.md §0.2). O fix é feature-local: registrar a linha.
- **Fix**: adicione o id da linha em `populate_global_context_menu` (1 linha por
  item). Diagnóstico: PRIMEIRO grep o id no populate antes de teorizar dispatch.
- **Código**: [`screens/hero/pre_populate.rs` `populate_global_context_menu`](../../crates/ph2d-editor-core/src/screens/hero/pre_populate.rs)
  · snapshot que mata o red herring: [`interaction/dispatch/pointer_up.rs`](../../crates/ph2d-editor-core/src/interaction/dispatch/pointer_up.rs)
  · handler de exemplo: [`screens/hero/chrome/falloff_handle.rs`](../../crates/ph2d-editor-core/src/screens/hero/chrome/falloff_handle.rs).

---

## 3. Painters & rounded chrome

### 3.1 Fill sharp dentro de stroke rounded


**Gate:** gate-deferred: documented incident; no class-level gate identified yet
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


**Gate:** gate-deferred: documented incident; no class-level gate identified yet
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


**Gate:** gate-deferred: documented incident; no class-level gate identified yet
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


**Gate:** gate-deferred: documented incident; no class-level gate identified yet
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


**Gate:** gate-deferred: documented incident; no class-level gate identified yet
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


**Gate:** gate-deferred: documented incident; no class-level gate identified yet
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


**Gate:** gate-deferred: documented incident; no class-level gate identified yet
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


**Gate:** gate-deferred: documented incident; no class-level gate identified yet
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


**Gate:** gate-deferred: documented incident; no class-level gate identified yet
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


**Gate:** crates/ph2d-editor-core/tests/arch_no_absolute_drag_pattern.rs (heuristic match — refine if wrong)
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


**Gate:** gate-deferred: documented incident; no class-level gate identified yet
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


**Gate:** gate-deferred: documented incident; no class-level gate identified yet
- **Sintoma**: Linear/Perceptual e RGB/HSV pareciam iguais
  (selected vs unselected).
- **Causa**: pill ativo usava `AccentSoft` (alpha 0.16) sobre Bg2
  → quase invisível em temas escuros.
- **Fix**: pill ativo usa `Accent` (saturado) + `AccentFg` no texto.
- **Código**: [`widget/radio_group.rs paint_segmented`](../../crates/ph2d-editor/src/widget/radio_group.rs).

### 6.3 Tabs de paleta única ("Default")


**Gate:** gate-deferred: documented incident; no class-level gate identified yet
- **Sintoma**: pílula "Default" ocupava espaço sem informação.
- **Causa**: paleta single-mode mas painter ainda renderizava o
  Tabs.
- **Fix**: skipa a faixa de tabs quando `palettes.len() == 1`.
- **Código**: [`widget/blender_color_picker/palette.rs`
  comentário "Single-palette picker: skip the palette-name tabs"](../../crates/ph2d-editor/src/widget/blender_color_picker/palette.rs).

---

## 7. Estado persistido vs ephemeral

### 7.1 Painel movível precisa de offset retido


**Gate:** gate-deferred: documented incident; no class-level gate identified yet
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


**Gate:** gate-deferred: documented incident; no class-level gate identified yet
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


**Gate:** gate-deferred: documented incident; no class-level gate identified yet
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


**Gate:** gate-deferred: documented incident; no class-level gate identified yet
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


**Gate:** gate-deferred: documented incident; no class-level gate identified yet
- **Sintoma**: valores baixos (0–10 %) exibiam stadium com caps
  iguais ao trilho. Visualmente o fill parecia maior que o valor.
- **Fix**: `fill_radius = radius.min(fill_w * 0.5)`. Quando o fill
  é mais estreito que o diâmetro do cap, o cap encolhe junto.
- **Código**: [`widget/progress_bar.rs`](../../crates/ph2d-editor/src/widget/progress_bar.rs).

### 9.5 Checkbox indeterminate: glifo certo


**Gate:** gate-deferred: documented incident; no class-level gate identified yet
- **Sintoma**: o estado "mixed" pintava um `+` (Plus) — lia como
  "adicionar" e não como "alguns filhos selecionados".
- **Fix**: adicionado `IconId::Minus` (path único `M5 12h14`); o
  pintor usa Minus para Indeterminate.
- **Código**: [`icons.rs IconId::Minus`](../../crates/ph2d-editor/src/icons.rs);
  [`widget/checkbox.rs paint_checkbox`](../../crates/ph2d-editor/src/widget/checkbox.rs).

### 9.6 Toggle disabled deve preservar on/off


**Gate:** gate-deferred: documented incident; no class-level gate identified yet
- **Sintoma**: `Toggle` desabilitado virava cinza tanto em on quanto
  em off — usuário não conseguia ler o valor.
- **Fix**: disabled+on → `AccentSoft`; disabled+off → `Border`. Estado
  on continua legível mesmo travado.
- **Código**: [`widget/toggle.rs paint_toggle`](../../crates/ph2d-editor/src/widget/toggle.rs).

### 9.7 Tabs hover overlay


**Gate:** gate-deferred: documented incident; no class-level gate identified yet
- **Sintoma**: tabs não-selecionadas não davam feedback sob o cursor.
- **Fix**: `paint_tabs_with_hover(hovered: Option<usize>, …)` desenha
  um `Bg2` rounded por baixo da tab quando hovered e não-selecionada.
  `paint_tabs` continua sendo o entry point estático.
- **Código**: [`widget/tabs.rs paint_tabs_with_hover`](../../crates/ph2d-editor/src/widget/tabs.rs).

### 9.8 Tooltip registry genérico


**Gate:** gate-deferred: documented incident; no class-level gate identified yet
- **Sintoma**: tooltips estavam hardcoded em `tooltip_for(id)` num
  match enorme; cada widget novo exigia editar topbar.rs.
- **Fix**: `WidgetStore::set_tooltip(id, text)` + `tooltip_for(id)`
  como side-table. `paint_hover_tooltip` lê do store. Qualquer
  painter pode registrar tooltip via uma linha.
- **Código**: [`interaction/state.rs tooltips`](../../crates/ph2d-editor/src/interaction/state.rs);
  [`screens/hero/topbar.rs paint_hover_tooltip`](../../crates/ph2d-editor/src/screens/hero/topbar.rs).

### 9.9 TextArea multi-line caret/selection


**Gate:** gate-deferred: documented incident; no class-level gate identified yet
- **Sintoma**: `TextArea` não mostrava caret nem seleção quando
  focado — invisível pro usuário.
- **Fix**: `paint_text_area_with_state(caret, selection_anchor, …)`
  divide `value` por `\n`, mede prefixo da linha do caret com
  `text_system.layout`, e desenha caret + AccentSoft por linha. Seleção
  multi-linha estende até a margem direita nas linhas intermediárias
  (convenção de editor).
- **Código**: [`widget/text_area.rs paint_text_area_with_state`](../../crates/ph2d-editor/src/widget/text_area.rs).

### 9.10 Click→byte multi-line aware (TextArea) e snap por linha


**Gate:** gate-deferred: documented incident; no class-level gate identified yet
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


**Gate:** gate-deferred: documented incident; no class-level gate identified yet
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


**Gate:** gate-deferred: documented incident; no class-level gate identified yet
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


**Gate:** gate-deferred: documented incident; no class-level gate identified yet
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


**Gate:** gate-deferred: documented incident; no class-level gate identified yet
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


**Gate:** gate-deferred: documented incident; no class-level gate identified yet
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


**Gate:** gate-deferred: documented incident; no class-level gate identified yet
- **Sintoma**: as setinhas ▲▼ no chip do NumberInput eram
  decorativas — clicar não fazia nada.
- **Fix**: `apply_number_stepper_if_hit` no Down handler detecta
  click dentro de `NumberInput::up_rect/down_rect` (geometria do
  próprio widget) e aplica +/- step. Step heurístico: `0.01`
  quando buffer contém `.` (fractional), `1.0` caso contrário.
  Espelha pro slider linkado se houver. Emite `ValueChanged`.
- **Código**: [`interaction/dispatch.rs apply_number_stepper_if_hit`](../../crates/ph2d-editor/src/interaction/dispatch.rs).

### 9.19 Glifos macOS Cmd/Return → tofu


**Gate:** crates/ph2d-editor-core/tests/no_tofu_glyphs.rs (heuristic match — refine if wrong)
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
- **RECORRÊNCIA (2026-05-21, commit `b62e0c5`)**: as toasts de troca de
  ferramenta usavam **U+2192 (→)** ("Tool → Padding") → tofu na tela.
  Mesma causa, mesmo fix (→ virou `·`). **Regra geral, vale pra QUALQUER
  string de UI** (toast, tooltip, label, pill): só ASCII + os poucos
  não-ASCII comprovadamente in-font (`·`). Setas/símbolos (→ ⌘ ↵ ✕ ▸ …)
  são tofu garantido. Code: `shells/desktop/src/{input_handlers,
  input_dispatch}.rs`, `shells/desktop/src/render_loop/mod.rs`.

### 9.20 Inspector scroll clamp + separador colorido por seção


**Gate:** crates/ph2d-editor-core/tests/arch_safe_clamp_only.rs (heuristic match — refine if wrong)
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


**Gate:** gate-deferred: documented incident; no class-level gate identified yet
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


**Gate:** gate-deferred: documented incident; no class-level gate identified yet
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


**Gate:** gate-deferred: documented incident; no class-level gate identified yet
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


**Gate:** gate-deferred: documented incident; no class-level gate identified yet
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


**Gate:** crates/ph2d-editor-core/tests/arch_no_absolute_drag_pattern.rs (heuristic match — refine if wrong)
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


**Gate:** crates/ph2d-editor-core/tests/arch_no_absolute_drag_pattern.rs (heuristic match — refine if wrong)
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


**Gate:** gate-deferred: documented incident; no class-level gate identified yet
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


**Gate:** gate-deferred: documented incident; no class-level gate identified yet
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


**Gate:** gate-deferred: documented incident; no class-level gate identified yet
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


**Gate:** gate-deferred: documented incident; no class-level gate identified yet
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


**Gate:** gate-deferred: documented incident; no class-level gate identified yet
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


**Gate:** gate-deferred: documented incident; no class-level gate identified yet
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


**Gate:** gate-deferred: documented incident; no class-level gate identified yet
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


**Gate:** crates/ph2d-editor-core/tests/arch_no_absolute_drag_pattern.rs (heuristic match — refine if wrong)
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


**Gate:** gate-deferred: documented incident; no class-level gate identified yet
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


**Gate:** gate-deferred: documented incident; no class-level gate identified yet
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


**Gate:** gate-deferred: documented incident; no class-level gate identified yet
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


**Gate:** gate-deferred: documented incident; no class-level gate identified yet
- **Sintoma**: CI typos detectou `componente`, `classe`, `limite` em
  .rs files como typos en-US (`componente` → `components` etc.).
- **Causa**: SKILL §10.1 manda comentários em inglês; `.typos.toml`
  exclui markdown PT-BR mas não `.rs`.
- **Fix**: reescrever os fragmentos PT-BR em inglês preservando a
  attribution do feedback do usuário. NÃO adicionar `.rs` ao excludes
  do typos — o problema é o comentário PT-BR em lugar errado, não a
  ferramenta.

### 10.17 Widget Gallery embutida (substitui o snapshot aposentado em 2026-05-14)


**Gate:** gate-deferred: documented incident; no class-level gate identified yet
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

---

## 11. Slot 1 — Color Equalization (2026-05-23)

Quatro bugs em sequência ao fechar o end-to-end do primeiro tool das
4 novas Image Tools (Color Equalization, Equalize Sizes, Rasterize,
Upscale). Cada um aponta para o MESMO erro arquitetural maior:
o painel novo divergiu sutilmente do setup canônico do Widget
Gallery, e cada divergência expôs uma assunção implícita do
dispatch/painter. Lição estrutural codificada em
[`DIRETRIZ.md §4.2`](../IntegracaoMultiAgente/DIRETRIZ.md#42-widget-gallery-é-a-fonte-de-verdade--copie-não-reinvente)
+ arch-gate
[`architecture_panel_chip_pill_no_stepper`](../../crates/ph2d-editor-core/tests/architecture_panel_chip_pill_no_stepper.rs).

### 11.1 Preview ausente no canvas — slider muda valor, canvas não atualiza


**Gate:** gate-deferred: documented incident; no class-level gate identified yet
- **Sintoma**: usuário arrasta os sliders Brightness / Contrast /
  Saturation no painel CEQ, valores numéricos mudam, mas a sprite no
  canvas fica congelada. Só ao clicar Apply o efeito aparece — UX
  inaceitável para uma game engine que é tempo-real-tudo.
- **Causa**: shell tinha `color_equalization_bridge.rs` mas sem o
  overlay-por-frame. O tool já expunha `take_params_dirty()` +
  `preview_rgba()`; a bridge não chamava nenhum dos dois. Comparar
  com `shells/desktop/src/render_loop/bgremoval_preview.rs` (que
  ESTAVA correto desde o sprint anterior) revelou o gap.
- **Fix**: adicionar `ColorEqualizationPreview { entity_bits, rgba:
  Arc<Vec<u8>>, width, height }` em `app_state.rs`; bridge refaz o
  cache quando `tool.take_params_dirty()`, pinta via
  `vector_scene.draw_image_rgba` no footprint world-space da sprite,
  zera em Apply / deactivate / selection-change.
- **Lição estrutural** (DIRETRIZ §4.2.2): todo painel novo que altera
  pixels TEM que vir com `render_loop/<slug>_bridge.rs` espelhando
  `bgremoval_preview.rs`. **Coordenador bounce se faltar.**
- **Código**: commit `903d63c`
  ([`shells/desktop/src/render_loop/color_equalization_bridge.rs`](../../shells/desktop/src/render_loop/color_equalization_bridge.rs)).

### 11.2 Chip drag absoluto-de-Down — clamp prega o valor no extremo


**Gate:** crates/ph2d-editor-core/tests/arch_no_absolute_drag_pattern.rs (heuristic match — refine if wrong)
- **Sintoma**: drag horizontal no chip funciona até bater na borda
  do range, mas reverter o cursor não move o valor de volta. Usuário
  arrasta de volta 50 px e "nada acontece" — o chip continua pregado
  no max. Para destravar o usuário precisa voltar o cursor até a
  posição de Down original.
- **Causa**: `NumberInputDragState` calculava `dx_total = event.x -
  start_x` (delta absoluto desde Down). Quando o valor batia o clamp
  (linked-slider 0..1 ou bounds explícitos), o `dx_total` continuava
  acumulando overshoot. Reverter o cursor só "consumia o overshoot"
  até voltar a um `dx_total` dentro do range válido.
- **Fix**: trocar para modelo incremental (Blender / After Effects):
  `step_dx = event - last_x`, aplicar ao value ATUAL, avançar
  `last_x` por Move via `advance_number_input_drag_anchor`. Reversão
  produz `step_dx` negativo na PRÓXIMA Move e o value reverte
  imediatamente.
- **Lição estrutural** (DIRETRIZ §4.2.4): scrub gestures
  (chip, sliders) usam SEMPRE delta incremental. `event.x - start_x`
  no Move handler é regressão — pegue em revisão.
- **Código**: commit `7b5f7c1`
  ([`crates/ph2d-editor-core/src/interaction/dispatch/pointer.rs`](../../crates/ph2d-editor-core/src/interaction/dispatch/pointer.rs)
  + [`drag.rs`](../../crates/ph2d-editor-core/src/interaction/drag.rs)).

### 11.3 Phantom stepper — valor sobe sozinho com mouse parado


**Gate:** gate-deferred: documented incident; no class-level gate identified yet
- **Sintoma**: usuário clica no chip CEQ, segura o botão sem mover o
  mouse, e o valor numérico sobe a cada 30 ms. Reportado como "os
  valores continuam subindo mesmo se mouse parado!".
- **Causa**: `apply_number_stepper_if_hit` recorta uma coluna de
  16-22 px no lado direito do rect de TODO `InteractiveState::NumberInput`
  como hit-zone de stepper arrow. Pra `paint_number_input_with_buffer`
  (forma boxed do Inspector — setinhas visíveis) o comportamento
  está correto. Pra `paint_number_chip` (forma pill da CEQ /
  Padding / etc. — sem arrows visíveis) a hit-zone é INVISÍVEL:
  click no lado direito arma `number_stepper_hold`, e
  `dispatch_tick` repete o increment a cada 30 ms (após 250 ms de
  delay inicial) enquanto o cursor fica parado.
- **Fix**: side-table `chips_without_steppers: BTreeSet<NodeId>` no
  `WidgetStore` + API `mark_chip_no_stepper(id)`. Dispatch pula o
  stepper hit-test pra ids marcados. `link_slider_number(slider,
  chip)` AUTO-MARCA o chip (todo caller no codebase é pill chip
  pareado).
- **Reincidência**: 4 painéis tinham o mesmo bug latente
  (`padding`, `upscale`, `equalize-sizes` + CEQ). Padding e Upscale
  já estavam em produção desde 2026-05-21 — usuário nunca clicava
  exatamente nos 16-22 px do canto direito, então o bug ficou
  dormente. Gate
  [`architecture_panel_chip_pill_no_stepper`](../../crates/ph2d-editor-core/tests/architecture_panel_chip_pill_no_stepper.rs)
  flushou os 4 ao mesmo tempo.
- **Lição estrutural** (DIRETRIZ §4.2.3): a infra de input
  ASSUMIA que toda NumberInput tem stepper arrows. Quando o painter
  varia (pill vs boxed), o estado interativo TEM que carregar a
  diferença — não dá pra deixar pro dispatch inferir.
- **Código**: commit `2f58b73`
  ([`crates/ph2d-editor-core/src/interaction/state/mod.rs`](../../crates/ph2d-editor-core/src/interaction/state/mod.rs)
  + [`dispatch/pointer.rs`](../../crates/ph2d-editor-core/src/interaction/dispatch/pointer.rs)).

### 11.4 Painel divergiu do Widget Gallery (causa raiz das três acima)


**Gate:** gate-deferred: documented incident; no class-level gate identified yet
- **Sintoma**: 11.1, 11.2 e 11.3 são manifestações distintas do
  mesmo erro arquitetural — o painel CEQ foi populado a partir do
  zero em vez de copiar o setup canônico `INSP_SAMPLE_SLIDER` +
  `INSP_SAMPLE_SLIDER_CHIP` do
  [`pre_populate.rs:212-231`](../../crates/ph2d-editor-core/src/screens/hero/pre_populate.rs).
  Cada sutileza esquecida virou um bug:

  | Esquecido no populate | Bug que aparece |
  |-----------------------|-----------------|
  | `link_slider_number(slider, chip)` | mirror manual em event.rs dessincroniza; chip/slider andam descolados |
  | bridge `take_params_dirty()` + `draw_image_rgba` | canvas congelado, só Apply aplica (11.1) |
  | storage chip em `0..1` (não unidade natural) | drag rate fica fora de escala; mistura de spaces no apply_event |
  | `mark_chip_no_stepper` (auto via `link_slider_number`) | phantom stepper na coluna direita do chip (11.3) |

- **Fix**: reescrever populate copiando linha por linha o Speed
  setup; `event.rs` virou forwarder thin (sem mirror manual);
  bridge ganhou overlay; arch-gate
  `architecture_panel_chip_pill_no_stepper` enforça pra futuro.
- **Lição estrutural** (DIRETRIZ §4.2): Widget Gallery é a ÚNICA
  fonte de verdade. **Painel novo COPIA literalmente — não
  reinventa.** Coordenador checklist obrigatório antes de mergear
  painel: link, mark (auto via link), storage `0..1`, bridge
  per-frame, `apply_event` forwarder. Faltou alguma → bounce.
- **Código**: commit `3bf8806`
  ([`crates/ph2d-panel-color-equalization/src/populate.rs`](../../crates/ph2d-panel-color-equalization/src/populate.rs)
  + [`event.rs`](../../crates/ph2d-panel-color-equalization/src/event.rs))
  + commit `fd37ca8` (DIRETRIZ §4.2 + arch-gate + 3 painéis latentes).

### 11.5 `apply_number_stepper_if_hit` assume painter (gap arquitetural latente)


**Gate:** gate-deferred: documented incident; no class-level gate identified yet
- **Diagnóstico residual**: o dispatch tem várias suposições
  silenciosas sobre o painter que NÃO são gateadas. A
  "todo NumberInput tem stepper arrows" foi a que mordeu em §11.3;
  outras provavelmente existem (axis-lock direction em chip drag,
  buffer mutation while focused, etc.). Cada uma vai pra mordida
  ⇒ side-table que diferencia o caso painter-específico do default.
- **Lição estrutural**: quando uma helper assume "o painter X
  desenha Y", documente a assunção no helper E adicione gate. Se a
  assunção é difícil de gatear (dispatch que olha rect, não AST),
  ofereça um opt-out explícito (`mark_*`) que painters não-padrão
  chamam no populate.
- **Próximo passo**: ao adicionar widget novo paint-only-variant
  (analog ao `paint_number_chip` vs `paint_number_input_with_buffer`),
  audit dispatch pra descobrir todas as zonas que carve sub-rect
  do widget e listar em side-table ou cap.
- **Código**: princípio aplicado em
  [`mark_chip_no_stepper`](../../crates/ph2d-editor-core/src/interaction/state/mod.rs)
  — model template pra futuras opt-outs paint-variant.

---

## 12. Color Equalization Phase 2+ + 4 sister image tools (2026-05-23+)

Bugs do polish e fan-out das image-tools depois do slot 1 fechar.
§11 cobriu o setup base (preview overlay, drag delta, stepper opt-out,
populate divergence). Esta seção cobre o que apareceu DEPOIS, ao
expandir CEQ (Phase 1→5: tonal+LUT+sharpen+denoise+posterize+quantize)
e replicar pros 4 image-tools restantes (Padding, BgRemoval, Upscale,
EqualizeSizes).

### 12.1 Event-router incompleto — controles novos silenciosamente mortos


**Gate:** gate-deferred: documented incident; no class-level gate identified yet
- **Sintoma**: usuário arrasta sliders de Exposure / Vibrance /
  Saturation / Sharpen / Denoise / LUT no painel CEQ — knob mexe, mas
  o pipeline não roda. Mesma coisa pros 47 options dos dropdowns LUT/
  Posterize/Quantize: clica, popover fecha, mas a opção não aplica.
- **Causa**: `panel/event.rs::apply_event` é o chokepoint que
  traduz `WidgetEvent::ValueChanged/Click` → `EditorAction::ToolPanelEvent`.
  O CEQ era um `slider_for_widget` com hardcoded 5 ids (Clip / Tile /
  Brightness / Contrast / Saturation) — Phase 1+2+3+5 adicionou 9
  sliders + 47 option ids + 4 botões mas ninguém atualizou a tabela.
  Tudo que faltava caía no `_ => false` → tool nunca via o evento.
- **Fix**: tabela `pairs: &[(slider_id, chip_id)]` cobrindo TODOS os 14
  slider+chip pairs registrados em populate; `is_dropdown_option(id)`
  varrendo `CEQ_LUT_{1,2}_OPTS` + `CEQ_POSTERIZE_OPTS` +
  `CEQ_QUANTIZE_OPTS`; `FORWARD_CLICK_IDS: &[NodeId]` lista todos os
  botões que viram `PanelEvent::Click`. Doc-comment do módulo diz
  explícito: "add a new control here whenever you wire one in
  populate".
- **Lição estrutural**: `populate.rs` registra widgets no store + paint
  pinta + dispatch infere clicks — mas a tradução pro tool é manual.
  Adicionar slider/chip/dropdown novo SEM cobrir o router é deixar
  um controle morto, sem warning. Considerar arch-test que liste
  todos NodeIds registrados em populate e checke se cada um tem arm
  no router OU está numa allowlist explícita.
- **Código**: [`crates/ph2d-panel-color-equalization/src/event.rs`](../../crates/ph2d-panel-color-equalization/src/event.rs).

### 12.2 Dropdown option NodeIds DEVEM ser registrados como Button


**Gate:** gate-deferred: documented incident; no class-level gate identified yet
- **Sintoma**: dropdowns abrem (chip click toggla `open`), popover
  pinta opções, usuário clica numa opção — NADA acontece. Sem evento.
- **Causa**: `dispatch::pointer::Down` só seta `active`/`active_rect`
  para ids que passam por `is_focusable(store, id)`. Esse helper
  retorna `false` quando `store.get(id) == None`. Os 47 option
  NodeIds eram só hit-registered (paint cria os rects no
  `hit_index_mut`) mas nunca registrados como InteractiveState. Down
  acertava o rect, `is_focusable` falhava, Up não chamava
  `apply_click`, nenhum `Click(option_id)` event saía. O Inspector
  showcase tinha o mesmo gap mas como ninguém clica em "sample
  dropdown options" em produção, o bug ficou dormente.
- **Fix**: `populate` registra cada option id como
  `InteractiveState::Button { state: Normal }`. O paint continua
  igual. Down agora passa is_focusable → Up chama apply_click → Click
  event fluiu pro tool.
- **Lição estrutural**: o pattern "option NodeIds são só visuais,
  não precisa store entry" é FALSO. Toda hit-registered NodeId que
  precisa de Click event tem que estar no WidgetStore. Compõe com
  §11.3 (chip pill vs boxed): há várias suposições silenciosas no
  dispatch sobre o WidgetStore — quando o painter desvia, o estado
  interativo TAMBÉM precisa desviar.
- **Código**: [`crates/ph2d-panel-color-equalization/src/populate.rs`
  — bloco `Dropdown option ids MUST be registered too`](../../crates/ph2d-panel-color-equalization/src/populate.rs).

### 12.3 `on_activate` direto-em-params bypassa pending_panel_reset


**Gate:** gate-deferred: documented incident; no class-level gate identified yet
- **Sintoma**: usuário mexe sliders, fecha o painel (Cancel/Apply),
  reabre — sliders ficam visualmente PRESOS na posição anterior, mas
  o pipeline roda contra defaults (chips mostram defaults). Desync
  brutal entre slider knob e chip number.
- **Causa**: o tool resetava `self.params = Default::default()`
  direto no `on_activate`. Não passava pelo `apply_ui_edit::ResetAll`,
  então a flag `pending_panel_reset` nunca era setada. A flag
  controla quando o bridge re-chama `Panel::populate(store)` —
  populate sobrescreve os valores do WidgetStore com defaults. Sem
  re-populate, o store mantém as posições do último drag e o paint
  lê dele.
- **Fix**: `on_activate` chama `self.apply_ui_edit(ResetAll)` — mesma
  path do botão Reset. Single chokepoint pra reset → flag sempre
  staged → bridge sempre repopulates → slider visual sincroniza com
  params.
- **Lição estrutural**: tool reset state em DOIS lugares (botão
  Reset + on_activate) tem que routar pela MESMA `apply_ui_edit`
  variant — qualquer atalho que assigne `self.params = default`
  direto vai esquecer flags downstream. Mesmo padrão de
  "single chokepoint para mutação" do §11.4.
- **Código**: [`crates/ph2d-tool-color-equalization/src/tool.rs`
  `on_activate`](../../crates/ph2d-tool-color-equalization/src/tool.rs).

### 12.4 Reset sem repopulate desincroniza slider visual ⟷ params


**Gate:** gate-deferred: documented incident; no class-level gate identified yet
- **Sintoma**: clicar "Reset to Defaults" zera os params (vê pela
  ausência de efeito no canvas) mas os sliders ficam nas posições do
  último drag.
- **Causa**: panel paint lê o slider track do `store.slider(id).value`,
  NÃO do snapshot dos params. `apply_ui_edit::ResetAll` mexia em
  `params` mas não tocava no store. O store mantinha o valor
  arrastado.
- **Fix**: novo padrão para os 5 image-tools — campo
  `pending_panel_reset: bool` no tool struct; `apply_ui_edit::ResetAll`
  seta `true`; bridge drena via `take_pending_panel_reset()`; quando
  true, bridge chama `Panel::populate(&mut hero.store)` via
  `panel::with_registry_opt(|reg| reg.find_by_panel_node_id(...)`.
  Populate sobrescreve cada slider/chip com o valor default (já que
  `WidgetStore::register` overwrites existing entries). Slider snap
  visual + chip texto resetam juntos com os params.
- **Lição estrutural**: panel state vive em DOIS lugares — `params`
  (tool) + `WidgetStore` (interaction). Reset tem que tocar nos dois.
  O padrão "pending_X flag + bridge drains" generaliza pra qualquer
  refresh-on-action que precise atravessar a fronteira tool↔store.
- **Código**: 5 tool crates + 5 bridges
  ([`color_equalization_bridge.rs`](../../shells/desktop/src/render_loop/color_equalization_bridge.rs)
  é o template).

### 12.5 Renderer sort por texture_id puro → Individual sprites pulam pra frente


**Gate:** gate-deferred: documented incident; no class-level gate identified yet
- **Sintoma**: aplicar CEQ / BgRemoval / Padding / Trim / MakeSquare
  em uma sprite que estava ATRÁS de outras → ela visualmente "salta"
  pra frente, sem que a Hierarchy panel mude a ordem.
- **Causa**: `SpriteRenderer::render` fazia `scratch.sort_by_key(|i|
  i.texture_id)`. `ATLAS_TEXTURE_ID = 0` < qualquer Individual id.
  Toda sprite Atlas pinta ANTES de qualquer Individual. `commit_edited_texture`
  vira a source de Atlas → Individual, então sprite recém-editada
  pula pra trás na ordem de pintura = aparece por cima.
- **Fix**: novo campo `RenderInstance.z_order: u32` (replace do
  `_pad: u32`, mesma posição/alinhamento, sem mudança de vertex
  layout). `sim_extract` atribui counter sequencial em
  `propagate_transforms` (DFS da ChildOf tree, com roots collated por
  RootOrder — mesma ordem da Hierarchy panel). Renderer agora faz
  `sort_by_key(|i| (i.z_order, i.texture_id))` — z primeiro
  (correto), texture_id como tiebreaker (batching dentro de mesmo z
  slice, mantém run-grouping pra mesma texture quando adjacentes na
  z-order).
- **Lição estrutural**: tem 6 image-tools que chamam
  `commit_edited_texture`; o bug afetava TODOS, não só CEQ. Quando o
  renderer sort por proxy (texture_id como proxy de "z"), qualquer
  refactor que mude a quantidade de tipos texture_id (Atlas + N
  Individual) muda a ordem visual. Sort por dimensão CORRETA (z) +
  proxy como tiebreaker.
- **Código**: [`crates/ph2d-render/src/sprite.rs RenderInstance.z_order`](../../crates/ph2d-render/src/sprite.rs);
  [`crates/ph2d-render/src/renderer.rs render` sort](../../crates/ph2d-render/src/renderer.rs);
  [`shells/desktop/src/render_loop/sim_extract.rs z_counter`](../../shells/desktop/src/render_loop/sim_extract.rs).

### 12.6 Live preview mono-sprite em multi-seleção


**Gate:** gate-deferred: documented incident; no class-level gate identified yet
- **Sintoma**: multi-select 2-3 sprites com CEQ ativo, ajusta sliders
  — só a PRIMARY mostra o efeito ao vivo. Apply baka em todas mas o
  usuário não consegue prever o look antes do bake.
- **Causa**: `app_state.color_equalization_preview: Option<ColorEqualizationPreview>`
  — apenas UM preview cacheado, indexado pelo entity_bits da primary.
- **Fix**: virou `BTreeMap<u64, ColorEqualizationPreview>`. Bridge
  loop por `iter_selected()`: para cada entity, push source no tool,
  roda pipeline, cacheia. Selection-change retain só os keys vivos
  (evict pra sprites que saíram da seleção). Overlay paint itera o
  Map, desenha cada preview no footprint da sua sprite.
- **Lição estrutural**: qualquer feature stateful-per-entity (preview,
  overlay, computed state) em tool multi-select TEM que cachear por
  entity desde o dia 1. `Option<X>` vs `BTreeMap<EntityBits, X>` é
  diferença arquitetural que vira refactor não-trivial depois.
- **Código**: [`shells/desktop/src/app_state.rs color_equalization_previews`](../../shells/desktop/src/app_state.rs);
  [`color_equalization_bridge.rs` loop multi-sprite](../../shells/desktop/src/render_loop/color_equalization_bridge.rs).

### 12.7 Preview heavy precisa de debounce-on-release, não per-frame


**Gate:** gate-deferred: documented incident; no class-level gate identified yet
- **Sintoma**: arrastar slider de CEQ é LENTO — frame rate cai a
  ~5-10 fps em sprites grandes ou com Quantize ativo.
- **Causa**: bridge fazia `take_params_dirty()` a cada frame; tool
  rerunava pipeline completo (CLAHE + 7 tonal + LUT + bilateral +
  sharpen + auto-* + auto-WB + posterize + quantize). Quantize sozinho
  é 50-150ms; bilateral com strength alto é 30-80ms. Em multi-sprite
  isso é N × per_frame_cost.
- **Fix**: detectar drag ativo via `hero.store.active_id()` matching
  contra os ids de slider/chip da tool. Durante drag NÃO drena
  `params_dirty` (flag persiste). Em release (`active_id` flipa pra
  None), drena e roda pipeline pra TODAS as sprites selecionadas em
  full quality. Slider knob feel: instantâneo. Apply visual: aparece
  no release.
- **Decisão UX**: explicitamente combinado com o usuário —
  "Ao arrastar qualquer slider não muda em tempo real, ao soltar o
  slider aplica a todas com qualidade full". Trade-off: perde-se
  preview ao vivo durante drag, ganha-se responsividade do knob +
  quality full no Apply.
- **Lição estrutural**: pipeline com N estágios CPU pesados (>20ms)
  + slider drag DEVE ser debounced on-release. `take_params_dirty`
  per-frame era o pattern dos paineis baratos; o CEQ growup pra
  9-stage pipeline rompeu a suposição.
- **Código**: [`color_equalization_bridge.rs is_ceq_drag_widget`](../../shells/desktop/src/render_loop/color_equalization_bridge.rs).

### 12.8 Image-tool ativo deve esconder Inspector


**Gate:** gate-deferred: documented incident; no class-level gate identified yet
- **Sintoma**: abre CEQ/BgRemoval/Padding/Upscale/EqualizeSizes —
  Inspector continua visível por baixo, ambos tentam pintar no slot
  da direita, z-order resolve aleatório.
- **Causa**: image-tool panels usam `ctx.layout.padding` (o mesmo
  slot que Inspector). Não há mutex visual — cada panel pinta no seu
  rect. Inspector renderiza primeiro, image-tool por cima (ou
  vice-versa dependendo de z bump).
- **Fix**: cada bridge faz `hero.panel_visibility.insert("inspector",
  !active)` ao lado do próprio toggle de visibility. Tool ativo =>
  Inspector escondido. Tool deactivate (Cancel/Apply) => Inspector
  visível de volta. Como apenas 1 image-tool é ativo por vez
  (mutex de set_active), não há corrida entre eles.
- **Lição estrutural**: docks visuais (slots compartilhados) NÃO
  são auto-resolvidos pelo renderer. Painel que dock no mesmo slot
  de outro panel precisa explicitar "eu hide o outro quando ativo".
- **Código**: 5 bridges ([color_equalization_bridge.rs:52](../../shells/desktop/src/render_loop/color_equalization_bridge.rs)
  + 4 sister bridges).

### 12.9 Bridge monosprite → multi-sprite via `iter_selected()`


**Gate:** gate-deferred: documented incident; no class-level gate identified yet
- **Sintoma**: Padding e BgRemoval só editam a primary sprite mesmo
  com 2+ selecionadas via Shift-click.
- **Causa**: bridges capturavam `hero.gizmo.selection` (singular
  Option<u64>) em vez de `hero.gizmo.iter_selected().collect()`. CEQ
  + Upscale + EqualizeSizes já usavam iter_selected; Padding e
  BgRemoval ficaram pra trás na migração.
- **Fix**: Padding bridge passou a retornar `Option<(spec, recenter,
  Vec<u64>)>` e o drain itera. BgRemoval bridge pushia N
  `OneShotImageOp` actions (um por sprite) — o existing leftover loop
  já drainava cada um sequencialmente.
- **Lição estrutural**: ao adicionar feature multi-select, audit
  EVERY bridge — facil ficar 60% migrado e 40% latente.
- **Código**: [`padding_bridge.rs:44`](../../shells/desktop/src/render_loop/padding_bridge.rs);
  [`bgremoval_preview.rs:93`](../../shells/desktop/src/render_loop/bgremoval_preview.rs).

### 12.10 Dropdown popover overflow → flip ABOVE quando não cabe


**Gate:** gate-deferred: documented incident; no class-level gate identified yet
- **Sintoma**: clicar Posterize / Quantize / LUT dropdowns perto do
  rodapé do painel CEQ → popover extende abaixo do viewport, opções
  do fim ficam cortadas off-screen.
- **Causa**: `Dropdown::popover_rect(chip)` sempre coloca o popover
  BELOW chip com gap. Sem awareness de viewport bound.
- **Fix**: novo método `popover_rect_clamped(chip, viewport)`. Prefer
  below quando cabe, flip above quando não, fallback no lado com
  mais espaço + clamp de altura. Novo `paint_dropdown_popover_in_viewport`
  aceita `Option<Rect>` viewport e usa o clamped. CEQ panel passa
  `ctx.viewport` nos 4 dropdowns + `option_rect_in(chip, panel_rect,
  i)` pra registrar hits no rect REAL (flipped quando preciso).
- **Lição estrutural**: floating popovers/menus/tooltips DEVEM
  considerar viewport bounds. Sem isso, mesmo um Dropdown bem feito
  vira inutilizável em painéis altos. Future: extender pra horizontal
  flip também (popovers a partir da margem direita).
- **Código**: [`crates/ph2d-editor-core/src/widget/dropdown.rs popover_rect_clamped`](../../crates/ph2d-editor-core/src/widget/dropdown.rs).

### 12.11 GPU pipeline cache via OnceLock — 1× compile, N× dispatch


**Gate:** gate-deferred: documented incident; no class-level gate identified yet
- **Sintoma**: preview GPU teoricamente é ~5-10× mais rápido que CPU,
  mas se cada rebuild instanciar `ChainedPipelineCache::new(&gpu)`
  paga 10-20ms de pipeline compile a cada slider release.
- **Fix**: `gpu::try_preview_chain() -> Option<&'static
  ChainedPipelineCache>` cacheia o cache num `OnceLock` process-wide.
  Compile paga 1× no primeiro preview, subsequente chamadas são puro
  upload + dispatches + readback (~5-20ms a 512²). Distinct do
  per-Apply `ChainedPipelineCache::new` (que reconstrói porque Apply
  é raro e em full-res o overhead é dwarfed pelo dispatch).
- **Lição estrutural**: wgpu pipeline objects são Arc-backed mas o
  shader compile + bind-group-layout build não. Cache deve ser
  cross-call quando o uso é hot (preview-per-release).
- **Código**: [`crates/ph2d-tool-color-equalization/src/gpu/mod.rs try_preview_chain`](../../crates/ph2d-tool-color-equalization/src/gpu/mod.rs).

### 12.12 Luminância em sRGB: BT.709 em LINEAR + re-encode pra perceptual


**Gate:** crates/ph2d-editor-core/tests/arch_color_space_typed.rs (heuristic match — refine if wrong)
- **Sintoma**: 3 LUT presets (Film Noir, Sepia, Bleach Bypass) ficavam
  "menos bons que a engine de referência". A diferença era sutil — red
  ~40% mais escuro que esperado nas highlights.
- **Causa em camadas**:
  1. Port inicial usava `luma_bt709_norm = 0.2126·R + 0.7152·G +
     0.0722·B` aplicado em sRGB gamma-encoded — convenção "rec.709
     luma" comum mas tecnicamente errada.
  2. Engine TS de referência usava `0.299·R + 0.587·G + 0.114·B`
     (BT.601), também em gamma-encoded — antigo "rec.601 luma",
     herança de NTSC/PAL.
  3. AMBAS são wrong colorimetricamente. sRGB tem primárias BT.709,
     então a Y verdadeira é `0.2126·Rlinear + 0.7152·Glinear +
     0.0722·Blinear` (coeficientes BT.709 em LINEAR space).
- **Fix**: `luma_srgb(r, g, b)`: decode sRGB→linear, aplica BT.709 em
  linear, re-encode Y_linear→sRGB. Devolve um perceptual brightness
  em [0,1] colorimetricamente correto. Custo: 4 pow() por pixel nos
  3 presets — negligível com debounce on-release.
- **Lição estrutural**: "BT.709 vs BT.601" é dimensão errada quando
  trabalhando em sRGB. A pergunta certa é "GAMMA-encoded shortcut vs
  LINEAR-light proper". sRGB primaries → BT.709 coefficients em
  LINEAR space → encode Y pra perceptual. Documente o por-que no
  docstring do helper pra próximo refactor não silenciosamente
  voltar pro shortcut.
- **Código**: [`crates/ph2d-tool-color-equalization/src/color_utils.rs luma_srgb`](../../crates/ph2d-tool-color-equalization/src/color_utils.rs).

### 12.13 CEQ panel sem scroll quando conteúdo overflowing


**Gate:** gate-deferred: documented incident; no class-level gate identified yet
- **Sintoma**: CEQ panel (~810px após Phase 2/3 adicionou histogram +
  14 sliders + LUT dropdowns + Posterize + Quantize + 4 auto buttons
  + CTA) não cabia no dock (~600px). Controles do rodapé invisíveis,
  unclickable. User dizia "scroll é padrão nos painéis do app" —
  CEQ não tinha.
- **Causa**: `panel_scroll` infra existia em `WidgetStore` (usada por
  GridSnap, Inspector, Widget Gallery) mas CEQ panel paint nunca
  wirou. `state.rs` tinha thread-locals `LAST_CONTENT_H` /
  `LAST_VISIBLE_H` como "parity placeholder", nunca usados.
- **Fix**: copiar pattern do GridSnap — `push_clip(body_rect)`,
  `y = body_top - scroll`, paint, `pop_layer`, calcular `content_h
  = (y + scroll) - body_top`, `set_panel_content_h/visible_h` no
  store, `paint_scrollbar` + register thumb hit. Novo
  `COLOR_EQUALIZATION_SCROLLBAR_ID` em editor-core scrollbar.rs +
  mapping em `dispatch::scroll::scrollbar_panel_for_id`. Hits
  registrados no espaço scrollado matching cliques scrollados — sem
  desync.
- **Lição estrutural**: scroll DEVE ser padrão pra painéis com
  conteúdo dinâmico. Adicionar 2-3 controles "sem scroll" parece OK
  até o painel cruzar o threshold. Future-proof: arch-test que
  exija content_h/visible_h publish em todo Panel<State>.
- **Código**: [`crates/ph2d-panel-color-equalization/src/paint.rs`
  scroll block](../../crates/ph2d-panel-color-equalization/src/paint.rs);
  [`crates/ph2d-editor-core/src/widget/scrollbar.rs COLOR_EQUALIZATION_SCROLLBAR_ID`](../../crates/ph2d-editor-core/src/widget/scrollbar.rs).

### 12.14 Dropdown labels longos truncam em chips estreitos


**Gate:** gate-deferred: documented incident; no class-level gate identified yet
- **Sintoma**: LUT preset options pintavam "Cinematic › Blockbuster",
  "Atmosphere › Golden Hour" — labels com prefix de grupo. No popover
  com chip width ~135px, texto wrappava em 2 linhas e era cortado
  pelo row_h.
- **Fix**: dropei o prefix. `LutPreset::ALL` já está clusterizado por
  grupo (None, 3 Cinematic, 4 Atmosphere, 4 Vintage, 4 Stylized) —
  ordem visual basta como signal de grouping, sem precisar prefixar
  cada label. Chip + opções cabem em uma linha. Header de grupo
  seria nice-to-have (precisa widget extension) — deferido.
- **Lição estrutural**: ao decidir entre "prefix label pra clareza"
  vs "espaço apertado", PREFIRA ordem implícita (ordem da lista) +
  separators visuais se possível. Prefixar todo item escala mal.
- **Código**: [`crates/ph2d-panel-color-equalization/src/paint.rs
  lut_options_for_slot`](../../crates/ph2d-panel-color-equalization/src/paint.rs).

### 12.15 Multi-sprite undo restaurava só a última sprite (CRITICAL data-loss)


**Gate:** gate-deferred: documented incident; no class-level gate identified yet
- **Sintoma**: Apply de qualquer image-tool sobre N>1 sprites
  selecionadas + Cmd+Z restaurava APENAS a última. As N-1 anteriores
  ficavam com pixels baked permanentemente. Bug latente em todos os
  8 image-tools (Trim/Make Square/Rasterize/Padding/CEQ/EqualizeSizes/
  Upscale/BgRemoval). Confirmado pelo Agent B da auditoria 2026-05-23.
- **Causa raiz arquitetural**: `image_edit_undo:
  Option<ImageEditSnapshot>` era SLOT ÚNICO. Cada per-sprite drain
  fazia `drop_undo_pre_source_if_individual(slot)` + `*slot = Some(snap)`
  — o loop multi-sprite sobrescrevia N-1 vezes, deixando apenas a
  última. `app_state.rs:173` documentava como "Single-level by
  design… Future M14.x replaces this with a proper command stack".
- **Fix arquitetural**: introduzir `ImageEditTransaction { entries:
  Vec<ImageEditSnapshot>, label: &'static str }` em
  [`app_state.rs`](../../shells/desktop/src/app_state.rs).
  Cada per-sprite drain agora APPENDA num `pending_undo_entries:
  &mut Vec<ImageEditSnapshot>` (assinatura mudou). Após cada loop
  multi-sprite, o orquestrador em
  [`render_loop/image_edit.rs`](../../shells/desktop/src/render_loop/image_edit.rs)
  chama `commit_image_edit_transaction(renderer, slot, pending)` que:
  (a) libera as pre-source textures da transação ANTERIOR; (b) seta
  `slot = Some(ImageEditTransaction { entries: pending, label })`.
  `drain_undo_image_edit` em
  [`image_edit/undo.rs`](../../shells/desktop/src/hero_intents/image_edit/undo.rs)
  itera todas as entradas (reverse) restaurando Sprite + Transform +
  liberando post-edit textures. Toast adapta: "Color EQ" vs
  "Color EQ (5 sprites)".
- **Helper renomeado**: `drop_undo_pre_source_if_individual` →
  `drop_undo_pre_sources_if_individual` (plural) — itera entries.
- **Sentinel preservado**: EqualizeSizes' fit-by-scale entries usam
  `post_individual_id = 0` (sem texture acquire); undo skipa o release.
- **Lição estrutural**: "single-level by design" + comment apontando
  pra M14.x = débito que sangra produção. Quando a refatoração couber
  em ~10 arquivos e a alternativa é data-loss silenciosa, fazer
  agora. A ergonomia de drains (push em vez de replace) ficou MAIS
  limpa que a anterior — bug-fix entregou também simplificação.
- **Código**: 11 arquivos:
  [`shells/desktop/src/app_state.rs`](../../shells/desktop/src/app_state.rs),
  [`main.rs commit_image_edit_transaction`](../../shells/desktop/src/main.rs),
  [`render_loop/image_edit.rs`](../../shells/desktop/src/render_loop/image_edit.rs),
  [`hero_intents/image_edit/{undo,color_equalization,bgremoval,padding,upscale,equalize_sizes,make_square,rasterize,trim_transparency}.rs`](../../shells/desktop/src/hero_intents/image_edit/).
