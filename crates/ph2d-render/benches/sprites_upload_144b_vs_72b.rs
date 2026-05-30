//! Frame-budget bench for the v4 `RenderInstance` ABI (Sprite Inspector
//! v2 — Sprite_projeto §10.5 / ADR-0070 §2.5 / handoff W1.T1.7b).
//!
//! ## What this measures (and what it deliberately does NOT)
//!
//! The v4 ABI doubled the instance stride (72 → 144 bytes). The spec's
//! open question (§10.5 "Mitigação opcional W2 decision" + ADR-0070 §2.5)
//! is whether that doubling makes per-frame instance upload a bottleneck
//! at the 10k-sprite scale — and if so, split into a compact (72B) +
//! extended (72B) two-buffer scheme.
//!
//! This bench measures the **CPU-side per-frame instance-construction
//! cost attributable to the stride**: building 10k instances into a
//! contiguous Vec (the dominant per-frame CPU work — allocation + the
//! per-field writes that scale with the struct size). It benches the v4
//! (144B) path against a v3-shaped (72B) baseline so the criterion report
//! shows the construction delta directly.
//!
//! NOTE on the `bytemuck::cast_slice`: it is a pointer reinterpret
//! (`len * size_of`), NOT a copy — `black_box(bytes.len())` keeps the
//! built Vec live but does NOT force a byte traversal. So this measures
//! build + alloc, not the `Queue::write_buffer` memcpy. That memcpy
//! (1.44 MB vs 720 KB for 10k) is also low-µs and, like GPU bandwidth, is
//! a W2 / Enio-smoke concern, not this CI-stable CPU bench.
//!
//! It does NOT measure GPU upload bandwidth (PCIe/unified-memory copy +
//! vertex fetch), which needs a live device and is non-deterministic
//! across machines/CI runners. Per ADR-0070, that GPU-side dual-buffer
//! decision is explicitly a **W2** call, informed by the Enio smoke and
//! a GPU-profiled run — not this CI-stable CPU bench.
//!
//! ## Threshold (interpretive, not a hard assert)
//!
//! Budget: 10k sprites @ 60 Hz = 16.7 ms/frame. The handoff trigger for
//! the reserved `-1` dual-buffer amendment (ADR-0070 §2.5; the amendment
//! doc is not yet written — slot `-1` is pre-reserved per
//! ADR-0070-amendment-2/-3) is the v4 build exceeding **8 ms** on an
//! M-series tier. Criterion only MEASURES; it does not
//! fail on a wall-clock threshold (that would be non-deterministic). If
//! the reported `build_and_cast_v4_144b/10000` time approaches 8 ms,
//! that is the signal to open amendment-1. Realistic expectation: tens
//! of µs (a 1.44 MB build + memcpy), i.e. ~0.x % of frame — 144B is not
//! a CPU bottleneck; any real cost is GPU bandwidth, deferred to W2.

use criterion::{Criterion, criterion_group, criterion_main};
use ph2d_render::RenderInstance;
use std::hint::black_box;

const N: usize = 10_000;

/// v3-shaped instance (the pre-v4 72-byte layout) used ONLY as the
/// bandwidth baseline for the comparison. Mirrors the v3 field set so
/// `size_of` is 72; it is never uploaded — just built + cast like the
/// v4 path so the two measurements are apples-to-apples.
#[repr(C)]
#[derive(Copy, Clone)]
struct RenderInstanceV3Baseline {
    world_pos: [f32; 2],
    size: [f32; 2],
    atlas_uv: [f32; 4],
    tint: [f32; 4],
    rotation: f32,
    premultiplied: f32,
    anchor: [f32; 2],
    texture_id: u32,
    z_order: u32,
}

// SAFETY: `#[repr(C)]`, all fields are `f32`/`u32` (Pod), no padding
// (every field 4-byte-grained, 72 % 4 == 0). Local to the bench; only
// used for a byte-slice cast of an owned Vec.
unsafe impl bytemuck::Zeroable for RenderInstanceV3Baseline {}
unsafe impl bytemuck::Pod for RenderInstanceV3Baseline {}

fn build_v4(n: usize) -> Vec<RenderInstance> {
    (0..n)
        .map(|i| {
            let f = i as f32;
            // ADR-0070-amendment-4: a small rotation as a 2×2 basis
            // (stride is now 156 B, not 144 — `rotation: f32` → `basis`).
            let (s, c) = (f * 0.001).sin_cos();
            RenderInstance {
                world_pos: [f, -f],
                size: [16.0, 16.0],
                atlas_uv: [0.0, 0.0, 1.0, 1.0],
                tint: [1.0, 1.0, 1.0, 1.0],
                basis: [c, s, -s, c],
                premultiplied: 0.0,
                anchor: [0.0, 0.0],
                per_corner_tint: [[1.0; 4]; 4],
                opacity: 1.0,
                flip_uv: 0,
                texture_id: (i % 8) as u32,
                z_order: i as u32,
            }
        })
        .collect()
}

fn build_v3(n: usize) -> Vec<RenderInstanceV3Baseline> {
    (0..n)
        .map(|i| {
            let f = i as f32;
            RenderInstanceV3Baseline {
                world_pos: [f, -f],
                size: [16.0, 16.0],
                atlas_uv: [0.0, 0.0, 1.0, 1.0],
                tint: [1.0, 1.0, 1.0, 1.0],
                rotation: f * 0.001,
                premultiplied: 0.0,
                anchor: [0.0, 0.0],
                texture_id: (i % 8) as u32,
                z_order: i as u32,
            }
        })
        .collect()
}

fn bench_upload(c: &mut Criterion) {
    // Sanity-pin the strides the comparison rests on, so a future ABI
    // edit that changes them makes the bench's premise visibly wrong.
    assert_eq!(std::mem::size_of::<RenderInstance>(), 144);
    assert_eq!(std::mem::size_of::<RenderInstanceV3Baseline>(), 72);

    let mut group = c.benchmark_group("sprites_upload");
    group.bench_function("build_and_cast_v4_144b/10000", |b| {
        b.iter(|| {
            let v = build_v4(black_box(N));
            let bytes: &[u8] = bytemuck::cast_slice(&v);
            black_box(bytes.len())
        });
    });
    group.bench_function("build_and_cast_v3_72b/10000", |b| {
        b.iter(|| {
            let v = build_v3(black_box(N));
            let bytes: &[u8] = bytemuck::cast_slice(&v);
            black_box(bytes.len())
        });
    });
    group.finish();
}

criterion_group!(benches, bench_upload);
criterion_main!(benches);
