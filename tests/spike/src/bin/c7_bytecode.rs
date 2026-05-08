//! C7 spike — bytecode ship build comparison.
//!
//! Compara: load de source `.luau` vs load de bytecode pré-compilado.
//! Mede:
//! - size delta (bytecode bytes vs source bytes)
//! - cold-start time (Lua::new() + load + first execute)
//!
//! Threshold C7 (docs/spike/2026-05-plan.md L122-126):
//! - cold start ≤ 100 ms (medido em iPad Pro M2 — aqui medimos no Mac M2 como
//!   proxy; iPad pode ser 1.5-2x mais lento mas a margem deve segurar).
//! - bytecode size ≤ 70% da source size.
//!
//! Fonte do script: composição sintética de gameplay típico (~3 KB de Luau).

use flate2::{Compression, write::GzEncoder};
use mlua::{Compiler, Lua};
use std::io::Write;
use std::time::Instant;

fn gzip_size(bytes: &[u8]) -> usize {
    let mut enc = GzEncoder::new(Vec::new(), Compression::best());
    enc.write_all(bytes).unwrap();
    enc.finish().unwrap().len()
}

const HELLO: &str = include_str!("../../hello.luau");
const CORO: &str = include_str!("../../coro_timing.luau");
const GC: &str = include_str!("../../gc_stress.luau");
const TYPICAL: &str = include_str!("../../c7_typical_script.luau");

fn percentile(sorted: &[f64], p: f64) -> f64 {
    if sorted.is_empty() { return 0.0; }
    let idx = ((sorted.len() as f64 - 1.0) * p).round() as usize;
    sorted[idx]
}

fn cold_start_source(source: &str) -> std::io::Result<f64> {
    let t0 = Instant::now();
    let lua = Lua::new();
    lua.load(source).exec().unwrap();
    Ok(t0.elapsed().as_secs_f64() * 1000.0)
}

fn cold_start_bytecode(bytecode: &[u8]) -> std::io::Result<f64> {
    let t0 = Instant::now();
    let lua = Lua::new();
    lua.load(bytecode).exec().unwrap();
    Ok(t0.elapsed().as_secs_f64() * 1000.0)
}

fn measure(label: &str, source: &str) -> mlua::Result<()> {
    // Ship build settings: O2, zero debug info, zero coverage.
    let compiler = Compiler::new()
        .set_optimization_level(2)
        .set_debug_level(0)
        .set_coverage_level(0);
    let bytecode = compiler.compile(source)?;
    let src_bytes = source.len();
    let bc_bytes = bytecode.len();
    let ratio = (bc_bytes as f64 / src_bytes as f64) * 100.0;
    let src_gz = gzip_size(source.as_bytes());
    let bc_gz = gzip_size(&bytecode);
    let ratio_gz = (bc_gz as f64 / src_gz as f64) * 100.0;

    println!("\n--- {label} ({src_bytes} bytes source) ---");
    println!("bytecode (raw):     {bc_bytes} bytes ({ratio:.1}% of source)");
    println!("bytecode (gzipped): {bc_gz} bytes vs source gzipped {src_gz} bytes ({ratio_gz:.1}%)");

    // Warm up + sample N=20 cold starts (each instantiates a fresh Lua VM).
    let mut src_times = Vec::with_capacity(20);
    let mut bc_times = Vec::with_capacity(20);
    for _ in 0..20 {
        src_times.push(cold_start_source(source).unwrap());
        bc_times.push(cold_start_bytecode(&bytecode).unwrap());
    }
    src_times.sort_by(|a, b| a.partial_cmp(b).unwrap());
    bc_times.sort_by(|a, b| a.partial_cmp(b).unwrap());

    println!("cold start (Lua::new + load + exec, p50/p99 of 20 samples):");
    println!("  source:   p50 {:.2} ms / p99 {:.2} ms", percentile(&src_times, 0.50), percentile(&src_times, 0.99));
    println!("  bytecode: p50 {:.2} ms / p99 {:.2} ms", percentile(&bc_times, 0.50), percentile(&bc_times, 0.99));

    let bc_p99 = percentile(&bc_times, 0.99);
    let pass_size = ratio <= 70.0;
    let pass_time = bc_p99 <= 100.0;
    println!("  size threshold (≤70%): {}", if pass_size { "PASS" } else { "FAIL" });
    println!("  time threshold (≤100ms p99 on Mac proxy): {}", if pass_time { "PASS" } else { "FAIL" });
    Ok(())
}

fn main() -> mlua::Result<()> {
    println!("=== C7 bytecode ship build ===");
    measure("hello.luau (tiny)", HELLO)?;
    measure("coro_timing.luau (tiny)", CORO)?;
    measure("gc_stress.luau (small)", GC)?;
    measure("c7_typical_script.luau (gameplay-sized)", TYPICAL)?;
    println!("\nNote: cold-start medido em Mac M-series. iPad Pro M2 \
provavelmente ~1.5x. Reaferir on-device em S3.");
    Ok(())
}
