//! [`VectorPenTool`] — interactive Pen authoring with cubic Bézier default.
//!
//! Per W1.T1.5. Stateful Tool with inherent `on_canvas_click` /
//! `current_network` / `take_committed_asset` methods that the T1.7
//! shell bridge calls after downcasting via [`Tool::as_any_mut`]
//! (ADR-0040 §3 documented exception, mirror of Painter T1.5
//! `PainterTool::queue_pointer`).

use glam::Vec2;
use ph2d_editor_core::floating_panel::{FloatingPanel, ToolId};
use ph2d_editor_core::tool::{PanelEvent, Tool};
use ph2d_vector_doc::{
    EditLog, FillSolid, Ph2dVectorAsset, RepresentationMode, StyleTable, TangentsCubic,
    VectorNetwork, VectorOp, VertexKind, WindingRule,
};

/// Default pixel tolerance for close-path detection. Tuned for typical
/// 1× DPI desktop; T1.7 may scale by device pixel ratio.
pub const DEFAULT_CLOSE_PATH_TOLERANCE_PX: f32 = 12.0;

/// Maximum vertices a single in-progress path may accumulate before the
/// tool refuses additional clicks (defensive — keeps the tool well
/// under the `AssetBounds::max_vertices = 100_000` global cap even if
/// the user clicks erratically).
pub const MAX_IN_PROGRESS_VERTICES: usize = 2_048;

/// Stateful Vector Pen tool.
///
/// Holds an in-progress [`VectorNetwork`] + [`EditLog`] + [`StyleTable`]
/// that the shell bridge reads each frame for preview rendering. On
/// close-path the tool builds a [`Ph2dVectorAsset`] via
/// [`Ph2dVectorAsset::from_network`] and exposes it via
/// [`Self::take_committed_asset`] for one call (drains).
#[derive(Debug, Clone)]
pub struct VectorPenTool {
    /// Live network being built. Reset by [`Self::reset_path`] after
    /// commit or [`Tool::on_deactivate`].
    network: VectorNetwork,

    /// Append-only log of operations applied to `network`. Survives
    /// commit so the asset round-trips with full provenance.
    edit_log: EditLog,

    /// Document style table. Pre-seeded with a single default fill at
    /// `FillRef = 0` on construction (used for every region the tool
    /// creates until the inspector W2 lets the user choose fills).
    styles: StyleTable,

    /// `FillRef` assigned to every region the tool closes.
    default_fill_ref: u32,

    /// Pixel distance under which a new click counts as "close to the
    /// first vertex" → triggers path-close.
    pub close_path_tolerance_px: f32,

    /// Asset emitted by the most recent close-path, awaiting bridge
    /// drain. `Some` for exactly one [`Self::take_committed_asset`]
    /// call after a close-path event.
    pending_committed: Option<Ph2dVectorAsset>,

    /// Authoring representation hint. Cubic = default visible per
    /// ADR-0056 §2.4. Spiro / Hyperbezier Assist Modes toggle via
    /// [`Self::set_authoring_hint`] (W2 wires the HUD `S` / `H` keys).
    authoring_hint: RepresentationMode,
}

impl Default for VectorPenTool {
    fn default() -> Self {
        let mut styles = StyleTable::default();
        let default_fill_ref = styles.insert_fill(FillSolid::default());
        Self {
            network: VectorNetwork::empty(),
            edit_log: EditLog::new(),
            styles,
            default_fill_ref,
            close_path_tolerance_px: DEFAULT_CLOSE_PATH_TOLERANCE_PX,
            pending_committed: None,
            authoring_hint: RepresentationMode::Cubic,
        }
    }
}

impl VectorPenTool {
    /// Construct a fresh Pen tool with default state.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Read-only handle on the in-progress network — the T1.7 shell
    /// bridge reads this per frame to render the live preview via
    /// `draw_vector_network`.
    #[must_use]
    pub fn current_network(&self) -> &VectorNetwork {
        &self.network
    }

    /// Read-only handle on the document style table (the T1.7 bridge
    /// passes both `&network` + `&styles` to `draw_vector_network`).
    #[must_use]
    pub fn current_styles(&self) -> &StyleTable {
        &self.styles
    }

    /// Drain the most-recently-committed asset, if any. Returns `Some`
    /// exactly once per close-path event.
    pub fn take_committed_asset(&mut self) -> Option<Ph2dVectorAsset> {
        self.pending_committed.take()
    }

    /// Toggle the authoring representation hint. W2 wires HUD `S` / `H`
    /// keys to flip between Cubic / SpiroAssist / HyperbezierAssist.
    pub fn set_authoring_hint(&mut self, mode: RepresentationMode) {
        self.authoring_hint = mode;
        // Apply now so the next round-trip carries the hint.
        let _ = self
            .edit_log
            .push_and_apply(VectorOp::SetAuthoringHint { mode }, &mut self.network);
    }

    /// Reset the in-progress path to empty. Called after commit + on
    /// tool deactivate. The default fill seeded on construction is
    /// re-seeded so the next path has a fill ready.
    pub fn reset_path(&mut self) {
        self.network = VectorNetwork::empty();
        self.edit_log = EditLog::new();
        self.styles = StyleTable::default();
        self.default_fill_ref = self.styles.insert_fill(FillSolid::default());
        self.authoring_hint = RepresentationMode::Cubic;
    }

    /// Main pointer handler — called by the T1.7 shell bridge each time
    /// the user clicks on the canvas.
    ///
    /// Returns the kind of event that occurred (for the bridge to drive
    /// preview invalidation / commit save / etc.).
    ///
    /// W1 ships straight-line segments only (`TangentsCubic::ZERO`).
    /// W2 adds click-and-drag tangent extrusion when pointer drag
    /// events arrive — the `TangentsCubic` data path is already
    /// present.
    pub fn on_canvas_click(&mut self, pos: Vec2) -> PenClickOutcome {
        // Defensive safety cap — refuses to grow indefinitely if the
        // user clicks without ever closing the path.
        if self.network.vertices.len() >= MAX_IN_PROGRESS_VERTICES {
            return PenClickOutcome::Rejected;
        }

        // Close-path detection: is the click within tolerance of the
        // FIRST vertex of the current path? Needs ≥ 3 vertices + at
        // least one segment to make a meaningful triangle.
        if self.network.vertices.len() >= 3 && !self.network.segments.is_empty() {
            let first_id = self.network.vertices[0].id;
            if let Some(near) = self
                .network
                .nearest_vertex(pos, self.close_path_tolerance_px)
                && near == first_id
            {
                self.close_current_path();
                return PenClickOutcome::ClosedPath;
            }
        }

        // Otherwise: add a vertex (+ a segment from the previous one,
        // if any).
        let new_vertex_id = self.network.next_vertex_id();
        if self
            .edit_log
            .push_and_apply(
                VectorOp::AddVertex {
                    id: new_vertex_id,
                    pos,
                    kind: VertexKind::Auto,
                },
                &mut self.network,
            )
            .is_err()
        {
            return PenClickOutcome::Rejected;
        }

        // If this is NOT the first vertex, connect from the previous
        // one with a straight cubic (zero tangents = degenerate-cubic
        // straight line; kurbo / Vello handle correctly).
        let connect = self.network.vertices.len() >= 2;
        if connect {
            let prev_id = self.network.vertices[self.network.vertices.len() - 2].id;
            let new_seg_id = self.network.next_segment_id();
            let _ = self.edit_log.push_and_apply(
                VectorOp::AddSegment {
                    id: new_seg_id,
                    start: prev_id,
                    end: new_vertex_id,
                    tangents: TangentsCubic::ZERO,
                },
                &mut self.network,
            );
            PenClickOutcome::ExtendedPath
        } else {
            PenClickOutcome::AddedFirstVertex
        }
    }

    /// Close the current path: connect last vertex back to the first
    /// via a final segment, build a region wrapping every segment in
    /// traversal order, assign `default_fill_ref`, and emit the asset
    /// to `pending_committed`.
    fn close_current_path(&mut self) {
        if self.network.vertices.len() < 3 {
            return;
        }
        let first_id = self.network.vertices[0].id;
        let last_id = self.network.vertices[self.network.vertices.len() - 1].id;

        // Closing segment last → first.
        let close_seg_id = self.network.next_segment_id();
        if self
            .edit_log
            .push_and_apply(
                VectorOp::AddSegment {
                    id: close_seg_id,
                    start: last_id,
                    end: first_id,
                    tangents: TangentsCubic::ZERO,
                },
                &mut self.network,
            )
            .is_err()
        {
            return;
        }

        // Region wraps every existing segment in insertion order
        // (forward traversal). For a 3-vertex triangle: segments
        // 0→1, 1→2, 2→0 — all forward, perfect winding-rule loop.
        let region_id = self.network.next_region_id();
        let segment_refs: smallvec::SmallVec<[(u32, bool); 16]> =
            self.network.segments.iter().map(|s| (s.id, true)).collect();
        let _ = self.edit_log.push_and_apply(
            VectorOp::AddRegion {
                id: region_id,
                segments: segment_refs,
                winding: WindingRule::NonZero,
            },
            &mut self.network,
        );
        let _ = self.edit_log.push_and_apply(
            VectorOp::SetRegionFill {
                id: region_id,
                fill: Some(self.default_fill_ref),
            },
            &mut self.network,
        );

        // Snapshot the committed state. The asset carries the
        // authoring_hint via VectorNetwork field; edit_log preserves
        // the full sequence so undo / replay works.
        let mut asset = Ph2dVectorAsset::from_network(self.network.clone(), self.styles.clone());
        asset.edit_log = self.edit_log.clone();
        self.pending_committed = Some(asset);

        // Reset for the next path. The bridge drains pending_committed
        // before the next click cycle.
        self.reset_path();
    }
}

/// Outcome reported by [`VectorPenTool::on_canvas_click`] so the shell
/// bridge can drive UI feedback (preview invalidation / commit-save
/// trigger / toast on rejection).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PenClickOutcome {
    /// First click in a new path — only vertex 0 was added, no segment yet.
    AddedFirstVertex,
    /// Subsequent click — vertex + connecting segment added.
    ExtendedPath,
    /// Click triggered close-path → asset committed; drain via
    /// [`VectorPenTool::take_committed_asset`].
    ClosedPath,
    /// Click was rejected (e.g. hit the safety cap, or apply failed
    /// silently). UI may emit a toast.
    Rejected,
}

impl Tool for VectorPenTool {
    fn id(&self) -> ToolId {
        ToolId::new("vector_pen")
    }

    fn label(&self) -> &str {
        "Vector Pen"
    }

    fn icon_slug(&self) -> &str {
        // Hyphen kept for SVG/Lucide convention — diff from id
        // ("vector_pen", snake_case for HR-15 i18n key). Same split
        // pattern as bgremoval (id "bgremoval" / slug "bg-removal").
        "vector-pen"
    }

    fn build_panel(&self) -> FloatingPanel {
        // W1 stub: panel vazio com title só. T1.7 shell bridge mostra
        // o triângulo preview no canvas (não no panel). W2 / W15 (Tool
        // Studio) instalam o panel real com sliders (Spiro tension,
        // tangent-symmetry mode, etc.).
        FloatingPanel::new(self.id(), "Vector Pen")
    }

    fn on_activate(&mut self) {
        // Clear any stale state from a prior session.
        self.reset_path();
    }

    fn on_deactivate(&mut self) {
        // Drop any in-progress path silently. T1.7 may want to emit a
        // Toast warning if the user had > 1 vertex but didn't close —
        // that's bridge UX, not tool responsibility.
        self.reset_path();
        self.pending_committed = None;
    }

    fn handle_panel_event(&mut self, _event: PanelEvent) {
        // No panel widgets in W1 — Vector Pen is canvas-driven only.
        // W2 / Tool Studio adds Spiro tension sliders, tangent symmetry
        // toggle, etc., wired here.
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fresh_tool_has_empty_network_and_one_seeded_fill() {
        let t = VectorPenTool::new();
        assert_eq!(t.current_network().vertices.len(), 0);
        assert_eq!(t.current_network().segments.len(), 0);
        assert_eq!(t.current_styles().fills.len(), 1);
    }

    #[test]
    fn first_click_adds_vertex_no_segment() {
        let mut t = VectorPenTool::new();
        let out = t.on_canvas_click(Vec2::new(10.0, 10.0));
        assert_eq!(out, PenClickOutcome::AddedFirstVertex);
        assert_eq!(t.current_network().vertices.len(), 1);
        assert_eq!(t.current_network().segments.len(), 0);
    }

    #[test]
    fn second_click_adds_vertex_plus_segment() {
        let mut t = VectorPenTool::new();
        t.on_canvas_click(Vec2::new(0.0, 0.0));
        let out = t.on_canvas_click(Vec2::new(10.0, 0.0));
        assert_eq!(out, PenClickOutcome::ExtendedPath);
        assert_eq!(t.current_network().vertices.len(), 2);
        assert_eq!(t.current_network().segments.len(), 1);
        assert_eq!(t.current_network().segments[0].start, 0);
        assert_eq!(t.current_network().segments[0].end, 1);
    }

    #[test]
    fn three_clicks_then_close_yields_committed_triangle_asset() {
        let mut t = VectorPenTool::new();
        t.on_canvas_click(Vec2::new(0.0, 0.0));
        t.on_canvas_click(Vec2::new(100.0, 0.0));
        t.on_canvas_click(Vec2::new(50.0, 86.6));
        // 4th click NEAR vertex 0 → close-path.
        let out = t.on_canvas_click(Vec2::new(1.0, 1.0));
        assert_eq!(out, PenClickOutcome::ClosedPath);

        let asset = t.take_committed_asset().expect("committed asset");
        assert_eq!(asset.network.vertices.len(), 3);
        assert_eq!(asset.network.segments.len(), 3, "3 segments closing loop");
        assert_eq!(asset.network.regions.len(), 1, "1 region wrapping triangle");
        assert_eq!(asset.network.regions[0].fill, Some(0));
        assert_eq!(asset.styles.fills.len(), 1);
        assert!(asset.network.validate().is_ok());
    }

    #[test]
    fn closing_click_too_far_from_first_vertex_extends_instead() {
        let mut t = VectorPenTool::new();
        t.on_canvas_click(Vec2::new(0.0, 0.0));
        t.on_canvas_click(Vec2::new(100.0, 0.0));
        t.on_canvas_click(Vec2::new(50.0, 86.6));
        // Click far from origin (100 px away >> default tolerance 12).
        let out = t.on_canvas_click(Vec2::new(200.0, 200.0));
        assert_eq!(out, PenClickOutcome::ExtendedPath);
        assert!(t.take_committed_asset().is_none());
        assert_eq!(t.current_network().vertices.len(), 4);
    }

    #[test]
    fn close_path_requires_at_least_3_vertices() {
        let mut t = VectorPenTool::new();
        t.on_canvas_click(Vec2::new(0.0, 0.0));
        t.on_canvas_click(Vec2::new(10.0, 0.0));
        // 3rd click at origin would normally close, but with only 2
        // vertices the tool extends instead.
        let out = t.on_canvas_click(Vec2::new(0.0, 0.0));
        assert_eq!(out, PenClickOutcome::ExtendedPath);
        assert!(t.take_committed_asset().is_none());
    }

    #[test]
    fn take_committed_asset_drains_after_one_call() {
        let mut t = VectorPenTool::new();
        t.on_canvas_click(Vec2::new(0.0, 0.0));
        t.on_canvas_click(Vec2::new(100.0, 0.0));
        t.on_canvas_click(Vec2::new(50.0, 86.6));
        t.on_canvas_click(Vec2::new(1.0, 1.0));
        assert!(t.take_committed_asset().is_some());
        assert!(t.take_committed_asset().is_none(), "drained");
    }

    #[test]
    fn reset_path_clears_state() {
        let mut t = VectorPenTool::new();
        t.on_canvas_click(Vec2::new(0.0, 0.0));
        t.on_canvas_click(Vec2::new(10.0, 0.0));
        t.reset_path();
        assert_eq!(t.current_network().vertices.len(), 0);
        assert_eq!(t.current_network().segments.len(), 0);
    }

    #[test]
    fn on_deactivate_drops_in_progress_and_pending() {
        let mut t = VectorPenTool::new();
        t.on_canvas_click(Vec2::new(0.0, 0.0));
        t.on_canvas_click(Vec2::new(100.0, 0.0));
        t.on_canvas_click(Vec2::new(50.0, 86.6));
        t.on_canvas_click(Vec2::new(1.0, 1.0));
        // pending_committed populated.
        <VectorPenTool as Tool>::on_deactivate(&mut t);
        assert!(t.take_committed_asset().is_none());
        assert_eq!(t.current_network().vertices.len(), 0);
    }

    #[test]
    fn set_authoring_hint_round_trips_via_op() {
        let mut t = VectorPenTool::new();
        t.set_authoring_hint(RepresentationMode::SpiroAssist);
        assert_eq!(
            t.current_network().authoring_hint,
            RepresentationMode::SpiroAssist
        );
    }

    #[test]
    fn rejected_after_max_in_progress_vertices() {
        let mut t = VectorPenTool::new();
        // Force the network to the cap manually (clicking N times would
        // be slow + would also hit segment alloc which is fine but
        // wasted in this test).
        for i in 0..MAX_IN_PROGRESS_VERTICES as u32 {
            t.network
                .vertices
                .push(ph2d_vector_doc::Vertex::auto(i, Vec2::ZERO));
        }
        let out = t.on_canvas_click(Vec2::new(50.0, 50.0));
        assert_eq!(out, PenClickOutcome::Rejected);
    }

    #[test]
    fn tool_id_label_icon_match_manifest() {
        let t = VectorPenTool::new();
        assert_eq!(t.id(), ToolId::new("vector_pen"));
        assert_eq!(t.label(), "Vector Pen");
        // icon_slug uses hyphen (SVG/Lucide convention) — id stays
        // snake_case for HR-15 i18n.
        assert_eq!(t.icon_slug(), "vector-pen");
    }
}
