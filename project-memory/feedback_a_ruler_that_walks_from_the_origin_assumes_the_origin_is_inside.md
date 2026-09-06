---
name: feedback_a_ruler_that_walks_from_the_origin_assumes_the_origin_is_inside
description: Uma bissecção que usa `lo = 0` como extremo INTERIOR só funciona enquanto todas as peças contêm a origem — a primeira de miolo vazio recebe uma acusação inventada.
metadata:
  type: feedback
---

Uma régua que bissecta a superfície **a partir da origem** (`lo = 0` como extremo «dentro»,
`hi = raio` como «fora») está a assumir, sem o dizer, que **a origem está DENTRO da peça**. Isso é
verdade para toda a população que existia quando ela foi escrita, e deixa de o ser na primeira peça
diferente.

**Why:** medido em 2026-09-05 no catálogo de formas 3D (`the_bounding_half_extents_contain_the_piece`,
28 primitivas). A **seta dobrada** é a primeira cujo miolo é **vazio** — o canto de dentro do «L» não
tem matéria. Ali a bissecção não tem invariante nenhuma: ela convergia para uma troca de sinal
qualquer e **acusou peça a `0,3459`** num eixo onde uma varredura densa mede `0,3386`. A cura pareceu
uma correcção de produto e não era.

**How to apply:** amostre a semi-recta **de fora para dentro** e guarde o **maior** `t` com matéria; o
`t` seguinte está fora por construção, e é entre esses dois que se bissecta. Sem pressuposto, uma
peça oca, um anel ou duas ilhas medem-se todos igual. ⚠️ E antes de acreditar numa acusação nova de
uma régua velha, pergunte *«que propriedade da população antiga ela usava sem declarar?»* — ver
[[feedback_a_gate_that_measures_the_rare_case_leaves_the_normal_one_without_a_ruler]].
