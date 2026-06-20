//! Vector module chrome NodeIds (VGRAPH_* geometry-graph + VECTOR_INSPECTOR_*).
use super::{NodeId, hash_node_id};

/// Vector Geometry-Graph panel (W3 T3.1) — docked panel that places the
/// `vector.source` node + drives its 8 params (sliders) and renders the cooked
/// `VectorNetwork` live. Outer-rect id for `z_order`; the 8 param sliders below.
pub const VGRAPH_PANEL: NodeId = hash_node_id("vgraph.panel");
pub const VGRAPH_KIND: NodeId = hash_node_id("vgraph.kind");
pub const VGRAPH_WIDTH: NodeId = hash_node_id("vgraph.width");
pub const VGRAPH_HEIGHT: NodeId = hash_node_id("vgraph.height");
pub const VGRAPH_SIDES: NodeId = hash_node_id("vgraph.sides");
pub const VGRAPH_INNER_RATIO: NodeId = hash_node_id("vgraph.inner_ratio");
pub const VGRAPH_TURNS: NodeId = hash_node_id("vgraph.turns");
pub const VGRAPH_SAMPLES_PER_TURN: NodeId = hash_node_id("vgraph.samples_per_turn");
pub const VGRAPH_ROTATION: NodeId = hash_node_id("vgraph.rotation");
/// Vector Inspector panel (W2 T2.4) — minimal right-docked panel hosting the
/// fill swatch (+ future vertex/node params). Outer-rect id for `z_order`.
pub const VECTOR_INSPECTOR_PANEL: NodeId = hash_node_id("vector_inspector.panel");
/// Vector Inspector close (X) button.
pub const VECTOR_INSPECTOR_CLOSE: NodeId = hash_node_id("vector_inspector.close");
/// Vector Inspector fill-color swatch — a picker swatch (opens the Blender
/// picker on Down via `is_picker_swatch`); the shell read-back applies the
/// picked color to the selected regions.
pub const VECTOR_INSPECTOR_FILL_SWATCH: NodeId = hash_node_id("vector_inspector.fill_swatch");
/// Vector Inspector Shape-kind picker (W2 §4.2) — the on-screen replacement for
/// the interim hotkeys 1-5. A vertical 5-option segmented control shown only
/// while the `vector_shape` tool is active. `_KIND` is the group/label id; the
/// five `_SHAPE_*` ids are the per-option Button hit-targets (the generic
/// RadioGroup dispatch is not wired yet, so each option is a Button — its
/// `Click` routes through the shell bridge to `VectorShapeTool::set_kind`).
pub const VECTOR_INSPECTOR_SHAPE_KIND: NodeId = hash_node_id("vector_inspector.shape_kind");
pub const VECTOR_INSPECTOR_SHAPE_RECT: NodeId = hash_node_id("vector_inspector.shape.rect");
pub const VECTOR_INSPECTOR_SHAPE_ELLIPSE: NodeId = hash_node_id("vector_inspector.shape.ellipse");
pub const VECTOR_INSPECTOR_SHAPE_POLYGON: NodeId = hash_node_id("vector_inspector.shape.polygon");
pub const VECTOR_INSPECTOR_SHAPE_STAR: NodeId = hash_node_id("vector_inspector.shape.star");
pub const VECTOR_INSPECTOR_SHAPE_SPIRAL: NodeId = hash_node_id("vector_inspector.shape.spiral");
