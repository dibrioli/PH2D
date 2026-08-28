---
name: feedback-a-probe-in-the-failure-branch-cannot-see-the-other-sides-successes
description: Uma sonda colocada no ramo em que A falhou nao pode ver os sucessos de A — e le'-se como "A nunca acerta".
metadata:
  type: feedback
---

Ao instrumentar *«qual das alternativas acertaria»*, repare **onde** a sonda vive. Se ela
está no ramo a que só se chega **depois de A falhar**, a coluna de A vem `0` por
construção — e lê-se como *«A nunca acerta»*, que é o contrário do que os dados dizem.

**Why:** medido no `ph2d-quadextract` (2026-08-27). A sonda das convenções de direcção
corria no `None` do `by_key.get(...)` de A, e imprimiu `d2 = 7/3/2` contra
`opposite(d2) = 0/0/0`. Conclusão aparente: trocar por `d2`. ⛔ Mas A fazia `4` resgates na
`eared` e `2` na `hooked` **antes** daquele ramo, e trocar **perdia-os** (`eared` de `4`
para `2`, `χ` e bordo a piorar). O que salvou a leitura foi outra coluna da mesma sonda —
`ambíguas = 0` —, que disse que as duas **nunca** colidem e que a resposta era a **união**,
não a escolha.

**How to apply:** ao comparar alternativas, meça-as **fora** do ramo de fracasso de
qualquer uma delas — ou registe, ao lado, quantas vezes cada uma já tinha acertado antes de
a sonda correr. E pergunte sempre se as alternativas são **exclusivas**: se não forem, a
resposta pode não ser escolher.
Parente de [[feedback-a-bucket-nobody-fills-reads-as-perfect]] e
[[feedback-a-ruler-placed-after-the-tidying-step-measures-the-tidying]].
