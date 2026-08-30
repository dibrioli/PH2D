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
    if sorted.is_empty() {
        return 0.0;
    }
    let idx = ((sorted.len() as f64 - 1.0) * p).round() as usize;
    sorted[idx]
}

fn main() -> mlua::Result<()> {
    let runtime = ScriptRuntime::new()?;
    runtime.eval(SCRIPT)?;
    let lua: &Lua = runtime.lua();
    let tick_alloc: Function = lua.globals().get("tick_alloc")?;

    // Luau default GC mode is incremental; `gc_step` runs one step.
    //
    // ⚠️ **`gc_step_kbytes(kb)` deixou de existir no `mlua` 0.12**, e a substituta não
    // recebe orçamento: `gc_step()` faz *"um passo básico, que em modo incremental
    // corresponde ao **tamanho de passo corrente**"*. O orçamento mudou de sítio — passou
    // do ponto de chamada para o **modo do coletor** (`gc_set_mode` +
    // `GcIncParams::step_size`), o que é a forma certa: um budget passado por chamada era
    // uma resposta por-quadro a uma pergunta que é de configuração.
    // ⇒ os números que esta sonda imprime **não são comparáveis** com os de uma corrida
    // anterior à subida: ela media passos de 1 KB e passa a medir o passo por omissão do
    // Luau. Isto é uma sonda de spike, não um gate — ninguém depende do valor.

    let mut pauses_ms: Vec<f64> = Vec::with_capacity(FRAMES);
    for _ in 0..FRAMES {
        let _: i64 = tick_alloc.call(())?;
        let t0 = Instant::now();
        // Um passo do coletor, no tamanho que o modo corrente define.
        lua.gc_step()?;
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
