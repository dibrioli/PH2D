//! [`VectorShapeTool`] — five primitive shapes with a drag + a panel
//! sub-mode picker.
//!
//! Per plan **T2.2** (`docs/Vector Module/17_plano_de_implementacao.md` §5).
//! Stateful `Tool` with inherent `begin_shape` / `update_shape` /
//! `finish_shape` / `cancel_shape` methods the shell bridge calls after
//! downcasting via [`Tool::as_any_mut`] (ADR-0040 §3 exception, mirror of
//! the Pen / Pencil / Painter tools).
//!
//! ## Sub-modes (panel toggle — plan §5 "5 sub-modes toggle no panel")
//!
//! The five [`ShapeKind`]s are a [`RadioGroup`] in the tool's
//! [`FloatingPanel`]; selecting one routes a
//! [`PanelEvent::SelectOption`] back to [`Self::handle_panel_event`]
//! (same plumbing the Move tool's Lock-Axis radio uses — no new central
//! wiring). The shell renders the active tool's panel automatically.
//!
//! ## Drag interpretation
//!
//! - **Rect / Ellipse:** anchor + current = two corners of the bounding box.
//! - **Polygon / Star / Spiral:** anchor = center, the drag vector sets
//!   the radius (its length) and rotation (its angle).
//!
//! ## Output
//!
//! Closed shapes (rect/ellipse/polygon/star) emit a filled region; the
//! spiral emits an open **stroked** path. Both render via the canonical
//! `ph2d_vector::draw_vector_network` (fill-pass + stroke-pass). The
//! committed asset carries a geometry [`EditLog`] (AddVertex/AddSegment/
//! AddRegion/SetRegionFill) that replays to the same network — the
//! replay-safe foundation T2.5 undo builds on.

use glam::Vec2;
use ph2d_a11y::NodeId;
use ph2d_editor_core::floating_panel::{
    FloatingPanel, PanelAnchor, PanelControl, PanelTab, ToolId,
};
use ph2d_editor_core::tool::{PanelEvent, Tool};
use ph2d_editor_core::widget::{RadioGroup, RadioOption};
use ph2d_vector_doc::{
    EditLog, FillSolid, Ph2dVectorAsset, StrokeCap, StrokeJoin, StrokeStyle, StyleTable,
    TangentsCubic, VectorNetwork, VectorOp, primitives,
};

/// RadioGroup node id for the sub-mode picker (unique within this tool's
/// panel; tool-panel ids are independent of the chrome `ids` set that the
/// `node_id_collisions` gate audits).
const SHAPE_KIND_NODE: NodeId = NodeId(250);
const OPT_RECT: NodeId = NodeId(251);
const OPT_ELLIPSE: NodeId = NodeId(252);
const OPT_POLYGON: NodeId = NodeId(253);
const OPT_STAR: NodeId = NodeId(254);
const OPT_SPIRAL: NodeId = NodeId(255);

/// Minimum drag length (world px) below which `finish_shape` commits
/// nothing (a click, not a drag).
pub const MIN_DRAG_DIST_PX: f32 = 2.0;

/// Same off-canvas magnitude guard as the Pen / Pencil tools.
pub const MAX_COORD_MAGNITUDE: f32 = 1.0e7;

/// Default stroke / fill width.
pub const DEFAULT_STROKE_WIDTH_PX: f32 = 2.0;

/// The five primitive shapes the tool can draw.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ShapeKind {
    /// Axis-aligned rectangle (drag = bounding box).
    #[default]
    Rect,
    /// Ellipse inscribed in the drag bounding box.
    Ellipse,
    /// Regular convex polygon (drag from center = radius + rotation).
    Polygon,
    /// Star (drag from center = outer radius + rotation).
    Star,
    /// Archimedean spiral — open stroked path.
    Spiral,
}

impl ShapeKind {
    /// Stable RadioGroup option value (also the HR-15-safe key).
    #[must_use]
    pub fn value(self) -> &'static str {
        match self {
            Self::Rect => "rect",
            Self::Ellipse => "ellipse",
            Self::Polygon => "polygon",
            Self::Star => "star",
            Self::Spiral => "spiral",
        }
    }

    /// Human-readable panel label.
    fn label(self) -> &'static str {
        match self {
            Self::Rect => "Rectangle",
            Self::Ellipse => "Ellipse",
            Self::Polygon => "Polygon",
            Self::Star => "Star",
            Self::Spiral => "Spiral",
        }
    }

    fn option_id(self) -> NodeId {
        match self {
            Self::Rect => OPT_RECT,
            Self::Ellipse => OPT_ELLIPSE,
            Self::Polygon => OPT_POLYGON,
            Self::Star => OPT_STAR,
            Self::Spiral => OPT_SPIRAL,
        }
    }

    fn from_value(v: &str) -> Option<Self> {
        Some(match v {
            "rect" => Self::Rect,
            "ellipse" => Self::Ellipse,
            "polygon" => Self::Polygon,
            "star" => Self::Star,
            "spiral" => Self::Spiral,
            _ => return None,
        })
    }

    /// All five, in panel order.
    pub const ALL: [ShapeKind; 5] = [
        Self::Rect,
        Self::Ellipse,
        Self::Polygon,
        Self::Star,
        Self::Spiral,
    ];
}

/// Stateful Vector Shape tool.
#[derive(Debug, Clone)]
pub struct VectorShapeTool {
    kind: ShapeKind,
    /// Drag anchor (pointer-down). `None` when idle.
    anchor: Option<Vec2>,
    /// Latest drag position.
    current: Vec2,
    styles: StyleTable,
    fill_ref: u32,
    stroke_ref: u32,
    pending_committed: Option<Ph2dVectorAsset>,
    /// Polygon side count (3..=128).
    pub polygon_sides: u32,
    /// Star point count.
    pub star_points: u32,
    /// Star inner/outer radius ratio (0..1).
    pub star_inner_ratio: f32,
    /// Spiral revolutions.
    pub spiral_turns: f32,
    /// Spiral vertices per turn.
    pub spiral_samples_per_turn: u32,
}

impl Default for VectorShapeTool {
    fn default() -> Self {
        let mut styles = StyleTable::default();
        let visible_fill = FillSolid {
            color: ph2d_color::OklchColor::opaque(0.6, 0.18, 250.0),
        };
        let fill_ref = styles.insert_fill(visible_fill);
        // StrokeStyle is `#[non_exhaustive]` — build via Default + fields.
        let mut stroke = StrokeStyle::default();
        stroke.width = DEFAULT_STROKE_WIDTH_PX;
        stroke.color = ph2d_color::OklchColor::opaque(0.6, 0.18, 250.0);
        stroke.cap = StrokeCap::Round;
        stroke.join = StrokeJoin::Round;
        let stroke_ref = styles.insert_stroke(stroke);
        Self {
            kind: ShapeKind::default(),
            anchor: None,
            current: Vec2::ZERO,
            styles,
            fill_ref,
            stroke_ref,
            pending_committed: None,
            polygon_sides: 6,
            star_points: 5,
            star_inner_ratio: 0.5,
            spiral_turns: 3.0,
            spiral_samples_per_turn: 24,
        }
    }
}

/// Outcome of a Shape pointer interaction.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShapeOutcome {
    /// A drag started.
    Started,
    /// The drag updated (preview should refresh).
    Updated,
    /// A shape was committed; drain via [`VectorShapeTool::take_committed_asset`].
    Committed,
    /// The drag was too small (a click) — nothing committed.
    TooSmall,
    /// Ignored (not dragging) / rejected (non-finite or out-of-range).
    Ignored,
}

impl VectorShapeTool {
    /// Fresh tool.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Currently selected sub-mode.
    #[must_use]
    pub fn kind(&self) -> ShapeKind {
        self.kind
    }

    /// Switch sub-mode (panel radio / programmatic).
    pub fn set_kind(&mut self, kind: ShapeKind) {
        self.kind = kind;
    }

    /// `true` while a drag is in progress.
    #[must_use]
    pub fn has_in_progress_shape(&self) -> bool {
        self.anchor.is_some()
    }

    /// Document style table (the bridge resolves fill / stroke refs).
    #[must_use]
    pub fn current_styles(&self) -> &StyleTable {
        &self.styles
    }

    /// Drain the most-recently-committed asset, if any.
    pub fn take_committed_asset(&mut self) -> Option<Ph2dVectorAsset> {
        self.pending_committed.take()
    }

    /// Inject a fill color (closed shapes) from the color picker.
    pub fn set_default_fill(&mut self, color: ph2d_color::OklchColor) {
        if let Some(slot) = self.styles.fills.get_mut(&self.fill_ref) {
            slot.color = color;
        }
    }

    /// Inject a stroke color + width (spiral) from the color picker.
    pub fn set_default_stroke(&mut self, color: ph2d_color::OklchColor, width: f32) {
        if let Some(slot) = self.styles.strokes.get_mut(&self.stroke_ref) {
            slot.color = color;
            slot.width = width.max(0.0);
        }
    }

    /// Begin a drag at `pos`. Rejects non-finite / out-of-range coords.
    pub fn begin_shape(&mut self, pos: Vec2) -> ShapeOutcome {
        if !Self::coord_is_sane(pos) {
            self.anchor = None;
            return ShapeOutcome::Ignored;
        }
        self.anchor = Some(pos);
        self.current = pos;
        ShapeOutcome::Started
    }

    /// Update the drag end-point.
    pub fn update_shape(&mut self, pos: Vec2) -> ShapeOutcome {
        if self.anchor.is_none() || !Self::coord_is_sane(pos) {
            return ShapeOutcome::Ignored;
        }
        self.current = pos;
        ShapeOutcome::Updated
    }

    /// Finish + commit the shape. Returns [`ShapeOutcome::TooSmall`] for a
    /// click-sized drag. Always returns to idle.
    pub fn finish_shape(&mut self) -> ShapeOutcome {
        let built = self.build_styled_network();
        self.anchor = None;
        let Some(network) = built else {
            return ShapeOutcome::TooSmall;
        };
        let edit_log = geometry_edit_log(&network);
        let mut asset = Ph2dVectorAsset::from_network(network, self.styles.clone());
        asset.edit_log = edit_log;
        self.pending_committed = Some(asset);
        ShapeOutcome::Committed
    }

    /// Abandon the in-progress drag (Esc / cancel).
    pub fn cancel_shape(&mut self) {
        self.anchor = None;
    }

    /// The live preview network for the current drag (styled), or `None`
    /// if not dragging / drag too small. The bridge renders this each
    /// frame via `draw_vector_network`.
    #[must_use]
    pub fn preview_network(&self) -> Option<VectorNetwork> {
        self.build_styled_network()
    }

    /// Build the styled network for the current `anchor → current` drag,
    /// or `None` if idle / non-finite / smaller than [`MIN_DRAG_DIST_PX`].
    fn build_styled_network(&self) -> Option<VectorNetwork> {
        let anchor = self.anchor?;
        let current = self.current;
        if !Self::coord_is_sane(current) {
            return None;
        }
        let delta = current - anchor;
        if delta.length() < MIN_DRAG_DIST_PX {
            return None;
        }
        let rotation = delta.y.atan2(delta.x);
        let radius = delta.length();
        let net = match self.kind {
            ShapeKind::Rect => primitives::rect(anchor, current),
            ShapeKind::Ellipse => {
                let center = (anchor + current) * 0.5;
                let radii = (current - anchor).abs() * 0.5;
                primitives::ellipse(center, radii)
            }
            ShapeKind::Polygon => primitives::polygon(anchor, radius, self.polygon_sides, rotation),
            ShapeKind::Star => primitives::star(
                anchor,
                radius,
                radius * self.star_inner_ratio,
                self.star_points,
                rotation,
            ),
            ShapeKind::Spiral => primitives::spiral(
                anchor,
                0.0,
                radius,
                self.spiral_turns,
                self.spiral_samples_per_turn,
                rotation,
            ),
        };
        Some(self.apply_style(net))
    }

    /// Assign fill (closed shapes) or stroke (open spiral) from the seeded
    /// style refs. The rule mirrors the renderer's: a region → fill; a
    /// segment with a `style_ref` → stroke.
    fn apply_style(&self, mut net: VectorNetwork) -> VectorNetwork {
        if net.regions.is_empty() {
            for seg in net.segments.iter_mut() {
                seg.style_ref = Some(self.stroke_ref);
            }
        } else {
            for region in net.regions.iter_mut() {
                region.fill = Some(self.fill_ref);
            }
        }
        net
    }

    fn coord_is_sane(p: Vec2) -> bool {
        p.x.is_finite()
            && p.y.is_finite()
            && p.x.abs() <= MAX_COORD_MAGNITUDE
            && p.y.abs() <= MAX_COORD_MAGNITUDE
    }
}

/// Synthesize a replay-safe geometry [`EditLog`] from an already-built
/// network: AddVertex / AddSegment / AddRegion / SetRegionFill. Each op is
/// network-applicable, so replaying the log from genesis reconstructs the
/// same geometry (the foundation for T2.5 undo). Stroke `style_ref` is
/// materialized on the network (SetStrokeStyle needs asset context absent
/// until T2.5), mirroring the Pencil.
fn geometry_edit_log(net: &VectorNetwork) -> EditLog {
    let mut log = EditLog::new();
    for v in &net.vertices {
        log.push(VectorOp::AddVertex {
            id: v.id,
            pos: v.pos,
            kind: v.kind,
        });
    }
    for s in &net.segments {
        log.push(VectorOp::AddSegment {
            id: s.id,
            start: s.start,
            end: s.end,
            tangents: TangentsCubic {
                out_at_start: s.out_at_start,
                in_at_end: s.in_at_end,
            },
        });
    }
    for r in &net.regions {
        log.push(VectorOp::AddRegion {
            id: r.id,
            segments: r.segments.clone(),
            winding: r.winding,
        });
        if let Some(fill) = r.fill {
            log.push(VectorOp::SetRegionFill {
                id: r.id,
                fill: Some(fill),
            });
        }
    }
    log
}

impl Tool for VectorShapeTool {
    fn id(&self) -> ToolId {
        ToolId::new("vector_shape")
    }

    fn label(&self) -> &str {
        "Vector Shape"
    }

    fn icon_slug(&self) -> &str {
        "vector-shape"
    }

    fn build_panel(&self) -> FloatingPanel {
        let options: Vec<RadioOption<String>> = ShapeKind::ALL
            .iter()
            .map(|k| RadioOption {
                value: k.value().to_string(),
                label: k.label().to_string(),
                id: k.option_id(),
            })
            .collect();
        let mut radio = RadioGroup::new(SHAPE_KIND_NODE, "Shape", options);
        radio.select(self.kind.value().to_string());

        let mut panel = FloatingPanel::new(self.id(), "Vector Shape")
            .with_tabs(vec![PanelTab {
                label: "Shape".into(),
                icon: None,
                active: true,
            }])
            .with_controls(vec![PanelControl::RadioGroup(radio)]);
        panel.anchor = PanelAnchor::BottomCenter;
        panel
    }

    fn on_activate(&mut self) {
        self.anchor = None;
    }

    fn on_deactivate(&mut self) {
        // Drop the in-progress drag; preserve `pending_committed` for the
        // bridge to drain (mirror of the Pen / Pencil).
        self.anchor = None;
    }

    fn handle_panel_event(&mut self, event: PanelEvent) {
        if let PanelEvent::SelectOption(id, value) = event
            && id == SHAPE_KIND_NODE
            && let Some(kind) = ShapeKind::from_value(&value)
        {
            self.kind = kind;
        }
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn drag_commit(tool: &mut VectorShapeTool, a: Vec2, b: Vec2) -> Ph2dVectorAsset {
        tool.begin_shape(a);
        tool.update_shape(b);
        assert_eq!(tool.finish_shape(), ShapeOutcome::Committed);
        tool.take_committed_asset().expect("committed asset")
    }

    #[test]
    fn fresh_tool_is_rect_idle_with_seeded_styles() {
        let t = VectorShapeTool::new();
        assert_eq!(t.kind(), ShapeKind::Rect);
        assert!(!t.has_in_progress_shape());
        assert_eq!(t.current_styles().fills.len(), 1);
        assert_eq!(t.current_styles().strokes.len(), 1);
    }

    #[test]
    fn rect_drag_commits_filled_region() {
        let mut t = VectorShapeTool::new();
        let asset = drag_commit(&mut t, Vec2::new(10.0, 10.0), Vec2::new(110.0, 60.0));
        assert_eq!(asset.network.regions.len(), 1);
        assert_eq!(asset.network.vertices.len(), 4);
        assert_eq!(asset.network.regions[0].fill, Some(0));
        assert!(asset.network.validate().is_ok());
    }

    #[test]
    fn each_kind_commits_and_validates() {
        for kind in ShapeKind::ALL {
            let mut t = VectorShapeTool::new();
            t.set_kind(kind);
            let asset = drag_commit(&mut t, Vec2::new(50.0, 50.0), Vec2::new(120.0, 90.0));
            assert!(
                asset.network.validate().is_ok(),
                "{kind:?} produced an invalid network"
            );
            if kind == ShapeKind::Spiral {
                assert!(asset.network.regions.is_empty(), "spiral is open");
                // Open stroked: every segment carries the stroke ref.
                assert!(
                    asset
                        .network
                        .segments
                        .iter()
                        .all(|s| s.style_ref == Some(0))
                );
            } else {
                assert_eq!(asset.network.regions.len(), 1, "{kind:?} is closed");
                assert_eq!(asset.network.regions[0].fill, Some(0));
            }
        }
    }

    #[test]
    fn committed_edit_log_replays_to_the_same_geometry() {
        let mut t = VectorShapeTool::new();
        t.set_kind(ShapeKind::Polygon);
        let asset = drag_commit(&mut t, Vec2::ZERO, Vec2::new(40.0, 0.0));
        let mut replay = VectorNetwork::empty();
        for op in &asset.edit_log.ops {
            assert!(
                op.apply_to_network(&mut replay).is_ok(),
                "non-applicable op"
            );
        }
        assert_eq!(replay.vertices.len(), asset.network.vertices.len());
        assert_eq!(replay.segments.len(), asset.network.segments.len());
        assert_eq!(replay.regions.len(), asset.network.regions.len());
        assert_eq!(replay.regions[0].fill, asset.network.regions[0].fill);
    }

    #[test]
    fn click_sized_drag_commits_nothing() {
        let mut t = VectorShapeTool::new();
        t.begin_shape(Vec2::new(5.0, 5.0));
        t.update_shape(Vec2::new(5.5, 5.0)); // < MIN_DRAG_DIST_PX
        assert_eq!(t.finish_shape(), ShapeOutcome::TooSmall);
        assert!(t.take_committed_asset().is_none());
    }

    #[test]
    fn preview_tracks_the_drag() {
        let mut t = VectorShapeTool::new();
        assert!(t.preview_network().is_none(), "idle = no preview");
        t.begin_shape(Vec2::ZERO);
        t.update_shape(Vec2::new(100.0, 40.0));
        let p = t.preview_network().expect("preview while dragging");
        assert_eq!(p.regions.len(), 1);
        assert_eq!(p.regions[0].fill, Some(0), "preview is styled");
    }

    #[test]
    fn panel_radio_switches_kind() {
        let mut t = VectorShapeTool::new();
        <VectorShapeTool as Tool>::handle_panel_event(
            &mut t,
            PanelEvent::SelectOption(SHAPE_KIND_NODE, "star".to_string()),
        );
        assert_eq!(t.kind(), ShapeKind::Star);
        // A foreign node id is ignored.
        <VectorShapeTool as Tool>::handle_panel_event(
            &mut t,
            PanelEvent::SelectOption(NodeId(999), "rect".to_string()),
        );
        assert_eq!(t.kind(), ShapeKind::Star);
    }

    #[test]
    fn panel_lists_five_sub_modes_with_active_selection() {
        let mut t = VectorShapeTool::new();
        t.set_kind(ShapeKind::Spiral);
        let panel = t.build_panel();
        let PanelControl::RadioGroup(rg) = &panel.controls[0] else {
            panic!("expected a RadioGroup control");
        };
        assert_eq!(rg.options.len(), 5);
        assert_eq!(rg.selected.as_deref(), Some("spiral"));
    }

    #[test]
    fn cancel_drops_in_progress() {
        let mut t = VectorShapeTool::new();
        t.begin_shape(Vec2::ZERO);
        t.update_shape(Vec2::new(50.0, 50.0));
        t.cancel_shape();
        assert!(!t.has_in_progress_shape());
        assert!(t.preview_network().is_none());
    }

    #[test]
    fn nan_begin_is_ignored() {
        let mut t = VectorShapeTool::new();
        assert_eq!(
            t.begin_shape(Vec2::new(f32::NAN, 0.0)),
            ShapeOutcome::Ignored
        );
        assert!(!t.has_in_progress_shape());
    }

    #[test]
    fn set_default_fill_and_stroke_reach_committed_asset() {
        use ph2d_color::OklchColor;
        let custom = OklchColor::opaque(0.7, 0.2, 30.0);
        // Closed shape → fill.
        let mut t = VectorShapeTool::new();
        t.set_default_fill(custom);
        let asset = drag_commit(&mut t, Vec2::ZERO, Vec2::new(50.0, 50.0));
        assert_eq!(asset.styles.fills.values().next().unwrap().color, custom);
        // Spiral → stroke.
        let mut t2 = VectorShapeTool::new();
        t2.set_kind(ShapeKind::Spiral);
        t2.set_default_stroke(custom, 4.0);
        let asset2 = drag_commit(&mut t2, Vec2::ZERO, Vec2::new(60.0, 0.0));
        let stroke = asset2.styles.strokes.values().next().unwrap();
        assert_eq!(stroke.color, custom);
        assert_eq!(stroke.width, 4.0);
    }

    #[test]
    fn tool_id_label_icon_are_stable() {
        let t = VectorShapeTool::new();
        assert_eq!(t.id(), ToolId::new("vector_shape"));
        assert_eq!(t.label(), "Vector Shape");
        assert_eq!(t.icon_slug(), "vector-shape");
    }
}
