//! [`VectorDirectTool`] — Direct Select: vertex + tangent editing.
//!
//! Per plan **T2.3** (`docs/Vector Module/17_plano_de_implementacao.md` §5)
//! and Coord decisions (`787b6fc`): grab a vertex or a cubic tangent handle
//! of a committed network and drag it. Mutations apply **in-place** via
//! event-sourced ops ([`VectorOp::MoveVertex`] / [`VectorOp::MoveTangent`])
//! pushed to that asset's [`EditLog`] — replay-safe, the foundation for
//! T2.5 Undo (`revert_last_op`).
//!
//! The shell calls the inherent methods after downcasting via
//! [`Tool::as_any_mut`] (ADR-0040 §3 exception), passing the shared
//! committed scene (`&mut [Ph2dVectorAsset]`) + the shared
//! [`VectorSelection`]. This tool holds only the transient drag grab.
//!
//! ## Smooth handles + tangent break (DoD "tangent break funciona")
//!
//! Dragging a tangent on a **smooth** vertex ([`VertexKind::Mirror`] /
//! [`VertexKind::Aligned`]) also mirrors the opposite incident tangent.
//! **Alt-drag breaks** the link: it sets the vertex to [`VertexKind::Free`]
//! and moves only the grabbed side, so the two handles become independent.

use glam::Vec2;
use ph2d_editor_core::floating_panel::{FloatingPanel, ToolId};
use ph2d_editor_core::tool::{PanelEvent, Tool};
use ph2d_vector_doc::{
    Ph2dVectorAsset, SegmentId, TangentSide, VectorOp, VectorSelection, VertexId, VertexKind,
};

/// Default grab tolerance (world px). The shell derives this from a
/// screen-px radius / camera scale so it stays constant on screen.
pub const DEFAULT_GRAB_TOLERANCE_PX: f32 = 8.0;

/// What a Direct-Select drag is moving.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GrabTarget {
    /// A vertex (moves the whole point; incident tangents follow).
    Vertex(VertexId),
    /// A cubic tangent handle of a segment.
    Tangent(SegmentId, TangentSide),
}

/// An in-progress Direct-Select drag.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DirectGrab {
    /// Committed-scene asset index being edited.
    pub asset: usize,
    /// What's grabbed.
    pub target: GrabTarget,
}

/// Outcome of a Direct-Select interaction.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DirectOutcome {
    /// Grabbed a vertex / tangent (drag started).
    Grabbed(GrabTarget),
    /// A drag move was applied.
    Moved,
    /// Nothing under the cursor — no grab.
    Missed,
    /// No drag in progress / no-op.
    Idle,
}

/// Direct Select tool. Holds only the transient drag grab.
#[derive(Debug, Clone, Default)]
pub struct VectorDirectTool {
    grab: Option<DirectGrab>,
}

impl VectorDirectTool {
    /// Fresh tool.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// `true` while dragging a vertex / handle.
    #[must_use]
    pub fn is_dragging(&self) -> bool {
        self.grab.is_some()
    }

    /// The current grab, if any (for the overlay).
    #[must_use]
    pub fn grab(&self) -> Option<DirectGrab> {
        self.grab
    }

    /// Hit-test at `pos` and start a drag. Tangent handles are tested
    /// before vertices (they sit on top), topmost asset first. On a hit
    /// the grabbed vertex is selected. Returns [`DirectOutcome::Missed`] if
    /// nothing is within `tolerance`.
    pub fn begin_drag(
        &mut self,
        committed: &[Ph2dVectorAsset],
        selection: &mut VectorSelection,
        pos: Vec2,
        tolerance: f32,
        inv_placements: &[[f32; 6]],
    ) -> DirectOutcome {
        for (idx, asset) in committed.iter().enumerate().rev() {
            // ADR-0076: map the world cursor into THIS shape's rest frame (it may be
            // gizmo-moved), so the hit-test lands on the shape's current visual.
            let local = inv_placements
                .get(idx)
                .map_or(pos, |&m| apply_affine(m, pos));
            // Tangent handles first (smaller, drawn over vertices).
            if let Some((seg, side)) = asset.network.nearest_tangent_handle(local, tolerance) {
                self.grab = Some(DirectGrab {
                    asset: idx,
                    target: GrabTarget::Tangent(seg, side),
                });
                if let Some(vid) = anchor_vertex(asset, seg, side) {
                    selection.select_only_vertex(idx, vid);
                }
                return DirectOutcome::Grabbed(GrabTarget::Tangent(seg, side));
            }
            if let Some(vid) = asset.network.nearest_vertex(local, tolerance) {
                self.grab = Some(DirectGrab {
                    asset: idx,
                    target: GrabTarget::Vertex(vid),
                });
                selection.select_only_vertex(idx, vid);
                return DirectOutcome::Grabbed(GrabTarget::Vertex(vid));
            }
        }
        DirectOutcome::Missed
    }

    /// Apply a drag to world position `pos`. `alt` breaks a smooth tangent.
    /// No-op if not dragging or the asset index is stale.
    pub fn drag_to(
        &mut self,
        committed: &mut [Ph2dVectorAsset],
        pos: Vec2,
        alt: bool,
        inv_placements: &[[f32; 6]],
    ) -> DirectOutcome {
        let Some(grab) = self.grab else {
            return DirectOutcome::Idle;
        };
        if !(pos.x.is_finite() && pos.y.is_finite()) {
            return DirectOutcome::Idle;
        }
        // ADR-0076: write the vertex/tangent in the shape's REST frame — map the
        // world cursor back through the grabbed shape's inverse placement.
        let local = inv_placements
            .get(grab.asset)
            .map_or(pos, |&m| apply_affine(m, pos));
        let Some(asset) = committed.get_mut(grab.asset) else {
            return DirectOutcome::Idle;
        };
        match grab.target {
            GrabTarget::Vertex(vid) => {
                // Tangent offsets are relative, so they follow the vertex.
                let _ = asset.edit_log.push_and_apply(
                    VectorOp::MoveVertex {
                        id: vid,
                        new_pos: local,
                    },
                    &mut asset.network,
                );
            }
            GrabTarget::Tangent(seg, side) => {
                apply_tangent_drag(asset, seg, side, local, alt);
            }
        }
        DirectOutcome::Moved
    }

    /// Finish the drag (pointer up).
    pub fn end_drag(&mut self) {
        self.grab = None;
    }

    /// Abandon the drag (Esc) — the already-applied move stays (undo via
    /// the edit_log); only the transient grab is dropped.
    pub fn cancel_drag(&mut self) {
        self.grab = None;
    }

    /// Set the [`VertexKind`] of every selected vertex — Corner ([`VertexKind::Free`])
    /// / Smooth ([`VertexKind::Mirror`]) / Asymmetric ([`VertexKind::Aligned`]) /
    /// Auto — and re-derive its incident tangents IMMEDIATELY (eager) so the curve
    /// reshapes the moment the type is chosen, not on the next handle drag (Enio).
    /// Smooth → collinear + equal magnitude; Asymmetric → collinear, keep each
    /// length; Auto → collinear along the prev→next chord with auto length; Corner
    /// → handles left independent. Kind itself is a hint (not a logged op); the
    /// tangent re-derivation logs `MoveTangent` and the snapshot undo captures it.
    /// Returns `true` if any selected vertex was changed.
    pub fn set_selected_vertex_kind(
        &self,
        committed: &mut [Ph2dVectorAsset],
        selection: &VectorSelection,
        kind: VertexKind,
    ) -> bool {
        let mut changed = false;
        for &(asset_idx, vid) in &selection.vertices {
            let Some(asset) = committed.get_mut(asset_idx) else {
                continue;
            };
            let Some(v) = asset.network.vertices.iter_mut().find(|v| v.id == vid) else {
                continue;
            };
            v.kind = kind;
            // `v`'s borrow ends here → re-derive the tangents eagerly.
            apply_kind_eager(asset, vid, kind);
            changed = true;
        }
        changed
    }
}

/// Apply a column-major affine `[xx, xy, yx, yy, zx, zy]` (the inverse placement,
/// world → rest) to a point. ADR-0076: lets Direct edit a gizmo-moved shape.
fn apply_affine(m: [f32; 6], p: Vec2) -> Vec2 {
    Vec2::new(
        m[0] * p.x + m[2] * p.y + m[4],
        m[1] * p.x + m[3] * p.y + m[5],
    )
}

/// The vertex a tangent handle is anchored to.
fn anchor_vertex(asset: &Ph2dVectorAsset, seg: SegmentId, side: TangentSide) -> Option<VertexId> {
    let s = asset.network.segments.iter().find(|s| s.id == seg)?;
    Some(match side {
        TangentSide::OutAtStart => s.start,
        TangentSide::InAtEnd => s.end,
    })
}

/// Apply a tangent-handle drag: move the grabbed side to world `pos`
/// (stored as an offset from its anchor vertex). On a smooth vertex
/// (Mirror/Aligned) without `alt`, mirror the opposite incident tangent;
/// `alt` breaks the link (sets the vertex to `Free`).
fn apply_tangent_drag(
    asset: &mut Ph2dVectorAsset,
    seg: SegmentId,
    side: TangentSide,
    pos: Vec2,
    alt: bool,
) {
    let Some(anchor_vid) = anchor_vertex(asset, seg, side) else {
        return;
    };
    let Some(anchor_pos) = asset
        .network
        .vertices
        .iter()
        .find(|v| v.id == anchor_vid)
        .map(|v| v.pos)
    else {
        return;
    };
    let new_tangent = pos - anchor_pos;

    let kind = asset
        .network
        .vertices
        .iter()
        .find(|v| v.id == anchor_vid)
        .map(|v| v.kind)
        .unwrap_or(VertexKind::Free);
    let smooth = matches!(kind, VertexKind::Mirror | VertexKind::Aligned);

    // Break: independent handles from now on. (Kind is a hint, not an op in
    // the FROZEN VectorOp set — mutate directly; the tangent positions
    // themselves stay logged via MoveTangent.)
    if alt
        && smooth
        && let Some(v) = asset
            .network
            .vertices
            .iter_mut()
            .find(|v| v.id == anchor_vid)
    {
        v.kind = VertexKind::Free;
    }

    let _ = asset.edit_log.push_and_apply(
        VectorOp::MoveTangent {
            seg,
            which: side,
            new_pos: new_tangent,
        },
        &mut asset.network,
    );

    // Update the opposite incident handle on a still-smooth vertex:
    // - Mirror   → equal + opposite (`-new_tangent`).
    // - Aligned  → COLLINEAR but the opposite keeps its OWN length
    //   (Illustrator "asymmetric smooth"): `-dir(new_tangent) * |opposite|`.
    if smooth
        && !alt
        && let Some((opp_seg, opp_side)) = opposite_tangent(asset, anchor_vid, seg)
    {
        let opp_new = if matches!(kind, VertexKind::Aligned) {
            let len = current_tangent(asset, opp_seg, opp_side).length();
            if new_tangent.length() > 1e-6 {
                -new_tangent.normalize() * len
            } else {
                Vec2::ZERO
            }
        } else {
            -new_tangent
        };
        let _ = asset.edit_log.push_and_apply(
            VectorOp::MoveTangent {
                seg: opp_seg,
                which: opp_side,
                new_pos: opp_new,
            },
            &mut asset.network,
        );
    }
}

/// The current tangent vector on `seg`'s `side` — read before mirroring so the
/// Aligned (asymmetric) case can preserve the opposite handle's own length.
fn current_tangent(asset: &Ph2dVectorAsset, seg: SegmentId, side: TangentSide) -> Vec2 {
    asset
        .network
        .segments
        .iter()
        .find(|s| s.id == seg)
        .map(|s| match side {
            TangentSide::OutAtStart => s.out_at_start,
            TangentSide::InAtEnd => s.in_at_end,
        })
        .unwrap_or(Vec2::ZERO)
}

/// Vertex position (network-local rest pose) by id.
fn vpos(asset: &Ph2dVectorAsset, vid: VertexId) -> Option<Vec2> {
    asset
        .network
        .vertices
        .iter()
        .find(|v| v.id == vid)
        .map(|v| v.pos)
}

/// Endpoint distance of a segment (for Auto handle lengths).
fn seg_chord_len(asset: &Ph2dVectorAsset, seg: SegmentId) -> f32 {
    asset
        .network
        .segments
        .iter()
        .find(|s| s.id == seg)
        .and_then(|s| Some((vpos(asset, s.start)?, vpos(asset, s.end)?)))
        .map(|(a, b)| (b - a).length())
        .unwrap_or(0.0)
}

/// Unit "outgoing" axis from the current handles: prefer the longer non-zero one
/// (the in-handle points backward, so its axis is negated). `None` if both ~zero.
fn handle_axis(out_t: Vec2, in_t: Vec2) -> Option<Vec2> {
    if out_t.length() > 1e-4 {
        Some(out_t.normalize())
    } else if in_t.length() > 1e-4 {
        Some(-in_t.normalize())
    } else {
        None
    }
}

/// Unit axis along the prev→next chord through `vid` (Auto + degenerate fallback).
fn chord_axis(
    asset: &Ph2dVectorAsset,
    out_seg: SegmentId,
    in_seg: SegmentId,
) -> Option<Vec2> {
    let next = asset.network.segments.iter().find(|s| s.id == out_seg)?.end;
    let prev = asset.network.segments.iter().find(|s| s.id == in_seg)?.start;
    let d = vpos(asset, next)? - vpos(asset, prev)?;
    (d.length() > 1e-4).then(|| d.normalize())
}

/// Re-derive `vid`'s incident tangents to satisfy `kind` IMMEDIATELY (eager) —
/// so picking Smooth/Asymmetric/Auto reshapes the curve at once, not on the next
/// handle drag. Only the canonical 2-handle case (one out segment + one in
/// segment) re-derives; Free leaves the handles independent; endpoints/unusual
/// topology are left as-is. Logged via `MoveTangent` (replay-safe).
fn apply_kind_eager(asset: &mut Ph2dVectorAsset, vid: VertexId, kind: VertexKind) {
    if matches!(kind, VertexKind::Free) {
        return;
    }
    let mut out: Option<(SegmentId, Vec2)> = None;
    let mut inc: Option<(SegmentId, Vec2)> = None;
    for s in &asset.network.segments {
        if s.start == vid && out.is_none() {
            out = Some((s.id, s.out_at_start));
        } else if s.end == vid && inc.is_none() {
            inc = Some((s.id, s.in_at_end));
        }
    }
    let (Some((out_seg, out_t)), Some((in_seg, in_t))) = (out, inc) else {
        return;
    };

    // Mirror/Aligned preserve the user's current tangent line; Auto uses the
    // prev→next chord (auto-smooth).
    let axis = match kind {
        VertexKind::Auto => chord_axis(asset, out_seg, in_seg),
        _ => handle_axis(out_t, in_t).or_else(|| chord_axis(asset, out_seg, in_seg)),
    };
    let Some(axis) = axis else {
        return;
    };

    let (out_new, in_new) = match kind {
        VertexKind::Mirror => {
            let mag = (out_t.length() + in_t.length()) * 0.5;
            (axis * mag, -axis * mag)
        }
        VertexKind::Aligned => (axis * out_t.length(), -axis * in_t.length()),
        VertexKind::Auto => {
            let out_len = seg_chord_len(asset, out_seg) / 3.0;
            let in_len = seg_chord_len(asset, in_seg) / 3.0;
            (axis * out_len, -axis * in_len)
        }
        VertexKind::Free => return,
    };
    let _ = asset.edit_log.push_and_apply(
        VectorOp::MoveTangent {
            seg: out_seg,
            which: TangentSide::OutAtStart,
            new_pos: out_new,
        },
        &mut asset.network,
    );
    let _ = asset.edit_log.push_and_apply(
        VectorOp::MoveTangent {
            seg: in_seg,
            which: TangentSide::InAtEnd,
            new_pos: in_new,
        },
        &mut asset.network,
    );
}

/// The other tangent handle incident to `anchor_vid` (a different segment
/// touching that vertex), if any.
fn opposite_tangent(
    asset: &Ph2dVectorAsset,
    anchor_vid: VertexId,
    exclude_seg: SegmentId,
) -> Option<(SegmentId, TangentSide)> {
    for s in &asset.network.segments {
        if s.id == exclude_seg {
            continue;
        }
        if s.start == anchor_vid {
            return Some((s.id, TangentSide::OutAtStart));
        }
        if s.end == anchor_vid {
            return Some((s.id, TangentSide::InAtEnd));
        }
    }
    None
}

impl Tool for VectorDirectTool {
    fn id(&self) -> ToolId {
        ToolId::new("vector_direct")
    }

    fn label(&self) -> &str {
        "Vector Direct Select"
    }

    fn icon_slug(&self) -> &str {
        "vector-direct"
    }

    fn build_panel(&self) -> FloatingPanel {
        FloatingPanel::new(self.id(), "Vector Direct Select")
    }

    fn on_deactivate(&mut self) {
        self.grab = None;
    }

    fn handle_panel_event(&mut self, _event: PanelEvent) {}

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ph2d_vector_doc::{Segment, StyleTable, VectorNetwork, Vertex};

    /// Two segments sharing vertex 1 (a smooth corner), kind set by caller.
    fn two_segment_asset(kind: VertexKind) -> Ph2dVectorAsset {
        let mut net = VectorNetwork::empty();
        net.vertices.push(Vertex::auto(0, Vec2::new(0.0, 0.0)));
        net.vertices
            .push(Vertex::new(1, Vec2::new(50.0, 0.0), kind));
        net.vertices.push(Vertex::auto(2, Vec2::new(100.0, 0.0)));
        // seg 0: 0→1, seg 1: 1→2.
        let mut s0 = Segment::straight(0, 0, 1);
        s0.in_at_end = Vec2::new(-10.0, 0.0); // handle at (40,0)
        let mut s1 = Segment::straight(1, 1, 2);
        s1.out_at_start = Vec2::new(10.0, 0.0); // handle at (60,0)
        net.segments.push(s0);
        net.segments.push(s1);
        Ph2dVectorAsset::from_network(net, StyleTable::default())
    }

    #[test]
    fn grab_vertex_then_move_applies_movevertex_op() {
        let mut scene = vec![two_segment_asset(VertexKind::Free)];
        let mut sel = VectorSelection::new();
        let mut t = VectorDirectTool::new();
        // Grab vertex 0 at (0,0).
        let out = t.begin_drag(&scene, &mut sel, Vec2::new(0.0, 0.0), 5.0, &[]);
        assert_eq!(out, DirectOutcome::Grabbed(GrabTarget::Vertex(0)));
        assert!(sel.contains_vertex(0, 0));
        // Move it.
        t.drag_to(&mut scene, Vec2::new(5.0, 12.0), false, &[]);
        let v = scene[0]
            .network
            .vertices
            .iter()
            .find(|v| v.id == 0)
            .unwrap();
        assert_eq!(v.pos, Vec2::new(5.0, 12.0));
        // Logged as MoveVertex (replay-safe for T2.5).
        assert!(
            scene[0]
                .edit_log
                .ops
                .iter()
                .any(|o| matches!(o, VectorOp::MoveVertex { id: 0, .. }))
        );
    }

    #[test]
    fn grab_tangent_handle_moves_that_side() {
        let mut scene = vec![two_segment_asset(VertexKind::Free)];
        let mut sel = VectorSelection::new();
        let mut t = VectorDirectTool::new();
        // Handle of seg 1 out_at_start sits at vertex1 + (10,0) = (60,0).
        let out = t.begin_drag(&scene, &mut sel, Vec2::new(60.0, 0.0), 4.0, &[]);
        assert_eq!(
            out,
            DirectOutcome::Grabbed(GrabTarget::Tangent(1, TangentSide::OutAtStart))
        );
        // Drag the handle to (60,20) → new offset from vertex1(50,0) = (10,20).
        t.drag_to(&mut scene, Vec2::new(60.0, 20.0), false, &[]);
        let s1 = scene[0]
            .network
            .segments
            .iter()
            .find(|s| s.id == 1)
            .unwrap();
        assert_eq!(s1.out_at_start, Vec2::new(10.0, 20.0));
    }

    #[test]
    fn smooth_vertex_mirrors_opposite_handle() {
        let mut scene = vec![two_segment_asset(VertexKind::Mirror)];
        let mut sel = VectorSelection::new();
        let mut t = VectorDirectTool::new();
        // Grab seg1 out handle at (60,0), drag up.
        t.begin_drag(&scene, &mut sel, Vec2::new(60.0, 0.0), 4.0, &[]);
        t.drag_to(&mut scene, Vec2::new(60.0, 20.0), false, &[]);
        // seg1.out = (10,20); the opposite handle (seg0.in_at_end on vertex1)
        // mirrors to -(10,20) = (-10,-20).
        let s0 = scene[0]
            .network
            .segments
            .iter()
            .find(|s| s.id == 0)
            .unwrap();
        assert_eq!(s0.in_at_end, Vec2::new(-10.0, -20.0), "smooth mirror");
    }

    #[test]
    fn aligned_vertex_keeps_opposite_handle_length() {
        // Asymmetric smooth: drag one handle → opposite stays COLLINEAR but
        // keeps its OWN length (unlike Mirror, which equalizes).
        let mut scene = vec![two_segment_asset(VertexKind::Aligned)];
        let mut sel = VectorSelection::new();
        let mut t = VectorDirectTool::new();
        // Grab seg1.out at (60,0) [offset (10,0) from vertex1@(50,0)]; seg0.in
        // starts at (-10,0) → length 10.
        t.begin_drag(&scene, &mut sel, Vec2::new(60.0, 0.0), 4.0, &[]);
        // Drag to (50,40) → grabbed offset (0,40); opposite collinear-opposite is
        // direction (0,-1) × original length 10 = (0,-10).
        t.drag_to(&mut scene, Vec2::new(50.0, 40.0), false, &[]);
        let s0 = scene[0].network.segments.iter().find(|s| s.id == 0).unwrap();
        assert!(
            (s0.in_at_end - Vec2::new(0.0, -10.0)).length() < 1e-3,
            "aligned opposite = collinear @ original length, got {:?}",
            s0.in_at_end
        );
    }

    #[test]
    fn set_selected_vertex_kind_changes_kind() {
        let mut scene = vec![two_segment_asset(VertexKind::Auto)];
        let mut sel = VectorSelection::new();
        sel.select_only_vertex(0, 1);
        let t = VectorDirectTool::new();
        assert!(t.set_selected_vertex_kind(&mut scene, &sel, VertexKind::Mirror));
        let v1 = scene[0]
            .network
            .vertices
            .iter()
            .find(|v| v.id == 1)
            .unwrap();
        assert_eq!(v1.kind, VertexKind::Mirror);
    }

    #[test]
    fn set_kind_smooth_re_derives_tangents_eagerly() {
        // Asymmetric, non-collinear handles → picking Smooth collinearizes +
        // equalizes them IMMEDIATELY (no drag needed).
        let mut net = VectorNetwork::empty();
        net.vertices.push(Vertex::auto(0, Vec2::new(0.0, 0.0)));
        net.vertices.push(Vertex::new(1, Vec2::new(50.0, 0.0), VertexKind::Free));
        net.vertices.push(Vertex::auto(2, Vec2::new(100.0, 0.0)));
        let mut s0 = Segment::straight(0, 0, 1);
        s0.in_at_end = Vec2::new(-5.0, 5.0); // not collinear with s1.out
        let mut s1 = Segment::straight(1, 1, 2);
        s1.out_at_start = Vec2::new(10.0, 0.0);
        net.segments.push(s0);
        net.segments.push(s1);
        let mut scene = vec![Ph2dVectorAsset::from_network(net, StyleTable::default())];
        let mut sel = VectorSelection::new();
        sel.select_only_vertex(0, 1);
        let t = VectorDirectTool::new();
        t.set_selected_vertex_kind(&mut scene, &sel, VertexKind::Mirror);
        let out = scene[0]
            .network
            .segments
            .iter()
            .find(|s| s.id == 1)
            .unwrap()
            .out_at_start;
        let inn = scene[0]
            .network
            .segments
            .iter()
            .find(|s| s.id == 0)
            .unwrap()
            .in_at_end;
        assert!(
            (out.length() - inn.length()).abs() < 1e-3,
            "equal magnitude: {} vs {}",
            out.length(),
            inn.length()
        );
        assert!(
            (out.normalize() + inn.normalize()).length() < 1e-3,
            "collinear-opposite"
        );
    }

    #[test]
    fn alt_drag_breaks_tangent_to_free_and_skips_mirror() {
        let mut scene = vec![two_segment_asset(VertexKind::Mirror)];
        let mut sel = VectorSelection::new();
        let mut t = VectorDirectTool::new();
        t.begin_drag(&scene, &mut sel, Vec2::new(60.0, 0.0), 4.0, &[]);
        t.drag_to(&mut scene, Vec2::new(60.0, 20.0), true, &[]); // alt
        // Vertex 1 becomes Free; opposite handle is NOT mirrored.
        let v1 = scene[0]
            .network
            .vertices
            .iter()
            .find(|v| v.id == 1)
            .unwrap();
        assert_eq!(v1.kind, VertexKind::Free, "tangent broken → Free");
        let s0 = scene[0]
            .network
            .segments
            .iter()
            .find(|s| s.id == 0)
            .unwrap();
        assert_eq!(
            s0.in_at_end,
            Vec2::new(-10.0, 0.0),
            "opposite handle unchanged"
        );
    }

    #[test]
    fn miss_returns_missed_no_grab() {
        let scene = vec![two_segment_asset(VertexKind::Free)];
        let mut sel = VectorSelection::new();
        let mut t = VectorDirectTool::new();
        let out = t.begin_drag(&scene, &mut sel, Vec2::new(500.0, 500.0), 5.0, &[]);
        assert_eq!(out, DirectOutcome::Missed);
        assert!(!t.is_dragging());
    }

    #[test]
    fn drag_without_grab_is_idle() {
        let mut scene = vec![two_segment_asset(VertexKind::Free)];
        let mut t = VectorDirectTool::new();
        assert_eq!(
            t.drag_to(&mut scene, Vec2::new(1.0, 1.0), false, &[]),
            DirectOutcome::Idle
        );
    }

    #[test]
    fn nan_drag_is_ignored() {
        let mut scene = vec![two_segment_asset(VertexKind::Free)];
        let mut sel = VectorSelection::new();
        let mut t = VectorDirectTool::new();
        t.begin_drag(&scene, &mut sel, Vec2::new(0.0, 0.0), 5.0, &[]);
        assert_eq!(
            t.drag_to(&mut scene, Vec2::new(f32::NAN, 0.0), false, &[]),
            DirectOutcome::Idle
        );
        // Vertex unmoved.
        let v = scene[0]
            .network
            .vertices
            .iter()
            .find(|v| v.id == 0)
            .unwrap();
        assert_eq!(v.pos, Vec2::new(0.0, 0.0));
    }

    #[test]
    fn tool_identity() {
        let t = VectorDirectTool::new();
        assert_eq!(t.id(), ToolId::new("vector_direct"));
        assert_eq!(t.icon_slug(), "vector-direct");
    }
}
