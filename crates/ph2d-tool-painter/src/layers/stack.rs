//! `impl LayerStack` (the layer-tree mutation/query API), split out of the
//! former `layers.rs` (pure mechanical move).

use super::*;

impl LayerStack {
    #[must_use]
    pub fn new() -> Self {
        Self {
            arena: Vec::new(),
            root: Vec::new(),
            active: None,
            next_id: 1,
        }
    }

    pub(crate) fn alloc_id(&mut self) -> LayerId {
        let id = LayerId(self.next_id);
        self.next_id += 1;
        id
    }

    /// Total layer count (including groups + their nested children).
    #[must_use]
    pub fn len(&self) -> usize {
        self.arena.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.arena.is_empty()
    }

    /// Top-level layer ids, top-to-bottom.
    #[must_use]
    pub fn root(&self) -> &[LayerId] {
        &self.root
    }

    /// Iterate every layer id in the arena (groups + nested children
    /// included), in arena order. Used by `PainterTool::handle_panel_event` to
    /// decode a per-row widget [`NodeId`] back to its layer.
    pub fn all_ids(&self) -> impl Iterator<Item = LayerId> + '_ {
        self.arena.iter().map(|l| l.id)
    }

    #[must_use]
    pub fn active(&self) -> Option<LayerId> {
        self.active
    }

    /// Modifier flags of the active layer — what the layers panel's modifier
    /// toolbar paints its toggle state from. `None` if there's no active layer.
    #[must_use]
    pub fn active_modifiers(&self) -> Option<LayerModifiers> {
        let l = self.get(self.active?)?;
        Some(LayerModifiers {
            is_raster: matches!(l.kind, LayerKind::Raster(_)),
            has_mask: l.mask.is_some(),
            clipping: l.clipping,
            alpha_locked: l.alpha_locked,
            is_reference: l.is_reference,
        })
    }

    /// `true` if `id` is a mask layer — owner-attached, not in the z-order, so
    /// the panel excludes it from the draggable row-set (a mask row is
    /// selectable but never drag-reparented).
    #[must_use]
    pub fn is_mask(&self, id: LayerId) -> bool {
        matches!(self.get(id).map(|l| &l.kind), Some(LayerKind::Mask(_)))
    }

    /// Set the primary selection. No-op if `id` is unknown.
    pub fn set_active(&mut self, id: LayerId) {
        if self.index_of(id).is_some() {
            self.active = Some(id);
        }
    }

    pub(crate) fn index_of(&self, id: LayerId) -> Option<usize> {
        self.arena.iter().position(|l| l.id == id)
    }

    #[must_use]
    pub fn get(&self, id: LayerId) -> Option<&Layer> {
        self.index_of(id).map(|i| &self.arena[i])
    }

    #[must_use]
    pub fn get_mut(&mut self, id: LayerId) -> Option<&mut Layer> {
        self.index_of(id).map(|i| &mut self.arena[i])
    }

    /// Add a raster layer at the **top** of the root stack and make it
    /// active. Returns `None` if the hard cap is reached.
    pub fn add_raster(
        &mut self,
        name: impl Into<String>,
        width: u32,
        height: u32,
    ) -> Option<LayerId> {
        if self.arena.len() >= HARD_CAP_LAYERS {
            return None;
        }
        let id = self.alloc_id();
        self.arena.push(Layer::new(
            id,
            name,
            LayerKind::Raster(RasterLayer { width, height }),
        ));
        self.root.insert(0, id);
        self.active = Some(id);
        Some(id)
    }

    /// Add a non-destructive adjustment layer (W4, ADR-0045) at the top of the
    /// root stack, neutral (no-op) for `kind`, and make it active. The outer
    /// [`Layer`] mirrors the inner [`AdjustmentLayer`]'s defaults at creation;
    /// the inner payload is authoritative thereafter (amendment-1). Returns
    /// `None` at the hard cap. The compositor applies it to the layers below.
    pub fn add_adjustment(
        &mut self,
        kind: ph2d_painter_effects::adjustments::AdjustmentKind,
    ) -> Option<LayerId> {
        if self.arena.len() >= HARD_CAP_LAYERS {
            return None;
        }
        let id = self.alloc_id();
        let name = format!("{kind:?}");
        let adj = AdjustmentLayer::new(id.0, name.clone(), kind);
        self.arena
            .push(Layer::new(id, name, LayerKind::Adjustment(adj)));
        self.root.insert(0, id);
        self.active = Some(id);
        Some(id)
    }

    /// Mutable access to an adjustment layer's payload (for the UI to edit its
    /// `params` / opacity / blend / mask). `None` if `id` is not an adjustment.
    pub fn adjustment_mut(&mut self, id: LayerId) -> Option<&mut AdjustmentLayer> {
        match &mut self.get_mut(id)?.kind {
            LayerKind::Adjustment(adj) => Some(adj),
            _ => None,
        }
    }

    /// Add an empty group at the top of the root stack. Returns `None` if
    /// the hard cap is reached.
    pub fn add_group(&mut self, name: impl Into<String>) -> Option<LayerId> {
        if self.arena.len() >= HARD_CAP_LAYERS {
            return None;
        }
        let id = self.alloc_id();
        self.arena.push(Layer::new(
            id,
            name,
            LayerKind::Group(GroupLayer::default()),
        ));
        self.root.insert(0, id);
        Some(id)
    }

    /// Create a grayscale mask bound to raster `parent` (§2.7) and return its
    /// id. The mask is NOT inserted into the z-order (`root` / group children) —
    /// it composites *through* its parent (multiplying the parent's alpha), so
    /// it lives in the arena referenced only by `parent.mask`. Rejected
    /// (`None`) if `parent` is unknown, isn't a raster, already has a mask, or
    /// the hard cap is reached. Does NOT change the active selection — the tool
    /// decides the edit target and allocates the (white) pixel buffer.
    pub fn add_mask(&mut self, parent: LayerId) -> Option<LayerId> {
        let (w, h) = match self.get(parent) {
            Some(Layer {
                kind: LayerKind::Raster(r),
                mask: None,
                ..
            }) => (r.width, r.height),
            _ => return None,
        };
        if self.arena.len() >= HARD_CAP_LAYERS {
            return None;
        }
        let id = self.alloc_id();
        self.arena.push(Layer::new(
            id,
            "Mask",
            LayerKind::Mask(MaskLayer {
                width: w,
                height: h,
                inverted: false,
            }),
        ));
        // Owner-attached: referenced via `parent.mask`, never in a sibling list.
        if let Some(p) = self.get_mut(parent) {
            p.mask = Some(id);
        }
        Some(id)
    }

    /// Toggle a mask's `Invert mask` flag (§2.7) — the compositor uses
    /// `1 - value`. No-op if `id` is unknown or not a mask.
    pub fn set_mask_inverted(&mut self, id: LayerId, inverted: bool) {
        if let Some(Layer {
            kind: LayerKind::Mask(m),
            ..
        }) = self.get_mut(id)
        {
            m.inverted = inverted;
        }
    }

    /// Duplicate `id` immediately ABOVE itself in its parent list, returning the
    /// new id and making it active. Clones the layer's metadata (the name gains
    /// " copy"). The mask child + reference flag are NOT copied (mask
    /// duplication needs a fresh buffer — a follow-up; reference is exclusive).
    /// The caller copies the pixel buffer. `None` at the cap or unknown `id`.
    pub fn duplicate(&mut self, id: LayerId) -> Option<LayerId> {
        if self.arena.len() >= HARD_CAP_LAYERS {
            return None;
        }
        let mut copy = self.get(id)?.clone();
        let new_id = self.alloc_id();
        copy.id = new_id;
        copy.name = format!("{} copy", copy.name);
        copy.mask = None;
        copy.is_reference = false;
        self.arena.push(copy);
        // Insert just above the source (index 0 = top, so `pos` itself is "above").
        let parent = self.parent_of(id);
        let inserted = self
            .sibling_list_mut(parent)
            .and_then(|list| {
                list.iter()
                    .position(|&x| x == id)
                    .map(|pos| list.insert(pos, new_id))
            })
            .is_some();
        if !inserted {
            self.arena.pop(); // orphan cleanup (source had no parent list — unreachable)
            return None;
        }
        self.active = Some(new_id);
        Some(new_id)
    }

    pub fn set_visible(&mut self, id: LayerId, visible: bool) {
        if let Some(l) = self.get_mut(id) {
            l.visible = visible;
            // Adjustment layers are inner-authoritative (amendment-1): mirror
            // the field into the payload so the compositor honors the change.
            if let LayerKind::Adjustment(adj) = &mut l.kind {
                adj.visible = visible;
            }
        }
    }

    pub fn set_opacity(&mut self, id: LayerId, opacity: f32) {
        if let Some(l) = self.get_mut(id) {
            let v = opacity.clamp(0.0, 1.0);
            l.opacity = v;
            if let LayerKind::Adjustment(adj) = &mut l.kind {
                adj.opacity = v;
            }
        }
    }

    pub fn set_blend_mode(&mut self, id: LayerId, mode: BlendMode) {
        if let Some(l) = self.get_mut(id) {
            l.blend_mode = mode;
            if let LayerKind::Adjustment(adj) = &mut l.kind {
                adj.blend_mode = mode;
            }
        }
    }

    /// Set a layer's clipping-mask modifier (§2.8) — non-destructive; the
    /// compositor clips it to the nearest non-clipping raster below. No-op if
    /// `id` is unknown.
    pub fn set_clipping(&mut self, id: LayerId, clipping: bool) {
        if let Some(l) = self.get_mut(id) {
            l.clipping = clipping;
        }
    }

    /// Set a layer's alpha-lock modifier (§2.10) — paint restricted to existing
    /// alpha. No-op if `id` is unknown.
    pub fn set_alpha_locked(&mut self, id: LayerId, locked: bool) {
        if let Some(l) = self.get_mut(id) {
            l.alpha_locked = locked;
        }
    }

    /// Set a layer's reference modifier (§2.9). Only ONE reference layer per
    /// canvas: setting `id` as reference clears `is_reference` on every other
    /// layer. No-op if `id` is unknown.
    pub fn set_reference(&mut self, id: LayerId, is_reference: bool) {
        if self.index_of(id).is_none() {
            return;
        }
        if is_reference {
            // Exclusive — exactly one reference layer at a time.
            for l in &mut self.arena {
                l.is_reference = l.id == id;
            }
        } else if let Some(l) = self.get_mut(id) {
            l.is_reference = false;
        }
    }

    /// The ordered list (top-to-bottom) of the parent that directly holds
    /// `id`: either a group's children or the root. Returns the owning
    /// group id (`None` = root) for callers that need to walk upward.
    pub(crate) fn parent_of(&self, id: LayerId) -> Option<LayerId> {
        self.arena.iter().find_map(|l| match &l.kind {
            LayerKind::Group(g) if g.children.contains(&id) => Some(l.id),
            _ => None,
        })
    }

    pub(crate) fn sibling_list_mut(
        &mut self,
        parent: Option<LayerId>,
    ) -> Option<&mut Vec<LayerId>> {
        match parent {
            None => Some(&mut self.root),
            Some(pid) => {
                let i = self.index_of(pid)?;
                match &mut self.arena[i].kind {
                    LayerKind::Group(g) => Some(&mut g.children),
                    _ => None,
                }
            }
        }
    }

    /// `(index, sibling_count)` of `id` within its parent (root or group).
    /// `None` if `id` is unknown. Used by the layers panel to enable/disable
    /// the per-row move-up/down (↑↓) reorder buttons at the list edges.
    #[must_use]
    pub fn sibling_pos(&self, id: LayerId) -> Option<(usize, usize)> {
        let list: &[LayerId] = match self.parent_of(id) {
            None => &self.root,
            Some(pid) => match &self.arena[self.index_of(pid)?].kind {
                LayerKind::Group(g) => &g.children,
                _ => return None,
            },
        };
        let idx = list.iter().position(|&x| x == id)?;
        Some((idx, list.len()))
    }

    /// Move `id` one step toward the FRONT (top of z-order, index 0) within its
    /// parent. No-op if already first or unknown.
    pub fn move_up(&mut self, id: LayerId) {
        if let Some((i, _)) = self.sibling_pos(id)
            && i > 0
        {
            self.reorder(id, i - 1);
        }
    }

    /// Move `id` one step toward the BACK (bottom of z-order) within its
    /// parent. No-op if already last or unknown.
    pub fn move_down(&mut self, id: LayerId) {
        if let Some((i, n)) = self.sibling_pos(id)
            && i + 1 < n
        {
            self.reorder(id, i + 1);
        }
    }

    /// Reorder `id` to `new_index` within its current parent (root or
    /// group). Clamps to the sibling count. No-op if `id` is unknown.
    pub fn reorder(&mut self, id: LayerId, new_index: usize) {
        let parent = self.parent_of(id);
        let Some(list) = self.sibling_list_mut(parent) else {
            return;
        };
        let Some(from) = list.iter().position(|&x| x == id) else {
            return;
        };
        let to = new_index.min(list.len().saturating_sub(1));
        if from == to {
            return;
        }
        let v = list.remove(from);
        list.insert(to, v);
    }

    /// Nesting depth of `id` (0 = top-level / root child). Walks the group
    /// chain upward.
    #[must_use]
    pub fn depth(&self, id: LayerId) -> usize {
        let mut depth = 0;
        let mut cur = id;
        while let Some(parent) = self.parent_of(cur) {
            depth += 1;
            cur = parent;
            if depth > MAX_GROUP_DEPTH {
                break; // defensive: never loop on a malformed cycle
            }
        }
        depth
    }

    /// Move `id` into `group_id` (appended to the top of the group's
    /// children). Rejected — returning `false` — if the move would exceed
    /// [`MAX_GROUP_DEPTH`] (§2.6 fold-to-8), if either id is unknown, if
    /// `group_id` isn't a group, or if `id == group_id`.
    pub fn move_into_group(&mut self, id: LayerId, group_id: LayerId) -> bool {
        if id == group_id || self.index_of(id).is_none() {
            return false;
        }
        // Target must be a group, and must not be a descendant of `id`
        // (that would orphan the subtree / create a cycle).
        if !matches!(
            self.get(group_id).map(|l| &l.kind),
            Some(LayerKind::Group(_))
        ) {
            return false;
        }
        if self.is_descendant(group_id, id) {
            return false;
        }
        // Depth check (audit W3): the moved item lands at depth
        // `depth(group_id) + 1`, and its OWN SUBTREE extends `subtree_height(id)`
        // deeper — so the deepest descendant sits at `depth(group_id) + 1 +
        // subtree_height(id)`. Reject when THAT would reach `MAX_GROUP_DEPTH`
        // (depths valid 0..=MAX_GROUP_DEPTH-1), matching the FROZEN savefile
        // validator (`ph2d-painter-stroke::validate_layer_node` rejects a Group
        // node at depth ≥ MAX_GROUP_DEPTH), so a runtime tree built here is
        // ALWAYS saveable. (Audit 2026-06-18: previously omitted `subtree_height`
        // — mirroring only `move_to_sibling_of`'s leaf case — which let a
        // reparent-into-group drag build an unsaveable >8-deep tree.)
        if self.depth(group_id) + 1 + self.subtree_height(id) >= MAX_GROUP_DEPTH {
            return false;
        }
        // Detach from current parent.
        let parent = self.parent_of(id);
        if let Some(list) = self.sibling_list_mut(parent)
            && let Some(pos) = list.iter().position(|&x| x == id)
        {
            list.remove(pos);
        }
        // Attach to the group (top of its children).
        if let Some(gi) = self.index_of(group_id)
            && let LayerKind::Group(g) = &mut self.arena[gi].kind
        {
            g.children.insert(0, id);
            return true;
        }
        false
    }

    /// Nesting height of `id`'s subtree (0 = leaf, 1 = group of leaves, …).
    /// Bounded by [`MAX_GROUP_DEPTH`] (defense vs a forged deep/cyclic tree).
    pub(crate) fn subtree_height(&self, id: LayerId) -> usize {
        self.subtree_height_bounded(id, 0)
    }

    pub(crate) fn subtree_height_bounded(&self, id: LayerId, depth: usize) -> usize {
        if depth > MAX_GROUP_DEPTH {
            return 0;
        }
        match self.get(id) {
            Some(Layer {
                kind: LayerKind::Group(g),
                ..
            }) if !g.children.is_empty() => {
                1 + g
                    .children
                    .iter()
                    .map(|&c| self.subtree_height_bounded(c, depth + 1))
                    .max()
                    .unwrap_or(0)
            }
            _ => 0,
        }
    }

    /// Move `moved` to sit directly before/after `target` in TARGET's parent
    /// list (drag-reorder / cross-parent reparent). Detaches `moved` first.
    /// Rejected (`false`) if either is unknown, `moved` is the base sprite
    /// (pinned at root bottom), `target` is a descendant of `moved` (cycle), or
    /// the move would exceed [`MAX_GROUP_DEPTH`]. Keeps the base sprite last in
    /// root (never inserts a layer below it).
    pub fn move_to_sibling_of(&mut self, moved: LayerId, target: LayerId, after: bool) -> bool {
        if moved == target || self.index_of(moved).is_none() || self.index_of(target).is_none() {
            return false;
        }
        if self.root.last() == Some(&moved) {
            return false; // the base sprite never moves
        }
        if self.is_descendant(target, moved) {
            return false; // can't move into your own subtree
        }
        let target_parent = self.parent_of(target);
        if self.depth(target) + self.subtree_height(moved) >= MAX_GROUP_DEPTH {
            return false; // moved's subtree wouldn't fit at target's depth
        }
        // Detach `moved` from its current parent.
        let from = self.parent_of(moved);
        if let Some(list) = self.sibling_list_mut(from)
            && let Some(p) = list.iter().position(|&x| x == moved)
        {
            list.remove(p);
        }
        // Insert before/after `target` in target's parent list (find pos AFTER
        // the detach so same-list index shifts are accounted for).
        let to_root = target_parent.is_none();
        let Some(list) = self.sibling_list_mut(target_parent) else {
            self.root.insert(0, moved); // target's parent vanished — don't lose `moved`
            return false;
        };
        let Some(target_pos) = list.iter().position(|&x| x == target) else {
            self.root.insert(0, moved);
            return false;
        };
        let mut idx = if after { target_pos + 1 } else { target_pos };
        if to_root {
            // The base sprite occupies the last root slot — never insert past it.
            idx = idx.min(list.len().saturating_sub(1));
        }
        list.insert(idx, moved);
        true
    }

    /// Move `moved` to the bottom of root, just ABOVE the base sprite (which
    /// stays pinned last). Used for a drop in the empty space below the list.
    /// Rejected (`false`) if `moved` is unknown or is the base itself.
    pub fn move_to_root_bottom_above_base(&mut self, moved: LayerId) -> bool {
        if self.index_of(moved).is_none() || self.root.last() == Some(&moved) {
            return false;
        }
        let from = self.parent_of(moved);
        if let Some(list) = self.sibling_list_mut(from)
            && let Some(p) = list.iter().position(|&x| x == moved)
        {
            list.remove(p);
        }
        let idx = self.root.len().saturating_sub(1); // just above the base
        self.root.insert(idx, moved);
        true
    }

    /// `true` if `maybe_descendant` is nested anywhere under `ancestor`.
    pub(crate) fn is_descendant(&self, maybe_descendant: LayerId, ancestor: LayerId) -> bool {
        let mut cur = maybe_descendant;
        let mut steps = 0;
        while let Some(parent) = self.parent_of(cur) {
            if parent == ancestor {
                return true;
            }
            cur = parent;
            steps += 1;
            if steps > MAX_GROUP_DEPTH {
                break;
            }
        }
        false
    }

    /// Remove `id` (and, if it's a group, its whole subtree). Clears the
    /// active selection if it pointed at a removed layer.
    pub fn remove(&mut self, id: LayerId) {
        // Collect the subtree ids first.
        let mut to_remove = Vec::new();
        self.collect_subtree(id, &mut to_remove);
        // Detach `id` from its parent list.
        let parent = self.parent_of(id);
        if let Some(list) = self.sibling_list_mut(parent)
            && let Some(pos) = list.iter().position(|&x| x == id)
        {
            list.remove(pos);
        }
        // Drop from arena.
        self.arena.retain(|l| !to_remove.contains(&l.id));
        // NOTE: `to_remove` was gathered by `collect_subtree`, which is
        // depth-bounded (defense-in-depth vs a forged/deserialized cycle), so
        // this never loops or over-collects even on a malformed tree.
        // Scrub any dangling mask reference pointing at a removed layer
        // (audit W3: a mask child is part of the owner's subtree, but a mask
        // removed independently must not leave its owner pointing at a dead id).
        for layer in &mut self.arena {
            if layer.mask.is_some_and(|m| to_remove.contains(&m)) {
                layer.mask = None;
            }
        }
        if self.active.is_some_and(|a| to_remove.contains(&a)) {
            self.active = self.root.first().copied();
        }
    }

    pub(crate) fn collect_subtree(&self, id: LayerId, out: &mut Vec<LayerId>) {
        self.collect_subtree_bounded(id, out, 0);
    }

    /// Recursion body for [`Self::collect_subtree`], depth-bounded so a
    /// forged/deserialized stack that smuggles a cycle (a group listing an
    /// ancestor in `children`) past the runtime construction guards cannot
    /// stack-overflow `remove`. Mirrors the compositor's `composite_into`
    /// guard; the runtime API (`move_into_group`'s `is_descendant` check)
    /// already prevents building such a tree.
    pub(crate) fn collect_subtree_bounded(
        &self,
        id: LayerId,
        out: &mut Vec<LayerId>,
        depth: usize,
    ) {
        out.push(id);
        if depth > MAX_GROUP_DEPTH {
            return;
        }
        // T3.5: an attached mask is owner-attached (referenced via `mask`, not in
        // any sibling list), so it's only reachable here — collect it so removing
        // the owner removes the mask too (no leak in the tool's `images` map).
        if let Some(mask_id) = self.get(id).and_then(|l| l.mask) {
            out.push(mask_id);
        }
        if let Some(Layer {
            kind: LayerKind::Group(g),
            ..
        }) = self.get(id)
        {
            for &child in &g.children {
                self.collect_subtree_bounded(child, out, depth + 1);
            }
        }
    }
}
