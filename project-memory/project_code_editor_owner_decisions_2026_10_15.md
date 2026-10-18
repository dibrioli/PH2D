---
name: project-code-editor-owner-decisions-2026-10-15
description: Editor de scripts Luau (line/CodeEditor) — as 4 decisões de produto do dono em 15/10/2026
metadata:
  node_type: memory
  type: project
  originSessionId: af38a609-65da-48fc-a6db-15ebe72c095a
  modified: 2026-10-15T18:15:20.762Z
---

O editor de scripts in-app (line/CodeEditor, plano em `docs/CodeEditor/`) é para o Luau que o `ph2d-script` já roda. Decisões do dono em 15/10/2026:
1. Script mora DENTRO do `.ph2dproj` (não arquivo solto) — degrau de schema.
2. Aceita empacotar o `luau-lsp` (~13 MB/OS) como processo-filho.
3. IA desligada agora; entra «da forma mais completa e poderosa possível» só no FIM da implementação.
4. Script pode apontar outro objeto pelo nome (arrastar da Hierarchy, como no Godot) — «e tudo que torne mais fácil, rápido e intuitivo». Reverte a regra antiga «scripts só por sinais».
5. 17/10: as explicações dos erros (I8) só em INGLÊS, como o resto do app (ordem de 19/09); o plano dizia PT/EN — o texto mora em `ph2d-i18n/src/code_explain.rs`, pronto para outro idioma quando o app inteiro o tiver.

**Why:** são produto, não técnica — só o dono muda. Detalhe e consequências: `docs/CodeEditor/00_decisoes_do_dono.md`.
**How to apply:** não reabrir estas perguntas; não pôr superfície de IA antes da última onda; toda referência a objeto é durável (StableId/nome), ver [[feedback_architecture_decisions_are_delegated_to_the_gold_standard]].
