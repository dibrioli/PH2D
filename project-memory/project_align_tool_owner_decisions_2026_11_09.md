---
name: project_align_tool_owner_decisions_2026_11_09
description: "Dono 09/11: ferramenta Align no editor de animações — por folha (deslocamento+escala+Pivot) e por quadro (deslocamento), SEM giro, sem nível por animação; onion = anterior + referência fixada"
metadata:
  type: project
---

Dono 09/11 aprovou o estudo `docs/GameEngineFuncional/28_alinhar_folhas_estudo.md`: **sem giro**; acerto por folha e por célula
(o deslocamento da célula mora na FOLHA, vale em toda animação); nenhum nível «por animação». Onion skin delegado: quadro anterior
ténue + quadro de referência fixado (1.º da animação de entrada), cada um com o seu botão. Manteve o ponto de encaixe do pivô no
centro do desenho.

**Why:** folhas de tamanhos diferentes desalinham os pés; sem ver um quadro sobre o outro não se alinha (lição do Aseprite).

**How to apply:** ondas A (dados+render, sobe o schema) → B (prévia Align) → C (por quadro, «Alinhar pelos pés») → D (cena + fotos).
