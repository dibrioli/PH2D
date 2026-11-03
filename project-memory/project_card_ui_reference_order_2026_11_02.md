---
name: project-card-ui-reference-order-2026-11-02
description: "Dono 02/11 — UI em cartões: referência 1 = os nossos painéis com smoke OK; depois Figma, Rive, e só então Inkscape/Godot/Unity"
metadata:
  node_type: memory
  type: project
  originSessionId: 009846d8-956d-48d2-8619-6cf28b0c9900
  modified: 2026-11-02T23:41:02.497Z
---

Ordem de referência para a UI em cartões (dono, 02/11/2026, ao fechar a linha de TESTE `line/TrelloUITeste`):
1. **Nós mesmos** — os painéis/cartões cujo smoke o dono APROVOU nessa linha (lista em `docs/CartoesUI/05_ESPECIFICACAO_UI_COMPLETA.md`).
2. **Figma** — widget ainda não criado: consultar o Figma PRIMEIRO.
3. **Rive** — se o Figma não resolve a complexidade do widget.
4. **Inkscape, Godot, Unity** — só se ainda não resolver.

**Why:** a linha de teste provou a UI («funciona muito bem e é linda»); a implementação real será uma linha nova a partir do main, seguindo o doc 05.
**How to apply:** antes de desenhar um widget de cartão, procure-o nos painéis com smoke OK; só depois Figma → Rive → os outros. Muitos painéis não foram trabalhados (a linha era só teste). Ver [[project-kanban-trello-clone-owner-decisions-2026-10-27]].
