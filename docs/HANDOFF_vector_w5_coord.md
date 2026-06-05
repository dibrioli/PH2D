═══════════════════════════════════════════════════════════════════
HANDOFF → COORDENADOR · Vector W5 — pressão→variable-width INTEGRADO (A+B)
Autor: Implementador Vector (sessão W5) · 2026-06-04 · baseline HEAD local
Commit local: **`19cd7e4`** (não pushado). 2 arquivos, só meus paths.
═══════════════════════════════════════════════════════════════════

## §1 — ENTREGUE (consome teus hooks foundational; ZERO edit em ph2d-vector/-doc)

**A. Live preview (`render_loop/vector_pencil_bridge.rs`):** o overlay in-progress agora
expande a polyline de samples em **banda variable-width** via `draw_variable_width_stroke`,
`widths = overlay_base × StrokeSample.pressure`. Substituiu o `scene.stroke` constante.
Device sem pressão (1.0) → largura constante = look do W2 (graceful).

**B. Commit persistido (`ph2d-tool-vector-pencil`):** o `decimate()` agora carrega o
`StrokeSample` inteiro (pressão sobrevive aos knots). No commit, traço com variação real de
pressão recebe **1 `StrokeStyle` por SEGMENTO** com `WidthProfile { start, end, bulge:0 }`
interpolando as pressões dos dois knots — o `draw_vector_network` expande `width_profile`
automático (sem geometria baked). Pressão constante (mouse/trackpad) = 1 style compartilhado
(sem bloat, idêntico ao W2). Styles num **StyleTable fresco por-asset** (não acumula).

**Gates:** 25 testes do pencil verdes (2 novos: constant-pressure shared-style +
pressure-ramp per-segment-profiles); **shell `ph2d-host-desktop` compila limpo**; clippy
limpo; rustfmt pinned 1.95. Sem mudança de Cargo.toml/lock (deps já existiam).

## §2 — SMOKE pra fechar a lente visual do T5.3 (teu)

Desenha com o Pencil variando pressão (Apple Pencil / Wacom / trackpad force):
- **Durante o drag:** o overlay azul afina/engrossa com a pressão (live).
- **Solta:** o traço persiste variable-width (cada segmento com `WidthProfile`).
- **Mouse/trackpad sem força:** largura constante (regressão-safe = W2).

Me reporta o visual ou fecha o T5.3 — o data-path está completo do input ao render.

## §3 — Piece C (§2.C do teu handoff) — OPCIONAL, NÃO feito

Riqueza extra do nó `vector.width-profile` (eixo `bulge`/contrast, ou emitir `WidthProfile` no
StrokeStyle em vez de banda baked). **Ortogonal ao goal do W5** (pressão→largura já completo via
A+B). Deixei como follow-up pra não inchar um nó já fechado — o tipo `WidthProfile{start,end,bulge}`
já existe, então é drop-in quando quiser (adicionar param `bulge` + somar `bulge·4t(1−t)` no
`half(t)` do `emit_band`). Me avisa se quer que eu faça.

## §4 — Anti-colisão
Outro implementador (Painter) com arquivos STAGED no índice compartilhado — meu commit foi
**scoped** (`git commit -- <meus 2 paths>`), nada alheio tocado.
═══════════════════════════════════════════════════════════════════
