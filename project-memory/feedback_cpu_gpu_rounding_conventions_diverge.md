---
name: feedback-cpu-gpu-rounding-conventions-diverge
description: Rust f32::round é half-AWAY; WGSL round() é half-EVEN — um enum-por-param roteado por round escolhe RAMOS diferentes na CPU e no kernel
metadata:
  type: feedback
---

Portando o `motion.oscillator` pra WGSL (GPU/M5 Fase 1, `line/gpu-nodes`): o CPU faz
`ctx.param("wave").round() as i32` e o kernel fazia `i32(round(params.wave))`. Parece o mesmo
código — **não é**: `f32::round` do Rust arredonda half-AWAY-from-zero (2.5→3), o `round()`
do WGSL arredonda half-to-EVEN (2.5→2). Num param que é ENUM (channel/wave/mode), meio-ponto
não é erro de ε — é **outro braço do switch**: outra waveform, outro canal, outro comportamento.

**Why:** ε-tolerância cobre aritmética; não cobre CONTROLE. Qualquer conversão float→decisão
(round, floor de negativo, cast saturante, comparação com NaN) tem convenção própria em cada
linguagem, e o meio-caminho é exatamente onde elas divergem — o pior lugar, porque o teste com
valores inteiros "normais" fica verde ([[feedback_gate_the_edges_of_the_domain]]).

**How to apply:** ao portar CPU→GPU (ou qualquer par de linguagens), todo sítio que converte
param em decisão ganha auditoria explícita: ou porta a convenção (half-away em WGSL =
`select(ceil(x - 0.5), floor(x + 0.5), x >= 0.0)`), ou reformula a decisão pra região onde as
duas concordam (comparação `< 0.5` num domínio que o `applicable`/validador já restringiu).
Nunca assuma que `round` é `round`. Mesma família: [[feedback_same_math_different_bookkeeping_diverges]].
