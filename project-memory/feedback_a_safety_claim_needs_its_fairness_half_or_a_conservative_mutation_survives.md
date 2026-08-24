---
name: feedback-a-safety-claim-needs-its-fairness-half-or-a-conservative-mutation-survives
description: Um gate que só afirma «isto é seguro» deixa passar toda mutação conservadora — falta a metade «e é justo»
metadata:
  type: feedback
---

Gate: *"o passo da marcha vezes o pior `‖∇f‖` nunca passa de 1"* — a afirmação que impede a peça de
furar. Uma mutação que classificava mais um construtor como perigoso (passo curto onde não era
preciso) **sobreviveu**: ela só torna a marcha mais lenta, e o lado seguro fica ainda mais seguro.

**Why:** toda classificação de segurança tem duas metades. *Segura*: quem é perigoso é tratado como
perigoso. *Justa*: quem **não** é não paga. Gatear só a primeira deixa a regressão de desempenho —
que é exactamente o defeito que a wave veio curar — sem defesa nenhuma.

**How to apply:** ao gatear um limite derivado, escreva as duas asserções. E cuidado com a
tautologia: a metade justa não pode ser a implementação escrita ao contrário. Aqui ela mede
(`‖∇f‖ ≤ 1 ⇒ passo inteiro`), exclui a família cuja reserva é do **construtor** e não do valor de uma
fixtura, e em troca gateia que essa reserva é **merecida** (a família tem de medir acima da barra).
Irmã de [[feedback-a-claim-no-mutation-can-kill-is-a-claim-about-nothing]].
