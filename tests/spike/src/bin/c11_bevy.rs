//! C11 spike — bevy_ecs 0.18 fixture.
//!
//! Cena mínima conforme docs/spike/2026-05-plan.md L130-138:
//! 200 entities (1 parent + 199 children), 1 hierarquia via ChildOf,
//! 3 components, 2 systems, 1 observer/hook em delete.
//!
//! Métricas C11 coletadas externamente:
//! - LOC: `wc -l src/bin/c11_bevy.rs`
//! - Compile time: `time cargo build --bin c11_bevy --features bevy_ecs --release`
//! - Binary size: `cargo bloat --bin c11_bevy --features bevy_ecs --release`
//! - Unsafe count: `grep -c "unsafe " src/bin/c11_bevy.rs`
//! - Stack trace quality: panic forcado + RUST_BACKTRACE=1
//!
//! Observação importante (LLM-friction medida durante spike):
//! - bevy_ecs 0.18 renomeou `Trigger<OnRemove, T>` → `On<Remove, T>`,
//!   `iter_entities()` → `query::<Entity>().iter(&world)`. Sem
//!   release notes claras na crates.io page; descoberta via grep
//!   no source.
//! - Observer "issue" inicial (0/200 fires) RESOLVIDO em S2: era bug do
//!   fixture, não do bevy. `world.query::<Entity>()` retorna entities
//!   INTERNAS (criadas por bevy ao usar ChildOf — ex: storage do Children
//!   component) que nunca tiveram Health. Despawn dessas não dispara
//!   On<Remove, Health> — comportamento correto.
//!   Fix: armazenar child IDs explicitamente no spawn loop e despawnar
//!   esses (não via query).

use bevy_ecs::prelude::*;

#[derive(Component, Debug)]
struct Position {
    x: f32,
    y: f32,
}

#[derive(Component, Debug)]
struct Velocity {
    x: f32,
    y: f32,
}

#[derive(Component, Debug)]
struct Health(i32);

fn move_sys(mut q: Query<(&mut Position, &Velocity)>) {
    for (mut p, v) in &mut q {
        p.x += v.x;
        p.y += v.y;
    }
}

fn damage_sys(mut q: Query<&mut Health>) {
    for mut h in &mut q {
        h.0 -= 1;
    }
}

fn on_remove_health(trigger: On<Remove, Health>) {
    println!(
        "observer fired: entity {:?} lost Health",
        trigger.event().entity
    );
}

fn main() {
    let mut world = World::new();
    world.add_observer(on_remove_health);

    let parent = world
        .spawn((
            Position { x: 0.0, y: 0.0 },
            Velocity { x: 1.0, y: 0.0 },
            Health(100),
        ))
        .id();

    let mut child_ids: Vec<Entity> = Vec::with_capacity(199);
    for i in 0..199 {
        let id = world
            .spawn((
                Position {
                    x: i as f32,
                    y: 0.0,
                },
                Velocity { x: 0.0, y: 1.0 },
                Health(50 + i),
                ChildOf(parent),
            ))
            .id();
        child_ids.push(id);
    }

    let mut schedule = Schedule::default();
    schedule.add_systems((move_sys, damage_sys));
    for _ in 0..60 {
        schedule.run(&mut world);
    }

    for e in child_ids.into_iter().take(5) {
        world.despawn(e);
    }

    let mut q = world.query::<&Health>();
    let alive_with_health = q.iter(&world).count();
    println!("C11 BEVY PASS — entities with Health alive: {alive_with_health}");
}
