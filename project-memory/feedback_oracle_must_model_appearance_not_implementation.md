---
name: feedback_oracle_must_model_appearance_not_implementation
description: "Teste-oráculo derivado do código só pega regressão; alvo de aparência tem de modelar a APARÊNCIA desejada, senão fica verde com o bug na tela"
metadata:
  node_type: memory
  type: feedback
---

Quando escrever um oráculo (teste que compara o render com um modelo analítico na
CPU), **derive o esperado da APARÊNCIA desejada, não da implementação**. Um oráculo
que replica o que o shader faz é apenas um detector de regressão: ele fica **verde
com o bug bem visível na tela**.

**Caso que ensinou (Flip, 2026-07-11):** escrevi
`ph2d-flip-render/tests/gpu_render.rs::assert_matches_analytic` replicando na CPU a
geometria do vertex + a máscara do fragment + a regra de depth (first-wins). 9
testes verdes, 2 mutações provadas (asserção-vermelha real) — e o smoke do Enio
reprovou na hora: as quinas saíam **mordidas**. A mordida É o first-wins; o oráculo
tinha codificado o bug como verdade. O esperado CORRETO era o **máximo** da máscara
sobre os segmentos que cobrem o pixel (a união — o que o olho espera de um traço),
não "o que o primeiro segmento pinta".

**Como aplicar:** antes de escrever o esperado, pergunte *"esta fórmula sai da
FÍSICA/aparência que quero, ou de reler o meu shader?"* Se sai do shader, o teste
não pode falhar por design errado — só por typo. Escreva o esperado a partir da
definição do objeto (a união dos discos ao longo da polilinha; o blend canônico; a
transferência de cor de referência), rode-o VERMELHO contra o código atual, e só
então implemente até ficar verde.

Corolário do [[feedback_no_industrial_claims_without_verification]] e da regra-mãe
da DIRETIVA (*verde-de-compilação vale zero*): **verde-de-oráculo-espelho também
vale zero.** Ver [[project_flip_stroke_analytic_coverage_gp]] e
[[feedback_harness_reproduces_mechanism_not_context]].
