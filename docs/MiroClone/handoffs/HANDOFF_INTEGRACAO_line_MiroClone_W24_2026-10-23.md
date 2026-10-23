# INTEGRAÇÃO — `line/MiroClone` W24: a barra de estilo foge do chrome · riscas de longe · doc 17 §6 fechado · a menção é uma palavra (2026-10-23)

> Leitor: o integrador. Antecessor no `main`: [W23](HANDOFF_INTEGRACAO_line_MiroClone_W23_2026-10-22.md). Primeira metade da onda (mecanismo,
> medidas, smoke): [HANDOFF_CONTINUACAO_W24_BARRA_RISCAS](HANDOFF_CONTINUACAO_W24_BARRA_RISCAS_2026-10-23.md) §1–§5 — NÃO re-derivada aqui.
> **A linha PAROU aqui: não integrou nem fez push.**

## 0 — Base e commits

- Merge-base = `main` `972f411a6f`. Confira: `git log --oneline 972f411a6f..HEAD | wc -l` = 15; `git diff --stat 972f411a6f..HEAD | tail -1` = 55 ficheiros, +2 360 / −77 (sem este handoff).
- `git log --oneline main..HEAD` (do mais novo ao mais velho):
  `9eccc6fe72` flake `opening_the_room_says_how_to_open_the_firewall` · `329408dd5a` fecho: tecto de LOC + CLAMP-OK · `efb442f92d` sem vão junto da seleção ·
  `0660e973d3` menção = palavra inteira · `46960a1876` clippy `clone_on_copy` · `547e930934` doc 17 §6 num bloco · `bb2d017375` menção = pessoa (Backspace) ·
  `41a5897f10` docs: nascença 0 fechado pelo dono · `0e04d80ce7` handoff de continuação · `9952261792` fmt · `17fccb4c9c` riscas de longe ·
  `bd06efd3e7` doc 17 §6 por grep · `fbceb7352e` cena 29 · `a9cfdddbc0` último recurso da barra · `6ad11719f8` a barra foge de todo o chrome.

## 1 — Feito

**(a) 1.ª metade** (`6ad11719f8`…`0e04d80ce7`; detalhe no handoff de continuação): `float_bar::strip_rects_clear_of` (a barra de estilo foge de gavetas,
painéis e barra de ferramentas; `Nav::floats` novo; `keep` em `style_rects_of`); cena 29 (`PH2D_BOARD_SMOKE=29`, `smoke_w24_bars.rs`);
`fotografa_cena.sh` aceita a tela virtual; riscas de longe (`Detail::hatch_final_below_px` = 11, `hatch_tone`, régua `measure_hatch_far.rs`, doc 16 §5);
doc 17 §6 trocado por tabela propriedade → gate real. ABERTO 3 da W23 (nascença 0) fechado pelo dono: não há ficheiros antigos, formato fica 17.

**(b) 2.ª metade (esta janela).**

1. **Lacunas do doc 17 §6 num BLOCO** (`547e930934`): 6 «nenhum» + 5 «parcial» viraram gates reais.
   - B13 `every_recognised_shape_lands_on_an_element_that_draws_it` — porta nova `Editor::smart_landing` (`pub(crate)`, `ph2d-board-edit/src/smart.rs`),
     extração pura de `smart_show`; `a_line_with_one_end_on_a_box_binds_only_that_end`; `ctrl_z_gives_every_family_its_stroke_back`;
     `the_board_has_no_recognition_thresholds_of_its_own` (censo). Os 4 vivem em `ph2d-board-edit/src/smart_law_tests.rs` (filho de `smart_tests`).
   - B13 oráculo `the_pen_held_still_is_their_draw_to_shape_but_for_the_written_divergences` (`ph2d-board-excalidraw/src/fidelity_bloco6_shape_tests.rs`):
     80 traços de `saidas/bloco6_traco.next.json`, 16 ms/600 ms como a corrida deles: **54 iguais · 12 «nenhum» deles = tinta nossa · 14 divergências escritas**
     (rotação mantida 2, o nosso triângulo 3, paralelogramos/trapézio/polígonos 6–8 = tinta limpa 7, seta de duas pontas 1, o retângulo 8:1 1 = a lei do lápis do
     Vector não o reconhece: sonda `recognise` cru+amostrado = `None`; o Xournal++ também não). Sem segurar = tinta crua 80/80. Adiciona **dev-dependency
     `ph2d-text`** em `ph2d-board-excalidraw/Cargo.toml` → `Cargo.lock`.
   - B12 `our_lasso_takes_exactly_what_is_ninety_percent_inside_where_theirs_needs_all` (`fidelity_bloco6_tests.rs`): 6 casos com a fração gravada, 3 separam as
     leis; o alvo MIRO = 0,9 está escrito À PARTE de `LASSO_COVER` (a 1.ª versão lia `LASSO_COVER` e sobreviveu à mutação 0,85).
   - B14 `a_mention_arrives_live_and_stays_in_the_file` (`ph2d-board-sync/src/replica_mention_tests.rs`, filho de `replica_tests`): duas réplicas, gravar/reabrir por
     `BoardSet::to_bytes`/`from_bytes`, não lida até abrir, a resposta é para o autor.
   - B15 `a_copy_to_another_board_keeps_its_arrows_frames_and_map` (`paste_elsewhere_tests.rs`; as duas portas, Ctrl+V e levantar/pousar; destino com os MESMOS ids) e
     `the_menu_sends_through_the_scene_chooser_and_new_board_creates_one` (`board_send_tests.rs`; copiar e mover, «New board»).

2. **DEFEITO curado — a menção era letras** (`bb2d017375`): Backspace deixava «@Ru» sem ninguém chamado (doc 17 §2.3 prometia bloco atómico).
   Cura: `WidgetStore::arm_atomic_words(id, words, find: WordFinder)` + `delete_atomic_word` em `ph2d-editor-core` `interaction/state` (`graph_ops.rs`; campo
   `atomic_words` em `mod.rs`; init em `store_core`), chamado PRIMEIRO no ramo Backspace de `interaction/dispatch/key.rs`; armado a cada pintura e ao escolher por
   `board_team_mention::arm_called` (chamados + menções da mensagem editada). Só Backspace: o Delete (para a frente) não é tratado por NENHUM `TextInput` do programa.
   Gate `backspace_takes_a_mention_whole_and_typing_before_it_carries_it` (`board_mention_tests.rs`).

3. **Auditoria do fecho (lente independente) — 2 defeitos, curados red-first.**
   - (i) `0660e973d3` menção = PALAVRA inteira: porta única `ph2d_board_model::label_spans` (exportada de `lib.rs`), usada por `Message::keep_mentions` (no `main` era
     `text.contains`: «@Ruiva» mantinha o Rui chamado), por `board_team_paint::mention_spans` e pelo Backspace atómico (ganha o rótulo mais longo que contém o cursor).
     Gate `a_mention_is_a_whole_word_of_the_text` (`team_tests.rs`) + caso «@Ruiva». ⚠️ pares de builds diferentes no MESMO protocolo podem discordar de um texto como
     «@Ruiva» (`keep_mentions` calcula-se localmente) — sem sobe de protocolo, aceito.
   - (ii) `efb442f92d` `strip_rects_clear_of`, 2.º fallback: somava `+gap` nas alturas «rente a um painel» enquanto `free_x` mede em `keep_pad` (= `Xs` + gap) ⇒ o rente
     tocava SEMPRE e a barra fugia para o bordo (medido y 1984 numa janela de 2000, seleção em 1000). Agora `keep_pad` e dois degraus: 1.º evita também a seleção, 2.º por cima dela
     (o chrome nunca; a janela minúscula precisa do 2.º). Gate `with_no_room_by_the_selection_the_strip_takes_the_nearest_free_height` (`float_bar_tests.rs`).

4. **Fecho:** clippy `clone_on_copy` (`46960a1876`); tecto de 700 LOC do workspace por CORTE (`329408dd5a`: testes partidos em módulos filhos; o registo das gavetas passou
   para `board_bar_layout::float_drawers` → `board_bar.rs` 694); `// CLAMP-OK` no clamp de `free_x` (o filtro garante `lo ≤ hi − w`); flake `opening_the_room_says_how_to_open_the_firewall`
   registado (`9eccc6fe72`: tabela de `FLAKES_DE_CARGA.md` + faixa isolada do nextest; vermelho a `load` 70–78, sozinho 3/3 a 0,50 s com `load` 9–10).

## 2 — Contadores como DELTA e contratos

- `FORMAT_VERSION` 17 · `PROTOCOL` 11 · `PROJECT_SCHEMA` 191 — **DELTA zero**. §6 congelados intocados. Prova:
  `git diff 972f411a6f..HEAD -- crates | grep -E '^[-+].*(FORMAT_VERSION|PROTOCOL|PROJECT_SCHEMA|NodeOp|NodeManifest|VectorOp)'` não devolve nada; nenhum ficheiro de
  `ph2d-nodegraph` / `ph2d-vector` / `ph2d-tool-*` / `ph2d-imageio` mudou.
- ADR próximo livre: **0189** (nenhum escrito). Cenas de smoke: **1..29** (`max_level: 29` em `ph2d-app-board/src/smoke.rs`; era 28). `Cargo.lock`: +1 aresta (`ph2d-text` → dev-dep de `ph2d-board-excalidraw`).
- i18n novas: `board.smoke.w24.*` (`ph2d-i18n/src/board_w24.rs`). `Detail`: +2 campos (`hatch_final_below_px`, `hatch_tone`).

## 3 — O que um merge pode partir

Da 1.ª metade (continuação §2): `strip_rects_fitting` não existe mais (→ `strip_rects_clear_of`); `style_rects_of` ganhou `keep`; `Nav::floats` novo; `paint_notice` devolve `Rect`;
`Detail` com 2 campos novos — literais completos em `measure_detail_cost.rs` e `ph2d-board-export`; `ROUTERS` `max_level` 29; `measure_bloco4` expõe `pub(super)`; **posições da
barra de estilo mudam onde há chrome** — gates de posição de outras ondas reconferem. Desta janela:
- `Editor::smart_landing` (`pub(crate)`, `ph2d-board-edit/src/smart.rs`): quem mexer em `smart_show` mexe nas duas.
- `WidgetStore::arm_atomic_words` e o tipo `WordFinder` (`pub`), campo novo `atomic_words`; o ramo Backspace de `interaction/dispatch/key.rs` chama `delete_atomic_word` ANTES do resto.
- `label_spans` exportada de `ph2d-board-model`; **semântica de `Message::keep_mentions` = palavra inteira** (antes substring).
- `strip_rects_clear_of`: ordem dos fallbacks (`keep_pad`, dois degraus).
- **Ordem de pintura:** alerta e barra do vídeo pintam ANTES da `board_bar` ⇒ barra de ferramentas e gavetas desenham POR CIMA deles (intencional; a auditoria assinalou — ver ABERTO 2).
- Dev-dep `ph2d-text` em `ph2d-board-excalidraw` (`Cargo.toml` + `Cargo.lock`: conflito aditivo).
- Testes novos vivem em módulos FILHOS (`smart_law_tests`, `replica_mention_tests`, …): um merge que reponha os pais inteiros estoura o tecto de 700 LOC.

## 4 — Provas de mutação

- **Lote 1 (12), todas vermelhas:** altura da elipse · `same_outline` do polígono · `tick` em repouso desde o início · `HOLD_SLOP`→12,0 · `smart_wire` ponta em `at(a)` ·
  `keep_drawn` de um só passo · `remap` sem efeito · `LASSO_COVER` 0,85 (SOBREVIVEU à 1.ª régua → régua corrigida → vermelha) · `serde(skip)` em `mentions` ·
  sem `delete_atomic_word` · palavras armadas vazias · escolhedor `except None`.
- **Lote 2 (4), todas vermelhas:** `pad`→`gap` · sem degrau da seleção · Backspace por substring · `keep_mentions` por `contains`.
- Mais as mutações da 1.ª metade (continuação §3).

## 5 — Bateria do fecho

| o quê | resultado |
|---|---|
| `cargo fmt` | ✓ |
| clippy workspace `--all-targets --features ph2d-spike/bevy_ecs -D warnings` | ✓ |
| `file_loc_caps` 4/4 · `arch_safe_clamp_only` 2/2 · `architecture_workspace_file_loc_cap` 2/2 · `the_shell_only_shrinks` 1/1 · `o_claude_md_cabe_no_orcamento` 1/1 · `architecture_toda_flake_de_carga` 2/2 | ✓ |
| `nextest-impacted`, corrida 1 (prazo 1800 s, `load` 70–78) | 19 396/19 422 corridos, 19 387 passaram, **3 falharam, 6 timeout** — os 9 passam sozinhos |
| `nextest-impacted`, corrida FINAL sobre `9eccc6fe72` (prazo próprio 5400 s, `load` 6→59; 1 145 s) | **19 422 corridos, 19 421 passaram**, 1 timeout: `ph2d-panel-painter-layers` `seam_estado_escondido::cada_interruptor_que_neutraliza…` (já na faixa pesada do FLAKES_DE_CARGA; sozinho 69,98 s; zero linhas dessa crate no diff) |

As 9 da corrida 1: falha `opening_the_room_says_how_to_open_the_firewall` (registada como flake, §1.4) e as duas do SIGTERM abaixo (contadas como falha pelo nextest); timeouts `ph2d-app-components` `as_duas_barras_do_igual…`,
`ph2d-app-flip` `luz::testes` ×2, `ph2d-app-light2d` `a_borda_nao_tem_pontos_pretos` e `a_sombra_nitida_nao_tem_escada`, `ph2d-panel-painter-layers` `seam_estado_escondido…`;
SIGTERM `ph2d-tool-painter` `midias_de_fabrica` / `presets_de_fabrica`. Todos carga (ver [FLAKES_DE_CARGA](../../DevOps/FLAKES_DE_CARGA.md)).

## 6 — Smoke (o dono; da pasta da LINHA, antes da integração)

Os passos são os do §5 do handoff de continuação, repetidos:

1. `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-MiroClone && cargo build --profile smoke -p ph2d-host-desktop && PH2D_BOARD_SMOKE=29 ./target/smoke/ph2d-host-desktop`
2. Abre o quadro «Bars». À esquerda, a gaveta das formas (a grelha de formas ao lado da barra de ferramentas) está aberta; a caixa de baixo à esquerda está selecionada e a barra
   dela (cores, traço, letra) aparece por cima, À DIREITA da gaveta, sem a tapar. Em cima à direita está aberto o painel «Frames».
3. Na barra da esquerda abra, uma de cada vez, Nota, Caneta e Modelos: a barra da caixa afasta-se da gaveta aberta. (Atenção: com Nota ou Caneta na mão um clique no quadro
   desenha — volte à seta antes de selecionar outra coisa.)
4. Clique na seta da barra da esquerda, selecione a caixa da direita, e abra em baixo à direita os Comentários e o Minimapa: a barra da caixa não fica por cima de nenhum painel.
5. Estreite muito a janela: a barra parte-se em mais linhas e nunca passa por cima da barra de ferramentas.
6. Bónus (riscas de longe): numa forma, escolha um preenchimento às riscas e afaste-se muito (roda do rato): de longe a forma continua a ler-se riscada/tom igual, sem piscar ao cruzar
   o ponto em que muda.
7. Deu errado se: a barra de uma caixa tapa uma gaveta, um painel ou a barra de ferramentas; sai da janela; ou de longe as formas às riscas mudam de cor de repente ao aproximar/afastar.

(A menção como palavra inteira tem gate, sem cena nova: no quadro de equipa, escrever «@Rui», Backspace apaga-a toda.)

## 7 — ABERTO depois da linha

1. **Smoke do dono da cena 29** (§6) — se chegar, mecanismo antes de sintoma.
2. **Ordem de pintura:** alerta e barra do vídeo ficam por BAIXO da barra de ferramentas e das gavetas. Além disso os flyouts ancorados à barra de estilo (formas, pontas de seta,
   facilitar) NÃO estão em `keep` e podem sobrepor uma gaveta aberta (achado da auditoria; não curado).
3. Herdados: letra que não nasce à escala do ecrã; README do oráculo (o W14 nomeia um teste apagado); pendentes do §4.2 da lista-mãe da W23.
