---
name: feedback-clean-text-merge-can-be-semantically-broken
description: "Merge sem conflito textual NÃO prova que a árvore compila — uma linha remove o símbolo, a outra o usa, e o git funde feliz"
metadata: 
  node_type: memory
  type: feedback
  originSessionId: 6316633f-521c-4b1d-a255-7662e2fda363
---

Na integração das 6 linhas (2026-07-12), `line/anim` **removeu** o `MotionTransport` (relógio
único, W4.T7) e `line/motion-value` tinha **13 usos** dele. O `git merge-tree` das duas passava
**limpo** — o conflito não é textual, é **semântico**: as duas mexeram em arquivos/regiões
diferentes. O merge "funciona" e a **compilação quebra**.

Pior que quebrar: um resolvedor apressado **restaura o campo removido** para "consertar o build"
— e a divergência que a outra linha acabou de matar volta **em silêncio**, com o merge verde.

**Why:** conflito textual é uma heurística de *proximidade de linhas*, não de significado. Remoção
de símbolo × uso de símbolo moram em arquivos distintos por construção, então o git nunca os
cruza. Só o **compilador** cruza.

**How to apply:**
- Rebase limpo ≠ pronto. Depois de todo rebase de integração, rode `cargo check --workspace`
  (é o que o `scripts/foundational-integrate.sh` faz no passo [4/5] — não pule).
- Quando um handoff disser "removi X", **grepe X nas outras linhas ANTES de integrar**: é um
  minuto de trabalho e nomeia o conflito antes de ele te morder.
- Ao traduzir os usos, a direção é **a da entrega**, nunca a da conveniência: quem removeu o
  símbolo o fez por um motivo (aqui: dois relógios que se auto-avançam divergem). Ressuscitá-lo
  desfaz a entrega.
- Relacionadas: [[feedback_derived_coordinate_seed_must_match_sample]] (a tradução tem de usar a
  MESMA derivação do original) · [[project_integrator_ship_catches_latents_budget_iterations]].
