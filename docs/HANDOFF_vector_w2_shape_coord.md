═══════════════════════════════════════════════════════════════════
HANDOFF → Coordenador · Vector W2 · T2.2 Shapes (impl → Coord wiring)
Autor: Implementador (slot-impl-vector) · 2026-06-01
═══════════════════════════════════════════════════════════════════

## §0 — TL;DR

**T2.2 Shape core PRONTO e verificado** (geradores + tool crate + registro,
gates verdes). A fiação central é **IDÊNTICA à do Pencil** (`69b3788`) —
modelo de drag, mesmos 7 edits, trocando `vector_pencil` → `vector_shape`.
Os 3 arquivos `vector_shape_*` do shell já estão escritos no meu território.

**Bônus:** a seleção de sub-modo (rect/ellipse/polygon/star/spiral) é um
**RadioGroup no panel** do tool → roteia `PanelEvent::SelectOption` pro
`handle_panel_event` (mesmo plumbing do Lock-Axis do Move tool). **Zero
plumbing central novo** para o panel — o shell já renderiza o panel do tool
ativo + roteia controles. Era a tua escolha "panel-toggle" (não teclado). ✓

NÃO pushei. Commits locais escopados.

## §1 — Pronto + verificado (slot-impl-vector)

| Artefato | Local | Verificação |
|---|---|---|
| **Shape generators** | `ph2d-vector-doc::primitives` (rect/ellipse/polygon/star/spiral) | 10 testes ✓, clippy ✓, commit `99e937a` |
| **Shape tool** | `ph2d-tool-vector-shape/` (crate novo) | 20 testes ✓, clippy ✓ (drag→generate→commit, panel RadioGroup, preview, replay-safe edit_log, color inject, guards) |
| IconId + SVG | `icons.rs` (`VectorShape` + `ALL_ICONS`) + `docs/design/icons/vector-shape.svg` | `enum_order_matches_svgs` ✓ |
| Design TOML | `docs/design/tools/vector_shape.toml` | design-sync 3/3 ✓ |
| Codegen (tool-sync) | register_all + register_all_tools + Cargo deps + icon-slugs | staleness 6/6 ✓ |

Geometria vem do mesmo `primitives` que o nó `vector-source` do W3 (T3.2)
consolida. Fechados = region preenchida; spiral = path aberto stroked —
ambos no `draw_vector_network` canônico (fill-pass + o stroke-pass que tu
landaste no `69b3788`).

## §2 — Fiação central (Coord) — espelha EXATAMENTE a do Pencil

Mesmos 7 edits do `69b3788`, símbolos `vector_shape`:

| # | Arquivo | Edit |
|---|---|---|
| 1 | `shells/desktop/Cargo.toml` | dep `ph2d-tool-vector-shape` |
| 2 | `render_loop/mod.rs` | `mod vector_shape_bridge;` + `vector_shape_bridge::dispatch(tools, camera, window_size, &mut self.committed_vector_pen_paths, vector_scene)` (mesma lista compartilhada); `shape_has_in_progress_shape(...)` no warn destrutivo |
| 3 | `input_dispatch.rs` | `mod vector_shape_input;` + roteamento de **DRAG** (Down→`try_vector_shape_pointer_down`, Move-pressed→`try_vector_shape_pointer_drag`, Up→`try_vector_shape_pointer_up`, off-canvas consume) — idêntico ao Pencil |
| 4 | `keyboard.rs` | Esc → `try_vector_shape_escape` |
| 5 | `ids.rs` | `pub const TOPBAR_VECTOR_SHAPE` (mirror `TOPBAR_VECTOR_PENCIL`) |
| 6 | `chrome/mod.rs` | `mod vector_shape_toggle;` + `vector_shape_toggle::apply(...)` na cadeia |
| 7 | `fixture.rs` | pill SHAPE (`IconId::VectorShape`) no cluster `vector_tools` |

Arquivos meus prontos: `render_loop/vector_shape_bridge.rs`,
`input_dispatch/vector_shape_input.rs`, `chrome/vector_shape_toggle.rs`.
(Não compilam standalone — dependem dos símbolos centrais; fluxo invertido,
igual ao Pencil. O bridge do shape usa só `draw_vector_network` + drain —
preview e committed renderizam pelo caminho canônico.)

**Se algum dos 3 der erro de compile após a fiação, me manda — são meus.**

## §3 — W2 restante

T2.1 Pencil ✓ (wired). T2.2 Shape core ✓ (aguarda fiação §2). Faltam: smoke
Day-8 do Enio (após §2; Shape pill → sub-modo → drag → commit), T2.3
Select/Direct, T2.4 Color picker (gotcha §4.1: `ph2d-painter-color::ClassicPicker`
não existe → confirmar widget), T2.5 Undo CRDT, T2.6 Audit.

═══════════════════════════════════════════════════════════════════
