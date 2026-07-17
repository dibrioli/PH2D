---
name: feedback_mirroring_time_must_mirror_the_shape
description: Inverter keys nao e espelhar tempos; o interp mora no key de SAIDA, entao ele muda de dono E se espelha
metadata:
  type: feedback
---

Inverter um clipe (`Track::reverse_about`) não é `t -> span - t`. O `Interp` descreve o segmento
que **sai** de um key, então a inversão (a) move cada interp para outro key — o segmento entre
`i` e `i+1` vira o que sai de `n-2-i` — e (b) **espelha a forma dele**.

**Why:** espelhar só os tempos deixa a curva com as acelerações antigas enquanto os valores
correm ao contrário: todo ease-out vira ease-in. Isso é exatamente o que um animador inverte um
clipe para **preservar**. E é invisível num teste que só verifique "o primeiro key virou último".

**How to apply:** ao espelhar/rotacionar/reordenar uma estrutura em que um elemento descreve a
**relação** com o vizinho (interp, aresta, junção, transição), o atributo troca de dono e se
transforma — dois passos, não um. O oráculo é a **propriedade**: `reversed(span - t) ==
original(t)` ponto a ponto, por variante. Pegadinhas achadas assim: `BezierW.dy` são offsets das
**âncoras**, e a inversão troca as âncoras (o dy viaja com a alça, não é negado); e `1-(1-x)`
não volta em `f64` — o round-trip compara a CURVA e o **kind** do segmento, nunca os bits.
Ver [[reference_topic_oracle_discipline]], [[feedback_test_with_product_numbers_not_convenient_ones]].
