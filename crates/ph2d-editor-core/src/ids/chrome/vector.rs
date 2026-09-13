//! Vector module chrome NodeIds (VGRAPH_* geometry-graph + VECTOR_INSPECTOR_*).
use super::{NodeId, hash_node_id};

/// Vector Geometry-Graph panel (W3 T3.1) — docked panel that places the
/// `vector.source` node + drives its 8 params (sliders) and renders the cooked
/// `VectorNetwork` live. Outer-rect id for `z_order`; the 8 param sliders below.
pub const VGRAPH_PANEL: NodeId = hash_node_id("vgraph.panel");
/// Vector Inspector panel (W2 T2.4) — minimal right-docked panel hosting the
/// fill swatch (+ future vertex/node params). Outer-rect id for `z_order`.
pub const VECTOR_INSPECTOR_PANEL: NodeId = hash_node_id("vector_inspector.panel");

// ── Vector tool Style panel (ADR-0108 cutover — docked `ph2d-panel-vector`) ──
// The `vector` tool's Style controls live in a right-docked `Panel<State>` (the
// tool `FloatingPanel` is unpainted). Width slider (1..20 px) + Stroke / Fill
// colour swatches (each opens the shared OKLCH picker via `is_picker_swatch`) +
// a Fill "None" affordance. Distinct slug family from the retired
// `vector_inspector.*` ids above.
/// Vector Style panel outer rect id (for `z_order` + hit-barrier).
pub const VECTOR_PANEL: NodeId = hash_node_id("vector.panel");
/// Vector Style panel close (X) button.
pub const VECTOR_CLOSE: NodeId = hash_node_id("vector.close");

// ── Formas: seletor + campos GENÉRICOS (catálogo data-driven) ───────────────
// Com 25+ formas, um id por forma e um id por parâmetro seria insustentável (e o
// painel, um pântano de `match`). Os ids são GERADOS por índice: o painel itera o
// catálogo (`ph2d_tool_vector::shapes`) e pinta o botão `i` e o campo `j`; a tool
// resolve o índice de volta para a forma / o parâmetro. Uma forma nova não precisa
// de id nenhum.

// ── Compound paths (ADR-0108 — subpaths + fill rule) ─────────────────────────
/// Merge the selected closed paths into ONE compound path (a contour inside
/// another becomes a hole, via `EvenOdd`). Inverse: [`VECTOR_COMPOUND_RELEASE`].
pub const VECTOR_COMPOUND_MAKE: NodeId = hash_node_id("vector.compound.make");

pub const VECTOR_SNAP_ON: NodeId = hash_node_id("vector.snap.on");

// ── Reforma da UI do painel (categoria ≠ tipo) ───────────────────────────────
// Bloco APPEND-ONLY (isolamento de linha): o 5º pill de modo, o chip de família do
// catálogo e os cabeçalhos COLAPSÁVEIS de cada seção. O painel hand-rolava 14
// rótulos de seção e ZERO `SectionHeader`, violando o canon
// (`docs/UI_Padrao/components/section_header.md`: toda seção usa
// `paint_section_header` e toda seção é colapsável).
