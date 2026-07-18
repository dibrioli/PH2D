//! **The WGSL kernels** — one variant per target column, split from `lib.rs` at
//! the HR-18 LOC cap along the seam the variants created.
//!
//! `lib.rs` owns the node (manifest, `eval`, registration); this owns what the
//! GPU runs. The delta is computed once in the shared library and each variant
//! only decides which column it lands on, which is exactly why they are separate
//! kernels: the generated module defines `write_<col>` for BOUND columns only.

use ph2d_nodegraph::gpu::{ColumnAccess, ColumnBinding, GpuKernel};
use ph2d_nodegraph::port::Dim;

/// The params every variant declares, in one order — the uniform layout is
/// per-variant, and identical lists mean a reader never has to ask which variant
/// a `params.<x>` belongs to.
pub(crate) const NS_PARAMS: &[&str] = &[
    "channel",
    "amplitude",
    "scale",
    "speed",
    "seed",
    "octaves",
    "roughness",
    "type",
];

/// **X / Y** — adds the delta to one component of `P`. The channel test is
/// `< 0.5`, which agrees with the CPU's `round()` for both values this variant
/// is selected for.
const NS_P: GpuKernel = GpuKernel {
    wgsl: "\
        let d = ns_delta(i);\n\
        var p = read_P(i);\n\
        if (params.channel < 0.5) { p.x = p.x + d; } else { p.y = p.y + d; }\n\
        write_P(i, p);\n",
    wgsl_lib: "\
        fn ns_delta(i: u32) -> f32 {\n\
            let p = read_P(i);\n\
            let seed = i32(ns_round(params.seed));\n\
            let oct = min(max(i32(ns_round(params.octaves)), 1), 8);\n\
            let ty = i32(ns_round(params.type_));\n\
            let s = ns_fbm(p.x * params.scale,\n\
            \x20   p.y * params.scale + params.playhead * params.speed,\n\
            \x20   seed, oct, params.roughness, ty);\n\
            return s * params.amplitude * read_falloff(i);\n\
        }\n\
        const NS_NORM: f32 = 1.0 / 1.5;\n\
        const NS_LACUNARITY: f32 = 2.0;\n\
        fn ns_round(x: f32) -> f32 {\n\
            // Rust f32::round = half away from zero (WGSL round is half-even).\n\
            return select(ceil(x - 0.5), floor(x + 0.5), x >= 0.0);\n\
        }\n\
        fn ns_hash(ix: i32, iy: i32, seed: i32) -> u32 {\n\
            var h: u32 = bitcast<u32>(ix) * 0x27d4eb2du\n\
                + bitcast<u32>(iy) * 0x165667b1u\n\
                + bitcast<u32>(seed) * 0x01934f07u;\n\
            h = h ^ (h >> 15u);\n\
            h = h * 0x2c1b3c6du;\n\
            h = h ^ (h >> 12u);\n\
            h = h * 0x297175f9u;\n\
            h = h ^ (h >> 15u);\n\
            return h;\n\
        }\n\
        fn ns_dot_grad(h: u32, dx: f32, dy: f32) -> f32 {\n\
            // The eight 2002 gradients (+-1,+-2)/(+-2,+-1), as +-u +- 2v.\n\
            let g = h & 7u;\n\
            var u = dx;\n\
            var v = dy;\n\
            if (g >= 4u) { u = dy; v = dx; }\n\
            let a = select(u, -u, (g & 1u) != 0u);\n\
            let b = select(2.0 * v, -2.0 * v, (g & 2u) != 0u);\n\
            return a + b;\n\
        }\n\
        fn ns_fade(t: f32) -> f32 {\n\
            return t * t * t * (t * (t * 6.0 - 15.0) + 10.0);\n\
        }\n\
        fn ns_grad_noise(x: f32, y: f32, seed: i32) -> f32 {\n\
            let x0 = floor(x);\n\
            let y0 = floor(y);\n\
            let ix = i32(x0);\n\
            let iy = i32(y0);\n\
            let fx = x - x0;\n\
            let fy = y - y0;\n\
            let u = ns_fade(fx);\n\
            let v = ns_fade(fy);\n\
            let n00 = ns_dot_grad(ns_hash(ix, iy, seed), fx, fy);\n\
            let n10 = ns_dot_grad(ns_hash(ix + 1, iy, seed), fx - 1.0, fy);\n\
            let n01 = ns_dot_grad(ns_hash(ix, iy + 1, seed), fx, fy - 1.0);\n\
            let n11 = ns_dot_grad(ns_hash(ix + 1, iy + 1, seed), fx - 1.0, fy - 1.0);\n\
            let nx0 = n00 + u * (n10 - n00);\n\
            let nx1 = n01 + u * (n11 - n01);\n\
            return (nx0 + v * (nx1 - nx0)) * NS_NORM;\n\
        }\n\
        fn ns_fbm(x0: f32, y0: f32, seed: i32, octaves: i32, roughness: f32, ty: i32) -> f32 {\n\
            let gain = clamp(roughness, 0.0, 1.0);\n\
            var x = x0;\n\
            var y = y0;\n\
            var amp = 1.0;\n\
            var sum = 0.0;\n\
            var total = 0.0;\n\
            for (var o = 0; o < octaves; o = o + 1) {\n\
                // Per-octave seed offset: octaves must be independent fields,\n\
                // not scaled copies of one (which would beat visibly).\n\
                let n = ns_grad_noise(x, y, seed + o * 1013);\n\
                var shaped = n;\n\
                if (ty == 1) {\n\
                    shaped = abs(n);\n\
                } else if (ty == 2) {\n\
                    let r = 1.0 - abs(n);\n\
                    shaped = r * r;\n\
                }\n\
                sum = sum + amp * shaped;\n\
                total = total + amp;\n\
                amp = amp * gain;\n\
                x = x * NS_LACUNARITY;\n\
                y = y * NS_LACUNARITY;\n\
            }\n\
            return sum / total;\n\
        }\n",
    bindings: &[
        // The target channel is materialized from its identity when absent —
        // the CPU's `apply_channel_delta` does the same (`base_vec2`).
        ColumnBinding {
            column: "P",
            dim: Dim::Vec2,
            access: ColumnAccess::ReadWrite,
            identity: [0.0; 4],
            port: 0,
        },
        ColumnBinding {
            column: "falloff",
            dim: Dim::Scalar,
            access: ColumnAccess::Read,
            identity: [1.0; 4],
            port: 0,
        },
    ],
    params: NS_PARAMS,
    count_law: None,
    variant_by_param: None,
    applicable: None,
};

/// **Rotation** — adds the delta to `rot`.
const NS_ROT: GpuKernel = GpuKernel {
    wgsl: "\
        write_rot(i, read_rot(i) + ns_delta(i));\n",
    wgsl_lib: "\
        fn ns_delta(i: u32) -> f32 {\n\
            let p = read_P(i);\n\
            let seed = i32(ns_round(params.seed));\n\
            let oct = min(max(i32(ns_round(params.octaves)), 1), 8);\n\
            let ty = i32(ns_round(params.type_));\n\
            let s = ns_fbm(p.x * params.scale,\n\
            \x20   p.y * params.scale + params.playhead * params.speed,\n\
            \x20   seed, oct, params.roughness, ty);\n\
            return s * params.amplitude * read_falloff(i);\n\
        }\n\
        const NS_NORM: f32 = 1.0 / 1.5;\n\
        const NS_LACUNARITY: f32 = 2.0;\n\
        fn ns_round(x: f32) -> f32 {\n\
            // Rust f32::round = half away from zero (WGSL round is half-even).\n\
            return select(ceil(x - 0.5), floor(x + 0.5), x >= 0.0);\n\
        }\n\
        fn ns_hash(ix: i32, iy: i32, seed: i32) -> u32 {\n\
            var h: u32 = bitcast<u32>(ix) * 0x27d4eb2du\n\
                + bitcast<u32>(iy) * 0x165667b1u\n\
                + bitcast<u32>(seed) * 0x01934f07u;\n\
            h = h ^ (h >> 15u);\n\
            h = h * 0x2c1b3c6du;\n\
            h = h ^ (h >> 12u);\n\
            h = h * 0x297175f9u;\n\
            h = h ^ (h >> 15u);\n\
            return h;\n\
        }\n\
        fn ns_dot_grad(h: u32, dx: f32, dy: f32) -> f32 {\n\
            // The eight 2002 gradients (+-1,+-2)/(+-2,+-1), as +-u +- 2v.\n\
            let g = h & 7u;\n\
            var u = dx;\n\
            var v = dy;\n\
            if (g >= 4u) { u = dy; v = dx; }\n\
            let a = select(u, -u, (g & 1u) != 0u);\n\
            let b = select(2.0 * v, -2.0 * v, (g & 2u) != 0u);\n\
            return a + b;\n\
        }\n\
        fn ns_fade(t: f32) -> f32 {\n\
            return t * t * t * (t * (t * 6.0 - 15.0) + 10.0);\n\
        }\n\
        fn ns_grad_noise(x: f32, y: f32, seed: i32) -> f32 {\n\
            let x0 = floor(x);\n\
            let y0 = floor(y);\n\
            let ix = i32(x0);\n\
            let iy = i32(y0);\n\
            let fx = x - x0;\n\
            let fy = y - y0;\n\
            let u = ns_fade(fx);\n\
            let v = ns_fade(fy);\n\
            let n00 = ns_dot_grad(ns_hash(ix, iy, seed), fx, fy);\n\
            let n10 = ns_dot_grad(ns_hash(ix + 1, iy, seed), fx - 1.0, fy);\n\
            let n01 = ns_dot_grad(ns_hash(ix, iy + 1, seed), fx, fy - 1.0);\n\
            let n11 = ns_dot_grad(ns_hash(ix + 1, iy + 1, seed), fx - 1.0, fy - 1.0);\n\
            let nx0 = n00 + u * (n10 - n00);\n\
            let nx1 = n01 + u * (n11 - n01);\n\
            return (nx0 + v * (nx1 - nx0)) * NS_NORM;\n\
        }\n\
        fn ns_fbm(x0: f32, y0: f32, seed: i32, octaves: i32, roughness: f32, ty: i32) -> f32 {\n\
            let gain = clamp(roughness, 0.0, 1.0);\n\
            var x = x0;\n\
            var y = y0;\n\
            var amp = 1.0;\n\
            var sum = 0.0;\n\
            var total = 0.0;\n\
            for (var o = 0; o < octaves; o = o + 1) {\n\
                // Per-octave seed offset: octaves must be independent fields,\n\
                // not scaled copies of one (which would beat visibly).\n\
                let n = ns_grad_noise(x, y, seed + o * 1013);\n\
                var shaped = n;\n\
                if (ty == 1) {\n\
                    shaped = abs(n);\n\
                } else if (ty == 2) {\n\
                    let r = 1.0 - abs(n);\n\
                    shaped = r * r;\n\
                }\n\
                sum = sum + amp * shaped;\n\
                total = total + amp;\n\
                amp = amp * gain;\n\
                x = x * NS_LACUNARITY;\n\
                y = y * NS_LACUNARITY;\n\
            }\n\
            return sum / total;\n\
        }\n",
    bindings: &[
        ColumnBinding {
            column: "rot",
            dim: Dim::Scalar,
            access: ColumnAccess::ReadWrite,
            identity: [0.0; 4],
            port: 0,
        },
        ColumnBinding {
            column: "P",
            dim: Dim::Vec2,
            access: ColumnAccess::Read,
            identity: [0.0; 4],
            port: 0,
        },
        ColumnBinding {
            column: "falloff",
            dim: Dim::Scalar,
            access: ColumnAccess::Read,
            identity: [1.0; 4],
            port: 0,
        },
    ],
    params: NS_PARAMS,
    count_law: None,
    variant_by_param: None,
    applicable: None,
};

/// **Size** — adds the delta to BOTH components, from the UNIT identity (never
/// `[0,0]`: unit scale is what "no size" means).
const NS_SIZE: GpuKernel = GpuKernel {
    wgsl: "\
        let d = ns_delta(i);\n\
        let s = read_size(i);\n\
        write_size(i, vec2<f32>(s.x + d, s.y + d));\n",
    wgsl_lib: "\
        fn ns_delta(i: u32) -> f32 {\n\
            let p = read_P(i);\n\
            let seed = i32(ns_round(params.seed));\n\
            let oct = min(max(i32(ns_round(params.octaves)), 1), 8);\n\
            let ty = i32(ns_round(params.type_));\n\
            let s = ns_fbm(p.x * params.scale,\n\
            \x20   p.y * params.scale + params.playhead * params.speed,\n\
            \x20   seed, oct, params.roughness, ty);\n\
            return s * params.amplitude * read_falloff(i);\n\
        }\n\
        const NS_NORM: f32 = 1.0 / 1.5;\n\
        const NS_LACUNARITY: f32 = 2.0;\n\
        fn ns_round(x: f32) -> f32 {\n\
            // Rust f32::round = half away from zero (WGSL round is half-even).\n\
            return select(ceil(x - 0.5), floor(x + 0.5), x >= 0.0);\n\
        }\n\
        fn ns_hash(ix: i32, iy: i32, seed: i32) -> u32 {\n\
            var h: u32 = bitcast<u32>(ix) * 0x27d4eb2du\n\
                + bitcast<u32>(iy) * 0x165667b1u\n\
                + bitcast<u32>(seed) * 0x01934f07u;\n\
            h = h ^ (h >> 15u);\n\
            h = h * 0x2c1b3c6du;\n\
            h = h ^ (h >> 12u);\n\
            h = h * 0x297175f9u;\n\
            h = h ^ (h >> 15u);\n\
            return h;\n\
        }\n\
        fn ns_dot_grad(h: u32, dx: f32, dy: f32) -> f32 {\n\
            // The eight 2002 gradients (+-1,+-2)/(+-2,+-1), as +-u +- 2v.\n\
            let g = h & 7u;\n\
            var u = dx;\n\
            var v = dy;\n\
            if (g >= 4u) { u = dy; v = dx; }\n\
            let a = select(u, -u, (g & 1u) != 0u);\n\
            let b = select(2.0 * v, -2.0 * v, (g & 2u) != 0u);\n\
            return a + b;\n\
        }\n\
        fn ns_fade(t: f32) -> f32 {\n\
            return t * t * t * (t * (t * 6.0 - 15.0) + 10.0);\n\
        }\n\
        fn ns_grad_noise(x: f32, y: f32, seed: i32) -> f32 {\n\
            let x0 = floor(x);\n\
            let y0 = floor(y);\n\
            let ix = i32(x0);\n\
            let iy = i32(y0);\n\
            let fx = x - x0;\n\
            let fy = y - y0;\n\
            let u = ns_fade(fx);\n\
            let v = ns_fade(fy);\n\
            let n00 = ns_dot_grad(ns_hash(ix, iy, seed), fx, fy);\n\
            let n10 = ns_dot_grad(ns_hash(ix + 1, iy, seed), fx - 1.0, fy);\n\
            let n01 = ns_dot_grad(ns_hash(ix, iy + 1, seed), fx, fy - 1.0);\n\
            let n11 = ns_dot_grad(ns_hash(ix + 1, iy + 1, seed), fx - 1.0, fy - 1.0);\n\
            let nx0 = n00 + u * (n10 - n00);\n\
            let nx1 = n01 + u * (n11 - n01);\n\
            return (nx0 + v * (nx1 - nx0)) * NS_NORM;\n\
        }\n\
        fn ns_fbm(x0: f32, y0: f32, seed: i32, octaves: i32, roughness: f32, ty: i32) -> f32 {\n\
            let gain = clamp(roughness, 0.0, 1.0);\n\
            var x = x0;\n\
            var y = y0;\n\
            var amp = 1.0;\n\
            var sum = 0.0;\n\
            var total = 0.0;\n\
            for (var o = 0; o < octaves; o = o + 1) {\n\
                // Per-octave seed offset: octaves must be independent fields,\n\
                // not scaled copies of one (which would beat visibly).\n\
                let n = ns_grad_noise(x, y, seed + o * 1013);\n\
                var shaped = n;\n\
                if (ty == 1) {\n\
                    shaped = abs(n);\n\
                } else if (ty == 2) {\n\
                    let r = 1.0 - abs(n);\n\
                    shaped = r * r;\n\
                }\n\
                sum = sum + amp * shaped;\n\
                total = total + amp;\n\
                amp = amp * gain;\n\
                x = x * NS_LACUNARITY;\n\
                y = y * NS_LACUNARITY;\n\
            }\n\
            return sum / total;\n\
        }\n",
    bindings: &[
        ColumnBinding {
            column: "size",
            dim: Dim::Vec2,
            access: ColumnAccess::ReadWrite,
            identity: [1.0, 1.0, 0.0, 0.0],
            port: 0,
        },
        ColumnBinding {
            column: "P",
            dim: Dim::Vec2,
            access: ColumnAccess::Read,
            identity: [0.0; 4],
            port: 0,
        },
        ColumnBinding {
            column: "falloff",
            dim: Dim::Scalar,
            access: ColumnAccess::Read,
            identity: [1.0; 4],
            port: 0,
        },
    ],
    params: NS_PARAMS,
    count_law: None,
    variant_by_param: None,
    applicable: None,
};

/// GPU compute kernel (ADR-0126) — **every channel**, by shipping one variant per
/// target column ([`GpuKernel::variant_by_param`]).
///
/// It used to cover X/Y only: Rotation and Size write a DIFFERENT column, which a
/// static binding set cannot switch on, so those fell back to the CPU. The delta
/// is computed identically in all three variants (the shared `ns_delta`);
/// only the column it lands on differs, which is precisely why they are variants
/// and not a branch inside one body — the generated module defines `write_<col>`
/// only for BOUND columns.
///
/// The channels map exactly as `channel_column` does, including its `_ => size`
/// catch-all for an out-of-range value.
pub(crate) const GPU_KERNEL: GpuKernel = GpuKernel {
    // The top-level shape IS the X/Y variant, so a caller that never resolves
    // still sees a real kernel rather than the empty (pass-through) one.
    wgsl: NS_P.wgsl,
    wgsl_lib: NS_P.wgsl_lib,
    bindings: NS_P.bindings,
    params: NS_PARAMS,
    count_law: None,
    variant_by_param: Some(|param| match param("channel").round() as i32 {
        2 => &NS_ROT,
        0 | 1 => &NS_P,
        _ => &NS_SIZE,
    }),
    applicable: None,
};
