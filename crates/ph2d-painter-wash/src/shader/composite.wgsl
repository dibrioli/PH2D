// cs_composite — subtractive (Beer–Lambert) glaze of the pigment field over a backdrop.
// v1: grid-resolution, white backdrop (the canvas-res bicubic sample + real backdrop is a
// later integration step). out_rgb = backdrop · exp(−absorb); alpha = 1 − exp(−k·mass).
//
// The whole "look" of subtractive watercolour lives in these three lines — swapping in
// Kubelka–Munk / Mixbox later is a composite-only change (the transport never moves).

struct CParams {
    width: u32,
    height: u32,
    coverage_k: f32, // mass → alpha rate
    _pad: f32,
}

@group(0) @binding(0) var<uniform> C: CParams;
@group(0) @binding(1) var<storage, read> pig: array<vec4<f32>>;
@group(0) @binding(2) var<storage, read_write> out_buf: array<u32>; // packed RGBA8

fn pack(rgba: vec4<f32>) -> u32 {
    let c = clamp(rgba, vec4<f32>(0.0), vec4<f32>(1.0)) * 255.0 + vec4<f32>(0.5);
    return (u32(c.x)) | (u32(c.y) << 8u) | (u32(c.z) << 16u) | (u32(c.w) << 24u);
}

@compute @workgroup_size(8, 8, 1)
fn cs_composite(@builtin(global_invocation_id) gid: vec3<u32>) {
    let x = gid.x;
    let y = gid.y;
    if (x >= C.width || y >= C.height) {
        return;
    }
    let i = y * C.width + x;
    let cell = pig[i];
    let absorb = max(cell.xyz, vec3<f32>(0.0));
    let backdrop = vec3<f32>(1.0, 1.0, 1.0);        // white paper (v1)
    let rgb = backdrop * exp(-absorb);              // subtractive glaze
    let alpha = 1.0 - exp(-C.coverage_k * max(cell.w, 0.0));
    out_buf[i] = pack(vec4<f32>(rgb, alpha));
}
