//! The GPU **lowering** pass — the compute twin of
//! `ph2d_eval_motion::lower_to_instances_onto`: gathers the stream convention
//! columns (`P` / `size` / `rot` / `tint` / `uv_rect`, absent → the same
//! defaults the CPU applies) straight into a buffer laid out as
//! [`ph2d_render::RenderInstance`], which the sprite renderer binds as its
//! instance vertex buffer. This is what makes the path readback-free: the
//! cook's last write IS the renderer's input.
//!
//! The instance is written word-by-word into an `array<u32>` (with
//! `bitcast` for the float fields) because the WGSL alignment rules cannot
//! mirror the `#[repr(C)]` struct — `anchor: vec2<f32>` sits at byte offset
//! 68, and WGSL requires vec2 alignment 8. The word layout below is pinned
//! against `size_of::<RenderInstance>()` by a unit test AND by the render
//! crate's own `render_instance_pod_size_v4` gate.

use crate::codegen::WORKGROUP_SIZE;

/// `RenderInstance` is 184 bytes = 46 32-bit words.
pub const INSTANCE_WORDS: u32 = 46;

/// The five stream columns the lowering reads, in binding order. Presence of
/// each (bit `i` of the pipeline-cache signature) selects between a storage
/// binding and the default.
pub const LOWER_COLUMNS: [&str; 5] = ["P", "size", "rot", "tint", "uv_rect"];

/// Generate the lowering module for a concrete column set. Binding 0 = the
/// uniforms, binding 1 = the instance output; then one `read` binding per
/// present column, in [`LOWER_COLUMNS`] order.
pub fn lower_module(present: [bool; 5]) -> String {
    let mut src = String::with_capacity(2048);
    src.push_str(
        "struct LowerParams {\n\
         \x20   count: u32,\n\
         \x20   _pad0: u32,\n\
         \x20   default_size: vec2<f32>,\n\
         \x20   default_uv: vec4<f32>,\n\
         }\n\
         @group(0) @binding(0) var<uniform> params: LowerParams;\n\
         @group(0) @binding(1) var<storage, read_write> instances: array<u32>;\n",
    );
    let tys = ["vec2<f32>", "vec2<f32>", "f32", "vec4<f32>", "vec4<f32>"];
    let mut slot = 2u32;
    for (i, col) in LOWER_COLUMNS.iter().enumerate() {
        if present[i] {
            src.push_str(&format!(
                "@group(0) @binding({slot}) var<storage, read> in_{col}: array<{}>;\n",
                tys[i]
            ));
            slot += 1;
        }
    }
    src.push('\n');
    // Reader per column: buffer when present, else the SAME default the CPU
    // lowering applies (`scalar_at`/`vec2_at`/`vec4_at` fallbacks).
    let defaults = [
        "vec2<f32>(0.0, 0.0)",           // P
        "params.default_size",           // size (caller-supplied, like the CPU)
        "0.0",                           // rot
        "vec4<f32>(1.0, 1.0, 1.0, 1.0)", // tint
        "params.default_uv",             // uv_rect (caller-supplied)
    ];
    for (i, col) in LOWER_COLUMNS.iter().enumerate() {
        if present[i] {
            src.push_str(&format!(
                "fn read_{col}(i: u32) -> {ty} {{ return in_{col}[i]; }}\n",
                ty = tys[i]
            ));
        } else {
            src.push_str(&format!(
                "fn read_{col}(i: u32) -> {ty} {{ _ = i; return {d}; }}\n",
                ty = tys[i],
                d = defaults[i]
            ));
        }
    }
    src.push_str(&format!(
        "\n\
        fn wf(w: u32, v: f32) {{ instances[w] = bitcast<u32>(v); }}\n\
        \n\
        @compute @workgroup_size({WORKGROUP_SIZE})\n\
        fn main(@builtin(global_invocation_id) gid: vec3<u32>) {{\n\
        \x20   let i = gid.x;\n\
        \x20   if (i >= params.count) {{ return; }}\n\
        \x20   let base = i * {INSTANCE_WORDS}u;\n\
        \x20   // world_pos (words 0-1) · size (2-3) · atlas_uv (4-7) · tint (8-11)\n\
        \x20   let p = read_P(i);\n\
        \x20   wf(base + 0u, p.x);\n\
        \x20   wf(base + 1u, p.y);\n\
        \x20   let s = read_size(i);\n\
        \x20   wf(base + 2u, s.x);\n\
        \x20   wf(base + 3u, s.y);\n\
        \x20   let uv = read_uv_rect(i);\n\
        \x20   wf(base + 4u, uv.x);\n\
        \x20   wf(base + 5u, uv.y);\n\
        \x20   wf(base + 6u, uv.z);\n\
        \x20   wf(base + 7u, uv.w);\n\
        \x20   let t = read_tint(i);\n\
        \x20   wf(base + 8u, t.x);\n\
        \x20   wf(base + 9u, t.y);\n\
        \x20   wf(base + 10u, t.z);\n\
        \x20   wf(base + 11u, t.w);\n\
        \x20   // basis (12-15): `rot` is authored in DEGREES (the app's angle\n\
        \x20   // unit); the conversion lives only here, exactly like the CPU\n\
        \x20   // lowering. RenderInstance is PresentWorld-only (HR-5 exempt),\n\
        \x20   // so hardware sin/cos is fine — parity vs the CPU's sin_cos is\n\
        \x20   // held by the ε gate, not bit-for-bit.\n\
        \x20   let rad = read_rot(i) * 0.017453292519943295;\n\
        \x20   let sn = sin(rad);\n\
        \x20   let cs = cos(rad);\n\
        \x20   wf(base + 12u, cs);\n\
        \x20   wf(base + 13u, sn);\n\
        \x20   wf(base + 14u, -sn);\n\
        \x20   wf(base + 15u, cs);\n\
        \x20   // premultiplied (16) + anchor (17-18): zero.\n\
        \x20   instances[base + 16u] = 0u;\n\
        \x20   instances[base + 17u] = 0u;\n\
        \x20   instances[base + 18u] = 0u;\n\
        \x20   // per_corner_tint (19-34): identity white.\n\
        \x20   for (var k = base + 19u; k < base + 35u; k = k + 1u) {{\n\
        \x20       wf(k, 1.0);\n\
        \x20   }}\n\
        \x20   // opacity (35) = 1 · flip_uv (36) = 0 · uv_xform (37-40) = identity.\n\
        \x20   wf(base + 35u, 1.0);\n\
        \x20   instances[base + 36u] = 0u;\n\
        \x20   wf(base + 37u, 1.0);\n\
        \x20   wf(base + 38u, 1.0);\n\
        \x20   instances[base + 39u] = 0u;\n\
        \x20   instances[base + 40u] = 0u;\n\
        \x20   // texture_id · z_order · sampling · clip_group · clip_meta (41-45):\n\
        \x20   // atlas / z 0 / default sampler / no clip — the CPU lowering's values.\n\
        \x20   instances[base + 41u] = 0u;\n\
        \x20   instances[base + 42u] = 0u;\n\
        \x20   instances[base + 43u] = 0u;\n\
        \x20   instances[base + 44u] = 0u;\n\
        \x20   instances[base + 45u] = 0u;\n\
        }}\n"
    ));
    src
}

/// Pipeline-cache signature for a lowering column set (bit per column).
pub fn lower_signature(present: [bool; 5]) -> u64 {
    present
        .iter()
        .enumerate()
        .fold(0u64, |sig, (i, &p)| sig | ((p as u64) << i))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn instance_words_matches_render_instance_size() {
        // 46 words × 4 = size_of::<RenderInstance>() — if the render ABI grows,
        // this trips before the GPU writes garbage into the tail.
        assert_eq!(
            (INSTANCE_WORDS as usize) * 4,
            std::mem::size_of::<ph2d_render::RenderInstance>()
        );
    }

    #[test]
    fn absent_columns_read_the_cpu_defaults() {
        let src = lower_module([false; 5]);
        assert!(src.contains("return vec2<f32>(0.0, 0.0);")); // P
        assert!(src.contains("return params.default_size;"));
        assert!(src.contains("return params.default_uv;"));
        assert!(src.contains("return vec4<f32>(1.0, 1.0, 1.0, 1.0);")); // tint
        assert!(!src.contains("var<storage, read> in_"));
    }
}
