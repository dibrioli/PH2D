---
name: feedback-a-symmetric-fixture-approves-a-law-that-distinguishes-nothing
description: Quatro vezes na mesma wave uma fixtura simétrica deixou passar uma lei que não distinguia nada — e o sinal é o parâmetro não ter efeito
metadata:
  type: feedback
---

Ao curar o entalhe do cotovelo, **quatro** fixturas aprovaram, cada uma, uma lei que não
distinguia nada:

1. com a junta no MEIO da barra, o centróide calha nela e o estimador devolvia o mesmo ponto para
   **todo** peso — `σ²` de `2e-5` a `100` dava o mesmo número;
2. com **dois** ossos há um par só, e o peso dele cancela-se na normalização ⇒ `wᵢ·wⱼ` e `wᵢ+wⱼ`
   eram indistinguíveis;
3. com os dois ossos a rodar em torno da mesma junta, ela **não se mexe** ⇒ `blend_linear(junta)` e
   `junta` eram o mesmo ponto;
4. sem um osso DOBRADO, o eixo do sub-osso é o do osso inteiro.

**Why:** uma simetria colapsa duas grandezas numa, e a fixtura deixa de poder falar sobre a
diferença. Três destas só apareceram por **mutação sobrevivente** — nenhuma revisão as via.

**How to apply:** ⭐ o sinal mais barato é **varrer o parâmetro**: se `σ` de `2e-5` a `100` dá o
mesmo número, o parâmetro não está a fazer nada, e o problema é a fixtura antes de ser a lei. E ao
escrever a fixtura, pergunte o que ela iguala por acidente — ponha a junta fora do centro, três
membros em vez de dois, pesos assimétricos, e o sujeito a MOVER-SE. Relacionado:
[[feedback-a-corpus-at-full-strength-cannot-test-the-strength-curve]] ·
[[reference-topic-fixture-discipline]] · [[reference-topic-mutation-proofs]]
