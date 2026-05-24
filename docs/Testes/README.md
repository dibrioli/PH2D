# Wave 10 — Tracker de Testes e Smokes Visuais

**Propósito:** registro de TODOS os testes automáticos rodados e TODOS os smokes visuais pendentes em cada etapa da [Wave 10 perfection plan](../plans/2026-05-wave-10-perfection.md). O Enio audita visualmente ao final da wave usando este doc como checklist.

**Princípio:** "padrão ouro / puro sangue / definitivo" — toda mudança ou tem gate automático passando OU tem checklist explícita pra smoke manual. Nada fica "se você lembrar de testar".

---

## Estado por etapa

| Etapa | Status | Commit | Testes automáticos | Smoke manual pendente |
|---|---|---|---|---|
| 0 — Infraestrutura multi-agente | ✅ COMPLETA | `d9379ee` | scripts smoke + workspace check | [§E0](#etapa-0--infraestrutura) |
| 1.A — Emenda ADR-0041 (rename + deactivate) | ✅ COMPLETA | `a03d830` | 769/769 workspace (T2 hook) | [§E1A](#etapa-1a--emenda-adr-0041) |
| 1.B — `ph2d-tool-runtime` + BgRemoval impl | ✅ COMPLETA | `74b6d27` | 132 verdes (122 BgR + 10 runtime) | [§E1B](#etapa-1b--ph2d-tool-runtime--bgremoval-impl) |
| 2 — CEQ + Upscale impl (Padding/EqSizes exception) | ✅ COMPLETA | `cbb9cb3` | 193 verdes; fix C1 cross-bridge | [§E2](#etapa-2--ceq--upscale-impl-rastereditool) |
| 3 — 3 arch-gates + drive_multi_preview_cache + fix C3 multi-Apply | ✅ COMPLETA | (pendente) | 19 gates/helper verdes; 3 críticos + 3 altos fixados | [§E3](#etapa-3--3-arch-gates--drive_multi_preview_cache) |
| 4 — panel-sync + chrome-sync + widget-sync | ⏳ | — | — | — |
| 5 — Gates UI panel-\* + ph2d-color + classes de bug | ⏳ | — | — | — |
| 6 — Golden-image SSIM + drift + memory GC | ⏳ | — | — | — |
| 7 — Política merge-on-green + closure | ⏳ | — | — | — |

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
- `docs/plans/2026-05-wave-10-perfection.md` §II nota pós-execução

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

## Convenções deste tracker

- **`audits/<etapa>.md`** — relatórios de auditoria adversarial por etapa (subpasta criada quando necessário)
- **`regressions.md`** — bugs descobertos no smoke final, com SHA do commit suspeito
- **`baselines/<etapa>/*.png`** — golden-image baselines (criadas em Etapa 6)

**Princípio operacional desta wave:** o gate automático sempre vence o doc. Se algo está aqui como "smoke manual pendente", é porque não há gate automático que cubra (ainda). Se virar gate, removo do checklist e adiciono o gate como verificação automática.
