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
    pub fn add_raster(&mut self, name: impl Into<String>, width: u32, height: u32) -> Option<LayerId> {
        if self.arena.len() >= HARD_CAP_LAYERS {
            return None;
        }
        let id = self.alloc_id();
        self.arena
            .push(Layer::new(id, name, LayerKind::Raster(RasterLayer { width, height })));
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
        self.arena
            .push(Layer::new(id, name, LayerKind::Group(GroupLayer::default())));
        self.root.insert(0, id);
        Some(id)
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
        if !matches!(self.get(group_id).map(|l| &l.kind), Some(LayerKind::Group(_))) {
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
        out.push(id);
        // TODO(W3.T3.5 mask wiring): when masks become creatable, a layer's
        // mask child must be collected here too (else removing the owner leaks
        // its mask). Deferred — the mask's structural membership (root vs
        // owner-attached) is undefined until T3.5 designs mask creation; the
        // dangling-owner-ref case is already handled by `remove`'s scrub.
        if let Some(Layer { kind: LayerKind::Group(g), .. }) = self.get(id) {
            for &child in &g.children {
                self.collect_subtree(child, out);
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
        assert_eq!(s.get(owner).unwrap().mask, None, "dangling mask ref scrubbed");
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
}
