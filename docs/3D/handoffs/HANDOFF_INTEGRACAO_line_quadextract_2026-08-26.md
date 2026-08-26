# HANDOFF — `line/quadextract`, 2026-08-26

> **34 commits.** A jornada partiu de quatro queixas do artista (2026-08-25) e fechou duas
> delas, mediu a terceira até à causa e deu à quarta um número com **controlo**.
> ⛔ **NÃO integrado, NÃO pushado** (CLAUDE.md §0.7).

## §1 — O que o produto ganha, e o que muda no botão

Tudo abaixo shipa **LIGADO** e tem interruptor para bissecar.

| o quê | onde | interruptor |
|---|---|---|
| ⭐ Remover **folhas de espessura zero** à entrada | `ph2d_remesh_iso::DOUBLED_REPAIR` | `PH2D_DOUBLED_REPAIR=0` |
| ⭐⭐⭐ O **bordo como linha de feição** (nos DOIS caminhos) | `retopo_global.rs` + `retopo_extract.rs` | `PH2D_BOUNDARY_FEATURE=0` |
| ⭐⭐ O **acabamento** na cadeia da extracção | `retopo_extract.rs` | `PH2D_EXTRACT_FINISH=0` |
| ⭐ Os **dois resgates** da travessia órfã | `ph2d-quadextract/src/walk.rs` | — (guardados por lei) |

### O antes/depois nas peças do artista

| peça | | furos | quads `>60°` | quads `>4×` | torção p99 |
|---|---|---|---|---|---|
| `t002` | antes | ⛔ `31` | ⛔ `22` | ⛔ `24` | `62,9°` |
| | **agora** | ⭐ `14` | ⭐ **`0`** | ⭐ **`0`** | ⭐ `31,1°` |
| `t001` | antes | `8` | `5` | `1` | `48,6°` |
| | **agora** | ⭐⭐ **`0`** (`χ = 2`) | ⭐ **`0`** | ⭐ **`0`** | ⭐ `31,5°` |
| `t003` | antes | `6` | `3` | `1` | `40,9°` |
| | **agora** | ⭐ `4` | `1` | ⚠️ `4` | ⭐ `29,0°` |

⚠️ **A única regressão nomeada:** a `t003` ganha 3 faces `>4×` (o p99 do aspecto **desce**,
`1,97 → 1,64` — são faces isoladas).

**Smoke:** `cd .../Worktrees/line-quadextract && cargo run -p ph2d-host-desktop --release`,
tecla `` ` ``, secção **Topology**, **`Quad Retopology`**.
O A/B de ontem contra hoje: prefixar `PH2D_BOUNDARY_FEATURE=0 PH2D_EXTRACT_FINISH=0
PH2D_DOUBLED_REPAIR=0`. As seis malhas gravadas estão em `~/Downloads/ph2d_comparar/`.

## §2 — O que está MEDIDO E RECUSADO (não reconstruir sem ler)

| recusa | onde vive a tabela |
|---|---|
| ⛔ a **lei do rebordo** no remalhe (perímetro exacto, produto pior) | `ph2d_remesh_iso::BORDER_LAW` |
| ⛔ as quatro reparações não-manifold de 25/08 | `ph2d_remesh_iso::MANIFOLD_REPAIR` |
| ⛔ subir o `ALIGN_WEIGHT` (satura em `20,4°` e triplica os furos) | `ACHADO_ordem_das_fases.md` §19.3 |
| ⛔ afinar o `ALPHA` da fase zero (`4×` compra `1,6°`, paga `10×` furos) | §19.4 |
| ⛔ as **linhas de feição** por curvatura (custam bordo nas peças dele) | §19.2 |

## §3 — Três afirmações MINHAS que a medição refutou

1. *«o remalhe cria não-manifold sozinho»* — em **11** peças limpas ele cria **zero**; eu
   comparara dois números da **mesma peça partida**. (§14)
2. *«a saída assenta na escultura: `0,000 %`»* — **tautológico** depois do acabamento, e o
   aviso estava escrito no doc do `detail_lost` com o número de 2026-08-21. (§19.1)
3. *«pode ser não-determinismo»* — era eu a **reconstruir o binário a meio da varredura**.
   Três corridas do binário parado saem idênticas. (§21.3)

## §4 — O estado das quatro queixas dele

| queixa | estado |
|---|---|
| «furos nas pontas» | ⭐⭐ **causa achada em duas frentes e curada**; `t001` fecha, `t002` `31 → 14` |
| «superfície irregular quanto mais densa» | ⭐ **medido: a aspereza é da escultura dele**, e a grade fina resolve-a — não somos nós (§18) |
| «relevos não obedecidos» | ⭐ **número com controlo** (`21,7°` contra `22,5°` = «não olhou»), duas alavancas recusadas (§19) |
| «edge loops nas transições» | ⏳ **por tocar** |

## §5 — A obra seguinte, por ordem de evidência

1. ⭐⭐⭐ **A quantização/layout** — é a única fase entre um campo comprovadamente alinhado
   (§19.3) e uma saída que não segue o relevo. As duas alavancas a montante estão fechadas.
2. ⭐ **As 2 órfãs que sobram na `t003`** — caem num canto de leque com holonomia ≠ identidade,
   onde o resgate **recusa de propósito** (§21.1). A cura pede desambiguar a saída certa, não
   escolher uma rota.
3. ⏳ A 4.ª queixa (edge loops nas transições), ainda sem régua.

## §6 — Ficheiros

`ph2d-mesh`: `manifold.rs` (+`_tests`), `feature_edges.rs` (+`_tests`), `lib.rs` ·
`ph2d-remesh-iso`: `lib.rs`, `lib_tests.rs` · `ph2d-quadfill`: `finish.rs`, `lib.rs` ·
`ph2d-quadextract`: `walk.rs`, `lib.rs`, `tests/gates_fixtures.rs`,
`examples/{chain_info,manifold_census}.rs` · `ph2d-gridmap`: `weld_round.rs` ·
`shells/desktop`: `sculpt3d_history_retopo_{global,extract}.rs`, `sculpt3d_scenes_quad.rs` ·
doc: `docs/3D/quad-remesh/ACHADO_ordem_das_fases.md` §13–§21.

⚠️ **Clean-room:** todo artefacto passou `scripts/cleanroom-sweep.sh` (56 entradas) antes de
cada commit. Nenhuma fonte do alvo entrou nesta janela.
