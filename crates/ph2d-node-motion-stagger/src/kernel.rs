//! The GPU kernels for `motion.stagger` — one per target column.
//!
//! The stagger's delta is a pure function of the element INDEX (`i / (n−1)`,
//! eased, remapped into `min..max`, masked by `falloff`), which is why it ports
//! to the device without state: every element's answer depends on its own index
//! and the dispatch width, both of which the kernel already has.
//!
//! **Every channel**, by shipping one variant per target column
//! ([`GpuKernel::variant_by_param`]). It used to cover X/Y only: Rotation writes
//! `rot` and Size writes `size`, and a static binding set cannot switch its
//! output column on a param, so those two channels fell back to the CPU. The
//! four channel values map exactly as the CPU's `channel_column` does —
//! including its `_ => size` catch-all for an out-of-range value.
//!
//! The delta is computed identically in all three variants (the shared
//! `SG_LIB`); only the column it lands on differs, which is precisely why the
//! variants exist and not a body-level branch. `SG_LIB` is one constant read
//! three times rather than three copies of the same text: an easing table that
//! must agree with itself in three places is a drift waiting to be found by a
//! parity gate that only fixtures one of them.
//!
//! `params.count` is the dispatch width — the same `n` the CPU divides by, with
//! the same `n <= 1` guard (dividing by `n−1` without it is a divide by zero).
//!
//! `sg_round` is round-half-AWAY-from-zero because `channel` / `ease_curve` /
//! `ease_dir` all pick BRANCHES: WGSL's builtin `round` is half-to-EVEN, and a
//! disagreement there would select a different curve, not shift a value by an ε.

use ph2d_nodegraph::gpu::{ColumnAccess, ColumnBinding, GpuKernel};
use ph2d_nodegraph::port::Dim;

const SG_PARAMS: &[&str] = &["channel", "min", "max", "ease_curve", "ease_dir", "reverse"];

/// The falloff mask, bound identically by every variant.
const SG_FALLOFF: ColumnBinding = ColumnBinding {
    column: "falloff",
    dim: Dim::Scalar,
    access: ColumnAccess::Read,
    identity: [1.0; 4],
    port: 0,
};

/// The whole easing table, ported verbatim from the CPU: 8 curve families × 3
/// directions, all polynomial or `sqrt` (HR-5 — no `pow`, no transcendental),
/// with In-Out built by reflecting the In base exactly as the CPU does.
///
/// Shared by all three variants — see the module note on why this is one
/// constant and not three copies.
const SG_LIB: &str = "\
    fn sg_round(x: f32) -> f32 {\n\
        return select(ceil(x - 0.5), floor(x + 0.5), x >= 0.0);\n\
    }\n\
    fn sg_bounce_out(t: f32) -> f32 {\n\
        let n1 = 7.5625;\n\
        let d1 = 2.75;\n\
        if (t < 1.0 / d1) { return n1 * t * t; }\n\
        if (t < 2.0 / d1) {\n\
            let u = t - 1.5 / d1;\n\
            return n1 * u * u + 0.75;\n\
        }\n\
        if (t < 2.5 / d1) {\n\
            let u = t - 2.25 / d1;\n\
            return n1 * u * u + 0.9375;\n\
        }\n\
        let u = t - 2.625 / d1;\n\
        return n1 * u * u + 0.984375;\n\
    }\n\
    fn sg_ease_in(curve: i32, t: f32) -> f32 {\n\
        if (curve == 1) { return t * t; }\n\
        if (curve == 2) { return t * t * t; }\n\
        if (curve == 3) { return t * t * t * t; }\n\
        if (curve == 4) { return t * t * t * t * t; }\n\
        if (curve == 5) { return 1.0 - sqrt(max(1.0 - t * t, 0.0)); }\n\
        if (curve == 6) {\n\
            let c1 = 1.70158;\n\
            let c3 = c1 + 1.0;\n\
            return c3 * t * t * t - c1 * t * t;\n\
        }\n\
        if (curve == 7) { return 1.0 - sg_bounce_out(1.0 - t); }\n\
        return t;\n\
    }\n\
    fn sg_ease(curve: i32, dir: i32, t: f32) -> f32 {\n\
        if (curve == 0) { return t; }\n\
        if (dir == 1) { return 1.0 - sg_ease_in(curve, 1.0 - t); }\n\
        if (dir == 2) {\n\
            if (t < 0.5) { return sg_ease_in(curve, 2.0 * t) * 0.5; }\n\
            return 1.0 - sg_ease_in(curve, 2.0 - 2.0 * t) * 0.5;\n\
        }\n\
        return sg_ease_in(curve, t);\n\
    }\n\
    fn sg_delta(i: u32) -> f32 {\n\
        var raw = 0.0;\n\
        if (params.count > 1u) {\n\
        \x20   raw = f32(i) / (f32(params.count) - 1.0);\n\
        }\n\
        if (params.reverse >= 0.5) { raw = 1.0 - raw; }\n\
        let e = sg_ease(i32(sg_round(params.ease_curve)),\n\
        \x20   i32(sg_round(params.ease_dir)), raw);\n\
        return (params.min + e * (params.max - params.min)) * read_falloff(i);\n\
    }\n";

/// **X / Y** — adds the delta to one component of `P`. The channel test is
/// `< 0.5`, which agrees with the CPU's `round()` for both values this variant
/// is selected for.
const SG_P: GpuKernel = GpuKernel {
    wgsl: "\
        let sg_d = sg_delta(i);\n\
        var sg_p = read_P(i);\n\
        if (params.channel < 0.5) {\n\
        \x20   sg_p.x = sg_p.x + sg_d;\n\
        } else {\n\
        \x20   sg_p.y = sg_p.y + sg_d;\n\
        }\n\
        write_P(i, sg_p);\n",
    wgsl_lib: SG_LIB,
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
        SG_FALLOFF,
    ],
    params: SG_PARAMS,
    count_law: None,
    variant_by_param: None,
    applicable: None,
};

/// **Rotation** — adds the delta to `rot` (identity `0`).
const SG_ROT: GpuKernel = GpuKernel {
    wgsl: "        write_rot(i, read_rot(i) + sg_delta(i));\n",
    wgsl_lib: SG_LIB,
    bindings: &[
        ColumnBinding {
            column: "rot",
            dim: Dim::Scalar,
            access: ColumnAccess::ReadWrite,
            identity: [0.0; 4],
            port: 0,
        },
        SG_FALLOFF,
    ],
    params: SG_PARAMS,
    count_law: None,
    variant_by_param: None,
    applicable: None,
};

/// **Size** — adds the delta to BOTH components, from the UNIT identity (never
/// `[0,0]`: unit scale is what "no size" means, and a bipolar stagger based at
/// zero would drive the sprite through zero and negative).
const SG_SIZE: GpuKernel = GpuKernel {
    wgsl: "\
        let sg_d = sg_delta(i);\n\
        let sg_s = read_size(i);\n\
        write_size(i, vec2<f32>(sg_s.x + sg_d, sg_s.y + sg_d));\n",
    wgsl_lib: SG_LIB,
    bindings: &[
        ColumnBinding {
            column: "size",
            dim: Dim::Vec2,
            access: ColumnAccess::ReadWrite,
            identity: [1.0, 1.0, 0.0, 0.0],
            port: 0,
        },
        SG_FALLOFF,
    ],
    params: SG_PARAMS,
    count_law: None,
    variant_by_param: None,
    applicable: None,
};

/// The registered kernel: the X/Y variant's shape, with the channel switch.
pub(crate) const GPU_KERNEL: GpuKernel = GpuKernel {
    // The top-level shape IS the X/Y variant, so a caller that never resolves
    // still sees a real kernel rather than the empty (pass-through) one.
    wgsl: SG_P.wgsl,
    wgsl_lib: SG_LIB,
    bindings: SG_P.bindings,
    params: SG_PARAMS,
    count_law: None,
    variant_by_param: Some(|param| match param("channel").round() as i32 {
        2 => &SG_ROT,
        0 | 1 => &SG_P,
        _ => &SG_SIZE,
    }),
    applicable: None,
};
