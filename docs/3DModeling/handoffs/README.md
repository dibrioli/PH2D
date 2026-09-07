# `3D Modeling` — handoffs (registro cronológico de sessão)

> **O que é esta pasta:** o registro de **como** o módulo foi construído — um arquivo por sessão de
> linha. O **pensamento** do módulo (planos, pesquisas, resultados medidos) fica **um nível acima**,
> em [`docs/3DModeling/`](..).
>
> ⚠️ **Isto NÃO é o estado atual do módulo.** O estado vivo é o
> **[`CLAUDE.md §5`](../../../CLAUDE.md)**; um handoff descreve o mundo **no dia em que foi escrito**
> e não é atualizado depois. Use-os para responder *"por que isto ficou assim?"* — nunca para decidir
> a próxima ação.

⚠️ **Esta linha registou o mecanismo de cada wave no doc vivo, não aqui:** as 56 seções de
[`06_resultados_cena_e_gizmo.md`](../06_resultados_cena_e_gizmo.md) são uma wave cada, com a tabela
medida e as provas de mutação ao lado. Esta pasta guarda o que atravessa a **fronteira da linha**.

**5 handoffs.**

| Data | Arquivo | Papel | Assunto |
|---|---|---|---|
| 2026-08-22 | [HANDOFF_INTEGRACAO_line_3DModeling_2026-08-22.md](HANDOFF_INTEGRACAO_line_3DModeling_2026-08-22.md) | integração | Handoff de INTEGRAÇÃO — `line/3DModeling`, 73 commits (DIRETRIZ §1.5.9) |
| 2026-08-23 | [HANDOFF_INTEGRACAO_line_3DModeling_2026-08-23.md](HANDOFF_INTEGRACAO_line_3DModeling_2026-08-23.md) | integração | Handoff de INTEGRAÇÃO — as waves **35–55**, 24 commits (DIRETRIZ §1.5.9) |
| 2026-08-24 | [HANDOFF_INTEGRACAO_line_3DModeling_2026-08-24.md](HANDOFF_INTEGRACAO_line_3DModeling_2026-08-24.md) | integração | Handoff de INTEGRAÇÃO — as waves **56e–58d**, 20 commits (DIRETRIZ §1.5.9) |
| 2026-08-26 | [HANDOFF_INTEGRACAO_line_3DModeling_2026-08-26.md](HANDOFF_INTEGRACAO_line_3DModeling_2026-08-26.md) | integração | Handoff de INTEGRAÇÃO — as waves **59–80**, 25 commits (DIRETRIZ §1.5.9) |
| 2026-08-29 | [HANDOFF_INTEGRACAO_line_3DModeling_2026-08-29.md](HANDOFF_INTEGRACAO_line_3DModeling_2026-08-29.md) | integração | Handoff de INTEGRAÇÃO — as waves **81–104-ter**, 45 commits (DIRETRIZ §1.5.9) |
| 2026-08-30 | [HANDOFF_INTEGRACAO_line_3DModeling_2026-08-30.md](HANDOFF_INTEGRACAO_line_3DModeling_2026-08-30.md) | integração | A peça **desaparecia** ao ganhar formas (lei do passo + orçamento), o arco preto da cruz, e a Hierarquia ganha `Delete`/`Ctrl+D` |
| 2026-09-03 | [HANDOFF_INTEGRACAO_line_3DModeling_2026-09-03.md](HANDOFF_INTEGRACAO_line_3DModeling_2026-09-03.md) | **FECHO** | 55 commits desde `066b4f92e`. ⛔⛔⛔ o **undo/redo pula etapas e NÃO está curado** (o §7.1 tem as **oito** suspeitas já eliminadas e as **três** que eram defeitos reais já curados) · ⚠️ `PROJECT_SCHEMA` **103 → 106**, três degraus · o chanfro honesto · o laço que subtrai |
| 2026-09-06 | [HANDOFF_INTEGRACAO_line_3DModeling_2026-09-06.md](HANDOFF_INTEGRACAO_line_3DModeling_2026-09-06.md) | **FECHO** | 17 commits desde `53832c884`. A **SUPERQUADRÁTICA** (W127) e a **SUPERFÓRMULA** de Gielis (W128), a mola e o gyroid, o cilindro com bojo · ⚠️ `PROJECT_SCHEMA` **114 → 115** (o `offset` do espelho, que era um controlo **morto**) · o divisor da W128 corria por LADRILHO (`642×`) |
| 2026-09-07 | [HANDOFF_INTEGRACAO_line_3DModeling_2026-09-07.md](HANDOFF_INTEGRACAO_line_3DModeling_2026-09-07.md) | **FECHO** | 10 commits desde `815555aed`. O **TRIÂNGULO** (W131), o **POLÍGONO de `N`** com os vértices arrastáveis no canvas (W132–W133), o **NÓ DE TORO `(p,q)`** (W134, mais os três defeitos de forma do report em W134b) e a **ROSCA + SERRILHADO** (W135) ⇒ o catálogo vai a **68** entradas sobre **58** primitivas e a fila de formas fecha de 10 para **6** · ⚠️ `FIELD_DOC_VERSION` **17 → 18**, e ⛔ **ele NÃO está no `collision-surface.sh`** · ⭐ nenhum dos quatro schemas da sonda se mexe |

---

## Onde está o resto

| pergunta | onde se responde |
|---|---|
| *o que o módulo **é**, e por que esta rota* | [`README.md`](../README.md) — a porta, com a tabela dos 6 docs |
| *o que foi **medido** em cada wave* | [`06_resultados_cena_e_gizmo.md`](../06_resultados_cena_e_gizmo.md) §1–§104 |
| *o que está **aberto**, hoje* | o **§13** do doc 06 — a lista viva |
| *por que **campo implícito** e não malha* | [ADR-0161](../../architecture/decisions/0161-3d-modeling-is-an-implicit-field-tree-and-what-the-artist-sees-is-the-traced-field.md) |
