# HANDOFF de CONTINUAÇÃO — `line/Vector`, prompt 49 (lições do VectorCraft) — 2026-10-11

> Janela fechada pela regra R1 (~140 chamadas). A próxima janela segue a FASE 1 do
> `MODELO_TROCA_DE_AGENTE_NA_LINHA.md` (o `git cherry main HEAD` dá `> 0`: são commits desta linha,
> NÃO faça `reset --keep`) e continua daqui. Tarefa: [`49_PROMPT_…`](../49_PROMPT_linha_vector_licoes_do_vectorcraft.md).

## Onde está

- Worktree `Worktrees/line-Vector`, branch `line/Vector`, base = `main` `8f20c60de` (a linha foi
  `reset --keep main` no início: os 6 commits velhos eram patches EMENDADOS pelo integrador de 07/10).
- Commits desta janela (todos `--no-verify`, nenhum push):
  `92cc9281d` docs (estudo, prompt 49, sondas → `ferramentas/`) · `4e71193d7` **cerca** ·
  `6d6dbae87` **W1** · `5b830149f` **W2** · `68455bb47` **W0**.

## O que fechou (números do gate `the_offset_sweep_matches_the_table`, 600 combinações)

| | merge-base (0.4) | W1 (0.5) | W2 |
|---|---|---|---|
| Round fora | 87/100 | 100/100 | 100/100 |
| Bevel fora | 35/100 | 40/100 (vazio ≥ 164) | **100/100** |
| Bevel dentro | 94/100 (vazio 44..64) | 94/100 | **100/100** |
| Miter fora/dentro, Round dentro | 100/100 | 100/100 | 100/100 |
| pânicos engolidos | 104 | 0 | 0 |

- **Cerca:** verdade analítica (retângulo por aresta + junção do lado de fora da curva) atada ao
  `resvg` pela fixture `tests/it/fixtures/estrela_offset_resvg.tsv` (pior 0,246 %, tecto 0,3 %);
  hook de `std::panic` conta os pânicos (`counting_panics`), também no gate de robustez (2 700 → 0).
- **W1:** `linesweeper` 0.5. Único vermelho de `nextest-impacted` (3 603 testes): a fita do Power
  Stroke trazia FUROS de área 0 → curado em `Region::into_paths` (`SLIVER_REL`). `catch_unwind`
  fica como cinto CONTADO (doc reescrito em `offset_path`).
- **W2 (BUGS #38):** dois mecanismos, medidos estágio a estágio: `drop_phantoms` apagava a resposta
  (piso `|d|` falso para cordas que se cortam) e o stroker da kurbo tem winding 0 dentro de
  retângulos da banda onde se dobra (no catálogo até 10,5 % a `d = 2`). Cura: `expand_band.rs` —
  banda Bevel/Miter = união de peças positivas (corridas do lado útil + o lado inútil só se cruzar);
  Round fica no stroker + `expand_phantom.rs`. Custo release no catálogo: **1,0× o stroker** no
  total (sonda `#[ignore] measure_the_band_against_the_stroker_on_the_catalog`).
- **W0:** `oracle_vectorcraft.rs` + `fixtures/vectorcraft_oracle.tsv` (592 passos) +
  `ferramentas/vectorcraft_2026-10-11/{oraculo_vc.py, juiz_resvg_offset.py}`. 578/592 concordam;
  as 14 discordâncias estão em `DISAGREE` com o lado certo.

## 2.ª janela (11/10) — itens 1 e 2 FECHADOS em `665d959c2`

- **#39** (a lente de 2 âncoras largada na volta do motor — a hipótese do XOR caiu) e **#40** (a
  junção Bevel/Miter pela corda; o ponto a `1e-17`) — mecanismos, números e gates no
  [BUGS](../BUGS_vector.md) #39/#40. A régua `raster.rs` passou a deslocar a grelha por frações
  irracionais (o losango era ela). `DISAGREE`: 10 linhas, 582/592; o balão é PH2D certo.
- Mutação feita (6/6 sangram; o filtro de segmento nulo era inalcançável e saiu) — **não** a
  repita no fecho; a tabela abaixo continua a valer para as cláusulas da W2.
- ⚠️ O Offset Bevel/Miter de TODA forma curva com quina mudou (`split_at_corners`, só
  Bevel/Miter): o `nextest-impacted` do fecho é obrigatório sobre os consumidores de `ph2d-app-vec`.
- Corrida dirigida da crate: 158 verdes · 24 ignorados; clippy `--all-targets -D warnings` limpo.

## Aberto (por ordem)

3. **W3 — kerning**, não começado. Leis: `crates/ph2d-vec-text/src/glyph.rs` doc de
   `wrapped_lines` (quebrador e desenhador com a MESMA `line_advance`); `parley`/`harfrust` já na
   árvore. ⛔ Ordem do coordenador: **NÃO** vira padrão — atrás de chave, antes/depois fotografado
   (`docs/Components/ferramentas/fotografa_cena.sh`; nunca `spectacle`). Oráculo:
   `vectorcraft-cli` `type.createOutlines` / `--outline-text` com `AV To Wa fi`, avanço por glifo.
4. **W4 — Build/Shape Builder**: correr BUGS #12/#28 + o corpus da W0 nos dois; abrir só se o
   nosso perder; senão registar a medição.
5. **Fecho (DIRETRIZ §1.5.9):** mutação das cláusulas novas (abaixo) → gate batched 1×
   (`BASE=$(git merge-base main HEAD) bash scripts/nextest-impacted.sh` — a última corrida foi
   ANTES da W2: rodar de novo, a W2 muda a representação do Offset Miter/Bevel nos consumidores de
   `ph2d-app-vec`) + clippy `--all-targets` + `CARGO_BUILD_WARNINGS=deny cargo check --workspace
   --all-targets` → handoff de integração → CLAUDE.md §5.1 (≤ 700 B) → `bash scripts/doc-index.sh`
   → `rm -rf target/*/incremental` → binário smoke 2×. **Cena de smoke** para o dono: estrela +
   Offset (Expand) com Round e Bevel, o slider de ponta a ponta — número contado no roteador da
   família (`PH2D_BUILD_SMOKE=<n>`), fotografada antes de mandar.

### Mutações a provar no fecho (cada uma tem de SANGRAR)

| cláusula | mutação | gate que tem de ficar vermelho |
|---|---|---|
| `loop_region` usa peças p/ Bevel/Miter | `round = true` sempre | `the_offset_sweep_matches_the_table` |
| `crosses_out` conservador | `return false` | `the_side_filter_never_changes_the_answer` |
| `push_positive` orienta | não inverter | idem + `every_piece_is_positive…` |
| corridas | `bends` sempre `false` (peça a peça) | só custo — esperado SOBREVIVER (registar) |
| `into_paths` tira furo-lasca | remover o filtro | `the_ink_is_one_connected_piece_with_no_loops` |
| contagem de pânicos | hook não incrementa | (controlo: precisa de um pânico — usar `linesweeper =0.4.0` só na prova) |

## Para o integrador (colisão)

- ⚠️ A `line/UIUX` mexeu em `penned_outline` (cortes do `miter_clip`). Esta linha **não tocou**
  `penned_outline` nem `miter_clip`, mas o **Offset Bevel/Miter deixou de passar por
  `penned_outline`** (só o Round e o Outline Stroke passam). Se a UIUX mudou a SEMÂNTICA do Miter
  (clip em vez de cair em bevel), o Offset Miter (peças: bevel além do `miter_limit`, a lei do
  SVG) e o Outline Stroke vão divergir — decisão de integração, não desta linha.
- `Cargo.lock`: só `linesweeper 0.4.0 → 0.5.0` (mesmas deps).
- Ficheiros novos: `expand_band.rs`, `expand_band_tests.rs`, `expand_phantom.rs`,
  `tests/it/{raster.rs, the_offset_sweep_matches_the_table.rs, oracle_vectorcraft.rs}`, duas
  fixtures. `expand.rs` 699 → 610 LOC (o caçador de fantasmas mudou-se). Nada na shell.

## Premissas do prompt que a medição derrubou

- «Bevel dentro vazio em 44–52»: é **44–64** (a verdade só morre em 76; o estudo só contou até o
  raio inscrito).
- «com 0.5 é `Region::of = None` ou o teste de desapareceu?»: nenhum dos dois — o `drop_phantoms`
  apagava, e sem ele a forma saía a MENOS (o stroker).
- O defeito do stroker não é da estrela: 13 das 44 formas do catálogo erram ≥ 2 % a `d = 2`.
- `resvg` não é juiz onde o traçado se dobra (Lua): ele e o VC cancelam a dobra como o stroker.
