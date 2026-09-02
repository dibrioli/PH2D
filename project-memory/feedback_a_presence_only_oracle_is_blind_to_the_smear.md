---
name: a_presence_only_oracle_is_blind_to_the_smear
description: Oito gates que só perguntavam «este pixel tem a cor certa?» deixaram passar uma peça a manchar a coluna inteira — o oráculo tem de exigir VAZIO fora da peça
metadata:
  type: feedback
---

2026-09-01, retrato do prefab: com o `v` recortado antes do teste, cada peça pintava toda a
altura da sua banda de colunas. Os oito gates (as duas peças aparecem · não está espelhado ·
não estica · determinístico · a célula viva · o pivô …) estavam VERDES, porque todos perguntavam
por presença em sítios que a mancha também pintava certo. A mutação que repõe o `clamp` sangra
**só** o gate novo — que pergunta a coluna central a meia altura do preto (tem de ser PRETA) e a
borda acima do preto (tem de ser TRANSPARENTE).

**Why:** presença e ausência são metades independentes; a segunda é a que uma mancha, um smear ou
uma peça a mais violam, e é a que quase nunca se escreve. Irmã de
[[absence_gate_needs_a_presence_sibling]] no sentido inverso.

**How to apply:** todo gate sobre uma composição afirma pelo menos um pixel que tem de ficar
VAZIO — fora de todas as peças, dentro da caixa. Ver
[[a_clamp_before_a_range_test_deletes_the_test]].
