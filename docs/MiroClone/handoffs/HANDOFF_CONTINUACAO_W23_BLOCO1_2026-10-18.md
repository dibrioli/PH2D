# CONTINUAÇÃO — `line/MiroClone` W23 bloco 1 fechado; o próximo é o bloco 2 (rede ao vivo) — 2026-10-18

> Leitor: a LLM da próxima janela. Lista-mãe: [HANDOFF_CONTINUACAO_W23_ABERTOS](HANDOFF_CONTINUACAO_W23_ABERTOS_2026-10-18.md)
> (não reescrita; tem uma nota no topo a apontar para aqui). Base: `main` de 18/10 (W22c). Commits do bloco:
> `cf3c3b030d` (A1+A6) · `aade84eaa3` (A3 A4 A5 A8) · `1543b06844` (limpeza + fotos).
> Antes de construir, confira no CÓDIGO (CLAUDE.md §5.0). Não há integração nem push: só por ordem do Enio.

## 1. O que o bloco 1 fez

**A1 — enquadramento.** Mecanismo MEDIDO nas fotos de 18/10: (1) `fit_pending` corria uma vez na 1.ª pintura com a
janela ainda 1024×768 (log da foto: `resize: 1024x768` e só depois `1366x740`); (2) a conta usava a área inteira,
mas o cartão «Live», a coluna dos cartões, a barra e a fila do canto pintam-se por cima. 12 cenas compensavam com
`MARGIN_X=450` em unidades do MUNDO. Cura:
- `Camera::fit_in(area, free, bbox, margin)` em `crates/ph2d-board-model/src/camera.rs`.
- Módulo novo `crates/ph2d-editor-core/src/screens/hero/board_nav_fit.rs`. Cada painel fixo regista o rect que
  PINTA (`fit::cover`): board_nav (barra + filas do canto + HUD em `clear_of`), frames, minimap, search,
  facilitate band, painel Live, painéis da equipa. `free_of(area, covers, bbox, margin)` corta a área pelo lado em
  que a caixa fica MAIOR (a 1.ª versão escolhia por área e a cena 22 a 1366 ficou sem letra — foto).
- `settle` corre em `paint.rs` ANTES de o quadro se desenhar (senão o tocador do vídeo e o clique seguinte liam
  câmaras diferentes: 3 testes de vídeo w18/w19/w20 caíram assim) e guarda um `FitHold` (`documents_nav.rs`) que
  refaz o enquadramento enquanto a câmara for a que ele deixou.
- Alt+1/Alt+2/«Fit»/ir a frame/achado/vídeo usam `fit::View`.
- As 12 cenas perderam a margem do mundo; cenas 1–5 deixaram a câmara à mão e chamam `board_nav::show`; cenas
  8/9: a margem virou respiro do frame (90).

**A6.** `toolbar_rects` (`board_bar_layout.rs`) parte a coluna de 19 botões em colunas de alturas iguais quando
não cabe; `layout::anchor` faz os painéis que abrem ao lado (formas, nota, caneta, modelos) ancorarem na última
coluna.

**A8a** curado pelo A1 (foto da cena 9). **A8b:** a regra «frame visto de longe» numa porta só:
`ph2d_board_model::far_frame` + `BoardDoc::under_far_frame` (`organize.rs`), usada pelo desenho (`frames.rs`),
pelos ícones de link (`board_nav.rs`) e pelo editor (`Editor::know_detail(zoom, px)`, chamado em `paint.rs`;
filtra `vote_marks` e as fichas de carimbo via `chip_where`).

**A3.** O «+n» dos avatares regista-se (`LiveButton::Everyone`; abre o painel na lista a partir de quem não
coube: `LivePanel::avatars_shown/people_from/people_shown`); o «+n» da lista é botão `MorePeople` que pagina e dá
a volta. i18n: `board.live.everyone_tip` / `more_people_tip`.

**A4.** `image_frame::paint_hand_back` (ph2d-board-render): fundo do vídeo com rascunho ou preenchimento às
riscas pinta-se pela lei das formas sem contorno, por trás do tocador. Só chega por `.excalidraw` importado
(nenhum controlo da UI põe `fill_style`), por isso sem foto.

**A5 — NÃO é defeito.** A janela redesenha a cada quadro (`ControlFlow::Poll` + `request_redraw` incondicional
em `present_title.rs:72`) e o anel anda pelo contador de desenhos. Gate: `the_loading_ring_turns_from_one_paint_to_the_next`.

**Já feitos antes desta janela (conferido no código):** B2 (setas e renomear no painel de Frames: teste
`the_frames_panel_goes_renames_and_reorders`); A2 (sinal de vida LAN: `Liveness{beat 2s, silence 10s}` em
`ph2d-board-net/src/host.rs`, teste `o_silencio_derruba_quem_se_calou_e_o_sinal_de_vida_segura_quem_so_nao_fala`
em `tests/it/ponta_a_ponta.rs`).

## 2. Gates novos (todos red-first por mutação; mutações restauradas)

| Gate | Onde |
|---|---|
| `a_scene_framed_before_the_window_grows_is_reframed_clear_of_the_panels` · `moving_the_view_ends_the_scene_reframe` · `fit_all_keeps_the_board_clear_of_the_live_card` | `board_nav_tests.rs` |
| `a_wide_scene_goes_below_the_card_and_a_tall_one_beside_it` · `every_cover_cuts_and_none_outside_does` | `board_nav_fit_tests.rs` |
| `fit_in_frames_the_box_inside_the_free_part_of_the_area` | `camera_tests.rs` (board-model) |
| `the_short_bar_wraps_in_a_low_window_and_every_button_stays_clickable` | `board_bar_tests.rs` |
| `the_plus_n_beside_the_avatars_opens_everyone_and_the_list_pages` | `board_live_tests.rs` |
| `a_frame_seen_from_far_hides_and_refuses_votes_and_stamps` | `facilitate_tests.rs` (ph2d-board-edit) |
| `a_hatched_video_background_is_painted_hatched` · `the_loading_ring_turns_from_one_paint_to_the_next` | `video_tests.rs` (ph2d-board-render) |

## 3. Prova de fecho e o que um merge pode partir

- `cargo fmt` limpo; clippy `-D warnings` limpo em board-model/edit/render, editor-core, app-board, i18n; 12
  filtros de teste por nome verdes.
- Suíte (`nextest-impacted`, 774 s): 18 318 testes, 18 317 verdes; o 1 vermelho era o tecto de LOC
  (`workspace_src_files_under_loc_cap`: `board_live_tests.rs` com 741 > 700) — curado em `b0fe974a41` (o teste
  do A3 foi para `board_live_people_tests.rs`), o tecto e o teste do A3 re-corridos por nome, verdes.
- Superfície de colisão: `paint.rs` (ordem `settle` → desenho), `documents_nav.rs` (`FitHold`), `board_bar_layout.rs`,
  `layout::anchor`, `organize.rs` (`far_frame`), `facilitate.rs`, `image_frame.rs`, `live.rs` do app-board
  (botões do Live). Quem mexer na ordem de `settle` em `paint.rs` parte os testes de vídeo w18/w19/w20.
- Nenhum contador de schema/registo/ADR mexido neste bloco (só lógica de vista, desenho e i18n).

## 4. Fotos e smoke

Fotos em `docs/MiroClone/ferramentas/w23_fotos/`: cenas 22 e 9 a 1366×768 e 1930×1040, cena 1 a 1366×768, cena 6
a 1366×560 (barra em 2 colunas), cena 13. Nota: a 1366×768 com os dois painéis laterais abertos sobram ~750 px de
quadro; a cena 22 vai inteira por baixo do cartão «Live», texto pequeno mas legível.

**Smoke ao dono (Enio):**
1. `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-MiroClone && PH2D_BOARD_SMOKE=22 cargo run --profile smoke -p ph2d-host-desktop`
2. Com a janela pequena e depois maximizada: o quadro «Retro online» aparece inteiro, nada por baixo do cartão «Live» nem da barra da esquerda.
3. Arraste a vista com a mão e depois mude o tamanho da janela: a vista fica onde você a deixou.
4. Feche o cartão «Live» e aperte Alt+1: o quadro enche o espaço livre.
5. Rode com `PH2D_BOARD_SMOKE=9`: o título «Facilitate a brainstorm» fica à vista ao lado da barra.
6. Diminua a altura da janela até a barra da esquerda não caber: ela passa a duas colunas e todos os botões continuam clicáveis; «mais formas» abre à direita da barra.

Deu errado se: algo ficar cortado na borda, por baixo do cartão, ou um botão da barra ficar fora da janela.
(O «+n» dos avatares só aparece com muita gente na sala — testado por código.)

## 5. ABERTO

- **Perguntas ao dono, sem resposta:** (1) YouTube Premium; (2) se «Rascunho/Final» troca também a letra importada.
- **Bloco 2 «rede ao vivo» (próximo).** A2 já está feito (acima). Falta:
  - **A9** — bind em `crates/ph2d-board-net/src/tcp.rs:36-51` varre 47320..47329 (`PORT_TRIES=10`); sem
    `SO_REUSEADDR` explícito; `Drop for Host` (`host.rs` ~463-486) espera as threads com `JOIN_DEADLINE=2s`; a porta
    fica presa se o accept passar do prazo. Adeus: `live::shutdown` (`ph2d-app-board/src/live.rs` ~264) no pedido
    de fechar a janela; processo morto não manda adeus. Teste vermelho cabe em `door_tests.rs` (perto de
    `fechar_acorda_todas_as_portas`, :153). **Meça os 2,5 s antes** de mexer.
  - **A12** — `join_by`/`dial` em `ph2d-app-board/src/live_internet.rs` (~117-170): cada tentativa faz `cfg.node()`
    (UDP novo) e `Rendezvous::new`; `link_failed` (~189-205) com `link_old`/`link_offline` sem teste; `connected()`
    em `ph2d-board-internet/src/driver.rs:428-460` fecha sem `path_lost` (`driver_dial.rs:123`) quando o stream
    não abre. Testes vizinhos: `node_tests.rs:148/158/177`.
  - **B8** — as escritas saem a cada imagem (60 Hz) nos dois transportes; a presença já é 30 Hz
    (`SEND_EVERY_MS=33`, `ph2d-board-sync/src/presence.rs`). `Replica::local` (`replica.rs:299`) drena
    `board.doc.take_changes()` — diferença de ESTADO, logo drenar a 30 Hz agrupa sozinho. O dreno é
    `s.tick(b, now)` em `host_tick` (`live.rs` ~363) e no `guest_tick`. **Meça bytes/pacotes por segundo num
    arrasto antes de mexer** ([MEDIR_VELOCIDADE.md](../../DevOps/MEDIR_VELOCIDADE.md)).
- **Blocos 3–6, pela ordem da lista-mãe:** 3 = A7, A10, A13, B1, B3, B4, B5 · 4 = B6, B7, B10, B11 · 5 = B9, A11 ·
  6 = B12, B13, B14 (com `/pd-feature`). Os testes que precisam do dono (furo real pela internet, Windows/Mac
  reais) ficam fora desta onda.
