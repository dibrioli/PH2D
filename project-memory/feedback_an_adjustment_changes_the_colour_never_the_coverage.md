---
name: feedback-an-adjustment-changes-the-colour-never-the-coverage
description: "Misturar a cor ajustada de volta por `over` com o alfa da base aplica só PARTE de um ajuste a 100 % num píxel translúcido — 79 degraus do GIMP, invisível em toda base opaca"
metadata:
  type: feedback
---

O braço da camada de ajuste do Painter (CPU `compose.rs` e placa `apply_adjustment_op`) devolvia a cor
ajustada como uma camada `Normal` com o MESMO alfa da base: `over` de `a` sobre `a` dá
`(cs·a + cb·a·(1−a)) / (a + a(1−a))` — com `a = 0,5`, ⅔ do ajuste. Um Invert a 100 % sobre a borda
macia de um traço saía mais fraco na borda. Todo gate de paridade usava base OPACA (onde as duas leis
coincidem); só o oráculo do GIMP sobre a grelha com alfa (P3 do ADR-0177) o viu. O combine do S/H na
placa já fazia o certo no `Normal` — a inconsistência entre dois braços era o cheiro.

**Why:** uma camada de ajuste muda a COR que o píxel tem, nunca a cobertura (Photoshop, GIMP): o modo
mistura as duas cores como opacas e o alfa da base fica.

**How to apply:** qualquer «efeito sobre o que está por baixo» (ajuste, filtro, máscara de cor) —
mede-se sobre uma base TRANSLÚCIDA além da opaca; se o alfa da base entra na mistura da cor, é este
defeito. Gates: `um_ajuste_de_ecra_e_o_do_gimp_perceptual`,
`gpu_um_ajuste_cheio_sobre_um_pixel_translucido_aplica_se_inteiro`. Ver [[reference_topic_code_pattern_gotchas]].
