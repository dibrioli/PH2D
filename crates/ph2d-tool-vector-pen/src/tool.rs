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

    /// Replace the default region fill with `color`. The shell bridge
    /// calls this on tool activation (or after a Color Picker change)
    /// so newly-closed paths get a visible, user-chosen color instead
    /// of the [`FillSolid::default`] mid-gray placeholder (R1 Lens-R
    /// MED-G3 — placeholder was invisible against typical dark/light
    /// canvas backgrounds).
    ///
    /// Mutates the seeded fill entry in-place (preserves
    /// `default_fill_ref` so any already-issued region keeps pointing
    /// at the right slot).
    pub fn set_default_fill(&mut self, color: ph2d_color::OklchColor) {
        if let Some(slot) = self.styles.fills.get_mut(&self.default_fill_ref) {
            slot.color = color;
        }
    }

    /// `true` if a path is currently being authored (≥ 1 vertex added,
    /// no close-path yet). T1.7 bridge uses this to decide whether to
    /// emit a destructive-deactivate Toast warning (mirror Painter's
    /// `has_painted_since_source`).
    #[must_use]
    pub fn has_in_progress_path(&self) -> bool {
        !self.network.vertices.is_empty()
    }

    /// Number of vertices in the in-progress path. Useful for UI
    /// counters ("Click N more or click the first vertex to close").
    #[must_use]
    pub fn vertex_count(&self) -> usize {
        self.network.vertices.len()
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
    ///
    /// ## Coordinate space contract (T1.7 bridge responsibility)
    ///
    /// `pos` is expected in **network-local pixel coordinates** — the
    /// same coordinate space the vertices are stored in (per
    /// `ph2d_vector_doc::Vertex` doc-comment). The shell bridge must
    /// convert screen → network-local via the active sprite's affine
    /// before calling this method. Mirror the Painter precedent in
    /// `shells/desktop/src/input_dispatch/painter_input.rs::painter_pointer_uv`.
    ///
    /// ## Defense against malformed input
    ///
    /// `pos` containing `NaN` or `Infinity` is rejected outright —
    /// returning `Rejected` instead of silently corrupting the network
    /// (e.g. a NaN slipping into `nearest_vertex` propagates through
    /// IEEE 754 comparisons and would spuriously trigger close-path —
    /// R1 Lens-M CRIT-L-M-1).
    pub fn on_canvas_click(&mut self, pos: Vec2) -> PenClickOutcome {
        // R1 Lens-M CRIT-L-M-1: NaN/Inf sanitize. Any pointer source
        // (driver bug, divide-by-zero in zoom→world transform,
        // `device_pixel_ratio = 0` on a docked panel) that produces a
        // non-finite coord must NOT touch the network.
        if !pos.x.is_finite() || !pos.y.is_finite() {
            return PenClickOutcome::Rejected;
        }

        // Defensive safety cap — refuses to grow indefinitely if the
        // user clicks without ever closing the path.
        if self.network.vertices.len() >= MAX_IN_PROGRESS_VERTICES {
            return PenClickOutcome::Rejected;
        }

        // Close-path detection + non-first-vertex hover.
        // - Click within tolerance of vertex 0 (the start) AND ≥ 3
        //   vertices + ≥ 1 segment → close the path.
        // - Click within tolerance of vertex N≠0 → NO-OP (UX:
        //   "click missed an existing vertex; nothing happens" — vs
        //   the original behavior of silently adding a duplicate
        //   vertex on top per R1 Lens-M HIGH-L-M-3).
        if !self.network.vertices.is_empty()
            && let Some(near) = self
                .network
                .nearest_vertex(pos, self.close_path_tolerance_px)
        {
            let first_id = self.network.vertices[0].id;
            if near == first_id
                && self.network.vertices.len() >= 3
                && !self.network.segments.is_empty()
            {
                self.close_current_path();
                return PenClickOutcome::ClosedPath;
            }
            // Close-but-not-first-vertex: no-op. Bridge may emit a
            // subtle UI cue ("snap-back disabled" toast) — that's UX
            // for T1.7, not data state.
            if near != first_id {
                return PenClickOutcome::NoOpNearExistingVertex;
            }
            // Else: near first vertex but path too short to close —
            // fall through and let the click add the next vertex
            // (the user is still building up the path).
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
    /// Click was rejected (cap exceeded, `NaN`/`Infinity` coords,
    /// or apply failed silently). UI may emit a toast.
    Rejected,
    /// Click landed within `close_path_tolerance_px` of an existing
    /// vertex that is NOT the path's start vertex. No mutation
    /// performed (avoids duplicate-vertex pollution per R1 Lens-M
    /// HIGH-L-M-3). UI may show a "snap target" hover indicator.
    NoOpNearExistingVertex,
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
        // Drop the in-progress path (volatile authoring state). But
        // **preserve `pending_committed`** — that asset was already
        // commit-emitted by the user's close-path click and is owed
        // to the bridge; clearing it would silently delete user data
        // if the bridge hasn't drained yet (R1 Lens-M HIGH-L-M-2).
        // T1.7 bridge MUST call `take_committed_asset()` post-
        // deactivate to drain any leftover commit before the next
        // tool activates.
        self.reset_path();
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
        // 3rd click AT origin would normally close, but with only 2
        // vertices the close-path guard (≥ 3) fails. Post R1 audit:
        // the click is near vertex 0 (the start), so it falls through
        // to the "near start but path too short to close" path and
        // adds the 3rd vertex (ExtendedPath).
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
    fn on_deactivate_clears_in_progress_but_preserves_pending_committed() {
        // R1 Lens-M HIGH-L-M-2 fix: pending_committed is user data
        // the bridge owes to drain — must survive on_deactivate so
        // the bridge can pick it up on the next tick.
        let mut t = VectorPenTool::new();
        t.on_canvas_click(Vec2::new(0.0, 0.0));
        t.on_canvas_click(Vec2::new(100.0, 0.0));
        t.on_canvas_click(Vec2::new(50.0, 86.6));
        t.on_canvas_click(Vec2::new(1.0, 1.0));
        // pending_committed populated.
        <VectorPenTool as Tool>::on_deactivate(&mut t);
        // In-progress reset:
        assert_eq!(t.current_network().vertices.len(), 0);
        // Pending preserved:
        assert!(
            t.take_committed_asset().is_some(),
            "pending_committed must survive on_deactivate for bridge to drain"
        );
    }

    #[test]
    fn on_deactivate_with_no_pending_does_not_panic() {
        let mut t = VectorPenTool::new();
        t.on_canvas_click(Vec2::new(0.0, 0.0));
        t.on_canvas_click(Vec2::new(10.0, 0.0)); // in-progress, no close
        <VectorPenTool as Tool>::on_deactivate(&mut t);
        assert!(t.take_committed_asset().is_none());
        assert_eq!(t.current_network().vertices.len(), 0);
    }

    #[test]
    fn nan_pos_is_rejected_without_corrupting_state() {
        // R1 Lens-M CRITICAL-L-M-1 fix: NaN/Inf coords would otherwise
        // propagate through IEEE 754 comparisons in `nearest_vertex`
        // and spuriously trigger close-path / silent asset corruption.
        let mut t = VectorPenTool::new();
        t.on_canvas_click(Vec2::new(0.0, 0.0));
        t.on_canvas_click(Vec2::new(100.0, 0.0));
        t.on_canvas_click(Vec2::new(50.0, 86.6));
        let initial_vertex_count = t.vertex_count();
        let nan_out = t.on_canvas_click(Vec2::new(f32::NAN, 0.0));
        assert_eq!(nan_out, PenClickOutcome::Rejected);
        let inf_out = t.on_canvas_click(Vec2::new(0.0, f32::INFINITY));
        assert_eq!(inf_out, PenClickOutcome::Rejected);
        let neg_inf_out = t.on_canvas_click(Vec2::new(f32::NEG_INFINITY, 0.0));
        assert_eq!(neg_inf_out, PenClickOutcome::Rejected);
        // Network unchanged + no spurious commit.
        assert_eq!(t.vertex_count(), initial_vertex_count);
        assert!(t.take_committed_asset().is_none());
    }

    #[test]
    fn click_near_non_first_vertex_is_noop_not_duplicate() {
        // R1 Lens-M HIGH-L-M-3 fix: prior behavior added a duplicate
        // vertex colocated with the existing one. Now it's a no-op.
        let mut t = VectorPenTool::new();
        t.on_canvas_click(Vec2::new(0.0, 0.0)); // vertex 0
        t.on_canvas_click(Vec2::new(100.0, 0.0)); // vertex 1
        t.on_canvas_click(Vec2::new(50.0, 86.6)); // vertex 2
        let before = t.vertex_count();
        // Click near vertex 1 (≠ first). Distance < 12px tolerance.
        let out = t.on_canvas_click(Vec2::new(101.0, 1.0));
        assert_eq!(out, PenClickOutcome::NoOpNearExistingVertex);
        assert_eq!(t.vertex_count(), before, "no vertex added");
        assert!(t.take_committed_asset().is_none(), "no spurious commit");
    }

    #[test]
    fn committed_asset_preserves_authoring_hint_set_pre_close() {
        let mut t = VectorPenTool::new();
        t.set_authoring_hint(RepresentationMode::SpiroAssist);
        t.on_canvas_click(Vec2::new(0.0, 0.0));
        t.on_canvas_click(Vec2::new(100.0, 0.0));
        t.on_canvas_click(Vec2::new(50.0, 86.6));
        t.on_canvas_click(Vec2::new(1.0, 1.0));
        let asset = t.take_committed_asset().expect("committed");
        assert_eq!(
            asset.network.authoring_hint,
            RepresentationMode::SpiroAssist,
            "authoring_hint set pre-close must survive into committed asset"
        );
    }

    #[test]
    fn committed_asset_edit_log_has_exact_op_sequence() {
        // 3 AddVertex + 2 AddSegment (during extend) + 1 AddSegment
        // (closing) + 1 AddRegion + 1 SetRegionFill = 8 ops total.
        let mut t = VectorPenTool::new();
        t.on_canvas_click(Vec2::new(0.0, 0.0));
        t.on_canvas_click(Vec2::new(100.0, 0.0));
        t.on_canvas_click(Vec2::new(50.0, 86.6));
        t.on_canvas_click(Vec2::new(1.0, 1.0));
        let asset = t.take_committed_asset().expect("committed");
        assert_eq!(
            asset.edit_log.ops.len(),
            8,
            "expected 3 AddVertex + 3 AddSegment + 1 AddRegion + 1 SetRegionFill = 8 ops"
        );
        assert!(matches!(asset.edit_log.ops[0], VectorOp::AddVertex { .. }));
        assert!(matches!(
            asset.edit_log.ops[7],
            VectorOp::SetRegionFill { .. }
        ));
    }

    #[test]
    fn second_path_after_close_starts_fresh() {
        let mut t = VectorPenTool::new();
        // First path: close.
        t.on_canvas_click(Vec2::new(0.0, 0.0));
        t.on_canvas_click(Vec2::new(100.0, 0.0));
        t.on_canvas_click(Vec2::new(50.0, 86.6));
        t.on_canvas_click(Vec2::new(1.0, 1.0));
        assert!(t.take_committed_asset().is_some());
        // Second path: starts at vertex 0 again (post-reset).
        let out = t.on_canvas_click(Vec2::new(200.0, 200.0));
        assert_eq!(out, PenClickOutcome::AddedFirstVertex);
        assert_eq!(t.vertex_count(), 1);
    }

    #[test]
    fn set_default_fill_replaces_seeded_color() {
        // R1 Lens-R MED-G3 fix: bridge can inject visible color
        // before close-path so triangle is rendered with user color.
        use ph2d_color::OklchColor;
        let mut t = VectorPenTool::new();
        let custom = OklchColor::opaque(0.7, 0.18, 250.0); // bluish
        t.set_default_fill(custom);
        t.on_canvas_click(Vec2::new(0.0, 0.0));
        t.on_canvas_click(Vec2::new(100.0, 0.0));
        t.on_canvas_click(Vec2::new(50.0, 86.6));
        t.on_canvas_click(Vec2::new(1.0, 1.0));
        let asset = t.take_committed_asset().expect("committed");
        let fill = asset
            .styles
            .fills
            .values()
            .next()
            .expect("at least one fill");
        assert_eq!(fill.color, custom);
    }

    #[test]
    fn has_in_progress_path_and_vertex_count_helpers() {
        let mut t = VectorPenTool::new();
        assert!(!t.has_in_progress_path());
        assert_eq!(t.vertex_count(), 0);
        t.on_canvas_click(Vec2::new(0.0, 0.0));
        assert!(t.has_in_progress_path());
        assert_eq!(t.vertex_count(), 1);
        t.on_canvas_click(Vec2::new(100.0, 0.0));
        assert_eq!(t.vertex_count(), 2);
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
