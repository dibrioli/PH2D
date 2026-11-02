---
name: project-boards-and-kanbans-live-in-the-project-assets
description: Dono 30/10 — Quadros e Kanbans ficam guardados no projeto até serem APAGADOS e aparecem no painel de Assets
metadata:
  type: project
---

Dono 30/10: «a menos que seja deletado, kanbans e boards ficam guardados no projeto. Temos os assets para
isso.» Quadros (MiroClone) e Kanbans (TrelloClone) são documentos do projeto que **só somem por um
apagar explícito** e ficam **listados e disponíveis no painel de Assets** (abrir, arrastar para o Quadro).

**Why:** respondeu à pendência W4 do Kanban («desfazer inserir Kanban deixa o documento vazio órfão —
apagar ou manter?»): nada é órfão, porque o painel de Assets o mostra.

**How to apply:** nunca apagar um Kanban/Quadro por desfazer a sua inserção num Quadro nem por estar
vazio/sem referência; o asset index (`crates/ph2d-asset-index`, `AssetKind`) tem de os expor. Liga com
[[project-image-is-a-project-asset-reference-2026-10-26]] e [[project-kanban-trello-clone-owner-decisions-2026-10-27]].
