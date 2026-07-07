//! Node UI side-metadata (Motion Nodes M1.R1).
//!
//! Keyed by `NodeTypeId`, **additive and non-frozen** — it lives beside the ops
//! on [`super::NodeRegistry`], entirely outside the frozen `NodeManifest` (8
//! fields) and its contract gate. The motion-graph editor reads it to label,
//! categorize, and shape each node card; a node registers its UI manifest right
//! next to its op in `register`.
//!
//! Param hints (`ParamUiHint`) + attribute access (`AttrAccess`) land with the
//! params panel (M1.P1) as further additive fields here.

/// Visual category of a node — selects the card's header tint (the `node-cat-*`
/// ColorTokens) so the palette's colors teach the library map (plan §2.4).
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum NodeUiCategory {
    /// Generators (grid, random, …) — green.
    Source,
    /// Distribution / cloning — muted green.
    Distribute,
    /// Spatial transforms (move, scale, rotate) — blue.
    Transform,
    /// Falloffs / focus fields — amber.
    Focus,
    /// Color / stylistic effects — magenta.
    Fx,
    /// Terminal / output — red.
    Output,
    /// Values, adapters, utilities — grey.
    Utility,
}

/// Card silhouette — the 7 body shapes (plan §2.4). `Rect` is the neutral
/// modifier default; the others read the node's role at a glance.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum NodeSilhouette {
    /// Rounded rectangle — a modifier (default).
    Rect,
    /// Cigar (fully rounded ends) — merge / group.
    Cigar,
    /// Circle — a terminal / sink.
    Circle,
    /// Diamond — a value / gate.
    Diamond,
    /// Trapezoid widening downward — a source / generator.
    TrapezoidDown,
    /// Trapezoid narrowing downward — a sink.
    TrapezoidUp,
    /// Tabbed rect — an event / signal node.
    Tabbed,
}

/// Per-node-type UI metadata (M1.R1). `display_name` is the English,
/// result-named label (HR-15); `category` + `silhouette` drive the card look.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct NodeUiManifest {
    pub display_name: &'static str,
    pub category: NodeUiCategory,
    pub silhouette: NodeSilhouette,
}
