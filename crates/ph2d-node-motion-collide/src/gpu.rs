//! The GPU kernel for `motion.collide` (ADR-0134 Fase 5) — the O(N²) push-apart,
//! answered on the device by the spatial grid.
//!
//! This is the node the grid was built for. Boids proved the machinery on a
//! simulation STEP (one dispatch per tick); this is a **relaxation SOLVER**, so it
//! sweeps `iterations` times, and every sweep moves the very column the grid
//! indexes — hence [`GridSpec::sweeps_param`], which makes the sequencer rebuild
//! the grid and feed the previous sweep's output back in.
//!
//! **It exists because the CPU became averaged Jacobi.** An in-place Gauss–Seidel
//! sweep cannot be run by a GPU at all — each pair must see the corrections of the
//! pairs before it, which is sequential by definition. The reference was changed
//! first (and for its own reason: Gauss–Seidel made the packing depend on the
//! stream's index order, measured at 1018 % of a disc diameter), and only then did
//! a device port become expressible. The two now compute the same thing, differing
//! solely in the ORDER of a float sum ⇒ ε (ADR-0134 D4).
//!
//! - **Gather, not scatter.** Each thread sums the corrections its OWN contacts ask
//!   of it and divides by their count. That is exactly the CPU's per-disc average,
//!   and it needs no atomics — a float `atomicAdd` does not exist in WGSL anyway.
//! - **The sweep radius is DERIVED, and then CULLED per disc.** The grid cell is
//!   `radius` but the interaction distance is `2·radius·spread`, so a hardcoded 3×3
//!   would silently miss contacts the moment `spread` moved: the reach is
//!   `ceil(min_dist / cell)` cells in each direction. But that ceiling is the WORST
//!   case — a disc pressed against the far edge of its own cell — and paying it on
//!   every disc made the cost a step function of `spread` rather than of the
//!   packing: crossing `spread = 1` stepped the sweep 25 cells → 49 and the cook
//!   **7.58 ms → 13.08 ms between two neighbouring values** (Enio's report on the
//!   breathing-packing demo, 2026-07-19). So each cell is asked whether its NEAREST
//!   point is even in range before its discs are touched — exact, never approximate
//!   (a contact needs `d2 < min_d2` and every point of the cell is at least `gap`
//!   away), which flattens the step to 1.00× and speeds the common case up ~40 %.

use ph2d_nodegraph::gpu::{ColumnAccess, ColumnBinding, GpuKernel, GridSpec};
use ph2d_nodegraph::port::Dim;

/// `P` rides port 0 (read + written); `inv_mass` is the PBD inverse mass on the
/// same port (absent ⇒ its identity `1` = free, which is the CPU's default); `v`
/// on port 1 is the animatable `spread` multiplier, broadcast (absent ⇒ `1`).
static BINDINGS: &[ColumnBinding] = &[
    ColumnBinding {
        column: "P",
        dim: Dim::Vec2,
        access: ColumnAccess::ReadWrite,
        identity: [0.0; 4],
        port: 0,
    },
    ColumnBinding {
        column: "inv_mass",
        dim: Dim::Scalar,
        access: ColumnAccess::Read,
        // 1 = free. The CPU reads an absent/mis-sized column as all-free, so the
        // identity IS that rule rather than a second spelling of it.
        identity: [1.0, 0.0, 0.0, 0.0],
        port: 0,
    },
    ColumnBinding {
        column: "v",
        dim: Dim::Scalar,
        access: ColumnAccess::ReadBroadcast,
        // Unconnected `spread` reads as 1 (`spread_amount`).
        identity: [1.0, 0.0, 0.0, 0.0],
        port: 1,
    },
];

/// `EPS` and the inverse-mass sanitiser, both mirroring the CPU to the letter:
/// a non-finite or negative weight reads as PINNED (`0`), never as an inverted push.
const WGSL_LIB: &str = r#"
const COLLIDE_EPS: f32 = 1e-9;

fn collide_w(x: f32) -> f32 {
    if (x == x && abs(x) < 3.4028235e38) {
        return max(x, 0.0);
    }
    return 0.0;
}
"#;

/// One averaged-Jacobi sweep, gathered. The sequencer runs this `iterations`
/// times (`GridSpec::sweeps_param`), rebuilding the grid between sweeps.
const WGSL: &str = r#"
    let pi = read_in_P(i);
    let radius = params.radius * read_spread_v(i);
    let min_dist = 2.0 * radius;
    // The CPU's early identity, to the letter: fewer than two discs, no radius,
    // or no strength ⇒ the input is returned untouched.
    if (min_dist <= 0.0 || params.strength <= 0.0 || params.count < 2u) {
        write_P(i, pi);
        return;
    }
    let min_d2 = min_dist * min_dist;
    let wi = collide_w(read_in_inv_mass(i));
    var delta = vec2<f32>(0.0, 0.0);
    var contacts = 0u;
    // Cells to sweep in each direction. The cell is `radius` and the contact
    // distance is `2·radius·spread`, so this is 2 at spread 1 — and it GROWS with
    // spread instead of quietly missing the contacts a fixed 3×3 would drop.
    let reach = max(1, i32(ceil(min_dist / max(params.radius, 1e-20))));
    let cell = params.grid_cell;
    let ci = grid_cell_of(pi);
    for (var dy = -reach; dy <= reach; dy = dy + 1) {
        for (var dx = -reach; dx <= reach; dx = dx + 1) {
            let c = ci + vec2<i32>(dx, dy);
            // `reach` is the WORST case — a disc pressed against the far edge of its
            // own cell. What THIS disc needs depends on where inside the cell it sits,
            // so skip a cell whose NEAREST point is already out of contact range,
            // before touching a single one of its discs. Exact, never approximate: a
            // contact needs `d2 < min_d2`, and every point of the cell is at least
            // this far, so nothing that could touch is ever dropped.
            let lo = vec2<f32>(f32(c.x), f32(c.y)) * cell;
            let gap = max(max(lo - pi, pi - (lo + vec2<f32>(cell, cell))), vec2<f32>(0.0, 0.0));
            if (dot(gap, gap) >= min_d2) { continue; }
            let b = grid_bucket_of(c);
            let hi = grid_starts[b + 1u];
            for (var s = grid_starts[b]; s < hi; s = s + 1u) {
                let j = grid_sorted[s];
                if (j == i) { continue; }
                let pj = read_in_P(j);
                // Exact-cell dedup: two cells can hash to one bucket, so count j
                // only while visiting the cell it actually lives in.
                let cj = grid_cell_of(pj);
                if (cj.x != c.x || cj.y != c.y) { continue; }
                let wj = collide_w(read_in_inv_mass(j));
                let sum_w = wi + wj;
                // Two immovable discs have no correction to share.
                if (sum_w <= 0.0) { continue; }
                let d = pj - pi;
                let d2 = dot(d, d);
                if (d2 >= min_d2) { continue; }
                var nrm = vec2<f32>(0.0, 0.0);
                var penetration = 0.0;
                if (d2 > COLLIDE_EPS) {
                    let len = sqrt(d2);
                    nrm = d / len;
                    penetration = min_dist - len;
                } else {
                    // Coincident. The CPU splits along x for even `i+j`, else y,
                    // with the normal pointing from the LOWER index to the higher
                    // — so a gathering thread flips it when it is the higher one.
                    var axis = vec2<f32>(0.0, 1.0);
                    if (((i + j) % 2u) == 0u) { axis = vec2<f32>(1.0, 0.0); }
                    if (i < j) { nrm = axis; } else { nrm = -axis; }
                    penetration = min_dist;
                }
                // What THIS disc is asked to move: its share of the penetration.
                delta = delta - nrm * (penetration * (wi / sum_w) * params.strength);
                contacts = contacts + 1u;
            }
        }
    }
    // The AVERAGE of what the contacts asked (mass splitting) — summing raw would
    // launch a crowded disc across the scene.
    if (contacts > 0u) {
        write_P(i, pi + delta / f32(contacts));
    } else {
        write_P(i, pi);
    }
"#;

/// The registered kernel. `iterations` is NOT a kernel param: the sequencer reads
/// it to decide how many times to run this pass (`GRID.sweeps_param`), so a copy
/// in the uniform would be a second answer to the same question.
pub const GPU_KERNEL: GpuKernel = GpuKernel {
    wgsl: WGSL,
    wgsl_lib: WGSL_LIB,
    bindings: BINDINGS,
    params: &["radius", "strength"],
    count_law: None,
    variant_by_param: None,
    applicable: None,
};

/// The neighbourhood grid: over `P` on port 0, cell = `radius`, swept `iterations`
/// times with the grid rebuilt from the moved positions each sweep.
pub const GRID: GridSpec = GridSpec {
    column: "P",
    port: 0,
    cell_param: "radius",
    sweeps_param: Some("iterations"),
};
