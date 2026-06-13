// GPU field reduction — max-water (the dry-check signal) + the wet bounding box,
// computed ON the GPU so the CPU never scans the O(grid) water mirror (4K real-time
// arch §4, replacing `DiffusionGrid::max_water` + `water_bbox`). One atomic pass;
// the 5-u32 result is read back SPORADICALLY (drying is slow — a few frames of
// latency on the dry-check is invisible, and the composite envelope is grown from
// the dab list, not from this).
//
// `water` is non-negative (∈ [0,1]), so its IEEE-754 bit pattern is monotonic and
// `atomicMax` over the bits yields the max water (dry cells contribute bits ≈ 0).
//
// stats layout (init by the driver before each dispatch):
//   [0] = max-water bits   (init 0)
//   [1] = min wet x        (init u32::MAX)   } only cells with water > threshold
//   [2] = min wet y        (init u32::MAX)   } vote on the bbox; if [1] stays
//   [3] = max wet x        (init 0)          } u32::MAX the field is bone-dry
//   [4] = max wet y        (init 0)          } (no wet cell) → bbox = None.

struct RParams {
    width: u32,
    height: u32,
    threshold: f32,
    _pad: u32,
}

@group(0) @binding(0) var<uniform> R: RParams;
@group(0) @binding(1) var<storage, read> water: array<f32>;
@group(0) @binding(2) var<storage, read_write> stats: array<atomic<u32>, 5>;

@compute @workgroup_size(8, 8, 1)
fn cs_reduce(@builtin(global_invocation_id) gid: vec3<u32>) {
    let x = gid.x;
    let y = gid.y;
    if (x >= R.width || y >= R.height) {
        return;
    }
    let w = water[y * R.width + x];
    atomicMax(&stats[0], bitcast<u32>(w));
    if (w > R.threshold) {
        atomicMin(&stats[1], x);
        atomicMin(&stats[2], y);
        atomicMax(&stats[3], x);
        atomicMax(&stats[4], y);
    }
}
