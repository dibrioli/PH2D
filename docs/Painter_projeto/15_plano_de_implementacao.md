# 15 — Plano de implementação executável (tasks granulares + smokes)

> Plano detalhado, à prova de falhas, com smokes visuais o mais cedo possível para o Enio aprovar/corrigir/melhorar antes de seguir. 17 waves × N tasks cada. **Mandato §0 do HANDOFF_node_system:** padrão-ouro absoluto, sem gambiarras.

---

## §0. Princípios operacionais do plano

Esses 8 princípios governam toda task abaixo. Violar qualquer um = task NÃO está pronta.

### 0.1 Visual-first em cada wave

Cada wave entrega **pelo menos 1 smoke visual** que o Enio testa via `./play.command` (ou `PH2D_HERO_SCREEN=1 cargo run -p ph2d-host-desktop`). Implementação sem smoke não fecha. "Testes passam" não é suficiente — feature precisa ser visível.

### 0.2 Smoke intra-wave em waves grandes

Waves com >5 tasks (W1, W3, W4, W5, W6, W7, W8, W10, W15) ganham **smokes intermediários** — não esperar fim da wave para o Enio ver primeiro pixel. Smokes intermediários são marcados `🟦 SMOKE-INTRA` no plano.

### 0.3 Primeira pintura no W1, day ~7

W1 não fica "duas semanas de infra invisível para Enio". Tasks T1.1..T1.5 entregam **stroke deixa pixel na sprite** no day ~7. Polish (brush real, sidebar, undo) acontece em W1 day 8-14 e W2.

### 0.4 Tasks numeradas + dependências explícitas

Cada task tem ID `T-W.N` (W = wave, N = task index). Cada task lista **deps** (tasks anteriores que precisam estar verdes antes). Sem deps implícitas — se T2.3 precisa de T1.4, está escrito.

### 0.5 Critérios de aceitação concretos

Cada task fecha com **≥3 critérios** verificáveis. "Implementar X" não é critério. "X faz Y; teste Z passa; smoke W mostra V" é.

### 0.6 Anti-padrões listados por task

Cada task referencia explicitamente bugs já catalogados em [`docs/UI_Bugs/README.md`](../UI_Bugs/README.md) + [`docs/Image Tools Bugs/README.md`](../Image%20Tools%20Bugs/README.md) + DIRETRIZ v7.0 §5.3 que **deve evitar**. "Não faça assim" é tão importante quanto "faça assim".

### 0.7 Triagem antes de cada task

Cada task começa com **triagem do caminho** per [DIRETRIZ v7.0 §2](../IntegracaoMultiAgente/DIRETRIZ.md): (A) drop-crate, (B) scaffold central, (C) Coord-only. Sem triagem, task não começa.

### 0.8 Auditoria adversarial obrigatória

Cada wave fecha com **≥2 agentes auditores em paralelo** (lente distinta: corretude/edge-cases · paridade/determinismo · consistência docs↔código · qualidade gold-standard). Findings → fix → re-audit → erro-zero. Espelha loop §1 do [`HANDOFF_node_system.md`](../HANDOFF_node_system.md).

---

## §1. Definition of Done (DoD) global por wave

Toda wave fecha quando **TODOS** os itens abaixo estão verdes:

- ☑ Todas as tasks da wave verdes (critérios de aceitação concretos cumpridos).
- ☑ Arch-gates pertinentes verdes (lista em DIRETRIZ v7.0 §5.1).
- ☑ `cargo clippy --workspace --all-targets --features ph2d-spike/bevy_ecs -- -D warnings` verde.
- ☑ `cargo fmt --check` verde.
- ☑ Tests da wave: `cargo test -p <crates afetados>` 100% green.
- ☑ Memória dentro do budget (HR-13, vide [08 §8.2](08_performance_memory.md)).
- ☑ Frame budget dentro do sub-budget Painter (HR-4, vide [08 §8.1.1](08_performance_memory.md)).
- ☑ A11y nodes presentes para widgets novos (HR-12, gate `hr12_widgets_a11y`).
- ☑ Strings via i18n Fluent (HR-15, gate `hr15_no_hardcoded_ui_strings` + `no_literal_color` + `no_magic_numeric`).
- ☑ ≥2 auditorias adversariais paralelas → findings remediadas → re-audit erro-zero.
- ☑ **Smoke visual do Enio aprovado** (explicit; cada wave lista o que ele testa).

Wave NÃO fecha se algum item está vermelho. Implementador NÃO move para wave seguinte sozinho — Coord ou Enio confirma fechamento.

---

## §2. Tabela de smokes — visão de helicóptero (Enio bate o olho aqui)

Smokes do Enio por wave. Cada smoke é o **único critério que vale** para a wave ser declarada pronta visualmente.

| Wave | Smoke principal (o que o Enio testa) | Smokes intra-wave |
|------|---------------------------------------|---------------------|
| W0 | ✅ **RATIFICADO 2026-05-26.** Spec final + **11 ADRs Accepted** (0043..0053; cascata original 7 + 4 novos pós-regra-perfeição). | — |
| **W1** | **Abre app, clica Painter pill, escolhe sprite, pinta com Round Hard, vê pixel sair na sprite. Sai do tool, commita.** | 🟦 Day 3: pill aparece + clica = `println!`. 🟦 Day 5: stamp compute deposita 1 quadrado branco. 🟦 Day 7: stroke contínuo com `round_hard` aparece. |
| W2 | Painta com sidebar (size + opacity), undo (2-finger), redo (3-finger), troca cor (Classic picker), commita. | 🟦 Day 3: sidebar visível sem funcionar. 🟦 Day 6: sidebar funcional. |
| **W3** | **Cria 5 layers raster, blend modes diferentes, mask numa, clipping mask, reorder via drag, alpha-lock em outra. Compositor mostra resultado correto.** | 🟦 Day 4: 2 layers stack + blend Normal funciona. 🟦 Day 7: 22 blend modes pickable. 🟦 Day 10: mask + clipping funcionam. |
| **W4** | **Aplica adjustment layer HSB, vê mudança live no canvas; reordena pra cima do Curves; pinta mask preta numa área; reedita HSB depois.** | 🟦 Day 4: 1 adjustment (HSB) layer básico. 🟦 Day 8: 4 adjustments. 🟦 Day 12: 12 adjustments. |
| **W5** | **Abre Brush Studio, edita Round Hard mudando spacing e jitter, vê live preview; salva como "Round Soft"; troca para Mixbox em oil_round, mistura azul+amarelo → verde vibrante.** | 🟦 Day 3: Brush Studio panel skeleton. 🟦 Day 7: 3 sections funcionais (Stroke Path / Shape / Rendering). 🟦 Day 12: Mixbox visível. 🟦 Day 16: Procedural grain visible. |
| W6 | Brush Library aparece com 12 brushes; testa cada um pintando 1 trace; importa .brushset Procreate. | 🟦 Day 5: 6 brushes funcionais. |
| W7 | Abre Color Panel, testa 5 modos (Disc/Classic/Harmony/Value/Palettes); ColorDrop fluxo com threshold gestual; eyedropper tap-and-hold; abre Reference Companion com imagem importada. | 🟦 Day 4: Disc + Classic + Palettes. 🟦 Day 8: Harmony + Value. 🟦 Day 10: ColorDrop + Eyedropper. |
| W8 | Seleciona área Rectangle; Transform Warp; aplica Liquify Push num stroke; Clone source point + paint. | 🟦 Day 4: Rectangle selection. 🟦 Day 8: Transform 4 modos. 🟦 Day 12: 5 destructive adj. |
| W9 | Ativa 2D Grid; pinta com Drawing Assist on; troca para Symmetry Radial 8 → vê stroke replicado 8×; QuickShape hold-at-end → circle perfeito; Edit Shape ajusta nodes. | 🟦 Day 4: 4 guides visíveis. 🟦 Day 7: QuickShape detect 6 forms. |
| W10 | Faz undo via 2-finger tap; abre QuickMenu via gesture configurado; testa atalhos B/E/S/G; configura Pencil Pro squeeze. | 🟦 Day 4: 5 gestos canvas. 🟦 Day 8: QuickMenu funcional. |
| W11 | Toggla Animation Assist; adiciona 4 frames via Group; onion skin 2/2 visível; preview loop fps=12. | 🟦 Day 4: timeline UI. 🟦 Day 8: playback funcional. |
| **W12** | **Canvas 1080p com 200 strokes → Reproject to 4K offline (progress bar) → resultado nítido vs bilinear blur. Time-lapse capture → exporta MP4. Replay det-mode bit-identical em CI cross-OS.** | 🟦 Day 5: Reproject single-stroke baseline. 🟦 Day 10: Time-lapse off-thread. 🟦 Day 14: replay det-mode. |
| **W13** | **Prompt MCP "pinte hachura cruzada de grafite na seleção" → LLM gera 50 strokes reais → aparecem editáveis traço-a-traço.** | 🟦 Day 3: tool MCP esqueleto. 🟦 Day 7: paint_strokes funcional. |
| **W14** | **Pinta stroke com pencil; abre Stroke Inspector (Ctrl+Shift+I); seleciona stroke via lasso; troca brush retroativamente → vê stroke virar ink. Pressure scale ajusta peso visual.** | 🟦 Day 5: Inspector UI básico. 🟦 Day 10: lasso temporal funcional. |
| **W15** | **Seleciona watercolor_wash; pinta próximo a área molhada → vê sangramento físico real; tilt do device → tinta escorre na direção da gravidade.** | 🟦 Day 7: fluid sim baseline (sem giroscópio). 🟦 Day 14: giroscópio integration. 🟦 Day 21: graceful degrade testado. |
| W16 | Export PSD com 5 adjustment layers → abre em Photoshop → layers editáveis. Import PSD volta com layers intactas. | 🟦 Day 5: PSD export básico. 🟦 Day 10: adjustment layers mapping. |
| W17 | **Smoke full v1.0**: cria pintura completa em 1 sessão usando ~20 features. Cross-platform smoke (Mac + Linux + Web). | — (polish, sem smokes intermediários) |

---

## §3. Pré-W1 — 7 ADRs (W0)

**Caminho:** **(C) Coord-only**. Foundational + contratos congelados.

### T0.1 — ADR-0043 Painter contract

**Arquivo:** `docs/architecture/decisions/0043-painter-contract.md`

**Conteúdo obrigatório:**
- Surface de `ph2d-tool-painter::PainterTool` (impl Tool + impl RasterEditTool).
- `PainterUiEdit` enum + `PainterUiSnapshot` struct + `PainterParams` struct.
- Caps congelados: `PainterUiEdit ≤ N variants` (definir N), `PainterParams ≤ N campos`.
- Arch-gate `painter_contract_surface` ativo.
- Status: **Proposed** → ratificado em commit final.

**Critérios de aceitação:**
1. ADR escrito segundo template em [`docs/architecture/decisions/`](../architecture/decisions/) (Contexto / Decisão / Consequências / Alternativas).
2. Caps numéricos definidos (não "será definido depois").
3. Linka ADR-0040 (Tool isolation) + ADR-0041 (RasterEdit rename) como pré-requisitos.
4. Enio aprova explicitamente.

**Deps:** nenhuma.

**Risco/edge case:** caps muito apertados → futuro precisa amendment. Caps muito largos → "spec do papel" sem efetividade. Mediar: pegue tamanho-real do bgremoval contract como baseline e adicione 20% folga.

---

### T0.2 — ADR-0044 Brush Engine GPU contract

**Conteúdo obrigatório:**
- Schema `Brush` struct (todos params §1.3 de [01_brush_engine.md](01_brush_engine.md)) + cap `Brush ≤ N campos`.
- Schema `Stamp` (96 bytes, layout `repr(C, align(16))`).
- W1 unified shader → W5+ specialized refactor path.
- Mixbox axis: `pigment_mode: PigmentMode { Linear | Mixbox }`.
- Procedural Grain enum (4 variants).
- Cap arch-gate `painter_contract_surface`: `Brush ≤ N`, `Stamp ≤ 96 bytes`, `RenderingMode ≤ 6 variants`.
- Gate `painter_stamp_specialize_when_budget_pressure` (vide [01 §1.8.3](01_brush_engine.md)).

**Critérios de aceitação:**
1. Schema Brush + Stamp totalmente especificados.
2. Mixbox referencia paper Sochorová+Jamriška 2021 + repo open-source.
3. Procedural Grain enum tem 4 variants documentados com params.
4. Caps numéricos definidos.

**Deps:** T0.1.

---

### T0.3 — ADR-0045 Adjustment Layers contract

**Conteúdo obrigatório:**
- `AdjustmentLayer` struct + `AdjustmentKind` enum (12 variants).
- `AdjustmentParams` typed per-kind.
- Compositor recomposition strategy (dirty rect aware).
- PSD interop mapping table (5 mapeiam 1:1, 7 baked).
- Razão técnica para 5 adjustments destructive-only (Liquify, Clone, Recolor, Glitch, Mesh Warp).

**Critérios de aceitação:**
1. Os 12 `AdjustmentKind` variants documentados com `AdjustmentParams` typed.
2. PSD blend-mode-style mapping table completo.
3. Compositor recompose protocol claro.

**Deps:** T0.1.

---

### T0.4 — ADR-0046 Stroke Vector History format

**Conteúdo obrigatório:**
- `StrokeRecord` schema (path, pressure, tilt, azimuth, barrel, brush ref, color, RNG seed, tool_mode, brush_params_snapshot hash).
- `StrokeHistory::Full` (default desktop / iPad Pro) vs `StrokeHistory::Ring(1000)` (fallback low-end).
- `.ph2d-painter` schema v1 (postcard binário, HR-14 versionado).
- W12 Reproject protocol (CPU fallback det-mode 5 strokes/s; GPU mode 50 strokes/s).
- HR-5 determinismo opt-in (det-mode flag + fixed-point coords).

**Critérios de aceitação:**
1. `StrokeRecord` schema completo (todos os campos).
2. Determinismo bit-identical em det-mode docado.
3. Memory budget per platform (vide [08 §8.2](08_performance_memory.md)).

**Deps:** T0.1.

---

### T0.5 — ADR-0047 Painter MCP Stroke Engine

**Conteúdo obrigatório:**
- Tools MCP: `painter_paint_strokes`, `painter_modify_stroke`, `painter_query_strokes`, `painter_inspect_stroke`.
- Schemas: `StrokeSpec`, `StrokeMods`, `StrokeFilter`, `StrokeRef`.
- HR-11 governance (destructive flag + confirmation token).
- Audit log format (JSON Lines).

**Critérios de aceitação:**
1. 4 tools MCP especificadas com input/output schemas.
2. Governance HR-11 ratificada.
3. Audit log format documentado.

**Deps:** T0.1, T0.4 (precisa do `StrokeRecord`).

---

### T0.6 — ADR-0048 Stroke Inspector retroativo

**Conteúdo obrigatório:**
- UI contract: lasso temporal selection, overlay path, retroactive modify protocol.
- Compositor re-render-slice protocol (não re-roda stack inteiro).
- Performance gates (selecionar 100 strokes em canvas com 10k total ≤ 100ms).

**Critérios de aceitação:**
1. UI contract claro (selection model + interaction).
2. Compositor re-render-slice protocol documentado.

**Deps:** T0.4 (precisa `StrokeRecord`).

---

### T0.7 — ADR-0049 Fluid Brushes Extension

**Conteúdo obrigatório:**
- Crate `ph2d-painter-fluid` API.
- Shallow Water solver parameters + algoritmo.
- `PlatformHost::gyroscope() -> Option<Vec3>` extension.
- Graceful degrade policy (detect via `memory_budget.fluid_capable() && perf_budget.fluid_headroom_ms > 3.0`).
- Opt-in per brush via flag `brush.fluid_enabled`.

**Critérios de aceitação:**
1. Solver parameters definidos.
2. Graceful degrade matrix per platform.
3. `PlatformHost` extension proposta + impacto cross-shell documentado.

**Deps:** T0.1, T0.2.

---

### T0.8 — Ativar arch-gate vazio `painter_contract_surface`

**Local:** novo arquivo `crates/ph2d-painter-brush/tests/architecture_painter_contract_surface.rs` (vazio mas presente — gate desliga após criar o crate em W1).

**Critérios:** gate stub presente; build verde.

**Deps:** T0.1, T0.2.

---

### T0.9 — Aprovação dos ADRs pelo Enio ✅ RATIFICADO 2026-05-26

**Smoke W0:** Enio leu a cascata expandida (audits adversariais + sense-check veterano + audit padrão-ouro + re-audits) e ratificou em 2026-05-26.

**Cascata final ratificada (11 ADRs, era 7 — expandida pós-regra perfeição):**

| ADR | Título | Status |
|---|---|---|
| 0043 | Painter contract | Accepted |
| 0044 | Brush Engine GPU + Mixbox + Procedural Grain | Accepted |
| 0045 | Adjustment Layers (Tier 1 + Tier 2 = 24 ship v1) | Accepted |
| 0046 | Stroke Vector History (`.ph2d-painter` canon + `.ph2d-painter-cache` sidecar) | Accepted |
| 0047 | MCP Stroke Engine (streaming protocol) | Accepted |
| 0048 | Stroke Inspector (dirty-rect propagation FULL v1) | Accepted |
| 0049 | Fluid Brushes Extension (device matrix obrigatório) | Accepted |
| **0050** | **Device heterogeneity layer** (PointerSource + palm rejection + driver quirks) | Accepted |
| **0051** | **Color profile pipeline** (sRGB/P3/HDR/ProPhoto explicit invariants) | Accepted |
| **0052** | **Tear-resistant stroke commit** (WAL + crash recovery + suspend handler) | Accepted |
| **0053** | **Cross-platform tier policy** (universal feature parity 5 tiers) | Accepted |

**Critérios cumpridos:**
1. ✅ 11 ADRs com status `Accepted`.
2. ✅ Commits locais de todos 11 ADRs em main (não pushados; push é fim-de-jornada sob ordem Enio).

**Deps:** T0.1..T0.8 (T0.8 redefinida: criar crate `ph2d-painter-contracts/` em vez de gate diferido — primeira task de W1).

**Próximo:** W1 abre.

---

## §4. W1 — Neck: brush engine core + Stroke Vector History

**Objetivo:** primeira pintura no PH2D. `PainterTool` ativa o modo, brush `round_hard` deposita pixels na sprite ativa via stamp pipeline GPU. Stroke history vetorial grava.

**Caminho:** **(A) Drop-crate fan-out** per DIRETRIZ v7.0 §3.A.

**Crates a criar (W1 — atualizado pós-audit 2026-05-26 Gap 2+3):**
- **Sequenciais (T1.1..T1.6):** `ph2d-tool-painter`, `ph2d-painter-brush`, `ph2d-painter-stroke` (caminho crítico para primeira pintura).
- **Paralelos (T-input/T-color/T-durability/T-tier):** materializam ADRs ratificados W0 que **precisam estar disponíveis ANTES do smoke Day 7** (primeira pintura), senão Stamp.color_oklab fica em color space ambíguo, Apple Pencil tilt/pressure ficam dummies, stroke perdido em crash mid-stroke, e tier detection é incidental. Vide §4.X abaixo.

**Estimativa total:** ~14-18 dias (1 implementador full-time) — bump de 10-14 → 14-18 para acomodar T-input/T-color/T-durability/T-tier (~4-5 dias paralelos a T1.3..T1.6). Audit C confirmou em 2026-05-26: smoke "primeira pintura" Day 7 com sabor Procreate exige Mixbox + Apple Pencil pipeline + Linear sRGB working color + WAL — todos integrados.

### §4.X — Tasks paralelas a T1.3..T1.6 (audit 2026-05-26 Gap 2+3)

**T-input — Crate `ph2d-painter-input` (ADR-0050 §2.1)** — precede T1.5 brush implementation. Conteúdo: `PointerSource` enum + per-device `PressureCurve`/`TiltCurve`/`BarrelCurve` + `PalmRejection` state machine + `DriverQuirk` catálogo + `HoverState` + Apple Pencil prediction. Sem ele, `RawPointerSample` em ADR-0046 §2.3 grava `device_source` dummy e Stamp ignora tilt/azimuth. Estimativa: 3-5 dias (medições reais cross-device).

**T-color — Extensão `ph2d-color` (ADR-0051)** — paralelo a T1.3. Conteúdo: novos módulos `oklab.rs`, `display_p3.rs`, `prophoto.rs`, `hdr.rs`, `profile.rs`, `mixbox_space.rs` + `ColorProfile` enum FROZEN. Sem ele, `stamp.color_oklab` → `LinearSrgb working` conversion é hardcoded incidental; Mixbox roda em sRGB nominal vs linear ambíguo. Estimativa: 2-3 dias.

**T-durability — Extensão `ph2d-painter-stroke::durability/` (ADR-0052)** — paralelo a T1.4 (commit stroke). Conteúdo: `StrokeJournal` WAL + `CrashRecovery` + `AutoSave` + `SuspendHandler` + `AtomicWriter`. Sem ele, stroke perdido em crash mid-stroke (Procreate sangrou 2014-2017). Estimativa: 3-4 dias.

**T-tier — Extensão `ph2d-host` (ADR-0053)** — paralelo a T1.3. Conteúdo: `DeviceCapability` struct + `DeviceTier` enum FROZEN + `GpuId` catálogo + `ThermalState` + `PlatformHost::device_capability()` trait extension. Sem ele, escolha Mixbox GPU vs CPU é incidental; fluid sim ativação imune a heuristic. Estimativa: 2 dias.

**T-codegen-stateful — Extensão `tool-sync` (audit F2 follow-up)** — paralelo a T1.X. Conteúdo: tool-sync regenera tabela `STATEFUL_TOOL_IDS` em `crates/ph2d-editor-core/src/screens/hero/chrome/image_actions.rs` (data fallback para pré-registry tests) + os 2 testes hand-maintained em registry-init. Estimativa: 1 dia.

**Total W1 (sequencial + paralelo):** 14-18 dias, com smoke Day 7 ainda atingível se Implementador rodar tasks paralelas em background.

### T1.1 — Skeleton `ph2d-tool-painter`

**Conteúdo:**
- `crates/ph2d-tool-painter/Cargo.toml` (deps: ph2d-tool-registry, ph2d-editor-core, ph2d-painter-brush, ph2d-a11y, ph2d-core, ph2d-vector).
- `src/lib.rs` com `#![forbid(unsafe_code)]` PRIMEIRO (per DIRETRIZ v7.0 §3.A.2 — Cargo recusa workspace sem lib.rs).
- `pub const MANIFEST: ToolManifest` (id="painter", cluster="image_tools").
- `pub fn register(reg: &mut Registry)` + `pub fn make() -> Box<dyn Tool>` (sabor 3).
- `src/icon.rs` com BezPath placeholder (paint brush Lucide-style 24×24).
- `src/tool.rs` com `impl Tool` stub: `id() -> "painter"`, `label_key() -> "tool.painter.label"`, `icon_slug() -> "painter"`, `as_any_mut() -> Some(self)`. Métodos restantes panic-com-todo!.
- IconId variant `Painter` adicionado em [`crates/ph2d-editor-core/src/icons.rs`](../../crates/ph2d-editor-core/src/icons.rs) **em ordem alfabética** (gate `enum_order_matches_svgs`).
- SVG `docs/design/icons/painter.svg` (Lucide-derived).

**Critérios de aceitação:**
1. `cargo check -p ph2d-tool-painter` verde.
2. `cargo run -p ph2d-tool-sync` rodado; `cargo test -p ph2d-tool-registry-init` (staleness) verde.
3. `cargo test -p ph2d-tool-painter` (smoke test inicial) verde.
4. Gate `enum_order_matches_svgs` verde (IconId alfabético).
5. `tool_manifest_design_sync` verde (TOML em `docs/design/tools/painter.toml` se a chrome-sync precisar).

**Anti-padrões a evitar:**
- ❌ Pular `lib.rs` primeiro — quebra workspace de outras sessões paralelas.
- ❌ Adicionar IconId fora de ordem — quebra `ICON_CMDS_BY_ID` e TODOS os ícones quebram.
- ❌ Usar `--no-verify` se gate falha — fix root cause.

**Smoke 🟦 Day 1:** `cargo run -p ph2d-host-desktop` abre app sem panic. Painter pill NÃO aparece ainda (precisa de T1.2).

**Deps:** T0.9 (ADRs aprovados).

---

### T1.2 — Pill na topbar Image Tools cluster

**Conteúdo:**
- Adicionar Painter ao cluster `"image_tools"` em `tool_manifest_design_sync` source.
- Implementar `is_painter_tool(id) -> bool` em `editor-core/src/tools/` data-driven (não hardcoded `id == "painter"`).
- Chrome handler stub em `crates/ph2d-editor-core/src/screens/hero/chrome/painter_pill.rs` (caso precise lógica adicional além do chip render automático).
- `cargo run -p ph2d-chrome-sync` se aplicável.

**Critérios:**
1. Painter pill aparece na topbar Image Tools.
2. Clicar pill → `EditorAction::ActivateTool("painter")` despachado.
3. Smoke: `println!("painter activated")` no `Tool::on_activate`.

**Anti-padrões:**
- ❌ Hardcoded `id == "painter"` em múltiplos sites — vide [`docs/Image Tools Bugs/README.md`](../Image%20Tools%20Bugs/README.md) §2.b ("pertencimento é data-driven").
- ❌ Hit-test e paint do pill desacoplados — gateie pela mesma condição (DIRETRIZ v7.0 §5.3.3).

**🟦 SMOKE-INTRA Day 3:** Enio abre app, clica pill Painter → vê `println!` no terminal. Pill highlight active. Trocar de tool → desativa.

**Deps:** T1.1.

---

### T1.3 — Skeleton `ph2d-painter-brush` + struct Brush + struct Stamp

**Conteúdo:**
- `crates/ph2d-painter-brush/Cargo.toml` (deps: serde, postcard, bytemuck, ph2d-tokens, ph2d-gpu).
- `src/lib.rs` com `#![forbid(unsafe_code)]`.
- `src/brush.rs` com `struct Brush { /* todos os params §1.3 */ }` + `version: u32 = 1` (HR-14).
- `src/stamp.rs` com `#[repr(C, align(16))] struct Stamp` (96 bytes, vide [01 §1.4](01_brush_engine.md)).
- `src/library.rs` com `pub const ROUND_HARD: Brush = Brush { /* defaults */ }`.
- Test unit: `Stamp` é Pod + Zeroable; size = 96 bytes.

**Critérios:**
1. `cargo check -p ph2d-painter-brush` verde.
2. `bytemuck::cast` de `[Stamp]` para `[u8]` compila e tem tamanho esperado.
3. Test `stamp_layout_96_bytes` verde.

**Deps:** T0.2 (ADR-0044 com schema completo).

---

### T1.4 — Stamp pipeline CPU side (`StampScheduler` + `StampPipeline`)

**Conteúdo:**
- `src/stamp_pipeline.rs` com `StampPipeline` struct:
  - Pool pré-alocado `Box<[Stamp; 4096]>` (HR-3 zero-alloc).
  - `fn encode(stamps: &[Stamp], encoder: &mut CommandEncoder, layer_texture: &Texture)` — upload + dispatch compute.
- `src/stamp_scheduler.rs` com `StampScheduler`:
  - `fn advance(input: PointerEvent) -> SmallVec<Stamp, 32>` — recebe input, emite stamps com spacing/jitter.
  - Bumpalo arena resetada per-stroke (HR-3).

**Critérios:**
1. Pool de 4096 stamps pré-alocado.
2. `cargo test -p ph2d-painter-brush stamp_scheduler` verde.
3. Test `no_alloc_emit_stamps` (dhat-rs) → 0 allocs em 100 frames sintéticos com 1k stamps/frame.

**Anti-padrões:**
- ❌ `Vec::push` que realoca em hot path (HR-3).
- ❌ `Box::new` em hot path.
- ❌ Alocar buffer GPU per-frame — use buffer pré-criado.

**Deps:** T1.3.

---

### T1.5 — Stamp compute shader (`shaders/stamp.wgsl`)

**Conteúdo:**
- Shader unified W1 (per [01 §1.8.3](01_brush_engine.md)) com `switch (stamp.rendering_mode)`.
- Workgroup 8×8 (64 threads/stamp/tile).
- Lê Shape texture (atlas R8Unorm 256×256), aplica alpha, escreve layer_out.
- W1 versão: hardcoda Shape `round_hard` builtin (atlas layer 0). Grain ignorada (passa pela fase só).

**Critérios:**
1. Shader compila via naga (`cargo test -p ph2d-painter-brush shader_compile`).
2. Smoke GPU: render 1 stamp em texture vazia 256×256 produz blob redondo no centro.
3. Goldenless test (visual SSIM ≥ 0.99 contra referência rasterizada CPU).

**Anti-padrões:**
- ❌ `dpdx/dpdy` em shader (HR-5 determinismo, mesmo que Painter esteja em PresentWorld — disciplina futura).
- ❌ `PipelineLayout::Auto` (vide SKILL_Stack §10.5).
- ❌ Workgroup size dinâmico — fixar em 8×8.

**🟦 SMOKE-INTRA Day 5:** dispatch 1 stamp em textura headless 256×256 → renderiza blob redondo branco no centro. Sem chrome, sem brush real. Provará pipeline GPU funciona.

**Deps:** T1.3, T1.4.

---

### T1.6 — Integração com Sprite ativo (`impl RasterEditTool`)

**Conteúdo:**
- `impl RasterEditTool for PainterTool` em `ph2d-tool-painter/src/tool.rs`:
  - `set_source(&mut self, entity, pixels, w, h)` — receive sprite atual, cache.
  - `current_preview(&self) -> Option<PreviewCache>` — render current layer texture.
  - `take_pending_commit(&mut self) -> Option<RasterApply>`.
  - `run_full(&mut self) -> RasterApply` — full-resolution render.
  - `deactivate(&mut self)` — limpa cache (Wave 10 §A2 audit).
- Integração com `editor-core` Tool dispatch.

**Critérios:**
1. `impl RasterEditTool` cobre os 5 métodos.
2. `set_source` + `deactivate` zeram cache (gate `cache_zeroed_on_deactivate` — auditoria Wave 10 §A1+A2).
3. Cap surface verde (`architecture_tool_contract_surface`: RasterEditTool ≤ 5 métodos).

**Anti-padrões:**
- ❌ Skip `deactivate` cache zero — vira stale-frame bug (Wave 10 §A1+A2).
- ❌ Adicionar 6º método em RasterEditTool — quebra cap, exige amendment ADR-0041 (não permitido aqui).

**Deps:** T1.4, T1.5.

---

### T1.7 — Bridge no shell (`painter_bridge.rs`)

**Conteúdo:**
- `shells/desktop/src/render_loop/painter_bridge.rs` espelhando [`bgremoval_preview.rs`](../../shells/desktop/src/render_loop/bgremoval_preview.rs).
- Usa os 4 helpers de [`ph2d-tool-runtime`](../../crates/ph2d-tool-runtime/) (Wave 10 / ADR-0042):
  - `drive_source_push` — push fresh source pixels quando selection drifts.
  - `drive_preview_cache` — drain `current_preview()` em PreviewCache.
  - `drive_pending_commit` — captura Apply selection.
  - `drive_deactivate_cleanup` — clear cache no deactivate.
- Painta cache via `vector_scene.draw_image_rgba`.

**Critérios:**
1. Bridge compila + lib.rs do shell registra.
2. Bridge usa os 4 helpers de `ph2d-tool-runtime` (não duplica lógica).
3. Smoke: ativar Painter + selecionar sprite → bridge faz source push → cache atualiza.

**Anti-padrões:**
- ❌ Reimplementar `drive_*` helpers no bridge — use ph2d-tool-runtime.
- ❌ Esquecer `drive_deactivate_cleanup` — bug stale-frame quando troca tool.

**Deps:** T1.6.

---

### T1.8 — Skeleton `ph2d-painter-stroke` + StrokeRecord + StrokeHistory

**Conteúdo:**
- `crates/ph2d-painter-stroke/Cargo.toml`.
- `src/lib.rs` com `#![forbid(unsafe_code)]`.
- `src/record.rs` com `struct StrokeRecord { /* schema completo ADR-0046 */ }`.
- `src/history.rs` com:
  - `enum StrokeHistory { Full(Vec<StrokeRecord>), Ring { records: VecDeque<StrokeRecord>, cap: usize } }`.
  - `fn push(&mut self, record: StrokeRecord)`.
  - `fn undo(&mut self) -> Option<StrokeRecord>` (pop).
  - `fn redo(&mut self, record: StrokeRecord)`.
- Test: push 10 strokes + undo 5 → history.len() == 5.

**Critérios:**
1. `cargo check -p ph2d-painter-stroke` verde.
2. Test roundtrip serde postcard de `StrokeRecord` (HR-14 versionado).
3. `StrokeHistory::Full` vs `Ring(1000)` correto fallback baseado em `available_ram`.

**Deps:** T0.4 (ADR-0046 com schema final).

---

### T1.9 — Stroke history integration no PainterTool

**Conteúdo:**
- PainterTool guarda `stroke_history: StrokeHistory`.
- Em cada stroke completo: emite `StrokeRecord` + push na history.
- Em `Tool::on_activate`: detect platform e initialize `Full` ou `Ring`.

**Critérios:**
1. Pintar 5 strokes → history.len() == 5.
2. `StrokeHistory::push` zero-alloc no Full case (Vec pré-cresce).
3. Sample test: stroke serializado e deserializado é bit-identical.

**Deps:** T1.6, T1.8.

---

### T1.10 — Stroke loop full (PointerEvent → Stamps → GPU → Layer)

**Conteúdo:**
- Conectar `dispatch_pointer` (ph2d-input) → PainterTool → StampScheduler → StampPipeline → layer_texture.
- Bridge paints cached preview em cima do canvas.

**Critérios:**
1. Pintar com mouse/Pencil deposita pixels na sprite ativa.
2. Stroke completo (pointer down → moves → up) emite `StrokeRecord` que aparece em `stroke_history`.
3. Frame budget cabe em 3.5ms (per [08 §8.1.1](08_performance_memory.md)).
4. Gate `painter_no_alloc_hot_path` (dhat-rs) verde.

**🟦 SMOKE-INTRA Day 7 — PRIMEIRA PINTURA:** Enio abre app, ativa Painter, escolhe sprite, pinta com mouse → vê stroke contínuo aparecer com `round_hard` brush. **Esta é a milestone-marker do W1.** Se isso falha, plano para e revisita.

**Deps:** T1.5, T1.6, T1.7, T1.9.

---

### T1.11 — Smoke visual + audit final W1

**Conteúdo:**
- Enio testa o smoke principal (vide §2).
- ≥2 agentes auditores em paralelo (lentes: corretude / determinismo / consistência docs↔código / gold-standard).
- Findings remediadas até erro-zero.

**Critérios:**
1. Enio aprova explicitamente o smoke.
2. ≥2 audits paralelos com findings remediadas.
3. DoD global (§1) ✓ todos os itens.

**Deps:** T1.10.

---

## §5. W2 — Vertical MVP: sidebar + undo + Classic color

**Objetivo:** Painter usável end-to-end com mouse/Pencil. Sidebar com size+opacity, undo/redo, color picker básico, commit-to-sprite.

**Caminho:** **(A) Drop-crate** para `ph2d-panel-painter-sidebar` + **(B) Scaffold** para Painter Panel docado.

**Estimativa:** ~7-10 dias.

### T2.1 — Painter sidebar widget (`ph2d-panel-painter-sidebar` ou inline)

**Conteúdo:**
- Sidebar layout vertical (vide [11 §11.4](11_ux_chrome.md)): size slider (top) + modifier square + opacity slider (bottom) + undo/redo buttons.
- Usar `paint_slider_with_chip_layout` widget canon (DIRETRIZ v7.0 §5.2 — copia padrão de Inspector/BgRemoval).
- `store.link_slider_number(size_slider_id, size_chip_id)` (DIRETRIZ v7.0 §5.2 regra 1).
- Storage `0..1`; display via `display_override` (size em px; opacity em %).
- HUD durante stroke: chrome fade alpha → 0.4 (vide [11 §11.5](11_ux_chrome.md)).

**Critérios:**
1. Sidebar visível quando Painter ativo (takeover mode).
2. Drag size slider muda brush size em real-time (chip mostra valor px).
3. Drag opacity slider muda brush opacity em real-time.
4. Gates UI: `no_literal_color`, `no_magic_numeric`, `hr12_widgets_a11y`, `architecture_widget_loc_cap` (≤500 LOC) verdes.

**Anti-padrões:**
- ❌ Mirror manual slider↔chip — vide DIRETRIZ v7.0 §5.2 regra 1.
- ❌ Hex color literal em paint — vide gate `no_literal_color`.
- ❌ Chip sem stepper visível se Fase 1.A do `2026-05-ui-source-of-truth.md` fechou — copie do canon atualizado.

**🟦 SMOKE-INTRA Day 3:** Sidebar aparece, sliders **visíveis** mas não funcionam ainda. Smoke validado.

**Deps:** T1.11.

---

### T2.2 — Undo/redo via stroke history

**Conteúdo:**
- 2-finger tap → `EditorAction::Undo` → `stroke_history.undo()` → re-compose layer.
- 3-finger tap → `EditorAction::Redo`.
- Snapshot a cada 50 strokes para otimização (vide [01 §1.14.3](01_brush_engine.md)).

**Critérios:**
1. Undo restaura layer ao estado pre-stroke.
2. Redo reaplica último undo.
3. Undo 250× sem corromper layer.
4. Performance: undo um único stroke ≤ 200ms p99 em canvas 4K.

**🟦 SMOKE-INTRA Day 6:** Enio pinta 5 strokes → undo 3 → redo 2 → vê estado intermediário correto.

**Deps:** T1.9, T1.10.

---

### T2.3 — Classic color picker (single mode W2)

**Conteúdo:**
- `crates/ph2d-painter-color/` skeleton.
- Implementar **apenas Classic mode** em W2 (Disc/Harmony/Value/Palettes ficam para W7).
- `paint_classic_picker` widget no `editor-core` (W2.B scaffold).
- Color thumb no topo direito da Painter top bar.

**Critérios:**
1. Color Classic picker abre popover quando clica color thumb.
2. Pick cor → primary color do PainterTool muda → próximo stroke usa nova cor.
3. Hex input parsing válido (`#RGB`, `#RRGGBB`, `#RRGGBBAA`).

**Deps:** T1.10.

---

### T2.4 — Modifier square (basic: eyedropper-while-held)

**Conteúdo:**
- Modifier square no centro da sidebar.
- Default action: tap-and-hold no canvas + touch outro lugar = eyedropper.
- Configurável no W10 (Gesture Controls); W2 ship default.

**Critérios:**
1. Hold modifier + tap canvas = eyedropper sample.
2. Cor sampled vai pra primary slot.

**Deps:** T2.1, T2.3.

---

### T2.5 — Commit-to-sprite via `on_deactivate`

**Conteúdo:**
- `Tool::on_deactivate` triggers `RasterApply` com layer final.
- Asset DB recebe novo blake3.
- HR-6 content-addressed.

**Critérios:**
1. Trocar de tool → sprite asset atualizado.
2. Reabrir documento → pintura persiste.
3. Blake3 do `.ph2d-painter` muda quando layer muda.

**Deps:** T1.6, T1.7.

---

### T2.6 — A11y nodes para sidebar + color thumb

**Conteúdo:**
- Cada slider emite `Node` AccessKit Role::Slider com value text + min/max.
- Color thumb Role::Button + descriptive label.
- Modifier square Role::Button + dynamic label da ação configurada.

**Critérios:**
1. Gate `hr12_widgets_a11y` verde.
2. Screen reader announces slider values + button actions.

**Deps:** T2.1, T2.3, T2.4.

---

### T2.7 — Smoke + audit W2

**Smoke W2:** vide §2.

**Deps:** T2.1..T2.6.

---

## §6. W3 — Multi-layer raster + 22 blend modes + masks + clipping

**Objetivo:** sistema de layers completo. Layer panel docado, 22 blend modes, mask, clipping mask, reference layer, alpha-lock, group.

**Caminho:** **(B) Scaffold central** (`ph2d-panel-painter-layers` é crate novo; compositor multi-layer toca foundational `editor-core`).

**Estimativa:** ~14-18 dias.

### T3.1 — `LayerStack` struct + tipos básicos (Raster, Group)

**Conteúdo:**
- `crates/ph2d-painter-layers/` ou inline em `ph2d-tool-painter/src/layers.rs`.
- `enum LayerKind { Raster(RasterLayer), Group(Vec<LayerId>), Mask(MaskLayer), ClippingMask, Reference, AlphaLocked, Adjustment(AdjustmentLayer) /* placeholder W4 */ }`.
- `struct LayerStack { layers: Vec<Layer>, active: LayerId }`.
- Operações: push/pop/reorder/visibility toggle/opacity adjust/blend_mode set.

**Critérios:**
1. Cria 3 layers Raster + 1 Group anidado.
2. Reorder via drag updates stack order.
3. Visibility toggle re-composita.

**Deps:** T2.7.

---

### T3.2 — Compositor com dirty rect (top-down)

**Conteúdo:**
- `crates/ph2d-painter-layers/src/compositor.rs` (ou inline).
- Algoritmo: top-down recursive (vide [02 §2.11](02_layers.md)).
- Dirty rect tracking per layer.
- LayerCache `BTreeMap<LayerId, CachedTexture>` (BTreeMap por HR-5 disciplina mesmo em Present).
- Eviction LRU se VRAM apertado.

**Critérios:**
1. Compositor 10 layers @ 4K cache-hot ≤ 5ms (gate `layers_composite_50_4k_under_5ms`).
2. Dirty rect 1×1 → recompose ≤ stamp_size área.
3. Test golden contra Photoshop reference output.

**Anti-padrões:**
- ❌ Iteração de `std::HashMap` em compositor (HR-5; use `BTreeMap`/`EntityHashMap`).
- ❌ Re-composite full canvas em cada pixel pintado.

**🟦 SMOKE-INTRA Day 4:** 2 raster layers stack (blend Normal) → vê layer top com alpha por cima de bottom.

**Deps:** T3.1.

---

### T3.3 — 22 blend modes implementation

**Conteúdo:**
- `crates/ph2d-painter-brush/src/blend.rs` com `enum BlendMode` + `fn apply(mode, dst, src) -> Color`.
- 22 modos conforme [02 §2.2](02_layers.md) (lista enxuta).
- Golden tests para cada modo contra referência Photoshop rasterizada (SSIM ≥ 0.9999).

**Critérios:**
1. 22 modos implementados.
2. Gate `layers_blend_mode_golden` (22 sub-tests) verde.
3. Distinct outputs (cada modo produz pixel distinct from Normal).

**🟦 SMOKE-INTRA Day 7:** Layer top com blend mode = `Multiply` sobre bottom branco → produz cor do top.

**Deps:** T3.2.

---

### T3.4 — Layer panel widget (`ph2d-panel-painter-layers`)

**Conteúdo:**
- Crate novo `ph2d-panel-painter-layers` (caminho B — Coord scaffold) ou inline.
- Layout vertical scroll: layer thumb + name + visibility checkbox + opacity slider + blend mode dropdown.
- Layer panel docado quando Painter active.
- Pinch dois layers → Merge Down.
- Drag reorder.

**Critérios:**
1. Layer panel visível quando Painter active.
2. Tap layer → primary selection muda.
3. Pinch dois layers → merge.
4. Layer count + budget display ("10/75").

**Deps:** T3.1, T3.2.

---

### T3.5 — Mask layer

**Conteúdo:**
- `MaskLayer` struct (R8Unorm grayscale, bound a um Raster parent).
- Compositor aplica mask multiplicando alpha.
- Pincel pinta em mask em luminance space.

**Critérios:**
1. Mask layer cria, pinta branco/preto.
2. Toggle "Invert mask" inverte (1-x).
3. "Apply mask" baked (destrutivo).

**Deps:** T3.2, T3.4.

---

### T3.6 — Clipping mask

**Conteúdo:**
- Toggle não-destrutivo: layer N clip a layer N-1 imediatamente abaixo.
- Múltiplas clipping masks consecutivas → todas clippadas à base.

**Critérios:**
1. Clipping mask só pinta onde layer abaixo tem alpha > 0.
2. Toggle on/off não destrói pintura.
3. UI: thumb indented + seta para baixo indicando clip.

**🟦 SMOKE-INTRA Day 10:** Line art layer 1 + paint colorido layer 2 clipped → cor só aparece dentro das linhas.

**Deps:** T3.4.

---

### T3.7 — Reference layer + Alpha-lock + Group

**Conteúdo:**
- Reference layer: toggle não-destrutivo (afeta apenas ColorDrop comportamento — implementação completa em W7).
- Alpha-locked: pincel restringido ao alpha existente.
- Group: container anidado, blend mode aplica ao stack inteiro, max 8 níveis de profundidade.

**Critérios:**
1. Reference layer toggle visível no menu da layer.
2. Alpha-lock toggled = pincel só pinta onde alpha > 0.
3. Group anidado 8 níveis OK; 9° folha automaticamente para 8.

**Deps:** T3.4.

---

### T3.8 — Layer panel gestures

**Conteúdo:**
- Swipe right (secondary select), 2-finger tap (opacity slider inline), 2-finger swipe right (alpha lock toggle), 2-finger hold (select content).
- Pinch dois layers → merge (já em T3.4).
- Long press → properties popover.

**Critérios:**
1. Cada gesto da [02 §2.4](02_layers.md) funciona.
2. Long press abre menu com 13 operações.

**Deps:** T3.4.

---

### T3.9 — Smoke + audit W3

**Smoke W3:** vide §2.

**Deps:** T3.1..T3.8.

---

## §7. W4 — Adjustment Layers (12 non-destructive)

**Objetivo:** Painter ganha 12 adjustments aplicáveis como non-destructive layers — Crítica A absorvida (ADR-0045).

**Caminho:** **(B) Scaffold** + **(C) Coord-only** para compositor mudanças (foundational).

**Estimativa:** ~12-16 dias.

### T4.1 — AdjustmentLayer struct + AdjustmentKind enum

**Conteúdo:**
- `enum AdjustmentKind { HSB, ColorBalance, Curves, GradientMap, BrightnessContrast, GaussianBlur, MotionBlur, Bloom, Noise, Sharpen, Halftone, ChromaticAberration }`.
- `AdjustmentParams` typed per-kind.
- `struct AdjustmentLayer { kind, params, mask, opacity, blend_mode, ... }`.
- Adicionar variant `LayerKind::Adjustment(AdjustmentLayer)` ao stack.

**Critérios:**
1. 12 variants documentadas.
2. AdjustmentParams typed per-kind.
3. `AdjustmentLayer` serializável (HR-14 versionado).

**Deps:** T3.9.

---

### T4.2 — Compositor recomposition com adjustments

**Conteúdo:**
- Compositor algorithm atualizado (vide [02 §2.10.X.3](02_layers.md)).
- Recomposition strategy: cache de "intermediate acc" em points-of-cut (cada adjustment é cut point).
- Mudança em adjustment → recompose só do adjustment para cima (não do bottom).

**Critérios:**
1. Drag slider de adjustment recompose ≤ 1ms em 4K + 10 layers.
2. Test golden: adjustment-em-layer-form produces visual idêntico a apply-then-bake.
3. Gate `adjustment_layer_recomposition_perf_4k` verde.

**Anti-padrões:**
- ❌ Recompose full stack a cada slider drag — quebra perf.
- ❌ Cache invalido após reordering — recompute affected slice apenas.

**🟦 SMOKE-INTRA Day 4:** 1 adjustment layer HSB sobre 1 raster → drag slider hue → vê hue mudar live.

**Deps:** T4.1.

---

### T4.3-T4.14 — Implementar cada uma das 12 adjustments (1 task cada)

Cada task implementa **1 adjustment** com:
- Compute shader (`adjustments/<kind>.wgsl`) ou CPU function dependendo de complexidade.
- `AdjustmentParams::<Kind>` struct.
- UI controls em popover (sliders).
- Golden test contra Photoshop reference (SSIM ≥ 0.999).

Lista:
- **T4.3** HSB (H/S/B sliders).
- **T4.4** Color Balance (Highlights/Midtones/Shadows × RGB).
- **T4.5** Curves (RGB + per-channel, 8 control points).
- **T4.6** Gradient Map (luminance → custom gradient).
- **T4.7** Brightness/Contrast.
- **T4.8** Gaussian Blur (radius).
- **T4.9** Motion Blur (angle + distance).
- **T4.10** Bloom (threshold + intensity + size).
- **T4.11** Noise (clouds/billows/ridges + scale/octave/turbulence).
- **T4.12** Sharpen (amount + radius).
- **T4.13** Halftone (pattern + angle + scale).
- **T4.14** Chromatic Aberration (displacement + mode).

**Critérios por task (12 tasks):**
1. Compute shader compila + golden visual.
2. UI popover com sliders apropriados.
3. Adjustment layer mode + Layer mode + Pencil mode todos funcionam (vide [06 §6.3.1](06_selection_transform_adjustments.md)).
4. PSD export mapping (5 mapeiam 1:1; 7 baked com warning).

**🟦 SMOKE-INTRA Day 8:** 4 adjustments funcionais (HSB, Color Balance, Curves, Gradient Map).
**🟦 SMOKE-INTRA Day 12:** 12 adjustments funcionais.

**Deps:** T4.1, T4.2 (cada task em paralelo após esses dois).

---

### T4.15 — Layer panel UI (Add Adjustment menu)

**Conteúdo:**
- Botão "+ Adjustment" ao lado de "+ Layer".
- Submenu com 12 kinds.
- Adjustment layer thumb mostra ícone do kind + nome editável.

**Critérios:**
1. Botão Add Adjustment funcional.
2. Cada kind cria layer com defaults sensatos.

**Deps:** T4.3-T4.14, T3.4.

---

### T4.16 — Smoke + audit W4

**Smoke W4:** vide §2.

**Deps:** T4.1..T4.15.

---

## §8. W5 — Brush Studio + Mixbox + Procedural Grain híbrido

**Objetivo:** editor completo de brush (12 sub-sections) + Mixbox pigment mixing + 4 Procedural Grain variants.

**Caminho:** **(B) Scaffold** para `ph2d-panel-painter-brush-studio` (novo painel grande); **(A) Drop-crate** para Mixbox + Procedural se separamos.

**Estimativa:** ~16-22 dias.

### T5.1 — Brush Studio panel skeleton (12 collapsible sections)

**Conteúdo:**
- `crates/ph2d-panel-painter-brush-studio/`.
- Layout vertical scroll com 12 sub-sections (vide [01 §1.7](01_brush_engine.md)): Stroke Path, Stabilization, Taper, Shape, Grain, Rendering, Wet Mix, Color Dynamics, Dynamics, Apple Pencil, Properties, About.
- Cada section collapsible (Stroke Path expanded default; others collapsed).
- HR-18 enforcement: decomposto em sub-módulos `src/brush_studio/{stroke_path, stabilization, taper, shape, grain, rendering, wet_mix, color_dynamics, dynamics, pencil, properties, about}.rs` cada ≤ 600 LOC.

**Critérios:**
1. Brush Studio panel aparece via tap-and-hold em brush thumb.
2. 12 sections collapsible (tap section header expand/collapse).
3. Cada sub-module ≤ 600 LOC; cada `paint_*` ≤ 200 LOC (HR-18).

**🟦 SMOKE-INTRA Day 3:** Panel aparece com 12 section headers, todas collapsed.

**Deps:** T2.7.

---

### T5.2 — Stroke Path + Shape + Rendering sections funcionais (primeiros 3)

**Conteúdo:**
- Stroke Path: 4 sliders (spacing, spacing_jitter, jitter_lateral, falloff).
- Shape: source picker + scatter + count + roundness sliders.
- Rendering: rendering_mode dropdown (6 modos), pigment_mode dropdown (Linear/Mixbox), flow + wet_edges + burnt_edges.
- Live preview at top da Brush Studio (stroke fixed S-curve com pressure ramp 0→1→0).

**Critérios:**
1. Edit slider → live preview atualiza ≤ 16ms (debounce).
2. Save brush → `.ph2d-brush` postcard escrito.
3. Load brush → params restored.

**🟦 SMOKE-INTRA Day 7:** Edita Round Hard, muda spacing, vê preview, salva como "Round Soft".

**Deps:** T5.1.

---

### T5.3 — Stabilization + Taper + Grain sections

**Conteúdo:**
- Stabilization: streamline_amount, stabilization, motion_filtering.
- Taper: 7 params (pressure + touch taper).
- Grain: grain_source picker (None / Bitmap / Procedural / Imported), grain_behavior toggle, depth slider.

**Critérios:**
1. Sliders interativos.
2. Live preview reflete cada mudança.

**Deps:** T5.2.

---

### T5.4 — Mixbox pigment mixing implementation

**Conteúdo:**
- `crates/ph2d-painter-brush/src/mixbox.rs`.
- Port do paper Sochorová+Jamriška 2021 (open-source MIT — adapt do C/Python repo).
- `fn mix_mixbox(c1: vec3<f32>, c2: vec3<f32>, t: f32) -> vec3<f32>` em CPU + WGSL.
- Integrado no stamp compute shader como axis `pigment_mode`.

**Critérios:**
1. Function `mix_mixbox` retorna correct para 7 test cases (Hansa Yellow / Cadmium Red / etc.).
2. Stamp shader compila com Mixbox branch.
3. Golden test: blue + yellow with Mixbox → vivid green (Hue OKLCH ~140° ± 10°).

**🟦 SMOKE-INTRA Day 12:** Brush oil_round com pigment_mode=Mixbox: pinta azul, pinta amarelo por cima → vê verde vibrante. **Esta é a milestone da W5.**

**Anti-padrões:**
- ❌ Usar Mixbox em ColorDrop ou compositor — fica confuso. Mixbox **only** em stamp blending.
- ❌ Mixbox para inks/pencils default — comportamento previsível Linear é esperado.

**Deps:** T5.2.

---

### T5.5 — Procedural Grain enum + 4 variants

**Conteúdo:**
- `enum ProceduralGrain { SimplexNoise, GaborNoise, PaperWeave, SprayDot }` (per [01 §1.3.5.1](01_brush_engine.md)).
- WGSL functions: `compute_simplex(uv, seed, ...)`, `compute_gabor(uv, ...)`, `compute_paper_weave(uv, ...)`, `compute_spray_dot(uv, ...)`.
- Integrado no stamp shader como fork sobre `grain_source` variant.

**Critérios:**
1. 4 variants implementadas + golden tests.
2. Substituem 4 grãos default (paper_subtle, spray_grain, noise_white, noise_pink) per [01 §1.6.8](01_brush_engine.md).
3. Atlas VRAM cai de 64MB → ~32MB.

**🟦 SMOKE-INTRA Day 16:** Brush `paper_subtle` com procedural Simplex: zoom 400% → grão fica nítido (vs bitmap original que pixelizaria).

**Deps:** T5.2.

---

### T5.6 — Wet Mix + Color Dynamics + Dynamics sections

**Conteúdo:**
- 3 sub-sections do Brush Studio com params completos (per [01 §1.3.7](01_brush_engine.md), [01 §1.3.8](01_brush_engine.md), [01 §1.3.9](01_brush_engine.md)).

**Critérios:**
1. Wet Mix params editáveis quando wet_mix_enabled = true.
2. Color Dynamics jitter visível em strokes.
3. Dynamics speed_size/speed_opacity reagem à velocidade.

**Deps:** T5.2.

---

### T5.7 — Apple Pencil + Properties + About sections

**Conteúdo:**
- Apple Pencil: pressure curve graph + tilt curve graph (XY draggable 8 control points).
- Properties: max_size_px, min_size_px, max_opacity, smudge_pull.
- About: metadata (name, author, signature, reset_points).

**Critérios:**
1. Pressure curve interativa (drag control points).
2. Reset points memorizados.

**Deps:** T5.2.

---

### T5.8 — `.ph2d-brush` save/load + CLI

**Conteúdo:**
- `ph2d-painter-brush::save_brush(brush, path)` postcard binário.
- `ph2d-painter-brush::load_brush(path)` deserialize.
- `ph2d-painter-brush-cli` binary (vide [09 §9.7.3](09_export_interop.md)) com 6 subcommands.

**Critérios:**
1. Save → load → idêntico (`brush_format_roundtrip`).
2. Hash blake3 determinístico.
3. CLI inspect/edit/compile/diff/validate/convert funcionam.

**Deps:** T5.2-T5.7.

---

### T5.9 — Smoke + audit W5

**Smoke W5:** vide §2.

**Deps:** T5.1..T5.8.

---

## §9. W6 — Full brush library (12 brushes)

**Objetivo:** 12 default brushes em 6 categorias funcionais.

**Caminho:** **(A) Drop-crate** para cada brush (são instâncias do mesmo crate `ph2d-painter-brush`, não crates separados — preset data).

**Estimativa:** ~8-12 dias.

### T6.1-T6.12 — Implementar 12 default brushes

Cada brush é um preset de `Brush` struct com params específicos (vide [01 §1.6.1-1.6.6](01_brush_engine.md)):

- **T6.1** `pencil_2b`
- **T6.2** `pencil_charcoal`
- **T6.3** `ink_studio_pen`
- **T6.4** `ink_brush_pen`
- **T6.5** `marker_fine`
- **T6.6** `marker_chisel`
- **T6.7** `oil_round` (Mixbox default)
- **T6.8** `oil_bristle` (Mixbox default)
- **T6.9** `watercolor_wash` (Mixbox default)
- **T6.10** `watercolor_detail` (Mixbox default)
- **T6.11** `airbrush_soft`
- **T6.12** `airbrush_textured`

**Critérios por task (12):**
1. Brush registrado em `DEFAULT_BRUSHES` const.
2. Golden stroke test (S-curve com pressure ramp; SSIM ≥ 0.99 vs reference).
3. Param defaults sensatos (vide [01 §1.6.1-1.6.6](01_brush_engine.md)).

**🟦 SMOKE-INTRA Day 5:** 6 brushes funcionais (1 por categoria).

**Deps:** T5.9.

---

### T6.13 — Brush Library panel docked

**Conteúdo:**
- Panel overlay (popover full-width quando aberto, fecha ao selecionar).
- Categorias collapsable (6 categorias).
- Recents (8 last used).
- Search bar.
- Long-press brush → Brush Studio.

**Critérios:**
1. Aparece via tap em Brushes pill (#6 da topbar).
2. Selecionar brush → ativa.

**Deps:** T6.1-T6.12.

---

### T6.14 — Procreate `.brush` / `.brushset` importer (lossy)

**Conteúdo:**
- `crates/ph2d-painter-brush/src/import_procreate.rs`.
- Lê ZIP, extrai plist/JSON + PNGs.
- Mapeia 1:1 onde possível; log warnings onde Procreate-specific.

**Critérios:**
1. Fixture `.brushset` Procreate importa sem panic.
2. Brushes importados aparecem em "Imported" categoria.
3. Warning log estruturado.

**Deps:** T5.8.

---

### T6.15 — Smoke + audit W6

**Smoke W6:** vide §2.

**Deps:** T6.1..T6.14.

---

## §10. W7 — Color modes completos (5) + ColorDrop + Eyedropper

**Objetivo:** 5 modos completos do Color Panel + ColorDrop com threshold gestual + Eyedropper.

**Caminho:** **(B) Scaffold** para crate `ph2d-painter-color` + widgets coordenados.

**Estimativa:** ~10-14 dias.

### T7.1 — Disc mode

**Conteúdo:**
- Anel de Hue + disco de Chroma OKLCH.
- Reticle + mira secundária.
- 1-finger drag, 2-finger pinch (zoom).

**Critérios:**
1. Pick em OKLCH com mira correta.
2. Test roundtrip OKLCH → OKLab → display.

**Deps:** T6.15.

---

### T7.2 — Classic mode (já em W2; refinement)

**Conteúdo:**
- Refine para layout exato do spec (square + hue slider + H/S/B sliders).

**Deps:** T2.3.

---

### T7.3 — Harmony mode

**Conteúdo:**
- Disco com dots para current color + variantes derivadas (Complementary, Split Complementary, Analogous, Triadic, Tetradic).
- Tap em dot → troca cor.
- Toggle "Lock harmony".

**Critérios:**
1. Cada relationship correto (Complementary = +180°; Triadic = 120°×2; etc.).
2. Lock harmony move pares juntos.

**Deps:** T7.1.

---

### T7.4 — Value mode

**Conteúdo:**
- 7 sliders numéricos (L, C, H, R, G, B, Hex).
- Out-of-gamut warning badge.
- "Snap to gamut" button.

**Critérios:**
1. Sliders sincronizados (mudar L em OKLCH atualiza RGB display).
2. Hex parse tolerante (`#RGB`, `#RRGGBB`, etc.).

**Deps:** T7.1.

---

### T7.5 — Palettes mode

**Conteúdo:**
- Cards / Compact layout toggle.
- Tap swatch (select), long-press (menu), drag (reorder).
- Create from image (k-means) + "+ New empty" + import `.swatches` `.ase` `.gpl`.
- Set as Default (faixa horizontal aparece nas outras 4 abas).

**Critérios:**
1. Import `.swatches` Procreate funciona.
2. Default palette aparece nas outras 4 abas.

**Deps:** T7.1-T7.4.

---

### T7.6 — ColorDrop com threshold gestual

**Conteúdo:**
- Drag color thumb → canvas → drop.
- Após release, dedo ainda pressionado: drag horizontal ajusta threshold.
- Barra no topo do canvas mostra valor.
- Reference layer integration (se ativa, usa geometry dela).

**Critérios:**
1. Flood fill 8-conectado correto.
2. Threshold ΔE OKLab.
3. Reference layer respeitada (W3 já implementou flag).

**🟦 SMOKE-INTRA Day 10 (W7):** ColorDrop com threshold drag visível.

**Deps:** T7.1.

---

### T7.7 — Eyedropper tap-and-hold

**Conteúdo:**
- Tap-and-hold no canvas (800ms default) → loupe.
- Half superior nova / inferior atual.
- Drag explora.
- Customizable em Gesture Controls (W10).

**Critérios:**
1. Loupe aparece após delay configurado.
2. Sample correto do pixel sob cursor.

**Deps:** nenhuma (paralelo com T7.6).

---

### T7.8 — Reference Companion (Canvas + Image modes)

**Conteúdo:**
- Window flutuante com handle (drag) + resize (corners).
- 2 abas: Canvas (mini-mirror live) + Image (import PNG/JPEG/WebP/HEIC).
- Pinch/pan dentro da window funciona independent.
- Eyedropper samplável da window.

**Critérios:**
1. Drag/resize funcionam.
2. Eyedropper na window samplea da window, não do canvas.

**Deps:** T7.7.

---

### T7.9 — Smoke + audit W7

**Smoke W7:** vide §2.

**Deps:** T7.1..T7.8.

---

## §11. W8 — Selection + Transform + 5 destructive adjustments

**Objetivo:** 4 tipos de selection, 4 modos de transform, 5 destructive adjustments restantes.

**Caminho:** **(B) Scaffold + Coord-only** para Selection/Transform infra (toca foundational).

**Estimativa:** ~12-16 dias.

### T8.1-T8.4 — 4 selection types

- **T8.1** Automatic (magic wand + threshold drag) — atalho `W`.
- **T8.2** Freehand (lasso + node-editable polyline) — atalho `L`.
- **T8.3** Rectangle (drag corner-to-corner) — atalho `M`.
- **T8.4** Ellipse — `Shift+M`.

**Critérios por task:**
1. Selection visível com marching ants.
2. Modificadores Add/Sub/Intersect (Shift/Alt/Shift+Alt).
3. Feather + Smooth + Expand/Contract + Invert.

**🟦 SMOKE-INTRA Day 4:** Rectangle selection + invert + feather 16px.

**Deps:** T7.9.

---

### T8.5-T8.8 — 4 transform modes

- **T8.5** Freeform (8 handles)
- **T8.6** Uniform (preserve ratio)
- **T8.7** Distort (corner pinning)
- **T8.8** Warp (4×4 mesh)

**Critérios:**
1. Cada modo move handles corretamente.
2. Bilinear / NearestNeighbor toggle.
3. Snapping + Magnetics togglables.

**🟦 SMOKE-INTRA Day 8:** Transform 4 modos visíveis e funcionais.

**Deps:** T8.1-T8.4.

---

### T8.9-T8.13 — 5 destructive adjustments

- **T8.9** Liquify (8 sub-brushes: Push/Twirl L/R/Pinch/Expand/Crystals/Edge/Reconstruct).
- **T8.10** Clone (`Alt+Click` source + paint).
- **T8.11** Recolor / Replace Color.
- **T8.12** Glitch (4 sub-modes).
- **T8.13** Mesh Warp.

**Critérios por task:**
1. Layer mode + Pencil mode (Adjustment Brush).
2. Smoke visual per kind.

**Anti-padrões:**
- ❌ Tentar fazer Liquify como Adjustment Layer — distorção é destrutiva (vide [02 §2.10.X.2](02_layers.md)).

**🟦 SMOKE-INTRA Day 12:** Liquify Push brush stroke desloca pixels; Clone source + paint clone com offset.

**Deps:** T7.9.

---

### T8.14 — Smoke + audit W8

**Smoke W8:** vide §2.

**Deps:** T8.1..T8.13.

---

## §12. W9 — Drawing Guides + QuickShape + Symmetry com Drawing Assist

**Objetivo:** 4 tipos de guides, QuickShape com 6 detect forms + Edit Shape, Symmetry com replicação real-time.

**Estimativa:** ~8-12 dias.

### T9.1-T9.4 — 4 drawing guides

- **T9.1** 2D Grid.
- **T9.2** Isometric.
- **T9.3** Perspective (1/2/3-point com vps draggable).
- **T9.4** Symmetry (V/H/Quadrant/Radial).

**Critérios por task:**
1. Guide aparece como overlay.
2. Drawing Assist toggleable (`Shift+G`): stroke snap (2D/Iso/Persp) ou replica (Symmetry).
3. Params persistidos em `.ph2d-painter`.

**🟦 SMOKE-INTRA Day 4:** 4 guides selecionáveis e visíveis.

**Deps:** T8.14.

---

### T9.5 — QuickShape detect (6 forms)

**Conteúdo:**
- Hold-at-end (500ms default) → classifica em line/arc/polygon/ellipse/triangle/quadrilateral.
- Edit Shape button aparece pós-snap.
- Modificadores durante hold: 2-finger (proportion lock), 3-finger (15° increments para line).

**Critérios:**
1. 6 forms detectadas em fixture strokes.
2. Edit Shape mode com nodes draggable.

**🟦 SMOKE-INTRA Day 7:** Desenha onda + hold → snap em arc; desenha quadrado + hold → snap em quadrilateral.

**Deps:** T9.1.

---

### T9.6 — Symmetry com Drawing Assist

**Conteúdo:**
- Stroke replicado em real-time conforme symmetry mode (V/H/Quadrant/Radial N).
- Render via Vello overlay (não no canvas commit imediato).

**Critérios:**
1. Radial 8 → 8 segments rotacionados.
2. Stroke aparece em N positions sem lag visible.

**Deps:** T9.4.

---

### T9.7 — Smoke + audit W9

**Smoke W9:** vide §2.

**Deps:** T9.1..T9.6.

---

## §13. W10 — Gestures + QuickMenu + Gesture Controls + Atalhos desktop + Apple Pencil Pro

**Objetivo:** sistema completo de gestos + customização + atalhos + Pencil Pro features.

**Estimativa:** ~10-14 dias.

### T10.1-T10.8 — 8 canvas gestures default

- **T10.1** 2-finger tap (Undo) + tap-and-hold (rapid undo).
- **T10.2** 3-finger tap (Redo) + tap-and-hold.
- **T10.3** 3-finger swipe down (Copy/Paste menu).
- **T10.4** 3-finger scrub horizontal (Clear active layer).
- **T10.5** Pinch (zoom) + pinch+twist (rotate canvas) + Quick pinch (Snap to Fit).
- **T10.6** 4-finger tap (Toggle Zen Mode / Full Screen).
- **T10.7** Touch & Hold (Eyedropper — refinement T7.7).
- **T10.8** Draw + Hold at end (QuickShape — refinement T9.5).

**🟦 SMOKE-INTRA Day 4:** 5 gestos canvas funcionais.

**Deps:** T9.7.

---

### T10.9 — QuickMenu radial (6 slots × 4 menus salváveis)

**Conteúdo:**
- 6 slots em estrela, 60° wedge cada.
- Switch entre 4 menus salváveis (swipe left/right dentro do menu).
- Tap em slot → executa ação.

**Critérios:**
1. 6 wedges hit-test correto.
2. 4 menus persistidos em Painter Preferences.

**🟦 SMOKE-INTRA Day 8:** QuickMenu via gesture configurado abre + tap slot ativa ação.

**Deps:** T10.1-T10.8.

---

### T10.10 — Gesture Controls panel

**Conteúdo:**
- Full panel customizável (vide [05 §5.6](05_gestos_input.md)).
- Conflict validation.
- Reset to factory defaults.

**Critérios:**
1. Reassign gesture → persistido em Preferences.
2. Conflict detection alerta.

**Deps:** T10.9.

---

### T10.11 — Atalhos de teclado desktop (Blender-style)

**Conteúdo:**
- Lista canônica de [05 §5.7](05_gestos_input.md).
- Tools (B/E/S/G/V/M/W/L/T/I/H/Z).
- Brush params (`[`/`]` size; `Shift+[`/`]` 1%; `0-9` opacity %).
- Canvas (`Ctrl+Z/Y/S/E/J/N/O/W/+/-/0/1`, `Space+drag`, `R`, `Tab`).
- Color (`X` swap; `D` reset; `C` open picker; `Alt+drag` eyedropper transient).
- Layers (`Ctrl+Shift+N/G/Alt+G/,/'`).
- Customization in Painter Preferences > Keyboard Shortcuts + 3 keymap presets (PH2D default, Photoshop-like, Krita-like).

**Critérios:**
1. Cada atalho registrado dispara ação correta.
2. Modal switch (tap = switch, hold = temporary).
3. Conflict detection.

**Deps:** T10.10.

---

### T10.12 — Apple Pencil Pro (squeeze + barrel-roll + double-tap)

**Conteúdo:**
- `UIPencilInteraction` integration (iOS shell).
- Per-brush barrel_roll curve (no Brush Studio §1.3.10).
- Painter Preferences → Apple Pencil section (double-tap action + squeeze action).

**Critérios:**
1. Pencil Pro devices: squeeze opens QuickMenu (default config).
2. Barrel-roll modula brush size em testes com Pencil Pro real ou mock.
3. UI greyed em devices sem suporte (tooltip "requires Apple Pencil Pro").

**Deps:** T10.10.

---

### T10.13 — Smoke + audit W10

**Smoke W10:** vide §2.

**Deps:** T10.1..T10.12.

---

## §14. W11 — Animation Assist

**Objetivo:** timeline frame-based completa.

**Estimativa:** ~10-14 dias.

### T11.1 — FrameTimeline + UI

**Conteúdo:**
- `crates/ph2d-painter-anim/`.
- `struct FrameTimeline { frames: Vec<FrameMeta>, frame_rate, loop_mode, ... }`.
- UI bottom da tela quando Animation Assist toggleado.
- Drag reorder frames.

**Critérios:**
1. Timeline aparece via Actions → Animation Assist.
2. Cada layer = 1 frame (default) ou group = 1 frame.
3. Drag reorder updates ordem.

**🟦 SMOKE-INTRA Day 4:** Timeline UI visible com 3 frames mockup.

**Deps:** T10.13.

---

### T11.2 — Onion skin

**Conteúdo:**
- Prev/next frames config (0-6 each side).
- Opacity + colors (Auto / Red-Blue classic / Custom).
- Loop onion toggle.

**Critérios:**
1. Onion frames visíveis com opacity fade.
2. Toggle `O` on/off.

**Deps:** T11.1.

---

### T11.3 — Playback (Loop / Ping-Pong / One Shot)

**Conteúdo:**
- 3 loop modes.
- Frame rate 1-60 fps.
- Play/Pause/Frame/Reset controls + atalhos `Space`/`→/←/Home/End`.

**Critérios:**
1. 3 loop modes produzem ordem correta.
2. Frame rate respeitado em playback.

**🟦 SMOKE-INTRA Day 8:** Anima 4 frames com play 12fps loop.

**Deps:** T11.1.

---

### T11.4 — BG/FG layers + Hold duration

**Conteúdo:**
- Long-press frame thumb → Set as Background / Foreground / Regular.
- Hold duration slider 1-10.
- Visual: BG/FG locked + Hold number badge no thumb.

**Critérios:**
1. BG visível em todos os frames.
2. Hold=3 frame fica 3 ticks no playback.

**Deps:** T11.3.

---

### T11.5 — Smoke + audit W11

**Smoke W11:** vide §2.

**Deps:** T11.1..T11.4.

---

## §15. W12 — Reproject + Time-lapse + Replay determinístico

**Objetivo:** Vetor Oculto operacional offline; time-lapse capture; replay bit-identical cross-OS.

**Estimativa:** ~12-16 dias.

### T12.1 — Reproject to Resolution (offline operation)

**Conteúdo:**
- Dialog "Reproject canvas to X×Y" com progress bar.
- Mode toggle: det-mode (CPU exact) vs GPU mode (approximate).
- Reroda stroke history em nova resolução escalando coords.

**Critérios:**
1. Canvas 1080p reprojetado para 4K produz resultado nítido (visível vs bilinear blur).
2. Det-mode produz bit-identical em 3 OS.
3. Progress bar com ETA + cancel button.

**🟦 SMOKE-INTRA Day 5:** Reproject single-stroke baseline (1 stroke → resize → vê resultado).

**Anti-padrões:**
- ❌ Prometer "real-time scaling" — é offline com progress bar.
- ❌ Apply GPU + det-mode juntos — det-mode require CPU fallback.

**Deps:** T11.5.

---

### T12.2 — Time-lapse capture off-thread + export

**Conteúdo:**
- Capture: a cada N ms (depende de quality preset), `copy_texture_to_buffer` para staging buffer.
- Encoder thread (separado): encode para MP4/HEVC via native APIs (AVAssetWriter / MediaCodec / Media Foundation / ffmpeg fallback).
- Actions → Video → Export Time-lapse: Full length ou 30 Seconds (auto-edit).

**Critérios:**
1. Capture não causa stutter no paint (CPU stamp vs encoder thread separados).
2. Studio quality 4K canvas encoded válido.
3. Auto-edit 30s identifica momentos de pintura ativa.

**🟦 SMOKE-INTRA Day 10:** Time-lapse export 1080p Good MP4 válido.

**Deps:** T12.1.

---

### T12.3 — Replay determinístico (det-mode CPU fallback)

**Conteúdo:**
- `--features det-painter` build com CPU stamp processor (sem GPU compute).
- Fixed-point Q16.16 coords / Q8.8 pressure/tilt.
- RNG seeded per stroke.
- CI test `painter_replay_determinism` em Linux/macOS/Windows compara hashes.

**Critérios:**
1. Det-mode produz `.ph2d-painter` com canvas bit-identical em 3 OS.
2. Gate `painter_replay_determinism` verde em CI matrix.

**🟦 SMOKE-INTRA Day 14:** CI replay-hash matrix verde.

**Deps:** T12.1.

---

### T12.4 — Smoke + audit W12

**Smoke W12:** vide §2.

**Deps:** T12.1..T12.3.

---

## §16. W13 — Painter MCP Stroke Engine

**Objetivo:** LLM via MCP gera sequências de strokes reais com brushes nativos.

**Estimativa:** ~6-8 dias (LOC pequeno, lift baixo).

### T13.1 — Tool `painter_paint_strokes`

**Conteúdo:**
- `crates/ph2d-mcp/src/painter/paint_strokes.rs`.
- `#[mcp_tool(name = "painter_paint_strokes", destructive = true)]`.
- Input: `StrokeSpec[]`, brush_handle, layer_id, confirmation_token.
- Implementation: constrói `Vec<Stamp>` a partir de `StrokeSpec` e invoca stamp pipeline.

**Critérios:**
1. Tool MCP exposta no schema.
2. Test fixture: gera 5 strokes via MCP → 5 stroke records aparecem no history.
3. Audit log entry per call.

**🟦 SMOKE-INTRA Day 3:** Tool MCP esqueleto chamável (retorna empty).

**Deps:** T12.4.

---

### T13.2 — Tool `painter_modify_stroke`

**Conteúdo:**
- Receives `stroke_id` + `StrokeMods`.
- Modifica `StrokeRecord` no history.
- Compositor recompõe affected slice.

**Critérios:**
1. Modify brush retroativamente funciona.
2. Modify color funciona.
3. Audit log.

**Deps:** T13.1.

---

### T13.3 — Tool `painter_query_strokes` + `painter_inspect_stroke`

**Conteúdo:**
- Query com filter (bbox, brush, color, timestamp).
- Inspect retorna `StrokeRecord` full.

**Critérios:**
1. Query filter correct.
2. Inspect non-destructive.

**Deps:** T13.1.

---

### T13.4 — Audit log + governance HR-11

**Conteúdo:**
- `audit.log` JSON Lines: timestamp, agent, brush, layer, strokes count, blake3 before/after.

**Critérios:**
1. Cada call destrutiva loga.
2. Sem token → tool refuses with hint.

**🟦 SMOKE-INTRA Day 7:** Enio com Claude/GPT MCP client envia "pinte 5 strokes paralelos verticais" → 5 strokes aparecem.

**Deps:** T13.1-T13.3.

---

### T13.5 — Smoke + audit W13

**Smoke W13:** vide §2.

**Deps:** T13.1..T13.4.

---

## §17. W14 — Stroke Inspector retroativo

**Objetivo:** UI dedicada para selecionar strokes individuais e modificar retroativamente.

**Estimativa:** ~12-16 dias (UI complexa).

### T14.1 — Stroke Inspector panel docked

**Conteúdo:**
- `crates/ph2d-panel-painter-inspector/`.
- Lista de strokes (scroll vertical) com thumb + brush + color + timestamp.
- Tap stroke → seleção primary (visualiza path no canvas).
- Atalho `Ctrl+Shift+I`.

**Critérios:**
1. Inspector aparece via atalho ou Actions.
2. Stroke list scrollable.
3. Tap stroke → visualization overlay com path.

**🟦 SMOKE-INTRA Day 5:** Inspector UI básico visível listando strokes.

**Deps:** T13.5.

---

### T14.2 — Lasso temporal selection no canvas

**Conteúdo:**
- Drag lasso (freeform ou retangular) seleciona strokes que intersectam.
- Multi-select via Shift+lasso.

**Critérios:**
1. Lasso retangular seleciona strokes em bbox.
2. Lasso freeform seleciona strokes que passam pela área.
3. Performance: lasso em canvas com 10k strokes ≤ 100ms p99.

**🟦 SMOKE-INTRA Day 10:** Lasso temporal funcional.

**Deps:** T14.1.

---

### T14.3 — Modify brush retroativamente

**Conteúdo:**
- Tool MCP `painter_modify_stroke` reusado (T13.2).
- UI: dropdown de brush + apply button.

**Critérios:**
1. Trocar brush retroativo → re-render only affected slice.
2. Performance: modify 10 strokes ≤ 500ms.

**Deps:** T14.2, T13.2.

---

### T14.4 — Modify color + pressure scale

**Conteúdo:**
- Color picker inline + slider de pressure_scale.

**Critérios:**
1. Modify color retro funciona.
2. Pressure scale ajusta peso visual.

**Deps:** T14.3.

---

### T14.5 — Delete stroke + undo

**Conteúdo:**
- Delete específico de stroke (vs undo cronológico).
- Undo recupera stroke deletado.

**Critérios:**
1. Delete remove stroke do history.
2. Undo recupera (mantém stack consistente).

**Deps:** T14.3.

---

### T14.6 — Smoke + audit W14

**Smoke W14:** vide §2.

**Deps:** T14.1..T14.5.

---

## §18. W15 — Fluid Brushes Extension

**Objetivo:** crate `ph2d-painter-fluid` opt-in com Shallow Water sim + giroscópio + graceful degrade.

**Estimativa:** ~18-24 dias (research + tuning + cross-platform).

### T15.1 — Crate `ph2d-painter-fluid` skeleton + ShallowWaterSolver

**Conteúdo:**
- `crates/ph2d-painter-fluid/`.
- `src/solver.rs` com Shallow Water 2D em compute shader (1/4 canvas res).
- Params: viscosity, evaporation_rate, gravity (default downward).
- `shaders/fluid.wgsl` compute pass.

**Critérios:**
1. Solver step funciona em texture 1024² (1/4 de 4K).
2. Per-frame cost ≤ 3.5ms em Apple M2.
3. Golden test: tinta deposita em uma área, frame-step → tinta espalha.

**🟦 SMOKE-INTRA Day 7:** Pintar watercolor_wash em texture → vê tinta espalhar nos frames seguintes.

**Anti-padrões:**
- ❌ Fluid sim sem dirty rect — recomputar canvas inteiro a cada frame é overkill.
- ❌ Tentar Navier-Stokes full — Shallow Water cobre o use case.

**Deps:** T14.6.

---

### T15.2 — GyroscopeInput via PlatformHost

**Conteúdo:**
- `trait PlatformHost { fn gyroscope(&self) -> Option<Vec3>; }` extension.
- iPad/iPhone shell (Swift): CMMotionManager.
- Android shell (Kotlin): SensorManager.
- Desktop: `None`.

**Critérios:**
1. iPad smoke: tilt device → gyroscope retorna Vec3 com gravidade.
2. Desktop: gyroscope returns None.
3. Solver consome gravity correto.

**🟦 SMOKE-INTRA Day 14:** Tilt iPad → tinta escorre na direção da gravidade.

**Deps:** T15.1.

---

### T15.3 — Opt-in flag per brush + graceful degrade

**Conteúdo:**
- `brush.fluid_enabled: bool`.
- Detection runtime: `if memory_budget.fluid_capable() && perf_budget.fluid_headroom_ms > 3.0`.
- Fallback: traditional Wet Mix matemático (vide [01 §1.3.7](01_brush_engine.md)).

**Critérios:**
1. Brush watercolor_wash em desktop M2: fluid ativo.
2. Brush watercolor_wash em web/Android entry: traditional Wet Mix (identidade preservada).
3. Test: simular `available_ram < threshold` → fluid disabled gracefully.

**🟦 SMOKE-INTRA Day 21:** Graceful degrade testado cross-platform (desktop ativo, web fallback).

**Deps:** T15.2.

---

### T15.4 — Watercolor + Oil brushes com fluid

**Conteúdo:**
- 4 default brushes ganham `fluid_enabled = true`: oil_round, oil_bristle, watercolor_wash, watercolor_detail.
- Sub-params per brush.

**Critérios:**
1. Cada brush testa fluid behavior visualmente.
2. Sangramento físico em watercolor visible quando próximo a área molhada.

**Deps:** T15.3.

---

### T15.5 — Smoke + audit W15

**Smoke W15:** vide §2.

**Deps:** T15.1..T15.4.

---

## §19. W16 — Export interop full + format freeze

**Objetivo:** PSD com adjustment layers mapping; PSD import; TIFF/WebP/animated; `.ph2d-painter` v1 freeze.

**Estimativa:** ~10-14 dias.

### T16.1 — PSD export com adjustment layers mapping

**Conteúdo:**
- 5 adjustment kinds mapeiam 1:1: HSB, Color Balance, Curves, Gradient Map, Brightness/Contrast.
- 7 baked com warning: Gaussian Blur, Motion Blur, Bloom, Noise, Sharpen, Halftone, Chromatic Aberration.
- 22 blend modes mapping (lista [09 §9.3.2](09_export_interop.md)).
- Masks preservadas.

**Critérios:**
1. PSD export do Painter abre em Photoshop com 5 adjustment layers editáveis.
2. Mapping completo em test fixture.

**🟦 SMOKE-INTRA Day 5:** PSD export básico.

**Deps:** T15.5.

---

### T16.2 — PSD import (lossy)

**Conteúdo:**
- `psd` crate (Rust mature).
- Raster layers + Mask + Group preservados.
- Adjustment layers PSD → discarded with warning.
- Text/Smart Objects flatten with warning.

**Critérios:**
1. Fixture PSD com 5 layers + 2 adjustment layers + 1 mask carrega.
2. Warnings estruturados.

**🟦 SMOKE-INTRA Day 10:** PSD adjustment layers mapping completo.

**Deps:** T16.1.

---

### T16.3 — TIFF multi-layer + WebP + Animated WebP

**Conteúdo:**
- TIFF: multi-page com layers.
- WebP: lossy/lossless toggle + alpha.
- Animated WebP: anim assist export.

**Critérios:**
1. Cada formato exporta válido.
2. Color profile (ICC) embedded em PNG/JPEG/TIFF.

**Deps:** nenhuma (paralelo).

---

### T16.4 — `.ph2d-painter` schema V1 freeze

**Conteúdo:**
- Arch-gate ativo: `architecture_painter_file_v1_freeze`.
- HR-14: campo `version: u32 = 1`; migrator stub para futuro v2.

**Critérios:**
1. Schema documentado.
2. Roundtrip save → load → bytes idênticos (blake3).

**Deps:** T16.1, T16.2.

---

### T16.5 — Smoke + audit W16

**Smoke W16:** vide §2.

**Deps:** T16.1..T16.4.

---

## §20. W17 — Final polish + v1.0 release

**Objetivo:** bug bash, perf tuning, i18n bundles complete, a11y review WCAG 2.2 AA.

**Estimativa:** ~14-21 dias.

### T17.1 — Bug bash

**Conteúdo:**
- Enio + agentes auditores caçam bugs em todas as features.
- Lista priorizada de bugs.
- Fix all P0/P1.

**Critérios:**
1. Zero P0 bugs.
2. P1 bugs todos com PR aberto.

**Deps:** T16.5.

---

### T17.2 — Performance tuning

**Conteúdo:**
- Profile com tracy/puffin.
- Identify hot spots.
- Stamp shader specialization (W5+ target) se gate `painter_stamp_specialize_when_budget_pressure` indica.
- Memory budget audit per platform.

**Critérios:**
1. Frame budget 4.5ms ≤ no baseline hardware.
2. Memory budget dentro de [08 §8.2](08_performance_memory.md) per platform.

**Deps:** T17.1.

---

### T17.3 — i18n bundles complete

**Conteúdo:**
- `crates/ph2d-tool-painter/locales/pt-BR.ftl` + `en-US.ftl` 100% complete.
- Gate `hr15_no_hardcoded_ui_strings` verde.

**Critérios:**
1. Cada string Painter via `t!()`.
2. pt-BR + en-US bundles 100%.

**Deps:** nenhuma (paralelo).

---

### T17.4 — A11y review WCAG 2.2 AA

**Conteúdo:**
- Auditoria a11y completa (Roles, labels, contrast, navigation).
- Screen reader smoke (VoiceOver macOS + NVDA Windows).
- Reduced motion respected.

**Critérios:**
1. Todos os widgets emitem Node (gate `hr12_widgets_a11y`).
2. WCAG 2.2 AA verde (color contrast AA mínimo, AAA quando possível).
3. Reduced motion → fade animations disabled.

**Deps:** nenhuma (paralelo).

---

### T17.5 — Painter v1.0 release marker

**Conteúdo:**
- Tag `painter-v1.0` no git.
- Update `SKILL_Stack` §11.9 com Painter como subsistema completo.
- Memory + CHANGELOG entry.
- ADR amendments se necessário.

**Critérios:**
1. Tag criada e pushed.
2. Docs atualizados.
3. **Smoke full v1.0 do Enio aprovado**.

**Deps:** T17.1-T17.4.

---

### T17.6 — Smoke full v1.0 + audit final

**Smoke W17:** Enio cria pintura completa em 1 sessão usando ~20 features. Cross-platform (Mac + Linux desktop pelo menos; iPad smoke se disponível).

**Critérios finais:**
1. Pintura entregável em qualidade equivalente ou superior a Procreate.
2. Performance, memory, a11y, i18n verdes.
3. **Enio declara "Painter v1.0 chegou em qualidade pro."**

**Deps:** T17.1..T17.5.

---

## §21. Matriz de auditoria adversarial

Cada wave termina com **≥2 auditores em paralelo**. Lentes diferentes garantem cobertura:

| Wave | Lente 1 | Lente 2 | Lente 3 (opcional) |
|------|---------|---------|--------------------|
| W1 | Corretude do stamp pipeline (matemática + GPU) | A11y + i18n + UI canon | — |
| W2 | UI canônica (slider/chip link + chip pill no_stepper) | A11y screen reader | — |
| W3 | Compositor correctness (blend modes golden) | Memory budget per platform | Determinismo |
| W4 | Adjustment layer recomposition perf | PSD interop mapping correctness | — |
| W5 | Brush Studio correctness (12 sections) | Mixbox + Procedural Grain golden | LOC caps (HR-18) |
| W6 | Brush library 12 golden tests | Procreate importer warnings | — |
| W7 | Color modes correctness (5 modos) | ColorDrop threshold gestural | A11y dos 5 modos |
| W8 | Selection/Transform correctness | Adjustments destructive 5 golden | — |
| W9 | Guides correctness (4 tipos) + QuickShape detect | Symmetry replication | — |
| W10 | Gestures conflict detection | Keyboard shortcuts conflict | A11y screen reader |
| W11 | Animation timeline correctness | Onion skin rendering | Performance |
| W12 | Reproject correctness + det-mode | Time-lapse off-thread correctness | Cross-OS hash |
| W13 | MCP API correctness + governance | Audit log completeness | Quality emergence (prompts) |
| W14 | Stroke Inspector UI correctness | Lasso temporal performance | Modify retroactive correctness |
| W15 | Shallow Water solver correctness | Graceful degrade testing | Gyroscope integration |
| W16 | PSD export/import correctness (round-trip) | Format freeze validation | Memory growth check |
| W17 | Bug bash final | Cross-platform smoke | Release marker validation |

---

## §22. Critério de saída do projeto (Painter v1.0)

Painter v1.0 declarado pronto quando:

- ☑ Todas as 17 waves fechadas (DoD §1 cumprida em cada).
- ☑ Smokes do Enio aprovados (todos os 17 smokes principais + intra-wave).
- ☑ ≥2 auditorias adversariais por wave concluídas a erro-zero.
- ☑ Performance dentro do budget HR-4 (sub-budget Painter 4.5ms).
- ☑ Memory dentro do budget HR-13 per platform (vide [08 §8.2](08_performance_memory.md)).
- ☑ A11y WCAG 2.2 AA verde + screen reader smoke.
- ☑ i18n bundles 100% (pt-BR + en-US).
- ☑ Cross-platform smoke (Mac desktop, Linux desktop, web target, iPad se hardware disponível).
- ☑ ADR-0043..0049 ratificados.
- ☑ Tag `painter-v1.0` no git.
- ☑ **Enio declara "Painter v1.0 chegou em qualidade pro" (espelha frase do Crisp Heavy).**

---

## §23. Como o Enio acompanha (HUD do progresso)

Para cada wave em andamento, o Coord mantém uma linha no `docs/SESSION_ACTIVE.md` ou em arquivo dedicado `docs/Painter_projeto/progress.md` (a definir; este plano NÃO cria por enquanto):

```
WAVE N - <título>
  Status: in_progress
  Task atual: T-N.M
  Smoke intra-wave próximo: <descrição>
  Bloqueios: <none | descrição>
  Auditoria: <pending | in_progress | done>
  Smoke do Enio: <pending | aguardando | aprovado>
```

Enio bate o olho periodicamente (1×/dia ou no fim de cada wave). Não há "polling de CI" — push e CI são prerrogativa de fim-de-jornada per DIRETRIZ v7.0 §8.

---

## §24. Estimativa total

| Conjunto | Estimativa |
|----------|------------|
| W0 (ADRs) | 1-2 semanas |
| W1 (Neck) | 2-3 semanas |
| W2 (Vertical) | 1.5-2 semanas |
| W3 (Layers) | 2.5-3 semanas |
| W4 (Adjustment Layers) | 2.5-3 semanas |
| W5 (Brush Studio + Mixbox + Procedural) | 3-4 semanas |
| W6 (Brush Library) | 1.5-2 semanas |
| W7 (Color) | 2-2.5 semanas |
| W8 (Selection + Transform + destructive) | 2.5-3 semanas |
| W9 (Guides + QuickShape) | 1.5-2 semanas |
| W10 (Gestures + Shortcuts + Pencil Pro) | 2-2.5 semanas |
| W11 (Animation Assist) | 2-2.5 semanas |
| W12 (Reproject + Time-lapse + Replay) | 2.5-3 semanas |
| W13 (MCP Stroke Engine) | 1-1.5 semanas |
| W14 (Stroke Inspector) | 2.5-3 semanas |
| W15 (Fluid Brushes) | 4-5 semanas |
| W16 (Export interop) | 2-2.5 semanas |
| W17 (Polish + Release) | 3-4 semanas |
| **Total** | **~37-49 semanas** (~9-12 meses 1 implementador full-time) |

Com paralelização entre Implementador + Coord-A + Coord-B, **8-10 meses calendário** factível (vide DIRETRIZ v7.0 §1).

---

## §25. Anti-padrões transversais (não-repita NUNCA)

Estes 12 anti-padrões aplicam a **qualquer task** do plano. Consulte antes de cada commit.

1. **Hardcoded color hex em paint** (`Color::rgba8(0x12, 0x34, ...)`) — vide gate `no_literal_color`.
2. **`f32` literal de UI fora do allowlist** — vide gate `no_magic_numeric`.
3. **String hardcoded em UI** — use `t!()` Fluent (HR-15).
4. **Widget sem `Node` AccessKit** — gate `hr12_widgets_a11y`.
5. **Mirror manual slider↔chip** — use `store.link_slider_number` (DIRETRIZ v7.0 §5.2).
6. **Chip pill sem stepper** se fase 1.A do `2026-05-ui-source-of-truth.md` fechou — canon mudou; copie do Inspector/Gallery atualizado.
7. **`std::HashMap` em compositor / state lateral** — use `BTreeMap` (HR-5, mesmo em Present para disciplina).
8. **Alocação em hot path** (`Box::new`, `Vec::push` que realoca) — HR-3 (`painter_no_alloc_hot_path` gate).
9. **Skip `RasterEditTool::deactivate` cache zero** — stale-frame bug (Wave 10 §A2 audit).
10. **`--no-verify` em commit** — fix root cause (DIRETRIZ v7.0 §7.2).
11. **Hardcoded `id == "painter"` em múltiplos sites** — data-driven via `is_painter_tool` helper (Image Tools Bugs §2.b).
12. **Hit-test e paint do widget gateados por condições diferentes** — Image Tools Bugs §2.c.

---

## §26. Onde buscar quando dúvida

Hierarquia de fontes:

1. **Esta página (§N específica)** — o plano.
2. **Spec docs** ([01](01_brush_engine.md) - [14](14_inovacoes_extraordinarias.md)) — o que implementar.
3. **DIRETRIZ v7.0** — como implementar (triagem, isolamento, gates).
4. **SKILL_Stack** — Hard Rules + arquitetura.
5. **ADRs 0040..0049** — contratos imutáveis.
6. **Code existente** (`crates/ph2d-tool-bgremoval`, `crates/ph2d-panel-bgremoval`, `shells/desktop/src/render_loop/bgremoval_preview.rs`) — templates canônicos.
7. **Bug docs** (`docs/UI_Bugs/`, `docs/Image Tools Bugs/`) — o que NÃO repetir.
8. **Memory persistente LLM** — perfil Enio + feedback acumulado.

Quando code contradiz spec: **confie no code**. Atualize spec depois.
Quando docs contradizem entre si: **DIRETRIZ v7.0 ganha sobre SKILL_Stack** (v7.0 é mais novo).

---

**Fim do plano.** Este doc é vivo: ao fim de cada wave, marcar progresso em §22 ou em arquivo `progress.md` dedicado. Sugestões/correções via PR normal (caminho B para alterações maiores; C para alterações foundational ao plano).
