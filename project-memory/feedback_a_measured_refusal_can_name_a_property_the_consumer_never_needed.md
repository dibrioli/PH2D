---
name: feedback-a-measured-refusal-can-name-a-property-the-consumer-never-needed
description: «A distância a uma espiral não é fechada» barrou duas formas durante meses — e o módulo nunca precisou da distância exacta, só de um MINORANTE; as duas ficaram feitas numa tarde.
metadata:
  type: feedback
---

⭐⭐⭐ Medido em 2026-09-05 (W123, doc 06 §124), a pedido do dono: *«usando fórmulas não ficam mais
leves? Implemente»*.

O plano de formas do modelador dava a **espiral** e a **base ondulada do Document** como *«tem de
ficar desenhada: a distância a uma espiral de Arquimedes / a uma senóide não é fechada»*. A
afirmação é **verdadeira**. E a propriedade que ela nomeia **não é a que o consumidor pede**: uma
marcha de esferas precisa de um **minorante** da distância — andar a menos custa passos, andar a
mais atravessa a superfície. O gate do módulo diz `passo × ‖∇f‖ ≤ 1`, nunca `f = dist`.

⭐ Para uma curva implícita o minorante é uma linha de álgebra: `|g| / max‖∇g‖ ≤ dist`. Na onda,
`‖∇g‖ = √(1 + base'(x)²)` tem um **máximo constante**, logo a divisão é rigorosa em todo o plano —
não uma aproximação com erro. **A superfície é a senóide ao bit**; conservador é só o valor longe
dela.

⚠️ E o preço de as ter deixado desenhadas estava medido no mesmo repo desde 28/08: o **mesmo**
cilindro custa `1,79 ns/ponto` por fórmula e `181,44 ns` desenhado com 192 lados (**`101×`**).
⭐⭐ Na espiral, `passo × ‖∇f‖` fica em `0,9899` de **1 a 32 voltas**: *o campo de uma espiral não
sabe quantas voltas ela tem*, enquanto um contorno paga por segmento.

**Why:** uma recusa medida guarda **a pergunta que lhe foi feita**, e o leitor seguinte lê a
conclusão sem a pergunta. Aqui a pergunta era *«a distância é fechada?»* e a do módulo é *«o campo
promete mais do que anda?»*.

**How to apply:** antes de aceitar uma recusa que barra trabalho, escreva **a propriedade que o
consumidor de facto exige** e compare-a com a que a recusa nomeia. Se forem diferentes, a recusa não
responde. Irmã directa:
[[feedback_a_measured_refusal_answers_one_question_recheck_it_when_yours_is_another]] ·
[[feedback_not_every_inexactness_is_danger_the_one_that_underestimates_is_slack]] ·
[[reference_topic_implicit_field_laws]]
