# 03 — Mapa do código (estado em `4e648ec2b4`, 2026-11-01)

Levantado por leitura (agentes `explorador`), sem correr nada. Linhas aproximadas onde marcado `~`.
Antes de agir, **confira o código** — a árvore anda.

## Painel Bones (`crates/ph2d-panel-skeleton`)

- Painel: `src/lib.rs:70-105` (`SkeletonPanel`, `ID "skeleton"`, título i18n `panel.skeleton.title`,
  nasce fechado `DEFAULT_VISIBLE = false`, slot `RightTop`). Abre por Window → Bones
  (`ph2d-editor-core/src/screens/hero/chrome/skeleton_toggle.rs`).
- Pintura: `src/paint.rs:24` `paint()` — moldura/título `panel.vector.section.bone` (`:37-45`),
  semeadura dos números no store antes de pintar (`:58-76`), corpo em `scroll_area::open_with` (`:87`),
  pop-ups diferidos (`take_pending_bone_action_dd`, `take_pending_bone_tip_dd`, `~:140-146`).
- Corpo: `src/section.rs:136` `body()` — **sem cabeçalhos de secção** (o título do painel é o único);
  usa `RowCtx` (`row_h: ROW_H_PX`, `row_gap: control_gap_px()`), `open_fold`/`close_fold`.
  Sub-blocos: `section_campos.rs` (Length/Strength/Segments/Curve*, `:23-110`; IK `:113-146`;
  `LENGTH_STEP :101`, `ANGLE_STEP` 5), `section_tip.rs`, `section_smart.rs`, `limit_rows :338-386`,
  `ik_rows :402-460`, `handles_row :514-540`.
- Eventos: `src/seam.rs:151` `apply_event()` — `Click` → `EditorAction::ToolPanelEvent(PanelEvent::Click)`,
  `ValueChanged` → `PanelEvent::SetValue(id, v)`. **`PanelEvent=4` é contrato congelado** (CLAUDE.md §6):
  a mudança de UI não pode alterar o contrato — só a pintura e o despacho dentro do painel.
- Estado: `src/state.rs` — `thread_local!` publicados pela shell (`set_current_bone :137`…); o painel
  não vê o ECS. Quem publica o osso em foco:
  `shells/desktop/src/render_loop/fase_selection_mirror_bone_focus.rs`.
- HR-15: `tests/it/every_word_this_panel_shows_comes_from_the_string_table.rs`; chaves em
  `ph2d-i18n/src/vector.rs:309-345`.
- **Não usa** o laço de secções do Inspector (sem `PlanoCtx`/`fecha_seccao`) ⇒ hoje não tem tema,
  contorno nem notas por secção. A mudança tem de o pôr no laço.

## Inspector principal (`crates/ph2d-panel-inspector`)

- Cabeçalho `src/paint_head.rs` (título, «+» Add Component `:68-75`, fechar, divisor `:110`).
- Ordem das secções `src/paint.rs:139-270`: cartões do topo (`paint_cards.rs:68`) → Name
  (`paint_frame_shared.rs:17-60`) → Visibility (`sections/identity.rs:51`) → Transform
  (`sections/transform.rs`) → sprite → 9-Slice/Ordering/Sampling/Material & Blend → física → opcionais.
- Transform: Position X/Y (m ou px conforme `DisplayUnit`), Rotation, Scale X/Y (passo 0,1), Skew X/Y
  (±~89,4°); reset `INSP_TRANSFORM_RESET`; linhas por `rows::fields_row` com coluna de rótulo medida
  por secção (`property_row::Seccao`). Sem Pivot/lock.
- Laço de secções com tema/contorno/notas e reordenação: `src/plano.rs`, `paint_frame.rs:164`.
- **Não mostra ossos** — o osso vive no painel Bones.

## Secções: tema, contorno, notas, ordem, dobra (`crates/ph2d-editor-core`)

- Tema: menu de contexto (`ids/menus.rs:28-51`, `screens/hero/menu_tables_section.rs`,
  `screens/hero/theme_menu.rs`); estado `interaction/state/section_prefs_ops.rs:59-81`
  (`section_theme`/`set_section_theme`), **persistido** em `~/.ph2d/sections.txt` (`:227-240`); tema
  custom `section_prefs_ops/tema_custom.rs`; desenho `widget/section_plan.rs:48` `tema_da_seccao`,
  `widget/section_cards::retheme`.
- Contorno: `CTX_MENU_OUTLINE_*` (`ids/menus.rs:8-11`); estado `interaction/state/chrome_ops.rs:590-602`
  (mapa em memória — **persistência não confirmada**); paleta `widget/panel_chrome/highlighter.rs:19`;
  desenho `widget/showcase/notes_chrome.rs` (`fecha_seccao`).
- Notas: `interaction/state/mod.rs:432` `notes_per_panel`, `notes_ops.rs`; menu «Create Note»
  `screens/hero/menu_tables.rs:19`; desenho `notes_chrome.rs` (`fecha_seccao`,
  `pinta_as_que_sobram`, chamado por `panel/scroll_area.rs:62`).
- Ordem: `widget/section_plan.rs:38` `ordem()`; arrasto pela pega `interaction/dispatch/section_drag.rs`,
  `widget/section_header/grip.rs:105`; persistida (`order=` em `sections.txt`).
- Dobra: `interaction/state/collapse_ops.rs:18/39/72`; chevron `widget/section_header/fold.rs:54`;
  cabeçalho `widget/section_header/mod.rs:133`, helper `panel/rows.rs:119`.

## Campo numérico e arrasto de hoje

- Desenho: `widget/number_input/mod.rs:265`; `labeled_number_field(_unit)` `panel/rows.rs:226-240`;
  chip `widget/slider_with_chip/number_chip.rs:46`; com unidade `widget/numeric_input_with_unit.rs:254`.
- Início: `interaction/dispatch/pointer_down.rs:~500-540` — Down no CORPO do campo (não no rótulo).
- Limiar 4 px (`interaction/drag.rs:126/132`); eixo dominante decide (`dispatch/pointer_move.rs:165-185`).
- Taxas: com intervalo = intervalo/250 px (H, `DRAG_RANGE_PX_H drag.rs:154`), /2500 (V); sem intervalo
  50×passo/px (H), 5×passo/px (V) (`drag.rs:145-146`); `number_drag_rate` registado vence.
- **Shift = ×0,001 (fino)** `DRAG_SHIFT_MUL drag.rs:147` — oposto ao Figma (ver [02 §3](02_especificacao.md)).
- Incremental por Move, cursor **não** preso, sem `warp`; Esc cancela (`dispatch/key.rs:251-258`);
  soltar confirma (`dispatch/pointer_up.rs:41`).

## Cursor preso (arrasto infinito)

- Sonda pronta: `crates/ph2d-probe-cursor-grab` (`probe_can_the_cursor_be_locked`, `--ignored`,
  precisa de uma mão a mexer o rato nas duas fases). Nenhum `device_event` na shell hoje.
- Mapa de plataformas: ver [02 §3](02_especificacao.md#arrasto-infinito--mecanismo-por-plataforma).

## Kanban — o que reutilizar (`crates/ph2d-app-kanban`)

> ⭐ **Desde a C1 (01/11)** o genérico vive em [`crates/ph2d-card-stack`](../../crates/ph2d-card-stack/src/lib.rs):
> `FLIGHT_TILT_DEG`/`FLIGHT_PIVOT_Y`, `EDGE_SCROLL_*`, `shadow::*`, `edge_depth`, `edge_scroll_px`,
> `slot_index`, `flight_pivot`, `Tilt`, e a lista VERTICAL com chave `Stack<K>` (`lift`/`drag`/`land`/
> `cancel`/`place`, `targets`, `reorder`; `drag(.., top: None)` = fora da lista, o vão fecha — C5).
> O Kanban importa-as (`metrics.rs`, `view.rs` `edge_scroll`/`retarget`, `tab.rs` `place`, `paint.rs` `flying`).

- Medidas `src/metrics.rs`: lista `LIST_W 272 :28`, `LIST_RADIUS 12 :29`, `LIST_GAP 12 :27`,
  `CARD_GAP 8 :45`, `HEADER_H 52 :6`, cartão `CARD_RADIUS 8 :77`, `CARD_PAD_X/Y 12/8 :78-79`,
  sombras `:94-115`, `FLIGHT_TILT_DEG 4`, `FLIGHT_PIVOT_Y 0.6 :104-105`, `EDGE_SCROLL_BAND 64`,
  900 px/s `:120-121`.
- Pintura: `paint_card.rs` (`card ~:98`, `rest_shadow :51`, `flight_shadow :81`), `paint_list.rs`,
  `paint.rs:367` `shadow()` = `blurred_rounded_rect` do Vello (sombra analítica real), voo `paint.rs:384`.
- Dinâmica: `pointer.rs:287` `start_drag` (`Lift`/`Drag`), `tab.rs:350` `place`, `place_list ~:262`,
  molas `Springs`/`CardSpring` (`tab.rs:84-100`), inclinação `tab.rs:387-401`; auto-scroll
  `view.rs:335-365`; relógio `frame.rs:12-28` → `docs.set_clock`; `wants_frame` `surface.rs:249`.
- **Genérico (reutilizar tal qual)**: `crates/ph2d-spring` (`Follow`, passo fixo 1/240 s, ζ=1, ω=12
  por omissão, `follow.rs:46-70`) e `UiMotion::law(Role::Travel)` (`ph2d-editor-core/src/motion.rs:470`;
  molas `EXPRESSIVE :172`, `DISCRETE :196`).
- **Preso ao Kanban**: `place`/`Lift`/`Drag`/`retarget`/`edge_scroll`/`flying` (tipos `ListBox`,
  `CardBox`). ⇒ Extrair um módulo genérico «lista de cartões com mola» (chave → rect; vão; voo;
  auto-scroll) que o Kanban e os painéis partilhem — é o que a regra
  «todo arrastar-reordenar voa como o cartão do Kanban» (memória `feedback_every_drag_reorder_flies_like_the_kanban_card`)
  pede. Não existe ainda (a paleta do Pixel reordena por função pura sem mola,
  `ph2d-app-pixel/src/paleta.rs:30`).

## Stack (Vello 0.11 · wgpu 30 · Rust 1.99, subida de 30/10)

- `vello` 0.10 → 0.11: a API de `Scene` é **idêntica** (diff só de `#[expect]` e um caso do emoji da
  Apple); 0.11 existe para acompanhar o `wgpu` 30. **Nenhuma capacidade de desenho nova** para esta
  UI. O que ela precisa já existe: `blurred_rounded_rect` (sombras), `push_layer` (alpha/clip para
  fades), `Affine` (inclinação), Parley para texto com `tnum`.
- O arrasto infinito depende do **winit** (`set_cursor_grab`, `DeviceEvent::MouseMotion`), não do Vello.
- Fontes embutidas: `crates/ph2d-text/fonts/` (Inter, Noto Sans, Atkinson Hyperlegible Next, JetBrains
  Mono, Virgil); a Inter é hoje a padrão (`ph2d-text/src/bundled.rs:19-36`) — passa a Noto Sans.

## Smokes de referência

- `PH2D_VEC_BONE_SMOKE=1..8` (7 = os três modos do esqueleto; o painel Bones abre pela cena).
  **8 = o painel Bones em cartões, cheio**: barra presa a 3 ossos, o osso da ponta já em foco (IK,
  Angle Limit, 6 segmentos curvos) e a ferramenta em Transform (`ph2d-app-vec/src/smoke_bone_cartoes.rs`).
- `PH2D_VEC_CARTOES_SMOKE=1` — **o Inspector em cartões, parado**: retângulo com Fill `4C8DF0` + Stroke `FFFFFF` 3 px, escolhido, ferramenta Vector, aba Inspector à frente (`ph2d-app-vec/src/smoke_cartoes_inspector.rs`, chamado de `vector_mode::smoke_step`). Foto × maquete C: `maquete/c4_maquete_x_app.png`.
- `PH2D_KANBAN_SMOKE=1..9`.
- Foto sem tocar no ecrã do dono: `docs/Components/ferramentas/fotografa_cena.sh` — aceita várias
  variáveis no 1.º argumento (`"HOME=<home isolado com layout.txt> PH2D_…=n"`), útil para abrir
  painéis pelo `layout.txt` (`open.<painel>=1` na secção da área).

## API fina para a C2/C3 (levantada 01/11, `explorador`; confira antes de usar)

- **Pintar um painel**: `PaintCtx` (`ph2d-editor-core/src/panel/paint_ctx.rs:20`: `host`, `slot`,
  `scene`, `text_system`). Bones: `ph2d-panel-skeleton/src/paint.rs:22` — moldura
  `paint_panel_surface` (`widget/panel_chrome.rs:219`; há `paint_panel_surface_floating` `:225`),
  título `paint_panel_title` (`:120`), corpo em `scroll_area::open_with`, `RowCtx`.
- **Clique**: `HitIndex::register(NodeId, Rect)` (`interaction/hit.rs:71`), `register_with_secondary`
  (`:112`), `register_section` (`:89`); acesso `ctx.host.store_and_hit_index_mut()`.
- **Campo numérico de hoje**: `RowCtx::labeled_number_field_unit` (`panel/rows.rs:231`) =
  `label_cell` + `value_col` + `register` + `paint_number_input_with_buffer`; o valor entra no store
  por `store.set_number_value` ANTES de pintar (`paint.rs`, bloco de semeadura).
- **Arrasto numérico**: semente em `dispatch/pointer_down.rs:~505-540` (`NumberInputDragState`,
  `drag.rs:23`) ⇒ é aqui que «só no ícone» entra; taxa `state/number_scrub.rs:92`
  `number_scrub_law`, aplicada em `dispatch/pointer_move.rs:~219-266` (`shift_mul`); fim
  `dispatch/pointer_up.rs:41` ⇒ `WidgetEvent::ValueChanged`.
- **Cursor**: só a shell mexe (`shells/desktop/src/input_dispatch/despacho_metodos_janela_e_vetor.rs:286`
  `win.set_cursor`). **Não existe** `set_cursor_grab`/`set_cursor_visible`/`set_cursor_position` nem
  `device_event` em lado nenhum, nem canal editor-core → shell para pedir o cursor ⇒ a C2 cria-o.
- **Reordenar secções hoje** (`dispatch/section_drag.rs`) **não é animado** ⇒ a C3 troca-o pelo
  `ph2d_card_stack::Stack<NodeId>`.
- **KeyMark**: `widget/property_box/key_mark.rs:276` `paint_key_mark(scene, theme, r, mark)` (losango
  `Spacing::Xs`), clique por `key_mark_target(id)` no `pointer_down` antes de tudo.
- **Ícones**: `paint_icons.rs:18` `paint_icon(scene, IconId, rect, color, stroke_w)`; dados em
  `icons.rs` (`IconId` `:71`, `cmds` `:313`, viewBox 24, Lucide).
- **Texto**: `TextSystem::layout(_with_weight)` (`ph2d-text/src/system.rs:181/242`); **sem** API de
  features OpenType (`tnum`) — a C2 tem de a abrir ou aceitar algarismos proporcionais.
- **Cena do smoke**: `PH2D_VEC_BONE_SMOKE=7` não foca osso nenhum ⇒ o painel mostra só Tool + Bind
  (foto `maquete/antes_painel_bones.png`); a C3 usa a `=8` (osso em foco, pele presa, IK e limite).

## Inspector em cartões (C4, 01/11)

- Liga: `ph2d-panel-inspector/src/paint.rs` (`cards::active()` = `PH2D_CARDS≠0` e sem sprite) →
  `cards::prepare` (campos, listas suspensas, dobras) → `paint_card_list_with(.., Footer)`.
- Cartões: `ph2d-panel-inspector/src/cards.rs`. Ids: os do Inspector (`INSP_TRANSFORM_*`,
  `INSP_ENTITY_NAME`, `INSP_VISIBILITY_CHECK`, `INSP_ORDER_Z_INDEX`, `INSP_SAMPLE_BLEND`,
  `INSP_ADD_COMPONENT`) e os do Vector (`VECTOR_OBJ_OPACITY_NUM`, `vector_obj_blend_option_id`,
  `vector_paint_{swatch,eye,del,row}_id`, `VECTOR_PAINT_OPACITY_NUM`, `VECTOR_WIDTH_NUM`,
  `VECTOR_ALIGN_*`, `VECTOR_CAP_*`, `VECTOR_JOIN_*`, `VECTOR_ARRANGE_ROTATE_CW/FLIP_H/FLIP_V`).
- Estado lido: `ph2d_tool_vector::appearance::{current_appearance, open_layer_index,
  current_vector_style}` (publicados pela `ph2d-app-vec`, `vector_bridge_publish.rs`).
- Corrente do Scale: `cards::{SCALE_LOCK, keep_proportion}` chamado em `event.rs` antes do
  `commit_transform_edit`.
- Seta do cartão dobrado (T19): `panel::cards` (cabeçalho), `metrics::CARD_CHEVRON*`. Caixa de texto do cartão em repouso = a caixa dos campos (`CardField`); a editar, a `TextInput`.
- Ícones novos (T23): `card-cap-{butt,round,square}`, `card-join-{miter,round,bevel}`, `card-opacity`, `card-path`, `card-weight` — e a lista à mão de `icons_tests.rs` (`ALL_ICONS`) que não tinha os 16 da C2 (o `enum_order_matches_svgs` estava vermelho desde a C2).
- **Cor do cartão (T25)**: TODA cor de dentro do cartão passa por `widget::card_ink::CardInk` (`of_card(store, secção, tema da lista)` → `.color(token)`): `panel::cards` (`CardCtx.ink`, `paint_card`, lista suspensa — a tinta viaja no `dropdown::stash`) e `widget::card_field` (campo, rótulo, ponto). Tons: `ph2d_tokens::derive::{CardTone, card_tone}`. A moldura da lista, o pé e o «…» ficam no tema da lista. Testes `two_card_themes_paint_two_different_cards_and_the_list_theme_stays_put`, `every_card_theme_keeps_its_boxes_off_its_surface`.
- **Rect publicado = a LISTA, flutuante (D24/T26)** (`set_floating_panel_rect(panel, lista)` em `panel::cards`): o canvas passa por trás; a costura de largura é a beirada da lista (`DockSides::with_floating` → `HeroLayout::float_lists` → `dock_seam`; ponto em `screens/hero/dock_reopen.rs::paint_float_seams`); `frame_layout` separa `docked_panel_rects`/`floating_panel_rects`; `layout.rs` (`chrome_x0/x1`) segura a faixa de ferramentas e a do fundo na coluna. Testes: `ph2d-panel-registry-init/tests/it/a_lista_de_cartoes_flutua_no_hero.rs` (hero montado: canvas por trás, clique abaixo vai ao canvas, costura na beirada, Tool dobra/desdobra 4×), `a_short_card_list_floats_and_keeps_its_seam_at_its_own_edge` (Bones), `a_floating_list_moves_the_seam_to_its_own_edge` (layout). Testes da lista no Bones: `ph2d-panel-skeleton/tests/cartoes_notas.rs`.
- **Cadeado (T27)**: `cards.rs` (cartão Objeto) lê `WidgetStore::hierarchy_row_of_entity` (publicado em `ph2d-panel-hierarchy::sync_from_hierarchy`) e pinta `ids::hier_lock_companion(linha)`; o despacho de companheiros (`pointer_down.rs`) e a Hierarquia (`event.rs`, `ToggleLock`) fazem o resto. Teste `ph2d-panel-registry-init/tests/it/o_cadeado_do_cartao_objeto.rs`.
- **Hex editável (T28)**: `cards::{hex_id, hex_layer, commit_hex}` + `fecha_seletor_do_hex` em `prepare`; `event.rs` (`Submit`); `panel::cards::CardCtx::swatch` (`edit`). Aplicação: `ph2d-app-vec/src/paint_stack_mixed.rs::apply_picker`.
- **Componentes em cartões (T29)**: `ph2d-panel-inspector/src/cards_componentes.rs` (`Abertas`, `plano` = `push_physics_sections` + `push_optional_sections`, `presentes` = a passagem de medida, `cartao` = o corpo de hoje com o cabeçalho absorvido); `plano.rs::into_tarefas`; `paint.rs` (ramo dos cartões + `paint_deferred_popovers`); `ph2d-editor-core/src/widget/section_header/mod.rs::absorb_next_header`. Cena `PH2D_VEC_CARTOES_SMOKE=2` (`ph2d-app-vec/src/smoke_cartoes_inspector.rs`). Teste `ph2d-panel-registry-init/tests/it/os_componentes_em_cartoes.rs`. **O corpo com a cara do cartão**: `ph2d-editor-core/src/panel/cards_corpo.rs` (`com`/`card_body`/`fields`/`choices`), ramos em `property_row.rs::paint_fields_row`, `property_row/escolha.rs::paint_choice_row` e `ph2d-panel-inspector/src/sections/rows.rs` (`seg_row`, `num_row_unit`); registo ao pintar `card_field_ops.rs::mark_card_field_at_paint`. Teste `os_corpos_dos_componentes_em_cartoes.rs`. **Dois campos por linha (T56)**: `sections/rows_par.rs` (`par_row` = um rótulo + `Eixos` ou dois rótulos; `seg_par_row` = duas escolhas se couberem) sobre `cards_corpo::columns`/`choices_pair`; a letra pintada de cada pega lê-se por `widget::card_field::painted_letter`; chamadores na física: `physics.rs::paint_shape_dims`, `physics_body.rs`, `physics_rows.rs`, `physics_area_rows.rs`. Teste `os_corpos_dos_componentes_em_cartoes::the_physics_body_card_puts_two_fields_per_row`.
- **Rolagem (T30)**: `panel::cards::paint_card_list_with` (`origem`, `open_with`/`close_parts`, `scroll_bar_of`). Teste `a_long_card_list_scrolls_under_the_wheel`.
- **Length em px (T32)**: `ph2d-app-skeleton/src/knobs.rs` (`px_per_unit`, `spec_on_panel`, `apply_from_panel`); publicado em `fase_selection_mirror_bone_focus.rs` (`set_current_bone_px_per_unit`), gravado em `fase_skeleton_verbs.rs`; o painel multiplica em `section_campos::campos_do_osso`. Teste `knobs_tests::the_bone_length_field_speaks_world_pixels`.
- **Cartão solto (C5, T33/T34)**: estado `ph2d-editor-core/src/interaction/state/card_loose_ops.rs` (`LooseCard`, `detach_card`/`return_loose_card`/`raise_loose_card`/`settle_loose_card`, `set_loose_card_rect`, `loose_{rect,ghost,return}_of`, `escreve`/`le` no `sections.txt`); pintura e gesto `panel/cards_solto.rs` (`arrasta`, `fantasma`, `pinta_soltos`, `publica`, `clique`, `preso` ao canvas) com os ganchos em `panel/cards.rs::paint_card_list_with` (`na_lista`, `reentra`, `solta`) e `paint_card_with` (a ação extra); clique antes dos painéis em `screens/hero.rs` (`apply_loose_click`); `ph2d_card_stack::Stack::detach` e `metrics::LOOSE_*`; `paint::stroke_rounded_rect_dashed`. Testes `ph2d-panel-registry-init/tests/it/o_cartao_solto_no_canvas.rs` (3) e `card_loose_ops_tests.rs` (2). Foto: semear `HOME/.ph2d/sections.txt` com `loose.7102543075790560757=17803112432873095310,980,150,272` (IK do Bones) e passar `HOME=…` no 1.º argumento do `fotografa_cena.sh`.
- **Dicas (T31)**: `panel::cards::dicas` (`comeca`/`icone`/`rotulo`, `icon_only_ids`, `field_hint`); `screens/hero/topbar/mod.rs::paint_hover_tooltip` cai na `field_hint`; textos em `ph2d-i18n/src/cartoes.rs` (`card.field.drag_hint*`, `card.insp.layer_*`, `card.bones.bend_*`). Censo `ph2d-panel-registry-init/tests/it/toda_dica_dos_cartoes.rs`.
- Menu do painel (D21): `panel::cards::{list_menu_of, apply_list_menu_click}`,
  `ContextMenuKind::CardListMenu`, `WidgetStore::reset_card_list`.
- **Rótulos que quebram**: `ph2d-editor-core/src/panel/cards_rotulos.rs` (`faixa` pinta os rótulos e as marcas de uma linha; `linhas` = no máximo duas, na última palavra que cabe). **Letras das caixas (T29)**: `widget/property_box/key_mark.rs::Eixos` no `Rotulo` → `panel/cards_corpo.rs::glifo`.
- **Estilos de cor (D28/D30, T35)**: modelo `ph2d-vec-scene/src/estilos.rs` (`ColorStyle`, `add/rename/duplicate/remove_estilo`, `set_estilo_color`, `link/unlink_estilo`, `estilo_de`, `usos_vivos`) + o `Serialize`/`Deserialize` da `VecScene` em `cad.rs` (v25 = `VecSceneV25`; dentro de `with_wire_layout(≤25)` grava a forma v25); escada do projeto `shells/desktop/src/project_schema.rs` (`PROJECT_SCHEMA_SEM_ESTILOS`, `fio_do_projeto`). Lista no cartão `ph2d-editor-core/src/panel/cards_estilos.rs` (`declare`, `botao_de`, `linha_de`, `solta_de`, `MAIS`/`NOME`/`HEX`, `on_event` antes dos painéis em `screens/hero.rs`, `abre_menu` em `dispatch/pointer_down_menus.rs`, `ContextMenuKind::ColorStyleRow` + `COLOR_STYLE_ROWS` + `CTX_MENU_STYLE_*`); o campo de cor ganhou `Widget::Swatch::detach`. Inspector: `cards.rs` (⠿ no cabeçalho do Fill/Stroke, nome do estilo no campo). Cena: `ph2d-app-vec/src/paint_stack_estilos.rs::drena` (pedidos → cena, vista → lista; chamado no `apply_picker`) e `paint_stack::set_color` (cor à mão solta). Ícone `card-styles`. Textos `card.styles.*`. Busca `BUSCA` + `filtrados`; a largura da caixa da cor em `panel/cards_colunas.rs` (`FIELD_VALUE_MIN_W`); a ligação viaja no recorte (`VecClip::ligacoes`, `structure.rs`). Smoke `PH2D_VEC_CARTOES_SMOKE=3|4|5` (5 = «Edit style», `abre_o_editor_do_primeiro`).
- **Lista abaixo da régua (D31)**: `paint_card_list_with` lê `PanelHostInternal::content_area` (`HeroScreen::last_content`); `metrics::LIST_MARGIN` = 5. **Solto com a aba atrás**: `panel_walk` + `WidgetStore::{begin,end}_loose_only_pass`.
- **Alfinete e sujeito (T36)**: editor-core `interaction/state/card_subject_ops.rs` (`set_card_subject`, `mark_card_pinnable`, `toggle_loose_pin`, `loose_pins_of`, `set_loose_card_title`, `note_card_press`/`card_press_subject`, `loose_pin_of`), `LooseCard::pin`; pintura `panel/cards_solto.rs` (`prepara` com o título e a dica do alfinete, `pinta_soltos` com `Extra { acoes, sub, apagado }` de `paint_card_with`, `indisponivel`); pressão em `dispatch/pointer_down.rs` (`note_card_press`). Bones: `state_preso.rs` (`Retrato`, `publica_presos`, `com_preso`, `set_current_bone_subject`), `cards_presos.rs` (`cards`, `COM_SUJEITO`), `paint.rs::semeia(store, so)`. App: `ph2d-app-skeleton::knobs::{live_bone, bone_subject}`. Shell: `fase_selection_mirror_bone_focus.rs::publica_o_osso` + `publica_presos`; `fase_bus_drain.rs` (`osso_selecionado` = osso preso sob a pressão, senão a seleção). Ícone `card-pin`. Testes `o_cartao_solto_no_canvas::{a_pinned_card_shows_and_asks_its_own_bone, a_loose_card_that_no_longer_applies_stays_and_says_so, dragging_a_card_to_the_bottom_edge_scrolls_the_list}`.
- **Auto-scroll (T38)**: `panel/cards_rolagem.rs::na_borda`, chamado em `paint_card_list_with` com o arrasto dentro da lista.
- **Bloco 8 (T57–T59, 02/11)**: **T57** `panel/cards_solto_rola.rs` (`Rola`, `corpo` = o corpo de TODO cartão — campos + notas —, com `Rola` o do solto dentro de `widget::scroll_area` com dono `loose_rect_of(k)`; `acima_do_hud`; `publica`), ligado em `paint_card_with` (`Extra::rola`) e `cards_solto::{pinta_soltos, pinta_os_soltos, publica}`. **T58** `widget::section_header::absorbing_next_header` lido por `ph2d-panel-inspector/src/paint_frame.rs::begin_section`. **T59** `sections/rows_moldura.rs::card_frame_with` (a moldura medida) chamada por `sections/player.rs::paint_cards`; pares `player.rs::PLAYER_PARES` (exposto por `lib.rs::player_card_pairs`) e `wheel.rs` (`par_row`). Cena `PH2D_VEC_CARTOES_SMOKE=6` (`ph2d-app-vec/src/smoke_cartoes_inspector.rs`, tecto 6 em `lib.rs`). Teste `ph2d-panel-registry-init/tests/it/o_bloco8_player_solto.rs`.
- **Bloco 9 (T60, 02/11) — o ↺ dos cartões de componente**: `ph2d-panel-inspector/src/cards_componentes.rs` (`reset_of` = id do ↺, exposto como `card_reset_of`; `repoe` = o componente da secção por `reveal::SECTION_COMPONENT` + `may_have_section`; `prepara` regista botões e dica, chamado em `paint.rs`; `reset_click` chamado em `event.rs`); estado `ph2d-editor-core/src/interaction/state/card_list_ops.rs` (`CardLists::resets`, `request_component_reset`, `take_component_resets`); app `ph2d-app-components/src/component_attach.rs` (`reset_asked`, `reset_by_name`: componente + linhas `row_of` em cadeia, depois `attach_by_name`); shell `render_loop/fase_component_palette.rs` (`ca::reset_asked`, mesmas `COMPONENT_SEEDS`); texto `card.insp.reset_component`. Testes `component_attach_tests.rs` (2) e `ph2d-panel-registry-init/tests/it/o_bloco9_reset_e_alfinete.rs`. **T61:** construído no bloco 10.
- **Bloco 10 (T61 + Ctrl+Z, 02/11) — alfinete em todo cartão de componente; as caixas re-semeiam-se**:
- **Bloco 11 (T63, 02/11) — a face de cada cópia solta**: `interaction/state/card_instance_ops.rs` (`loose_face_of(k)` = a chave de tema/contorno/notas do solto; `is_loose_face`; `loose_at(x,y)` = (instância, painel) sob o ponteiro; `passa_face(painel, de, para, mover)` copia ou move tema, contorno e notas); `card_loose_ops.rs` chama-a em `detach_card` (cópia nova, sem suspenso: copia da secção), `return_loose_card` (sem suspenso: move para a secção) e `settle_suspended_card` (move); `card_list_ops::reset_card_list` (cores) limpa também as faces dos soltos do painel. Pintura: `panel/cards.rs` `paint_card_with` lê `face = extra.rola.map_or(card.section, |k| k.face)` para a tinta e o contorno; `cards_solto_rola::Rola::face` (notas do corpo); sombra (`cards_solto.rs`) e corda (`cards_solto_some.rs`) pela face. Menu: `interaction/dispatch/pointer_down_menus.rs` — sobre um solto o `panel_under` é o dono dele, `SectionOutline` leva a face (se o título é o dele) e `CreateNote` a face. `widget/showcase/notes_chrome::pinta_as_que_sobram` salta as notas de uma face viva. ⚠️ A DOBRA continua pelo id da secção (a cópia 1 partilha-a com a lista). Teste `o_bloco11_tema_por_copia::each_loose_copy_keeps_its_own_theme_and_notes`. **T29 parte 1:** `property_row/marca.rs` (`paint_check_row(s)` + ramo do cartão), `panel/cards_corpo.rs` (`Marca`, `check`, `chip`), `widget/dropdown/mod.rs` (`paint_dropdown_chip` → `corpo::chip`), `ph2d-card-stack::metrics::{CHECK_BOX, CHECK_GAP}`.
  - shell: `render_loop/snapshots_inspector.rs` (`Alvo {bits, n, todos}` com `da_selecao`/`preso`; `publish(hero, alvo, …)`; `publish_global` = catálogos, `asset_index_build::publish_for_frame`, árvore de tags, unidade), `snapshots_inspector_physics.rs` (`physics(alvo, …)`), `snapshots_inspector_sprite.rs` (`sprite_info(hero, alvo, …)`), `snapshots.rs` (`Preso {bits, live, law}`; closure `publica` corrida por `com_cada_preso` e depois para a seleção), `fase_snapshot_readouts.rs` (`presos`: vista e veredito do player de cada preso), `fase_snapshots_publish.rs` (closure `ultimas` = luz, corpo de luz, constraints, sequência; fase 2 dos presos com `tardios::publica`), `fase_macros.rs` (cartão Macro dos presos), `fase_selection_mirror_bone_focus.rs` (sai `publica_presos`; fica a corda).
  - painel `ph2d-panel-inspector`: `retrato.rs` (macro `retrato!` = lista única dos ~48 pares `current_inspector_*`/`set_current_inspector_*` + `slot_image::atual`; `Retrato::{captura, instala}` derive PartialEq; `com_cada_preso` (pub, a shell), `de`, `com_retrato`, `com_preso`; um preso novo parte do retrato vazio), `cards_presos.rs` (`ensaia`: semente da seleção (`sync::semeia_tudo(.., true, false)`) com o retrato do preso num ensaio, guardada por instância; `monta`: marca pinnable Transform + toda secção presente, `Preso {k, pin, secao, componente: Option<(Presente, LiveSnapshots)>}`; `cards`: o def com o retrato à volta do corpo; caches `MEDIDOS`/`MONTADOS` pela chave (retrato do preso, da seleção, unidade)), `cards_componentes.rs` (`presentes(.., so: Option<NodeId>)`, `Presente: Clone`, o ↺ lê o transform do retrato instalado), `event.rs` (`apply_event` corre `apply_event_do_retrato` dentro de `com_retrato(card_press_subject)`), `sync.rs` (`sync_inspector_from_snapshots` = só a troca (foco, arrasto, seletor, nome, busca de tags) + `semeia = entity_changed || bus vazio` + `vivas`; `semeia_tudo(state, host, semeia, trocou)`), `sync_sections.rs` (`limpa_busca_de_tags` pub(crate), chamada na troca), `paint.rs` (ensaia + monta antes dos defs).
  - núcleo `ph2d-editor-core`: `interaction/state/card_ensaio_ops.rs` (`Ensaio`, `EstadosPresos{estados, cores}` com `estado(id)`, `Trocados`; `abre_ensaio`/`fecha_ensaio`, `set_pinned_states`/`pinned_states`, `troca_estados_presos`/`repoe_estados` (salta a caixa viva do gesto sobre ESTE preso), `poe_estados_presos` chamado por `note_card_press`), `CardLists::pinned_states` (sai `resync`/`take_component_resync`), `panel/cards_solto.rs` (`pinta_soltos` recebe `&mut` e troca os estados à volta de cada solto; o `HitIndex` sai e volta em `pinta_os_soltos`), `action_bus_queue.rs` (`truncate`).
  - cena `ph2d-app-vec/src/smoke_cartoes_inspector.rs`: =6 dois objectos «Player A»/«Player B» (B com Float Height e Speed ×2); =7 prende o Player solto ao A e escolhe o B (`prende_e_escolhe_o_segundo`).
  - testes: `ph2d-panel-registry-init/tests/it/o_bloco9_reset_e_alfinete.rs` (+2), `o_inspector_armado::{BITS pub, player_armado}`, `o_bloco8_player_solto::solto_de` pub(super); bloco5/bloco7 portados para `com_cada_preso`/`pinned_states`.
- **Bloco 4 (T39–T43, 02/11)**: a lista pinta-se em `paint_card_list_pinned` (o `_with` chama-a com `pinned = &[]`); o menu «…» mudou-se para `panel/cards_menu.rs`. **T39** `LooseEdge`, `loose_edge_of`, `WidgetStore::{loose_edge, begin_loose_resize}`, `LooseCard::resize`, `CardDrag::{loose, edge}` (`card_list_ops.rs`), porta no `pointer_down` antes do cabeçalho do cartão, guarda da dobra no `pointer_up`, `cards_solto::largura`, faixas registadas depois do cartão em `pinta_soltos`; cursor `card_field::ew_resize_under` (shell `cursor_preso.rs`). **T40** `cards_solto::fantasmas`; Bones `cards_presos::cards` → `(lista, presos)` e `paint::valores`; `WidgetStore::{set_pinned_values, pinned_values}` aplicados em `note_card_press`; `card_field::com_valores` (sobreposição ao pintar o preso). **T41** `ph2d_card_stack::Stack::{arrive, arriving_dx, smooth_heights}`; `WidgetStore::{take_card_arrival, return_loose_cards_of}` (o `reset_card_list(order)` devolve os soltos); o que regressa pinta-se depois da lista, sem clique; cada cartão recortado à altura vista. **T42** `pinta_soltos` (o `indisponivel` saiu), `Extra::destaque`, `cards_solto::piscar`, `metrics::LOOSE_PIN_BLINK_*`. **T43** o `clip` do corpo desconta `LIST_PAD_BOTTOM`. Testes: `ph2d-panel-registry-init/tests/it/o_cartao_solto_bloco4.rs` (6), `cards_solto_tests.rs`, `ph2d-card-stack` `an_arriving_card…`/`a_folding_card…`.
- **Alfinete com corda (T44, 02/11)**: estado `LooseCard::rope` + `CardLists::ropes`; `toggle_loose_pin` cicla os três estados; `loose_ropes_of`, `set_loose_rope`/`loose_rope` (`card_subject_ops.rs`). Física: `ph2d-node-motion-verlet-rope/src/corda.rs` (`Corda::passo`, `Feitio` — chama o `step` privado do nó com `pin_tail`, sem contacto, perfil uniforme; espaço do nó, `y` para cima). Passo por quadro: crate nova **`ph2d-card-rope`** (`anda(store, painel, alvo)`/`anda_com`, `comprimento` = folga do vão horizontal + tensão a prumo (T44c), `rect_de_agora` = o cartão onde a mão o põe neste quadro (T44c); genérica — quem compõe dá o sujeito preso em ECRÃ; a borda do cartão pelo rect publicado; estado em `thread_local`), porque o tecto de linhas da `ph2d-app-skeleton` estava cheio (folga zero; a cura é mover, nunca subir). O sujeito-osso: `ph2d_skeleton_live::skin_live::bone_middle` (meio da raiz–ponta de `bone_segments`, o osso que se desenha). Composição na shell em `fase_selection_mirror_bone_focus.rs` com `gfx.camera.world_to_screen(.., scene_window)` (troca de linhas, zero crescimento). Pintura `panel/cards_solto.rs::corda` (antes da sombra; a 1.ª ponta re-presa à borda do cartão DESTE quadro, `na_borda`) e `solda` (T44b, depois do cartão), cor `KanbanCard`, ícone `card-pin-rope`, dica `card.loose.rope`. Feitio em `metrics::LOOSE_ROPE_*`. Duplo clique nos botões do solto = clique (`screens/hero.rs::apply_event`). Foto: `PH2D_CARTAO_CORDA_SMOKE=1` prende com corda o 1.º solto do Bones com sujeito (uma vez). Testes `o_alfinete_com_corda::the_pin_cycles_pinned_then_rope_then_loose` e o `a_pinned_card_shows_and_asks_its_own_bone` revisto (registry-init), `ph2d-card-rope` `tests::*` (2), `skin_live::tests::the_middle_of_a_bone_is_the_middle_of_the_drawn_bone`, `corda::tests::a_ui_rope_hangs_between_its_two_ends_and_follows_them` (verlet-rope). T44d: `return_loose_card` passa o `CardDrag` da secção a arrasto da LISTA (`loose = false`) — sem isso o voo congelava ao reentrar até largar; teste `a_card_dragged_back_keeps_following_the_hand_in_every_pin_state` (registry-init).
- **Bloco 5 (T45–T48, 02/11)**: **T45** `card_loose_ops.rs` (`PINO` = `loose_pin.`, `escreve`/`le_pino`), `CardLists::pin_keys`, `card_subject_ops.rs` (`loose_pin_keys`, `loose_pin_key`, `set_loose_pin_key`, `rebind_loose_pin`; `toggle_loose_pin`/`unpin_loose_card` largam a chave e marcam o ficheiro sujo); `ph2d-card-rope/src/chave.rs` (`resolve`, `transform_de`, `origem`; dep `ph2d-ecs`), chamado na shell em `fase_selection_mirror_bone_focus.rs` antes de `publica_presos`. **T46** `ph2d_card_stack::Stack::enter`; `panel/cards_entra_sai.rs` (`itens`, `fotografa`, `pinta_saida`, `saindo`) ligado em `paint_card_list_pinned`; `panel/cards_solto_some.rs` (`passo`, `toma`, `cola`, `apaga`, `alfa`, `recolhem`) ligado em `cards_solto::pinta_soltos`/`pinta_os_soltos`; `CardLists::reels` + `set_reeling_rope`/`reeling_ropes_of`; `ph2d-card-rope` `Cordas::recolhem`; `metrics::{LOOSE_ROPE_REEL_S, LOOSE_FADE_S}`. **T47** `ph2d-panel-inspector/src/cards_presos.rs` (`COM_SUJEITO = [CARD_TRANSFORM]`, `publica_presos`, `transform_do_gesto`, `cards`), `paint.rs` (`paint_card_list_pinned` com os presos), `event.rs` (Transform e Reset pelo `transform_do_gesto`); shell: o bloco `#[cfg(feature = "panel-inspector")]` em `fase_selection_mirror_bone_focus.rs` (troca de linhas: −5 no saldo). **T48** `Corda::empurra` (verlet-rope), `ph2d-card-rope` `passo`/`fora`, `metrics::LOOSE_ROPE_CLEAR`. Testes: `ph2d-panel-registry-init/tests/it/o_bloco5_abertos.rs` (3), `ph2d-card-rope` `tests::*` (+3 e a sonda).
- **Bloco 6 (T50–T53, smoke do dono 02/11)**: **T50** a corda pinta-se em `panel/cards_solto_some.rs` `cordas` (chamada no fim de `cards_solto::pinta_os_soltos`, depois de `publica`; absorve o `recolhem`), `pinta_corda`, `trocos` (pura, teste `a_rope_segment_over_another_loose_card_gets_its_own_tone`); `cards_solto::na_borda` `pub(super)`; saíram `corda`/`solda` de `pinta_soltos`, `Corda::empurra`, `ph2d-card-rope` `fora`, `metrics::LOOSE_ROPE_CLEAR`. **T51** `LooseCard::h` + `altura()` (`card_loose_ops.rs`), gravada em `cards_solto::publica`, lida em `pinta_soltos`/`arrasta`; `WidgetStore::loose_card_rect_now` (o rect deste quadro: publicado ou a geometria do cartão; a mão; o suspenso) — o `rect_de_agora` da `ph2d-card-rope` delega nele. **T52** `ph2d-card-rope` `Cordas::antes: BTreeMap<NodeId, Instant>`. **T53** `CardLists::suspended`; `return_loose_card` suspende quando o arrasto é dele e tem alfinete; `detach_card` restaura; `settle_suspended_card`, `suspended_card`; `loose_ropes_of` inclui o suspenso; `cards_solto::larga` (a queda: solta ou assenta) chamada em `paint_card_list_pinned` no lugar do `detach_card` (−2 linhas em `cards.rs`).
- **Bloco 7 (T54, 02/11)**: o solto é uma INSTÂNCIA — `LooseCard::section`, o mapa `CardLists::loose` e todo o estado por cartão (rect, alfinete, corda, chave, números, título, bordas, regresso, suspenso) chaveados pela instância; `interaction/state/card_instance_ops.rs` (`loose_instance_of`, `loose_section`, `is_loose_in`, `free_loose_instance`, `loose_sec.<instância>=<secção>` no `sections.txt` só quando instância ≠ secção). `detach_card` escolhe a 1.ª instância livre (o suspenso T53 volta à dele); `return_loose_card` passa o arrasto à SECÇÃO e a chegada é da secção; `take_card_drop`/`is_card_section`/`is_card_pinnable` resolvem a instância. Pintura: `cards_solto::instancia` (o def com o id da instância; os campos do corpo ficam os da secção), `prepara`/`fantasmas` por instância; `cards.rs` troca 4 linhas (`soltos` no lugar de `todos` no cancelamento da queda — a causa do «não permite»). Painéis: `cards_presos` (Inspector e Bones) dão `d.section = instância`. Testes `o_bloco7_duas_copias.rs` (2), `card_loose_ops_tests::two_loose_copies…`. Foto `maquete/b7_duas_copias.png` (cópia 2 = `loose.5919464000992983208=…` + `loose_sec.5919464000992983208=664538832509876160`).
