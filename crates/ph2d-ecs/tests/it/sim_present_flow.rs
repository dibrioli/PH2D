//! Integration test for the M4 gate: 200 entities Position+Velocity in
//! SimWorld, movement system runs there, extract pushes derived
//! RenderInstance into PresentWorld, presentation reads PresentWorld
//! read-only.

use ph2d_ecs::{
    Component, PresentComponent, PresentWorld, Query, Schedule, SimComponent, SimWorld, extract,
};

#[derive(Component, Debug, Clone, Copy)]
struct Position {
    x: f32,
    y: f32,
}
impl SimComponent for Position {}

#[derive(Component, Debug, Clone, Copy)]
struct Velocity {
    x: f32,
    y: f32,
}
impl SimComponent for Velocity {}

/// Presentation-side derived: just position; render system would also
/// stash sprite handles, animation frame, etc.
#[derive(Component, Debug, Clone, Copy, PartialEq)]
struct RenderInstance {
    x: f32,
    y: f32,
}
impl PresentComponent for RenderInstance {}

fn move_sys(mut q: Query<(&mut Position, &Velocity)>) {
    for (mut p, v) in &mut q {
        p.x += v.x;
        p.y += v.y;
    }
}

#[test]
fn gate_m4_two_world_flow_with_200_entities() {
    let mut sim = SimWorld::new();
    let mut present = PresentWorld::new();

    // Spawn 200 entities Position+Velocity in SimWorld.
    let mut sim_ids = Vec::with_capacity(200);
    for i in 0..200 {
        let id = sim
            .world_mut()
            .spawn((
                Position {
                    x: i as f32,
                    y: 0.0,
                },
                Velocity { x: 1.0, y: 0.5 },
            ))
            .id();
        sim_ids.push(id);
    }
    assert_eq!(sim.entity_count(), 200);

    // Run movement system in SimWorld (one tick).
    let mut sim_schedule = Schedule::default();
    sim_schedule.add_systems(move_sys);
    sim_schedule.run(sim.world_mut());

    // Extract: SimWorld read-only → PresentWorld mutable.
    // Pre-build the QueryState while we still have &mut access to sim
    // (bevy_ecs::World::query requires &mut to populate cache). Inside
    // the extract! macro body, `sim_w` is &World — only iter() is
    // available, which preserves the read-only contract.
    let mut sim_query = sim
        .world_mut()
        .query::<(bevy_ecs::entity::Entity, &Position)>();
    extract!(sim => present, |sim_w, present_w| {
        for (_e, pos) in sim_query.iter(sim_w) {
            present_w.spawn(RenderInstance { x: pos.x, y: pos.y });
        }
    });

    // PresentWorld now has 200 RenderInstance entities derived from sim.
    assert_eq!(present.entity_count(), 200);

    // Read-only check from PresentWorld. Note: bevy_ecs::World::query
    // requires &mut to populate the QueryState cache; the actual
    // iteration takes &World immutably. M5+ may add a `query_cached`
    // helper that hides the &mut quirk.
    let mut q = present.world_mut().query::<&RenderInstance>();
    let collected: Vec<RenderInstance> = q.iter(present.world()).copied().collect();
    assert_eq!(collected.len(), 200);
    // After 1 tick of move_sys: x = i + 1.0, y = 0.5.
    // Iteration order is bevy_ecs internal — sort to assert.
    let mut sorted = collected.clone();
    sorted.sort_by(|a, b| a.x.partial_cmp(&b.x).unwrap());
    for (i, r) in sorted.iter().enumerate() {
        assert!((r.x - (i as f32 + 1.0)).abs() < 1e-4);
        assert!((r.y - 0.5).abs() < 1e-4);
    }
}

#[test]
fn extract_macro_can_use_existing_present_entities() {
    let mut sim = SimWorld::new();
    let mut present = PresentWorld::new();

    let e = sim
        .world_mut()
        .spawn((Position { x: 10.0, y: 20.0 }, Velocity { x: 0.0, y: 0.0 }))
        .id();

    // First extract: create.
    extract!(sim => present, |sim_w, present_w| {
        let pos = sim_w.get::<Position>(e).unwrap();
        present_w.spawn(RenderInstance { x: pos.x, y: pos.y });
    });
    assert_eq!(present.entity_count(), 1);

    // Second extract: extract again, this time we expect to see
    // present_w able to mutate / re-spawn. (Real extract systems would
    // diff or reuse entities; we test that present is fully writable.)
    extract!(sim => present, |sim_w, present_w| {
        let pos = sim_w.get::<Position>(e).unwrap();
        present_w.spawn(RenderInstance {
            x: pos.x * 2.0,
            y: pos.y * 2.0,
        });
    });
    assert_eq!(present.entity_count(), 2);
}

#[test]
fn sim_world_independent_of_present_world() {
    let mut sim = SimWorld::new();
    let mut present = PresentWorld::new();

    sim.world_mut()
        .spawn((Position { x: 0.0, y: 0.0 }, Velocity { x: 1.0, y: 1.0 }));

    // PresentWorld is empty even though SimWorld has entities.
    assert_eq!(sim.entity_count(), 1);
    assert_eq!(present.entity_count(), 0);
}
