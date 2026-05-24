# Wave 10 — Tracker de Testes e Smokes Visuais

**Propósito:** registro de TODOS os testes automáticos rodados e TODOS os smokes visuais pendentes em cada etapa da [Wave 10 perfection plan](../plans/2026-05-wave-10-perfection.md). O Enio audita visualmente ao final da wave usando este doc como checklist.

**Princípio:** "padrão ouro / puro sangue / definitivo" — toda mudança ou tem gate automático passando OU tem checklist explícita pra smoke manual. Nada fica "se você lembrar de testar".

---

## Estado por etapa

| Etapa | Status | Commit | Testes automáticos | Smoke manual pendente |
|---|---|---|---|---|
| 0 — Infraestrutura multi-agente | ✅ COMPLETA | `d9379ee` | scripts smoke + workspace check | [§E0](#etapa-0--infraestrutura) |
| 1.A — Emenda ADR-0041 (rename + deactivate) | ✅ COMPLETA | `a03d830` | 769/769 workspace (T2 hook) | [§E1A](#etapa-1a--emenda-adr-0041) |
| 1.B — `ph2d-tool-runtime` + BgRemoval impl | ✅ COMPLETA | (pendente) | 132 verdes (122 BgR + 10 runtime) | [§E1B](#etapa-1b--ph2d-tool-runtime--bgremoval-impl) |
| 2 — 4 tools impl `RasterEditTool` | ⏳ | — | — | — |
| 3 — Gates + apaga shell legacy | ⏳ | — | — | — |
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

## Convenções deste tracker

- **`audits/<etapa>.md`** — relatórios de auditoria adversarial por etapa (subpasta criada quando necessário)
- **`regressions.md`** — bugs descobertos no smoke final, com SHA do commit suspeito
- **`baselines/<etapa>/*.png`** — golden-image baselines (criadas em Etapa 6)

**Princípio operacional desta wave:** o gate automático sempre vence o doc. Se algo está aqui como "smoke manual pendente", é porque não há gate automático que cubra (ainda). Se virar gate, removo do checklist e adiciono o gate como verificação automática.
