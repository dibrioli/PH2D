# 13 — Arrastar o trecho do MEIO de uma seta cotovelo (W23 B5(1))

> Leitor: a LLM que implementa ou audita. Oráculos corridos em 19/10 (fixtures com cabeçalho em
> [`ferramentas/elbow_oracle/`](ferramentas/elbow_oracle/)). Hoje, numa seta `Route::Elbow`, agarrar
> a bolinha do meio de um trecho INSERE um ponto de ajuste (`wire.rs` `begin_bend`, `Mid(i)`): o ponto
> guarda os DOIS eixos no mundo, e quando as caixas mexem a seta faz degraus.

## 1. Estado da arte — MEDIDO (R1 = 0,0 100×100 · R2 = 400,300 100×100 · trecho +60)

| | guarda | R2 → (400,500) | R2 → (700,300) | trecho impossível (R2 em x=150 / do outro lado) | larga quando |
|---|---|---|---|---|---|
| **Excalidraw 0.18.1** | `fixedSegments: [{index, start, end}]` — coordenada ABSOLUTA, `index` = último ponto do trecho | fica em x=310 | fica em x=310 | **atravessa a caixa** (vai a 310 e volta) — foto `excalidraw_C3_*` | a rota passa a recta (e não volta) · duplo clique |
| **tldraw 5.5.2** | `elbowMidPoint` — UMA fracção do vão entre os recuos das pontas (36 cada): 0,5 → 0,76316 = (310−136)/(364−136) | vira L; a fracção fica guardada, sem uso | x = 136 + 0,763 × 528 = 538,95 (proporcional) | espelha; mesma fracção (x=−210) | nunca; volta a valer quando a forma volta a Z |
| **Miro** (documentação) | nada na API (`shape: elbowed`, pontas) | — | — | — | — |

Tentado e abandonado: Excalidraw PR #8615 («fake mid-element», «removal attempts to restore an optimal
route», fechado) → PR #8952 (`fixedSegments` + `startIsSpecial/endIsSpecial`), que admite «subtle
heuristics to guess the lost information» e um commit «Broken but refactor is needed»; #9197 (oscilação
perto de 0,5). Fontes em `elbow_oracle/docs_publicos.json`.

**Escolha: a lei do tldraw (fracção do vão).** Uma grandeza só, sem índice frágil; segue as caixas;
nunca entra numa caixa (a fracção vive entre os recuos, fora delas); não se perde quando a forma da
rota muda e volta. A do Excalidraw atravessa caixas (C3) e perde o trecho na recta — recusada.

## 2. Desenho — uma porta por pergunta

- **Modelo** (`ph2d-board-model/src/connector.rs`): `Connector::mids: Vec<Option<f64>>`, um por TRECHO
  entre estações (`waypoints.len() + 1`); vazio = tudo de nascença. Ler um índice que falta = `None`.
  Porta única: `Connector::mid(leg) -> Option<f64>` / `set_mid(leg, Option<f64>)` (normaliza: corta os
  `None` do fim). Inserir/apagar um ponto de ajuste parte/funde o trecho: `insert_waypoint` /
  `remove_waypoint` passam a ser os ÚNICOS sítios que mexem `waypoints` e `mids` juntos.
- **Lei** (`ph2d-board-route`): `elbow_mid(leg_pts, frac) -> Vec<[f64;2]>` — depois do roteador
  (`ph2d-vec-connect` NÃO muda), se o trecho saiu em Z (3 segmentos entre os recuos: H-V-H ou V-H-V), o
  segmento do meio vai para `a + frac·(b−a)`, `a`/`b` = fim do recuo de cada lado. Outra forma (L,
  recta, U, laço) = a fracção não se usa e fica guardada. Porta única para «a fracção deste ponto»:
  `frac_at(leg_pts, coord)` (o inverso; o arrasto e a importação usam-na).
- **Gesto** (`ph2d-board-edit/src/wire.rs`): numa seta Elbow, a bolinha `Mid(i)` de um trecho em Z vira
  `WireHandle::Segment(i)`: arrastar move o segmento só na perpendicular (a fracção), UM passo de
  desfazer; duplo clique larga-a (`None`). Trecho que não é Z mantém o `Mid(i)` de hoje (insere ponto).
  Cursor: `BoardCursor` ↔/↕ pelo eixo do segmento (a porta `cursor_at` do B5(5)).
- **Fio ao vivo**: `Field::Mids` (no FIM do enum e do `Field::ALL`), `PROTOCOL` 7 → 8.
- **Formato**: `FORMAT_VERSION` 13 → 14, `legacy_v13.rs` (a fixture gravada ANTES da mudança).
- **Excalidraw**: ler `fixedSegments` (o segmento do meio de um Z) → fracção por `frac_at`; escrever o
  `fixedSegments` do nosso trecho fixo (coordenada absoluta + `index`). Doc 07 deixa de o dar como perdido.

## 3. Contratos e schema

Contratos §6 (Nodes, Tools, Vector `ADR-0056..0068`): não tocados — a lei vive em `ph2d-board-route`
depois do roteador; `ph2d-vec-connect`/`ph2d-vector` sem diff (prova: `git diff --stat main -- crates/ph2d-vec-* crates/ph2d-vector` vazio).
`PROJECT_SCHEMA` não sobe (`project_schema.rs:405`: o `BoardSet` tem versão própria). Sobem
`FORMAT_VERSION` (14) e `PROTOCOL` (8) — números que SOMAM entre linhas: só esta linha os mexe.

## 4. UI (as quatro condições)

Existe: `WireHandle::Segment` no overlay `Wire` (um campo `segments`, pintado como pega distinta da
bolinha). Pintado e registado: o alcance do clique é o raio pintado (`Metrics::handle`). O clique chega:
`pointer_down` → `wire_handle_at` antes do arrasto do corpo. A sequência leva a algum lugar: soltar
grava UM passo, o fio leva `Field::Mids`, gravar/reabrir mantém.

## 5. Gates (red-first) e fixture

Fixture = os números do tldraw acima (R1/R2, +60, R2→700,300; R2→400,500 e volta).
1. route: `a_dragged_middle_follows_its_fraction_when_a_box_moves` · `a_fraction_unused_by_an_l_comes_back_with_the_z` · `the_middle_never_enters_a_box`.
2. edit: `dragging_the_middle_of_an_elbow_moves_it_across_in_one_step` (sem ponto novo) · `a_double_click_lets_the_middle_go` · `a_waypoint_split_keeps_each_leg_its_own_middle`.
3. formato: ida-e-volta 14 + os bytes do 13 lidos com `mids` vazio. 4. fio: `Field::Mids` (o teste exaustivo de `fields_tests`).
5. Excalidraw: o caso `w9` com `fixedSegments` (`entradas/gera_w9_casos.py:296`) → fracção → ida-e-volta.
6. ecrã: o arrasto pela porta da shell (`board_bar_tests` `T`), não pelo editor só.

## 6. Smoke

Cena 25 (`PH2D_BOARD_SMOKE=25`, conta-se no roteador `smoke.rs`): duas caixas desencontradas com uma seta
cotovelo; os números (x do segmento antes/depois) saem da sonda headless antes da mensagem ao dono.
