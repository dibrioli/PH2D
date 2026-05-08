//! C12 spike — WASM marshalling latency.
//!
//! Mede latência por chamada Luau → Rust → wasmtime call com payload primitivo
//! (i32). WASM module é trivial (`add_one(i32) -> i32`) para isolar overhead
//! puro de marshalling.
//!
//! Threshold C12 (docs/spike/2026-05-plan.md L141-144):
//! - sub-1 µs p99 em hardware alvo (Mac M2 medido aqui).
//! - Para payload >256 bytes, usar buffer Luau zero-copy compartilhado.
//!   Esta bench testa só primitivo i32 (caso comum).

use mlua::{Function, Lua};
use ph2d_script::ScriptRuntime;
use std::time::Instant;
use wasmtime::{Engine, Instance, Module, Store, TypedFunc};

const WAT: &str = r#"
(module
  (func (export "add_one") (param i32) (result i32)
    local.get 0
    i32.const 1
    i32.add
  )
  (func (export "compute") (param i32 f32) (result f32)
    local.get 1
    local.get 0
    f32.convert_i32_s
    f32.add
  )
)
"#;

const N_CALLS: usize = 100_000;

fn percentile(sorted: &[f64], p: f64) -> f64 {
    if sorted.is_empty() { return 0.0; }
    let idx = ((sorted.len() as f64 - 1.0) * p).round() as usize;
    sorted[idx]
}

fn main() -> mlua::Result<()> {
    // Set up wasmtime
    let engine = Engine::default();
    let module = Module::new(&engine, WAT).expect("compile WASM");
    let mut store = Store::new(&engine, ());
    let instance = Instance::new(&mut store, &module, &[]).expect("instantiate");
    let add_one: TypedFunc<i32, i32> = instance
        .get_typed_func(&mut store, "add_one")
        .expect("typed func add_one");

    // Set up Luau with a Rust-bridged function that calls into WASM.
    let runtime = ScriptRuntime::new()?;
    let lua: &Lua = runtime.lua();

    // Pre-build the bridge: Luau-callable function that internally calls WASM.
    // We can't capture wasmtime's Store/TypedFunc in a closure that mlua accepts
    // directly (lifetime + Send constraints). Instead, measure each layer
    // separately and report the composition.

    // Path A: pure WASM call (Rust → wasmtime)
    let mut wasm_only = Vec::with_capacity(N_CALLS);
    for i in 0..N_CALLS {
        let t0 = Instant::now();
        let _ = add_one.call(&mut store, i as i32).unwrap();
        wasm_only.push(t0.elapsed().as_secs_f64() * 1e6); // µs
    }

    // Path B: Luau → Rust callback (no WASM)
    lua.globals().set(
        "rust_callback",
        lua.create_function(|_, n: i32| Ok(n + 1))?,
    )?;
    let bench_luau_callback: Function = lua
        .load(
            r#"
return function(n)
    return rust_callback(n)
end
"#,
        )
        .eval()?;
    let mut luau_to_rust = Vec::with_capacity(N_CALLS);
    for i in 0..N_CALLS {
        let t0 = Instant::now();
        let _: i32 = bench_luau_callback.call(i as i32)?;
        luau_to_rust.push(t0.elapsed().as_secs_f64() * 1e6);
    }

    // Path C: full chain Luau → Rust → WASM via thread-local closure
    // (simulação — wasmtime Store não é Send/Sync, então usamos uma function
    // que faz a chamada via std thread-local. Aqui simplificamos medindo o
    // upper bound: Luau call overhead + WASM call overhead somados.)
    let upper_bound_full: Vec<f64> = wasm_only
        .iter()
        .zip(luau_to_rust.iter())
        .map(|(w, l)| w + l)
        .collect();

    let sort_helper = |v: &mut Vec<f64>| v.sort_by(|a, b| a.partial_cmp(b).unwrap());
    sort_helper(&mut wasm_only);
    sort_helper(&mut luau_to_rust);
    let mut full = upper_bound_full;
    sort_helper(&mut full);

    println!("=== C12 WASM marshalling latency ({N_CALLS} calls each) ===\n");
    let print_stats = |label: &str, v: &[f64]| {
        let p50 = percentile(v, 0.50);
        let p95 = percentile(v, 0.95);
        let p99 = percentile(v, 0.99);
        let max = v.last().copied().unwrap_or(0.0);
        println!("{label:30}  p50 {p50:.3} µs  p95 {p95:.3} µs  p99 {p99:.3} µs  max {max:.3} µs");
    };
    print_stats("Rust → wasmtime (i32→i32)", &wasm_only);
    print_stats("Luau → Rust callback (i32)", &luau_to_rust);
    print_stats("Upper bound full chain", &full);
    println!("\nthreshold p99 ≤ 1.000 µs");

    let p99_full = percentile(&full, 0.99);
    let pass = p99_full <= 1.0;
    if pass {
        println!("\nC12 PASS — full chain p99 within sub-µs budget");
        Ok(())
    } else {
        println!(
            "\nC12 FAIL — full chain p99 = {p99_full:.3} µs (threshold 1.000 µs).\n\
             Implicação: para hot-path callbacks frequentes, agrupar chamadas\n\
             ou usar WASM diretamente (sem trampoline Luau)."
        );
        std::process::exit(1);
    }
}
