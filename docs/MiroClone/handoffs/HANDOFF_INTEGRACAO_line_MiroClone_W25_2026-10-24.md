# INTEGRAÇÃO — `line/MiroClone` W25: o que se pinta depois das gavetas não tapa o chrome (2026-10-24)

> Leitor: o integrador. Antecessor (também por integrar, na MESMA linha): [W24](HANDOFF_INTEGRACAO_line_MiroClone_W24_2026-10-23.md) —
> integra-se a linha inteira (W24 + W25) de uma vez. **A linha PAROU aqui: não integrou nem fez push.**

## 0 — Base e commits

- Merge-base = `main` `972f411a6f` (o `main` não andou durante a W25). A linha: 23 commits à frente; os da W25 sobre o fecho da W24 (`9f0098d8d5`):
  este handoff · `ff92cd26c5` flakes `macro_estado_inicial` ×2 · `2af3a09822` gates dos menus em três · `412a863037` fecho: apresentação + tecto de LOC · `62e6617e98` README do oráculo · `06e0459bd4` a regra.
- `git diff --stat 9f0098d8d5..HEAD`: 16 ficheiros, +796 / −186 — todos em `crates/ph2d-editor-core/src/screens/hero/` menos o README do oráculo e este handoff.

## 1 — Feito

Os dois achados da auditoria do fecho W24 (§7.2 dele), confirmados no código E pelo `hit_index` antes de curar (red-first, um bloco só):

| caso | medido antes | cura |
|---|---|---|
| menus da barra de estilo («Forma», pontas, carimbos/agrupar) | abriam por baixo do BOTÃO: com a barra em 2+ linhas tapavam as outras linhas dela (Opacity, Look, Round, Font…, 638 encontros na grelha de 216 casos); ao canto, o minimapa | `board_bar_layout::MenuRoom::origin`: por baixo da barra INTEIRA, senão por cima, fora do chrome (`keep` + a barra curta); sem vão, rente ao botão como antes |
| aviso do topo (o com botão W23 B15 e o da recusa) | pintado DEPOIS dos painéis da direita e sempre no topo: tapava o «Frames» a 1024/800/600 | `board_alert::banner_rect` (as duas portas usam-na): a altura mais alta livre do chrome e da barra curta, centrado à direita dela; sem lugar nenhum, no topo, por cima |
| barra do vídeo e frase «instale um navegador» | pintadas ANTES da barra curta e das gavetas: o Play ficava por baixo (o clique ia para a ferramenta) com o vídeo à esquerda | `board_video_bar::by_video`: por baixo/por cima do vídeo, fora do chrome, e encolhe a barra do tempo (largura mínima) antes de desistir |
| (auditoria do fecho W25) a barra do vídeo na APRESENTAÇÃO | fugia do chrome do quadro pintado por baixo da sala (invisível): subia para cima do vídeo (y 347 com o vídeo até 644) | `board_video_bar::chrome` devolve nada com `team.show` aberto |

**Uma regra só:** `float_bar::rect_clear_of(area, keep, (w, h), ys, cx)` (o vão sem `keep` mais perto de `cx`, na 1.ª altura de `ys`) e
`float_bar::heights_beside` — a barra de estilo (`strip_rects_clear_of`) passou a usá-las também, sem mudar de comportamento.
**Ordem de pintura nova** (`hero/paint.rs`): `board_nav` → `board_bar::paint_tools` (barra curta + gavetas, saiu de `board_bar::paint`
para `board_bar_layout.rs`) → `board_alert` → `board_video_bar` → `board_bar::paint` (aviso da recusa, barra de estilo, menus). Cada um
lê `fit::painted` = o que já se pintou, e por isso foge dele; o `hit_index` é last-wins, por isso nos recursos declarados (sem lugar)
ganha o clique quem está por cima.

**Herdados do W24 §7.3 — todos fechados:** a letra à escala do ecrã, `textAlignVertical`, a letra importada e o YouTube Premium foram
fechados pelo dono em 22/10 ([W23 §7](HANDOFF_INTEGRACAO_line_MiroClone_W23_2026-10-22.md)) — o W24 listava-os por engano; os nomeados do
bloco 5: o caminho de longe das riscas = W24, o «menor erro» = inexprimível (doc 07 §3). README do oráculo: já explicava o teste
apagado do W14; faltava a W15 na lista das entradas a regravar (agora QUATRO).

## 2 — Contadores e contratos

- `FORMAT_VERSION` 17 · `PROTOCOL` 11 · `PROJECT_SCHEMA` 191 · ADR livre 0189 · cenas `max_level` 29 — **DELTA zero**. Prova:
  `git diff 972f411a6f..HEAD -- crates | grep -E '^[-+].*(FORMAT_VERSION|PROTOCOL|PROJECT_SCHEMA|NodeOp|NodeManifest|VectorOp)'` vazio;
  nenhum ficheiro de `ph2d-nodegraph`/`ph2d-vector`/`ph2d-tool-*`/`ph2d-imageio`/`Cargo.*` na W25. Sem i18n nova.
- Foundational: só `ph2d-editor-core` (`screens/hero/`), aditivo — duas funções `pub` novas em `float_bar` (`rect_clear_of`, `heights_beside`)
  e `board_bar::paint_tools`; nenhum consumidor fora de `screens/hero/` (grep em `crates/` e `shells/`).

## 3 — O que um merge pode partir

- **Assinaturas:** `board_video_bar::layout(area, video, (time_w, nav), keep)` e `install_rect(area, video, text_w, keep)`; `board_alert::layout(ts, (area, keep), …)`;
  `organize::paint_notice(…, (area, keep), key)`; `heads_rects`/`reshape_rects` passaram a `pub(super)` e recebem `&MenuRoom` (o `pub use shapes::reshape_rects` saiu);
  `paint_grid`/`paint_reshape_grid`/`fac::paint_flyout` recebem `&MenuRoom` em vez de `area` + `bar`.
- **`board_bar::paint` partida em dois:** quem chamar só `board_bar::paint` deixa de ter a barra curta e as gavetas — tem de chamar `paint_tools` antes (como `paint.rs`).
  `PAINTED_ON` limpa-se em `paint_tools`, `STYLE_BAR` em `paint`.
- **Posições mudam onde havia sobreposição:** menus abaixo da barra INTEIRA (antes do botão), o aviso desce quando o topo está ocupado, a barra do vídeo pode
  ficar mais estreita que o vídeo. Gates de posição de outras ondas reconferem.
- O teste `without_a_browser_each_video_says_to_install_one_below_it` limpa `nav.floats` antes de chamar `paint_install` fora da pintura (senão a frase fugia de si mesma).

## 4 — Gates e provas de mutação

Gates novos (todos VERMELHOS antes da cura): `the_reshape_menu_never_covers_the_chrome` · `the_heads_menu_never_covers_the_chrome` ·
`the_stamps_menu_never_covers_the_chrome` · `the_top_notice_is_never_under_the_chrome` (`board_bar_overlay_tests.rs`, filho de `place`) ·
`the_video_bar_is_never_under_the_chrome` (`board_video_overlay_tests.rs`, filho de `video`) · `in_the_presentation_the_video_bar_ignores_the_hidden_board_chrome`
(`board_team_flow_tests.rs`). A régua do aviso NÃO lê a conta do produto: o lugar livre procura-se à força (`free_spot`, de 4 em 4 px, folga `Spacing::Lg`,
contra o registado E o pintado) — tapar só passa quando nenhum lugar do quadro leva o aviso (a 800/600 com os painéis da direita, a frase da recusa de 344 px).

Mutações — 1.ª ronda (grelha de 216 casos, por um agente): 6/6 vermelhas — M1 menu ancorado ao botão · M2 menus sem `keep` · M3 aviso sem `keep` · M4 barra do vídeo
sem `keep` · M5 sem encolher · M6 `paint_tools` depois da barra do vídeo. 2.ª ronda, sobre a grelha reduzida (`2af3a09822`): M1, M2, M3 e M7 (apresentação sem o
`show.is_some()`) — 4/4 vermelhas.

Tempo dos gates novos (nextest, load 100–150): forma 40 s · pontas 35 s · carimbos 31 s · aviso 48 s · vídeo 4,4 s · apresentação 0,6 s. O gate dos menus em
216 casos levava 557–655 s sozinho e deu TIMEOUT (180 s) na suíte da 2.ª corrida — partido em três na grelha que apanha as mutações, e o clique vai ao
rectângulo registado (o `T::click_bar` varre o ecrã de 2 em 2 px).

## 5 — Bateria do fecho

| o quê | resultado |
|---|---|
| `cargo fmt --all --check` | ✓ |
| clippy workspace `--all-targets --features ph2d-spike/bevy_ecs -D warnings` (sobre `412a863037`) | ✓ |
| `file_loc_caps` 4/4 · `arch_safe_clamp_only` 2/2 · `architecture_workspace_file_loc_cap` 2/2 (curado por corte: `board_video_tests.rs` 707 → 673) · `the_shell_only_shrinks` 1/1 · `o_claude_md_cabe_no_orcamento` 1/1 · `architecture_toda_flake_de_carga` 2/2 | ✓ |
| auditoria independente (2 lentes: colocação · costura/ordem/estado) | 1 achado (a apresentação) — curado red-first; lente 2 sem achado |
| `nextest-impacted`, 2.ª corrida (`412a863037`, load 284 → 300–490 → 16, 3 569 s) | 19 411/19 426 passaram, 15 TIMEOUT: o gate dos menus (desta linha — curado em `2af3a09822`) e 14 em crates sem linhas no diff (light2d ×10, components ×1, flip ×2, painter-layers ×1 — este sozinho passou em 130 s a load 513) |
| `nextest-impacted`, 3.ª corrida (`2af3a09822`, load 220 → 13, 1 269 s) | **19 424/19 428**; os 14 TIMEOUT da 2.ª não repetiram. 4 vermelhos, todos em crates com ZERO linhas no diff e todos verdes sozinhos: `ph2d-panel-painter-layers` `macro_estado_inicial::cada_controlo_mexido_…` e `…::o_estado_inicial_leva_…` (TIMEOUT 180 s; sozinhos 82 · 81 s com a compilação) → **registados** (tabela do FLAKES_DE_CARGA + faixa de 6 × 60 s do irmão `seam_estado_escondido`, que deu TIMEOUT a 360 s e passou sozinho em 82 s); `ph2d-host-code-scenes` `a_cena_3_conduzida_passo_a_passo` (a espera PRÓPRIA de 30 s do passo 3, `Wait(ListHas("set"))`; sozinho ✓ 95 s) — não registado: a faixa do nextest não lhe alarga a espera interna; se repetir, é da linha do CodeEditor |
| `architecture_toda_flake_de_carga` depois do registo | ✓ 2/2 |

## 6 — Smoke (o dono; da pasta da LINHA, antes da integração)

1. `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-MiroClone && cargo build --profile smoke -p ph2d-host-desktop && PH2D_BOARD_SMOKE=29 ./target/smoke/ph2d-host-desktop`
2. Abre o quadro «Bars». Clique na seta da barra da esquerda e selecione a caixa da direita. Estreite a janela até a barra da caixa (cores, traço, letra) se partir em duas linhas.
3. Na barra da caixa, clique no botão «Forma» (o ícone da forma): a grelha das formas abre por BAIXO da barra inteira (ou por cima, se em baixo não houver lugar) — nunca por cima da 2.ª linha da barra.
4. Com o painel «Frames» aberto em cima à direita e a janela estreita: os avisos do quadro (por exemplo ao enviar algo para outro quadro, o aviso com «Undo») descem para onde há lugar em vez de taparem o painel.
5. (Com internet) Cole um link do YouTube (Ctrl+V), clique no play, abra a gaveta das formas (na barra da esquerda) e arraste o vídeo para junto dela: a barra do vídeo (play, tempo, volume) fica sempre clicável, fora da barra da esquerda e da gaveta — se não couber, fica mais curta.
6. Deu errado se: um menu da barra tapa uma linha da própria barra ou um painel; o aviso tapa o «Frames» com espaço livre ao lado; o play do vídeo fica por baixo da barra da esquerda.

## 7 — ABERTO depois da linha

1. Smoke do dono do §6.
2. O aviso do topo numa janela onde nenhum lugar o leva (800 px com os painéis da direita abertos, a frase da recusa de 344 px) fica no topo por cima do painel:
   a cura completa é partir a frase em duas linhas (não há ajudante de quebra de linha no `ph2d-editor-core`) — não pedido.
3. `a_cena_3_conduzida_passo_a_passo` (`shells/code-scenes`): a espera interna de 30 s do passo 3 não chega sob a suíte — para a linha do CodeEditor, se repetir.
