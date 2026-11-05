---
name: project_smooth_edges_per_brush_owner_decision_2026_11_04
description: "Dono 04/11: Smooth Edges por traço, um valor por pincel (revoga a D1 de 27/10); Wet Liner e Wet Wash 0,12 + Edge Darkening 0; consertar o Grain escolhido à mão"
metadata:
  type: project
---

Dono 04/11, depois de ver o 0,12 geral na tela: «Smooth Edges deve ser por traço mesmo e cada pincel deve ter o seu. assim não perde detalhes.» Wet Liner e Wet Wash: Edge Darkening 0 e Smooth Edges 0,12 por padrão. Consertar também o Grain escolhido à mão (10 pincéis de cerdas com Grain próprio).

**Why:** o 0,12 na tela inteira gasta o que é fino: pó de pastel 14,6 grãos/100 px (Rebelle 56), grão de 2H/HB, traço fino. No mesmo dia ele tinha pedido «deixe no papel» e 0,12 geral; esta é a palavra final. Revoga a D1 de 27/10 («para todo o papel», `docs/Painter/49_realshader/AUDITORIA_W3b.md:5`), mas continua a valer dela que o efeito age na cor E no relevo.

**How to apply:** valor na célula, pousado com o traço (padrão: `AparenciaCelula`; a tentativa de `65d6a6a16d` custou 4 B/célula), lido na composição; controlo no cartão do pincel, gravado no preset. O painel não recompõe o que já está pintado. Os outros pincéis em 0 é a leitura proposta ao dono, à espera da confirmação. Medições: [Bug #61 e #62](../docs/Painter/BUGS_painter.md).
