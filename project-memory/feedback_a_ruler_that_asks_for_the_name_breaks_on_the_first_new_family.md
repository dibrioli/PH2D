---
name: a-ruler-that-asks-for-the-name-breaks-on-the-first-new-family
description: A bancada perguntou pelo VERBO onde a pergunta é ao GRIP, e quase registei uma divergência de 11× num verbo que shipa há meses
metadata:
  type: feedback
---

Medido em 2026-09-13 (`line/sculpt3d`): a bancada de paridade decidia como
conduzir o gesto com `if brush.verb == Verb::Thumb { total } else { incremento }`.
No dia em que ela passou a correr o **agarrar** — que é o mesmo grip, e também
quer o total — ele recebeu o incremento e leu `0,054` contra os `0,600` do
oráculo. Eu estive a uma frase de escrever no handoff que *«o nosso agarrar
diverge por 11×»*.

**Why:** a pergunta verdadeira era *«este gesto mede o puxão TOTAL ou o
INCREMENTO?»*, e a resposta dela já vive numa porta do produto (`Verb::grip()`).
Perguntar pelo NOME funciona enquanto a lista tem um membro; ela quebra
silenciosamente na primeira família nova — e quebra **para o lado caro**, porque
o número errado parece um achado sobre o produto.

**How to apply:** numa bancada, pergunte sempre à **porta que o produto usa**
para decidir, nunca ao nome do caso. E quando uma comparação com o lado aprovado
der um número grande e redondo (`11×`, `2,5×`), gaste os quatro minutos do
**discriminador mais barato** antes de acreditar nele: aqui foi entregar o mesmo
puxão total em `1`, `2`, `4` e `12` eventos — as quatro corridas deram
`0,600000`, e isso disse que o motor estava certo e a régua errada. *Uma
discordância com o oráculo é uma hipótese sobre o alvo **ou** sobre a nossa
régua, e a régua é a mais barata de conferir primeiro* — ver
[[reference-topic-measurement-discipline]].
