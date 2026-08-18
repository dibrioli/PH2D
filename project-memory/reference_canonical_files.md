---
name: reference_canonical_files
description: "Onde mora cada COISA no repo (tool, painel, widget, registry, gate) — a tabela verificada contra o disco. NÃO guarda versões nem contagens: essas rotam, e é o roteador do CLAUDE.md §1 que as tem."
metadata:
  node_type: memory
  type: reference
  originSessionId: fe59209c-4f42-43aa-a540-0a60c10ff373
---

> ⚠️ **REESCRITA em 2026-08-18.** A versão anterior era um retrato de 2026-05-19 indexado como
> *"paths canônicos"* — a primeira parada de toda LLM que procura onde algo mora. Em três meses
> ela apodreceu inteira, e cada erro mandava a LLM ao lugar errado com confiança:
>
> | ela dizia | media na verdade |
> |---|---|
> | DIRETRIZ **v6.1**, ~**833** LOC, com a lista das suas seções | **v8.3**, **1.261** linhas, e a numeração das seções mudou |
> | SKILL **v2.14**, ~1050 LOC | **v2.15** |
> | `crates/ph2d-editor-core/tests/` — **11 gates ativos** | **53** |
> | `docs/HANDOFF_node_system.md` — *"tracker VIVO"* | arquivado em `docs/archive/handoffs-2026-06-16/` |
> | raiz do repo = `/Volumes/MAC_EXTERNO/...` + memória em `/Users/dibrioli/...` | caminhos de um Mac que não é a máquina de desenvolvimento |
>
> **A lição, e é geral:** *uma memória que afirma uma VERSÃO ou uma CONTAGEM está a marcar
> encontro com o próprio erro.* O que sobrevive é **onde as coisas moram** — isso muda por
> `git mv`, que é raro e visível. O resto se lê da fonte, sempre.

## Onde mora cada coisa (verificado contra o disco em 2026-08-18)

| O que | Onde |
|-------|------|
| Tool nova | `crates/ph2d-tool-<slug>/` — uma crate isolada por tool ([ADR-0040](../docs/architecture/decisions/0040-tool-as-isolated-feature-crate.md)) |
| Painel novo | `crates/ph2d-panel-<slug>/` — uma crate por painel, feature-gated |
| Widget primitivo | `crates/ph2d-editor-core/src/widget/<slug>.rs` — **cap 500 LOC** |
| Handler de chrome (ação da TopBar) | `crates/ph2d-editor-core/src/screens/hero/chrome/<slug>.rs` — um por arquivo |
| Registro de tools | `crates/ph2d-tool-registry-init/src/lib.rs` → `register_all` (alfabético, gateado) |
| Registro de painéis | `crates/ph2d-panel-registry-init/src/lib.rs` → `register_all_panels` |
| Vitrine de widgets | `crates/ph2d-editor-core/src/widget/showcase/` |
| Gates de arquitetura | `crates/ph2d-editor-core/tests/` |
| Gate dos tokens | `crates/ph2d-tokens/tests/mockup_tokens_exist.rs` |
| Design system (fonte) | `docs/design/` — `tokens.json` · `styles/tokens.css` · `screens/*.html` · `tools/*.toml` · `icons/*.svg` |

⚠️ **Todas as dez linhas foram testadas com `test -e` no dia da reescrita.** Se uma falhar,
o defeito é este arquivo — corrija-o, não contorne.

## O que NÃO perguntar a esta memória

| pergunta | onde se responde |
|---|---|
| que doc leio para a minha tarefa? | [`CLAUDE.md §1`](../CLAUDE.md) — o roteador leia-por-tarefa |
| qual o estado do módulo X? | `CLAUDE.md §5` |
| o que está congelado? | `CLAUDE.md §6` |
| qual o número do ADR NNNN? | [`docs/architecture/decisions/README.md`](../docs/architecture/decisions/README.md) — índice **derivado** (`bash scripts/adr-index.sh`) |
| qual o `PROJECT_SCHEMA` / registro / teto de LOC de hoje? | `bash scripts/collision-surface.sh` — **conte, nunca escolha** ([[feedback_numbers_that_sum_across_lines_count_dont_pick]]) |
| que versão tem a DIRETRIZ / o SKILL? | o cabeçalho deles. ⛔ Não copie o número para cá — foi assim que este arquivo apodreceu |

## Arquivo (⛔ não referenciar como canônico)

`docs/archive/` guarda a história, **verbatim**, com um `README.md` derivado em cada pasta
(`python3 scripts/archive-index.py <pasta>`). As grandes: `estado-2026-08-18/` (o antigo
`CLAUDE.md §5`), `docs-2026-08-18/` e `tracker-physics-2026-08-18/` (os docs cortados em
2026-08-18), `handoffs-2026-06-16/`, `multi-agente-pre-v6.0/`.

⛔ **As recusas medidas que foram para o arquivo continuam a valer:** cada doc vivo cortado leva
no fim uma tabela `⛔ Recusas MEDIDAS` com o link para a linha exata. Consulte-a antes de propor
otimização — ver `CLAUDE.md §5.0`.

Vide também [[feedback_stale_comment_and_dead_code_lie]] e
[[feedback_a_rule_only_exists_if_it_is_on_the_path_of_who_executes_it]].
