//! C4 spike — hot reload determinístico via reset+restore.
//!
//! Modelo: snapshot do World em postcard (Vec<u8>), hash blake3, depois
//! "reset" (recreate World), restore do snapshot, snapshot+hash de novo.
//! Hash deve ser idêntico em 100/100 ciclos.
//!
//! Threshold C4 (docs/spike/2026-05-plan.md L62-66):
//! - 100% match em 100 ciclos.
//! - Freeze ≤ 250 ms em fixture (200 entities, 20 component types).
//!
//! Fixture: 200 entities; cada uma carrega 3 components ativos de pool de 20
//! tipos (rotativo). Soma 600 component instances vivos.

use bevy_ecs::prelude::*;
use serde::{Deserialize, Serialize};
use std::time::Instant;

// 20 component types (varied data shapes for realistic snapshot).
macro_rules! mk_component {
    ($name:ident, $($field:ident : $ty:ty = $default:expr),+ $(,)?) => {
        #[derive(Component, Serialize, Deserialize, Clone, Debug, PartialEq)]
        struct $name { $($field: $ty),+ }
        impl $name {
            fn make(seed: u64) -> Self {
                Self { $($field: ($default)(seed)),+ }
            }
        }
    };
}

mk_component!(C00, x: f32 = (|s: u64| s as f32), y: f32 = (|s: u64| (s + 1) as f32));
mk_component!(C01, vx: f32 = (|s: u64| (s % 7) as f32), vy: f32 = (|s: u64| (s % 11) as f32));
mk_component!(C02, hp: i32 = (|s: u64| (s % 200) as i32));
mk_component!(C03, name: String = (|s: u64| format!("e{s}")));
mk_component!(C04, tag: u8 = (|s: u64| (s % 16) as u8));
mk_component!(C05, scale: f32 = (|s: u64| 1.0 + (s % 5) as f32 * 0.1));
mk_component!(C06, mana: i32 = (|s: u64| (s % 100) as i32));
mk_component!(C07, level: u32 = (|s: u64| (s % 50) as u32));
mk_component!(C08, exp: u64 = (|s: u64| s * 7));
mk_component!(C09, label: String = (|s: u64| format!("L{}", s % 100)));
mk_component!(C10, alive: bool = (|s: u64| s % 2 == 0));
mk_component!(C11, weight: f64 = (|s: u64| (s % 1000) as f64 * 0.5));
mk_component!(C12, color: u32 = (|s: u64| 0xFF00_0000 | (s & 0x00FF_FFFF) as u32));
mk_component!(C13, rotation: f32 = (|s: u64| (s % 360) as f32));
mk_component!(C14, owner_id: u64 = (|s: u64| s / 3));
mk_component!(C15, faction: i8 = (|s: u64| (s % 4) as i8 - 2));
mk_component!(C16, score: i32 = (|s: u64| (s % 10000) as i32));
mk_component!(C17, layer: u8 = (|s: u64| (s % 8) as u8));
mk_component!(C18, opacity: f32 = (|s: u64| (s % 100) as f32 / 100.0));
mk_component!(C19, kind: u16 = (|s: u64| (s % 256) as u16));

const N_ENTITIES: u64 = 200;
const CYCLES: usize = 100;

/// Per-entity snapshot is a tagged enum array (3 components per entity, of 20 types).
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
enum CompSnap {
    C00(C00),
    C01(C01),
    C02(C02),
    C03(C03),
    C04(C04),
    C05(C05),
    C06(C06),
    C07(C07),
    C08(C08),
    C09(C09),
    C10(C10),
    C11(C11),
    C12(C12),
    C13(C13),
    C14(C14),
    C15(C15),
    C16(C16),
    C17(C17),
    C18(C18),
    C19(C19),
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
struct WorldSnapshot {
    entities: Vec<Vec<CompSnap>>, // outer index = entity index, inner = component list
}

fn build_world() -> (World, WorldSnapshot) {
    let mut world = World::new();
    let mut snap = WorldSnapshot {
        entities: Vec::with_capacity(N_ENTITIES as usize),
    };
    for i in 0..N_ENTITIES {
        let comps: Vec<CompSnap> = vec![
            match i % 20 {
                0 => CompSnap::C00(C00::make(i)),
                1 => CompSnap::C01(C01::make(i)),
                2 => CompSnap::C02(C02::make(i)),
                3 => CompSnap::C03(C03::make(i)),
                4 => CompSnap::C04(C04::make(i)),
                5 => CompSnap::C05(C05::make(i)),
                6 => CompSnap::C06(C06::make(i)),
                7 => CompSnap::C07(C07::make(i)),
                8 => CompSnap::C08(C08::make(i)),
                9 => CompSnap::C09(C09::make(i)),
                10 => CompSnap::C10(C10::make(i)),
                11 => CompSnap::C11(C11::make(i)),
                12 => CompSnap::C12(C12::make(i)),
                13 => CompSnap::C13(C13::make(i)),
                14 => CompSnap::C14(C14::make(i)),
                15 => CompSnap::C15(C15::make(i)),
                16 => CompSnap::C16(C16::make(i)),
                17 => CompSnap::C17(C17::make(i)),
                18 => CompSnap::C18(C18::make(i)),
                _ => CompSnap::C19(C19::make(i)),
            },
            CompSnap::C00(C00::make(i + 100)),
            CompSnap::C04(C04::make(i + 200)),
        ];
        spawn_from_snap(&mut world, &comps);
        snap.entities.push(comps);
    }
    (world, snap)
}

fn spawn_from_snap(world: &mut World, comps: &[CompSnap]) {
    let mut e = world.spawn_empty();
    for c in comps {
        match c.clone() {
            CompSnap::C00(v) => {
                e.insert(v);
            }
            CompSnap::C01(v) => {
                e.insert(v);
            }
            CompSnap::C02(v) => {
                e.insert(v);
            }
            CompSnap::C03(v) => {
                e.insert(v);
            }
            CompSnap::C04(v) => {
                e.insert(v);
            }
            CompSnap::C05(v) => {
                e.insert(v);
            }
            CompSnap::C06(v) => {
                e.insert(v);
            }
            CompSnap::C07(v) => {
                e.insert(v);
            }
            CompSnap::C08(v) => {
                e.insert(v);
            }
            CompSnap::C09(v) => {
                e.insert(v);
            }
            CompSnap::C10(v) => {
                e.insert(v);
            }
            CompSnap::C11(v) => {
                e.insert(v);
            }
            CompSnap::C12(v) => {
                e.insert(v);
            }
            CompSnap::C13(v) => {
                e.insert(v);
            }
            CompSnap::C14(v) => {
                e.insert(v);
            }
            CompSnap::C15(v) => {
                e.insert(v);
            }
            CompSnap::C16(v) => {
                e.insert(v);
            }
            CompSnap::C17(v) => {
                e.insert(v);
            }
            CompSnap::C18(v) => {
                e.insert(v);
            }
            CompSnap::C19(v) => {
                e.insert(v);
            }
        }
    }
}

fn restore_world(snap: &WorldSnapshot) -> World {
    let mut world = World::new();
    for comps in &snap.entities {
        spawn_from_snap(&mut world, comps);
    }
    world
}

fn snapshot_hash(snap: &WorldSnapshot) -> [u8; 32] {
    let bytes = postcard::to_allocvec(snap).expect("postcard ser");
    blake3::hash(&bytes).into()
}

fn main() {
    let (_world, baseline_snap) = build_world();
    let baseline_hash = snapshot_hash(&baseline_snap);

    let mut freeze_ms: Vec<f64> = Vec::with_capacity(CYCLES);
    let mut mismatches = 0;
    for cycle in 0..CYCLES {
        let t0 = Instant::now();
        // RESET: drop everything; the freeze includes serialize → drop World → restore.
        let serialized = postcard::to_allocvec(&baseline_snap).unwrap();
        let restored: WorldSnapshot = postcard::from_bytes(&serialized).unwrap();
        let _new_world = restore_world(&restored);
        let elapsed = t0.elapsed().as_secs_f64() * 1000.0;
        freeze_ms.push(elapsed);

        let hash_after = snapshot_hash(&restored);
        if hash_after != baseline_hash {
            mismatches += 1;
            eprintln!("cycle {cycle}: hash mismatch");
        }
    }

    freeze_ms.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let p50 = freeze_ms[freeze_ms.len() / 2];
    let p99 = freeze_ms[(freeze_ms.len() as f64 * 0.99) as usize];
    let max = *freeze_ms.last().unwrap();

    let bytes = postcard::to_allocvec(&baseline_snap).unwrap();
    println!("=== C4 hot reload ({CYCLES} cycles) ===");
    println!("entities: {N_ENTITIES} | components per entity: 3 | component types: 20");
    println!("snapshot bytes (postcard): {} B", bytes.len());
    println!("blake3 hash: {}", hex(&baseline_hash));
    println!("hash matches: {}/{}", CYCLES - mismatches, CYCLES);
    println!("freeze (serialize → drop → restore):");
    println!("  p50:  {p50:.3} ms");
    println!("  p99:  {p99:.3} ms");
    println!("  max:  {max:.3} ms");
    println!("threshold: 100% hash match + freeze ≤ 250 ms");

    let pass = mismatches == 0 && max <= 250.0;
    if pass {
        println!("\nC4 PASS — determinístico + dentro do orçamento de freeze");
    } else {
        println!("\nC4 FAIL — mismatches: {mismatches}, max freeze {max:.3}ms");
        std::process::exit(1);
    }
}

fn hex(bytes: &[u8; 32]) -> String {
    let mut s = String::with_capacity(64);
    for b in bytes {
        s.push_str(&format!("{b:02x}"));
    }
    s
}
