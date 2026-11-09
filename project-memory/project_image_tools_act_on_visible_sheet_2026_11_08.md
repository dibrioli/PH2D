---
name: project-image-tools-act-on-visible-sheet-2026-11-08
description: Dono 08/11 — Trim, Padding, Make Square, Upscale e demais ferramentas de imagem agem na folha VISÍVEL do Animated, como o Painter; consertar tudo antes de integrar a GEF 26
metadata:
  type: project
---

Dono 08/11 (linha GameEngineFuncional, GEF 26 várias folhas): **«SIM»** — as ferramentas de imagem fora do cartão (Trim, Padding, Make Square, Upscale e afins) passam a agir na folha que está À VISTA, como o Painter já faz. E **«Vamos consertar tudo antes de integrar»**: os achados baixos da auditoria final (roda durante o voo, cartão sem folha 0, mensagem de upload, LoadSheet calado, cor das folhas na macro) fecham antes da integração.

**Why:** com várias folhas, editar a folha 0 quando a 2.ª está à vista é a mesma família do bug do Painter que a auditoria achou (tinta na folha errada).

**How to apply:** lista e modelo no handoff `docs/GameEngineFuncional/handoffs/HANDOFF_CONTINUACAO_2026-11-08_VARIAS_FOLHAS_AUDITADA.md`; censar todos os verbos de imagem antes de construir. Ver [[project-animated-sprite-card-owner-decisions-2026-11-06]].
