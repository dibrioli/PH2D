//! C10 spike — GC pause sob stress.
//!
//! Fixture: 10_000 tabelas Lua persistentes + 1_000 coroutines pending +
//! per-tick allocation (~50 short-lived tables/frame). Loop 600 frames.
//! Mede pause máximo de `gc_step` por frame.
//!
//! Threshold C10 (docs/spike/2026-05-plan.md L76-78):
//! - ≤ 1.5 ms p99
//! - Acima → mover heavy logic para WASM ou reduzir alocação JS é mandatório.
//!
//! Importante: Luau usa GC incremental mark-sweep com generational mode.
//! `gc_step` em mlua roda 1 step incremental (não full collection). O pause
//! de cada step é o que importa para HR-9 (frame budget).

use mlua::{Function, Lua};
use ph2d_script::ScriptRuntime;
use std::time::Instant;

const SCRIPT: &str = include_str!("../../gc_stress.luau");
const FRAMES: usize = 600;

fn percentile(sorted: &[f64], p: f64) -> f64 {
    if sorted.is_empty() { return 0.0; }
    let idx = ((sorted.len() as f64 - 1.0) * p).round() as usize;
    sorted[idx]
}

fn main() -> mlua::Result<()> {
    let runtime = ScriptRuntime::new()?;
    runtime.eval(SCRIPT)?;
    let lua: &Lua = runtime.lua();
    let tick_alloc: Function = lua.globals().get("tick_alloc")?;

    // Luau default GC mode is incremental; gc_step_kbytes runs one step.
    // Per-frame step budget chosen to keep heap stable under per-tick allocation.

    let mut pauses_ms: Vec<f64> = Vec::with_capacity(FRAMES);
    for _ in 0..FRAMES {
        let _: i64 = tick_alloc.call(())?;
        let t0 = Instant::now();
        // Step the collector; arg = number of bytes/units to collect.
        // 1024 is a reasonable per-frame budget step.
        lua.gc_step_kbytes(1)?;
        let elapsed = t0.elapsed();
        pauses_ms.push(elapsed.as_secs_f64() * 1000.0);
    }

    pauses_ms.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let p50 = percentile(&pauses_ms, 0.50);
    let p95 = percentile(&pauses_ms, 0.95);
    let p99 = percentile(&pauses_ms, 0.99);
    let max = *pauses_ms.last().unwrap();

    let mem_kb = lua.used_memory() as f64 / 1024.0;
    println!("=== C10 results ({FRAMES} frames, GC mode Incremental) ===");
    println!("Lua heap: {mem_kb:.1} KB");
    println!("GC step pause (per frame):");
    println!("  p50:  {p50:.4} ms");
    println!("  p95:  {p95:.4} ms");
    println!("  p99:  {p99:.4} ms");
    println!("  max:  {max:.4} ms");
    println!("threshold p99 ≤ 1.5 ms");

    // Sanity check: also measure a forced full collection as worst-case upper bound.
    let t0 = Instant::now();
    lua.gc_collect()?;
    let full_collect_ms = t0.elapsed().as_secs_f64() * 1000.0;
    println!("\nFull `gc_collect()` (worst-case upper bound): {full_collect_ms:.3} ms");

    let pass = p99 <= 1.5;
    if pass {
        println!("\nC10 PASS — GC step pause within budget");
        Ok(())
    } else {
        println!("\nC10 FAIL — GC step pause exceeds 1.5 ms p99");
        std::process::exit(1);
    }
}
