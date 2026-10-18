# CONTINUAÇÃO — `line/MiroClone` W23: bloco 3 «quadro» fechado menos três itens do B5; o próximo é o resto do B5 e depois o bloco 4 — 2026-10-18

> Leitor: a LLM da próxima janela. Lista-mãe: [HANDOFF_CONTINUACAO_W23_ABERTOS](HANDOFF_CONTINUACAO_W23_ABERTOS_2026-10-18.md) ·
> anterior: [HANDOFF_CONTINUACAO_W23_CARTAO_LIVE](HANDOFF_CONTINUACAO_W23_CARTAO_LIVE_2026-10-18.md). Fidelidade:
> [07_fidelidade_excalidraw.md](../07_fidelidade_excalidraw.md) (já actualizado: A10 em §3, B4). Sem integração nem push: só por ordem do Enio.
> Commits do bloco (mais novo primeiro): `e302e4ab54` · `23d62b46b3` · `c6a625b553` · `78b556aac3` · `9d08c901c0` · `54817e9641` ·
> `787bf9a11e` · `03c1ff48f8` · `d6d87ee2c4` · `b5486b9360` · `2164c47194` · `45cfa88267` · `dd1f85cdac` · `28ac8c5f13` · `bc2423b2fc`.

## 1. Conferido ANTES de construir (não são trabalho)

- **A13 NÃO é defeito** — decidido na W19 (handoff W19 integração §7: «Roda do rato fica do quadro — `PlayerInput::Wheel`/`Mouse::Wheel` saíram»).
- **B1 já coberto** — o Miro só tem nota quadrada e retangular; temos as duas (`Item::NoteWide`, gate `board_bar_notes_tests`).
- **B3(c) vertical já feito** — `the_vertical_layout_is_the_same_law_turned`.
- **B4 já feito** — `ph2d-board-route/src/heads.rs`, `size = min(base, leg/2)`; o doc 07 foi corrigido. **B2** feito no bloco anterior.

## 2. O que mudou

**A7** `bc2423b2fc` — `ph2d-board-words::keywords` guarda a forma escrita quando há maiúscula além da 1.ª letra («API», «iPhone»); a de
começo de frase baixa. Gates `an_acronym_keeps_its_capitals_in_the_title` (antes dava «#api #docs #late»),
`a_word_whose_capitals_say_something_keeps_them`.

**A10** `28ac8c5f13` — `anchor_for` (`ph2d-board-route`): por FORA da forma o foco Orbit é o CURSOR (o `fixedPoint` do next; oráculo
`saidas/w10_R_ponta_banda_diag.next.json`: R 500,100 200×120, seta de (330,380) largada em (497,130) ⇒ `[-0.015, 0.25]`, ponta
(494,0; 134,49)); por dentro o ponto do contorno. Antes a ponta caía em ~138,8. Gate
`a_tip_dropped_in_the_outer_band_keeps_the_cursor_as_its_focus` (a ±0,01). O realce (outra metade do A10) já estava curado na W10.

**B3(a)** `dd1f85cdac` — `Editor::set_font_size` (`ph2d-board-edit/src/mind_font.rs`): numa RAIZ escala letra e tamanho de cada nó por
`size/letra` da raiz (linear; o teste confere contra o moldador real a < 0,5 px), `mind_settle`, UM passo (`history.record`); num nó
não-raiz nada muda. A barra do nó oferece a letra só na raiz (`board_bar_mind::groups/shows`); `Item::Font` chama `set_font_size`.
⚠️ Os tamanhos oferecidos são os `FONT_SIZES` (16/20/28/36): o 14 de nascença do mapa só volta por desfazer. Gates
`the_roots_font_scales_the_whole_map_in_one_step`, `the_font_is_offered_on_the_root_and_scales_the_whole_map`.

**B3(b)** `dd1f85cdac` — `mind_hover.rs`: `mind_target` (o seleccionado ou, sem selecção, o pairado `self.hover`); o «+» aparece ao pairar;
`mind_minus` (nó não-raiz com filhos à vista) no meio da borda de fora, pintado como a pega do «+» (`ph2d-board-render/src/mind.rs`);
clicar recolhe e selecciona (`mindmap_drop.rs`). Gate `hovering_a_node_shows_its_plus_and_a_minus_that_collapses`.
⚠️ A lei de POSIÇÃO do «−» é nossa (o Miro não a publica): a borda de onde saem os ramos.

**B3(d)** `45cfa88267` + `78b556aac3` — `sketch::paint_branch`: no rascunho o ramo sai pelo `hand_line` das setas, em cache (sal
`BRANCH_SALT = 1<<61`). O ramo tem SEMPRE traço: os nós do mapa são só texto e o traço à mão herdava «sem contorno» — a foto da cena 24
mostrou o mapa SEM ramos (o 1.º teste usava nós com contorno e passava). Gate `a_sketched_map_draws_its_branches_by_hand` (com nós sem contorno).

**B5(3)** `2164c47194` — `board_bar_heads.rs`: o «…» (`Item::MoreHeads(i)`) de cada grupo de pontas abre a grelha das 15 (`Head::ALL`);
`BoardLive::heads` (fecha ao trocar a selecção); itens no FIM da tabela; `head_key` cobre as 15. O gate `the_w16_items_come_last_and_move_no_id`
foi actualizado (o vídeo deixou de ser o último; a cauda é a grelha). Gate `the_more_tips_grid_offers_every_head_of_the_catalog`.

**B5(8)** `b5486b9360` — `Editor::bulk_draft` + `board_view::paint_bulk_hint`: o rascunho em massa vazio mostra «One idea per line. Press Esc
or Ctrl+Enter to make the notes» (tinta da nota a meia força). Gate `the_empty_bulk_draft_says_what_to_do` (glifos pintados).

**B5(5)** `d6d87ee2c4` — `ph2d-board-edit/src/cursor.rs` `Editor::cursor_at` → `BoardCursor` (eixo da pega rodado pelo ângulo da moldura;
Rotate); `board_view::cursor`; a shell (`despacho_metodos_janela_e_vetor.rs` `update_eyedropper_cursor`) traduz para `CursorIcon` (Grab na de
rodar). Gates `each_handle_asks_for_the_cursor_of_its_axis`, `the_screen_asks_for_the_resize_cursor_over_a_handle`.

**B5(6)** `03c1ff48f8` + `e302e4ab54` — `board_view::scroll(BoardScroll::{Wheel,Pan,Pinch})`; shell `WindowEvent::MouseWheel` → `on_wheel_event`
(PixelDelta sem Ctrl/Super sobre um quadro = Pan) e `WindowEvent::PinchGesture` → `on_pinch_gesture`, em
`shells/desktop/src/input_dispatch/trackpad_board.rs` (módulo irmão: `input_dispatch.rs` passava do tecto HR-18). A pinça do TOQUE
continua pela `on_mouse_wheel` (zoom). Gate `two_fingers_pan_the_board_and_a_pinch_zooms_it`.
⚠️ A escolha PixelDelta⇒trackpad NÃO tem gate (precisa do evento do sistema); no X11 o touchpad pode chegar como LineDelta (= zoom).

**B5(7)** `787bf9a11e` — `board_menu.rs`: o botão direito chega à porta do quadro (`board_view::canvas_down`, braço Secondary — a shell chama-a
primeiro) e abre `ContextMenuKind::BoardElement` (Copy · Paste · Duplicate · Bring to front · Send to back · Lock · Delete) sobre o elemento,
que fica seleccionado. Linhas `CTX_MENU_BOARD_EL_*` registadas com as do menu das abas (`document_tabs_menu::populate`). O comando
enfileira-se em `BoardLive::menu_command` e corre na pintura seguinte (`board_menu::run_pending`: é lá que há o moldador); Copy vai também à
área de transferência; Paste cola o que o QUADRO copiou (texto de fora continua pelo Ctrl+V).
Lição: o 1.º desenho ligava no pré-despacho do hero e o teste pelo hero passava — a janela real nunca abriria. O gate
`right_click_on_a_shape_opens_its_menu` vai pela porta da shell.

**B5(4), metade** `54817e9641` + `e302e4ab54` — abas de documento a mais: piso de 4 letras; em transbordo a fila mostra k abas com a activa
no meio e ‹ › (`Target::Prev/Next`, ids `DOC_TAB_PREV/NEXT`) que abrem a vizinha (dá a volta). Gates
`too_many_boards_show_a_window_with_the_active_one_and_arrows`, `the_tab_arrows_open_the_next_and_previous_board`. (A outra metade, o
desfazer, está em ABERTO §5.3.)

**Cena 24** `9d08c901c0` + `c6a625b553` — `PH2D_BOARD_SMOKE=24` (`smoke_w23.rs`, ROUTERS `max_level` 24): o mapa da cena 8 em rascunho com 6
lições do bloco 3. Gate `the_w23_scene_draws_the_map_by_hand_with_its_lessons`. Fotos:
`docs/MiroClone/ferramentas/w23_fotos/bloco3_cena24_{1366x768,1930x1040}.png`.
⚠️ O §5 do `CLAUDE.md` diz `PH2D_BOARD_SMOKE=1..22`: **o integrador conta 24** (23 = a cena do lado de quem entra, do cartão Live).

## 3. O que um merge pode partir

- `board_bar_heads.rs`: a tabela de itens da barra ganhou cauda (a grelha das 15) — outra linha que acrescente itens ao fim da tabela
  reabre `the_w16_items_come_last_and_move_no_id`.
- `ids/chrome/documents.rs`: +2 ids (`DOC_TAB_PREV/NEXT`); `CTX_MENU_BOARD_EL_*` no registo do menu das abas. Ids e `ContextMenuKind` são
  contagens que SOMAM entre linhas — reconte, não escolha.
- `shells/desktop/src/input_dispatch.rs` (no tecto HR-18) e `input_dispatch/trackpad_board.rs` (novo): quem tocar o despacho do rato/roda
  na shell conflita aqui; a cura de crescimento é mover, nunca subir o número.
- `CLAUDE.md` §5 (Quadro): `PH2D_BOARD_SMOKE=1..22` → `1..24`.
- O formato gravado NÃO mudou neste bloco (zero campo novo).

## 4. Prova de fecho

- Mutações M1–M16 todas vermelhas (agente `mutacao`; uma por item acima, incl. a porta do botão direito e o registo das linhas).
- `clippy -D warnings --all-targets` limpo em `ph2d-board-words`, `-edit`, `-route`, `-render`, `ph2d-editor-core`, `ph2d-app-board`,
  `ph2d-i18n`, `ph2d-host-desktop`.
- Suíte `PH2D_SUITE=onda nextest-impacted`: 19 366 corridos, 19 364 passaram; 2 falhas MINHAS curadas em `e302e4ab54` e re-corridas por
  nome, verdes: `file_loc_caps::shell_files_respect_hr18_loc_cap` (`input_dispatch.rs` 620 > 600) e
  `arch_safe_clamp_only::clamp_calls_in_ui_are_safe_or_justified` (`.clamp` nas abas → `max`/`min`). loadavg 3,7.

## 5. ABERTO (nesta ordem)

1. **B5(1)** arrastar o trecho do MEIO de uma seta cotovelo. O modelo não guarda trechos fixos (o `fixedSegments` do Excalidraw perde-se ao
   importar): precisa de campo novo no `Connector` (formato gravado + o fio ao vivo). Desenhar com cuidado (`/pd-feature`).
2. **B5(2)** rótulo ao longo da linha (hoje a meio da rota, horizontal; `ph2d-board-render/src/lib.rs:~520`). O Miro deixa-o seguir a
   linha; também campo de formato.
3. **B5(4)** desfazer as operações de aba (apagar um quadro pergunta antes; desfazer não existe).
4. Depois: bloco 4 (B6, B7, B10, B11 — MEDIR antes), bloco 5 (B9, A11), bloco 6 (B12, B13, B14, com `/pd-feature`).
5. Perguntas ao dono sem resposta: YouTube Premium; «Rascunho/Final» e a letra importada.

## 6. Smoke ao dono (curto)

1. `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-MiroClone && cargo build -p ph2d-host-desktop --profile smoke && PH2D_BOARD_SMOKE=24 ./target/smoke/ph2d-host-desktop`
2. Siga as 6 notas amarelas: pairar num nó (aparecem «+» e «−»; o «−» fecha o ramo e aparece o número) · clicar na raiz «Plan» e escolher
   uma letra maior na barra (o mapa todo cresce) · os ramos estão desenhados à mão · criar uma seta e, com ela seleccionada, clicar no «…»
   ao lado das pontas (aparecem as 15) · botão direito numa forma (o menu com Copy/Duplicate/…) · dois dedos no trackpad movem o quadro e a
   pinça dá zoom · com muitos quadros (carregar no + das abas várias vezes) aparecem ‹ › nas abas.
3. Deu errado se: o «−» não aparece ao pairar num nó com filhos; mudar a letra da raiz não mexe nos filhos; os ramos somem ou saem lisos;
   o menu do botão direito não abre; dois dedos dão zoom em vez de mover.
