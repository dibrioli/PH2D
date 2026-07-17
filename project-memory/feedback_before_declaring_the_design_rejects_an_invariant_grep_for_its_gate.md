---
name: feedback-before-declaring-the-design-rejects-an-invariant-grep-for-its-gate
description: "Antes de escrever num gate que 'o design rejeita este invariante', grepe — o repo pode já ter um gate afirmando o contrário, e quem está errado é você"
metadata: 
  node_type: memory
  type: feedback
  originSessionId: b294ecd6-99c8-41cf-ac4b-c6001c30b1c7
---

Escrevi, com todas as letras, num doc comment de gate: *"'o mesmo caminho, 2 amostragens, é a mesma
figura' é verdade do DEPÓSITO e **falso do sculpt por design** — `amount[i] += w` é SOMA, não envelope,
então demorar esculpe mais, como no Blender."* Confiante, plausível, e **errado**.

O motor **espaça dabs por DISTÂNCIA**: a lista de dabs é IDÊNTICA em qualquer taxa de polling — só o
*batching* muda. A soma é sobre dabs **sobrepostos ao longo do caminho**, não sobre a taxa do mouse. Eu
misturei duas coisas que compartilham a palavra "mais".

E o repo **já tinha o gate**: `a_faster_mouse_does_not_sculpt_deeper` (`sculpt_tests.rs:200`, W1) afirma
exatamente o invariante que eu declarei rejeitado — para a ALTURA, com o raciocínio inteiro escrito, e
até com o motivo de usar toque LEVE (na saturação o bug se esconde). Um mouse de 1000 Hz não pode
esculpir mais fundo que um de 125 Hz.

O que me levou ao erro: o gate que eu tinha escrito **passou contra código quebrado**, e em vez de
perguntar *por que ele não alcança o bug* eu inventei uma teoria de por que ele **não deveria** existir —
e a promovi a documentação. Racionalizar um gate verde é mais perigoso que apagá-lo: o comentário vira
uma lei falsa que o próximo agente vai obedecer.

(O gate certo era o **irmão** dele para a MATÉRIA — `a_faster_mouse_does_not_grow_a_different_rim`. A
advecção é a única escrita do sculpt que o gate da altura não enxerga. Mutação do pixel vivo: 45/255 de
diferença entre um mouse lento e um rápido.)

**Why:** "o design rejeita X" é uma afirmação sobre o SISTEMA, não sobre o seu diff — e o sistema tem
memória escrita (gates, ADRs, doc comments). Alegar isso sem grepar é inventar uma cerca de Chesterton
onde já existe uma placa dizendo o oposto ([[feedback_documented_decision_chesterton_fence]]). O custo
não é o gate perdido: é a mentira que fica no comentário.

**How to apply:** antes de escrever *"o design rejeita/permite deliberadamente X"* num gate ou handoff,
**grepe pelo invariante** (`grep -rn "faster\|coarse\|same path\|idempot"` nos testes do módulo). Se
existir um gate afirmando o contrário, ele ganha até você provar o contrário no código. E quando um gate
seu passa contra código quebrado, a resposta é **instrumentar e CONTAR**
([[feedback_a_mutation_that_survives_may_mean_a_missing_gate]]), nunca teorizar por que ele estava certo
em passar.
