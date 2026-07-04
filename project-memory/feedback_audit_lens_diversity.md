---
name: feedback-audit-lens-diversity
description: Auditorias adversariais exigem rotacionar LENTES em rounds sucessivos (não apenas múltiplos rounds da mesma lente); cada nova lente pega Crítico/Alto que as anteriores não pegaram
metadata: 
  node_type: memory
  type: feedback
  originSessionId: 99698f22-e200-4483-8825-0e4a8b03c18e
---

Regra: rotacionar lentes adversariais em rounds sucessivos. Cada nova
lente pega Crítico/Alto que lentes anteriores não pegaram. **Padrão-ouro
EXIGE multi-lens, não multi-rounds-da-mesma-lente.**

**Why:** Round 3 da sessão T1.4 (lente spec/HR/det) pegou Crítico C2
(premul invariant em blending modes — `uniform_blending` /
`intense_blending` misturando dst_premul com src_unmul num `mix()`) que
rounds 1+2 (lentes WGSL/ABI + Rust idiomatic) NÃO pegaram. Esse bug
teria explodido em T1.5 ping-pong A↔B porque dst real volta
premultiplicado, ao contrário do day-5 test que mascarou com dst=0.
Após dois "padrão-ouro round N" outro Crítico ainda surgiu apenas
mudando a lente.

**How to apply:**

1. ≥2 lentes paralelas por round em qualquer task substancial.
2. Em rounds sucessivos, rotacionar **pelo menos 1** lente — não repetir
   o mesmo combo de lentes do round anterior.
3. Lentes canônicas (rodízio recomendado):
   - (a) corretude WGSL / ABI / alinhamento / size_of/align_of
   - (b) Rust idiomatic + HR-3 zero-alloc hot-path + API ergonomics
   - (c) cross-platform GPU + wgpu features + driver-specific quirks
   - (d) spec compliance + HR-1..18 + determinismo bit-identical CPU↔GPU
   - (e) regressões pós-remediação (re-verifica findings fechados)
   - (f) test coverage real vs verbal claim (gates executáveis vs comentários)
4. Round N+1 sempre legítimo enquanto nova lente continuar pegando ≥1
   Alto/Crítico — não baixar barra antes de ≥1 round com lente nova e
   zero findings desse nível.
5. Gates executáveis > verbal claims. Se a sessão afirma "bit-identical"
   ou "size invariant" ou "premul preserved", escreva o teste que falha
   se a propriedade quebrar. Não comente — codifique.

Relacionado: [[feedback-perfection-no-deferrals]],
[[project-painter-t14-complete-2026-05-26]].
