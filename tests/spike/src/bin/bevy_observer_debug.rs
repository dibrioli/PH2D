//! Investigação focada do bug observer bevy 0.18 detectado em C11.
//!
//! Test 1 (sabidamente OK): spawn solo + despawn → observer fires.
//! Test 2 (sabidamente FAIL): fixture cheia → observer não fires.
//!
//! Aqui isolamos cada variável: schedule, hierarchy, batch despawn.

use bevy_ecs::prelude::*;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

#[derive(Component, Debug)]
struct Position { x: f32, y: f32 }

#[derive(Component, Debug)]
struct Velocity { x: f32, y: f32 }

#[derive(Component, Debug)]
struct Health(i32);

fn move_sys(mut q: Query<(&mut Position, &Velocity)>) {
    for (mut p, v) in &mut q { p.x += v.x; p.y += v.y; }
}

fn damage_sys(mut q: Query<&mut Health>) {
    for mut h in &mut q { h.0 -= 1; }
}

fn run_test<F: FnOnce(&mut World)>(label: &str, body: F) -> usize {
    let mut world = World::new();
    let counter = Arc::new(AtomicUsize::new(0));
    let cc = counter.clone();
    world.add_observer(move |_: On<Remove, Health>| {
        cc.fetch_add(1, Ordering::SeqCst);
    });
    body(&mut world);
    let n = counter.load(Ordering::SeqCst);
    println!("{label:60}  observer fires: {n}");
    n
}

fn main() {
    println!("=== Bevy 0.18 observer behavior probe ===\n");

    run_test("T1: spawn(Health) + despawn — minimal", |w| {
        let e = w.spawn(Health(42)).id();
        w.despawn(e);
    });

    run_test("T2: spawn(P,V,H) + despawn — multi-component bundle", |w| {
        let e = w.spawn((Position{x:0.0,y:0.0}, Velocity{x:1.0,y:0.0}, Health(42))).id();
        w.despawn(e);
    });

    run_test("T3: spawn(H) × 5 + despawn each", |w| {
        let mut ids = vec![];
        for _ in 0..5 { ids.push(w.spawn(Health(1)).id()); }
        for e in ids { w.despawn(e); }
    });

    run_test("T4: spawn parent(H) + child(H, ChildOf) + despawn child only", |w| {
        let p = w.spawn(Health(100)).id();
        let c = w.spawn((Health(50), ChildOf(p))).id();
        w.despawn(c);
    });

    run_test("T5: same as T4 but despawn parent (cascades to child)", |w| {
        let p = w.spawn(Health(100)).id();
        let _c = w.spawn((Health(50), ChildOf(p))).id();
        w.despawn(p);
    });

    run_test("T6: spawn(H) + run schedule once + despawn", |w| {
        let e = w.spawn((Position{x:0.0,y:0.0}, Velocity{x:1.0,y:0.0}, Health(42))).id();
        let mut sched = Schedule::default();
        sched.add_systems((move_sys, damage_sys));
        sched.run(w);
        w.despawn(e);
    });

    run_test("T7: spawn(H) + 60 ticks + despawn", |w| {
        let e = w.spawn((Position{x:0.0,y:0.0}, Velocity{x:1.0,y:0.0}, Health(42))).id();
        let mut sched = Schedule::default();
        sched.add_systems((move_sys, damage_sys));
        for _ in 0..60 { sched.run(w); }
        w.despawn(e);
    });

    run_test("T8: parent + 199 children + 60 ticks + despawn 5 children", |w| {
        let p = w.spawn((Position{x:0.0,y:0.0}, Velocity{x:1.0,y:0.0}, Health(100))).id();
        for i in 0..199 {
            w.spawn((Position{x:i as f32,y:0.0}, Velocity{x:0.0,y:1.0}, Health(50+i), ChildOf(p)));
        }
        let mut sched = Schedule::default();
        sched.add_systems((move_sys, damage_sys));
        for _ in 0..60 { sched.run(w); }
        let mut q = w.query::<Entity>();
        let to_despawn: Vec<Entity> = q.iter(w).filter(|e| *e != p).take(5).collect();
        for e in to_despawn { w.despawn(e); }
    });

    run_test("T9: 200 entities (NO hierarchy) + 60 ticks + despawn 5", |w| {
        let mut ids = vec![];
        for i in 0..200 {
            ids.push(w.spawn((Position{x:i as f32,y:0.0}, Velocity{x:0.0,y:1.0}, Health(50+i))).id());
        }
        let mut sched = Schedule::default();
        sched.add_systems((move_sys, damage_sys));
        for _ in 0..60 { sched.run(w); }
        for e in ids.into_iter().take(5) { w.despawn(e); }
    });

    run_test("T10: parent + 199 children + NO schedule + despawn 5", |w| {
        let p = w.spawn((Position{x:0.0,y:0.0}, Velocity{x:1.0,y:0.0}, Health(100))).id();
        let mut child_ids = vec![];
        for i in 0..199 {
            child_ids.push(w.spawn((Position{x:i as f32,y:0.0}, Velocity{x:0.0,y:1.0}, Health(50+i), ChildOf(p))).id());
        }
        for e in child_ids.into_iter().take(5) { w.despawn(e); }
    });

    run_test("T11: parent + 199 children + 60 ticks + despawn 5 (BY ID, not query)", |w| {
        let p = w.spawn((Position{x:0.0,y:0.0}, Velocity{x:1.0,y:0.0}, Health(100))).id();
        let mut child_ids = vec![];
        for i in 0..199 {
            child_ids.push(w.spawn((Position{x:i as f32,y:0.0}, Velocity{x:0.0,y:1.0}, Health(50+i), ChildOf(p))).id());
        }
        let mut sched = Schedule::default();
        sched.add_systems((move_sys, damage_sys));
        for _ in 0..60 { sched.run(w); }
        for e in child_ids.into_iter().take(5) { w.despawn(e); }
    });

    println!("\n(world dropped: pode ou não disparar mais — depende de cleanup hooks)");
}
