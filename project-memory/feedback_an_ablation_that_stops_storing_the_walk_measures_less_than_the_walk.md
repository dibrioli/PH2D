---
name: feedback-an-ablation-that-stops-storing-the-walk-measures-less-than-the-walk
description: "Uma ablação que deixa de GUARDAR o que um laço calcula mede menos do que o laço custa — o compilador corta a aritmética que ninguém lê; o plano do §9.18 (E) contava com 0,084 ms e o passeio guardado mede 0,130"
metadata:
  type: feedback
---

O plano da emissão por peça (doc 121 §9.18 E, 05/10) pôs o custo do passeio de cada cópia pelos troços no `E1F`
do §9.17 (`0,084` ms, iGPU) — uma ablação que tirava a emissão e com ela tudo o que o passeio produzia. Medido
com o passeio a GUARDAR o que anda (`n0`, `n1`, o prefixo das peças, segurado contra o compilador por uma escrita
que nunca acontece): `0,130` ms — o limite inteiro do modelo, e a recusa do (E).

**Why:** um shader (e um compilador em geral) elimina o cálculo cujo resultado ninguém lê; a ablação que corta a
SAÍDA mede o laço sem a aritmética dele, e a premissa construída sobre esse número cai na prova.

**How to apply:** toda ablação que remove um consumidor segura o cálculo que fica com um efeito que o compilador
não prova inútil (`if x < -1e30 { buf[0] = x }`), e escreve no doc QUAL saída ela guarda. Ao herdar um número de
ablação para uma premissa, confira se a ablação guardava o que a premissa precisa. Família:
[[reference_topic_measurement_discipline]].
