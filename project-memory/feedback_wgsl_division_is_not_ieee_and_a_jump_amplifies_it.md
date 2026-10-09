---
name: feedback_wgsl_division_is_not_ieee_and_a_jump_amplifies_it
description: A divisão f32 do WGSL tem 2,5 ULP (1599/1599 = 0,99999994 na RTX); numa função com SALTO um ULP vira um degrau inteiro — dividir por `pe_div` (Markstein/fma) para o dispositivo dar o quociente da CPU
metadata:
  type: feedback
---

**Medido no ciclo 17 (doc 126 §5.1, 2026-10-09):** o kernel do `motion.stagger` calculava
`raw = f32(i) / last` e, para a última linha, a placa (RTX 5060 Ti) dava `0,99999994` onde a CPU
dá `1`. Nas 8 curvas do ciclo 2 (contínuas) isso era invisível; no `Expo` de Penner (`0` em `t = 0`,
`2⁻¹⁰` logo depois) e no `Steps` o ULP virou `0,3 %` do curso (`cpu 7,045 × gpu 7,042`, falhou o
gate de paridade de `2e-3`).

**Why:** a especificação WGSL dá `2,5 ULP` à divisão `f32` (e ao `sqrt`); o `f32::div` da CPU é
IEEE correctamente arredondado. Uma lei com SALTO (degrau, `floor`, ramo `t <= 0`, comparação) não
amortece a diferença: amplifica-a ao tamanho do salto.

**How to apply:** em código WGSL que tem de dar a resposta da CPU e alimenta uma função
descontínua, divida por `pe_div` da `ph2d-ease` (`y0 = 1/b; y = fma(fma(-b,y0,1),y0,y0);
q = a*y; return fma(fma(-q,b,a),y,q);`) — fechou a paridade em `9,5e-7`. Os coeficientes
transcendentais vão por BITS (`bitcast<f32>(0x…u)` / `f32::from_bits`), nunca por decimal: o WGSL
lê o literal como abstract-float `f64` e arredonda de novo. Relacionado:
[[feedback_a_tie_broken_by_rounding_is_a_quadrant_artefact_not_a_law]].
