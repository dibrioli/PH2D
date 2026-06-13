// GPU dab splat — the resident-sim input pass (4K real-time architecture, §4 of
// HANDOFF_painter_fluid_gpu_composite). Replaces the per-frame FULL-GRID `deposit`
// upload (`cs_deposit`) with a tiny `array<Dab>` (the dabs the tool emitted this
// frame, O(dabs) ~ dozens), splatted DIRECTLY onto the GPU-resident water +
// pigment so the CPU never allocs/scans/uploads an O(grid) buffer per frame.
//
// ## Mirror of the CPU `ph2d_painter_brush::diffusion::DiffusionGrid::splat`
// The CPU splats dabs SEQUENTIALLY, each clamping water to 1.0 after it adds. Here
// each output CELL is one invocation that loops the dab list IN THE SAME ORDER,
// applying the same falloff + the same per-dab water clamp. For a fixed cell that
// reproduces the CPU's update sequence — same covered cells, same falloff, same
// order — so the SHAPE is exact; the only divergence is FMA contraction (Metal
// fuses `a*b+c`), worth ~1e-7, far below the ~1e-4 the diffuse/advect gather passes
// settle for and invisible after the composite's u8 quantization.
//
// **Pigment = PV (=8) `vec4<f32>` per cell = 32 channels (+ stain)** (ADR-0080/0081): the dab
// carries the per-cell pigment of its colour already weighted by the peak deposit
// mass (`cell_from_color_mass(colour, pigment_mass)` on the CPU side), so a cell
// adds `dab.pig[v] * fall` per vec4 — the mass-weighted K/S accumulation that mixes
// subtractively. Water is `f32`. Cell index = y*width + x; channels at `c*PV + v`.

const PV: u32 = 8u;

struct SplatParams {
    width: u32,
    height: u32,
    origin_x: u32,
    origin_y: u32,
    n_dabs: u32,
    _p0: u32,
    _p1: u32,
    _p2: u32,
}

// std430: cx,cy,r,water_add pack the first 16 B (vec4-aligned); `pig` is 8 vec4
// (128 B, the 32-channel pigment incl. stain) → 144 B stride, matching the
// `#[repr(C)] DabGpu` on the Rust side.
struct Dab {
    cx: f32,
    cy: f32,
    r: f32,
    water_add: f32,
    pig: array<vec4<f32>, 8>,
}

@group(0) @binding(0) var<uniform> S: SplatParams;
@group(0) @binding(1) var<storage, read_write> water: array<f32>;
@group(0) @binding(2) var<storage, read_write> pig: array<vec4<f32>>;
@group(0) @binding(3) var<storage, read> dabs: array<Dab>;
// Set timer / "gel" (ADR-0079-amendment-2, Bleed Limit) — fresh paint RE-MOBILIZES a set wash:
// painting over a gelled cell drops its set timer in proportion to the dab coverage, so the wick
// frees up again there and the artist can keep working / extending the wash (the rest stays bound).
@group(0) @binding(4) var<storage, read_write> gel: array<f32>;

fn pidx(cell: u32, v: u32) -> u32 { return v * (S.width * S.height) + cell; }

@compute @workgroup_size(8, 8, 1)
fn cs_splat(@builtin(global_invocation_id) gid: vec3<u32>) {
    let gx = S.origin_x + gid.x;
    let gy = S.origin_y + gid.y;
    if (gx >= S.width || gy >= S.height) {
        return;
    }
    let i = gy * S.width + gx;
    var w = water[i];
    var p: array<vec4<f32>, 8>;
    for (var v = 0u; v < PV; v = v + 1u) {
        p[v] = pig[pidx(i, v)];
    }
    let fx = f32(gx);
    let fy = f32(gy);
    var reset = 0.0; // strongest dab coverage here → how much to re-mobilize the set timer
    for (var d: u32 = 0u; d < S.n_dabs; d = d + 1u) {
        let db = dabs[d];
        let dx = fx - db.cx;
        let dy = fy - db.cy;
        let dist = sqrt(dx * dx + dy * dy) * (1.0 / db.r);
        if (dist >= 1.0) {
            continue;
        }
        let fall = 1.0 - dist * dist * (3.0 - 2.0 * dist); // 1 at centre → 0 at rim
        w = min(w + db.water_add * fall, 1.0);
        reset = max(reset, fall);
        for (var v = 0u; v < PV; v = v + 1u) {
            p[v] = p[v] + db.pig[v] * fall;
        }
    }
    water[i] = w;
    for (var v = 0u; v < PV; v = v + 1u) {
        pig[pidx(i, v)] = p[v];
    }
    // Re-mobilize: fresh paint frees the wick under the dab (gel ← gel·(1 − coverage)).
    gel[i] = gel[i] * (1.0 - reset);
}
