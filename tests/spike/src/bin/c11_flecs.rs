//! C11 spike — flecs_ecs 0.2.2 fixture.
//!
//! Cena mínima conforme docs/spike/2026-05-plan.md L130-138:
//! 200 entities (1 parent + 199 children via flecs::ChildOf),
//! 3 components, 2 systems, 1 observer/hook em delete.
//!
//! Métricas C11 coletadas externamente:
//! - LOC: `wc -l src/bin/c11_flecs.rs`
//! - Compile time: `time cargo build --bin c11_flecs --features flecs_ecs --release`
//! - Binary size: `cargo bloat --bin c11_flecs --features flecs_ecs --release`
//! - Unsafe count: `grep -c "unsafe " src/bin/c11_flecs.rs`
//! - Stack trace quality: panic forcado + RUST_BACKTRACE=1

use flecs_ecs::prelude::*;

#[derive(Debug, Component)]
struct Position { x: f32, y: f32 }

#[derive(Debug, Component)]
struct Velocity { x: f32, y: f32 }

#[derive(Debug, Component)]
struct Health(i32);

fn main() {
    let world = World::new();

    world
        .observer::<flecs::OnRemove, &Health>()
        .each_iter(|it, index, _hp| {
            println!("observer fired: entity {} lost Health", it.entity(index));
        });

    let move_sys = world
        .system::<(&mut Position, &Velocity)>()
        .each(|(p, v)| {
            p.x += v.x;
            p.y += v.y;
        });

    let damage_sys = world
        .system::<&mut Health>()
        .each(|h| {
            h.0 -= 1;
        });

    let parent = world
        .entity_named("parent")
        .set(Position { x: 0.0, y: 0.0 })
        .set(Velocity { x: 1.0, y: 0.0 })
        .set(Health(100));

    let mut children: Vec<EntityView> = Vec::with_capacity(199);
    for i in 0..199 {
        let e = world
            .entity()
            .set(Position { x: i as f32, y: 0.0 })
            .set(Velocity { x: 0.0, y: 1.0 })
            .set(Health(50 + i as i32))
            .child_of(parent);
        children.push(e);
    }

    for _ in 0..60 {
        move_sys.run();
        damage_sys.run();
    }

    for child in children.iter().take(5) {
        child.destruct();
    }

    let mut alive = 0usize;
    world.query::<&Health>().build().each(|_| {
        alive += 1;
    });
    println!("C11 FLECS PASS — alive entities with Health: {alive}");
}
