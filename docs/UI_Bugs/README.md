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
