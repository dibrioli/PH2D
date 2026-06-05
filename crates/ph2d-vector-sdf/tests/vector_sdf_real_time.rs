//! Gate `vector_sdf_real_time` (plan §8 T5.2) — the SDF Hybrid draft must
//! preview N concurrent boolean ops / 50+ paths in real time.
//!
//! Two parts:
//! - A **CI-green correctness gate**: the draft pipeline (network_sdf → boolean_sdf
//!   → marching) handles a dense 64-path boolean (2 × 32-path operands) and yields
//!   a non-empty silhouette, within a generous CPU regression bound (the CPU path
//!   is the deterministic fallback; the real-time claim is the GPU companion).
//! - A **GPU 120 FPS gate** (`#[ignore]`, needs a device): the same 64-path boolean
//!   on the GPU must fit the 8.33 ms ProMotion budget.

use ph2d_gpu::GpuContext;
use ph2d_vector_doc::{Region, Segment, VectorNetwork, Vertex, VertexId, VertexKind, WindingRule};
use ph2d_vector_sdf::gpu::GpuSdf;
use ph2d_vector_sdf::{Bounds, SdfOp, boolean_sdf, marching_contour, network_sdf};
use std::time::Instant;

/// Draft grid resolution (matches the bridge's interactive draft res).
const RES: u32 = 96;

/// A network of `count` 10×10 squares laid out on an 8-wide grid, shifted by
/// `(dx, dy)` — a quick way to build a dense multi-path (multi-region) operand.
fn scatter_squares(count: u32, dx: f32, dy: f32) -> VectorNetwork {
    let mut net = VectorNetwork::empty();
    let (mut vid, mut sid): (VertexId, u32) = (0, 0);
    for k in 0..count {
        let cx = (k % 8) as f32 * 30.0 + dx;
        let cy = (k / 8) as f32 * 30.0 + dy;
        let base = vid;
        for &(x, y) in &[
            (cx, cy),
            (cx + 10.0, cy),
            (cx + 10.0, cy + 10.0),
            (cx, cy + 10.0),
        ] {
            net.vertices
                .push(Vertex::new(vid, glam::Vec2::new(x, y), VertexKind::Auto));
            vid += 1;
        }
        let mut region = Region::new(k, WindingRule::NonZero);
        for i in 0..4u32 {
            net.segments
                .push(Segment::straight(sid, base + i, base + (i + 1) % 4));
            region.segments.push((sid, true));
            sid += 1;
        }
        net.regions.push(region);
    }
    net
}

/// Co-located sampling window over both operands (so `boolean_sdf` can combine).
fn shared_bounds(a: &VectorNetwork, b: &VectorNetwork) -> Bounds {
    let ba = Bounds::of_network(a, 8.0);
    let bb = Bounds::of_network(b, 8.0);
    Bounds {
        min: ba.min.min(bb.min),
        max: ba.max.max(bb.max),
    }
}

#[test]
fn vector_sdf_real_time_handles_64_path_boolean_cpu() {
    // 2 × 32 squares = a 64-path Union; the draft must produce a contour.
    let a = scatter_squares(32, 0.0, 0.0);
    let b = scatter_squares(32, 15.0, 15.0);
    let bounds = shared_bounds(&a, &b);

    let t = Instant::now();
    let sdf_a = network_sdf(&a, RES, bounds);
    let sdf_b = network_sdf(&b, RES, bounds);
    let draft = boolean_sdf(&sdf_a, &sdf_b, SdfOp::Union).expect("co-located grids combine");
    let contour = marching_contour(&draft);
    let ms = t.elapsed().as_secs_f64() * 1000.0;

    assert!(!contour.is_empty(), "a 64-path silhouette has a contour");
    eprintln!("[sdf] CPU 64-path draft @ {RES}²: {ms:.3} ms");
    // Generous CPU regression guard — the GPU is the real-time path (see below).
    assert!(ms < 250.0, "pathological CPU regression: {ms} ms");
}

#[test]
#[ignore = "needs a GPU device — the 120 FPS real-time gate; run with --include-ignored"]
fn vector_sdf_real_time_gpu_64_paths_under_120fps() {
    let Some(gpu) = GpuContext::new(GpuContext::default_instance(), None).ok() else {
        eprintln!("no GPU — skipping");
        return;
    };
    let a = scatter_squares(32, 0.0, 0.0);
    let b = scatter_squares(32, 15.0, 15.0);
    let bounds = shared_bounds(&a, &b);
    let pipe = GpuSdf::new(&gpu);

    // Warm the pipeline (first dispatch allocates).
    let _ = pipe.network_sdf(&gpu, &a, RES, bounds);

    let iters = 30;
    let t = Instant::now();
    for _ in 0..iters {
        let sdf_a = pipe.network_sdf(&gpu, &a, RES, bounds);
        let sdf_b = pipe.network_sdf(&gpu, &b, RES, bounds);
        let draft = boolean_sdf(&sdf_a, &sdf_b, SdfOp::Union).expect("grids combine");
        let _ = marching_contour(&draft);
    }
    let per_frame = t.elapsed().as_secs_f64() * 1000.0 / f64::from(iters);
    eprintln!("[sdf] GPU 64-path boolean draft @ {RES}²: {per_frame:.3} ms/frame");
    assert!(
        per_frame < 8.33,
        "64-path SDF draft must fit the 120 FPS budget, got {per_frame} ms/frame"
    );
}
