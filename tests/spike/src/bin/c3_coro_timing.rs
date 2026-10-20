//! C3 spike — coroutine timing measurement, sobre as ESPERAS DA CENA (`ph2d.wait` num gancho, o
//! `SceneScripts` que as retoma; CodeEditor item 6 — o `Scheduler` de protótipo saiu).
//!
//! Spawn 1000 coroutines com waits distribuidos em [0.500, 0.510)s para evitar
//! todos coincidirem com o mesmo tick boundary. Tick scheduler em dt fixo
//! 16.667ms (60Hz). Mede |actual - target| por task.
//!
//! Threshold C3 (docs/spike/2026-05-plan.md L57-60):
//! - |actual − target| ≤ 16.6 ms p99 (1 frame).
//! - **Interpretação:** o plano escreveu "16.6 ms" como aproximação de "1
//!   frame at 60Hz". Frame real = 1/60 = 16.667 ms. PASS se p99 ≤ dt = 16.667ms.
//! - Zero coroutines vazadas após 1.000 ciclos.

use ph2d_ecs::{SimWorld, StableId, Transform};
use ph2d_script::{LuauScript, ProjectScripts, ScriptHost, ScriptValue};

const SCRIPT: &str = include_str!("../../scene_waits.luau");
const N_TASKS: usize = 1000;
const BASE_TARGET_S: f64 = 0.500;
const SPREAD_S: f64 = 0.010; // distribute waits in [0.500, 0.510)
const DT_S: f64 = 1.0 / 60.0;
const FRAME_MS: f64 = DT_S * 1000.0;
const MAX_TICKS: usize = 120;

fn percentile(sorted: &[f64], p: f64) -> f64 {
    if sorted.is_empty() {
        return 0.0;
    }
    let idx = ((sorted.len() as f64 - 1.0) * p).round() as usize;
    sorted[idx]
}

fn main() -> mlua::Result<()> {
    let mut sim = SimWorld::new();
    let id = sim
        .world_mut()
        .get_resource_or_init::<ProjectScripts>()
        .insert("c3.luau", SCRIPT);
    let mut objects = Vec::with_capacity(N_TASKS);
    for i in 0..N_TASKS {
        let secs = BASE_TARGET_S + (i as f64 / N_TASKS as f64) * SPREAD_S;
        let mut script = LuauScript::library(id);
        script.own.insert("alvo".into(), ScriptValue::Number(secs));
        let e = sim
            .world_mut()
            .spawn((Transform::IDENTITY, StableId(i as u64 + 1), script))
            .id();
        objects.push(e);
    }
    let mut host = ScriptHost::new()?;
    for tick in 1..=MAX_TICKS {
        host.scene_sync(sim.world_mut());
        let r = host.scene_tick(&mut sim, DT_S);
        assert!(r.failed.is_empty(), "{:?}", r.failed);
        if host.scene().waiting_count() == 0 {
            println!("all waits resumed by tick {tick}");
            break;
        }
    }
    assert_eq!(
        host.scene().waiting_count(),
        0,
        "leak: the scene still has pending waits"
    );

    let mut errors_ms: Vec<f64> = objects
        .iter()
        .map(|e| {
            let t = sim.world().get::<Transform>(*e).expect("pose");
            f64::from(t.translation.x) * 1000.0
        })
        .collect();
    assert!(
        errors_ms.iter().all(|e| *e >= 0.0),
        "a wait resumed before its target"
    );

    errors_ms.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let p50 = percentile(&errors_ms, 0.50);
    let p95 = percentile(&errors_ms, 0.95);
    let p99 = percentile(&errors_ms, 0.99);
    let max = *errors_ms.last().unwrap();

    println!(
        "\n=== C3 results ({N_TASKS} scene waits, targets in [{BASE_TARGET_S:.3}, {:.3})s, dt {FRAME_MS:.3}ms) ===",
        BASE_TARGET_S + SPREAD_S
    );
    println!("error (actual − target) in ms:");
    println!("  p50:  {p50:.3} ms");
    println!("  p95:  {p95:.3} ms");
    println!("  p99:  {p99:.3} ms");
    println!("  max:  {max:.3} ms");
    println!("threshold p99 ≤ {FRAME_MS:.3} ms (1 frame at 60Hz)");

    let pass = p99 <= FRAME_MS + 1e-3;
    if pass {
        println!("\nC3 PASS — p99 within 1 frame, 0 leaks ({N_TASKS}/{N_TASKS} resumed)");
        Ok(())
    } else {
        println!("\nC3 FAIL — p99 exceeds 1 frame");
        std::process::exit(1);
    }
}
