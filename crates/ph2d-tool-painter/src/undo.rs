//! Painter undo/redo controller — **transactional (structural) layer undo**.
//!
//! # What it covers
//!
//! Every structural layer edit — add / delete / duplicate / group / reorder a
//! layer, create a mask or adjustment, move a layer, switch the active layer —
//! is recorded as a *before/after* pair of full model snapshots. Undo rolls the
//! model back to `before`; redo rolls forward to `after`. This is the
//! layers + effects editor's undo history.
//!
//! # Design
//!
//! [`UndoController`] keeps two chronological stacks of [`UndoEntry`] (the
//! `undo` stack the user can step back through, the `redo` stack populated by
//! `undo` and cleared by any new edit — the standard linear-history contract),
//! bounded to `max_depth`. Each entry carries BOTH endpoints (`before` + `after`
//! [`ModelSnapshot`]s) so the swap needs no live state and is allocation-stable;
//! a snapshot's `canvas_rgba` is `Arc`-shared so the clone on a (rare, user-paced)
//! structural edit is cheap CoW.

use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

use crate::compositor::LayerImage;
use crate::layers::{LayerId as RtLayerId, LayerStack};

/// A full snapshot of the editable layer model, for **transactional (structural)
/// undo** — adding / deleting / duplicating a layer, creating a mask or
/// adjustment, or switching the active layer. A structural edit reshapes the
/// whole model, so the undo entry stores the entire state to roll back to.
///
/// `canvas_rgba` is `Arc`-shared (cheap CoW clone — the tool already wraps the
/// active layer's working buffer in an `Arc`); the active layer id lives INSIDE
/// `layers` (`LayerStack` owns it), so restoring `layers` restores the active
/// target too. `images` is the per-layer pixel store for every NON-active layer.
#[derive(Clone, Debug)]
pub struct ModelSnapshot {
    pub layers: LayerStack,
    pub images: BTreeMap<RtLayerId, Arc<LayerImage>>,
    /// **Impasto** relief per layer — captured with the pixels so undoing a stroke takes back the
    /// thickness it laid down, not just the colour (a snapshot that forgot it would leave the light
    /// pass reporting a ridge over paint that is no longer there). Empty for every layer that was
    /// never sculpted, which is the norm — the map is lazy.
    pub heights: BTreeMap<RtLayerId, Arc<Vec<f32>>>,
    /// Paint coverage per layer, captured with the relief it belongs to (see `PainterTool::covers`).
    pub covers: BTreeMap<RtLayerId, Arc<Vec<u8>>>,
    /// The paint's MATERIAL per layer, captured with the relief and the coverage — the three are one
    /// fact about one stroke and they must roll back together.
    ///
    /// Left out when the material landed (2026-07-13), and the hole was nearly invisible: on bare canvas
    /// an undone stroke's coverage goes to zero, so the light weights its stale material by zero and
    /// nothing shows. Paint the stroke over EXISTING paint, though, and undo restores the lower stroke's
    /// coverage under the upper one's material — the paint underneath comes back wearing the wrong
    /// gloss. `undoing_a_stroke_restores_the_material_underneath_it`.
    pub mats: BTreeMap<RtLayerId, Arc<Vec<ph2d_painter_brush::material::MaterialBytes>>>,
    pub canvas_rgba: Arc<Vec<u8>>,
    pub selection: BTreeSet<RtLayerId>,
    /// The open on-canvas shape editor (Curve / Ellipse / Polygon), captured so a structural undo/redo
    /// restores the live overlay TOGETHER with the pixels — the two can never desync. `None` = no shape
    /// open (a layer op, or a committed/cancelled shape).
    pub shape: Option<Box<ShapeEditState>>,
    /// The **Offset** slider track at capture time, restored alongside the shape so undoing an Offset drag
    /// reinstates its value. Ignored when `shape` is `None`.
    pub offset_norm: f32,
    /// The **accumulated** Offset (px) from prior Apply & Keep presses (see `PaintState::shape_offset_base_px`),
    /// restored with `offset_norm` so undoing an Apply & Keep reinstates the pre-commit Offset exactly.
    pub offset_base_px: f32,
    /// The in-progress drag-preview's saved pixels, if a shape preview was live — so a restore can peel the
    /// preview back to the pristine baseline before re-stamping the editor's geometry (no double paint).
    pub preview_patch: Option<PreviewPatch>,
    /// The PARKED stroke shapes (multi-shape) at capture time — every simultaneously-editable shape that
    /// isn't the one live editor (`shape`). Captured so a structural undo/redo restores the whole editable
    /// set in lock-step with the baked/preview pixels, not just the active one. Empty = single/no shape.
    pub parked_shapes: Vec<ParkedShapeState>,
    /// The ACTIVE stroke shape's boolean Operation (wire `0`=Overlay `1`=Add `2`=Remove) at capture time —
    /// the parked shapes carry theirs inline ([`ParkedShapeState::op`]) but the active one lives outside the
    /// editor state, so without this a centre-square op-cycle tap was NOT undoable (Enio 2026-07-05).
    pub active_op: u8,
    /// The **Mask** brush's transient scratch buffer + its target layer at capture time. A mask stroke
    /// mutates only this scratch (it swaps in/out of `canvas_rgba`, which stays unchanged), so without
    /// capturing it here a mask stroke produced a no-op undo entry and could not be rolled back. Restoring
    /// it alongside the layers keeps the live mask-in-progress in lock-step with the global undo/redo.
    /// Empty `Arc` + `None` = no scratch.
    pub mask_scratch: Arc<Vec<u8>>,
    pub mask_scratch_target: Option<crate::layers::LayerId>,
    /// The **Selection** mask + its active flag at capture time (ADR-0103). A selection edit mutates only
    /// this document-wide coverage buffer (not `canvas_rgba`), so — exactly like `mask_scratch` — it must
    /// be captured here or the edit would be a no-op undo entry. Restoring it in lock-step with the pixels
    /// keeps the selection and the layers on the ONE interleaved timeline. Empty `Arc` + `false` = none.
    pub selection_mask: Arc<Vec<u8>>,
    pub selection_active: bool,
    /// The CRISP (pre-Feather) selection accumulator + the Feather amount, captured so undo/redo restores
    /// the exact effective mask AND keeps the Feather slider re-derivable from the right base.
    pub selection_crisp: Arc<Vec<u8>>,
    pub selection_feather: f32,
    /// The **parametric selection shape list** (ADR-0103 Am.2) at capture time — the source of truth the
    /// mask is a derived cache of. Captured so a structural undo/redo restores the editable SHAPES (curve
    /// anchors + handles, box params) in lock-step with the mask: without it, undo restored only the
    /// rasterized mask and the next `recompose_selection_mask` regenerated the edited shape → the selection
    /// curve point edit "came back" (Enio 2026-07-03). Empty = no parametric selection.
    pub(crate) selection_shapes: Vec<crate::tool::SelectionEntry>,
    /// The **Deform** session at capture time — the cumulative displacement map + the pristine `pre` +
    /// the active flag (Deform Wave 1). Captured so undo/redo rolls the WARP back in lock-step with the
    /// pixels: without it, undoing a deform stroke restored the pixels but dropped the displacement, so
    /// Reconstruct could no longer un-warp the remaining deform (Enio 2026-07-04). Empty `Arc`s + `false`
    /// = no session. `Arc`-shared, so a non-deform snapshot carries an empty (0-byte) map for free.
    pub(crate) deform: WarpSnap,
    /// The protection/selection GATE epoch at capture time ([`gate`]): `ref` = the canvas when the
    /// epoch began, `free` = the unrestricted painting. Captured under the same-commit law so
    /// undo/redo restores the CEILING mid-epoch — dropping the planes instead would re-seed the next
    /// stroke's epoch from a canvas that already ate one epoch's worth of feather paint, and the
    /// `(1−keep)` term would compound across undo-cut epochs (the disease, one level up). Empty = no
    /// epoch; `Arc`-shared, so a non-gated snapshot carries two 0-byte handles for free.
    pub gate_ref: Arc<Vec<u8>>,
    pub gate_free: Arc<Vec<u8>>,
    /// The **Sculpt** session at capture time (`docs/Painter/18…` §10.4).
    ///
    /// Captured for a reason the Deform did NOT have, and it is the reason the plan demanded it: the
    /// sculpt writes the layer's relief **live**, so a snapshot taken mid-preview captures an
    /// already-carved plane. Drop the session on restore and the shape editor's next re-stamp opens a
    /// FRESH one whose frozen source is that carved plane — and re-runs the kernel on top of it. Undo,
    /// redo, and the ridge has been smoothed twice; do it again and it melts further. The pixels are
    /// perfect the whole time, so nothing else in the system notices.
    ///
    /// Rolling the session back with the relief makes the restored state a state the sculpt can *continue
    /// from*, which is the same thing `deform_disp` buys the warp.
    pub(crate) sculpt: SculptSnap,
}

/// The Deform session as an undo snapshot — `Arc`-shared, so a snapshot taken while nobody is deforming
/// carries empty (0-byte) buffers and costs nothing.
///
/// The frozen impasto planes (`pre_h` / `pre_cover` / `pre_mats`, W4) ride beside the pixels' `pre` for
/// the same reason `disp` does (Enio 2026-07-04): an undo mid-session must leave Reconstruct able to
/// un-warp what remains — of the BODY as much as of the colour. Empty when the session's layer carried
/// no relief.
#[derive(Clone, Debug)]
pub(crate) struct WarpSnap {
    pub(crate) disp: Arc<Vec<[f32; 2]>>,
    pub(crate) pre: Arc<Vec<u8>>,
    pub(crate) pre_h: Arc<Vec<f32>>,
    pub(crate) pre_cover: Arc<Vec<u8>>,
    pub(crate) pre_mats: Arc<Vec<ph2d_painter_brush::material::MaterialBytes>>,
    pub(crate) relief_layer: Option<RtLayerId>,
    pub(crate) active: bool,
}

/// The Sculpt session as an undo snapshot — `Arc`-shared, so a snapshot taken while nobody is sculpting
/// carries two empty (0-byte) buffers and costs nothing.
#[derive(Clone, Debug, Default)]
pub(crate) struct SculptSnap {
    /// The layer's relief as the stroke found it — the frozen source every re-render reads.
    pub(crate) pre: Arc<Vec<f32>>,
    /// The accumulated per-texel touch.
    pub(crate) amount: Arc<Vec<f32>>,
    /// The plane family's per-texel target (`Σ w·plane(i)`) — empty in the Smooth family.
    ///
    /// It rides the snapshot and the blur memo does not, and the asymmetry is the point: the memo is a
    /// function of `pre` alone, so the restore can throw it away and let it rebuild. `plane_sum` is a
    /// function of the **dab list**, which no longer exists — drop it on restore and the next re-stamp
    /// divides by it anyway, pulling the footprint toward height 0. Doc 18 §10.4 states the rule with a scar
    /// behind it: *ao adicionar um plano, adicione-o ao snapshot no mesmo commit.*
    pub(crate) plane_sum: Arc<Vec<f32>>,
    /// The **matter** as the stroke found it — coverage, material and the layer's pixels.
    ///
    /// Here for the same reason `plane_sum` is, and the same rule (§10.4): **Inflate moves paint**, so it
    /// re-renders the coverage / material / colour from these every frame. Restore the relief without them
    /// and the next re-render derives the paint from a canvas the gesture has ALREADY written on — the
    /// smear compounds, once per undo, and the first thing anyone would blame is the kernel.
    pub(crate) pre_cover: Arc<Vec<u8>>,
    /// See [`Self::pre_cover`].
    pub(crate) pre_mats: Arc<Vec<ph2d_painter_brush::material::MaterialBytes>>,
    /// See [`Self::pre_cover`]. RGBA8.
    pub(crate) pre_rgba: Arc<Vec<u8>>,
    /// Which layer the session belongs to — and, the session dying at commit, also *whether there is an
    /// uncommitted gesture at all* (`None` ⇒ no session). The two used to be separate fields.
    pub(crate) layer: Option<RtLayerId>,
    /// The window the stroke touched — what a re-stamp restores, and what a knob edit re-renders.
    pub(crate) bbox: Option<crate::compositor::Region>,
}

/// Plain-data snapshot of an open on-canvas shape editor, stored in a [`ModelSnapshot`] so a structural
/// undo/redo reinstates the editable overlay with the pixels. Geometry only — the transient grab/gizmo
/// fields reset to idle on restore. Curve handle kinds are kept as their wire `u8` so this module stays
/// free of the editor types.
#[derive(Clone, Debug, PartialEq)]
pub enum ShapeEditState {
    Curve(CurveState),
    Ellipse(EllipseState),
    Polygon(PolygonState),
    Line(LineState),
}

/// Editable Curve / Free Hand state (see `tool::paint::curve::CurveEditor`).
#[derive(Clone, Debug, PartialEq)]
pub struct CurveState {
    pub points: Vec<[f32; 2]>,
    pub handles: Vec<[[f32; 2]; 2]>,
    pub kinds: Vec<u8>,
    pub selected: Option<usize>,
    pub added_point: bool,
    pub closed: bool,
    pub editing: bool,
    pub freehand: bool,
    pub seed: u64,
    pub anchor: [f32; 2],
    pub stabilized: [f32; 2],
}

/// Editable Ellipse state (see `tool::paint::ellipse::EllipseEditor`).
#[derive(Clone, Debug, PartialEq)]
pub struct EllipseState {
    pub center: [f32; 2],
    pub u: [f32; 2],
    pub rx: f32,
    pub ry: f32,
    pub editing: bool,
    pub seed: u64,
}

/// Editable Polygon state (see `tool::paint::polygon::PolygonEditor`).
#[derive(Clone, Debug, PartialEq)]
pub struct PolygonState {
    pub center: [f32; 2],
    pub u: [f32; 2],
    pub rx: f32,
    pub ry: f32,
    pub sides: u32,
    pub editing: bool,
    pub seed: u64,
}

/// Editable Line (polyline) state (see `tool::paint::line::LineEditor`). Plain corner points, no handles;
/// per-corner Fillet/Chamfer carried as `(tag, amount)` wire pairs (`0` sharp / `1` fillet / `2` chamfer).
#[derive(Clone, Debug, PartialEq)]
pub struct LineState {
    pub points: Vec<[f32; 2]>,
    pub closed: bool,
    pub editing: bool,
    pub corner_mods: Vec<(u8, f32)>,
    pub seed: u64,
}

impl ShapeEditState {
    /// Geometry equality IGNORING the curve's `selected` index — selecting a point is not an undoable
    /// change (the no-op check in `commit_shape_txn` drops it), though selection IS restored on undo.
    #[must_use]
    pub fn geom_eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Curve(a), Self::Curve(b)) => {
                a.points == b.points
                    && a.handles == b.handles
                    && a.kinds == b.kinds
                    && a.added_point == b.added_point
                    && a.closed == b.closed
                    && a.editing == b.editing
                    && a.freehand == b.freehand
                    && a.seed == b.seed
                    && a.anchor == b.anchor
                    && a.stabilized == b.stabilized
            }
            _ => self == other,
        }
    }
}

/// A PARKED (inactive but still-editable) stroke shape captured for undo: its geometry plus the wire `u8`
/// of its Operation (`0`=Overlay `1`=Add `2`=Remove — see `tool::paint::stroke_multi::StrokeOp`). Stroke
/// multi-shape keeps a list of these alongside the one live editor (`shape`); a structural undo/redo
/// restores the whole list so every simultaneously-editable shape rolls back in lock-step with the pixels.
/// Kept as a wire `u8` so this module stays free of the `paint` editor types (mirrors `CurveState.kinds`).
#[derive(Clone, Debug, PartialEq)]
pub struct ParkedShapeState {
    pub state: ShapeEditState,
    pub op: u8,
}

/// The in-progress drag-preview's saved pixels (a small bbox), carried in a [`ModelSnapshot`] so a restore
/// can peel the live preview back to the pristine baseline before re-stamping it (no double paint). `None`
/// for a snapshot taken with no live preview (layer ops, a committed shape).
#[derive(Clone, Debug, PartialEq)]
pub struct PreviewPatch {
    pub x: u32,
    pub y: u32,
    pub w: u32,
    pub h: u32,
    pub pixels: Vec<u8>,
    /// The gate epoch's FREE-plane pixels under the same rect (pre-preview), when an epoch was live —
    /// the undo twin of `DragPreview::free_pixels`. Restoring the canvas rect without peeling the free
    /// rect would leave the preview's ghost in the unrestricted plane, and the shape re-stamp after an
    /// undo would deepen the feather by one phantom stroke.
    pub free_pixels: Option<Vec<u8>>,
}

/// Default cap on retained undo entries (ring depth). The caller can raise or
/// lower it from its memory budget. The oldest entry beyond the cap is dropped
/// (that depth becomes non-undoable, like a ring history).
pub const DEFAULT_MAX_DEPTH: usize = 300;

/// Marker for entries that may COALESCE with an immediately-preceding entry of the same kind: a run of
/// repeated same-kind actions (N progressive-Simplify presses, N boolean-op taps on the same shape)
/// collapses into ONE undo step whose `before` is the state before the FIRST action and whose `after`
/// tracks the latest (Enio 2026-07-05: the curve workflow's undo was too granular — "cada mínima ação
/// entra na sequência"). Any other action (a normal entry) breaks the run.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CoalesceKind {
    /// A **Simplify** press (stroke curve or selection curves) — progressive presses are one logical
    /// "simplify to taste" action; one undo restores the pre-Simplify curve.
    Simplify,
    /// A centre-square boolean-op cycle tap on the ACTIVE stroke shape.
    OpCycleStroke,
    /// A centre-square Add↔Remove tap on selection shape `i` — taps on DIFFERENT shapes don't coalesce.
    OpCycleSelection(usize),
}

/// One retained history entry: a structural edit stored as BOTH endpoints (the
/// model `before` and `after` the edit). Carrying both means the entry needs no
/// live state to swap directions; structural edits are user-paced (rare) so two
/// model snapshots is a fine trade for the simpler, allocation-stable swap.
#[derive(Clone, Debug)]
struct UndoEntry {
    before: Box<ModelSnapshot>,
    after: Box<ModelSnapshot>,
    /// `Some` for a coalescible action — see [`CoalesceKind`]. Plain entries carry `None` and thereby
    /// BREAK any run in progress.
    kind: Option<CoalesceKind>,
}

/// Snapshot-based undo/redo for the editable layer model.
///
/// The caller (the [`PainterTool`](crate::tool::PainterTool)) drives it at two
/// points: just after a structural edit commits ([`Self::record_structural`]),
/// and when the gesture/shell requests [`Self::undo`] / [`Self::redo`] (each
/// returns the [`ModelSnapshot`] the caller must reinstall).
#[derive(Debug)]
pub struct UndoController {
    undo: Vec<UndoEntry>,
    redo: Vec<UndoEntry>,
    max_depth: usize,
}

impl Default for UndoController {
    fn default() -> Self {
        Self::new(DEFAULT_MAX_DEPTH)
    }
}

impl UndoController {
    /// New controller with an explicit retained-depth ceiling.
    #[must_use]
    pub fn new(max_depth: usize) -> Self {
        Self {
            undo: Vec::new(),
            redo: Vec::new(),
            // A depth of 0 would make undo permanently unavailable; clamp to 1
            // so a degenerate budget still allows a single level.
            max_depth: max_depth.max(1),
        }
    }

    /// Record a STRUCTURAL transition (add/delete/duplicate layer, mask/adjustment
    /// create, active-layer switch). `before` is the model to roll back to on undo;
    /// `after` is the model to roll forward to on redo. Pushing it clears the redo
    /// branch (standard linear-history semantics).
    pub fn record_structural(&mut self, before: ModelSnapshot, after: ModelSnapshot) {
        self.undo.push(UndoEntry {
            before: Box::new(before),
            after: Box::new(after),
            kind: None,
        });
        self.redo.clear();
        self.cap();
    }

    /// Record a COALESCIBLE structural transition: when the newest undo entry carries the SAME
    /// [`CoalesceKind`] (and no redo branch intervenes), the run extends — the top entry keeps its
    /// original `before` and adopts this `after` — so N repeated same-kind actions undo as ONE step.
    /// Otherwise it pushes a fresh entry (which starts a new run).
    pub fn record_structural_coalesced(
        &mut self,
        kind: CoalesceKind,
        before: ModelSnapshot,
        after: ModelSnapshot,
    ) {
        if self.redo.is_empty()
            && let Some(top) = self.undo.last_mut()
            && top.kind == Some(kind)
        {
            *top.after = after;
            return;
        }
        self.undo.push(UndoEntry {
            before: Box::new(before),
            after: Box::new(after),
            kind: Some(kind),
        });
        self.redo.clear();
        self.cap();
    }

    /// Undo the most recent structural edit: roll back to its `before` model and
    /// park the entry on the redo stack so a later [`Self::redo`] can roll forward
    /// to `after`. Returns the model to reinstall, or `None` if nothing to undo.
    pub fn undo(&mut self) -> Option<Box<ModelSnapshot>> {
        let entry = self.undo.pop()?;
        let restore = entry.before.clone();
        self.redo.push(entry);
        Some(restore)
    }

    /// Redo the most recently undone structural edit: roll forward to its `after`
    /// model and park the entry back on the undo stack. Returns the model to
    /// reinstall, or `None` if the redo stack is empty.
    pub fn redo(&mut self) -> Option<Box<ModelSnapshot>> {
        let entry = self.redo.pop()?;
        let restore = entry.after.clone();
        self.undo.push(entry);
        Some(restore)
    }

    /// `true` if there is at least one edit to undo (drives the `undo_enabled`
    /// affordance).
    #[must_use]
    pub fn can_undo(&self) -> bool {
        !self.undo.is_empty()
    }

    /// `true` if there is at least one undone edit to redo (drives the
    /// `redo_enabled` affordance).
    #[must_use]
    pub fn can_redo(&self) -> bool {
        !self.redo.is_empty()
    }

    /// Retained undo depth (for tests / memory introspection).
    #[must_use]
    pub fn undo_depth(&self) -> usize {
        self.undo.len()
    }

    /// Retained redo depth.
    #[must_use]
    pub fn redo_depth(&self) -> usize {
        self.redo.len()
    }

    /// Drop the controller's history (e.g. on `set_source` of a fresh canvas).
    pub fn clear(&mut self) {
        self.undo.clear();
        self.redo.clear();
    }

    /// Enforce the ring depth ceiling: drop the oldest entries (front of the
    /// Vec) so the retained undo stack never exceeds `max_depth`.
    fn cap(&mut self) {
        if self.undo.len() > self.max_depth {
            let overflow = self.undo.len() - self.max_depth;
            self.undo.drain(0..overflow);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn model(active_px: u8) -> ModelSnapshot {
        ModelSnapshot {
            layers: LayerStack::new(),
            images: BTreeMap::new(),
            heights: BTreeMap::new(),
            mats: BTreeMap::new(),
            covers: BTreeMap::new(),
            canvas_rgba: Arc::new(vec![active_px; 16]),
            selection: BTreeSet::new(),
            shape: None,
            offset_norm: 0.5,
            offset_base_px: 0.0,
            preview_patch: None,
            parked_shapes: Vec::new(),
            active_op: 0,
            mask_scratch: Arc::new(Vec::new()),
            mask_scratch_target: None,
            selection_mask: Arc::new(Vec::new()),
            selection_active: false,
            selection_crisp: Arc::new(Vec::new()),
            selection_feather: 0.0,
            selection_shapes: Vec::new(),
            gate_ref: Arc::new(Vec::new()),
            gate_free: Arc::new(Vec::new()),
            deform: WarpSnap {
                disp: Arc::new(Vec::new()),
                pre: Arc::new(Vec::new()),
                pre_h: Arc::new(Vec::new()),
                pre_cover: Arc::new(Vec::new()),
                pre_mats: Arc::new(Vec::new()),
                relief_layer: None,
                active: false,
            },
            sculpt: SculptSnap::default(),
        }
    }

    #[test]
    fn undo_rolls_back_to_before() {
        let mut c = UndoController::new(DEFAULT_MAX_DEPTH);
        c.record_structural(model(0x11), model(0x22));
        assert!(c.can_undo());
        let restored = c.undo().expect("one entry to undo");
        assert_eq!(restored.canvas_rgba.as_slice(), &[0x11; 16]);
        assert!(!c.can_undo());
        assert!(c.can_redo());
    }

    #[test]
    fn redo_rolls_forward_to_after() {
        let mut c = UndoController::new(DEFAULT_MAX_DEPTH);
        c.record_structural(model(0x11), model(0x22));
        c.undo();
        let restored = c.redo().expect("one entry to redo");
        assert_eq!(restored.canvas_rgba.as_slice(), &[0x22; 16]);
        assert!(c.can_undo());
        assert!(!c.can_redo());
    }

    #[test]
    fn new_edit_clears_redo_branch() {
        let mut c = UndoController::new(DEFAULT_MAX_DEPTH);
        c.record_structural(model(0), model(1));
        c.record_structural(model(1), model(2));
        c.undo();
        assert!(c.can_redo());
        c.record_structural(model(1), model(3));
        assert!(!c.can_redo(), "a new edit must invalidate the redo branch");
    }

    /// A run of same-kind coalescible entries collapses to ONE undo step spanning first-before →
    /// latest-after; a plain entry breaks the run; an undo/redo boundary never merges across.
    #[test]
    fn coalesced_runs_merge_and_break_correctly() {
        let mut c = UndoController::new(DEFAULT_MAX_DEPTH);
        c.record_structural_coalesced(CoalesceKind::Simplify, model(0), model(1));
        c.record_structural_coalesced(CoalesceKind::Simplify, model(1), model(2));
        c.record_structural_coalesced(CoalesceKind::Simplify, model(2), model(3));
        assert_eq!(c.undo_depth(), 1, "three Simplify presses = one entry");
        let restored = c.undo().expect("entry");
        assert_eq!(
            restored.canvas_rgba.as_slice(),
            &[0; 16],
            "one undo restores the state before the FIRST press"
        );
        let fwd = c.redo().expect("entry");
        assert_eq!(
            fwd.canvas_rgba.as_slice(),
            &[3; 16],
            "redo lands on the LATEST press"
        );
        // A different kind never merges.
        c.record_structural_coalesced(CoalesceKind::OpCycleSelection(0), model(3), model(4));
        c.record_structural_coalesced(CoalesceKind::OpCycleSelection(1), model(4), model(5));
        assert_eq!(
            c.undo_depth(),
            3,
            "different shapes' taps stay separate entries"
        );
        // A plain entry breaks the run: the next same-kind action starts a NEW entry.
        c.record_structural_coalesced(CoalesceKind::Simplify, model(5), model(6));
        c.record_structural(model(6), model(7));
        c.record_structural_coalesced(CoalesceKind::Simplify, model(7), model(8));
        assert_eq!(
            c.undo_depth(),
            6,
            "a plain entry between runs prevents merging"
        );
    }

    /// After an undo, a new same-kind action must NOT merge into the undone entry (the redo branch is
    /// discarded and a fresh run starts) — merging across the boundary would corrupt the timeline.
    #[test]
    fn coalescing_never_merges_across_an_undo_boundary() {
        let mut c = UndoController::new(DEFAULT_MAX_DEPTH);
        c.record_structural_coalesced(CoalesceKind::Simplify, model(0), model(1));
        c.undo();
        c.record_structural_coalesced(CoalesceKind::Simplify, model(0), model(9));
        assert_eq!(c.undo_depth(), 1);
        assert!(!c.can_redo(), "the redo branch was discarded");
        let restored = c.undo().expect("entry");
        assert_eq!(restored.canvas_rgba.as_slice(), &[0; 16]);
    }

    #[test]
    fn depth_cap_drops_oldest() {
        let mut c = UndoController::new(4);
        for v in 0..10u8 {
            c.record_structural(model(v), model(v + 1));
        }
        assert!(c.undo_depth() <= 4, "cap enforced; got {}", c.undo_depth());
    }

    #[test]
    fn clear_drops_both_stacks() {
        let mut c = UndoController::new(DEFAULT_MAX_DEPTH);
        c.record_structural(model(0), model(1));
        c.undo();
        c.clear();
        assert!(!c.can_undo() && !c.can_redo());
    }
}
