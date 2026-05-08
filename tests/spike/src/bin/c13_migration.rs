//! C13 spike — save migration end-to-end (HR-14).
//!
//! Cenário: build N tem `Health { value, max }` + `StateTableV1`.
//! Build N+1: `Health` ganha `regen_rate`, `StateTableV1` ganha
//! `last_hit_tick`. Função `migrate_v1_to_v2` faz a transição pura.
//!
//! Threshold C13 (docs/spike/2026-05-plan.md L146-150):
//! - 5/5 fixtures de saves v1 abrem corretamente em v2.
//! - Migração é função pura (sem I/O, sem panic em input válido).
//! - HR-14: todo struct com save tem field `version: u32` no início.

use serde::{Deserialize, Serialize};

// === Build N (v1) ===

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
struct SaveV1 {
    version: u32,
    health: HealthV1,
    state_table: StateTableV1,
    entities: Vec<EntityRecordV1>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
struct HealthV1 {
    value: i32,
    max: i32,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
struct StateTableV1 {
    fsm_state: String,
    flags: Vec<(String, bool)>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
struct EntityRecordV1 {
    id: u64,
    name: String,
    health: HealthV1,
}

// === Build N+1 (v2) ===

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
struct SaveV2 {
    version: u32,
    health: HealthV2,
    state_table: StateTableV2,
    entities: Vec<EntityRecordV2>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
struct HealthV2 {
    value: i32,
    max: i32,
    regen_rate: f32, // NEW in v2
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
struct StateTableV2 {
    fsm_state: String,
    flags: Vec<(String, bool)>,
    last_hit_tick: u64, // NEW in v2
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
struct EntityRecordV2 {
    id: u64,
    name: String,
    health: HealthV2,
}

// === Migration: pure function v1 → v2 ===

#[derive(Debug)]
#[allow(dead_code)] // fields used via Debug derive in error reports.
enum MigrationError {
    UnsupportedVersion(u32),
    DecodeError(postcard::Error),
}

fn migrate_v1_to_v2(v1: SaveV1) -> SaveV2 {
    SaveV2 {
        version: 2,
        health: HealthV2 {
            value: v1.health.value,
            max: v1.health.max,
            regen_rate: 0.0, // safe default for new field
        },
        state_table: StateTableV2 {
            fsm_state: v1.state_table.fsm_state,
            flags: v1.state_table.flags,
            last_hit_tick: 0, // safe default
        },
        entities: v1
            .entities
            .into_iter()
            .map(|e| EntityRecordV2 {
                id: e.id,
                name: e.name,
                health: HealthV2 {
                    value: e.health.value,
                    max: e.health.max,
                    regen_rate: 0.0,
                },
            })
            .collect(),
    }
}

fn load(bytes: &[u8]) -> Result<SaveV2, MigrationError> {
    // Peek version field (always first u32 by HR-14 convention).
    let v1: Result<SaveV1, _> = postcard::from_bytes(bytes);
    if let Ok(s) = v1 {
        if s.version == 1 {
            return Ok(migrate_v1_to_v2(s));
        }
    }
    let v2: SaveV2 = postcard::from_bytes(bytes).map_err(MigrationError::DecodeError)?;
    if v2.version == 2 {
        Ok(v2)
    } else {
        Err(MigrationError::UnsupportedVersion(v2.version))
    }
}

fn make_v1_fixture(seed: u64) -> SaveV1 {
    SaveV1 {
        version: 1,
        health: HealthV1 {
            value: (seed * 11 % 200) as i32,
            max: 200,
        },
        state_table: StateTableV1 {
            fsm_state: format!("state_{}", seed % 4),
            flags: vec![
                (format!("met_npc_{}", seed % 3), true),
                ("tutorial_done".into(), seed > 1),
            ],
        },
        entities: (0..3)
            .map(|i| EntityRecordV1 {
                id: seed * 100 + i,
                name: format!("entity_{seed}_{i}"),
                health: HealthV1 {
                    value: 50 + i as i32,
                    max: 100,
                },
            })
            .collect(),
    }
}

fn main() {
    println!("=== C13 save migration end-to-end (5 fixtures v1 → v2) ===\n");
    let mut all_pass = true;
    for seed in 1..=5 {
        let v1 = make_v1_fixture(seed);
        let bytes = postcard::to_allocvec(&v1).expect("ser v1");
        let loaded = match load(&bytes) {
            Ok(v) => v,
            Err(e) => {
                println!("fixture {seed}: FAIL load — {e:?}");
                all_pass = false;
                continue;
            }
        };
        // Verify migration preserved progress.
        let manual = migrate_v1_to_v2(v1.clone());
        if loaded != manual {
            println!("fixture {seed}: FAIL — load() != migrate()");
            all_pass = false;
            continue;
        }
        // Spot-check key invariants.
        if loaded.version != 2 {
            println!(
                "fixture {seed}: FAIL — version {} (esperava 2)",
                loaded.version
            );
            all_pass = false;
            continue;
        }
        if loaded.health.value != v1.health.value || loaded.health.max != v1.health.max {
            println!("fixture {seed}: FAIL — health values mudaram");
            all_pass = false;
            continue;
        }
        if loaded.health.regen_rate != 0.0 {
            println!("fixture {seed}: FAIL — regen_rate default não-zero");
            all_pass = false;
            continue;
        }
        if loaded.state_table.last_hit_tick != 0 {
            println!("fixture {seed}: FAIL — last_hit_tick default não-zero");
            all_pass = false;
            continue;
        }
        if loaded.state_table.flags != v1.state_table.flags {
            println!("fixture {seed}: FAIL — flags mudaram");
            all_pass = false;
            continue;
        }
        println!(
            "fixture {seed}: PASS — v1 ({} bytes) → v2 ({} bytes)",
            bytes.len(),
            postcard::to_allocvec(&loaded).unwrap().len(),
        );
    }

    // Idempotência: v2 round-trip não muda nada.
    let v2 = migrate_v1_to_v2(make_v1_fixture(1));
    let bytes_v2 = postcard::to_allocvec(&v2).unwrap();
    let reloaded = load(&bytes_v2).unwrap();
    let idempotent = reloaded == v2;
    println!(
        "\nidempotência v2→v2: {}",
        if idempotent { "PASS" } else { "FAIL" }
    );
    if !idempotent {
        all_pass = false;
    }

    if all_pass {
        println!(
            "\nC13 PASS — 5/5 fixtures migram, v2 idempotente, regen_rate/last_hit_tick zeram corretamente"
        );
    } else {
        println!("\nC13 FAIL");
        std::process::exit(1);
    }
}
