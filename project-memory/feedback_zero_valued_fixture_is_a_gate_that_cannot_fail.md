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

---

## Adendo 2026-08-25 — o zero pode ser uma POSIÇÃO, não um valor de param

Ao medir o `Use Layer as Seed` do `motion.noise` empilhei oito peças **na origem** e li
envergadura `0,000000` nas duas metades — com o modo ligado e desligado. A conclusão
*"elas partilham o campo"* estava **certa pelo motivo errado**: a origem é um **ponto de
rede** do ruído de gradiente, e ele vale **zero ali para todo seed, por construção** (o
`lib.rs` do nó já o dizia em prosa: *"gradient noise is zero at every lattice point"*).
A fixtura não estava num valor neutro de param nenhum — estava sobre o **zero da própria
função**. Bastou deslocá-la `(0,37; 0,21)` para a cura aparecer: `0,000000` → `0,798292`.

⇒ **A pergunta «qual constante esconde o erro?» tem de incluir o DOMÍNIO, não só os
params.** Uma fixtura pousada num zero, num ponto fixo ou numa simetria da função sob
teste é neutra sem que nenhum número escrito no teste seja neutro.
