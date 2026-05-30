//! Inspector §7 (Ordering / Sorting) commit handler — Sprite Inspector
//! v2 W3.
//!
//! Each [`OrderingFieldEdit`] maps to an *optional* ECS sorting
//! component (presence = override, spec §02). Editing therefore queues
//! a `SetComponent` (insert/update) or `RemoveComponent` (detach) rather
//! than mutating an always-present struct like `apply_sprite_field`
//! does. Read-modify-write fields (`YSort`, `SortingGroup`) read the
//! entity's current component-or-default so editing one sub-field
//! preserves the siblings. The caller applies the queued commands via
//! `apply_editor_commands`.

use ph2d_core::Vec2;
use ph2d_ecs::scene::{ComponentRegistry, EditorCommand, EditorCommandQueue};
use ph2d_ecs::sorting::SortingLayers;
use ph2d_ecs::{
    Entity, LayerId, OrderInLayer, ShowBehindParent, SimWorld, SortPoint, SortingGroup,
    SortingLayer, TopLevel, World, YSort, ZAsRelative, ZIndexOverride,
};
use ph2d_editor::{InspectorOrderingInfo, InspectorOrderingMixed, OrderingFieldEdit};
use serde::Serialize;

/// The §7 ordering fields of one entity, read as display values (absent
/// component → its pipeline default). Shared by the snapshot producer
/// and the BulkSelect compare so the two never drift.
type OrderingFields = (
    Option<i32>, // z_index (None = no override / DFS)
    bool,        // z_as_relative
    bool,        // show_behind_parent
    u8,          // sorting_layer (default-layer index when absent)
    i32,         // order_in_layer
    bool,        // y_sort_enabled
    u8,          // y_sort_point tag
    [f32; 2],    // y_sort_axis
    bool,        // sorting_group
    bool,        // sort_at_root
    bool,        // top_level
);

fn ordering_fields(world: &World, entity: Entity, default_layer: u8) -> OrderingFields {
    let ys = world.get::<YSort>(entity).copied();
    let group = world.get::<SortingGroup>(entity).copied();
    (
        world.get::<ZIndexOverride>(entity).map(|z| z.0),
        world.get::<ZAsRelative>(entity).is_none_or(|r| r.0),
        world.get::<ShowBehindParent>(entity).is_some(),
        world
            .get::<SortingLayer>(entity)
            .map_or(default_layer, |l| l.0.0),
        world.get::<OrderInLayer>(entity).map_or(0, |o| o.0),
        ys.is_some_and(|y| y.enabled),
        ys.map_or(0, |y| y.sort_point.tag()),
        ys.map_or([0.0, 1.0], |y| [y.axis.x, y.axis.y]),
        group.is_some(),
        group.is_some_and(|g| g.sort_at_root),
        world.get::<TopLevel>(entity).is_some(),
    )
}

/// BulkSelect (T2.0) for §7: which ordering fields diverge across the
/// `selected` entities vs the `primary` tuple.
#[allow(clippy::float_cmp)] // exact compare: same stored value = not mixed
fn ordering_mixed(
    world: &World,
    selected: &[u64],
    primary: &OrderingFields,
    default_layer: u8,
) -> InspectorOrderingMixed {
    let mut m = InspectorOrderingMixed::default();
    for &bits in selected {
        let f = ordering_fields(world, Entity::from_bits(bits), default_layer);
        m.z_index |= f.0 != primary.0;
        m.z_as_relative |= f.1 != primary.1;
        m.show_behind_parent |= f.2 != primary.2;
        m.sorting_layer |= f.3 != primary.3;
        m.order_in_layer |= f.4 != primary.4;
        m.y_sort_enabled |= f.5 != primary.5;
        m.y_sort_point |= f.6 != primary.6;
        m.y_sort_axis |= f.7 != primary.7;
        m.sorting_group |= f.8 != primary.8;
        m.sort_at_root |= f.9 != primary.9;
        m.top_level |= f.10 != primary.10;
    }
    m
}

/// Build the §7 ordering snapshot for `entity_bits`, or `None` when the
/// entity has no `Transform` (not an Inspector-worthy entity).
pub(super) fn build_ordering_info(
    world: &World,
    entity_bits: u64,
    selected: &[u64],
    selected_count: usize,
) -> Option<InspectorOrderingInfo> {
    let entity = Entity::from_bits(entity_bits);
    world.get::<ph2d_ecs::Transform>(entity)?;
    let default_layer = world
        .get_resource::<SortingLayers>()
        .map_or(2, |l| l.default_index());
    let f = ordering_fields(world, entity, default_layer);
    let mixed = if selected.len() > 1 {
        ordering_mixed(world, selected, &f, default_layer)
    } else {
        InspectorOrderingMixed::default()
    };
    Some(InspectorOrderingInfo {
        entity_bits,
        z_index: f.0,
        z_as_relative: f.1,
        show_behind_parent: f.2,
        sorting_layer: f.3,
        order_in_layer: f.4,
        y_sort_enabled: f.5,
        y_sort_point: f.6,
        y_sort_axis: f.7,
        sorting_group: f.8,
        sort_at_root: f.9,
        top_level: f.10,
        selected_count,
        mixed,
    })
}

/// Queue a `SetComponent` (insert/update) for the registered component
/// `canonical_name` on `entity_bits`. A no-op if the name isn't
/// registered or the value won't encode (neither happens for the W3
/// components — registration + serde are gate-asserted).
fn queue_set<T: Serialize>(
    queue: &EditorCommandQueue,
    registry: &ComponentRegistry,
    entity_bits: u64,
    canonical_name: &str,
    value: &T,
) {
    if let Some(entry) = registry.get_by_name(canonical_name)
        && let Ok(data) = postcard::to_allocvec(value)
    {
        let _ = queue.push(EditorCommand::SetComponent {
            entity: entity_bits,
            type_id: entry.type_id,
            data,
        });
    }
}

/// Queue a `RemoveComponent` (detach) for the registered component
/// `canonical_name` on `entity_bits`. Idempotent at apply time.
fn queue_remove(
    queue: &EditorCommandQueue,
    registry: &ComponentRegistry,
    entity_bits: u64,
    canonical_name: &str,
) {
    if let Some(entry) = registry.get_by_name(canonical_name) {
        let _ = queue.push(EditorCommand::RemoveComponent {
            entity: entity_bits,
            type_id: entry.type_id,
        });
    }
}

/// Apply one [`OrderingFieldEdit`] (§7) by queueing the right
/// `SetComponent` / `RemoveComponent` against the optional sorting
/// component it maps to. The commands are applied by the caller's
/// `apply_editor_commands` pass.
pub(super) fn apply_ordering_edit(
    sim: &SimWorld,
    entity_bits: u64,
    edit: OrderingFieldEdit,
    queue: &EditorCommandQueue,
    registry: &ComponentRegistry,
) {
    let entity = Entity::from_bits(entity_bits);
    let cur_ysort = || sim.world().get::<YSort>(entity).copied().unwrap_or_default();
    let cur_group = || {
        sim.world()
            .get::<SortingGroup>(entity)
            .copied()
            .unwrap_or_default()
    };
    match edit {
        OrderingFieldEdit::ZIndex(Some(v)) => queue_set(
            queue,
            registry,
            entity_bits,
            "ph2d::ecs::ZIndexOverride",
            &ZIndexOverride(ZIndexOverride::clamped(v)),
        ),
        OrderingFieldEdit::ZIndex(None) => {
            queue_remove(queue, registry, entity_bits, "ph2d::ecs::ZIndexOverride")
        }
        OrderingFieldEdit::ZAsRelative(b) => {
            queue_set(queue, registry, entity_bits, "ph2d::ecs::ZAsRelative", &ZAsRelative(b))
        }
        OrderingFieldEdit::ShowBehindParent(true) => queue_set(
            queue,
            registry,
            entity_bits,
            "ph2d::ecs::ShowBehindParent",
            &ShowBehindParent,
        ),
        OrderingFieldEdit::ShowBehindParent(false) => {
            queue_remove(queue, registry, entity_bits, "ph2d::ecs::ShowBehindParent")
        }
        OrderingFieldEdit::SortingLayer(idx) => queue_set(
            queue,
            registry,
            entity_bits,
            "ph2d::ecs::SortingLayer",
            &SortingLayer(LayerId(idx)),
        ),
        OrderingFieldEdit::OrderInLayer(v) => {
            queue_set(queue, registry, entity_bits, "ph2d::ecs::OrderInLayer", &OrderInLayer(v))
        }
        OrderingFieldEdit::YSortEnabled(b) => {
            let ys = YSort { enabled: b, ..cur_ysort() };
            queue_set(queue, registry, entity_bits, "ph2d::ecs::YSort", &ys);
        }
        OrderingFieldEdit::YSortPoint(tag) => {
            let ys = YSort {
                sort_point: SortPoint::from_tag(tag),
                ..cur_ysort()
            };
            queue_set(queue, registry, entity_bits, "ph2d::ecs::YSort", &ys);
        }
        OrderingFieldEdit::YSortAxis(a) => {
            let ys = YSort {
                axis: Vec2::new(a[0], a[1]),
                ..cur_ysort()
            };
            queue_set(queue, registry, entity_bits, "ph2d::ecs::YSort", &ys);
        }
        // Preserve `sort_at_root` if the component was already present.
        OrderingFieldEdit::SortingGroup(true) => {
            queue_set(queue, registry, entity_bits, "ph2d::ecs::SortingGroup", &cur_group())
        }
        OrderingFieldEdit::SortingGroup(false) => {
            queue_remove(queue, registry, entity_bits, "ph2d::ecs::SortingGroup")
        }
        OrderingFieldEdit::SortAtRoot(b) => {
            let g = SortingGroup { sort_at_root: b };
            queue_set(queue, registry, entity_bits, "ph2d::ecs::SortingGroup", &g);
        }
        OrderingFieldEdit::TopLevel(true) => {
            queue_set(queue, registry, entity_bits, "ph2d::ecs::TopLevel", &TopLevel)
        }
        OrderingFieldEdit::TopLevel(false) => {
            queue_remove(queue, registry, entity_bits, "ph2d::ecs::TopLevel")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ph2d_ecs::scene::{apply_editor_commands, register_ecs_components};

    /// Build a sim + registry + queue, run one edit, apply it, and hand
    /// the world back for assertions.
    fn world_after(edits: &[OrderingFieldEdit]) -> (SimWorld, Entity) {
        let mut sim = SimWorld::new();
        let entity = sim.world_mut().spawn_empty().id();
        let mut reg = ComponentRegistry::new();
        register_ecs_components(&mut reg);
        let queue = EditorCommandQueue::new();
        for &edit in edits {
            apply_ordering_edit(&sim, entity.to_bits(), edit, &queue, &reg);
            apply_editor_commands(sim.world_mut(), &queue, &reg).unwrap();
        }
        (sim, entity)
    }

    #[test]
    fn z_index_attach_clamps_then_detach() {
        let (sim, e) = world_after(&[OrderingFieldEdit::ZIndex(Some(i32::MAX))]);
        assert_eq!(
            sim.world().get::<ZIndexOverride>(e).unwrap().0,
            ZIndexOverride::Z_MAX,
            "attach clamps to the gateable range"
        );
        let (sim, e) = world_after(&[
            OrderingFieldEdit::ZIndex(Some(7)),
            OrderingFieldEdit::ZIndex(None),
        ]);
        assert!(
            sim.world().get::<ZIndexOverride>(e).is_none(),
            "ZIndex(None) detaches the component"
        );
    }

    #[test]
    fn markers_toggle_insert_and_remove() {
        let (sim, e) = world_after(&[OrderingFieldEdit::ShowBehindParent(true)]);
        assert!(sim.world().get::<ShowBehindParent>(e).is_some());
        let (sim, e) = world_after(&[
            OrderingFieldEdit::TopLevel(true),
            OrderingFieldEdit::TopLevel(false),
        ]);
        assert!(sim.world().get::<TopLevel>(e).is_none());
    }

    #[test]
    fn ysort_subfield_edits_preserve_siblings() {
        // Enable, then set a custom axis, then switch sort point — each
        // read-modify-writes the YSort without clobbering the others.
        let (sim, e) = world_after(&[
            OrderingFieldEdit::YSortEnabled(true),
            OrderingFieldEdit::YSortAxis([1.0, 1.0]),
            OrderingFieldEdit::YSortPoint(SortPoint::Custom.tag()),
        ]);
        let ys = sim.world().get::<YSort>(e).unwrap();
        assert!(ys.enabled);
        assert_eq!(ys.axis, Vec2::new(1.0, 1.0));
        assert_eq!(ys.sort_point, SortPoint::Custom);
    }

    #[test]
    fn sorting_group_toggle_preserves_sort_at_root() {
        // Turn the group on, set sort_at_root, toggle the group off then
        // on — `SortingGroup(true)` preserves the prior sort_at_root only
        // while the component lives; a full off→on resets to default.
        let (sim, e) = world_after(&[
            OrderingFieldEdit::SortingGroup(true),
            OrderingFieldEdit::SortAtRoot(true),
        ]);
        assert!(sim.world().get::<SortingGroup>(e).unwrap().sort_at_root);

        let (sim, e) = world_after(&[
            OrderingFieldEdit::SortingGroup(true),
            OrderingFieldEdit::SortingGroup(false),
        ]);
        assert!(
            sim.world().get::<SortingGroup>(e).is_none(),
            "toggling the group off detaches it"
        );
    }

    #[test]
    fn sorting_layer_and_order_attach() {
        let (sim, e) = world_after(&[
            OrderingFieldEdit::SortingLayer(4),
            OrderingFieldEdit::OrderInLayer(-3),
        ]);
        assert_eq!(sim.world().get::<SortingLayer>(e).unwrap().0, LayerId(4));
        assert_eq!(sim.world().get::<OrderInLayer>(e).unwrap().0, -3);
    }
}
