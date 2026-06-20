//! Read-only layer-MODEL accessors — the projection the docked layers panel
//! renders (stack view, active/selection state, row order). No mutation; no
//! cache or preview side effects. `impl PainterTool` (one of several blocks in
//! this crate). Split out of the former `tool/layers.rs` god-file (pure move).

use super::super::*;

impl PainterTool {
    // ── W3 layer model (runtime canon, ADR-0046-amд-1 Option A) ─────────

    /// Read-only view of the runtime layer stack — what the docked layers
    /// panel renders (the shell snapshots `layers().clone()` per frame).
    #[must_use]
    pub fn layers(&self) -> &LayerStack {
        &self.layers
    }

    /// Canvas dimensions `(width, height)` of the current source — what the GPU
    /// preview composites at. `(0, 0)` before any `set_source`.
    #[must_use]
    pub fn source_size(&self) -> (u32, u32) {
        self.source_size
    }

    /// `true` when the active edit target is a grayscale mask.
    #[must_use]
    pub fn active_is_mask(&self) -> bool {
        self.layers
            .active()
            .and_then(|id| self.layers.get(id))
            .is_some_and(|l| matches!(l.kind, LayerKind::Mask(_)))
    }

    /// `true` when the active layer has its alpha locked — paint is then
    /// restricted to the layer's existing alpha (§2.10).
    #[must_use]
    pub fn active_alpha_locked(&self) -> bool {
        self.layers
            .active()
            .and_then(|id| self.layers.get(id))
            .is_some_and(|l| l.alpha_locked)
    }

    /// The first raster layer (depth-first pre-order) inside group `id`, or
    /// `None` for an empty group / a group containing only groups. Lets
    /// activating a group "enter" its top paintable layer (a group itself has no
    /// pixel buffer). Masks are owner-attached (not in `children`), so this only
    /// ever returns a raster.
    pub(crate) fn first_paintable_descendant(&self, id: RtLayerId) -> Option<RtLayerId> {
        let children = match self.layers.get(id).map(|l| &l.kind) {
            Some(LayerKind::Group(g)) => g.children.clone(),
            _ => return None,
        };
        for child in children {
            match self.layers.get(child).map(|l| &l.kind) {
                Some(LayerKind::Raster(_)) => return Some(child),
                Some(LayerKind::Group(_)) => {
                    if let Some(found) = self.first_paintable_descendant(child) {
                        return Some(found);
                    }
                }
                _ => {}
            }
        }
        None
    }

    /// The visible layer rows in panel order (top-to-bottom pre-order): each
    /// root entry, descending into non-collapsed groups. Masks are owner-
    /// attached sub-rows (not in the z-order) and are excluded — they are not
    /// range-selectable or groupable. Drives `select_range` + `group_selected`.
    pub(crate) fn visible_row_order(&self) -> Vec<RtLayerId> {
        fn walk(stack: &LayerStack, ids: &[RtLayerId], out: &mut Vec<RtLayerId>) {
            for &id in ids {
                out.push(id);
                if let Some(LayerKind::Group(g)) = stack.get(id).map(|l| &l.kind)
                    && !g.collapsed
                {
                    walk(stack, &g.children, out);
                }
            }
        }
        let mut out = Vec::new();
        walk(&self.layers, self.layers.root(), &mut out);
        out
    }

    /// The current multi-selection folded with the active layer — the set of
    /// rows the panel highlights. Published each frame by the bridge
    /// (`set_current_selection`). Always includes the active layer (so a fresh
    /// tool with an empty `selection` still highlights its active row).
    #[must_use]
    pub fn selection(&self) -> std::collections::BTreeSet<RtLayerId> {
        let mut s = self.selection.clone();
        if let Some(a) = self.layers.active() {
            s.insert(a);
        }
        s
    }
}
