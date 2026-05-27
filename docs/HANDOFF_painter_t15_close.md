# 🎨 HANDOFF — Painter PH2D · pós-T1.5 ship + smoke ✓ · 2026-05-26

**Para:** próxima LLM (agente novo) pegar T1.6 sobre Painter.

**Smoke Day-7 confirmado pelo Enio** com screenshot: traço opaque-orange
contínuo num sprite jester colorido. Stamps alpha-over acumulando,
bounds respeitados, sem rubber-band leak. T1.5 padrão-ouro
**ratificado**.

---

## 1. Estado da entrega (sessão T1.5)

**Commits locais (4, não-pushados):**
- `14e416f` — T1.5 ship (Day-7 marker) — 20 files, +2925/-143
- `9dbde5c` — T1.5 rounds 4+5 perf/API/ship-readiness — 8 files, +246/-60
- `d1b493c` — T1.5 round 6 spec compliance polish — 2 files, +59/-28
- `ad327ed` — docs handoff close — 1 file, +339

**Branch:** `main` local, **52 commits ahead** de `origin/main`. Push é
fim-de-jornada sob ordem do Enio (NÃO push autônomo).

**Verde:** 180 tests · clippy `-D warnings` · fmt · machete · workspace
check. ADR-0043+ADR-0044 caps intactos. HR-3/HR-5 gates ativos.

**Auditoria adversarial executada: 6 rounds × 12 lentes paralelas
rotacionadas → 75 findings (12C + 16H + 32M + 15L)** todos remediados
em código OU documentados como W2 follow-ups. Round 6 com lentes NOVAS
(M thread-safety + N spec compliance) primeira round com ZERO
Critical/High = padrão-ouro threshold per regra
[`feedback-audit-lens-diversity`](file:///Users/dibrioli/.claude/projects/-Volumes-MAC-EXTERNO-PROJETOS--PH2D-definitiva/memory/feedback_audit_lens_diversity.md).

| Round | Lentes | Findings (C/H/M/L) |
|---|---|---:|
| 1 | A spec+HR+det+regr-T1.4 / B shell+GPU+Rust+idiomatic | 4/6/12/4 |
| 2 | C test-coverage-vs-claim / D regressões-round-1 | 2/3/5/1 |
| 3 | E Day-7-validity / F cross-tool-lifecycle | 3/2/5/3 |
| 4 | G HR-3/perf / H API+safety+panic | 2/4/9/3 |
| 5 | I round-4-regr / J ship-readiness | 1/1/1/0 |
| 6 | M thread-safety / N spec+ADR+HR compliance | **0/0/0/4** |

---

## 2. PRÓXIMA TASK — T1.6 brush mature

**Decisão Enio (2026-05-26):** seguir o plano da Wave 1. Próxima task é
**T1.6 brush mature** — completa o brush expandindo de "round_hard
único axis-aligned" para "shape atlas + scatter + count + rotation +
flip + color dynamics stamp-level".

**Spec canônica:** [`docs/Painter_projeto/01_brush_engine.md`](Painter_projeto/01_brush_engine.md)
§1.3.4 (Shape params) + §1.3.8 (Color Dynamics) + §1.6 (Library — 12
built-ins canônicos).

**Plano de execução:** [`docs/Painter_projeto/15_plano_de_implementacao.md`](Painter_projeto/15_plano_de_implementacao.md)
§5 (Wave 1 - T1.6).

### 2.1 Sub-tasks T1.6 (ordem sugerida)

1. **Shape atlas binding (GPU + CPU paridade)** — substitui
   `round_hard_shape()` inline procedural do shader por
   `texture_2d_array<f32>` binding `@binding(4)`. Atlas builder em
   `crates/ph2d-painter-brush/src/library.rs` consolida built-ins
   procedurais como slots 0..N (round_hard slot 0 já existe). CPU
   equivalente em `cpu_render.rs` (paridade ULP-bounded preservada).
2. **Shape param fields no Brush** (já existem como sub-struct
   `ShapeParams`; só precisa ser usado pelo scheduler + shader):
   - `shape_scatter: f32` (0..360°) — rotação aleatória por stamp
   - `shape_count: u32` (1..16) + `shape_count_jitter: f32` — múltiplos
     stamps por pointer event espalhados conforme scatter
   - `shape_rotation_follow: bool` — rotation = stroke direction
   - `shape_randomized: bool` — rotação aleatória inicial por stroke
   - `shape_flip_x: bool` + `shape_flip_y: bool` — bits no `Stamp.flags`
     (`FLAG_SHAPE_FLIP_X = 1`, `FLAG_SHAPE_FLIP_Y = 2` já reservados!)
3. **StampScheduler upgrade** — emit múltiplos stamps por pointer event
   conforme `shape_count`; aplicar rotation per stamp:
   - rotation_follow=true → `stamp.rotation_rad = atan2(stroke_dir.y,
     stroke_dir.x)`
   - scatter > 0 → `+ det_random(stamp_index, 0xCD) * scatter_rad`
   - randomized=true (stroke-level) → fixed rotation no `begin_stroke`
4. **Shader stamp.wgsl upgrade** — `cs_stamp` aplica `rotation_rad`
   nos cálculos uv (rotação ao redor do center_offset):
   ```wgsl
   let cos_r = cos(stamp.rotation_rad);
   let sin_r = sin(stamp.rotation_rad);
   let local_x = (f32(pixel_local_x) - center_offset) * cos_r - (f32(pixel_local_y) - center_offset) * sin_r;
   // similar para local_y
   ```
   Plus uv flip se `flags & FLAG_SHAPE_FLIP_X`.
5. **Color Dynamics stamp-level jitter** (`ColorDynamicsParams.
   stamp_hue_jitter`, `stamp_saturation_jitter`, `stamp_lightness_
   jitter`): scheduler aplica jitter ao `stroke_color_oklab` per stamp
   via `det_random`. Stroke-level + pressure/tilt modulations adiados
   pra W14+.

### 2.2 Acceptance T1.6

- **Smoke visual:** Painter pill ativo → brush size 64 + spacing 0.3 +
  scatter 30° + count 3 → click+drag produz **cluster de stamps
  rotacionados** ao longo do path (não mais traço single-stamp).
- **`color_dynamics.stamp_hue_jitter = 0.5`** → cada stamp tem hue
  diferente (variação visível ao longo do stroke).
- **`shape_rotation_follow = true`** → stamps oblongos (oval roundness <
  1) alinham com direção do stroke automaticamente.
- **Arch-gates ainda verdes:** `ph2d-painter-contracts` **75 arch-gate
  tests** pós-R3 (era 74 + `det_painter_feature_is_declared` adicionado
  em audit T1.6 R3). Caps mantidos: PainterUiEdit≤24, Brush≤14
  top-level, ShapeParams≤20 fields, ColorDynamicsParams≤36, Stamp=96B
  align(16). **Behavioral tests separados:** `ph2d-painter-brush` lib
  shipa **150+ unit tests** (R3+R4+R5 additions incluem
  `oval_hard_*` × 5, `color_jitter_cross_channel_axis_independence`,
  `det_random_distribution_*` × 2, `multi_quadrant_chroma`, etc) +
  1 dhat integration test `painter_no_alloc_hot_path`.
  `ph2d-tool-painter` shipa 23 tests. **Total painter-suite: ~250
  tests verdes** (audit T1.6 G1-H3).
- **HR-5 determinismo (revisado audit T1.6 U-1):** o subsistema
  **integer** (PRNG mixer wyhash + Stamp ABI byte layout + index
  arithmetic) é bit-identical cross-OS. O subsistema **float** (trig
  `cos/sin/atan2`, `sqrt`, OKLab cubic) é **ULP-bounded ~1-4 ULP
  cross-backend, NOT bit-identical** — `cos/sin/sqrt` lower to
  backend intrinsics (Metal/Vulkan/D3D12) cuja precisão impl-defined
  per WGSL spec, e `f32::cos/sin` Rust libm divergem entre
  toolchain/targets. **HR-5 strict cross-OS** requer `--features
  det-painter` (declarado em `crates/ph2d-painter-brush/Cargo.toml`
  como audit T1.6 U-2; wiring progressive — currently no-op stub,
  ativará com T-numerical-parity W2+ + Mixbox + GPU bypass).
  Gates ativos: `shader_oklab_coefficients_bit_identical_with_rust`
  (15 OKLab literals pinned), `cpu_shader_textual_parity_all_six_modes`
  (rendering mode formulas pinned), `cpu_shader_shape_kernels_textual_parity`
  (4 shape kernels pinned), `color_jitter_cross_channel_axis_independence`
  (PRNG bit-equality within single target), `det_random_distribution_is_
  roughly_uniform` + `det_random_axis_tag_independence_bit_distinct`
  (mixer integer determinism), `det_painter_feature_is_declared` (feature
  flag declared).
- **HR-3 hot path:** scheduler.advance é provado **zero-alloc** via
  novo gate `painter_no_alloc_hot_path` (dhat-rs, audit T1.6 Z-1).
  Combined baseline (round_hard) + worst-case (square_hard +
  shape_count=16 + 4 jitters + scatter + rotation_follow + randomized
  + count_jitter) sob um único dhat::Profiler — 100 frames × 10
  advances cada, total_blocks delta = 0 hard-asserted. Pool 4096
  absorve multi-stamp; `pool_cap_respected_with_high_shape_count`
  prova cap + capacity-equality (Vec não realloca).

### 2.3 Critério Day-N (T1.6 close)

- Variedade visual visível **4 brushes built-in** (round_hard /
  round_soft / square_hard / **oval_hard** — `oval_hard` adicionado em
  audit T1.6 V-2 pra dar demo visual de `shape_rotation_follow`)
  com diferenças notáveis. Library expandida em `library.rs`.
- Color Dynamics stamp_hue_jitter funcional + **bit-identical
  within single Rust target** seed-determinístico (audit T1.6 U-1
  revisado — cross-OS bit-equality requer det-painter, ULP-bounded
  por default).
- Auditoria adversarial ≥2 rounds × ≥2 lentes paralelas (rotacionar de
  rounds 1-6 anteriores). Zero Crit/High aceito. **T1.6 entregou 6
  rounds × 13 lentes distintas** (R1: O atlas / P color / Q multi-stamp;
  R2: R regressions / S spec; R3: T edge cases / U cross-OS / W test
  quality / Z perf; R4: A1 R3-regression / V acceptance; R5: B1 R4-
  regression / C1 acceptance verification; R6: D1 prod safety / E1
  recovery integrity / G1 doc cross-ref) — exceeds spec.

### 2.5 CI close sequence (post-ship follow-ups, audit T1.6 G1-H2)

Closing T1.6 in CI required 8 follow-up commits beyond the audit
remediation commit `932316e` — surfacing alheia work-in-progress
(bgremoval Add-Area feature) that was already dirty in the working
tree when my clippy/fmt sweeps ran:

| Commit | Rationale |
|--------|-----------|
| `4f8d266` | Cargo.lock dhat entry + fmt `color_eq_bridge.rs` |
| `7dd462c` | `cargo fmt --all` sweep — 12 alheia files needing fmt |
| `f6d8d0a` | Workspace clippy sweep (premul/silhouette/etc); collateral capture of bgremoval tool.rs (~506 LOC alheia WIP) |
| `fe8544b` | Recovery: Add-Area symbols (panel-bgremoval ids/event/paint_sections/populate + tool params + protect_brush + editor-core ids) |
| `4cb519d` | Recovery: bgremoval `run_pipeline` 7-arg signature (algorithm/mod.rs + compose.rs) |
| `c8a4ebb` | Recovery: Add-Area FINAL (render renderer.rs `replace_individual_pixels` + 5 shell files) |
| `c411c5f` | fmt color_equalization_bridge.rs (1 residual line) |
| `08df125` | typos.toml whitelist (35 PT-BR tokens) + 3 EN typos fix |

**Implication for alheia bgremoval agent:** `git pull --rebase` on
return. Their Add-Area feature is 100% in origin/main but their LOCAL
WIP redesign (binary-mask + un-protect pass in
`prepare_combined_protect_canvas` — Enio 2026-05-27 follow-up to
"área não é sujeita a ajustes finais") was NOT captured per audit
T1.6 E1-5. Manual reconciliation needed.

CI verde (final): https://github.com/dibrioli/PH2D/actions/runs/26497569381
— all jobs success (lint + MSRV + workspace tests × ubuntu/macos/
windows + C9 replay hash × 3 OS + C9 ECS comparison).

### 2.4 Esforço estimado

**3-5 dias** sequencial single-agent. Sub-tasks 1-2 e 5 podem ir
paralelas (atlas binding + color dynamics em scheduler) se quiser
fan-out, mas a integração sub-tasks 3+4 (scheduler + shader) é
coupled — manter sequencial nessa parte.

---

## 3. Estado git + working tree

```
HEAD: ad327ed
Branch: main (52 commits ahead de origin/main)
```

**Working tree alheio (NÃO TOCAR — outras sessões em vôo):**
- `docs/Painter_projeto/*.md` modified — outras docs sessions
- `docs/SESSION_ACTIVE.md` modified
- `docs/UI_Fonts/` untracked — agente tema
- `Cargo.lock` modified — múltiplos
- `shells/desktop/src/render_loop/color_equalization_bridge.rs`,
  `bgremoval_preview.rs` — fmt drift ou edits de outros agentes
- `crates/ph2d-imageio-gif/` untracked — agente imageio
- `docs/Painter_projeto/14_inovacoes_extraordinarias.md` + outros
  untracked
- `test_strip` untracked

Outras sessões committaram em paralelo durante a sessão T1.5:
- `9127011` agente imageio (W1.T6 + auditoria 5-lente Onda 1)
- `4eabab4`, `4084ee4`, `b3c15fa` agente bgremoval (overlay halos +
  rotation/scale tracking)

---

## 4. Aprendizados desta sessão (LEIA antes de codar)

### 🔥 Padrão-ouro = rotação de lentes adversariais até zero Crit/High

6 rounds × 12 lentes paralelas rotacionadas. Cada nova lente encontrou
Crit/High que as anteriores não viram. Padrão-ouro NÃO é "audit
múltiplos rounds da mesma lente" — é "rotacionar lentes até uma round
com ZERO Crit/High". Memory
[`feedback-audit-lens-diversity`](file:///Users/dibrioli/.claude/projects/-Volumes-MAC-EXTERNO-PROJETOS--PH2D-definitiva/memory/feedback_audit_lens_diversity.md)
codifica.

**Para T1.6:** lance ≥2 lentes paralelas por round, rotacione de A-N
já usadas. Sugestões pra T1.6: O (atlas binding correctness) + P (color
dynamics determinismo) + Q (multi-stamp HR-3 budget em
`shape_count=16`).

### 🔬 Arc-based hot-path otimização é sutil

R4-LG-1 (Arc canvas) precisou R5-LI-C fix em `run_full` (`mem::replace
+ unwrap_or_clone` em vez de `Arc::unwrap_or_clone(Arc::clone(...))`).
**Lição:** sempre considere refcount no momento da unwrap; use
`mem::replace` quando quer ownership transfer.

### 🎨 Default semantics — 3 sites devem alinhar

R4-LH-1 pegou `RenderingMode::default` desviou de `Stamp::zeroed()
::rendering_mode → from_u32(0)` e `RenderingParams::default`. Fix
revert pra LightGlaze. **Lição:** ao alterar `#[default]`, verifique
Default derivações + ABI byte-zero decode + per-tipo defaults explicitos.

### 📐 11 anti-padrões catalogados (todos com gates executáveis)

C2 premul invariant blending, D-3.H7 oklab bit-identical, R1 A-M7 +
R6-LN-2 textual_parity_all_six_modes, R3-LE-1 stroke gap smear (break_
segment), R3-LE-2 rubber-band leak under Painter, R3-LE-3 invisible
default color (opaque orange), R3-LF-2 silent stroke loss em selection
drift, R4-LG-1 Arc canvas, R4-LH-1 3-sites defaults triangulation,
R4-LH-3 set_source assert_eq release, R5-LI-C mem::replace antes de
unwrap_or_clone. Esses já estão no código + tests, próxima sessão NÃO
precisa re-descobrir.

### 🚧 W2 follow-ups documentados — débito real, não silent

8 follow-ups em `crates/ph2d-tool-painter/src/tool.rs` module header
(linhas 15-78 — **referência canônica de débito técnico T1.5**):
- R3-LE-4 commit path unwired (Apply button W2)
- R3-LE-5 / R4-LH-8 stale canvas após external mutation
- R3-LF-3 failed Apply destrói canvas
- R3-LF-4 cancel via tool-switch silently drops strokes
- R4-LG-2 PREMUL canvas storage (35% per-pixel speedup)
- R4-LG-3 per-pixel match dispatch hoist (const generic)
- R4-LG-6 CPU regime size cap (UI soft-cap 256)
- R6-LN-3 HR-18 LOC cap policy ambiguidade
- R6-LN-4 HR-15 hardcoded Toast strings (workspace-wide)

Esses NÃO bloqueiam T1.6. Atacar em sessão W2 dedicada.

---

## 5. Memória persistente a atualizar

Memória `project_painter_t15_complete_2026_05_26.md` (already exists).
**Update** ao começar a sessão T1.6 com:
- Smoke Enio confirmado (screenshot jester traço orange) → T1.5
  ratificado
- Próxima sessão = T1.6 brush mature
- Total handoff: 75 findings remediados / 6 rounds / 12 lentes

NÃO criar memória nova; update existing.

---

## 6. Tier-1 leitura obrigatória antes de codar T1.6

1. [`docs/HANDOFF_painter.md`](HANDOFF_painter.md) §0 (mandato) + §1
   (LOOP) — governança "padrão-ouro absoluto"
2. **Este arquivo** ([`HANDOFF_painter_t15_close.md`](HANDOFF_painter_t15_close.md))
   — estado pós-T1.5
3. [`docs/IntegracaoMultiAgente/DIRETRIZ.md`](IntegracaoMultiAgente/DIRETRIZ.md)
   v7.0 §0/§2/§3.A/§5
4. [`CLAUDE.md`](../CLAUDE.md)
5. **Spec T1.6:** [`docs/Painter_projeto/01_brush_engine.md`](Painter_projeto/01_brush_engine.md)
   §1.3.4 (Shape) + §1.3.8 (Color Dynamics) + §1.6 (Library)
6. **Plano T1.6:** [`docs/Painter_projeto/15_plano_de_implementacao.md`](Painter_projeto/15_plano_de_implementacao.md)
   §5 (Wave 1 milestones)
7. **Estado pós-T1.5** (referência da arquitetura):
   - [`crates/ph2d-tool-painter/src/tool.rs`](../crates/ph2d-tool-painter/src/tool.rs)
     module header (linhas 1-78) — W2 follow-ups documentados
   - [`crates/ph2d-painter-brush/src/{stamp_scheduler.rs, cpu_render.rs,
     stamp_pipeline.rs, shader/stamp.wgsl, library.rs}`](../crates/ph2d-painter-brush/src/)
     — engine state pós-T1.5
   - [`crates/ph2d-painter-brush/src/{shape.rs, color_dynamics.rs}`](../crates/ph2d-painter-brush/src/)
     — sub-struct fields a serem usados (já existem!)
   - [`crates/ph2d-painter-contracts/tests/architecture_painter_contract_surface.rs`](../crates/ph2d-painter-contracts/tests/architecture_painter_contract_surface.rs)
     — 74 gates ativos
8. **Memórias:**
   - [`project_painter_t15_complete_2026_05_26.md`](file:///Users/dibrioli/.claude/projects/-Volumes-MAC-EXTERNO-PROJETOS--PH2D-definitiva/memory/project_painter_t15_complete_2026_05_26.md)
   - [`feedback_audit_lens_diversity.md`](file:///Users/dibrioli/.claude/projects/-Volumes-MAC-EXTERNO-PROJETOS--PH2D-definitiva/memory/feedback_audit_lens_diversity.md)
   - [`feedback_perfection_no_deferrals.md`](file:///Users/dibrioli/.claude/projects/-Volumes-MAC-EXTERNO-PROJETOS--PH2D-definitiva/memory/feedback_perfection_no_deferrals.md)
   - [`feedback_fanout_registry_init_friction.md`](file:///Users/dibrioli/.claude/projects/-Volumes-MAC-EXTERNO-PROJETOS--PH2D-definitiva/memory/feedback_fanout_registry_init_friction.md)
   - [`feedback_scoped_commit_shared_index.md`](file:///Users/dibrioli/.claude/projects/-Volumes-MAC-EXTERNO-PROJETOS--PH2D-definitiva/memory/feedback_scoped_commit_shared_index.md)
   - [`feedback_communication_style.md`](file:///Users/dibrioli/.claude/projects/-Volumes-MAC-EXTERNO-PROJETOS--PH2D-definitiva/memory/feedback_communication_style.md)

---

## 7. Comando concreto pra próxima LLM começar

```
TRIAGEM
- Tarefa: T1.6 brush mature — shape atlas binding (GPU + CPU) +
          shape_scatter + shape_count + shape_rotation_follow +
          shape_flip_x/y + color_dynamics stamp-level jitter +
          library expand (≥3 built-ins)
- Caminho: (A) drop-crate continuation
            • edita  ph2d-painter-brush (scheduler + shader + cpu_render
                                          + library + atlas builder)
            • edita  ph2d-tool-painter (não precisa mudar API pública)
            • shell sem edits (nada de novo no input/bridge)
- Toca contrato congelado? NÃO
            • Stamp ABI 96B align(16) intocado
            • Tool / RasterEditTool / PanelEvent só consumidos
            • Brush sub-cap fields já existem (ShapeParams ≤ 20,
              ColorDynamicsParams ≤ 36 — usar slots existentes)
            • ph2d-painter-contracts arch-gate ativa naturalmente
- Razão (1 linha): T1.5 entregou "1 brush axis-aligned"; T1.6 entrega
  "brush maduro com shape atlas + multi-stamp + rotation + jitter +
  library expanded" — completa o padrão-ouro perceptual do brush
  engine, Wave 1 caminho natural.
- Esforço: 3-5 dias single-agent (paralelizável em sub-tasks 1-2-5)
```

**Leitura mínima antes de codar:** ver §6 acima. Tempo total estimado
~2h para internalização completa.

**Após o handoff, faça TRIAGEM da T1.6 e reporte ao Enio. Use o template
acima como ponto de partida.**

---

## 8. Mandato §0 ainda vale

**Padrão-ouro absoluto. Sem gambiarras. Sem "v1 que dá pro gasto".**

6 rounds × 12 lentes provaram: padrão-ouro é rotação de lentes até
zero Crit/High. Lance ≥2 lentes paralelas após CADA task substancial.

A barra é: **sucessor do Procreate em pelo menos 5 dimensões técnicas**
([14_inovacoes_extraordinarias §14.8.1](Painter_projeto/14_inovacoes_extraordinarias.md)).

T1.5 entregou Day-7 (primeira pintura visível). T1.6 entrega brush
maduro. T2+ adiciona Layers + Sidebar + ... Cada wave move a barra um
passo a mais.

Vai com tudo.
