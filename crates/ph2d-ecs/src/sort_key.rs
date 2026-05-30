//! Canonical sprite ordering pipeline — Sprite Inspector v2 W3.T3.8
//! (spec [`05_ordering_sorting.md`](../../../docs/Sprite_projeto/05_ordering_sorting.md)).
//!
//! This is the sorting equivalent of `propagate_transforms`: a single
//! deterministic pass that turns the scene tree + the optional sorting
//! components ([`crate::sorting`]) into a total render order. The
//! extract phase stamps the resulting rank onto `RenderInstance.z_order`.
//!
//! # The 7 canonical stages (spec §5.1)
//!
//! The [`SortKey`] is compared lexicographically. The fields encode the
//! fixed pipeline:
//!
//! 1. **Viewport** — camera separation. Single field, `0` until
//!    multi-viewport lands (spec §5.2: "not Sprite Inspector").
//! 2. **SortingLayer** (named) + **OrderInLayer** (micro within layer).
//! 3. **Z** (`ZIndexOverride` + `ZAsRelative` cascade).
//! 4. **YSort** (cascaded projected position).
//! 5. **SortingGroup** — handled by computing fields 2–4 at the *group
//!    root* so a multi-piece block sorts as one unit; internal order
//!    falls to `intra_z` + `draw_order`.
//! 6. **ShowBehindParent** — folded into `draw_order` (a child marked
//!    show-behind emits before its parent in the traversal).
//! 7. **DFS counter** — `draw_order` fallback.
//!
//! ## Z-before-YSort reconciliation (spec §5.1 vs §5.2)
//!
//! Spec §5.1 lists YSort (stage 3) before Z (stage 4), but §5.2 passo 4
//! states the *behavioral* rule explicitly: **"Z primeiro buckets,
//! dentro do bucket o YSort ordena"** (Godot semantics, which the spec
//! adopts wholesale). A divergent `ZIndexOverride` must *break* the
//! YSort cascade — that only happens if Z outranks YSort in the key. We
//! therefore order the key `… z, ysort …`, honoring the normative §5.2
//! passo-4 statement; §5.1's stage list is the conceptual enumeration,
//! and the Z/YSort relative priority is governed by §5.2. (Coord
//! decision under solo-mandate §7.2; flagged for ADR-0073 amendment.)
//!
//! Determinism (HR-5): every key field is an integer; the only float is
//! the YSort projection, quantized through `libm::roundf` so the result
//! is byte-identical cross-platform.

use bevy_ecs::entity::{Entity, EntityHashMap};
use bevy_ecs::hierarchy::ChildOf;
use bevy_ecs::world::World;
use ph2d_core::Vec2;

use crate::sorting::{
    OrderInLayer, ShowBehindParent, SortPoint, SortingGroup, SortingLayer, SortingLayers, TopLevel,
    YSort, ZAsRelative, ZIndexOverride,
};

/// Fixed-point scale for the YSort projection (1 unit ≈ 1/1024 m). The
/// projection is quantized to `i64` so the comparison never touches a
/// float — guaranteeing identical ordering on every platform.
const YSORT_SCALE: f32 = 1024.0;

/// One sprite's composed sort key. Compared lexicographically; a
/// smaller key draws *first* (furthest back). Derived `Ord` walks the
/// fields top-to-bottom, which is exactly the canonical pipeline.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct SortKey {
    /// Stage 1 — viewport / camera separation.
    pub viewport: u8,
    /// Stage 2a — named macro layer index (computed at the group root).
    pub sort_layer: i32,
    /// Stage 2b — manual micro-ordering within the layer (group root).
    pub order_in_layer: i32,
    /// Stage 3 — effective cascaded Z bucket (group root). Buckets
    /// before YSort so a divergent Z breaks the cascade (spec §5.2).
    pub z: i64,
    /// Stage 4 — quantized YSort projection (group root); `0` when no
    /// ancestor enables YSort.
    pub ysort: i64,
    /// Stage 5 internal — the member's *own* Z override, so pieces
    /// inside a SortingGroup block still respect a per-piece Z while the
    /// block as a whole sorts at the root's Z. `0` for the common case.
    pub intra_z: i64,
    /// Stages 6–7 — ShowBehindParent-aware DFS draw counter. Keeps each
    /// SortingGroup block contiguous (its subtree is visited as a unit).
    pub draw_order: i64,
}

/// Per-entity input gathered during the extract walk: the sprite
/// entity and the world position used for YSort projection.
#[derive(Copy, Clone, Debug)]
pub struct SortInput {
    pub entity: Entity,
    pub world_pos: Vec2,
}

/// Reusable scratch for [`compute_sort_ranks_into`]. Threaded across
/// frames like `WorklistBuf` so the extract hot path is allocation-free
/// after warm-up (HR-3): every buffer is `clear()`ed (capacity
/// retained), never reallocated, as long as the scene stays under the
/// warmed capacity. `children` holds *slot indices* (`u32`) rather than
/// per-parent `Vec`s of `Entity`, so clearing reuses the inner `Vec`
/// allocations instead of freeing them.
#[derive(Default)]
pub struct SortScratch {
    /// entity → its index in `inputs`.
    in_set: EntityHashMap<u32>,
    /// children slots keyed by parent slot (index = parent slot).
    children: Vec<Vec<u32>>,
    /// slots whose nearest in-set ancestor is none (draw-order roots).
    roots: Vec<u32>,
    /// emit traversal stack: `(slot, expand?)`.
    stack: Vec<(u32, bool)>,
    /// draw_order indexed by slot.
    draw_order: Vec<i64>,
    /// memo: entity → group-root entity.
    group_cache: EntityHashMap<Entity>,
    /// memo: entity → effective cascaded Z.
    z_cache: EntityHashMap<i64>,
    /// `(key, slot)` pairs to sort.
    keyed: Vec<(SortKey, u32)>,
    /// output `(entity, rank)`.
    out: Vec<(Entity, u32)>,
    /// entity → final render rank, for O(1) lookup while stamping
    /// `z_order` onto the present world.
    rank_by_entity: EntityHashMap<u32>,
}

impl SortScratch {
    /// Final render rank for `entity` (0 = furthest back), valid after
    /// the most recent [`compute_sort_ranks_into`].
    pub fn rank(&self, entity: Entity) -> Option<u32> {
        self.rank_by_entity.get(&entity).copied()
    }
}

impl SortScratch {
    pub fn new() -> Self {
        Self::default()
    }

    fn reset(&mut self, n: usize) {
        self.in_set.clear();
        // Keep the per-parent inner Vecs (clear, don't drop) to reuse
        // their capacity; grow the outer Vec only if the scene grew.
        if self.children.len() < n {
            self.children.resize_with(n, Vec::new);
        }
        for v in self.children.iter_mut().take(n) {
            v.clear();
        }
        self.roots.clear();
        self.stack.clear();
        self.draw_order.clear();
        self.draw_order.resize(n, 0);
        self.group_cache.clear();
        self.z_cache.clear();
        self.keyed.clear();
        self.out.clear();
        self.rank_by_entity.clear();
    }
}

/// Convenience wrapper that owns a one-shot [`SortScratch`]. Prefer
/// [`compute_sort_ranks_into`] on the hot path.
pub fn compute_sort_ranks(world: &World, inputs: &[SortInput]) -> Vec<(Entity, u32)> {
    let mut scratch = SortScratch::new();
    compute_sort_ranks_into(&mut scratch, world, inputs);
    scratch.out.clone()
}

/// Compute a total render order for `inputs`, reading the optional
/// sorting components off `world`, into `scratch.out` (`(entity, rank)`
/// pairs, `rank` 0 = furthest back). Stamp `rank` onto
/// `RenderInstance.z_order`.
///
/// `inputs` must be in scene DFS pre-order (as produced by
/// `propagate_transforms`); that order seeds the deterministic
/// `draw_order` and sibling resolution. Allocation-free after warm-up.
pub fn compute_sort_ranks_into(scratch: &mut SortScratch, world: &World, inputs: &[SortInput]) {
    let n = inputs.len();
    scratch.reset(n);
    for (i, s) in inputs.iter().enumerate() {
        scratch.in_set.insert(s.entity, i as u32);
    }

    let default_layer = world
        .get_resource::<SortingLayers>()
        .map_or(2, |l| l.default_index()) as i32;

    // --- draw_order: ShowBehindParent-aware DFS over the input set ---
    // Group each slot under the nearest in-set ancestor (full ChildOf
    // chain), preserving input order — that is the sibling order.
    for (i, s) in inputs.iter().enumerate() {
        match nearest_set_ancestor(world, s.entity, &scratch.in_set) {
            Some(parent_slot) => scratch.children[parent_slot as usize].push(i as u32),
            None => scratch.roots.push(i as u32),
        }
    }
    let mut counter: i64 = 0;
    // Seed roots in reverse so they pop (and emit) in input order.
    for &r in scratch.roots.iter().rev() {
        scratch.stack.push((r, true));
    }
    while let Some((slot, expand)) = scratch.stack.pop() {
        if expand {
            // Emission order per node:
            //   [show_behind child subtrees] [self] [normal child subtrees]
            // Pushed reversed because the work stack is LIFO. Partition
            // in place without allocating: count behind first.
            let kids_len = scratch.children[slot as usize].len();
            // Push normal children (reverse), then self-emit, then
            // show_behind children (reverse) — so popping yields
            // behind → self → normal.
            for idx in (0..kids_len).rev() {
                let c = scratch.children[slot as usize][idx];
                if world
                    .get::<ShowBehindParent>(inputs[c as usize].entity)
                    .is_none()
                {
                    scratch.stack.push((c, true));
                }
            }
            scratch.stack.push((slot, false));
            for idx in (0..kids_len).rev() {
                let c = scratch.children[slot as usize][idx];
                if world
                    .get::<ShowBehindParent>(inputs[c as usize].entity)
                    .is_some()
                {
                    scratch.stack.push((c, true));
                }
            }
        } else {
            scratch.draw_order[slot as usize] = counter;
            counter += 1;
        }
    }

    // --- per-entity key ---
    for (i, s) in inputs.iter().enumerate() {
        let e = s.entity;
        let anchor = group_root(world, e, &mut scratch.group_cache);
        let sort_layer = world
            .get::<SortingLayer>(anchor)
            .map_or(default_layer, |l| l.0.0 as i32);
        let order_in_layer = world.get::<OrderInLayer>(anchor).map_or(0, |o| o.0);
        let z = effective_z(world, anchor, &mut scratch.z_cache);
        let ysort = ysort_key(world, anchor, inputs, &scratch.in_set);
        let intra_z = world
            .get::<ZIndexOverride>(e)
            .map_or(0, |zo| ZIndexOverride::clamped(zo.0) as i64);
        let key = SortKey {
            viewport: 0,
            sort_layer,
            order_in_layer,
            z,
            ysort,
            intra_z,
            draw_order: scratch.draw_order[i],
        };
        scratch.keyed.push((key, i as u32));
    }

    // `draw_order` is unique per slot, so the key is a strict total
    // order; sort by it. Unstable is fine (no equal keys).
    scratch.keyed.sort_unstable_by_key(|(key, _)| *key);
    for rank in 0..scratch.keyed.len() {
        let slot = scratch.keyed[rank].1;
        let entity = inputs[slot as usize].entity;
        scratch.out.push((entity, rank as u32));
        scratch.rank_by_entity.insert(entity, rank as u32);
    }
}

/// Nearest strict ancestor of `e` that is itself in the sprite set
/// (`in_set`), walking the full `ChildOf` chain. `None` if no sprite
/// ancestor — i.e. `e` is a root within the set. A [`TopLevel`] entity
/// detaches: it has no draw-order parent (treated as a root).
fn nearest_set_ancestor(world: &World, e: Entity, in_set: &EntityHashMap<u32>) -> Option<u32> {
    if world.get::<TopLevel>(e).is_some() {
        return None;
    }
    let mut cur = world.get::<ChildOf>(e).map(|c| c.parent());
    while let Some(p) = cur {
        if let Some(slot) = in_set.get(&p) {
            return Some(*slot);
        }
        if world.get::<TopLevel>(p).is_some() {
            return None;
        }
        cur = world.get::<ChildOf>(p).map(|c| c.parent());
    }
    None
}

/// The entity whose layer/order/Z/YSort represent `e`'s sort block
/// (spec §5.5). Rule:
/// - the nearest ancestor-or-self carrying `SortingGroup { sort_at_root:
///   true }` (escape hatch — that node is its own block root), else
/// - the *outermost* ancestor-or-self carrying `SortingGroup` (the
///   block groups the whole subtree), else
/// - `e` itself (singleton).
///
/// The walk stops at a [`TopLevel`] boundary (a detached node is its own
/// root).
fn group_root(world: &World, e: Entity, cache: &mut EntityHashMap<Entity>) -> Entity {
    if let Some(c) = cache.get(&e) {
        return *c;
    }
    // Walk the ancestor-or-self chain (stopping at a TopLevel boundary)
    // twice without allocating: first looking for the nearest
    // `sort_at_root = true` (escape hatch), tracking the outermost
    // plain `SortingGroup` as we go for the fallback.
    let mut outermost: Option<Entity> = None;
    let mut node = Some(e);
    let mut detached_self = world.get::<TopLevel>(e).is_some();
    let mut result = None;
    while let Some(n) = node {
        if let Some(g) = world.get::<SortingGroup>(n) {
            if g.sort_at_root {
                result = Some(n);
                break;
            }
            outermost = Some(n);
        }
        // A TopLevel node is its own root: stop ascending past it.
        if detached_self || world.get::<TopLevel>(n).is_some() && n != e {
            break;
        }
        node = world.get::<ChildOf>(n).map(|c| c.parent());
        detached_self = false;
    }
    let result = result.or(outermost).unwrap_or(e);
    cache.insert(e, result);
    result
}

/// Effective cascaded Z for `entity` (spec §5.2 passo 4 + §5.9). With
/// `ZAsRelative(true)` (default) the effective Z adds the parent's
/// effective Z, saturating into the gateable range. Cascade stops at a
/// [`TopLevel`] boundary. Memoized.
fn effective_z(world: &World, entity: Entity, cache: &mut EntityHashMap<i64>) -> i64 {
    if let Some(z) = cache.get(&entity) {
        return *z;
    }
    let own = world
        .get::<ZIndexOverride>(entity)
        .map_or(0, |z| ZIndexOverride::clamped(z.0));
    let relative = world.get::<ZAsRelative>(entity).is_none_or(|r| r.0);
    let parent_z = if relative && world.get::<TopLevel>(entity).is_none() {
        match world.get::<ChildOf>(entity).map(|c| c.parent()) {
            Some(p) => effective_z(world, p, cache),
            None => 0,
        }
    } else {
        0
    };
    let eff = ZIndexOverride::clamped((own as i64).saturating_add(parent_z) as i32) as i64;
    cache.insert(entity, eff);
    eff
}

/// Quantized YSort key for `entity` (spec §5.2 passo 3). `0` unless a
/// strict ancestor (up to a [`TopLevel`] boundary) has
/// `YSort { enabled: true, .. }`. Higher projected position → larger
/// key → drawn in front.
fn ysort_key(
    world: &World,
    entity: Entity,
    inputs: &[SortInput],
    in_set: &EntityHashMap<u32>,
) -> i64 {
    if world.get::<TopLevel>(entity).is_some() {
        return 0;
    }
    let mut cur = world.get::<ChildOf>(entity).map(|c| c.parent());
    while let Some(p) = cur {
        if let Some(ys) = world.get::<YSort>(p)
            && ys.enabled
        {
            // Center / Pivot both project the world translation in W3
            // (precise pivot offset is a follow-up); Custom only changes
            // the axis. The position comes from the extract's
            // GlobalTransform (in `inputs`); an ancestor outside the
            // sprite set has no projected point → neutral 0.
            let point = match ys.sort_point {
                SortPoint::Center | SortPoint::Pivot | SortPoint::Custom => in_set
                    .get(&entity)
                    .map(|&slot| inputs[slot as usize].world_pos)
                    .unwrap_or(Vec2::ZERO),
            };
            let proj = point.x * ys.axis.x + point.y * ys.axis.y;
            return libm::roundf(proj * YSORT_SCALE) as i64;
        }
        if world.get::<TopLevel>(p).is_some() {
            break;
        }
        cur = world.get::<ChildOf>(p).map(|c| c.parent());
    }
    0
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sorting::SortPoint;
    use bevy_ecs::hierarchy::ChildOf;

    fn input(e: Entity, x: f32, y: f32) -> SortInput {
        SortInput {
            entity: e,
            world_pos: Vec2::new(x, y),
        }
    }

    /// Map entity → rank for ergonomic assertions.
    fn ranks(world: &World, inputs: &[SortInput]) -> EntityHashMap<u32> {
        compute_sort_ranks(world, inputs).into_iter().collect()
    }

    #[test]
    fn plain_dfs_fallback_preserves_input_order() {
        let mut w = World::new();
        let a = w.spawn_empty().id();
        let b = w.spawn_empty().id();
        let c = w.spawn_empty().id();
        let inputs = [input(a, 0.0, 0.0), input(b, 0.0, 0.0), input(c, 0.0, 0.0)];
        let r = ranks(&w, &inputs);
        assert_eq!(r[&a], 0);
        assert_eq!(r[&b], 1);
        assert_eq!(r[&c], 2);
    }

    #[test]
    fn sorting_layer_buckets_before_dfs() {
        let mut w = World::new();
        // `a` authored first but on a higher layer than `b`.
        let a = w.spawn(SortingLayer(crate::sorting::LayerId(4))).id();
        let b = w.spawn(SortingLayer(crate::sorting::LayerId(0))).id();
        let inputs = [input(a, 0.0, 0.0), input(b, 0.0, 0.0)];
        let r = ranks(&w, &inputs);
        assert!(r[&b] < r[&a], "lower layer index draws first");
    }

    #[test]
    fn z_override_buckets_before_ysort() {
        // YSort parent with two children; the higher-Y child would draw
        // in front, but giving the lower-Y child a higher Z must move it
        // in front (Z buckets before YSort — §5.2 passo 4).
        let mut w = World::new();
        let parent = w.spawn(YSort::default()).id();
        let low_y = w.spawn((ChildOf(parent), ZIndexOverride(10))).id();
        let high_y = w.spawn(ChildOf(parent)).id();
        let inputs = [
            input(parent, 0.0, 0.0),
            input(low_y, 0.0, 1.0),
            input(high_y, 0.0, 100.0),
        ];
        let r = ranks(&w, &inputs);
        assert!(
            r[&high_y] < r[&low_y],
            "the Z=10 child must draw in front despite lower Y"
        );
    }

    #[test]
    fn ysort_cascade_orders_children_by_y() {
        let mut w = World::new();
        let world_node = w.spawn(YSort::default()).id();
        let tree = w.spawn(ChildOf(world_node)).id();
        let player = w.spawn(ChildOf(world_node)).id();
        let rock = w.spawn(ChildOf(world_node)).id();
        // Authored order tree, player, rock; Y: tree=10, player=5, rock=15.
        let inputs = [
            input(world_node, 0.0, 0.0),
            input(tree, 0.0, 10.0),
            input(player, 0.0, 5.0),
            input(rock, 0.0, 15.0),
        ];
        let r = ranks(&w, &inputs);
        // player (y=5) behind, tree (y=10) middle, rock (y=15) front.
        assert!(r[&player] < r[&tree]);
        assert!(r[&tree] < r[&rock]);
    }

    #[test]
    fn ysort_custom_axis_projects_diagonally() {
        let mut w = World::new();
        let root = w
            .spawn(YSort {
                enabled: true,
                axis: Vec2::new(1.0, 1.0),
                sort_point: SortPoint::Custom,
            })
            .id();
        let a = w.spawn(ChildOf(root)).id(); // x+y = 2
        let b = w.spawn(ChildOf(root)).id(); // x+y = 10
        let inputs = [
            input(root, 0.0, 0.0),
            input(a, 1.0, 1.0),
            input(b, 5.0, 5.0),
        ];
        let r = ranks(&w, &inputs);
        assert!(r[&a] < r[&b], "larger diagonal projection draws in front");
    }

    #[test]
    fn show_behind_parent_draws_child_before_parent() {
        let mut w = World::new();
        let player = w.spawn_empty().id();
        let shadow = w.spawn((ChildOf(player), ShowBehindParent)).id();
        let body = w.spawn(ChildOf(player)).id();
        let inputs = [
            input(player, 0.0, 0.0),
            input(shadow, 0.0, 0.0),
            input(body, 0.0, 0.0),
        ];
        let r = ranks(&w, &inputs);
        // shadow before player before body.
        assert!(r[&shadow] < r[&player], "shadow draws behind the parent");
        assert!(r[&player] < r[&body]);
    }

    #[test]
    fn sorting_group_keeps_block_contiguous() {
        // A character group (root + 2 pieces) interleaved in authoring
        // with a world sprite that shares the default layer/Z. The
        // block must stay contiguous.
        let mut w = World::new();
        let char_root = w.spawn(SortingGroup::default()).id();
        let body = w.spawn(ChildOf(char_root)).id();
        let hat = w.spawn(ChildOf(char_root)).id();
        let world_sprite = w.spawn_empty().id();
        // Author order: char_root, body, hat, world_sprite.
        let inputs = [
            input(char_root, 0.0, 0.0),
            input(body, 0.0, 0.0),
            input(hat, 0.0, 0.0),
            input(world_sprite, 0.0, 0.0),
        ];
        let r = ranks(&w, &inputs);
        let block = [r[&char_root], r[&body], r[&hat]];
        let lo = *block.iter().min().unwrap();
        let hi = *block.iter().max().unwrap();
        assert_eq!(hi - lo, 2, "the 3 block members are contiguous ranks");
        // world_sprite is outside the contiguous block span.
        assert!(r[&world_sprite] < lo || r[&world_sprite] > hi);
    }

    #[test]
    fn sort_at_root_escapes_the_block() {
        // A descendant with SortingGroup{sort_at_root:true} on a high
        // layer must leave the parent block and sort by its own layer.
        let mut w = World::new();
        let char_root = w.spawn(SortingGroup::default()).id();
        let body = w.spawn(ChildOf(char_root)).id();
        let escapee = w
            .spawn((
                ChildOf(char_root),
                SortingGroup { sort_at_root: true },
                SortingLayer(crate::sorting::LayerId(0)),
            ))
            .id();
        // char_root + body sit on the default layer (index 2); escapee
        // forced to layer 0 (further back) → ranks before the block.
        let inputs = [
            input(char_root, 0.0, 0.0),
            input(body, 0.0, 0.0),
            input(escapee, 0.0, 0.0),
        ];
        let r = ranks(&w, &inputs);
        assert!(
            r[&escapee] < r[&char_root] && r[&escapee] < r[&body],
            "escapee uses its own layer 0, drawing behind the default-layer block"
        );
    }

    #[test]
    fn z_relative_cascade_saturates_without_overflow() {
        let mut w = World::new();
        let mut cache = EntityHashMap::default();
        let parent = w.spawn(ZIndexOverride(ZIndexOverride::Z_MAX)).id();
        let child = w
            .spawn((ChildOf(parent), ZIndexOverride(ZIndexOverride::Z_MAX)))
            .id();
        let z = effective_z(&w, child, &mut cache);
        assert_eq!(z, ZIndexOverride::Z_MAX as i64, "cascade clamps, no overflow");
    }

    #[test]
    fn z_absolute_ignores_parent() {
        let mut w = World::new();
        let mut cache = EntityHashMap::default();
        let parent = w.spawn(ZIndexOverride(100)).id();
        let child = w
            .spawn((ChildOf(parent), ZIndexOverride(5), ZAsRelative(false)))
            .id();
        assert_eq!(effective_z(&w, child, &mut cache), 5);
    }

    #[test]
    fn top_level_breaks_z_and_ysort_cascade() {
        let mut w = World::new();
        let mut cache = EntityHashMap::default();
        let parent = w.spawn((YSort::default(), ZIndexOverride(50))).id();
        let detached = w
            .spawn((ChildOf(parent), TopLevel, ZIndexOverride(3)))
            .id();
        // Z: TopLevel ignores parent's 50.
        assert_eq!(effective_z(&w, detached, &mut cache), 3);
        // YSort: TopLevel is not Y-sorted by the parent.
        let inputs = [input(detached, 0.0, 99.0)];
        let in_set: EntityHashMap<u32> = [(detached, 0u32)].into_iter().collect();
        assert_eq!(ysort_key(&w, detached, &inputs, &in_set), 0);
    }
}
