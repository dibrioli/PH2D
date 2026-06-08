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
// settle for and invisible after the composite's u8 quantization. A cell a dab
// doesn't cover (`dist >= 1`) is skipped, exactly as the CPU's `d >= 1.0` cutoff
// inside the per-dab integer bbox (every `dist < 1` cell lies inside that bbox, so
// the two acceptance sets are identical).
//
// Pigment is `vec4<f32>` (xyz = linear-RGB mass, w preserved) to match the solver's
// `pig_a` layout; water is `f32`. Index = y*width + x. One dispatch covers the
// UNION bbox of all dabs (origin = `S.origin`); cells outside every dab's reach
// loop the list and skip all of it (cheap — the list is tiny).

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

// std430: cx,cy,r,water_add pack the first 16 B; rgb (vec3, 16-aligned) + pad the
// next 16 B → 32 B stride, matching the `#[repr(C)] DabGpu` on the Rust side.
struct Dab {
    cx: f32,
    cy: f32,
    r: f32,
    water_add: f32,
    rgb: vec3<f32>,
    _pad: f32,
}

@group(0) @binding(0) var<uniform> S: SplatParams;
@group(0) @binding(1) var<storage, read_write> water: array<f32>;
@group(0) @binding(2) var<storage, read_write> pig: array<vec4<f32>>;
@group(0) @binding(3) var<storage, read> dabs: array<Dab>;

@compute @workgroup_size(8, 8, 1)
fn cs_splat(@builtin(global_invocation_id) gid: vec3<u32>) {
    let gx = S.origin_x + gid.x;
    let gy = S.origin_y + gid.y;
    if (gx >= S.width || gy >= S.height) {
        return;
    }
    let i = gy * S.width + gx;
    var w = water[i];
    var p = pig[i].xyz;
    let fx = f32(gx);
    let fy = f32(gy);
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
        p = p + db.rgb * fall;
    }
    water[i] = w;
    pig[i] = vec4<f32>(p, pig[i].w);
}
