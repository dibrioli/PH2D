---
name: feedback-the-representation-can-delete-the-special-case
description: Caso especial teimoso na referência costuma ser artefato da REPRESENTAÇÃO dela — troque a forma e ele some (com a verruga junto)
metadata: 
  node_type: memory
  type: feedback
  originSessionId: 08ec2d6a-35b1-4812-a467-769e905c0028
---

Quando a referência descreve um algoritmo com casos especiais ("cíclica sem corte tem ZERO
segmentos → *fallback*", "o último segmento enrola em DOIS ranges"), pergunte se os casos são
do **problema** ou da **representação** dela. Muitas vezes são dela.

No Segment mode do Flip (ADR-0114 §4.B) o Blender descreve pedaços por `IndexRange`
(begin..end). Essa escolha obriga: um `fallback` explícito quando não há corte, dois ranges
quando o pedaço cruza a costura, e um `clamp_range(p + 1)` que **satura** — e a saturação
entrega o último ponto ao pedaço errado quando o corte cai na costura (`{2}`+`{0,1,3}` onde a
geometria manda `{2,3}`+`{0,1}`). Trocando para um **vetor de donos** (`dono[p]` = id do
pedaço), os três somem: sem corte todo mundo tem dono 0 (o fallback é o caso geral, não um
`if`), o pedaço que enrola é o mesmo dono em duas corridas do vetor, e não há range para
saturar — logo a verruga não tem onde existir.

**Why:** casos especiais portados sem exame viram `if`s que ninguém entende, e a verruga vem
de brinde. O caso especial é uma pista de que a forma do dado está errada para o problema.

**How to apply:** antes de portar, escreva a RESPOSTA que o algoritmo produz (o conjunto, o
mapa) e pergunte que estrutura a expressa **sem** ramo. Se a sua forma apaga o caso especial,
gateie a divergência — mas saiba o que o gate guarda: não é um `if` deste arquivo (não há), é
a **escolha de representação**, e ele fica vermelho se alguém reintroduzir a forma antiga.
Diga isso no doc, senão a próxima linha "conserta" de volta para a referência. Ver
[[reference_topic_mutation_proofs]] e
[[feedback_before_declaring_the_design_rejects_an_invariant_grep_for_its_gate]].
