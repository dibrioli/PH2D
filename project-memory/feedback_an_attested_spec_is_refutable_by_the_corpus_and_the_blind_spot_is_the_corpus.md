---
name: feedback-an-attested-spec-is-refutable-by-the-corpus-and-the-blind-spot-is-the-corpus
description: Uma fórmula de espec clean-room ATESTADA caiu por medição — ela concorda com a certa em toda fixtura CENTRADA na origem
metadata:
  type: feedback
---

A espec do pincel de contorno, atestada à 3.ª passagem do R-pré, dava uma fórmula
para a direcção contra a qual o arrasto é projectado. **Quatro** fixturas do
corpus refutam-na; a lei que o corpus impõe é `n̂ = −unit(ponto_origem)`, e com ela
a paridade foi de **43 para 49** de 61.

**Why:** as duas fórmulas dão a **mesma resposta** em toda peça **centrada na
origem** — que era a maioria do corpus. O revisor não tinha como as separar: o
ponto cego é do **CORPUS**, não do auditor. *Uma espec atestada é uma afirmação
sobre o que o revisor podia ver.*

**How to apply:** (1) uma espec atestada não é oráculo — o **corpus** é; quando os
dois discordam, meça e declare a divergência no código, com a tabela das fixturas
que a separam ao lado. (2) Ao escrever ou auditar uma espec, procure a **família
de casos degenerados** em que as candidatas coincidem (origem, escala 1, simetria,
ponto neutro de um knob) e pergunte se o corpus tem algum caso FORA dela — se não
tiver, a espec não foi testada nesse ponto, foi apenas não-contradita. Irmão de
[[feedback_a_corpus_at_a_knobs_neutral_point_does_not_test_that_knob]].
