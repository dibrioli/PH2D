//! C14 spike — memory budget compliance (HR-13).
//!
//! Stress fixture: 10_000 ECS entities + 1_000 Luau coroutines pendentes +
//! 200 active Luau scripts (cada uma com small state) + 50 WASM modules
//! instanciados. Mede memory total (process RSS).
//!
//! Threshold C14 (docs/spike/2026-05-plan.md L152-155):
//! - total < ~1000 MB iPad app target (HR-13).
//! - Mac M-series tem mais RAM disponível que iPad — medição aqui é proxy
//!   superior; iPad real será ~similar ou um pouco menor (sem Rosetta etc).

use bevy_ecs::prelude::*;
use mlua::Function;
use ph2d_script::{Scheduler, ScriptRuntime};
use std::process::Command;
use wasmtime::{Engine, Module};

// Components are populated for the stress workload but never read — the
// purpose is to measure ECS world memory per (entity × 4 component) cell.
#[allow(dead_code)]
#[derive(Component)]
struct Position { x: f32, y: f32 }

#[allow(dead_code)]
#[derive(Component)]
struct Velocity { x: f32, y: f32 }

#[allow(dead_code)]
#[derive(Component)]
struct Health(i32);

#[allow(dead_code)]
#[derive(Component)]
struct Tag(u8);

const N_ENTITIES: u64 = 10_000;
const N_COROUTINES: usize = 1_000;
const N_SCRIPTS: usize = 200;
const N_WASM_MODULES: usize = 50;

const BUDGET_MB: f64 = 1000.0;

const SCRIPT_BODY: &str = r#"
ph2d = ph2d or {}
function ph2d.wait(seconds)
    coroutine.yield(seconds)
end
function task_body(seconds)
    ph2d.wait(seconds)
end
local state = {}
for i = 1, 50 do
    state[i] = { x = i, y = i * 2, name = "obj_" .. i, flags = { a = true, b = false } }
end
return state
"#;

const WAT: &str = r#"
(module
  (memory (export "mem") 1)
  (func (export "noop") (param i32) (result i32)
    local.get 0
    i32.const 1
    i32.add
  )
)
"#;

/// Read RSS memory in MB (macOS).
fn rss_mb() -> f64 {
    // ps -o rss= -p <pid> on macOS returns RSS in KB
    let pid = std::process::id();
    let out = Command::new("ps")
        .args(["-o", "rss=", "-p", &pid.to_string()])
        .output()
        .expect("ps");
    let s = String::from_utf8_lossy(&out.stdout);
    let kb: f64 = s.trim().parse().unwrap_or(0.0);
    kb / 1024.0
}

fn main() -> mlua::Result<()> {
    let baseline_mb = rss_mb();
    println!("=== C14 memory budget ===");
    println!("baseline RSS: {baseline_mb:.1} MB\n");

    // ECS world: 10_000 entities × 4 components
    let mut world = World::new();
    for i in 0..N_ENTITIES {
        world.spawn((
            Position { x: i as f32, y: 0.0 },
            Velocity { x: 1.0, y: 0.0 },
            Health((i % 200) as i32),
            Tag((i % 16) as u8),
        ));
    }
    let after_ecs = rss_mb();
    println!("after 10k entities × 4 components: {after_ecs:.1} MB  (Δ {:.1} MB)", after_ecs - baseline_mb);

    // 200 Luau scripts ativos (cada um com state heap)
    let mut runtimes: Vec<ScriptRuntime> = Vec::with_capacity(N_SCRIPTS);
    for _ in 0..N_SCRIPTS {
        let rt = ScriptRuntime::new()?;
        rt.eval(SCRIPT_BODY)?;
        runtimes.push(rt);
    }
    let after_scripts = rss_mb();
    println!("after 200 Luau script runtimes (each with state): {after_scripts:.1} MB  (Δ {:.1} MB)", after_scripts - after_ecs);

    // 1000 coroutines em um único runtime (sharing)
    let coro_rt = ScriptRuntime::new()?;
    coro_rt.eval(SCRIPT_BODY)?;
    let task_body: Function = coro_rt.lua().globals().get("task_body")?;
    let mut sched = Scheduler::new();
    for _ in 0..N_COROUTINES {
        let thread = coro_rt.spawn(task_body.clone())?;
        sched.add(thread, 1000.0)?; // wait long enough to never fire
    }
    let after_coroutines = rss_mb();
    println!("after 1k coroutines pending: {after_coroutines:.1} MB  (Δ {:.1} MB)", after_coroutines - after_scripts);

    // 50 WASM modules instanciados
    let engine = Engine::default();
    let module = Module::new(&engine, WAT).expect("compile WASM");
    let mut wasm_instances: Vec<wasmtime::Instance> = Vec::with_capacity(N_WASM_MODULES);
    let mut store = wasmtime::Store::new(&engine, ());
    for _ in 0..N_WASM_MODULES {
        let inst = wasmtime::Instance::new(&mut store, &module, &[]).expect("instantiate");
        wasm_instances.push(inst);
    }
    let after_wasm = rss_mb();
    println!("after 50 WASM modules instanciados: {after_wasm:.1} MB  (Δ {:.1} MB)", after_wasm - after_coroutines);

    let total_growth = after_wasm - baseline_mb;
    println!("\nTotal stress growth: {total_growth:.1} MB");
    println!("Total RSS: {after_wasm:.1} MB (budget: {BUDGET_MB:.0} MB iPad-target)");

    let pass = after_wasm <= BUDGET_MB;
    if pass {
        println!("\nC14 PASS — total memory dentro do budget iPad");
        Ok(())
    } else {
        println!("\nC14 FAIL — total RSS {after_wasm:.1} MB excede {BUDGET_MB:.0} MB");
        std::process::exit(1);
    }
}
