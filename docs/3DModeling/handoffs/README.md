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

**2 handoffs.**

| Data | Arquivo | Papel | Assunto |
|---|---|---|---|
| 2026-08-22 | [HANDOFF_INTEGRACAO_line_3DModeling_2026-08-22.md](HANDOFF_INTEGRACAO_line_3DModeling_2026-08-22.md) | integração | Handoff de INTEGRAÇÃO — `line/3DModeling`, 73 commits (DIRETRIZ §1.5.9) |
| 2026-08-23 | [HANDOFF_INTEGRACAO_line_3DModeling_2026-08-23.md](HANDOFF_INTEGRACAO_line_3DModeling_2026-08-23.md) | integração | Handoff de INTEGRAÇÃO — as waves **35–55**, 24 commits (DIRETRIZ §1.5.9) |
| 2026-08-24 | [HANDOFF_INTEGRACAO_line_3DModeling_2026-08-24.md](HANDOFF_INTEGRACAO_line_3DModeling_2026-08-24.md) | integração | Handoff de INTEGRAÇÃO — as waves **56e–58d**, 20 commits (DIRETRIZ §1.5.9) |
| 2026-08-26 | [HANDOFF_INTEGRACAO_line_3DModeling_2026-08-26.md](HANDOFF_INTEGRACAO_line_3DModeling_2026-08-26.md) | integração | Handoff de INTEGRAÇÃO — as waves **59–80**, 25 commits (DIRETRIZ §1.5.9) |

---

## Onde está o resto

| pergunta | onde se responde |
|---|---|
| *o que o módulo **é**, e por que esta rota* | [`README.md`](../README.md) — a porta, com a tabela dos 6 docs |
| *o que foi **medido** em cada wave* | [`06_resultados_cena_e_gizmo.md`](../06_resultados_cena_e_gizmo.md) §1–§56 |
| *o que está **aberto**, hoje* | o **§13** do doc 06 — a lista viva |
| *por que **campo implícito** e não malha* | [ADR-0161](../../architecture/decisions/0161-3d-modeling-is-an-implicit-field-tree-and-what-the-artist-sees-is-the-traced-field.md) |
