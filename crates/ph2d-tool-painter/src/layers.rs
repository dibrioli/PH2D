//! Layer stack data model (W3.T3.1) — `docs/Painter_projeto/02_layers.md`.
//!
//! Pure data + operations: create / reorder / nest / visibility / opacity /
//! blend-mode / active-selection. The actual pixel buffers (RGBA8 raster,
//! R8 mask) live in the tool's canvas + the GPU `LayerCache`; this model
//! holds only metadata (dimensions + handles + flags), so it stays cheap
//! to clone, diff, serialize, and reason about in tests.
//!
//! # Kinds vs modifiers
//!
//! The design doc (§2.1) lists *Clipping Mask*, *Reference*, and
//! *Alpha-lock* alongside Raster/Group/Mask, but it also states each is a
//! **modifier of a raster layer**, not a standalone bitmap. We model that
//! faithfully: [`LayerKind`] has the three real kinds (Raster, Group,
//! Mask), and the modifiers are boolean flags on [`Layer`]. *Adjustment*
//! layers are W4 (§7) and intentionally absent here.
//!
//! # Z-order
//!
//! [`LayerStack::root`] and [`GroupLayer::children`] are ordered
//! **top-to-bottom** (index 0 = topmost, matching the layer panel). The
//! compositor walks them in reverse (bottom-up) per §2.11.

use ph2d_painter_brush::BlendMode;
use serde::{Deserialize, Serialize};

/// Maximum group nesting depth (§2.6). A would-be level-9 group folds to
/// level 8 (the deeper insert is rejected).
pub const MAX_GROUP_DEPTH: usize = 8;

/// Hard cap on total layers per canvas (§2.5), mirrors Procreate. The
/// dynamic budget (`f(dimensions, format, MemoryBudget)`) clamps below
/// this; the stack itself only enforces the hard ceiling.
pub const HARD_CAP_LAYERS: usize = 999;

/// Stable per-canvas layer identity. Allocated monotonically by
/// [`LayerStack`]; never reused within a stack's lifetime so stale handles
/// (undo, cache keys) resolve unambiguously.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct LayerId(pub u64);

/// Bitmap raster layer (RGBA8 / RGBA16F per canvas profile). Pixels live
/// in the tool canvas + GPU cache; the model holds dimensions only.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RasterLayer {
    pub width: u32,
    pub height: u32,
}

/// Grayscale (R8) mask bound to a parent raster layer (§2.7). White =
/// visible, black = hidden; multiplies the parent's alpha.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MaskLayer {
    pub width: u32,
    pub height: u32,
    /// `Invert mask` toggle — composite uses `1 - value` when set.
    pub inverted: bool,
}

/// Container grouping N child layers (§2.1). Applies its blend-mode +
/// opacity to the composited child stack.
#[derive(Clone, Debug, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct GroupLayer {
    /// Child ids, top-to-bottom (same convention as [`LayerStack::root`]).
    pub children: Vec<LayerId>,
    pub collapsed: bool,
}

/// The three real layer kinds. Modifiers (clip/reference/alpha-lock) are
/// flags on [`Layer`], not kinds (see module docs).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum LayerKind {
    Raster(RasterLayer),
    Mask(MaskLayer),
    Group(GroupLayer),
}

/// A single layer: identity + kind + composite params + modifier flags.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Layer {
    pub id: LayerId,
    pub name: String,
    pub kind: LayerKind,
    pub blend_mode: BlendMode,
    /// Layer opacity in `[0, 1]`.
    pub opacity: f32,
    pub visible: bool,
    /// `Lock` — blocks edits (paint/transform) but still composites.
    pub locked: bool,
    /// Alpha-lock modifier (§2.10) — paint restricted to existing alpha.
    pub alpha_locked: bool,
    /// Clipping-mask modifier (§2.8) — clips to the layer directly below.
    pub clipping: bool,
    /// Reference-layer modifier (§2.9) — geometry source for ColorDrop.
    pub is_reference: bool,
    /// Optional grayscale mask child (§2.7).
    pub mask: Option<LayerId>,
}

impl Layer {
    fn new(id: LayerId, name: impl Into<String>, kind: LayerKind) -> Self {
        Self {
            id,
            name: name.into(),
            kind,
            blend_mode: BlendMode::Normal,
            opacity: 1.0,
            visible: true,
            locked: false,
            alpha_locked: false,
            clipping: false,
            is_reference: false,
            mask: None,
        }
    }

    #[must_use]
    pub fn is_group(&self) -> bool {
        matches!(self.kind, LayerKind::Group(_))
    }
}

/// The layer stack for one canvas. A flat arena (`arena`) keyed by
/// [`LayerId`], plus the top-level z-order (`root`); groups reference
/// their children by id. This keeps reorder/nest cheap and lets the
/// compositor walk the tree recursively without moving pixel data.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct LayerStack {
    arena: Vec<Layer>,
    root: Vec<LayerId>,
    active: Option<LayerId>,
    next_id: u64,
}

impl Default for LayerStack {
    fn default() -> Self {
        Self::new()
    }
}

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

    fn alloc_id(&mut self) -> LayerId {
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

    /// Set the primary selection. No-op if `id` is unknown.
    pub fn set_active(&mut self, id: LayerId) {
        if self.index_of(id).is_some() {
            self.active = Some(id);
        }
    }

    fn index_of(&self, id: LayerId) -> Option<usize> {
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
            .and_then(|list| list.iter().position(|&x| x == id).map(|pos| list.insert(pos, new_id)))
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
        }
    }

    pub fn set_opacity(&mut self, id: LayerId, opacity: f32) {
        if let Some(l) = self.get_mut(id) {
            l.opacity = opacity.clamp(0.0, 1.0);
        }
    }

    pub fn set_blend_mode(&mut self, id: LayerId, mode: BlendMode) {
        if let Some(l) = self.get_mut(id) {
            l.blend_mode = mode;
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
    fn parent_of(&self, id: LayerId) -> Option<LayerId> {
        self.arena.iter().find_map(|l| match &l.kind {
            LayerKind::Group(g) if g.children.contains(&id) => Some(l.id),
            _ => None,
        })
    }

    fn sibling_list_mut(&mut self, parent: Option<LayerId>) -> Option<&mut Vec<LayerId>> {
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
        // Depth check (audit W3): cap the moved item at depth
        // `MAX_GROUP_DEPTH - 1` (i.e. ≤ 8 levels, depths 0..=7). This matches
        // the FROZEN savefile validator (`ph2d-painter-stroke` rejects a Group
        // node at depth ≥ MAX_GROUP_DEPTH), so a runtime tree built here is
        // always saveable. (Was `> MAX_GROUP_DEPTH`, which allowed a 9th level
        // the savefile would reject — divergence flagged to the Coordinator.)
        if self.depth(group_id) + 1 >= MAX_GROUP_DEPTH {
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

    /// `true` if `maybe_descendant` is nested anywhere under `ancestor`.
    fn is_descendant(&self, maybe_descendant: LayerId, ancestor: LayerId) -> bool {
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

    fn collect_subtree(&self, id: LayerId, out: &mut Vec<LayerId>) {
        self.collect_subtree_bounded(id, out, 0);
    }

    /// Recursion body for [`Self::collect_subtree`], depth-bounded so a
    /// forged/deserialized stack that smuggles a cycle (a group listing an
    /// ancestor in `children`) past the runtime construction guards cannot
    /// stack-overflow `remove`. Mirrors the compositor's `composite_into`
    /// guard; the runtime API (`move_into_group`'s `is_descendant` check)
    /// already prevents building such a tree.
    fn collect_subtree_bounded(&self, id: LayerId, out: &mut Vec<LayerId>, depth: usize) {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn three_rasters_and_a_nested_group() {
        // T3.1 criterion 1: create 3 raster layers + 1 nested group.
        let mut s = LayerStack::new();
        let a = s.add_raster("Layer 1", 64, 64).unwrap();
        let b = s.add_raster("Layer 2", 64, 64).unwrap();
        let _c = s.add_raster("Layer 3", 64, 64).unwrap();
        let g = s.add_group("Group").unwrap();
        // Newest is on top: root = [Group, L3, L2, L1].
        assert_eq!(s.root().first(), Some(&g));
        assert_eq!(s.len(), 4);
        // Nest two rasters into the group.
        assert!(s.move_into_group(a, g));
        assert!(s.move_into_group(b, g));
        assert_eq!(s.depth(a), 1);
        assert_eq!(s.depth(g), 0);
        // a and b left the root; group + L3 remain at root.
        assert!(!s.root().contains(&a));
        assert!(!s.root().contains(&b));
        assert_eq!(s.root().len(), 2);
    }

    #[test]
    fn reorder_updates_stack_order() {
        // T3.1 criterion 2.
        let mut s = LayerStack::new();
        let a = s.add_raster("a", 8, 8).unwrap();
        let b = s.add_raster("b", 8, 8).unwrap();
        let c = s.add_raster("c", 8, 8).unwrap();
        // root top-to-bottom = [c, b, a]
        assert_eq!(s.root(), &[c, b, a]);
        // Move `a` to the top.
        s.reorder(a, 0);
        assert_eq!(s.root(), &[a, c, b]);
        // Move `a` back to the bottom.
        s.reorder(a, 2);
        assert_eq!(s.root(), &[c, b, a]);
    }

    #[test]
    fn visibility_opacity_blend_setters() {
        // T3.1 criterion 3 (+ opacity/blend).
        let mut s = LayerStack::new();
        let a = s.add_raster("a", 8, 8).unwrap();
        s.set_visible(a, false);
        assert!(!s.get(a).unwrap().visible);
        s.set_opacity(a, 1.5); // clamps
        assert_eq!(s.get(a).unwrap().opacity, 1.0);
        s.set_opacity(a, -0.2);
        assert_eq!(s.get(a).unwrap().opacity, 0.0);
        s.set_blend_mode(a, BlendMode::Multiply);
        assert_eq!(s.get(a).unwrap().blend_mode, BlendMode::Multiply);
    }

    #[test]
    fn add_raster_sets_active() {
        let mut s = LayerStack::new();
        let a = s.add_raster("a", 8, 8).unwrap();
        assert_eq!(s.active(), Some(a));
        let b = s.add_raster("b", 8, 8).unwrap();
        assert_eq!(s.active(), Some(b));
    }

    #[test]
    fn group_nesting_capped_at_8_levels_matching_savefile() {
        let mut s = LayerStack::new();
        let mut groups = Vec::new();
        for i in 0..MAX_GROUP_DEPTH + 2 {
            groups.push(s.add_group(format!("g{i}")).unwrap());
        }
        // Nest the chain as deep as the cap allows.
        let mut deepest = groups[0]; // depth 0
        for &g in &groups[1..] {
            if s.move_into_group(g, deepest) {
                deepest = g;
            } else {
                break;
            }
        }
        // Deepest group sits at depth MAX_GROUP_DEPTH-1 = 7 → 8 levels (0..=7),
        // matching the frozen savefile validator (rejects a Group at depth ≥ 8).
        assert_eq!(s.depth(deepest), MAX_GROUP_DEPTH - 1);
        // A further nest (would reach depth MAX_GROUP_DEPTH) is rejected.
        let extra = *groups.last().unwrap();
        assert!(
            !s.move_into_group(extra, deepest),
            "nesting past {MAX_GROUP_DEPTH} levels must be rejected (savefile parity)"
        );
    }

    #[test]
    fn remove_scrubs_dangling_mask_reference() {
        // audit W3: removing a layer that another layer references as its mask
        // must scrub the now-dangling ref (no `mask: Some(dead_id)`).
        let mut s = LayerStack::new();
        let owner = s.add_raster("owner", 8, 8).unwrap();
        let mask = s.add_raster("mask", 8, 8).unwrap();
        s.get_mut(owner).unwrap().mask = Some(mask);
        s.remove(mask);
        assert!(s.get(owner).is_some(), "owner survives mask removal");
        assert_eq!(
            s.get(owner).unwrap().mask,
            None,
            "dangling mask ref scrubbed"
        );
    }

    #[test]
    fn cannot_nest_group_into_its_own_descendant() {
        let mut s = LayerStack::new();
        let outer = s.add_group("outer").unwrap();
        let inner = s.add_group("inner").unwrap();
        assert!(s.move_into_group(inner, outer));
        // Moving `outer` into `inner` would create a cycle → rejected.
        assert!(!s.move_into_group(outer, inner));
    }

    #[test]
    fn remove_group_drops_subtree() {
        let mut s = LayerStack::new();
        let a = s.add_raster("a", 8, 8).unwrap();
        let g = s.add_group("g").unwrap();
        s.move_into_group(a, g);
        assert_eq!(s.len(), 2);
        s.remove(g);
        assert_eq!(s.len(), 0);
        assert_eq!(s.active(), None);
    }

    #[test]
    fn remove_active_repoints_active_to_root() {
        let mut s = LayerStack::new();
        let a = s.add_raster("a", 8, 8).unwrap();
        let b = s.add_raster("b", 8, 8).unwrap();
        s.set_active(b);
        s.remove(b);
        // active falls back to a root layer (a).
        assert_eq!(s.active(), Some(a));
    }

    #[test]
    fn ids_are_never_reused() {
        let mut s = LayerStack::new();
        let a = s.add_raster("a", 8, 8).unwrap();
        s.remove(a);
        let b = s.add_raster("b", 8, 8).unwrap();
        assert_ne!(a, b, "freed id must not be reused");
    }

    #[test]
    fn add_mask_binds_to_raster_and_is_out_of_zorder() {
        // T3.5: a mask attaches to its raster parent via `mask`, NOT into root.
        let mut s = LayerStack::new();
        let r = s.add_raster("r", 8, 8).unwrap();
        let m = s.add_mask(r).unwrap();
        assert_eq!(s.get(r).unwrap().mask, Some(m), "parent points at the mask");
        assert!(matches!(s.get(m).unwrap().kind, LayerKind::Mask(_)));
        assert!(!s.root().contains(&m), "mask is not in the z-order");
        assert_eq!(s.len(), 2, "both live in the arena");
    }

    #[test]
    fn add_mask_rejects_group_and_double_mask() {
        let mut s = LayerStack::new();
        let g = s.add_group("g").unwrap();
        assert!(s.add_mask(g).is_none(), "a group cannot have a mask");
        let r = s.add_raster("r", 8, 8).unwrap();
        assert!(s.add_mask(r).is_some());
        assert!(s.add_mask(r).is_none(), "no second mask on one raster");
    }

    #[test]
    fn remove_parent_also_removes_its_mask() {
        // T3.5 closes the old collect_subtree TODO: removing the owner must drop
        // its owner-attached mask too (else it leaks in the tool's images map).
        let mut s = LayerStack::new();
        let parent = s.add_raster("p", 4, 4).unwrap();
        let mask = s.add_mask(parent).unwrap();
        assert_eq!(s.len(), 2);
        s.remove(parent);
        assert_eq!(s.len(), 0, "parent + mask both removed");
        assert!(s.get(mask).is_none(), "mask did not leak");
    }

    #[test]
    fn reference_layer_is_exclusive() {
        // §2.9: only one reference layer per canvas — setting a new one clears
        // the previous.
        let mut s = LayerStack::new();
        let a = s.add_raster("a", 4, 4).unwrap();
        let b = s.add_raster("b", 4, 4).unwrap();
        s.set_reference(a, true);
        assert!(s.get(a).unwrap().is_reference);
        s.set_reference(b, true);
        assert!(s.get(b).unwrap().is_reference);
        assert!(!s.get(a).unwrap().is_reference, "previous reference cleared");
        s.set_reference(b, false);
        assert!(!s.get(b).unwrap().is_reference, "toggled off");
    }

    #[test]
    fn alpha_lock_and_clipping_flags_round_trip() {
        let mut s = LayerStack::new();
        let r = s.add_raster("r", 4, 4).unwrap();
        s.set_alpha_locked(r, true);
        s.set_clipping(r, true);
        assert!(s.get(r).unwrap().alpha_locked);
        assert!(s.get(r).unwrap().clipping);
        s.set_alpha_locked(r, false);
        assert!(!s.get(r).unwrap().alpha_locked);
    }

    #[test]
    fn remove_mask_alone_scrubs_parent_ref() {
        let mut s = LayerStack::new();
        let parent = s.add_raster("p", 4, 4).unwrap();
        let mask = s.add_mask(parent).unwrap();
        s.remove(mask);
        assert!(s.get(parent).is_some(), "parent survives mask removal");
        assert_eq!(s.get(parent).unwrap().mask, None, "dangling mask ref scrubbed");
    }
}
