---
name: feedback-a-cure-may-not-need-the-cause-ask-a-different-question
description: "Duas explicações de um 3,9× caíram por medição e a cura veio de outra pergunta — o custo saiu de 1 406 ms para 74 sem a causa ter sido achada"
metadata:
  type: feedback
---

**Perseguir a CAUSA de um custo pode ser mais caro do que o remover.** Medido 2026-09-21 na `W9` do
Render3d: o `pinta_bordas` custava `1 360 ms` a compilar contra `347` do `pinta`, do MESMO texto
(`3,9×`). Duas explicações foram construídas e refutadas — o desenrolar do laço das quatro
sub-amostras (com **uma** ainda custa `1 213 ms`) e o laço em si (**sem laço nenhum**, `1 088 ms`).
A causa continua sem endereço. ⭐ **E a cura veio de outra pergunta: *porque é que a peça está dentro
do shader?*** — o grafo de chamadas do WGSL disse que aquele passe alcança a fita da peça por **um
caminho só**, atrás de uma guarda que o valor de fábrica não abre. Tirá-la fez o texto deixar de
mudar com a peça, o cache passar a acertar, e **`1 406 ms → 74 ms`** (`19×`), sem que o `3,9×` fosse
explicado.

**Why:** as duas primeiras perguntas eram *«porque é que este pedaço é caro?»*, que leva a afinar; a
terceira foi *«porque é que este pedaço existe neste sítio?»*, que leva a removê-lo. ⚠️ **As duas
refutações não foram desperdício** — elas é que fecharam o caminho da afinação, e é por isso que a
recusa fica escrita com os números ao lado.

**How to apply:** quando duas hipóteses de causa caem sobre um custo, **pare de procurar a causa** e
pergunte o que põe aquele trabalho ali. E prefira a cura que muda a **CHAVE** de um cache à que
muda o conteúdo dele: a primeira prova-se por igualdade (a imagem é a mesma) e a segunda precisa de
entender o compilador de outra pessoa. ⛔ A régua desta espécie de cura é sempre a **CONTA** e nunca
o valor — as duas rotas dão a mesma saída, que é o que a torna segura e o que a torna impossível de
medir por valor (*uma cura apagada também não muda a saída*). Ver
[[reference_topic_measurement_discipline]] · [[reference_topic_gate_discipline]] ·
[[feedback_a_ruler_whose_parameter_sits_inside_a_clamp_measures_the_clamp]].
