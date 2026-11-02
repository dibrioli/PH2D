---
name: feedback-every-drag-reorder-flies-like-the-kanban-card
description: "Dono 01/11 — todo arrastar-para-reordenar (amostras da paleta, quadros…) tem os MESMOS sinais visuais do cartão do Kanban/Trello"
metadata:
  node_type: memory
  type: feedback
  originSessionId: f17ea438-bccf-446a-93f7-516bedd7722e
  modified: 2026-11-01T18:42:10.432Z
---

Dono, 01/11/2026 (smoke da P6k, paleta do Pixel): «arrastar a cor deveria ter os mesmos indicativos
visuais de arrastar o cartão do Trello». A amostra arrastada voa sob o cursor e as outras abrem o vão
onde ela vai cair — a lei do cartão do Kanban (`ph2d_editor_core::paint_flight`), já usada pelo bloco
da linha do tempo do Pixel (P6i).

Mesmo dia, 2.º smoke: o vão sozinho «não ficou muito evidente» — **sempre** uma linha vertical
destacada (acento) logo à esquerda da primeira célula onde o bloco cai.

**Why:** um gesto, uma cara em todo o app; o dono aprende uma vez.

**How to apply:** qualquer lista reordenável por arrasto nova (ou existente sem voo) reutiliza o voo e
a ordem prevista do Kanban — nunca um arrasto de linha sem pré-visualização.
