---
name: feedback-refactor-workflow
description: "Durante refatorações grandes multi-track, não fazer push/PR/CI até o fim — Enio testa manualmente, depois empurra como onda única"
metadata: 
  node_type: memory
  type: feedback
  originSessionId: 8dd4bacc-01e4-4931-9976-82e9878a91e5
---

Durante refatorações multi-track (Tracks A/B/C/D/E etc.), commits são **locais apenas**. Não fazer `git push`, não abrir PR, não acionar CI até **toda** a refatoração estar concluída.

**Why:** Enio quer validar manualmente o comportamento do app antes de empurrar uma sequência grande de commits pro GitHub. Push/CI cedo polui o histórico remoto com etapas intermediárias e gasta tempo de CI (~30min por run) que pode quebrar em estados incompletos.

**How to apply:**
- Após cada commit local de track/marco, **só** `cargo check` + `cargo test --lib` locais
- Pular completamente o protocolo PRCI (15-min polling) durante refatorações
- No fim de toda a refatoração, Enio faz teste manual no app real
- Só depois do OK do Enio: push em onda única + PR + CI babysit padrão

Distinto do fluxo normal (vide [[feedback-ci-handling]] e CLAUDE.md §CI) onde push imediato após commit é o default.
