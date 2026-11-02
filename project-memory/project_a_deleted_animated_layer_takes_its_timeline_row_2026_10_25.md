---
name: project-a-deleted-animated-layer-takes-its-timeline-row-2026-10-25
description: "Decisão do dono (25/10, auditoria F31): apagar uma camada/item animado faz a linha dela SUMIR da timeline, e o Desfazer devolve as duas (o After Effects)"
metadata:
  type: project
---

Decisão do Enio, 25/10 (auditoria do smoke F31, `line/UIUX`): ao apagar uma camada que tem animação (tinta/efeito/filtro da forma do Vetor, item de lista de componente), **a linha dela na timeline some junto, e o Desfazer traz as duas de volta** — o modelo do After Effects. A alternativa recusada era deixar a linha marcada como «camada apagada».

**Why:** no smoke a linha da camada apagada ficava na timeline sem nome nem marca, indistinguível de uma viva, e o Ctrl+Z não a recolava.

**How to apply:** uma camada apagada segue a lei do OBJECTO apagado — o binding fica `missing` (escondido, não aplicado, guardado) e revive quando o undo global devolve o mesmo `ItemId`; nunca purgar no undo próprio da timeline (dois históricos ⇒ dois Ctrl+Z). Porta: `ph2d_ecs::canal_item_vivo`. Toda família nova com itens identificados alimenta essa porta. Liga com [[project-autokey-records-everything-2026-10-19]].
