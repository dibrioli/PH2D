# ADR-0028 — Wave 2: build-time codegen + design canonical sources + lint guards

**Status:** Accepted
**Data:** 2026-05-16
**Decisor(es):** Enio + LLM Coordenador
**Antecessor:** [ADR-0027 — Convention-by-discovery](0027-convention-by-discovery.md)

## Contexto

Wave 1 (ADR-0027) entregou:

- `ph2d-tool-registry` + `ph2d-tool-registry-init` infraestrutura
- 4 tool-crates (`make_square`, `grid_snap`, `bgremoval`, `trim_transparency`) com
  `MANIFEST` const + `register(&mut Registry)`
- Shell decomposition (`init.rs` / `forwarding.rs` / `hero_intents.rs`)
- HR-18 declarada (mas inativa)

Auditoria pós-Wave-1 em 2026-05-16 identificou 10 pontos de colisão remanescentes
que continuam impedindo trabalho multi-agente paralelo:

| # | Ponto | Sintoma |
|---|-------|---------|
| 1 | `tokens.json` divergia dos valores hard-coded em `color.rs` | Designer (Claude Design) editava JSON; Rust ignorava |
| 2 | `icons.rs::cmds()` tinha 715 LOC de match-arms manuais portados de SVG | Cada ícone novo = edit central |
| 3 | NodeId chrome alocados à mão em ranges 100..1099 | 6 colisões numéricas silenciosas pré-Wave 2 |
| 4 | `topbar_clusters()` hard-coded; tool nova precisava editar fixture central | Source-of-truth fragmentada |
| 5 | Sem source canonical para tool functionality | TOML ↔ Rust drift inevitável |
| 6 | Sem lint anti-`0xRRGGBB` literal em widget/screens | Regressão de tema silenciosa |
| 7 | Arquivos > 600 LOC sem gate (HR-18 declarada mas inativa) | God-files crescendo |
| 8 | `panel.rs` 2869 LOC, `state.rs` 1250 LOC monolíticos | Multi-agente colidia em 4119 LOC |
| 9 | `screens/hero.rs` 3300 LOC com 50+ campos de estado | Toda interação tocava esta god-struct |
| 10 | `main.rs` 2416 LOC com 20 `pending_X` drains | Action Bus pendente |

## Decisão

PH2D adota **codegen-from-design-canonical + lint-as-spec** como mecanismo
permanente:

### 1. `docs/design/` é canonical

Toda aparência (tokens, icons) e declaração funcional (tools) origina aqui.

- `docs/design/tokens.json` — 4 themes × 33 OKLCH color tokens.
- `docs/design/icons/*.svg` — 100 Lucide-derived glyphs (24×24 design space).
- `docs/design/tools/*.toml` — per-tool functional spec (id / cluster / zone / order /
  icon_slug / a11y_role / label / memory_budget).

### 2. `build.rs` codegen elimina sync manual

- `crates/ph2d-tokens/build.rs` lê `tokens.json` → emite `tokens_generated.rs` com
  const arrays de 4 themes resolvidos (`$inherits` aplicado).
- `crates/ph2d-editor/build.rs` lê `docs/design/icons/*.svg` → emite
  `icons_generated.rs` com per-icon `IconCmd` arrays + `ICON_CMDS_BY_ID` + `lookup_cmds`
  + `ALL_ICON_SLUGS`.

Cargo `cargo:rerun-if-changed=` em ambos garante rebuild automático.

### 3. Hash-derived NodeIds

`hash_node_id(s: &'static str) -> NodeId` em `ph2d-tool-registry::node_id`
(FNV-1a 64-bit const-fn) substitui ranges manuais em `screens/hero/ids.rs`. 250
consts migradas; 12 fixture row ids (HIER_PLAYER..HIER_MAIN_CAMERA) ficam numeric
porque participam de bit-flag math (`EYE_TOGGLE_BIT` / `EXPAND_TOGGLE_BIT`).

`COMPANION_ROW_ID_MAX = 2^32` guard nos detection helpers garante que hash-output
chrome (que pode acidentalmente setar bits 61/62) não seja mis-detectado como row
companion.

### 4. Chrome derivado do Registry

`paint_image_action_row()` consome `Registry::cluster("image_tools")` em vez de
lista hardcoded. NodeIds dos chrome consts (`IMAGE_ACTION_TRIM` etc.) usam os
MESMOS slugs que os manifest ids — uma única hash, mesma `NodeId`.

`ph2d-editor::install_registry()` + `installed_registry()` via `OnceLock` evitam
propagar `&Registry` por todos os painters.

### 5. Design ↔ Manifest cross-validation

`crates/ph2d-tool-registry-init/tests/tool_manifest_design_sync.rs` enforça
parity field-by-field entre cada `docs/design/tools/<slug>.toml` e o `MANIFEST`
const correspondente. Drift falha CI com diff legível.

### 6. Lint guards

- `no_literal_color.rs` em `ph2d-editor/tests/` bloqueia novos hex literals em
  `widget/**` e `screens/**`. Allowlist `// LITERAL-COLOR-OK: <reason>`.
- `chrome_manifest_coverage.rs` em registry-init enforça que chrome consts ↔
  manifest ids hashes alinhados.
- `node_id_collisions.rs` em ph2d-editor enforça uniqueness pairwise + reserva
  `NodeId::ROOT` + companion-bit safety.

### 7. HR-18 file LOC cap ativo

`shells/desktop/tests/file_loc_caps.rs` gates `.rs` files ≤ 600 LOC.
Exceções via `// ph2d-loc-cap: <reason>` no top do arquivo (primeiros 20
linhas). Duas exceções ativas hoje, ambas pendentes Wave 2.5:

- `shells/desktop/src/main.rs` — 2421 LOC (Action Bus PR 11.8 reduz).
- `shells/desktop/src/hero_intents.rs` — 696 LOC (mesma Action Bus reduz / remove).

Inventário automático em `loc_cap_exceptions_inventory` test emite a lista de
exceções a cada `cargo test` para visibilidade contínua.

## Wave 2.5 — débito conhecido

Os splits internos a seguir foram deferidos por demandarem 2+ sessões cada e
não bloqueam o objetivo de "convention-by-discovery operacional":

- **PR 11.7a** — `grid_snap/panel.rs` (2869 LOC) → multi-file split per kind.
- **PR 11.7d** — `screens/hero.rs` (3300 LOC) → decompor `HeroScreen` state em
  `InspectorState` + `HierarchyState` + `ImageEditState` + `ViewState`.
- **PR 11.8** — Action Bus + drain residuals (main.rs < 400, hero_intents.rs
  colapsa via dispatcher).
- **PR 11.10** — Golden image tests (Vello headless rendering).
- **PR 11.11** — `lib.rs` trim aggressive (`pub use` cleanup com path
  migration de ~30-50 sites).

Plano em [docs/Migracao/2026-05-wave-2-eliminating-all-collisions.md](../../Migracao/2026-05-wave-2-eliminating-all-collisions.md)
(seções PR 11.7a/d/8/10/11).

## Consequências

### Positivas

- **Designer edita TOML/SVG/JSON; Rust replica automaticamente.** Zero sync
  manual entre design canonical e implementação.
- **Adicionar tool nova é 1 crate + 1 linha** em registry-init. Coordenador
  revisa, agente periférico executa, zero contato com chrome painters.
- **Colisões silenciosas eliminadas** — hash NodeIds + cross-validation tests
  fazem qualquer drift falhar build, não comportamento.
- **HR-18 ativo** previne god-file growth daqui pra frente.

### Negativas / a aceitar

- `build.rs` introduz dependência de `OUT_DIR` em todos os builds (compile time
  ligeiramente maior, mas codegen é simples + cached).
- Hash-derived NodeIds tornam debugging com `gdb` menos intuitivo (NodeId(0xAF63...)
  em vez de `NodeId(102)`); compensado pelos `tests/node_id_collisions.rs` com
  printable diagnostics.
- Wave 2.5 debt explícito — main.rs / hero.rs / panel.rs ainda god-files. HR-18
  excepts marker os mark, e a inventory test os surfaces a cada run.

## Métricas

| Métrica | Pré-Wave-2 (2026-05-15) | Pós-Wave-2 (2026-05-16) |
|---------|-------------------------|--------------------------|
| Manual NodeId consts | 253 | 0 (hash-derived) |
| Manual icon match-arms (icons.rs) | 715 LOC | 0 (codegen) |
| Manual color resolve fns (color.rs) | 4 × ~50 LOC = 200 LOC | 0 (codegen) |
| Sources of truth para tools | 4 fragmented (fixture + topbar + icons + color) | 1 canonical (`docs/design/tools/*.toml`) |
| Architecture tests | 1 (interaction_no_alloc) | 7 (collisions / coverage / design-sync / lint / cap × 2 / no-literal-color) |
| Files over 600 LOC (shells) | 2 ungated | 2 with explicit exceptions, cap active |
| Tests verde (workspace) | 1235 | 1296 |

## Status de migração

- ✅ PR 11.1.0 — tokens.json reset to Round 9 (c4f0da6)
- ✅ PR 11.1 — tokens build.rs (aa5331c)
- ✅ PR 11.2 — icons build.rs (e9577d7)
- ✅ PR 11.3 — NodeId hash universal (5e54638)
- ✅ PR 11.4 — Chrome derivado do Registry (8f82407)
- ✅ PR 11.5 — Design canonical TOMLs (7661a7d)
- ✅ PR 11.6 — no-literal-color lint (3d57906)
- ✅ PR 11.7c — topbar.rs split (bc5a456)
- ✅ PR 11.7b — hierarchy.rs split (8843d01)
- ✅ PR 11.9 — HR-18 file-LOC cap ativo (f2cbb20)
- 🔜 Wave 2.5: PR 11.7a, 11.7d, 11.8, 11.10, 11.11

## Referências

- [Plano Wave 2 canonical](../../Migracao/2026-05-wave-2-eliminating-all-collisions.md)
- [SKILL §HR-18](../../../SKILL_Stack_PH2D_Definitiva.md#hr-18--crescimento-bounded-em-shell-binaries)
- [ADR-0027 — Convention-by-discovery (Wave 1)](0027-convention-by-discovery.md)
