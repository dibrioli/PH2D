---
name: a-ruler-normalised-by-what-the-cure-zeroes-measures-it-backwards
description: Uma régua cujo DENOMINADOR é a grandeza que a cura leva a zero imprime números absurdos e lê a cura ao contrário — escolha o denominador antes de acreditar no resultado
metadata: 
  node_type: memory
  type: feedback
  originSessionId: 7c66683a-d39b-477a-ad5a-a6529d503e36
  modified: 2026-08-29T15:37:33.685Z
---

Ao medir se uma cura funciona, **o denominador da régua não pode ser a grandeza que a cura
existe para anular**.

Caso real (L-System, 2026-08-29). A pergunta era *"o `Step Scale` torna o crescimento suave?"*.
A régua foi `pior_passo / subida_total`, que é a certa para uma planta que CRESCE. Mas o
`Step Scale` existe precisamente para deixar uma figura de refinamento **do mesmo tamanho** —
logo a subida vai a ~0 e a razão explode. A varredura imprimiu **619 050 %** para a
configuração que era a candidata a cura, e **69 %** para a que não fazia nada: a régua
**recomendava não curar**.

A cura da régua foi trocar o denominador para a **MÉDIA** da grandeza (não a variação dela):
`pior_passo / média_do_tamanho`. Aí Bush deu 105 % em vez de 619 050 %, e a conclusão honesta
apareceu — o `Step Scale` estabiliza o tamanho e **não** torna contínua a forma, que são duas
coisas diferentes.

**Why:** um número absurdo (`619 050 %`, `1e17`, `NaN`) num relatório de medição quase nunca é
o fenómeno: é o denominador a passar por zero. E o modo de falha caro não é o número absurdo —
é a linha VIZINHA, que parece plausível e ordena a decisão errada.

**How to apply:** antes de correr uma varredura, pergunte *"o que esta cura leva a zero?"*. Se
a resposta aparecer no denominador, troque-o por uma escala que a cura **não** move (a média, o
valor de referência, o tamanho da peça). E trate todo valor com ordem de grandeza fora da
tabela como suspeita da régua, nunca como descoberta — relacionado:
[[feedback_an_unlabelled_probe_column_gets_read_backwards]] e
[[feedback_ask_what_number_the_opposite_answer_would_print]].
