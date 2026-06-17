═══════════════════════════════════════════════════════════════════
HANDOFF → Implementador Painter · AVALIAÇÃO DE ESTADO + próximo sprint priorizado
Autor: Coordenador (jornada 2026-06-05) · base: `avaliacao_e_melhorias.md` + assessment de código
═══════════════════════════════════════════════════════════════════

## §0 — TL;DR (leia isto + §3, o resto é referência)
1. **O Painter está SÓLIDO até W3 + W4 parcial.** W0/W1/W2/W3 fechados e auditados. W4
   (adjustment layers) é **~11/24 kinds reais** + UI dos demais. **As 5 "inovações
   extraordinárias" da avaliação são 0–20% (só type-layer/contrato).**
2. **DESBLOQUEIO NOVO (esta jornada):** a infra GPU multi-pass espacial **landou** em
   `ph2d-render` (Gaussian/Sharpen/Motion/ChromaticAberration provados em Metal). Os 4 kinds
   espaciais que estavam "stub aguardando Coord" **agora têm o mecanismo GPU pronto** — falta
   só TUA fiação (§3 P0).
3. **Teu próximo sprint (§3):** P0 = fechar a malha espacial (wire + refs CPU) + Noise/Halftone
   per-pixel. P1 = resto do W4 + dívida LOC. P2 = verificar o commit-path do W2 (risco de strokes
   não persistirem). Depois W5 = **primeiro portão de inovação** (Mixbox + grão procedural — types
   já existem).

## §1 — ONDE ESTAMOS (mapa de waves, evidência no assessment)
| Wave | Estado | Nota |
|---|---|---|
| W0 ADRs | ✅ FECHADO | 11 ADRs Accepted (0043..0053) |
| W1 brush core + StrokeHistory | ✅ FECHADO | smoke Day-7 OK; `StrokeRecord` schema completo (Q16.16, pressão, tilt, brush_ref, seed) |
| W2 sidebar+undo+Classic color | ✅ FECHADO | ⚠️ verificar follow-ups (commit-path, §3 P2) |
| W3 layers+22 blend+mask+clip+grupo | ✅ FECHADO | compositor GPU em `ph2d-render`, 22 modos, dirty-rect |
| **W4 adjustment layers (24 kinds)** | **🟡 PARCIAL** | ~15/24 CPU real, 9/24 com `gpu_code`; **6 espaciais eram stub → agora GPU-prontos**; falta UI bespoke de vários |
| W5 Brush Studio + Mixbox + grão procedural | 🟡 só types | `PigmentMode`/`ProceduralGrain` enums existem; **zero compute** |
| W6 brush library 12 + .brushset | 🔴 4 builtin só | atlas deferido |
| W7 color panel 5 modos + ColorDrop + eyedropper | 🟡 só Classic (W2) | 4/5 modos + Reference Companion não-iniciados |
| W8 selection/transform/liquify/clone | 🔴 type-layer | — |
| W9 guides/symmetry/quickshape | 🔴 | — |
| W10 gestures/quickmenu/atalhos/Pencil | 🟡 2/3-finger undo só | resto não-iniciado |
| W11 animation assist | 🔴 | — |
| W12 Reproject 1080p→4K + replay det | 🟡 contrato+structs | **replay compute = stub** (é a Inovação 1) |
| W13 MCP stroke engine | 🟡 contrato | tool `painter_paint_strokes` não-iniciada |
| W14 Stroke Inspector retroativo | 🔴 | depende de W12 |
| W15 fluid brushes | 🔴 crate vazia | `ph2d-fluids` "empty pending" |
| W16 PSD export/import | 🔴 | mapping documentado, I/O stub |
| W17 polish v1.0 | 🔴 | — |

## §2 — AS 5 INOVAÇÕES DA AVALIAÇÃO (a tese: "não só clonar o Procreate")
| Inovação | Status | Onde mora |
|---|---|---|
| 1. **Vetor Oculto** (resolução-independente / replay) | 🟡 **80% infra / 0% compute** | `StrokeRecord`+`StrokeHistory` gravam tudo (det Q16.16+seed); **Reproject re-render = stub** (W12) |
| 2. **Mixbox/Kubelka-Munk** (pigmento subtrativo) | 🔴 **0%** | `mixbox.rs` = lerp linear; sem solver KM, sem LUT, sem shader (W5) |
| 3. **Grão procedural** (Simplex/Gabor) | 🟡 **20% types / 0% compute** | enum 4-variants pronto; **sem WGSL**; atlas bitmap 64MB ainda ativo (W5) |
| 4. **Fluidos** (shallow-water + giroscópio) | 🔴 **0%** | `ph2d-fluids` vazia (W15) |
| 5. **MCP stroke agent** (LLM pinta strokes reais) | 🔴 **5% contrato** | ADR-0047 só; tool não-iniciada (W13) |

**Leitura honesta:** temos um "excelente Painter multiplataforma sabor-Procreate" arquiteturalmente
impecável até W3/W4. **Nenhuma das 5 inovações que tirariam ele do clone está funcional** — todas
são scaffolds de tipo/contrato. O caminho mais curto pra primeira inovação VIVA é **W5
(Mixbox + grão procedural)**, cujos types já estão definidos. (Decisão de priorizar inovação vs
fechar features clássicas é do Enio — eu recomendo em §4.)

## §3 — TEU PRÓXIMO SPRINT (priorizado, escopo da tua posse)
Posse tua: `ph2d-tool-painter`, `ph2d-painter-brush`, `ph2d-panel-painter-layers`, dispatch do
curve-editor em `editor-core`. **`ph2d-render` é Coord (eu) — NÃO toca.**

### 🔴 P0 — Fechar a malha espacial do W4 (mecanismo GPU já existe, é só ligar)
A infra está em `ph2d-render` (commits `ee1028a`/`49c475d`/`f1a3621`/`97d8086`, 4 gates Metal verdes).
Briefing completo: [`HANDOFF_painter_w4_spatial_gaussian_impl.md`](HANDOFF_painter_w4_spatial_gaussian_impl.md). Resumo do que é teu:
1. **Wire do flatten (`ph2d-tool-painter`):** pros kinds espaciais (`gpu_code()` retorna `None`),
   emitir `LayerOp::SpatialAdjustment{ kernel, params, blend_mode, opacity }` em vez de cair no
   CPU-path. Mapa: `GaussianBlur→SPATIAL_GAUSSIAN[radius]`, `Sharpen→SPATIAL_SHARPEN[amount,radius]`,
   `MotionBlur→SPATIAL_MOTION[distance,angle_rad]`, `ChromaticAberration→SPATIAL_CHROMA[r,g,b,falloff]`
   (codes re-exportados de `ph2d_render`).
2. **Refs CPU canônicas (`ph2d-painter-brush::adjustments/compute.rs`):** `apply_gaussian`,
   `apply_sharpen`, `apply_motion_blur`, `apply_chromatic_aberration` (a math σ↔radius / unsharp /
   box / falloff é tua). Hoje eu uso pesos PROVISIONAIS; me entrega os teus que eu reconcilio no
   `gaussian_weights`/`motion_weights` e re-rodo a paridade. (Os 4 já passam ±4B com placeholder;
   só trocar a curva.)
3. **Smoke do Enio:** aplicar GaussianBlur numa layer → ver borrar live no slider-drag (<ms, GPU).

### 🔴 P0 — Noise + Halftone (per-pixel, NÃO espera infra)
São per-pixel (hash/screen-function na coord) → vão no switch escalar via `gpu_code()`/`gpu_params()`
igual Vibrance, + a ref CPU em `compute.rs`. **Independentes da minha infra.** Fecha os 2 últimos
não-espaciais do W4.

### 🟡 P1 — Resto do W4 (kinds com UI bespoke ainda sem compute / parcial)
Por evidência do assessment, faltam CPU real ou wiring de: **ColorLookupLut** (.cube parsing),
**ShadowsHighlights** (8-param tonal — combine espacial; me entrega a ref CPU que eu faço a infra),
**Bloom** (bright-pass+mip — **precisa de infra pyramid extra minha**; coordena). Os bespoke-UI
(Curves 2D / GradientMap / SelectiveColor / Levels / ChannelMixer / B&W / ColorBalance) já têm CPU —
confirma que a UI está wirada end-to-end (paridade slider→compute→recompose).

### 🟡 P1 — Dívida LOC/tokens (documentada, não-bloqueante mas vence o gate no ship)
- Split `paint_adjust.rs` (829 LOC, OVERAGE_OK) + `event.rs::apply_event_impl` (299) em sibling files.
- Tokenizar os `1.5px` ring-outlines (hoje `// LITERAL-PX-OK` temporário em `paint_adjust.rs`).

### 🟠 P2 — VERIFICAR o commit-path do W2 (risco real)
Handoff antigo lista `R3-LE-4 — commit path unwired (Apply precisa wiring pra salvar strokes)` +
`R3-LF-3 failed-Apply destrói canvas` + `R3-LF-4 cancel-via-tool-switch dropa strokes silencioso`.
**Confirma se a pintura realmente persiste** (pinta → troca tool → reabre doc → strokes lá?). Se
não, é P0 disfarçado de P2 (uma feature "fechada" que não salva é pior que uma não-feita).

## §4 — FORWARD: W5 é o primeiro PORTÃO DE INOVAÇÃO (recomendação ao Enio, não-ordem)
Fechado o W4, o caminho mais curto pra primeira inovação viva (a tese da avaliação) é **W5**:
- **Mixbox (Inovação 2):** `PigmentMode` já existe; falta o solver Kubelka-Munk (paper
  Sochorová+Jamriška 2021 + LUT 3D) + wiring no `stamp.wgsl`. Smoke matador: azul+amarelo→verde
  vibrante (vs cinza do Procreate).
- **Grão procedural (Inovação 3):** enum pronto; falta WGSL (Simplex/Gabor) no `stamp.wgsl` →
  mata o atlas de 64MB + tiling-zero + zoom infinito.
- **Brush Studio (shell W5):** painel que edita os params live.
Mixbox+grão são **compute em `stamp.wgsl`** (tua pasta `ph2d-painter-brush` + shader) — fan-out
limpo, pouca dependência minha. Reproject (Inovação 1) é W12 e mais pesado. Fluidos (4) e MCP (5)
são waves distantes. **Sugiro: fecha W4 → ataca W5 Mixbox primeiro** (maior "uau" por esforço).

## §5 — POSSE / GIT (disciplina multi-agente)
- **Tua posse:** `ph2d-tool-painter`, `ph2d-painter-brush` (contrato `AdjustmentKind` CONGELADO ≤32 —
  NÃO adiciona variante; os 24 já existem), `ph2d-panel-painter-layers`, curve dispatch em `editor-core`.
- **Coord (eu):** `ph2d-render` (compositor + infra espacial), foundational, ship.
- **Vector impl ATIVO em W6** (`ph2d-node-vector-fill` / `ph2d-vector-*`) — paralelo, sem overlap.
- Commit scoped: `git add -- <teus paths>` · `git commit --no-verify -m "msg" -- <paths>` ·
  `git status` antes · sem push (Coord shipa 1×/jornada). RAM ≤3 cargos.
═══════════════════════════════════════════════════════════════════
