---
name: feedback-layered-defenses-need-per-layer-gates
description: "Fix com defesas redundantes — a mutação de UMA camada não sangra (a outra segura); cada camada exige o PRÓPRIO gate, e o gate-repro documenta que só sangra com todas removidas"
metadata: 
  node_type: memory
  type: feedback
  originSessionId: 92714982-3cf5-48f6-96d6-acbdbe13b4f5
---

No P0 do Inflate (2026-07-15), o fix tinha 3 camadas independentes (sentinela + orçamento por-fonte +
taper). Mutei a sentinela: o gate-repro ficou VERDE — o orçamento segurava sozinho. Mutei o cap via
`if false &&`: verde de novo — o clamp DENTRO da fórmula do taper já era o cap. Nenhum comentário de
mutação estava certo.

**Why:** defesas em camadas são boas de ter e péssimas de gatear ingenuamente: a mutação de uma camada é
absorvida pela vizinha, o gate "que não sangra" parece frouxo, e a tentação é remover a camada
"redundante" — que na verdade protege um caso que o gate-repro não encena (a sentinela não segurava o
retângulo; segurava o SOMBREAMENTO — uma parede não-tocada vencendo e calando a fonte legítima).

**How to apply:** ao provar mutações de um fix multi-camada: (1) pergunte O QUE cada camada protege
sozinha e escreva um gate POR CAMADA com a fixture que só ela salva; (2) no gate-repro principal,
documente que ele só sangra com TODAS removidas — e cite o born-red do código original como a prova;
(3) uma mutação que não sangra em fix novo = ou o comentário mente, ou está faltando o gate da camada
([[feedback_a_mutation_that_survives_may_mean_a_missing_gate]] — este é o caso "gate faltando",
sistematizado).
