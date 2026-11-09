---
name: feedback_a_field_census_category_decides_what_the_macro_records
description: Classificar um campo do BrushSpec em CAMPOS_FORA muda o que a MACRO grava — `dente` como DA_OBRA derrubou 188 testes de macro (09/11)
metadata:
  type: feedback
---

Pôr um campo no censo `CAMPOS_FORA` não é arrumação: a **categoria** decide o que `campos_da_macro()` leva. `dente` (papel da tela) foi posto como `DA_OBRA`; a macro passou a gravá-lo sem conversão e `json/meio.rs` caiu no `unreachable` — **188 testes de macro vermelhos** na suíte #4 da rodada de 09/11, todos de uma causa.

**Why:** o `dente` é enchido em cada dab pelo papel da tela (`Dente::do_papel`), logo é `POR_DAB`; `DA_OBRA` diz «viaja com o documento» e a macro obedece.

**How to apply:** ao classificar um campo novo no censo, pergunte «a macro grava isto?» e corra um teste de macro por nome depois. Os testes do grupo que fez a cura não pegam; só a suíte inteira pegou.
