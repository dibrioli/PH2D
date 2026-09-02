---
name: a_clamp_before_a_range_test_deletes_the_test
description: Um `clamp` aplicado a um valor ANTES de o testar contra o intervalo apaga o teste — e o efeito visível é uma peça a manchar a coluna inteira
metadata:
  type: feedback
---

Medido 2026-09-01 (`asset_card_portrait::blit`, report do Enio com foto — *«ainda não representa
fielmente o objeto»*). O `v` do sampler era recortado a `[0, 1]` e só DEPOIS testado com
`(0.0..1.0).contains(&(1.0 - v))`: depois do `clamp` o valor está sempre no intervalo, o teste
nunca recusa, e cada peça era desenhada em toda a altura do retrato dentro das suas colunas.

**Why:** o `clamp` foi posto para o `sx`/`sy` do sampler não sair da textura — a intenção certa,
no sítio errado. O teste de «está dentro do quad?» tem de correr sobre o valor CRU; o
recorte, se for preciso, vem depois, na leitura do texel.

**How to apply:** ao ver `let x = (…).clamp(a, b); if !(a..b).contains(&x)`, o `if` é morto.
E o oráculo que apanha isto não é «o pixel certo tem a cor certa» — é **«o pixel FORA da peça
está vazio»**: oito gates de presença deixaram a mancha passar, e o gate que a apanhou pergunta
pelos dois lados. Ver [[a_presence_only_oracle_is_blind_to_the_smear]] e
[[reproduce_with_the_real_constructors_and_look_at_the_image]].
