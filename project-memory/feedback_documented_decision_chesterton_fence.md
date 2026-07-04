---
name: feedback-documented-decision-chesterton-fence
description: "Não 'corrija' código com base em primeiros-princípios quando há comentário dizendo 'intentionally NOT X' + razão — é cerca de Chesterton; verifique a história antes"
metadata: 
  node_type: memory
  type: feedback
  originSessionId: 08f6a613-4a63-4a4e-8305-1b658212543e
---

Quando um comentário no código diz **"intentionally NOT used here"** / "deliberately X, NOT Y" + uma razão, isso é uma **cerca de Chesterton documentada**: o autor original JÁ considerou a tua "correção" e a rejeitou por um motivo que tu pode não enxergar de imediato. NÃO sobrescreva com base em raciocínio de primeiros-princípios isolado.

**Why:** em 2026-06-01 eu troquei `premultiply_rgba8` (byte-space) → `premultiply_rgba8_in_linear` no Painter/BgRemoval achando que byte-space era "matematicamente errado" (halo nas bordas translúcidas). **Reintroduzi um bug que o Enio já tinha corrigido há tempos** (`008b5bf`, revertido em `3870733`). A verdade documentada que ignorei: o halo era artefato do **path do Vello** (`Rgba8Unorm` raw-byte); foi curado **movendo o preview pro path do sprite shader** (`Rgba8UnormSrgb` + premul blend), onde byte-space é correto e bate byte-a-byte com o Apply. O `premultiply_rgba8_in_linear` é helper vestigial do "Fix C" (Vello), sem caller de produção — e deve continuar assim. O comentário que sobrescrevi dizia literalmente "the gamma-correct variant is intentionally NOT used here".

**How to apply:** antes de "corrigir" código que parece errado, especialmente cor/gamma/blend: (1) se há comentário explicando por que NÃO se faz o óbvio, trate como decisão ratificada — leia a história (git log/blame, ADRs, o path completo do pipeline) ANTES de mexer; (2) meu cálculo de primeiros-princípios pode estar certo em abstrato mas errado pro path real (ex.: o halo dependia de Vello vs sprite-shader, não da matemática do premul isolada); (3) "padrão-ouro" inclui NÃO regredir trabalho já validado — o Enio sabe a história do produto, eu não; (4) se realmente acho que uma decisão documentada está errada, pergunto/confirmo a razão original em vez de sobrescrever. Relacionado: [[feedback_convention_vs_inertia]] (o inverso — nem toda convenção tem gate; mas comentário com RAZÃO explícita ≠ inércia), [[feedback_perfection_no_deferrals]].
