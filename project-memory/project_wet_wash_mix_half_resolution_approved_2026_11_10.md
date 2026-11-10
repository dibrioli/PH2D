---
name: project_wet_wash_mix_half_resolution_approved_2026_11_10
description: Dono 10/11 aprovou a mistura do Wet Wash Mix a meia definição (Bug #84 cura 6): ~1,8× em traço rápido, o grão fino com metade da força
metadata:
  type: project
---

O dono aprovou («Aprovado», 10/11) a cura 6 do Bug #84: no Wet Wash Mix a pegada e a película da mistura correm numa
tela de metade do lado e o plano do traço é ampliado (`MisturaOpaca::meia_definicao`, só a receita liga, sem UI).
360 px/quadro 43,5 → 24 ms; o grão fino fica com metade da energia — o dono viu o lado a lado e aceitou.

**Why:** o véu largo e macio não mostra o detalhe de 1–2 px que a meia definição come; a velocidade em traço rápido
era a queixa dele (6 fps).

**How to apply:** estender a outro pincel só se for LARGO e MACIO e lento à velocidade do dono (9 × 40 px), mostrado
lado a lado e aprovado pincel a pincel — receita em `docs/Painter/Pinceis/COMO_ACELERAR_UM_PINCEL.md` §5. Nunca em
pincel de grão fino, dente do papel, borda dura ou píxel. Ver [[project_brushes_as_they_are_are_the_reference_2026_11_05]].
