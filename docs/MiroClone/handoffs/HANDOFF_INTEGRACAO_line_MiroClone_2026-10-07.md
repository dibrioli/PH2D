# HANDOFF DE INTEGRAÇÃO — `line/MiroClone` → `main` (2026-10-07)

> Para o **agente integrador** (DIRETRIZ §1.5.3–1.5.4, §1.5.9). A linha FECHOU e PAROU: não integra
> nem pusha. O detalhe de cada onda (leis, fontes, recusas medidas, o que custou) vive nos handoffs de
> continuação — [W0b](HANDOFF_CONTINUACAO_W0b_2026-10-05.md) ·
> [W0b2](HANDOFF_CONTINUACAO_W0b2_2026-10-05.md) · [W1](HANDOFF_CONTINUACAO_W1_2026-10-06.md) ·
> [W2](HANDOFF_CONTINUACAO_W2_2026-10-06.md) · [W3](HANDOFF_CONTINUACAO_W3_2026-10-06.md) ·
> [W4](HANDOFF_CONTINUACAO_W4_2026-10-07.md) — e no plano [`../02_plano.md`](../02_plano.md) (§3 estado
> por onda, §6 recusas medidas).

## 1. Identidade

| | |
|---|---|
| branch | `line/MiroClone` · worktree `Worktrees/line-MiroClone` |
| HEAD | ver `git rev-parse line/MiroClone` (o último commit é o deste handoff; o anterior, `b8c75d74c`, é o do portão) |
| base (merge-base) | `a46c4c200` — **o `main` não andou** desde o fork (`git rev-parse main` = `a46c4c200` em 07/10) ⇒ integra por `--ff-only` se ninguém aterrar antes |
| commits | 68 (`git log --oneline a46c4c200..line/MiroClone`) |
| diff | 311 ficheiros, +45 951 / −407 |
| smoke | W0b, W1, W2, W3, W4 **aprovados pelo dono** (W4: 07/10, «smoke OK» depois das correcções da barra) |

O que a linha É: o **Quadro** (um Miro/Excalidraw dentro do editor) — documentos em ABAS na barra de
cima (nunca objectos da cena), formas de fluxograma com texto, setas presas às formas (roteador do
Vector reusado), notas adesivas com as leis do Miro, caneta/marcador/borrachas/laser, e o botão
«Rascunho ↔ Final» (traço à mão com as opções do Excalidraw medidas, letra à mão Virgil).

## 2. Foundational / partilhado tocado — e porque é aditivo

| onde | o quê | aditivo? |
|---|---|---|
| **crates novas** (8) | `ph2d-board-{model,geom,layout,edit,render,route,rough}`, `ph2d-app-board` (família) | sim — drop-crates |
| `shells/desktop` (17 ficheiros) | `ProjectFile.boards: Vec<u8>` no FIM + `PROJECT_SCHEMA 183 → 184` (`project.rs`, `project_schema.rs`, save/load/migrate e testes); o rato e o teclado do quadro (`input_dispatch.rs` +55, `input_dispatch/keyboard_board.rs` novo, `keyboard.rs` +5); `undo_route.rs` (`UndoOwner::Board` primeiro); `init.rs` (+2) | sim — campo no fim, ramos novos que só actuam com uma aba de quadro activa |
| `ph2d-editor-core` (36) | `documents.rs` (as abas e o `BoardLive`), `screens/hero/board_*.rs` (barra, painéis, vista, teclas), `document_tabs*.rs` (abas), `paint.rs` (o quadro pinta-se por cima da área de desenho; as sobreposições da cena saíram para `paint_canvas_overlays.rs`), `menu_*`, `pre_*` | sim — com UMA mudança de comportamento: ver §2.1 |
| `ph2d-vec-connect` (3) | `ends.rs` novo (as leis antes do A\* — `exit_point`, `port_side`, `bbox_exit`, `obstacles_in_play`), `route.rs` (`f_key`: o custo do A\* ordena-se arredondado a 1e-6) | ⚠️ ver §2.1 |
| `ph2d-app-vec` (2) | `connector_live.rs`/`connector_walls.rs` passam a usar as leis de `ph2d-vec-connect::ends` (−150 linhas, comportamento igual: 27 testes de conector verdes) | sim |
| `ph2d-text` (5) | a `Virgil` embutida (`fonts/Virgil-Regular.ttf` + `Virgil-LICENSE.txt`), registada como `HAND_FAMILY = "PH2D Virgil"` em `register_bundled` | sim — mais uma família registada; as 3 da interface intactas |
| `ph2d-i18n` (3) | `board.rs` (todas as chaves `board.*`), `shell_media.rs` (+3: a recusa do blob de quadros ilegível) | sim |
| `ph2d-app-registry-init` (2) | a família `ph2d-app-board` no registo gerado | sim |
| `ph2d-vector` (1) | re-export de 4 linhas | sim |
| `.typos.toml` | 4 palavras portuguesas | sim |

### 2.1 Mudanças de COMPORTAMENTO fora do quadro (o integrador tem de as ver)

1. ⚠️ **`ph2d-vec-connect::f_key` muda rotas do VECTOR também** (W2): o A\* ordenava pelo custo CRU e
   dois caminhos de custo igual (±1e-13) eram decididos pelo arredondamento — a seta «tremia» ao arrastar
   (846 saltos em 3 600 passos, medido). Arredondado a 1e-6, o desempate pela centralidade decide (2
   saltos): no Vector as rotas empatadas passam ao CENTRO do vão, que é a lei documentada.
2. ⚠️ **`HeroScreen::rulers_live()` passa a incluir «sem quadro activo»** (07/10, report do dono): as
   réguas e as guias da CENA não mudam; com uma aba de QUADRO activa deixam de pintar (já era assim desde
   W0) **e** de responder ao rato — antes a faixa invisível de 20 px e as guias roubavam 75 % do botão
   Select e metade de cada botão da barra do quadro (medido). `paint.rs` deixou de repetir a condição; o
   gate `the_paint_and_the_gesture_ask_the_same_door_about_the_rulers` lê a condição nova.

## 3. Símbolos que podem colidir — `bash scripts/collision-surface.sh` (07/10, HEAD `b8c75d74c`)

```text
SUPERFÍCIE DE COLISÃO — line/MiroClone contra main
  merge-base a46c4c200   ·   67 commit(s)   ·   311 arquivo(s)
▸ SCHEMAS — ⚠️ o valor se CONTA contra o main do dia; confira nos TRÊS sítios
  ⚠ PROJECT_SCHEMA                        184   (base: 183)
  ⚠   └ tripla do gate               (184, 13, 22)   (base: (183, 13, 22))
    VEC_SCENE_SCHEMA                       22   (base: 22)
    FLIP_SCHEMA                            13   (base: 13)
    DOC_VERSION (timeline)                 18   (base: 18)
  ✗ FIELD_DOC_VERSION                     —   (base: —)
▸ REGISTRO DE COMPONENTES — ph2d-ecs 106 · ph2d-render (espelho) 107 · ph2d-script (espelho) 107 (base iguais)
▸ CONTRATO CONGELADO (§6) — crates/ph2d-nodegraph/src/node.rs intocado · crates/ph2d-editor-core/src/tool.rs intocado
▸ ADR — último no disco: 0179 · esta linha não cria ADR
▸ Cargo.lock — 8 pacotes novos, todos INTERNOS (ph2d-app-board, ph2d-board-{edit,geom,layout,model,render,rough,route});
  nenhum pacote externo novo (o `serde_json` das dev-deps já estava no lock)
▸ MARCADORES DE CONFLITO — nenhum
▸ TETOS DE LOC — nenhum arquivo da linha passa do teto
```

- **`PROJECT_SCHEMA` +1** (o campo `boards`): conte o degrau como DELTA sobre o `main` do dia, nos três
  sítios (escada, tripla do gate, ficheiro irmão) — `python3 scripts/schema-recount.py`.
- ⚠️ **`FIELD_DOC_VERSION` «sonda cega» NÃO é desta linha:** a constante já não existe em lado nenhum
  (saiu do `main` com a poda do 3D); a linha do `collision-surface.sh:92` está órfã — defeito do script
  no `main`, não corrigido aqui (ferramenta partilhada).
- Formato do ficheiro do Quadro: `ph2d_board_model::FORMAT_VERSION = 4` (blob próprio dentro do
  `boards`, versão própria; lê 1, 2 e 3 — fixtures `format_v2.bin`/`format_v3.bin` GRAVADAS pelos builds
  antigos). Nenhuma outra linha toca este blob.
- Ids de chrome: as abas de documento (`ids/chrome/documents.rs`) e os botões das barras do quadro são
  DERIVADOS (`SALT ^ (i+1)`, salto de meio distinto das abas — gate
  `every_bar_item_has_a_distinct_id_that_maps_back` varre 10 000 abas).

## 4. Contratos congelados

**Nenhum encostado.** Prova: `collision-surface.sh` (acima: `node.rs` e `tool.rs` intocados) e
`git diff a46c4c200..HEAD --stat -- crates/ph2d-vector-doc crates/ph2d-nodegraph
crates/ph2d-editor-core/src/tool.rs` **vazio**; os gates corridos à parte no fecho: nós
(`ph2d-nodegraph`, filtro `contract`) **3/3**, ferramentas (`architecture_tool_contract_surface`)
**4/4**, vector (`ph2d-vector-doc`, filtro `contract`) **11/11**. O `ph2d-vector/src/lib.rs` só
re-exporta mais dois nomes do `kurbo` (`Line`, `ParamCurveArclen`).

## 5. Portão de fecho (batched, 1× sobre `a46c4c200..HEAD`, 07/10)

| portão | resultado |
|---|---|
| `BASE=a46c4c200 bash scripts/nextest-impacted.sh` | **17 157 / 17 157** (9 409 fora do impacto) |
| clippy do `ship.sh` (`--workspace --all-targets --features ph2d-spike/bevy_ecs -D warnings`) | limpo (1.ª corrida: 3 imports órfãos + 1 `clone_on_copy` em teste — corrigidos em `b8c75d74c`, re-corrida limpa) |
| `CARGO_BUILD_WARNINGS=deny cargo check --workspace --all-targets` | limpo |
| `cargo fmt --all --check` | limpo (1 linha do gate da shell, corrigida) |
| `file_loc_caps`, `fn_loc_caps`, `architecture_no_downcast_to_concrete_tool_in_shell` (shell) | 9 / 9 |
| `arch_*` / `architecture_*` do `ph2d-editor-core` (incl. `arch_safe_clamp_only`, `architecture_workspace_file_loc_cap`) | 121 / 121 |
| `bash scripts/censos-da-arvore-combinada.sh` (§1.5.9 5-bis) | 114 / 114, os 12 censos com controlo |
| `cargo machete` nas 8 crates do quadro | nenhuma dependência morta |
| `scripts/check-standalone-optional.sh` · `scripts/check-workflow-packages.sh` | verdes (9 crates · 32 nomes citados) |
| código `#[cfg(target_os` escrito/movido | nenhum ⇒ cruzamento macOS não se aplica |
| auditoria (2 lentes, W4) | **correcção**: 8 candidatos — todos concordam (`hand_lettered` porta única; `sketchable` × letra × desenho; `ThemeInk`; `head_scale`; espessuras; `BoardLive::gesture`; `rulers_live`). **costura**: 2 achados — «predefinições sem hit» **refutado** (o braço cai no `true` e regista); «cor/espessura com a borracha pega na caneta» é a lei de produto (documentada em `pen::apply`) |
| prova de mutação (W4) | **16 / 17** — M1 equivalente (filtro redundante: `touches` já recusa não-traços); M12/M16 eram buracos, tapados e re-mutados. Mutações à mão nos gates novos: o oráculo do rascunho sangra sem vértices presos, sem a regra do tamanho e com a elipse a 0,95; o gate do gesto pendurado sangra sem a cura («o clique no Select foi engolido») |

## 6. Números MEDIDOS (o detalhe nos handoffs de cada onda)

- **Velocidade** (plano §2, `--release`, placa da casa): 100 000 rectângulos — CPU 3,4 ms/quadro, placa
  1,47 ms; 100 000 formas com texto longe — placa 1,48 ms; 100 000 formas + 10 000 setas — arrastar uma
  forma revê 1 seta (3,9 ms), parado 0 revisões.
- **Oráculo Excalidraw 0.18.1** corrido por script (`ferramentas/excalidraw_oracle/`):
  `ph2d-board-rough` = rough.js 4.6.4 e perfect-freehand 1.2.0 **ao último dígito** (29 + 11 casos, 1e-9);
  o rascunho do quadro = o `d` do Excalidraw **número a número** com a mesma semente em ≥ 60 formas e 6
  setas (as opções dele: vértices presos, `adjustRoughness`, `curveFitting 1`, primitivas por forma).
- **A barra**: com a porta antiga a faixa das réguas cobria 432/576 px do Select e 288/576 de cada outro
  botão; o gate `no_shell_gesture_zone_sits_on_a_board_bar_button` confere cantos e centro de cada botão
  da barra curta e dos três painéis contra a faixa, as guias, a costura e o reabrir das docas.
- **A captura do dono** (`capturas_excalidraw/formas_finas_do_dono.png`, régua
  `mede_traco_excalidraw.py`): canto 40 px < ¼ do lado ⇒ tecto 32 ⇒ escala 1,25 ⇒ traço ≈ 1,2 unidades
  (a `thin` = 1 do Excalidraw).

## 7. O que só o `ship.sh` / o CI pegaria

- Nada pendente: o clippy do `ship.sh`, o `deny warnings` do workspace, o fmt, o machete, o standalone e
  os pacotes dos workflows correram ao fechar (§5).
- ⚠️ O CI corre ~26 pacotes; as suítes do quadro e os censos vivem em `tests/it` e nas crates novas — o
  portão do integrador sobre a árvore combinada é quem as volta a correr.

## 8. Ordem e dependências

- Uma linha só, sem dependência de outra. O `main` não andou ⇒ `--ff-only`. Se outra linha aterrar antes:
  rebase, e o que mais pode chocar é `PROJECT_SCHEMA` (degrau), `paint.rs`/`offers.rs` do
  `ph2d-editor-core` (a condição das réguas, §2.1), `input_dispatch.rs` da shell e o
  `ph2d-app-registry-init` (registo gerado — regenere, nunca à mão).

## 9. Smokes (todos com o binário `smoke` já compilado na worktree)

```bash
cd ~/Documentos/Projetos/PH2D/Worktrees/line-MiroClone
PH2D_BOARD_SMOKE=1 ./target/smoke/ph2d-host-desktop   # abas: dois quadros, zoom/vista, «Scene» volta à cena
PH2D_BOARD_SMOKE=2 ./target/smoke/ph2d-host-desktop   # formas: fluxograma com texto e o catálogo de 18
PH2D_BOARD_SMOKE=3 ./target/smoke/ph2d-host-desktop   # setas curvas presas, rótulos, pontos de ajuste, pontos azuis
PH2D_BOARD_SMOKE=4 ./target/smoke/ph2d-host-desktop   # notas: cores do Miro, P/M/G, pilha, em massa, pega de grelha
PH2D_BOARD_SMOKE=5 ./target/smoke/ph2d-host-desktop   # caneta, borrachas, laser, Rascunho ↔ Final, letra à mão
PH2D_THEME=sunstone PH2D_BOARD_SMOKE=5 ./target/smoke/ph2d-host-desktop   # a tinta do tema num tema claro
```

O que NÃO foi smokado pelo dono: a pressão de uma mesa digitalizadora (não chega ao app — `winit 0.30`
não a entrega em desktop); quadros gravados ANTES de 07/10 num tema escuro guardam a tinta clara antiga
(os novos usam a tinta do documento).

## 10. O que fica ABERTO (para a próxima onda, W5 — Organizar)

1. **Pressão da mesa** — decisão de dependência do dono (subir o `winit` ou um caminho por plataforma);
   o traço já guarda `Ink::pressure` e afina quando ela vier `true`.
2. Capturas do Miro pedidas e por medir: P/G das notas, vão do `Tab`, sombra, pega de quatro pontos;
   traço da caneta e do marcador sobre branco (a transparência do marcador é 0,4 por omissão).
3. Ponta de seta: o Excalidraw corta-a a metade do último troço quando é curto
   (`min(25, comprimento·0,5)`); a nossa tem tamanho fixo.
4. Desempenho do RASCUNHO em quadros grandes não medido (o traço à mão guarda-se em cache por elemento;
   a 1.ª geração de 10 000 formas à mão está por medir na régua `measure_encode_cost`).
5. Não feitos do Miro: laço, smart drawing; duplo-clique para editar uma predefinição (aqui à vista).
6. Herdados (W2–W3): arrastar o segmento do cotovelo, rótulo ao longo da linha, pontas de UML na barra,
   transbordo das abas, desfazer das operações de aba, cursor sobre as pegas, trackpad, menu do botão
   direito numa forma; texto-guia do modo em massa.
7. `collision-surface.sh:92` (`FIELD_DOC_VERSION`) órfão no `main` (§3).

## 10-bis. A máquina

`target/*/incremental` reclamado; binário `smoke` quente; `pgrep -af 'ph2d|cargo|rustc'` sem processos da
linha (conferido no fecho).
