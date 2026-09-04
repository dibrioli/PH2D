---
name: a-contradiction-between-two-readings-can-be-the-order-the-probe-produced-them
description: Duas leituras de sonda que "não se explicam por hipótese nenhuma" podem ser a ORDEM em que a sonda as produziu, não um mecanismo desconhecido
metadata:
  type: feedback
---

Uma investigação da `line/components` **parou a meio** em 2026-09-02 sobre uma contradição
declarada insolúvel: a sonda media `overrides = 0` numa cópia aninhada acabada de pintar **e** ao
mesmo tempo via a cor da receita interna chegar à cena. O registo dizia *«as duas leituras juntas não
se explicam por nenhuma das hipóteses óbvias»*, e o código da sonda foi apagado.

Em 2026-09-04 a resposta apareceu em cinco minutos, e **não era um mecanismo novo**: era a regra do
1.º encontro, escrita no doc do próprio passe (*«sem eco não há atribuição, e aí o mestre ganha»*).
A sonda pintava **antes** de existir um passe que semeasse o eco ⇒ o passe seguinte não tinha a que
atribuir a diferença e **achatava a pintura**. Com um passe de aquecimento antes de pintar, a
excepção nasce, e as duas leituras passam a ser uma só coisa.

**Why:** um passe com ESTADO (eco, cache, memo, ledger, primeiro-quadro) tem um caminho FRIO e um
caminho quente, e eles dão respostas diferentes de propósito. Uma sonda que corre o passe uma vez
mede o frio e lê-o como se fosse o comportamento. A contradição não estava no produto — estava entre
duas fases do mesmo produto.

**How to apply:** antes de declarar uma leitura inexplicável, pergunte *quantas vezes o passe correu
antes da medição, e o que ele guarda entre corridas*. Se ele guarda alguma coisa, **aqueça-o** (uma
corrida antes de tocar em nada) e volte a medir. E ao arquivar uma investigação parada, escreva a
ORDEM das operações da sonda ao lado das leituras — sem ela a próxima pessoa reconstrói o mistério
em vez da experiência. Relacionado: [[a-ruler-placed-after-the-tidying-step-measures-the-tidying]],
[[a-probe-in-the-failure-branch-cannot-see-the-other-sides-successes]].
