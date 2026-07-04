---
name: M5 perf headroom validated
description: M5 sprite renderer validado visualmente até 100k sprites @ 60Hz no Mac M-series — folga >32× sobre o gate (1k @ 60Hz < 3.5ms)
type: project
originSessionId: 3810fc76-ee39-499c-932e-822ab7813c1b
---
Em 2026-05-08 Enio rodou `cargo run --release -p ph2d-host-desktop`
com `SPRITE_COUNT = 100_000` no Mac M-series e o app manteve 60 FPS
sem queda. O gate oficial do M5 é "1000 sprites @ 60Hz, frame budget
< 3.5ms" — a validação real cobriu **100×** isso.

**Why:** Enio quis ver onde quebrava ("100000"). Não quebrou. Provou
que o pipeline (extract clear+spawn 100k entities/frame + instance
buffer dinâmico + single instanced draw) escala bem além do target.

**How to apply:** decisões de design pra M8+ podem assumir que CPU
extract + GPU instanced draw NÃO é o gargalo de cenas 2D típicas
(< 10k sprites). Otimizar prematuramente extract/render = trabalho
desperdiçado. Se aparecer queda de FPS antes de M10 (physics) ou
M11 (Vello vector), o suspeito **não** é o sprite path — investigar
o componente novo primeiro.

A branch que validou (m5/perf-tools com FPS counter + S-shadow toggle)
foi descartada após validação; pode ser reintroduzida como `ph2d-debug`
se precisar de instrumentos visuais futuros.
