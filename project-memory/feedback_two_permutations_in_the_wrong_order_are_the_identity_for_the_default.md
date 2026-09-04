---
name: feedback-two-permutations-in-the-wrong-order-are-the-identity-for-the-default
description: "Conjugação com as duas permutações trocadas é a IDENTIDADE no caso de omissão — toda fixtura canónica passa, e só o gate do caso NÃO-canónico a vê"
metadata: 
  node_type: memory
  type: feedback
  originSessionId: eed39e8c-c3cb-4514-a6c1-5e9da25f6c30
  modified: 2026-09-02T21:59:12.854Z
---

Numa conjugação `P⁻¹ ∘ f ∘ P`, escrever as duas permutações **na ordem errada** produz código que é
**exactamente correcto** quando `P` é a identidade — que é o caso de omissão, e o caso que
praticamente toda fixtura exercita.

Caso medido (PH2D, `line/3DModeling`, 2026-09-02): o bound novo da composição replicava a
conjugação da `stack::conjugado`. A árvore, **como mapa de ponto**, faz `sai(φ(entra(p)))` — porque
um `remap_xyz` substitui as coordenadas de **quem avalia**, logo a permutação do *lado de fora* age
**primeiro**. Eu escrevi `entra(φ(sai(p)))`. Com o eixo de omissão (`shift = 0`) as duas são a
identidade ⇒ a suíte inteira ficou verde, e o único reprovado foi
`the_law_on_another_axis_is_the_canonical_law_conjugated`.

**How to apply:**
- ⭐ Ao replicar uma conjugação, derive a ordem **como mapa de ponto**, não a leia da forma do
  código: `h.remap_xyz(a,b,c)` avaliado em `p` é `h` avaliado em `(a(p),b(p),c(p))` ⇒ o remap
  **exterior** é o que o ponto atravessa **primeiro**.
- ⛔⛔ **Uma fixtura no caso de omissão não testa uma permutação** — ela testa a identidade. Toda lei
  conjugada precisa de um gate no eixo **não**-canónico, e ele é o único que fala
  ([[feedback_a_corpus_sitting_at_a_knobs_neutral_point_does_not_test_that_knob]]).
- ⚠️ O sintoma é enganador: o campo fica **diferente por eixo**, logo a peça muda de forma quando o
  artista troca o eixo do modificador — e nada acusa, porque cada eixo é internamente consistente.
