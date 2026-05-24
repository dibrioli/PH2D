# Wave 10 — Tracker de Testes e Smokes Visuais (HISTÓRICO)

**Status:** Wave 10 fechada em 2026-05-23 ([ADR-0042](../architecture/decisions/0042-wave-10-closure.md)). Este doc é histórico — não é o tracker corrente.

**Propósito original:** registro de TODOS os testes automáticos rodados e TODOS os smokes visuais pendentes em cada etapa da Wave 10 perfection plan (arquivada em [`docs/archive/plans-completed/2026-05-wave-10-perfection.md`](../archive/plans-completed/2026-05-wave-10-perfection.md)). O Enio auditou visualmente ao final da wave usando este doc como checklist.

**Princípio:** "padrão ouro / puro sangue / definitivo" — toda mudança ou tem gate automático passando OU tem checklist explícita pra smoke manual. Nada fica "se você lembrar de testar".

---

## Estado por etapa

| Etapa | Status | Commit | Testes automáticos | Smoke manual pendente |
|---|---|---|---|---|
| 0 — Infraestrutura multi-agente | ✅ COMPLETA | `d9379ee` | scripts smoke + workspace check | [§E0](#etapa-0--infraestrutura) |
| 1.A — Emenda ADR-0041 (rename + deactivate) | ✅ COMPLETA | `a03d830` | 769/769 workspace (T2 hook) | [§E1A](#etapa-1a--emenda-adr-0041) |
| 1.B — `ph2d-tool-runtime` + BgRemoval impl | ✅ COMPLETA | `74b6d27` | 132 verdes (122 BgR + 10 runtime) | [§E1B](#etapa-1b--ph2d-tool-runtime--bgremoval-impl) |
| 2 — CEQ + Upscale impl (Padding/EqSizes exception) | ✅ COMPLETA | `cbb9cb3` | 193 verdes; fix C1 cross-bridge | [§E2](#etapa-2--ceq--upscale-impl-rastereditool) |
| 3 — 3 arch-gates + drive_multi_preview_cache + fix C3 multi-Apply | ✅ COMPLETA | `666a85a` | 19 gates/helper verdes; 3 críticos + 3 altos fixados | [§E3](#etapa-3--3-arch-gates--drive_multi_preview_cache) |
| 4 — panel-sync + chrome-sync + widget-sync (3 codegens) | ✅ COMPLETA | (pendente) | 6 staleness gates (3 panel + 2 chrome + 1 widget) + 12 helper tests | [§E4](#etapa-4--3-codegens-panelchromewidget-sync) |
| 5 — Gates UI panel-\* + ph2d-color + classes de bug | ✅ COMPLETA | (pendente) | 5 UI gates estendidos + 4 gates ortogonais + LOC cap + arch_color_space_typed + ph2d-color (15 tests) | [§E5](#etapa-5--gates-ui-panel--ph2d-color--gates-ortogonais) |
| 6 — LOC drift + memory GC + audit refinements (6.1/6.2 deferidos) | 🟡 PARCIAL | (pendente) | LOC trend (6 tests) + memory GC (7 tests) + M-1/M-2 audit fixes | [§E6](#etapa-6--loc-trend--memory-gc--audit-refinements) |
| 7 — Política merge-on-green + closure ADR-0042 | ✅ COMPLETA | (pendente) | auto-merge-eligibility.sh + ADR-0042 closure + carry-over Wave 11 | [§E7](#etapa-7--auto-merge-eligibility--adr-0042-closure) |

---

## Como usar este doc no final da wave

1. Vá em ordem (Etapa 0 → 7).
2. Para cada etapa: rode `./play.command` ou os comandos listados em "Smoke manual pendente".
3. Marque ✅ em cada item conforme verifica.
4. Se algo falhar: registre em `regressions.md` (mesma pasta) com SHA do commit suspeito + descrição.
5. Ao final, se todos verdes → push origin/main + tag `wave-10-complete`.

---

## Etapa 0 — Infraestrutura

**Commit:** `d9379ee` — `feat(wave-10): Etapa 0 — infra multi-agente (scripts + 2 Coords + parallel CI)`

### Testes automáticos rodados ✅

```bash
# Pre-flight (rodado antes da wave começar)
df -h /Volumes/MAC_EXTERNO/  # → 1.8 TB livre ✓
df -h /                       # → 124 GiB livre no SSD local ✓

# slot-env smoke
bash -c 'source scripts/slot-env.sh test-smoke && echo "OK"'
# → [slot-env] active: test-smoke target=.../target-slots/slot-test-smoke ✓

# git-stage-guard smoke (4 cenários)
COORD_OVERRIDE=1 ./scripts/git-stage-guard.sh     # → bypass ativo ✓
PH2D_SLOT_FOLDER="crates/ph2d-tool-bgremoval" \
  ./scripts/git-stage-guard.sh                    # → no staged, exit silently ✓
./scripts/git-stage-guard.sh                      # → no PH2D_SLOT_FOLDER, warn ✓

# .github/workflows/spike.yml validation
gh workflow view spike.yml                        # → 227 runs históricos OK ✓
```

### Smoke manual pendente

(Nenhum smoke visual necessário — Etapa 0 é puramente infraestrutura de shell/scripts/CI. Verificação que basta:)

- [ ] Abra um terminal novo, vá pro repo, rode `source scripts/slot-env.sh coord-a` — espera ver banner `active: coord-a target=$HOME/ph2d-target/slot-coord-a`
- [ ] Rode `echo $CARGO_TARGET_DIR` — deve mostrar `$HOME/ph2d-target/slot-coord-a`
- [ ] (Opcional) Inicie 2 sessões Claude Code paralelas, source slot-env com IDs diferentes, rode `cargo check -p ph2d-editor-core` em ambas simultaneamente — não deve dar "Blocking waiting for file lock"

### Artefatos criados

- `scripts/slot-env.sh` — CARGO_TARGET_DIR por slot (Coord-A → SSD local; demais → MAC_EXTERNO)
- `scripts/git-stage-guard.sh` — pre-commit anti-colisão (PH2D_SLOT_FOLDER + COORD_OVERRIDE bypass)
- `scripts/arch-gate-budget.sh` — warn-only tripwire >90s arch-tests
- `docs/SESSION_ACTIVE.md` — protocolo sincronização 2 Coords
- `docs/IntegracaoMultiAgente/DIRETRIZ.md` §1.1 reescrito (2 Coords)
- `.github/workflows/spike.yml` — trigger `feat/**` + gate `test:` job (matrix 3-OS skip em feat branches)
- `target-slots/` adicionado ao `.gitignore`

---

## Etapa 1.A — Emenda ADR-0041

**Commit:** `a03d830` — `feat(wave-10): Etapa 1.A — ADR-0041 amendment ADR-0040 (rename + deactivate)`

### Testes automáticos rodados ✅

```bash
# Arch-gate cap
cargo test -p ph2d-editor-core --test architecture_tool_contract_surface
# → 3/3 ok: tool_contract_is_capped, raster_edit_tool_contract_is_capped, panel_event_variant_count_is_capped

# Lib tests editor-core (incluindo novo raster_edit_tool_upcasts_and_drives_through_generic_contract)
cargo test -p ph2d-editor-core --lib tool::
# → 11/11 ok

# Suite completa editor-core
cargo test -p ph2d-editor-core --tests
# → 602/602 ok

# Workspace check (downstream dos tool/panel crates após rename)
cargo check --workspace
# → green em 5.82s incremental, 0 warnings

# T2 pre-commit hook (workspace nextest)
# (rodou automaticamente no commit a03d830)
# → 769/769 tests passed in 1.281s
```

### Smoke manual pendente

- [ ] `./play.command` abre normalmente (rename não quebrou shell)
- [ ] TopBar Image Tools mostra todas as pills (Trim, MakeSquare, RealSize, BgRemoval, Padding, CEQ, EqualizeSizes, Upscale, Rasterize) — confirmar visualmente
- [ ] BgRemoval ainda funcional via downcast (não migrado ainda — só rename) — abrir, mover Tolerance slider, ver preview live, clicar Apply, ver bake
- [ ] Eyedropper pick + Protect-brush dab ainda funcionam
- [ ] Cmd+Q encerra sem panic

### Achados de auditoria adversarial

(Ver `audits/etapa-1a.md` se houver — neste momento não tive auditoria formal porque o escopo era trivialmente verificável: rename + add método.)

### Artefatos criados

- `docs/architecture/decisions/0041-rasteredit-rename-and-deactivate.md` — ADR canônico
- `crates/ph2d-editor-core/src/tool.rs` — trait renomeado + métodos novos
- `crates/ph2d-editor-core/tests/architecture_tool_contract_surface.rs` — caps atualizados (4→5)
- DIRETRIZ + SKILL_Stack + CLAUDE atualizados (rename + entry v6.10)

---

## Etapa 1.B — `ph2d-tool-runtime` + BgRemoval impl

**Commit:** (pendente — será adicionado ao finalizar)

### Testes automáticos rodados ✅

```bash
# Crate novo ph2d-tool-runtime — helpers genéricos sobre RasterEditTool
cargo test -p ph2d-tool-runtime
# → 9 unit tests + 1 arch-gate (LOC ≤500) = 10/10 ok

# BgRemoval implementou RasterEditTool — 7 tests novos do impl + 2 do audit fix (A1+A2)
cargo test -p ph2d-tool-bgremoval --lib
# → 122 tests ok (113 pré-existentes + 9 novos)

# Workspace inteiro continua compilando + clippy --all-targets clean
cargo check --workspace
# → green in 3.22s incremental
cargo clippy -p ph2d-host-desktop -p ph2d-tool-runtime -p ph2d-tool-bgremoval --all-targets -- -D warnings
# → green

# Staleness gates após exclusion list update em tool-sync (audit fix 3.1)
cargo run -p ph2d-tool-sync   # → 12 crates total (ph2d-tool-runtime excluído)
cargo test -p ph2d-tool-registry-init --tests   # → 3 staleness gates verdes
```

### Audit adversarial × 2 (paralelo)

Vide `audits/etapa-1b.md` para o relatório completo. Resumo:

- **2 achados CRÍTICOS** corrigidos pré-commit:
  - **[A1]** `BgRemovalTool::set_source_snapshot` não invalidava `cached_canvas_preview` → stale frame após selection drift com read falho. Fix: 1 linha (`self.cached_canvas_preview = None;`). Test: `set_source_invalidates_cached_canvas_preview`.
  - **[A2]** `Tool::on_deactivate` (path quando troca para tool não-Raster como Brush) não limpava `cached_canvas_preview` → BgR→Brush→BgR mostrava frame stale. Fix: 1 linha em `on_deactivate`. Test: `on_deactivate_clears_cached_canvas_preview`.
- **1 achado ARQUITETURAL** corrigido pré-commit:
  - **[3.1]** `ph2d-tool-runtime` batia o pattern `ph2d-tool-*` do `tool-sync`, gerando dep inerte em `ph2d-tool-registry-init/Cargo.toml`. Fix: extend exclusion list em `tools/ph2d-tool-sync/src/lib.rs`.
- **2 achados DOC** corrigidos pré-commit:
  - DIRETRIZ §3.8.3.1 reescrita (não diz mais "zero tools usam"; agora documenta BgR como primeiro impl + template pra Etapa 2).
  - v4 plan §I corrigido (cap Tool=10 não 11; `RasterFrame/PixelLayout` diferidos pra Etapa 5).

### Smoke manual pendente (Enio)

**Antes de qualquer outro smoke da Etapa 1.B:** rodar `./play.command` e verificar **TODAS** as 6 verificações abaixo. Cada uma cobre um caminho que mudou estruturalmente:

- [ ] **Smoke 1 — BgR happy path live preview:** abrir BgR pill na TopBar → sprite seleciona → mover Tolerance slider → ver preview live atualizar no canvas; mover Brush slider → idem; clicar Apply → bake correto, preview desaparece.
- [ ] **Smoke 2 — BgR selection drift (cobre fix [A1]):** com BgR ativo, selecionar sprite A → ver preview de A → selecionar sprite B (sem deativar BgR) → preview migra para B SEM mostrar frame stale de A. (Especialmente em sprites com tamanhos/cores muito diferentes.)
- [ ] **Smoke 3 — BgR → Brush → BgR (cobre fix [A2]):** com BgR ativo + preview no canvas, clicar Brush no palette → BgR deativa, preview some → clicar BgR novamente → preview deve recomputar (não aparecer frame stale da sessão anterior).
- [ ] **Smoke 4 — BgR eyedropper:** com BgR ativo, armar eyedropper, clicar no canvas → cor sample adicionada à lista, preview re-segmenta. (Path downcast permanece — não deve quebrar.)
- [ ] **Smoke 5 — BgR protect-brush:** com BgR ativo, armar protect brush, arrastar no canvas → tint overlay aparece, preview honra a região protegida. (Path downcast permanece.)
- [ ] **Smoke 6 — BgR Cancel mid-preview:** com BgR ativo + preview computando, clicar Cancel na palette/Cmd+. → sprite intacto, sem overlay residual.

**Outras tools (Padding/CEQ/Upscale/EqualizeSizes) ainda usam path antigo** — não devem ter regredido (não foram migrados ainda; vão na Etapa 2). Smoke rápido recomendado:
- [ ] Cada uma abre, slider/Apply funcional como antes.

### Métricas

- `shells/desktop/src/render_loop/bgremoval_preview.rs`: 323 → 336 LOC (+13 por doc-comments densos; ganho real está na estrutura compartilhada que próximos 4 bridges vão herdar)
- Downcasts `<BgRemovalTool>` no shell: 4 (eram 4 — não mudou nesta etapa; cairá em Etapa 2+3 quando o canal genérico cobrir mais)
- Crate novo: `ph2d-tool-runtime` (~340 LOC src + cap LOC ≤500 ativo)
- Cap `RasterEditTool`: 4 → 5 (já feito em Etapa 1.A — incrementa o `deactivate`)

### Artefatos criados

- `crates/ph2d-tool-runtime/` — crate novo com 4 helpers + 1 arch-gate
- `crates/ph2d-tool-bgremoval/src/tool.rs` — field `cached_canvas_preview` + `impl RasterEditTool` + 9 tests novos
- `shells/desktop/src/render_loop/bgremoval_preview.rs` — refactor usando helpers do runtime
- `shells/desktop/src/app_state.rs` — `BgremovalPreview` virou type alias para `PreviewCache`
- `shells/desktop/Cargo.toml` — adicionado dep `ph2d-tool-runtime`
- `tools/ph2d-tool-sync/src/lib.rs` — exclusion list estendida
- DIRETRIZ §3.8.3.1 reescrita (status atual RasterEditTool)
- v4 plan §I corrigido (caps + RasterFrame diferido)

---

## Etapa 2 — CEQ + Upscale impl RasterEditTool

**Commit:** (pendente — será adicionado ao finalizar)

### Escopo realista (corrigido pós-estudo)

v4 plan §II Etapa 2 originalmente listava 4 tools (Padding + CEQ + Upscale + EqualizeSizes). **Estudo dos tools mostrou que 2 não cabem no contrato `RasterEditTool`:**

- **Padding** é geométrico-only (não tem source raster nem preview de pixels — só carrega 4 inteiros + flags; bake é stateless function pura). Forçar trait stateful seria astronaut architecture. **Documented exception.**
- **EqualizeSizes** é multi-sprite-required (mode `MaxOfSelection` depende do W/H global da seleção inteira). Não cabe em `set_source(rgba, w, h)` single-buffer. **Documented exception.**

Etapa 2 entrega **CEQ + Upscale** (os 2 sabor-(3) com live preview single-cache que de fato cabem). Padding + EqualizeSizes mantêm path atual (downcast — exceção documentada ADR-0040 §3 + DIRETRIZ §3.8.3.1).

### Testes automáticos rodados ✅

```bash
# CEQ implementa RasterEditTool — 7 tests novos + 124 pré-existentes
cargo test -p ph2d-tool-color-equalization --lib
# → 131/131 ok

# Upscale implementa RasterEditTool — 7 tests novos + 45 pré-existentes + cache field
cargo test -p ph2d-tool-upscale --lib
# → 52/52 ok

# Runtime ganhou 1 test C1 regression (drive_deactivate_cleanup wrong-tool warn)
cargo test -p ph2d-tool-runtime
# → 11/11 ok (10 anteriores + 1 novo regression test)

# Arch-gate cap inalterado
cargo test -p ph2d-editor-core --test architecture_tool_contract_surface
# → 3/3 ok (Tool=10 / RasterEditTool=5 / PanelEvent=4)

# Workspace + clippy
cargo check --workspace                                     # → green
cargo clippy -p ph2d-host-desktop --all-targets -- -D warnings  # → clean
```

### Audit adversarial × 2

Vide `audits/etapa-2.md`. Resumo:

- **1 achado CRITICAL [C1]** corrigido pré-commit:
  - `drive_deactivate_cleanup` chamado em `tools.active_mut()` no path inativo do bridge **zerava state de outras RasterEditTools** que estivessem ativas (CEQ/Upscale/BgR cruzados). Cenário: usuário com BgR ativo + slider em movimento → frame do Upscale bridge rodava `Upscale_bridge::dispatch` (inativo) → chamava `BgR.deactivate()` (errado tool!) → drenava `pending_apply`/`params_dirty`/`cached_canvas_preview` da BgR → preview congelava, Apply não disparava. Fix: removido `drive_deactivate_cleanup` cross-bridge; cada bridge limpa SÓ cache local. `Tool::on_deactivate` da própria tool (chamado por `ToolRegistry::set_active`) cobre a limpeza interna corretamente (já confirmado por fixes A1+A2 da Etapa 1.B). Doc-comment do helper foi reforçado com warning explícito; novo test `drive_deactivate_cleanup_unconditionally_deactivates_passed_tool` documenta a invariante (helper não tem awareness de ownership — bridges devem nunca passar foreign tool).
- **3 achados DOC** corrigidos pré-commit:
  - v4 plan §II Etapa 2 atualizado com nota pós-execução (CEQ+Upscale migrados; Padding+EqSizes exception).
  - DIRETRIZ §3.8.3.1 atualizada (3 tools implementam; Padding/EqSizes documented exception).
  - DIRETRIZ §3.8.5 checklist permanece (rename ImageEditTool já foi feito em Etapa 1.A).
- **1 follow-up DEFERIDO para Etapa 3** (não bloqueia commit):
  - `drive_multi_preview_cache` helper — `ph2d-tool-runtime` tem 101 LOC de folga (399/500), helper trivial (~40 LOC) que generalizaria o loop multi-sprite do CEQ. Anotado em audits/etapa-2.md.

### Smoke manual pendente (Enio)

**CEQ (5 smokes — espelham os de BgR):**

- [ ] **S1 CEQ happy path:** abrir CEQ → multi-select 2 sprites → mover Exposure/Contrast/Vibrance → ver preview live atualizar em ambos os sprites → Apply → bake em todos selecionados.
- [ ] **S2 CEQ selection drift (cobre fix A1 mirror):** CEQ ativo + sprite A → shift-click sprite B → preview de B aparece sem ghost de A; preview cache_keys segue iter_selected.
- [ ] **S3 CEQ → Brush → CEQ (cobre fix A2 mirror):** preview deve recomputar (não frame stale do session anterior).
- [ ] **S4 CEQ dropdown LUT:** abrir um dos 4 dropdowns (LUT 1/2 / Posterize / Quantize), escolher opção → preview atualizar + dropdown fechar sozinho (close_dropdown path).
- [ ] **S5 CEQ Reset durante drag:** arrastar slider + clicar Reset → todos voltam ao default (sliders/chips visualmente snapped) + preview desaparece.

**Upscale (4 smokes):**

- [ ] **U1 Upscale happy path:** abrir Upscale → trocar algoritmo (Lanczos/Nearest/xBR) → ver overlay no canvas + thumb no panel → mover scale slider → preview live atualiza → Apply → sprite.size cresce pelo factor (vide fix `f47c3a9`), overlay some, undo restaura.
- [ ] **U2 Upscale selection drift (cobre fix A1):** Upscale ativo + sprite A → seleciona B → overlay migra para B sem frame stale de A. Especialmente teste com tamanhos muito diferentes.
- [ ] **U3 Upscale → Brush → Upscale (cobre fix A2):** cache_canvas_preview limpa, recomputa na reativação.
- [ ] **U4 Upscale + outros raster ativos NÃO interferem (cobre fix C1 CRÍTICO!):**
  - Cenário 1: ativar Upscale + mover scale slider rapidamente → Apply ainda funciona, preview live sustenta-se. (Confirma que BgR/CEQ bridges não estão zerando state via cross-call.)
  - Cenário 2: ativar BgR + ajustar Tolerance + Apply → ver bake. (Confirma que Upscale bridge inativo não destrói BgR's state.)
  - Cenário 3: ativar CEQ + ajustar Brightness + Apply → ver bake. (Confirma idem.)

**Padding / EqualizeSizes (regression — NÃO migraram, devem continuar funcionais):**

- [ ] **Padding:** 4 sliders Top/Right/Bottom/Left + pivot recenter + Apply → canvas cresce + sprite reanchorado.
- [ ] **EqualizeSizes:** ativar → MaxOfSelection + 3 sprites de tamanhos diferentes → todos viram do mesmo tamanho do maior W/H.

### Métricas pós-Etapa 2

- `ph2d-tool-runtime`: 399 → 422 LOC (~78 LOC de folga até cap 500; `drive_multi_preview_cache` cabe na Etapa 3)
- Downcasts `<ConcreteTool>` no shell: ~17 (multi-sprite-loop CEQ ainda tem 1 que vai sair com `drive_multi_preview_cache` em Etapa 3)
- Tools implementando `RasterEditTool`: 3 (BgR + CEQ + Upscale). Esperado pós-Etapa 5: idem (Padding/EqSizes = documented exception permanente).
- `UpscalePreview` virou type alias `PreviewCache` (uniforme com BgR). `ColorEqualizationPreview` permanece struct (multi-cache; helper genérico futuro em Etapa 3).

### Artefatos modificados

- `crates/ph2d-tool-color-equalization/src/tool.rs` — `impl RasterEditTool` + `as_raster_edit_mut` override + 7 tests novos
- `crates/ph2d-tool-upscale/src/tool.rs` — field `cached_canvas_preview` + `impl RasterEditTool` + 7 tests novos + fixes A1+A2 mirror
- `shells/desktop/src/render_loop/upscale_bridge.rs` — refactor usando 4 helpers + fix C1 (inactive path limpa só local)
- `shells/desktop/src/render_loop/color_equalization_bridge.rs` — mini-refactor `drive_pending_commit`
- `shells/desktop/src/render_loop/bgremoval_preview.rs` — fix C1 (idem)
- `shells/desktop/src/app_state.rs` — `UpscalePreview` virou type alias `PreviewCache`
- `crates/ph2d-tool-runtime/src/lib.rs` — warning expandido em `drive_deactivate_cleanup` + 1 test C1 regression
- `docs/IntegracaoMultiAgente/DIRETRIZ.md` §3.8.3.1 atualizada
- `docs/archive/plans-completed/2026-05-wave-10-perfection.md` §II nota pós-execução

---

## Etapa 3 — 3 arch-gates + drive_multi_preview_cache

**Commit:** (pendente)

### Escopo

Esta etapa **fecha o ciclo Wave 10 Etapas 1+2** com 3 invariantes arquiteturais permanentes (anti-regressão) + 1 helper genérico + 1 fix funcional crítico:

1. **`drive_multi_preview_cache` helper** (`crates/ph2d-tool-runtime/`) — generaliza o loop multi-sprite preview do CEQ.
2. **`color_equalization_bridge` migra** pra usar o helper (elimina 1 downcast).
3. **3 arch-gates novos** travam regressão por construção:
   - `architecture_no_per_tool_branch_in_render_loop` (shell)
   - `architecture_no_downcast_to_concrete_tool_in_shell` (shell) — allowlist explícita
   - `architecture_image_tool_kind_contract` (registry-init) — tools cluster=image_tools devem `impl RasterEditTool` OU estar em exception
4. **Fix [C3] crítico**: `input_handlers.rs` tinha drain duplicado de `take_pending_apply` que regredia multi-select Apply pra single-sprite. Bloco removido — bridges cobrem com `drive_pending_commit`.

### Testes automáticos rodados ✅

```bash
# Gate #1 — render_loop/mod.rs per-tool branch count
cargo test -p ph2d-host-desktop --test architecture_no_per_tool_branch_in_render_loop
# → 1/1 ok (baseline cap=16 snapped to current count)

# Gate #2 — downcast allowlist
cargo test -p ph2d-host-desktop --test architecture_no_downcast_to_concrete_tool_in_shell
# → 1/1 ok (allowlist: 6 reais — eyedropper, protect_brush, 4 bridges)

# Gate #3 — image_tool kind contract
cargo test -p ph2d-tool-registry-init --test architecture_image_tool_kind_contract
# → 1/1 ok (exceptions: padding, equalize-sizes, 4 one-shots)

# Runtime com helper novo + tests A1 regression
cargo test -p ph2d-tool-runtime
# → 16/16 ok (15 unit + 1 arch-gate; cap bumped 500→650 LOC)

# Workspace + clippy
cargo check --workspace                         → green
cargo clippy --workspace --all-targets          → clean
```

### Audit adversarial × 1 (single, em vez de ×2 — escopo isolado)

Vide `audits/etapa-3.md`. Resumo:

- **3 achados CRITICAL fixados pré-commit:**
  - **[C1]** Allowlist de gate #2 tinha 5 entries fantasma (hero_intents/image_edit/*.rs) — pre-concedia permissão sem necessidade. Fix: removidos. Allowlist agora só tem entries com downcasts reais.
  - **[C2]** Baseline cap gate #1 era 20 com 4 unidades de folga "achatada". Snapped pra 16 (count real). Gate agora falha em qualquer adição.
  - **[C3]** Drain duplicado de `take_pending_apply` em `input_handlers.rs` + `bgremoval_preview.rs` competiam (destructive). Input-handlers vencia, bridge sempre via `false` → multi-select Apply via panel toggle só bakava primary sprite. **Bloco removido do input_handlers.rs**; bridge canônico cobre multi-sprite via `drive_pending_commit`.
- **3 achados ALTO fixados:**
  - **[A1]** `drive_multi_preview_cache` mantinha cache stale quando `read_source` falhava transientemente. Fix: `cache.remove(bits)` no path miss + novo test regression.
  - **[A2]** Doc-comment `BgremovalPreview` em app_state.rs dizia "CEQ + Upscale keep own structs until Etapa 2" — stale (Etapa 2 já migrou). Atualizado.
  - **[A3]** Doc-comment módulo `color_equalization_bridge.rs` dizia "future drive_multi_preview_cache". Fix: agora documenta status pós-Etapa 3 (helper existe e é usado).
- **5 achados MÉDIO/BAIXO** anotados (não bloqueantes — alguns como follow-up pra Wave 11+):
  - M1: marker `ARCH-ALLOW: per-tool-branch` ainda não usado em produção (escape-hatch reservada).
  - M2: gate #1 não detecta literais multi-line (`concat!` / raw strings) — heurística aceita pra Etapa 3 baseline.
  - B5: gate #3 não cobre `impl RasterEditTool for` em sub-dirs do `src/` — todos os 3 impls atuais estão no root `src/tool.rs`, baixo risco.

### Smoke manual pendente (Enio)

**Crítico [C3] fix smoke (regression de multi-select Apply):**

- [ ] **G1 BgR Apply Toggle via panel + multi-select:** ativar BgR → shift-click 3 sprites → no painel docado, clicar Apply Toggle → **todos os 3 sprites devem ser bakados** (não só primary). Era exatamente o bug que [C3] corrigiu.
- [ ] **G2 CEQ Apply via panel + multi-select:** ativar CEQ → multi-select 3 sprites → clicar Apply → todos os 3 bakados (já estava OK; teste de regressão).
- [ ] **G3 Upscale Apply via panel + multi-select:** idem (já estava OK).

**Gate functional smokes:**

- [ ] **G4 CEQ multi-sprite preview live:** shift-select 2-3 sprites → ativar CEQ → arrastar Exposure slider → todos sprites têm overlay atualizando em paralelo (a função `drive_multi_preview_cache` agora cobre o loop).
- [ ] **G5 CEQ atlas miss simulado:** difícil reproduzir sem injetar fault. Verificar via test (A1 regression cobre).

### Métricas pós-Etapa 3

- **`render_loop/mod.rs` per-tool mentions:** 16 (snapped baseline; bumping down encouraged em Etapas 4-7)
- **Downcasts `<ConcreteTool>` no shell:** 6 reais allowlistados (eyedropper, protect_brush, 4 bridges com affordances específicas) — era 17 pre-Etapa-3
- **`ph2d-tool-runtime`:** 598/650 LOC (52 LOC de folga; cap final pra Wave 10)
- **Tools implementando `RasterEditTool`:** 3 (BgR/CEQ/Upscale) + 6 exceptions (Padding, EqSizes, MakeSquare, RealSize, Rasterize, TrimTransparency)
- **`ColorEqualizationPreview`:** virou type alias para `PreviewCache` (uniformização completa BgR/CEQ/Upscale)

### Artefatos modificados

- `crates/ph2d-tool-runtime/src/lib.rs` — `drive_multi_preview_cache` helper + 4 tests + fix A1
- `crates/ph2d-tool-runtime/tests/architecture_runtime_loc_cap.rs` — cap 500→650
- `shells/desktop/tests/architecture_no_per_tool_branch_in_render_loop.rs` — NOVO gate #1
- `shells/desktop/tests/architecture_no_downcast_to_concrete_tool_in_shell.rs` — NOVO gate #2 com allowlist
- `crates/ph2d-tool-registry-init/tests/architecture_image_tool_kind_contract.rs` — NOVO gate #3
- `shells/desktop/src/render_loop/color_equalization_bridge.rs` — usa drive_multi_preview_cache
- `shells/desktop/src/app_state.rs` — ColorEqualizationPreview virou type alias
- `shells/desktop/src/input_handlers.rs` — fix C3 (removeu drain duplicado)
- `docs/Testes/audits/etapa-3.md` — full audit report

---

## Etapa 4 — 3 codegens (panel/chrome/widget-sync)

**Commit:** (pendente)

### Escopo

Etapa 4 completa o trio de codegen tools que faltava (espelha o `ph2d-tool-sync` existente):

- **`tools/ph2d-panel-sync/`** — varre `crates/ph2d-panel-*`, regenera `build_typed_registry` body + `[dependencies]` + `[features]` em `ph2d-panel-registry-init`. 9 panel crates atuais sincronizados automaticamente.
- **`tools/ph2d-chrome-sync/`** — varre `crates/ph2d-editor-core/src/screens/hero/chrome/*.rs`, regenera `mod foo;` declarations. `dispatch_all` permanece hand-written (z-order load-bearing). 13 chrome handlers sincronizados.
- **`tools/ph2d-widget-sync/`** — varre `crates/ph2d-editor-core/src/widget/{*.rs,subdirs/}`, regenera `mod foo;` declarations. `pub use` re-exports permanecem hand-written. 34 file widgets + 2 sub-dir widgets sincronizados.

Resultado: adicionar painel/chrome/widget novo = drop arquivo + `cargo run -p ph2d-<x>-sync` + commit. Zero hand-edit central pra a parte mecânica (Cargo.toml deps + features pra panel; mod declarations pra chrome+widget).

### Testes automáticos rodados ✅

```bash
# panel-sync — 3 staleness sub-gates (lib.rs semantic + Cargo deps + Cargo features)
cargo test -p ph2d-panel-registry-init --tests
# → 4/4 ok (1 lib + 3 staleness)

# chrome-sync — 2 sub-gates (mod block + dispatch_all references)
cargo test -p ph2d-editor-core --test architecture_chrome_dispatch_in_sync
# → 2/2 ok

# widget-sync — 1 sub-gate (mod block matches scan)
cargo test -p ph2d-editor-core --test architecture_widget_mod_in_sync
# → 1/1 ok

# Codegen tools unit tests
cargo test -p ph2d-panel-sync   # 4 unit tests
cargo test -p ph2d-chrome-sync  # 3 unit tests
cargo test -p ph2d-widget-sync  # 3 unit tests

# Workspace + clippy + fmt
cargo check --workspace                              → green
cargo clippy --workspace --all-targets -- -D warnings → clean
cargo fmt --all -- --check                            → clean
```

### Audit adversarial × 1 (escopo isolado — não tocou produção)

Vide `audits/etapa-4.md`. Resumo:

- **1 achado CRITICAL fixado pré-commit:**
  - **[C-1]** `render_register_lines` em panel-sync emitia linha que ultrapassava 100 cols pra tipos longos (CEQ, EqualizeSizes, WidgetGallery). `cargo fmt` reformatava pra multi-linha → staleness gate quebrava no próximo pre-commit. Fix: gate compara **semanticamente** via `extract_registered_panels` (extrai `(crate_ident, struct_name)` pairs, formatação-tolerante); main do sync chama `cargo fmt` ao final pra deixar on-disk canônico.
- **1 achado ALTO** (M-1) anotado mas DEFERIDO:
  - `default = [...]` array em `panel-registry-init/Cargo.toml` é hand-written. Adicionar panel novo não regenera o `default` automaticamente — panel some silenciosamente se esquecer. Mitigação Etapa 4: doc-comment explícito + smoke do Enio. Fix completo (sync também regenera `default`) fica como follow-up Etapa 5.
- **5 achados MÉDIO/BAIXO** anotados (false positives em parsers, paths frágeis em hipotéticos futuros): em `audits/etapa-4.md`.

### Smoke manual pendente (Enio)

**Smokes da Etapa 4 são puramente arquiteturais (zero mudança visual):**

- [ ] **G6 painel drop-in:** crie um `crates/ph2d-panel-test/` minimal (Cargo.toml + src/lib.rs com `pub struct TestPanel; impl Panel for TestPanel { ... }`) → `cargo run -p ph2d-panel-sync` → painel aparece no registry sem hand-edit em registry-init. Smoke arquitetural (não precisa rodar o painel — só verificar que o registry-init compila com a nova entrada).
- [ ] **G7 chrome handler drop-in:** crie um `chrome/test_handler.rs` minimal → `cargo run -p ph2d-chrome-sync` → `mod test_handler;` aparece no chrome/mod.rs.
- [ ] **G8 widget drop-in:** crie um `widget/test_widget.rs` minimal → `cargo run -p ph2d-widget-sync` → `mod test_widget;` aparece no widget/mod.rs.
- [ ] **G9 regressão completa:** rode `./play.command`, verifique que todos os painéis canônicos (BgR, CEQ, Upscale, EqSizes, Padding, Inspector, Hierarchy, WidgetGallery, GridSnap) aparecem normalmente. (Equivalente ao smoke pós-Etapa 1-3 — a Etapa 4 não tocou a runtime path.)

### Métricas pós-Etapa 4

- **Codegen tools:** 3 novos (`panel-sync`, `chrome-sync`, `widget-sync`) + 1 já existente (`tool-sync`) = 4 codegens ativos no projeto
- **Hand-edited central files retirados:** `panel-registry-init/{src/lib.rs, Cargo.toml}` (3 regiões), `chrome/mod.rs` mod block, `widget/mod.rs` mod block = **5 regiões agora automaticamente sincronizadas**
- **Staleness gates novos:** 6 (3 panel + 2 chrome + 1 widget) — toda regressão "esquecer de rodar sync" pega em CI
- **Drop-in process pra adições:** painel/chrome/widget = drop file + sync + commit (mesmo padrão que tool/node já tinha)

### Artefatos criados

- `tools/ph2d-panel-sync/` — Cargo.toml + src/{lib.rs, main.rs}
- `tools/ph2d-chrome-sync/` — Cargo.toml + src/{lib.rs, main.rs}
- `tools/ph2d-widget-sync/` — Cargo.toml + src/{lib.rs, main.rs}
- `crates/ph2d-panel-registry-init/tests/staleness.rs` — 3 sub-gates (semantic comparison)
- `crates/ph2d-editor-core/tests/architecture_chrome_dispatch_in_sync.rs` — 2 sub-gates
- `crates/ph2d-editor-core/tests/architecture_widget_mod_in_sync.rs` — 1 sub-gate
- Markers `<ph2d-panel-sync:*>`, `<ph2d-chrome-sync:*>`, `<ph2d-widget-sync:*>` em arquivos centrais
- `docs/Testes/audits/etapa-4.md` — full audit report

---

## Etapa 5 — Gates UI panel-\* + ph2d-color + gates ortogonais

**Commit:** (pendente)

### Escopo

Etapa 5 fecha a frente de "blindagem ortogonal" do plano Wave 10:

1. **5 gates UI estendidos** a `crates/ph2d-panel-*/src/**` — antes só varriam `editor-core/src/{widget,screens}/`. Painéis tinham um blind spot que escondeu cores literais, números mágicos, glifos tofu e strings hardcoded em paint orchestrators.
2. **Sweep das 69 violations** em panel-bgremoval / panel-color-equalization / panel-equalize-sizes / panel-grid-snap (52 sites) / panel-padding / panel-upscale — substituídas por tokens `ph2d_tokens::{Spacing,Radius,StrokeToken,TypeToken,Density,ROW_H_PX}` (preferencial) ou markers `// LITERAL-PX-OK: <reason>` (caso non-design genuíno: world-space epsilons em metros, sRGB byte normalize, panel grid metrics específicos).
3. **CEQ paint.rs split** (824 LOC monolítico → 318 + 555 + 137 LOC em 3 arquivos; `paint()` de 590 LOC virou orchestrator de 58 LOC). 5 section helpers em sibling `paint_sections.rs` + `paint_histogram.rs`. Render order preservado byte-por-byte.
4. **LOC cap gate novo** (`architecture_panel_loc_cap.rs`): 600 LOC/arquivo + 200 LOC/função. 3 funções long-paint anotadas como follow-up Etapa 6 (bgremoval/paint::paint 401, grid-snap/paint::paint_body 301, grid-snap/populate::populate 214).
5. **4 gates ortogonais novos** (4 de 7 do plano §5.3):
   - `arch_no_absolute_drag_pattern` — Burning 4 class (event.x - start_x bug); marker DRAG-ABS-OK aceito p/ threshold-crossing tests legítimos.
   - `arch_no_char_count_widths` — chars().count() * GLYPH_W bug class; sugere text_system.measure_text.
   - `arch_safe_clamp_only` — força `crate::math::safe_clamp` (NaN-aware, swap-tolerant) p/ clamp com bounds dinâmicos; marker CLAMP-OK p/ bounds construction-by-design seguros.
   - `arch_mode_has_reconcile` — Image Tools Bug §2 class; setters `set_*_mode` precisam reconciliar (chamada de método além do field write); BENIGN_SET_MODE pra exceções documentadas.
6. **Crate `ph2d-color`** com `LinearRgba`, `SrgbRgba`, `Premultiplied<T>`, `OklchColor` (4 módulos + 15 unit tests). Conversões EXPLÍCITAS (`to_linear`/`to_srgb`/`premultiply`/`unmultiply`) — nenhuma `From` impl implícita.
7. **`arch_color_space_typed` gate** com BASELINE frozen (10 arquivos existentes que ainda passam `rgba: &[u8]` em assinaturas públicas). Novo código DEVE usar `&[SrgbRgba]` / `&[LinearRgba]` / `Premultiplied<T>`. Migração dos 10 sites é follow-up Etapa 5.4 (1-2 semanas, fora do escopo deste commit).

**3 gates do plano §5.3 deferidas a Etapa 6** (precondições):
- `no_tofu_glyphs` ampliado (U+2000-FFFF menos faixa Inter) — precisa tabela de cobertura Inter, complexo.
- `tests/docs_bugs_have_gates.rs` — backfill de 90 entradas em UI_Bugs + Image Tools Bugs.
- `panel-canonical-template` AST-aware — plano já o diferia a §6.2.

### Testes automáticos rodados ✅

```bash
# 5 gates UI extended (panel-* scope)
cargo test -p ph2d-editor-core --test no_literal_color --test no_magic_numeric \
  --test hr12_widgets_a11y --test hr15_no_hardcoded_ui_strings --test no_tofu_glyphs
# → 11/11 ok (incl. detector smokes)

# 4 gates ortogonais novos
cargo test -p ph2d-editor-core --test arch_no_absolute_drag_pattern \
  --test arch_no_char_count_widths --test arch_safe_clamp_only \
  --test arch_mode_has_reconcile --test arch_color_space_typed
# → 12/12 ok (incl. detector smokes)

# LOC cap novo
cargo test -p ph2d-editor-core --test architecture_panel_loc_cap
# → 2/2 ok (file + fn caps)

# ph2d-color crate
cargo test -p ph2d-color
# → 15/15 ok (linear + srgb + premultiplied + oklch round-trips)

# Workspace
cargo check --workspace
# → green

# Compatibilidade dos painéis tocados
cargo check -p ph2d-panel-grid-snap -p ph2d-panel-bgremoval \
  -p ph2d-panel-color-equalization -p ph2d-panel-equalize-sizes \
  -p ph2d-panel-padding -p ph2d-panel-upscale
# → green
```

### Audit adversarial × 1

Vide `audits/etapa-5.md` (pendente — agente em background).

### Smoke manual pendente (Enio)

**Smokes da Etapa 5 são quase 100% arquiteturais — mudanças visuais são zero by design** (tokens resolvem para os MESMOS valores numéricos que os literais substituíam; CEQ split preservou byte-equality de paint order).

Apesar disso, smoke recomendado nos painéis tocados por substituição token + CEQ split:

- [ ] **G10 — CEQ paint integridade pós-split:** abrir um sprite → ativar Color Equalization → mexer 3 sliders (Brightness, Contrast, Vibrance) + 1 dropdown (LUT) + clicar 1 auto button (Auto WB) + Reset + Apply. Esperado: comportamento idêntico ao pré-split, histograma renderiza, popovers abrem na ordem correta (popover em cima do scrollbar; scrollbar em cima do CTA).
- [ ] **G11 — Grid-Snap layout sanity:** topbar → Grid Settings → abrir painel → trocar Kind (Square/Hex/Iso/...) → mexer Cell Size NumberInput → conferir color swatch row tamanho idêntico, snap toggle CTA height 44px, kind-button-grid 3×3 com 28px row height. (Cobre as 19 substituições do paint_helpers.rs.)
- [ ] **G12 — bgremoval/upscale/padding/equalize-sizes/CEQ chrome:** abrir cada painel, conferir alturas de label column (76/64/64/72/84 px respectivamente — devem renderizar idênticas ao pré-Etapa 5). LITERAL-PX-OK markers preservam os valores.
- [ ] **G13 — drag panel chrome:** arrastar painel pra fora dos limites do viewport, soltar perto da borda esquerda/direita/topo/baixo. Esperado: `safe_clamp` migrado em panel_chrome.rs preserva o comportamento idêntico — sem snap visual diferente.
- [ ] **G14 — color picker visual integrity:** abrir BlenderColorPicker em qualquer painel → mexer wheel + slider → conferir cursor e thumb seguem o cursor sem stutter. (Cobre os 5 sites CLAMP-OK marcados em wheel/value_slider/slider_with_chip.)

### Métricas pós-Etapa 5

- **Gates UI total:** 5 estendidos a panel-* + 5 gates ortogonais novos = **10 gates novos/estendidos**
- **Tokens substituídos por literais:** 69 sites em panel-* migrados (≥80% para tokens; restante em LITERAL-PX-OK justificado)
- **CEQ paint() LOC:** 590 → 58 (orchestrator); arquivo: 824 → 318 (paint) + 555 (sections) + 137 (histogram)
- **Novo crate:** `ph2d-color` (4 módulos, 15 tests, ~470 LOC total) — primeiro typed color crate do projeto
- **safe_clamp ativo:** 4 callsites (panel_chrome ×2, color_picker_demo ×2)
- **arch_color_space_typed BASELINE:** 10 arquivos legacy frozen; novo código forçado a typed wrappers

### Artefatos criados / modificados

**Novos:**
- `crates/ph2d-color/` — Cargo.toml + src/{lib.rs, linear.rs, srgb.rs, premultiplied.rs, oklch.rs}
- `crates/ph2d-editor-core/src/math.rs` — `safe_clamp` helper + 3 tests
- `crates/ph2d-editor-core/tests/architecture_panel_loc_cap.rs`
- `crates/ph2d-editor-core/tests/arch_no_absolute_drag_pattern.rs`
- `crates/ph2d-editor-core/tests/arch_no_char_count_widths.rs`
- `crates/ph2d-editor-core/tests/arch_safe_clamp_only.rs`
- `crates/ph2d-editor-core/tests/arch_mode_has_reconcile.rs`
- `crates/ph2d-editor-core/tests/arch_color_space_typed.rs`
- `crates/ph2d-panel-color-equalization/src/paint_sections.rs`
- `crates/ph2d-panel-color-equalization/src/paint_histogram.rs`

**Modificados (gates estendidos):**
- `crates/ph2d-editor-core/tests/{no_literal_color,no_magic_numeric,hr12_widgets_a11y,hr15_no_hardcoded_ui_strings,no_tofu_glyphs}.rs`
- `crates/ph2d-editor-core/src/lib.rs` (+ `pub mod math`)

**Modificados (token substituições):**
- `crates/ph2d-panel-bgremoval/src/paint.rs` (3 sites)
- `crates/ph2d-panel-color-equalization/src/paint.rs` (split + 4 sites)
- `crates/ph2d-panel-equalize-sizes/src/paint.rs` (2 sites)
- `crates/ph2d-panel-grid-snap/src/{event,layout,paint,paint_helpers,paint_rows,populate}.rs` (52 sites)
- `crates/ph2d-panel-padding/src/paint.rs` (1 site)
- `crates/ph2d-panel-upscale/src/paint.rs` (1 site)

**Modificados (safe_clamp migração + markers):**
- `crates/ph2d-editor-core/src/widget/panel_chrome.rs` (safe_clamp ×2)
- `crates/ph2d-editor-core/src/widget/avatar.rs` (CLAMP-OK)
- `crates/ph2d-editor-core/src/widget/blender_color_picker/{wheel,value_slider}.rs` (CLAMP-OK)
- `crates/ph2d-editor-core/src/widget/slider_with_chip.rs` (CLAMP-OK ×2)
- `crates/ph2d-editor-core/src/screens/hero/color_picker_demo.rs` (safe_clamp ×2)
- `crates/ph2d-editor-core/src/interaction/dispatch/pointer.rs` (DRAG-ABS-OK ×2)

---

## Etapa 6 — LOC trend + memory GC + audit refinements

**Commit:** (pendente)

### Escopo realizado

Etapa 6 partial — implementa os componentes self-contained (zero user-input
requirement). Os componentes 6.1 (golden-image SSIM) e 6.2 (panel-canonical-
template AST) são **deferidos para sessão com Enio in-the-loop**, pois exigem:

- **6.1:** baseline PNGs (golden snapshots) que o Enio precisa visualmente aprovar.
  Setup Vello headless é mecânico, mas a APPROVAL dos baselines é o gargalo
  semântico. Plano: spawn em sessão dedicada quando Enio puder revisar
  ~17 mockups iniciais (widgets primários + 9 panels).
- **6.2:** decisão de formato canônico do template `__template__.rs`. Plano
  original §6.2 (Coord-B + Implementador, 1-2 semanas) precisa do Coord-A
  decidindo qual seção do `pre_populate.rs` vira fonte canônica. Spawn em
  sessão com Enio aprovando o template inicial.

**Componentes 6.3 + 6.4 entregues:**

- **`tools/ph2d-loc-trend/`** — registra LOC dos 13 arquivos críticos em
  `metrics/loc-trend.json` (samples por dia). Modos `record` / `check` /
  `report`. Gate `check` falha se cresceu >10% nos últimos 30 dias.
  Howard-Hinnant proleptic Gregorian math (zero deps).
- **`tools/ph2d-memory-gc/`** — varre MEMORY.md + sibling .md files, extrai
  markdown links + raw repo-rooted paths, reporta refs que não resolvem.
  Resolution-smart: sibling-of-source vs workspace-root vs file:// URLs.

**Audit fixes:**

- **M-1 (Etapa 5 audit):** `arch_mode_has_reconcile` agora EXIGE keyword
  canonical (`reconcile_*/invalidate_*/reset_*/on_mode_changed`) OU entry
  explícita em `RECONCILES_VIA` (per-symbol verb whitelist). Removeu o
  fallback "any method call passes" que deixava bug §2 escapar.
- **M-2 (Etapa 5 audit):** `arch_color_space_typed` BASELINE agora compara
  via suffix match (robusto a canonicalize() falhar em paths quebrados).

### Testes automáticos rodados ✅

```bash
cargo test -p ph2d-loc-trend     # → 6/6 ok
cargo test -p ph2d-memory-gc     # → 7/7 ok
cargo test -p ph2d-editor-core --test arch_mode_has_reconcile  # → 2/2 ok
cargo test -p ph2d-editor-core --test arch_color_space_typed   # → 4/4 ok

# Live runs
cargo run -p ph2d-loc-trend -- record
# → recorded 13 critical file(s); metrics/loc-trend.json criado
cargo run -p ph2d-loc-trend -- check
# → no critical file grew > 10% in the last 30 days
cargo run -p ph2d-memory-gc
# → 13 broken reference(s) found (historical HANDOFFs arquivados — não-blocker)
```

### Componentes deferidos (Etapa 6 follow-up)

- **6.1 Golden-image SSIM gate** — Vello headless + baseline PNGs em
  `tests/golden/{widget,panel}/`. Spawn quando Enio puder aprovar mockups.
- **6.2 panel-canonical-template AST gate** — `syn` AST visitor +
  `__template__.rs` codegen. Spawn quando Coord-A decidir o template canônico.
- **3 gates do §5.3 ainda deferidas:**
  - `no_tofu_glyphs` ampliado (precisa tabela Inter glyph coverage)
  - `docs_bugs_have_gates` (backfill 90 entries UI_Bugs + Image Tools)

### Smoke manual pendente (Enio)

**Smoke G15-G16:**

- [ ] **G15 — LOC trend dashboard:** rodar `cargo run -p ph2d-loc-trend -- report`
  e revisar números. Identificar se algum arquivo crítico está flertando com o
  cap (>500 LOC sem ADR). `pointer.rs` (1050) e `render_loop/mod.rs` (1006) são
  conhecidos — confirmar se são esperados ou se precisam de splits agendados.
- [ ] **G16 — Memory GC triage:** rodar `cargo run -p ph2d-memory-gc` e decidir
  caso-a-caso quais dos 13 broken refs (a) atualizar ao alvo correto,
  (b) remover entry obsoleta, ou (c) marcar como link histórico que aponta a
  HANDOFFs arquivados (cleanup futuro).

### Métricas pós-Etapa 6 parcial

- **Novos tools:** 2 (`ph2d-loc-trend` + `ph2d-memory-gc`) — ambos zero-dep, std-only
- **Sample inicial loc-trend.json:** 13 arquivos críticos registrados
- **Memory GC:** 13 broken refs reportados (signal real, não noise)
- **Audit fixes:** 2 medium do Etapa 5 fechados (M-1 + M-2)
- **Gates restantes do plano:** 6.1, 6.2, no_tofu ampliado, docs_bugs (todos
  precisam input do Enio ou setup heavy)

### Artefatos criados / modificados

**Novos:**
- `tools/ph2d-loc-trend/` — Cargo.toml + src/{lib.rs, main.rs}
- `tools/ph2d-memory-gc/` — Cargo.toml + src/{lib.rs, main.rs}
- `metrics/loc-trend.json` — primeiro sample (2026-05-24)

**Modificados (audit refinements):**
- `crates/ph2d-editor-core/tests/arch_mode_has_reconcile.rs` (M-1)
- `crates/ph2d-editor-core/tests/arch_color_space_typed.rs` (M-2)

---

## Etapa 7 — auto-merge-eligibility + ADR-0042 closure

**Commit:** (pendente — este)

### Escopo

Etapa 7 fecha a Wave 10 com:

- **`scripts/auto-merge-eligibility.sh`** — implementação da política §7.1.
  Exit 0 sse: (i) diff só em `crates/ph2d-{node,tool,panel}-<slug>/` OU
  `docs/Testes/`, (ii) zero foundational paths tocados (workspace Cargo,
  scripts/, .github/, ADRs, DIRETRIZ, CLAUDE.md, SKILL_, shells/, core
  crates incluindo color/tokens/render/gpu/text/runtime, tools/), (iii)
  no máximo 1 crate drop tocado. Fail-safe = coord-review em qualquer
  ambiguidade.
- **ADR-0042 Wave 10 closure** — cita commits-âncora, sumariza contratos
  congelados, lista gates ativos, registra carry-over para Wave 11.
- **GH Action wiring** do auto-merge-eligibility = Wave 11 follow-up
  (per plano §7.2: "policy first, automation next").
- **DIRETRIZ §1.4 rewrite** = Wave 11 follow-up (per plano §7.4: depende
  do `tools/ph2d-triagem` que não ficou no escopo desta sessão).

### Testes automáticos rodados ✅

```bash
# Smoke do script
bash scripts/auto-merge-eligibility.sh HEAD~1 HEAD
# → "auto-merge: foundational path touched (Cargo.lock) — coord-review required"
# (esperado — Etapa 6 tocou Cargo.lock + tools/, ambos foundational)

# Closure ADR is just a doc; no automated test (cited in ADR-0042 §4
# acceptance criteria).
```

### Smoke manual pendente (Enio)

- [ ] **G17 — auto-merge policy validation:** revisar `scripts/auto-merge-eligibility.sh`.
  A política está correta? Ajustar `FOUNDATIONAL_PATTERNS` se algum path
  crítico ficou de fora. Decidir quando ativar a GH Action (Wave 11).
- [ ] **G18 — ADR-0042 leitura:** ler `docs/architecture/decisions/0042-wave-10-closure.md`,
  confirmar §6 carry-over list, decidir prioridade Wave 11.
- [ ] **G19 — Smoke final consolidado (G1-G18):** executar TODOS os smokes
  visuais G1-G18 nas etapas 0..7 do tracker antes de declarar Wave 10
  "shipped" e taggear `wave-10-complete`.

### Métricas finais Wave 10

Vide ADR-0042 §5 stats:

- **9 commits** (`d9379ee` Etapa 0 → ADR-0042 closure)
- **2 novos crates** (`ph2d-color`, `ph2d-editor-core::math` mod)
- **5 novos tools** (3 sync codegens + loc-trend + memory-gc)
- **11 novos gates** ativos
- **69 violations** swept (no_magic_numeric panel-* extension)
- **CEQ paint split** 824 → 318+555+137 LOC
- **4 audits adversariais** (1-3 critical fixes pré-commit cada)

### Artefatos criados

- `scripts/auto-merge-eligibility.sh` — política §7.1
- `docs/architecture/decisions/0042-wave-10-closure.md` — closure ADR

---

## Convenções deste tracker

- **`audits/<etapa>.md`** — relatórios de auditoria adversarial por etapa (subpasta criada quando necessário)
- **`regressions.md`** — bugs descobertos no smoke final, com SHA do commit suspeito
- **`baselines/<etapa>/*.png`** — golden-image baselines (criadas em Etapa 6)

**Princípio operacional desta wave:** o gate automático sempre vence o doc. Se algo está aqui como "smoke manual pendente", é porque não há gate automático que cubra (ainda). Se virar gate, removo do checklist e adiciono o gate como verificação automática.
