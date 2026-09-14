---
name: feedback-a-collection-ordered-by-distance-is-not-ordered-by-adjacency
description: Ligar pontos pela ordem do vector desenhou cordas através da peça — a lista vinha ordenada por DISTÂNCIA, não por vizinhança
metadata:
  type: feedback
---

A cadeia de vértices de um pincel de borda é devolvida ordenada por **distância à
âncora**, porque a lei anda a borda **nos dois sentidos ao mesmo tempo**:
`[94, 0, 92, 1, 90, 4, …]`, com distâncias `[0, 0,131, 0,131, 0,262, 0,262, …]`.
Medido, **45 de 47** pares consecutivos **não** são vizinhos de borda.

Uma polilinha construída com `windows(2)` sobre essa lista não desenha a borda:
desenha um ziguezague de **cordas atravessando** a peça.

**Why:** «a lista dos vértices afectados» e «o caminho que os liga» são duas
perguntas diferentes, e um `Vec` responde à primeira parecendo responder às duas.
⛔ **E as réguas óbvias ficam verdes:** a contagem de pedaços é a mesma, os pesos
são os mesmos, o laço fecha na mesma. *Quem desenha uma LIGAÇÃO tem de gatear a
RELAÇÃO — que estes dois pontos são de facto vizinhos —, nunca o número de
linhas.*

**How to apply:** antes de ligar elementos pela ordem de um contentor, pergunte o
que define essa ordem; se não for a adjacência, reconstrua o percurso pela
relação (um passeio pela vizinhança) e escreva o gate que verifica cada ligação.
Irmão de [[feedback_a_gate_only_proves_what_its_fixture_contains]].
