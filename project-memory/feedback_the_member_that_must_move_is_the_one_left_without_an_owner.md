---
name: feedback-the-member-that-must-move-is-the-one-left-without-an-owner
description: Ao dar dono a um grupo de variaveis restringidas, a RAIZ e' a que fica sem dono — precisamente por ser a que tem de se mexer.
metadata:
  type: feedback
---

Quando se impõe uma relação sobre um grupo de escalares, marca-se cada membro como
«conduzido» para o relaxador antigo não lhe tocar — **e exclui-se a raiz**, porque ela *é*
a que tem de se mexer. ⛔ Mas a raiz também passa a ser escrita pela lei nova; excluí-la
deixa **duas leis sobre o mesmo escalar**, com denominadores diferentes.

**Why:** medido no `ph2d-gridmap` (2026-08-27). A `attach_ties` fazia `if x != root`. Uma
raiz que fosse incógnita **livre** ficava travada por outro caminho (`freeze_free`), então
o defeito só aparecia nas raízes de classe **simples** — `6` delas, e exactamente `6`
pregos com passo `NaN`. Removida a condição: não-finitos de `3 119` para **`0`**, e a
extracção deixou de ser recusada.

**How to apply:** *marcar a raiz é inócuo quando a lei nova a escreve de qualquer modo* —
a marca só fecha a porta da lei ANTIGA. Ao escrever «todos menos o dono», pergunte quem
escreve o dono **pela outra porta**, e conte a população em que as duas travas não se
sobrepõem: é aí que o defeito vive. Parente de
[[feedback-a-constraint-imposed-in-one-phase-and-not-the-next-is-a-starting-point]] e
[[feedback-a-denominator-above-the-curvature-is-slow-below-is-inf]].
