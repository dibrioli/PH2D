---
name: feedback-specialisation-pays-by-amortisation-count-the-rays-per-region
description: Compilar uma árvore por região só compensa se houver trabalho suficiente naquela região — conte quantos raios amortizam a montagem antes de estender a especialização a uma segunda passagem.
metadata:
  type: feedback
---

A W56e especializou a marcha primária do traçado por **ladrilho × fatia de profundidade** e ganhou
`2,5×`. A segunda passagem (a re-amostragem das bordas do anti-serrilhado) ficou a marchar a árvore
**inteira** — e parecia um esquecimento óbvio a corrigir. ⛔ **Medido A/B no mesmo processo: neutro
a pior, e revertido.**

| arestas | s/AA | c/AA simples | c/AA por ladrilho |
|---|---|---|---|
| 64 | 29,0 ms | **36,4 ms** | 39,0 ms |
| 256 | 86,8 ms | **105,9 ms** | 109,7 ms |

**Why:** *a especialização paga-se por AMORTIZAÇÃO.* A montagem da fita custa o mesmo por ladrilho
nas duas passagens; a primária dilui-a por **4 096** raios (`64×64`) e a de borda só tem **~256**
naquele ladrilho — a silhueta atravessa-o em ~64 pixels × 4 amostras. **16× menos raios para
amortizar a mesma montagem**, e é essa razão que come o ganho da árvore menor.

**How to apply:** antes de estender uma compilação-por-região a um segundo consumidor, **conte os
itens que ele tem por região** e compare com os do primeiro. Se a razão for de uma ordem de
grandeza, o ganho da árvore menor não paga a montagem — e a saída, se houver, é **reaproveitar** a
fita já montada (remove a montagem em vez de a diluir), não montá-la outra vez.

⚠️ E o ponto de partida também era falso: a nota dizia que o anti-serrilhado tinha tornado o traçado
`2,4×` mais caro. Ele custa **22–34 %**, e o traçado está a `1,2×` da medição original — as waves de
perf seguintes tinham desmentido a nota e ninguém a reconferiu
([[feedback-a-measured-refusal-answers-one-question-recheck-it-when-yours-is-another]]).
