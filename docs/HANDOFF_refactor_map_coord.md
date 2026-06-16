═══════════════════════════════════════════════════════════════════
HANDOFF / PLANO — Refatoração de arquivos oversized (padrão-ouro)
Autor: Coordenador · 2026-06-03 · pedido do Enio ("tool.rs muito grande,
tendência a crescer; estamos no início do APP; identifique + levante agentes")
═══════════════════════════════════════════════════════════════════

## §0 — Princípio
Estamos no início do APP: o custo de fixar fronteiras de módulo AGORA é
mínimo e o retorno (multi-agente sem colisão, build incremental, leitura)
é enorme. Padrão-ouro: **um arquivo = um tópico coeso**, teto ~700 linhas
(impl) / ~800 (tests), tipo público fica no mesmo path, `impl` espalhado
por arquivos do MESMO crate (Rust permite). Refator = MOVE mecânico puro:
zero rename público, zero mudança de assinatura, zero "melhoria" de lógica.

## §1 — Survey (top offenders por LOC, 2026-06-03)
| LOC  | Arquivo | Estado | Ação |
|------|---------|--------|------|
| 4999 | `ph2d-tool-painter/src/tool.rs` | **CONTENDIDO** (Painter impl ativo + wiring CompositorCache) | §2 — split sequenciado PELO Painter impl |
| 2879 | `ph2d-tool-bgremoval/src/tool.rs` | livre | **agente rodando** (Coord) |
| 2699 | `ph2d-painter-brush/src/stamp_scheduler.rs` | quase-contendido (crate do Painter) | §3 — fila, pós-checkpoint Painter |
| 2467 | `ph2d-asset-ktx2/src/lib.rs` | livre (mas tem ISPC? checar) | §3 — fila (cargo SOLO se ISPC) |
| 2442 | `ph2d-tool-color-equalization/src/algorithm.rs` | livre | **agente rodando** (Coord) |
| 2280 | `ph2d-editor-core/.../dispatch/tests.rs` | foundational (test) | §3 — baixa prioridade |
| 1647 | `ph2d-editor-core/src/ids.rs` | **foundational** (Coord-only) | §3 — split por domínio + cuidado: muitos dependentes |
| 1476 | `ph2d-editor-core/src/screens/hero.rs` | **foundational** | §3 — Coord-only |
| 1473 | `shells/desktop/src/render_loop/mod.rs` | **CONTENDIDO** (bridges live) | §3 — pós-checkpoint shell |
| 1427 | `ph2d-editor-core/src/interaction/state/mod.rs` | foundational | §3 |
| 1382 | `ph2d-painter-brush/src/adjustments.rs` | contendido (Painter) | §3 — fila |
| 1301 | `ph2d-render/src/layer_compositor.rs` | Coord | §3 — Coord faz |
| 1245 | `shells/desktop/src/input_dispatch.rs` | **CONTENDIDO** (keyboard/vector_undo live) | §3 — pós-checkpoint |

**Regra de colisão:** os arquivos que MAIS precisam (tool.rs, render_loop,
input_dispatch, painter-brush) são exatamente os que os impls Vector+Painter
editam AGORA. Refatorar concorrente = colisão garantida. Logo: **eu (Coord)
refatoro só o que está livre; o resto é sequenciado** (o dono faz o split num
commit dedicado, no seu próximo checkpoint, antes/depois da feature — nunca no
meio). ≤3 cargos simultâneos (RAM 8 GiB) — por isso 2 agentes Coord agora, não 6.

## §2 — FLAGSHIP: ✅ FEITO (2026-06-04, commit `dccf9e3`)
`tool.rs` (5233 LOC) → `tool/` com 7 arquivos: `mod.rs`(581 struct+Default+
pending-select) · `internal.rs`(409 free-fns+ToolPixelSource, `pub(crate)`) ·
`lifecycle.rs`(735 stroke/journal/ui/undo) · `layers.rs`(863 layer model) ·
`runtime.rs`(404 dock+accessors+preview-drive) · `trait_impls.rs`(399 Tool+
RasterEditTool) · `tests.rs`(1877). **Move mecânico puro** verificado por
round-trip (partição linha-a-linha; só wrappers `impl{}`/`mod{}` regenerados).
Privados inerentes → `pub(crate)` onde cross-file (contrato pub intacto).
Gates: painter_contract 81/81, tool_contract 3/3, clippy `--all-targets`
`-D warnings` limpo, 164 lib + 35 integ verdes, shell compila. **Esta é a
EXEMPLAR** (§4) que os próximos tools/crates stateful seguem. Método: tokenizer
brace-aware (ignora comments/strings) + slicer Python com asserção round-trip —
reusável p/ os splits abaixo.

### Survey + progresso 2026-06-04
**✅ FEITOS nesta jornada (todos verificados: tests + clippy --all-targets + gates):**
- `ph2d-tool-painter/tool.rs` (5233) → `tool/` 7 arquivos — `dccf9e3`
- `ph2d-painter-brush/stamp_scheduler.rs` (2699) → `stamp_scheduler/` 3 — `77baf23`
- `ph2d-tool-painter/compositor.rs` (1202) → `compositor/` 4 — `9809a6a`
- `ph2d-tool-painter/layers.rs` (1068) + `ph2d-painter-brush/cpu_render.rs` (1297) — `38f9aef`
- `ph2d-painter-brush/adjustments.rs` (1463) → `adjustments/` 3 (+gate scoping fix) — `106c86e`
- `ph2d-tool-bgremoval/algorithm/chroma.rs` (1338) → `chroma/` 3 — `94b16cb`
- **Subsistema Painter inteiro de-god-objected.** Método: tokenizer brace-aware
  (`/tmp/rust_tok.py`) + slicer com asserção round-trip. **Pegadinhas aprendidas:**
  (a) `#[cfg(test)]`/`mod tests {` em 2 linhas → confira o exato; (b) assinaturas
  multi-linha + `;` dentro de `[u8; 3]` enganam detecção de fim → use o tokenizer
  de braces, não heurística de `;`; (c) `include_str!("X.rs")` de teste vira
  `concat!(include_str!("mod.rs"), ...)`; (d) gates que scaneiam por filename
  (`crate_source_excluding`) precisam casar componente de path após split-em-dir;
  (e) campos privados de struct construída cross-file + métodos privados chamados
  cross-file → `pub(crate)` (contrato pub intacto).

**FILA RESTANTE (recomendo: shipar o batch atual primeiro — 51 commits — antes destes):**
| LOC | Arquivo | Cat | Nota |
|---|---|---|---|
| ~~2467~~ | ~~`ph2d-asset-ktx2/src/lib.rs`~~ | ✅ **DONE** `021e84a` (2026-06-04) | split em mods irmãos: limits/error/format/image/decode/patch + tests.rs; API pública idêntica (3 deps compilam); doc-links cross-módulo → `crate::`-qualified |
| 2280 | `editor-core/.../dispatch/tests.rs` | foundational test (livre) | só tests |
| 1666 | `editor-core/src/ids.rs` | **foundational** (muitos deps) | split por domínio |
| 1511 | `shells/.../render_loop/mod.rs` | **CONTENDED** (bridges) | só no checkpoint shell |
| 1494 | `editor-core/src/screens/hero.rs` | **foundational** | Coord cuidado |
| ~~1480~~ | ~~`ph2d-render/src/layer_compositor.rs`~~ | ✅ **DONE** `0a92e4c` (2026-06-04) | parent-child: mod.rs mantém tipos/pods/free-fns, impl(578L)+tests viram filhos via `use super::*` (zero pub(crate)); `include_str!`→`../shaders/`; ABI/WGSL gates verdes |
| 1427 | `editor-core/src/interaction/state/mod.rs` | **foundational** | Coord |
| 1252 | `shells/.../input_dispatch.rs` | **CONTENDED** (keyboard WIP) | NÃO tocar agora |
| ~~1223~~ | ~~`ph2d-imageio-ora/src/lib.rs`~~ | ✅ **DONE** `122f37f` (2026-06-04) | mods irmãos import/export/blend; doc-links `ph2d_imageio::`-qualified (fixou LayerEffect); cargo doc zero-warning |
| ~~1156~~ | ~~`ph2d-render/src/sprite.rs`~~ | ✅ **DONE** `fac4e88` (2026-06-04) | dir-split component/instance/vertex; GPU pods movidos verbatim; 10 `[Sprite::x]` doc-links qualificados; offset/Pod ABI guards verdes |
JÁ FEITOS antes (sumiram do topo): bgremoval/tool.rs, color-equalization/algorithm.rs.

---

## §2-old — receita do split de tool.rs (mantida p/ referência do método)
**Quem:** o Painter impl (é o teu arquivo ativo). **Quando:** commit dedicado,
SEPARADO do wiring CompositorCache. Recomendo ORDEM: (1) landa o wiring
CompositorCache (HANDOFF_painter_w5_compositor_cache_tool_wiring_coord.md, é
pequeno) e commita; (2) DEPOIS faz este split como 1 commit mecânico isolado
("refactor(painter): split PainterTool god-object into tool/ submodules"). Assim
o diff do split é puro-move e revisável; o wiring não some no ruído.

O crate JÁ tem o padrão certo nos irmãos (`color.rs`/`compositor.rs`/`layers.rs`/
`params.rs`/`undo.rs`). `tool.rs` é o único god-object: o tipo `PainterTool`
(~186 métodos) + ~1800 linhas de testes inline. Seams reais já marcados pelos
banners `// ── ... ──` e pelos blocos `impl`:

Layout-alvo — criar `crates/ph2d-tool-painter/src/tool/`:
- `tool/mod.rs` — `struct PainterTool` + campos + `impl Default` + `pub use` das
  partes. **`PainterTool` continua em `ph2d_tool_painter::PainterTool`** (lib.rs
  intacto). `ToolPixelSource` + seu `impl LayerPixelSource` ficam aqui ou em
  `tool/pixel_source.rs`.
- `tool/core.rs` — construção/`set_source`, mutadores de estado base, e o
  DRIVE de composite (drain / `run_full` / dirty-rect lane). É onde o wiring
  CompositorCache mora (drain + flag).
- `tool/layers_api.rs` — grupo "W3 layer model" (add/select/move/visibility/
  opacity/blend/group/mask) (≈ 1388–2165).
- `tool/panel_events.rs` — `handle_panel_event` + per-row decode (≈ 3345) + dock
  toggle (≈ 2165).
- `tool/tool_trait.rs` — `impl Tool for PainterTool` (≈ 2587: id/icon/activate/
  pointer…).
- `tool/raster_edit.rs` — `impl RasterEditTool for PainterTool` (≈ 2891) +
  stamp/stroke entry.
- `tool/multiselect.rs` — W3 multi-seleção (≈ 4698).
- `tool/mask.rs` — mask Invert/Apply (≈ 4799).
- `tool/adjustments.rs` — W4 adjustment create/edit + `set_adjustment_param`
  (≈ 4915) + a invalidação `invalidate_above` do cache.
- Testes: mover cada `#[cfg(test)] mod tests` para JUNTO do código que cobre
  (um por submódulo) OU `tool/tests/` por tópico. Os ~1800 LOC de teste somem
  do arquivo principal — esse é metade do ganho.

Invariantes (gold standard, mecânico):
- API pública IDÊNTICA (mesmos `pub`/paths; `architecture_painter_contract_surface`
  e `architecture_tool_contract_surface` DEVEM continuar verdes — contam itens
  pub, não arquivos).
- Visibilidade entre submódulos: `pub(crate)` mínimo onde um método passa a ser
  chamado cross-file.
- Zero mudança de comportamento. Bug avistado → reporta, NÃO conserta no split.
- Gate de LOC/fn-size do crate continua verde (e PARA de mascarar dívida —
  arquivos menores = parser do gate não erra, vide memória panel-LOC-gate).
- `cargo check/clippy/test -p ph2d-tool-painter` verdes; `cargo fmt -p`.
- Commit SCOPED: `git add -- crates/ph2d-tool-painter/src/tool.rs crates/ph2d-tool-painter/src/tool/...`; `--no-verify`; trailer Co-Authored-By; NÃO pusha.

## §3 — Fila sequenciada (depois)
1. **Coord, livre AGORA (rodando):** bgremoval/tool.rs, color-equalization/algorithm.rs.
2. **Coord, próximo (livre):** ktx2/lib.rs (cargo SOLO se ISPC), imageio-ora/tiff
   lib.rs, ph2d-render/layer_compositor.rs + sprite.rs (meus).
3. **Painter impl, no checkpoint:** tool.rs (§2), depois painter-brush
   {stamp_scheduler.rs, adjustments.rs, cpu_render.rs}.
4. **Shell, no checkpoint:** render_loop/mod.rs (extrair cada bridge já é um
   arquivo; mod.rs vira só o orquestrador) + input_dispatch.rs.
5. **Coord-only + ADR (foundational, cauteloso):** editor-core/ids.rs (split por
   domínio de id), screens/hero.rs, interaction/state/mod.rs. Muitos dependentes
   → arch-gate de surface + grep de call-sites antes.

## §4 — Padrão a replicar (do que sair dos 2 agentes Coord)
O 1º crate refatorado vira o EXEMPLAR documentado aqui (layout `tool/` +
convenção de onde vão testes + visibilidade). Os splits seguintes (tool.rs
incl.) seguem o mesmo molde — consistência > improviso.
═══════════════════════════════════════════════════════════════════
