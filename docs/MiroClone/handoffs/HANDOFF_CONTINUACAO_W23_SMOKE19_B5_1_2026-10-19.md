# CONTINUAÇÃO — `line/MiroClone` W23: smoke do dono (19/10) curado e B5(1) feito; o próximo é o B5(2) — 2026-10-19

> ✅ **Smoke do dono 19/10: OK** (cenas 24 e 25). O próximo é o §5 ABERTO: B5(2), depois B5(4).

> Leitor: a LLM da próxima janela. Anterior: [HANDOFF_CONTINUACAO_W23_BLOCO3](HANDOFF_CONTINUACAO_W23_BLOCO3_2026-10-18.md) ·
> lista-mãe: [HANDOFF_CONTINUACAO_W23_ABERTOS](HANDOFF_CONTINUACAO_W23_ABERTOS_2026-10-18.md). HEAD `a83d27b7e4`, base = main `100d34c8e0`.
> Sem integração nem push: só por ordem do Enio. Plano e oráculos do B5(1): [13_trecho_do_cotovelo.md](../13_trecho_do_cotovelo.md).

## 1. Smoke do dono 19/10 — curado (`b19c0745e8`, `d0d7eaaab2`)

1. **O «−» do mapa tapava a pega `Resize(E)` da moldura.** `mind_minus` (`ph2d-board-edit/src/mind_hover.rs`) punha-o no meio da borda.
   Agora `Overlay::mind_minus = (ponto, dir)` e afasta-se `mind_minus_offset = Metrics::dot/2` px de ecrã (16 px reais; o «+» a 32; pega
   de raio 6). Gate red-first `the_minus_of_a_selected_node_stays_clear_of_the_frame_handles` (offset 0: `Resize(NE)` a 14,5 px). Foto
   `ferramentas/w23_fotos/smoke19_menos_fora_da_pega_1366x768.png`.
2. **«Não há … ao lado das pontas».** MEDIDO pela foto (`w23_fotos/smoke19_antes_reticencias_apagadas_1366x768.png`): o «…» estava na barra
   e no índice de cliques; o ícone `more-horizontal` do Feather (círculos r=1 contornados, caixa 24) encolhia no botão de 16 px da fila
   em pontinhos de ~1 px cinza, ao lado do «··» do pontilhado. As 3 suspeitas do handoff anterior (`Selected::Both`, corte do layout,
   filtro) foram DESCARTADAS pelo explorador. Cura: `board_bar_look::more_dots(r)` — três pontos CHEIOS, passo = 1/4 do botão, ponto =
   vão = passo/2 (a 1.ª cura, medida pelo miolo de 8 px, colava os pontos num traço — apanhado na foto). Vale para `MoreHeads` e
   `MoreShapes`. Gate `the_more_dots_read_as_three_dots_on_both_bar_buttons` (16 e 24 px; mutação ponto>vão: vermelho).
   ⚠️ O overflow da `tool_bar` (chrome partilhado, `tool_bar.rs:103`) continua com o `more-horizontal` fino — não é do quadro, não mexi.
3. **Cena 24** traz agora duas caixas From/To com uma seta presa às duas, seleccionada ao abrir (antes o dono desenhava a seta); a lição 4
   aponta-a; a lição 6 diz onde estão as abas («the row at the very top (Scene, Board tweaks, +)»). Fotos
   `w23_fotos/smoke19_cena24_{1366x768,1930x1040}.png`.

## 2. B5(1) — arrastar o segmento do MEIO da seta cotovelo (FEITO)

Commits: `10c11f8be2` plano · `e2428e5898` modelo+rota · `84989ee59e` gesto · `c39a03eaa7` Excalidraw+cena 25+W8 · `461bf1b038`
W9/W10+docs · `a83d27b7e4` fmt. Oráculos CORRIDOS por script em `ferramentas/elbow_oracle/` (Excalidraw 0.18.1, tldraw 5.5.2; Miro só
documentação). Escolhida a lei do tldraw (`elbowMidPoint`: fracção do vão entre recuos); **recusado** o fixo absoluto do Excalidraw
(atravessa a caixa — foto `excalidraw_C3_*` — e perde o fixo na recta).

- **Modelo:** `Connector::mids: Vec<Option<f64>>` (`#[serde(default)]`), um por trecho, alinhado com `waypoints`; portas únicas `set_mid`,
  `insert_waypoint`, `remove_waypoint` (`wire.rs` passou a usá-las).
- **Formato do quadro 13 → 14** (`FORMAT_VERSION`), `legacy_v13.rs` (`ConnectorV13`/`KindV13`/`ElementV13`). ⚠️ Os leitores 9–12 liam a seta
  com a struct de HOJE e partiriam (postcard é posicional): passam agora pela `ConnectorV13`/`KindV13`. Fixture `fixtures/format_v13.bin`
  do build `10c11f8be2`. Suíte de `ph2d-board-model`: 128 verdes.
- **Fio ao vivo:** `Field::Waypoints` leva `(waypoints, mids)` — um registo; `PROTOCOL` 7 → 8 (`ph2d-board-sync/session.rs`).
- **Rota:** `ph2d-board-route/src/elbow_mid.rs` `place` depois do roteador (`ph2d-vec-connect` sem diff); `Routed::elbows:
  Vec<Option<ElbowMid>>`; a chave da `RouteCache` leva `mids`.
- **Gesto:** `ph2d-board-edit/src/elbow_drag.rs` (`Gesture::Slide`, `WireHandle::Segment`, duplo clique larga, cursor ↔/↕ por `cursor_at`);
  pílula deitada ao longo do segmento (`ph2d-board-render/src/overlay.rs`). `gesture.rs` estava a 699 linhas: `is_note`/`same_pose`
  foram para `gesture_end.rs` (agora 696 — no tecto, não cresça).
- **Excalidraw:** `ph2d-board-excalidraw/src/elbow.rs` lê `fixedSegments` (o nosso segmento do meio) e escreve-o; doc 07 actualizado.
  Oráculos W8, W9, W10 regravados (diff só `mids: []` e um `fixedSegments: null`).
- **Cena 25** `PH2D_BOARD_SMOKE=25` (`smoke_w23_elbow.rs`, ROUTERS `max_level` 25): Start/Finish (caixas 0,0 e 400,300, 100×100), seta
  cotovelo seleccionada, 4 lições. Fotos `w23_fotos/cena25_cotovelo_{1366x768,1930x1040}.png`.
- **Gates** (todos com mutação vermelha registada): `a_dragged_middle_follows_its_fraction_when_a_box_moves`,
  `a_fraction_unused_by_a_straight_line_comes_back_with_the_z`, `the_middle_never_enters_a_box`,
  `a_waypoint_split_keeps_each_other_leg_its_own_middle`, `a_format_13_file_still_opens_with_its_elbow_unpinned`,
  `the_waypoints_field_carries_the_middles`, `dragging_the_middle_of_an_elbow_moves_it_across_in_one_step`,
  `a_double_click_lets_the_middle_go`, `the_excalidraw_fixed_middle_comes_in_as_our_fraction_and_goes_back`,
  `an_unpinned_elbow_writes_no_fixed_segments`, `the_w23_elbow_scene_opens_with_the_middle_bar_in_reach` (pelo rato real da shell: 250 → 310).
- ⚠️ `the_middle_never_enters_a_box` passou sob a mutação «ignorar a fracção» (o meio de nascença já fica fora); a que o morde é tirar o
  clamp — NÃO corrida.

## 3. O que um merge pode partir (para o integrador)

- `FORMAT_VERSION` 14 e `PROTOCOL` 8 são números que SOMAM entre linhas — só esta linha os mexe; reconte, não escolha.
- `CLAUDE.md` §5 (Quadro) diz `PH2D_BOARD_SMOKE=1..22` → conta-se **1..25** (23 = cena do lado de quem entra, 24 e 25 desta linha).
- `Field::Waypoints` mudou de codificação: outra linha que o toque no fio reabre os gates do `ph2d-board-sync`.
- A tabela de itens da barra NÃO mudou neste bloco.

## 4. Prova de fecho de onda

- `cargo fmt` aplicado nas 9 crates; `PH2D_SUITE=fecho cargo clippy -p ph2d-board-{model,route,sync,edit,render,excalidraw} -p ph2d-app-board
  -p ph2d-editor-core -p ph2d-i18n --all-targets -- -D warnings` limpo; testes por nome verdes.
- A suíte `nextest-impacted` NÃO correu nesta onda (corre ao fechar a linha). Binário smoke compilado; 2.ª corrida:
  «Finished `smoke` profile [optimized] target(s) in 0.27s» sem Compiling.

## 5. ABERTO (nesta ordem)

1. **B5(2)** rótulo ao longo da linha (hoje a meio da rota, horizontal; `ph2d-board-render/src/lib.rs:~520`). Miro: sem notas no repo —
   pesquisar/medir PRIMEIRO; campo de formato ⇒ formato 15 (mesmo desenho do `legacy_v13.rs`, leitor 14 por struct congelada).
2. **B5(4)** desfazer as operações de aba (hoje apagar um quadro pergunta antes; desfazer não existe).
3. Bloco 4 (B6, B7, B10, B11 — MEDIR antes) · bloco 5 (B9, A11) · bloco 6 (B12, B13, B14 com `/pd-feature`).
4. Perguntas ao dono sem resposta: YouTube Premium; «Rascunho/Final» e a letra importada.
5. Smoke do dono por fazer: cenas 24 e 25 (§6).

## 6. Smoke ao dono (curto)

1. `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-MiroClone && PH2D_BOARD_SMOKE=25 ./target/smoke/ph2d-host-desktop` — arrastar a
   barrinha no meio da linha vertical para o lado; arrastar a caixa Finish; duplo clique na barrinha; Ctrl+Z. Deu errado se a linha
   atravessa uma caixa, a barrinha não segue a caixa, ou o Ctrl+Z desfaz em dois passos.
2. `PH2D_BOARD_SMOKE=24` (mesmo `cd`) — os «…» depois de cada grupo de pontas na barra da seta seleccionada (três pontos nítidos);
   pairar/seleccionar «Ideas» e ver o «−» fora da pega de redimensionar. Deu errado se o «…» é um risco cinza ou o «−» cobre a pega.
