//! C9 spike — replay determinístico (fixture local).
//!
//! Cenário: world fixo + input log de 600 ticks. Roda simulação N=100 vezes
//! no MESMO host (Mac M-series). Hash blake3 do world final deve ser idêntico
//! em todas as runs.
//!
//! Threshold C9 (docs/spike/2026-05-plan.md L68-72):
//! - hash idêntico Linux/macOS/Windows.
//!
//! **Limitação local:** este test verifica determinismo intra-host.
//! Cross-platform exige CI matrix (Linux + Mac runner + Windows runner) —
//! a ser configurado em `.github/workflows/spike.yml` na Semana 3.
//! Determinismo cross-platform requer (HR-5):
//! - f32 ops em ordem fixa (sem reordenamento SIMD count-dependent)
//! - sem fast-math/-ffast-math
//! - RNG seed explícita (Pcg64Mcg recomendado)
//! - GPU compute proibido em pipeline determinístico
//!
//! Esta fixture já segue HR-5: zero GPU, RNG seed fixo, ordem de iteração
//! determinística (via Vec<Entity> capturado no spawn).

use bevy_ecs::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Component, Serialize, Deserialize, Clone, Debug, PartialEq)]
struct Position { x: f32, y: f32 }

#[derive(Component, Serialize, Deserialize, Clone, Debug, PartialEq)]
struct Velocity { x: f32, y: f32 }

#[derive(Component, Serialize, Deserialize, Clone, Debug, PartialEq)]
struct Health(i32);

const N_ENTITIES: u64 = 200;
const TICKS: u64 = 600;
const REPS: usize = 100;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
struct WorldHash {
    entities: Vec<EntitySnap>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
struct EntitySnap {
    id: u64,
    position: Position,
    velocity: Velocity,
    health: i32,
}

/// Simple deterministic RNG (Lehmer / linear congruential) — no_std-friendly,
/// f32-safe (no SIMD reorder), seedable explicitly.
fn lcg(seed: u64, n: u64) -> u64 {
    seed.wrapping_mul(6_364_136_223_846_793_005)
        .wrapping_add(1442695040888963407)
        .wrapping_add(n)
}

/// Build a fresh world from fixed seeds. Entity ids stored in Vec for
/// deterministic iteration order (HR-5).
fn build_world() -> (World, Vec<Entity>) {
    let mut world = World::new();
    let mut ids = Vec::with_capacity(N_ENTITIES as usize);
    for i in 0..N_ENTITIES {
        let s = lcg(0xCAFE_F00D, i);
        let id = world.spawn((
            Position {
                x: ((s & 0xFFFF) as f32) / 65535.0,
                y: (((s >> 16) & 0xFFFF) as f32) / 65535.0,
            },
            Velocity {
                x: (((s >> 32) & 0xFFFF) as f32 - 32768.0) / 65535.0,
                y: (((s >> 48) & 0xFFFF) as f32 - 32768.0) / 65535.0,
            },
            Health((s % 200) as i32),
        )).id();
        ids.push(id);
    }
    (world, ids)
}

/// Fixed input log: at each tick, apply a deterministic perturbation to
/// some entities. No GPU, no system parallelism (single-threaded loops).
fn step(world: &mut World, ids: &[Entity], tick: u64) {
    for (i, &e) in ids.iter().enumerate() {
        // Move
        if let Some(mut pos) = world.get_mut::<Position>(e) {
            let v_seed = lcg(tick, i as u64);
            let dx = (((v_seed & 0xFF) as f32) - 128.0) / 128.0;
            let dy = ((((v_seed >> 8) & 0xFF) as f32) - 128.0) / 128.0;
            pos.x += dx * 0.001;
            pos.y += dy * 0.001;
        }
        // Damage proportional to tick
        if let Some(mut h) = world.get_mut::<Health>(e) {
            if (tick + i as u64) % 7 == 0 {
                h.0 -= 1;
            }
        }
    }
}

fn snapshot(world: &mut World, ids: &[Entity]) -> WorldHash {
    let mut entities = Vec::with_capacity(ids.len());
    for &e in ids {
        let p = world.get::<Position>(e).cloned().unwrap_or(Position { x: 0.0, y: 0.0 });
        let v = world.get::<Velocity>(e).cloned().unwrap_or(Velocity { x: 0.0, y: 0.0 });
        let h = world.get::<Health>(e).map(|x| x.0).unwrap_or(0);
        entities.push(EntitySnap {
            id: e.to_bits(),
            position: p,
            velocity: v,
            health: h,
        });
    }
    WorldHash { entities }
}

fn run_once() -> [u8; 32] {
    let (mut world, ids) = build_world();
    for tick in 0..TICKS {
        step(&mut world, &ids, tick);
    }
    let snap = snapshot(&mut world, &ids);
    let bytes = postcard::to_allocvec(&snap).unwrap();
    blake3::hash(&bytes).into()
}

fn hex(bytes: &[u8; 32]) -> String {
    let mut s = String::with_capacity(64);
    for b in bytes { s.push_str(&format!("{b:02x}")); }
    s
}

fn main() {
    println!("=== C9 replay determinismo intra-host ({REPS} runs × {TICKS} ticks × {N_ENTITIES} entities) ===\n");
    let baseline = run_once();
    println!("baseline hash: {}", hex(&baseline));
    let mut mismatches = 0;
    for run in 1..=REPS {
        let h = run_once();
        if h != baseline {
            println!("run {run:3}: MISMATCH {}", hex(&h));
            mismatches += 1;
        }
    }

    if mismatches == 0 {
        println!("\nC9 PASS (intra-host) — {REPS}/{REPS} runs idênticos");
        println!("\nNota: cross-platform Linux/Mac/Windows requer CI matrix");
        println!("(.github/workflows/spike.yml). HR-5 já cumprido (sem GPU, sem fast-math,");
        println!("RNG explícita, ordem de iteração determinística).");
    } else {
        println!("\nC9 FAIL (intra-host) — {mismatches}/{REPS} mismatches");
        std::process::exit(1);
    }
}
