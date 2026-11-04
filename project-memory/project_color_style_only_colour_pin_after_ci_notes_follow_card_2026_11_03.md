---
name: project-color-style-only-colour-pin-after-ci-notes-follow-card-2026-11-03
description: "Dono 03/11 — estilo de cor controla SÓ a cor (opacidade é da linha); alfinete nos demais cartões só depois do CI; a nota anda com o cartão solto"
metadata:
  node_type: memory
  type: project
---

Decisões do dono em 03/11 (Inspector em cartões):

1. **Estilo de cor controla só a cor.** A opacidade continua sendo da linha (Fill/Stroke), nunca do estilo.
2. **Alfinete nos cartões restantes** (Object, Appearance, Fill, Stroke, Ordering, Material): só DEPOIS do CI da integração. Plano: `docs/CartoesUI/09_alfinete_nos_cartoes_PLANO.md`.
3. **Emenda: a nota anda com o cartão solto.** Solto, a nota fica no cartão (nenhuma sobra no fim do painel); apagada lá, não volta ao devolver o cartão; se não foi apagada, volta uma vez.

**Why:** o dono decide o produto; estilo que misturasse opacidade tiraria da linha o que é dela, e mexer no alfinete antes do CI arrisca a integração.
**How to apply:** ao tocar estilos de cor, não grave opacidade no estilo; não leve o alfinete a outro cartão antes do CI verde. Handoff: `docs/CartoesUI/handoffs/HANDOFF_INTEGRACAO_NovaUI_2026-11-03.md`.
