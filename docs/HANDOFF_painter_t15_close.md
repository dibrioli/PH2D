# 🎨 HANDOFF — Painter PH2D · pós-T1.5 ship · 2026-05-26

**Para:** próxima LLM que pega o Painter na sessão seguinte.

---

## 1. Estado da entrega (esta sessão)

**Commits locais (3, não-pushados):**
- `14e416f` — feat(painter): T1.5 ship — Day-7 marker via CPU stamp render
  (20 files, +2925/-143)
- `9dbde5c` — fix(painter): T1.5 rounds 4+5 — perf, API, ship-readiness fixes
  (8 files, +246/-60)
- `d1b493c` — fix(painter): T1.5 round 6 — spec compliance polish
  (2 files, +59/-28)

**Branch:** `main` local, **51 commits ahead** de `origin/main`. Push é
fim-de-jornada sob ordem do Enio (NÃO push autônomo).

**Tasks fechadas nesta sessão (cumulativas com a sessão anterior):**

W1 completo até T1.5:
- T0.8 ✓ (ph2d-painter-contracts/ arch-gate) — sessão T1.3
- T1.1 ✓ (ph2d-tool-painter/ skeleton) — sessão T1.3
- T1.2 ✓ (pill dispatch data-driven) — sessão T1.3
- T1.3 ✓ (ph2d-painter-brush/ skeleton + Brush + Stamp ABI) — sessão T1.3
- T-color parcial ✓ (oklab.rs + mixbox_space.rs) — sessão T1.4
- T1.4 ✓ (shader/stamp.wgsl + stamp_pipeline.rs) — sessão T1.4
- **T1.5 ✓ (Day-7 marker — primeira pintura visível via CPU stamp render
  + ping-pong shader infra + shell wiring completo)** — esta sessão

**Verde (180 tests totais):**
- `ph2d-painter-brush`: 77 testes
- `ph2d-painter-contracts`: 74 testes (arch-gate)
- `ph2d-tool-painter`: 23 testes
- `ph2d-host-desktop` painter_input: 6 testes (pure-fn `uv_from_bounds`)
- Clippy `--all-targets -- -D warnings` limpo em painter crates
- Fmt limpo
- `cargo machete` limpo

**Auditoria adversarial executada nesta sessão: 6 rounds × 12 lentes paralelas rotacionadas:**

| Round | Lentes | Findings (C/H/M/L) | Status |
|-------|--------|--------------------:|--------|
| 1 | A spec+HR+det+regr-T1.4 / B shell+GPU+Rust+idiomatic | 4/6/12/4 | remediado |
| 2 | C test-coverage-vs-claim / D regressões-round-1 | 2/3/5/1 | remediado |
| 3 | E Day-7-validity / F cross-tool-lifecycle | 3/2/5/3 | remediado |
| 4 | G HR-3/perf / H API+safety+panic | 2/4/9/3 | remediado |
| 5 | I round-4-regr / J ship-readiness | 1/1/1/0 | remediado |
| 6 | M thread-safety / N spec+ADR+HR compliance | 0/0/0/4 | resolvido |

**Total: 75 findings** (12 Crit + 16 High + 32 Med + 15 Low) — todos
remediados em código OU documentados como W2 follow-ups explicitos.

**Round 6 com lentes NOVAS encontrou ZERO Critical/High** → padrão-ouro
threshold per regra [`feedback-audit-lens-diversity`](file:///Users/dibrioli/.claude/projects/-Volumes-MAC-EXTERNO-PROJETOS--PH2D-definitiva/memory/feedback_audit_lens_diversity.md).

---

## 2. Próxima task — TRIAGEM PRA O ENIO

T1.5 fechou Day-7 marker (primeira pintura visível). W1 tem várias
direções viáveis pra próxima sessão. **Sugestão ao Enio:** ele escolhe
entre as opções abaixo via `AskUserQuestion` na PRIMEIRA mensagem.

### Opção (A) — **T1.6 brush mature** (RECOMENDADO — caminho natural W1)

Spec: `docs/Painter_projeto/01_brush_engine.md` §1.3.4 (shape atlas wired
+ scatter + count + rotation_follow + flip_x/y) + §1.3.8 (color
dynamics jitter per-stamp).

O que entregar:
- Shape atlas binding (substitui `round_hard_shape()` inline procedural
  do shader → texture_2d_array<f32> via `library::round_hard_shape()`
  R8 256×256 + atlas builder em `library.rs`).
- `shape_scatter` (rotation aleatória por stamp em radianos).
- `shape_count` + `shape_count_jitter` (múltiplos stamps por pointer
  event espalhados conforme scatter).
- `shape_rotation_follow` (rotation = stroke direction).
- `shape_flip_x` / `shape_flip_y` (bits no `Stamp.flags`).
- Color dynamics jitter per-stamp (`color_dynamics.stamp_hue_jitter`
  etc. — apenas stamp-level, stroke-level e modulations W14+).
- Atualiza CPU equivalent em `cpu_render.rs` (paridade ULP-bounded
  preservada).

Esforço: 3-5 dias. Bloqueia: nada novo (T-input opcional, T-color full
opcional).

### Opção (B) — **T-input** (ADR-0050) — Pencil/tablet/curves

Crate novo `crates/ph2d-painter-input/` com `PointerSource` + per-device
`PressureCurve` + `TiltCurve` + `BarrelCurve` + `PalmRejectionConfig` +
`DriverQuirk` enum.

Substitui o stub `PointerSample` do scheduler. Habilita pressure-modulated
size + tilt-modulated rendering. Wave 2 sidebar precisa pra slider de
sensibilidade.

Esforço: 3-5 dias. Bloqueia: T1.6 quando colocar shape_pressure_roundness
real + tilt_roundness.

### Opção (C) — **T-color full** (ADR-0051) — `ColorProfile = 8 FROZEN`

Substitui `OklchColor` stub local (`crates/ph2d-tool-painter/src/params.rs:48`)
por `ph2d_color::OklchColor` canônico. Adiciona Display P3 + ProPhoto +
HDR + `ExportFormat`. Habilita Wave 2 color picker com hue em degrees
honrando o contrato R5-LI-N (refresh `stroke_color_oklab` mid-stroke).

Esforço: 2-3 dias. Bloqueia: Wave 2 color UI.

### Opção (D) — **T-durability mínimo** (ADR-0052) — WAL recovery

Crate novo `crates/ph2d-painter-stroke/durability/` — `StrokeJournal.
append` + `flush_every_8` (WAL). Recovery crash mid-stroke.

Hoje T1.5 fechou Day-7 sem durability (paint perde em crash). Pra
"production-ready" precisa.

Esforço: 3-4 dias.

### Opção (E) — **W2 follow-ups resolutos** (CONSOLIDAÇÃO)

Resolver os 7+ follow-ups documentados em
`crates/ph2d-tool-painter/src/tool.rs` module header:
- R3-LE-4 commit path wiring (sidebar Apply button OR Cmd+Enter)
- R3-LE-5/R4-LH-8 sprite-version tracking (stale canvas pós-external)
- R3-LF-3 drain_painter Result<(), Failed> + teardown gating
- R3-LF-4 toast warning em on_deactivate w/ painted strokes
- R4-LG-2 PREMUL canvas storage (35% per-pixel speedup)
- R4-LG-3 per-pixel match dispatch hoist via const generic
- R4-LG-6 CPU regime size cap (UI soft-cap brush size em 256)

Esforço: 4-6 dias. Torna T1.5 ship "real shippable feature" em vez de
só Day-7 smoke.

### Opção (F) — **T-tier** (ADR-0053) — small + atomic

Crate `ph2d-host` ganha `DeviceCapability` + `DeviceTier = 5 FROZEN` +
`GpuId` + `ThermalState` + `PlatformHost::tier()`. Permite gating de
features por device class.

Esforço: 1-2 dias.

### Recomendação resumida

**T1.6 (opção A)** é o caminho natural da Wave 1 — entrega "brush
maduro" com shape atlas + scatter + count + flip + color dynamics
stamp-level. Cobre maior parte do "padrão-ouro" perceptual sem precisar
de T-input/T-color full ainda. Pode ser executado em uma sessão única.

T-input (B) e T-color full (C) são paralelas — agentes separados podem
trabalhar simultaneamente sem colisão.

---

## 3. Estado git + working tree

```
HEAD: d1b493c
Branch: main (51 commits ahead de origin/main)
```

**Working tree alheio (NÃO TOCAR — outras sessões em vôo):**
- `docs/Painter_projeto/*.md` modified — outras sessões de docs Painter
- `docs/SESSION_ACTIVE.md` modified
- `docs/UI_Fonts/` untracked — agente tema
- `Cargo.lock` modified — múltiplos
- `shells/desktop/src/render_loop/color_equalization_bridge.rs` modified
  — provavelmente fmt drift de outra sessão
- `shells/desktop/src/render_loop/bgremoval_preview.rs` modified — fmt
  drift mas conteúdo de agentes bgremoval recentes
- `crates/ph2d-imageio-gif/` untracked — agente imageio
- `docs/Painter_projeto/14_inovacoes_extraordinarias.md` + outros
  untracked
- `test_strip` untracked

Outras sessões committaram em paralelo durante esta:
- Agente imageio: `9127011` (W1.T6 + auditoria 5-lente Onda 1)
- Agente bgremoval: `4eabab4`, `4084ee4`, `b3c15fa` (overlay halos +
  rotation/scale tracking)

---

## 4. Aprendizados desta sessão (LEIA antes de tocar Painter)

### 🔥 Padrão-ouro EXIGE rotação de lentes adversariais

6 rounds × 12 lentes paralelas rotacionadas:
- Round 1 (A+B): correctness baseline (4 Crit found)
- Round 2 (C+D): test-coverage-vs-claim + regressões round 1 (2 Crit)
- Round 3 (E+F): Day-7 end-to-end + cross-tool lifecycle (3 Crit
  reproducíveis hoje)
- Round 4 (G+H): HR-3/perf + API/safety (2 Crit — Arc canvas firehose
  + 3 defaults discordantes)
- Round 5 (I+J): round-4 fix regressão + ship readiness (1 Crit —
  `run_full` defeats Arc opt; 1 ship blocker `cargo machete`)
- Round 6 (M+N): thread-safety + spec compliance (0/0/0/4 — primeira
  round com zero Crit/High)

**Lição**: cada nova lente pega Crit/High que as anteriores não viram.
Padrão-ouro NÃO é "audit múltiplos rounds da mesma lente" — é "rotacionar
lentes até uma round com ZERO Crit/High". Memory
[`feedback-audit-lens-diversity`](file:///Users/dibrioli/.claude/projects/-Volumes-MAC-EXTERNO-PROJETOS--PH2D-definitiva/memory/feedback_audit_lens_diversity.md)
codifica isso.

### 🔬 Arc-based hot-path otimização SUBTLE

R4-LG-1 (Arc canvas) parecia clean — `Arc::make_mut` em queue_pointer +
`Arc::clone` em take_preview_arc. Mas R5-LI-C pegou que `run_full` usava
`Arc::unwrap_or_clone(Arc::clone(&self.canvas_rgba))` — o `Arc::clone`
ANTES de unwrap forçava sempre a branch clone. Fix: `mem::replace +
unwrap_or_clone`. **Lição**: Arc semantics são sutis; sempre considerar
"qual refcount no momento da unwrap?" e usar `mem::replace` quando
quer ownership transfer.

### 🎨 Default semantics — 3 sites devem alinhar

Round 4 R4-LH-1 pegou que `RenderingMode::default = UniformGlaze` (round 1
A-M1) NÃO batia com `Stamp::zeroed()::rendering_mode = 0 → from_u32(0) =
LightGlaze` E com `RenderingParams::default = LightGlaze`. Três defaults
discordantes. **Fix:** revert pra LightGlaze (alinhamento ABI-first).
**Lição**: quando muda default de tipo via `#[default]`, verifique
também `Default::default()` derivações em structs que usam o tipo +
ABI byte-zero decode.

### 📐 Anti-padrões catalogados (todos com gates executáveis)

- D-3.C2: premul invariant em uniform/intense blending (unmul → lerp →
  re-premul Porter-Duff)
- D-3.H7: shader_oklab_coefficients_bit_identical_with_rust
- R1 A-M7 / R6-LN-2: shader_flag_constants + textual_parity_all_six_modes
- R3-LE-1: stroke gap smear (break_segment em footprint exit)
- R3-LE-2: rubber-band leak under Painter (consume off-canvas Down)
- R3-LE-3: invisible default color (opaque orange L=0.7 C=0.18)
- R3-LF-2: silent stroke loss em selection drift (gate drive_source_push)
- R4-LG-1: Arc canvas (zero-copy preview drain)
- R4-LH-1: defaults triangulation (3 sites)
- R4-LH-3: set_source assert_eq release-active
- R5-LI-C: mem::replace antes de unwrap_or_clone

Esses já estão no código + tests. Próxima sessão **não precisa** re-
descobrir.

### 🚧 W2 follow-ups documentados (não-blockers Day-7, mas debt real)

7 follow-ups em `tool.rs` module header (linhas 15-66 — referência
canônica de débito técnico T1.5). LEIA antes de codar W2 sidebar/
brush studio/etc.

---

## 5. Memórias persistentes a salvar (próxima LLM)

Crie 1 memória após pegar este handoff:

`project_painter_t15_audit_complete_2026_05_26.md` (project type):

T1.5 fechado padrão-ouro pós 6 rounds × 12 lentes paralelas. 3 commits
locais (14e416f + 9dbde5c + d1b493c). 180 tests verde. 75 findings
remediados (12C + 16H + 32M + 15L). Round 6 com lentes NOVAS (M
thread-safety + N spec compliance) primeira round com ZERO Crit/High =
padrão-ouro threshold.

Já existe `project_painter_t15_complete_2026_05_26.md` desta sessão —
**update** essa memory em vez de criar nova (atualize commit count,
total finds, total rounds, total tests).

---

## 6. Comando concreto pra próxima LLM começar

```
TRIAGEM
- Decisão pendente: caminho W1 pós-T1.5 — qual task pegar?
  (A) T1.6 brush mature (RECOMENDADO — caminho natural W1, 3-5d)
  (B) T-input (ADR-0050) — Pencil/tablet/curves (3-5d, bloqueia T1.6 sub-features)
  (C) T-color full (ADR-0051) — substitui OklchColor stub (2-3d)
  (D) T-durability mínimo (ADR-0052) — WAL recovery (3-4d)
  (E) W2 follow-ups consolidação (7 itens, 4-6d) — torna T1.5 production-ready
  (F) T-tier (ADR-0053) — DeviceCapability/DeviceTier (1-2d)

- Toca contrato congelado (nodegraph / Tool / RasterEditTool / Stamp ABI)?
  - (A) NÃO — adiciona campos ao Brush sub-structs (cap room available)
  - (B) NÃO — novo crate ph2d-painter-input
  - (C) NÃO — ext ph2d-color + substituição transparente do stub
  - (D) NÃO — novo crate ph2d-painter-stroke
  - (E) NÃO — pure-fn refactor + UI wiring
  - (F) NÃO — ext ph2d-host

- Caminho: (A) drop-crate continuation OU (A) drop-crate fan-out
- Razão: depende da escolha do Enio acima

- Recomendação: T1.6 (opção A) — completa o "brush maduro" da Wave 1
  sem dependências bloqueantes. Pode ir em uma sessão única.
```

**Leitura mínima antes de codar (Tier-1 obrigatório):**

1. `docs/HANDOFF_painter.md` §0 (mandato) + §1 (LOOP) — fonte da
   governança "padrão-ouro absoluto"
2. `docs/HANDOFF_painter_t15_close.md` (este arquivo) — estado pós-T1.5
3. `docs/IntegracaoMultiAgente/DIRETRIZ.md` v7.0 §0/§2/§3.A/§5
4. `CLAUDE.md`
5. `crates/ph2d-tool-painter/src/tool.rs` module header (linhas 1-66) —
   W2 follow-ups documentados; **referência canônica de débito técnico
   T1.5**
6. `crates/ph2d-painter-brush/src/{stamp_scheduler.rs, cpu_render.rs,
   stamp_pipeline.rs, shader/stamp.wgsl}` — engine state pós-T1.5
7. `shells/desktop/src/{input_dispatch/painter_input.rs,
   render_loop/painter_bridge.rs, hero_intents/image_edit/painter.rs}` —
   shell wiring state pós-T1.5
8. `crates/ph2d-painter-contracts/tests/architecture_painter_contract_
   surface.rs` — 74 gates ativos (arch caps)
9. Memórias:
   - `project_painter_t15_complete_2026_05_26.md` (esta sessão)
   - `feedback_audit_lens_diversity.md` (multi-lens rotation)
   - `feedback_perfection_no_deferrals.md`
   - `feedback_fanout_registry_init_friction.md`

Após o handoff, faça TRIAGEM da próxima task escolhida pelo Enio e
reporte.

---

## 7. Mandato §0 ainda vale

**Padrão-ouro absoluto. Sem gambiarras. Sem "v1 que dá pro gasto".**

6 rounds × 12 lentes provaram: mesmo após 5 rounds "padrão-ouro", a
6ª round com novas lentes (thread-safety + spec compliance) provou
zero Crit/High = essa é a barra real, não "rodou alguns testes e
passou".

A barra é: **sucessor do Procreate em pelo menos 5 dimensões técnicas**.

T1.5 entregou Day-7 (primeira pintura visível). Próxima sessão move a
barra um passo a mais — escolha do Enio entre 6 opções viáveis.

Vai com tudo.
