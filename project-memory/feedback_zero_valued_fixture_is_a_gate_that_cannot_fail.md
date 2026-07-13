---
name: feedback-zero-valued-fixture-is-a-gate-that-cannot-fail
description: Fixture com o valor NEUTRO (0, vazio, identidade) faz o gate passar mesmo com a feature quebrada — o zero é o único valor que esconde o erro
metadata:
  type: feedback
---

Os gates da alça de fade (B4) usavam uma fixture com **cunha = 0** e **fade = 0**. Resultado: o
`blend_px` (que ancora a alça na ponta da cunha) **nunca era executado**, e o `start_ease` era
sempre 0 — então os dois gates de arrasto passariam com a alça ancorada no lugar errado E com o
drag ignorando a fade que já existe (o bug que o `arch_no_absolute_drag_pattern` existe para
pegar). Bastou dar 0,25 s de fade à fixture: **os dois ficaram vermelhos na hora**.

**Why:** o zero é o ponto fixo de quase toda transformação — `x + 0`, `x * 1`, `offset(0)`,
`lerp(a, a)`. Um teste no zero não distingue "a função certa" de "a função que devolve a entrada".
É o mesmo erro de [[feedback_test_with_product_numbers_not_convenient_ones]] (`px_to_world = 1.0` é
o único valor que esconde erro de unidade), e de [[feedback_gate_the_edges_of_the_domain]].

**How to apply:** toda fixture nasce com valores **não-neutros e diferentes entre si** (0,25 e
0,4, não 0 e 0; 2 e 3, não 1 e 1). Antes de aceitar um gate verde, pergunte: *"qual constante eu
poderia zerar no código sem que este teste percebesse?"* — se existe uma, a fixture está no ponto
fixo dela. E depois **mute o código** para confirmar ([[feedback_mutate_the_code_not_just_the_test]]).
