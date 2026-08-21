---
name: feedback-one-parameter-two-roles-makes-the-wrong-call-defensible
description: Um parâmetro que serve dois papéis produz o erro que passa na revisão — porque o argumento errado é CORRETO para metade dos usos
metadata:
  type: feedback
---

Quando um parâmetro é lido por dois motivos diferentes, **parta-o em dois**. Não é
estética: enquanto ele for um, existe uma chamada errada que é **defensável**, e ela
vai ser escrita — com um comentário a justificá-la.

**Why:** medido em 2026-08-21 (quad remesher). `fill(reference, layout, …)` usava
`reference` para duas coisas:

1. a tabela de posições que os índices do `layout` indexam
2. a superfície sobre a qual reprojetar o resultado

A porta do produto passou-lhe a malha **original**, raciocinando — **corretamente** —
sobre o papel (2), e escreveu ao lado um comentário a defendê-lo (*"reprojetar sobre
a saída intermédia somaria os dois erros e perderia a silhueta"* — verdade). Mas o
layout tinha sido traçado sobre a malha **intermédia**, com espaço de índice
próprio. Cada índice foi ler a posição de um vértice arbitrário, e o produto voltou
destruído. *Um argumento correto para metade dos usos do mesmo argumento.*

⚠️ **E não havia como falhar alto:** a fase intermédia quase sempre REDUZ a
contagem, então todo índice caía **dentro** do alcance — leitura silenciosa. Nas
entradas em que ela REFINA, o mesmo defeito era `index out of bounds` e matava a
janela.

⛔ **A cura óbvia estava errada.** Trocar o argumento para a malha intermédia
consertava a geometria e apagava, em silêncio, a intenção legítima do papel (2). ⇒
`fill(indexed, surface, …)`. **Um erro que a assinatura torna inexprimível não
precisa de gate.**

**How to apply:**

1. Se um parâmetro é lido em dois sítios por **razões diferentes**, escreva os dois
   nomes e veja se algum chamador os quer diferentes. Basta **um**.
2. ⚠️ **O sinal de alarme é o comentário que justifica o argumento.** Se a chamada
   precisa de um parágrafo a explicar *por que esta malha e não aquela*, o parâmetro
   está a fazer duas perguntas e o chamador só respondeu a uma.
3. ⚠️ **Nenhum teste teria visto.** Em 100 % da cobertura os dois papéis colapsavam
   na **mesma variável**; o produto era o único chamador que os separava — costura
   não-testada ([[feedback_painter_inefficiency_4_causes]], causa nº 1).
4. Onde a assinatura não puder separar, ponha uma **pré-condição que confira a
   correspondência** — aqui, o comprimento de cada arco medido contra o que a fase
   anterior declarou (coerente `1,000` exacto, trocado `5,40×`).

Irmã de [[feedback_a_suite_of_topological_assertions_is_blind_to_geometry]] (foi
por isso que ninguém viu) e de
[[feedback_widely_constructed_type_favors_optional_component_over_appended_field]].
