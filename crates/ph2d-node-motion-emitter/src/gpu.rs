//! `gpu.rs` — the emitter's **WGSL mirror**, split from `lib.rs` at the HR-18 LOC cap
//! along the seam the module already had: its parent holds the LAW (`emit`, `window`,
//! `birth_offset`, `radial_axis`) and this holds the hand-written restatement of that law
//! for the device.
//!
//! ⚠️ The cut is deliberate and it is not a size hack: a mirror is a second answer to a
//! question its parent already answers, and it is only load-bearing while a FIXTURE drives
//! both paths. Keeping it in a file of its own is what makes "does anything read this?" a
//! question with an address — the parity gate is `gpu_cpu_parity::the_emitter_generator_
//! matches_the_cpu`, whose fixture varies the shape, the direction AND the size for exactly
//! that reason.

use super::{MAX_ALIVE, window};
use ph2d_nodegraph::gpu::{ColumnAccess, ColumnBinding, GpuKernel};
use ph2d_nodegraph::port::Dim;

/// The GPU kernel (ADR-0126/0130): **side metadata**, not a lowering — the manifest is
/// untouched, `eval`/`emit` stays canonical. The emitter is a **generator**, so the kernel is
/// all-Write (like `motion.grid`), and its dispatch size is playhead-dependent (`source_count`
/// takes the playhead, ADR-0130): `n(t)` is the alive-window length.
///
/// **The kernel derives `first` from `params.count`, it does NOT recompute the cap.** The CPU
/// (`emit`) computes `n = min(span, max)` then `first = newest + 1 − n`; the cook runs that
/// same `source_count` and writes `n` into `params.count`, so the kernel takes `first =
/// u32(floor(t·rate)) + 1 − count` — one `floor`, no `ceil`/`span`/cap to diverge on. Parity is
/// by construction: `params.playhead` is `clock.playhead as f32` (`lib.rs`, the uniform pack),
/// which is the exact `f32` the CPU `emit` reads (`ctx.playhead() as f32`), and `source_count`
/// truncates the same way — so `newest`, `n` and `first` are computed from one shared `f32`
/// ([[feedback_test_with_product_numbers_not_convenient_ones]] — the number that must match is
/// the one the product uses).
///
/// The hash (`em_hash3`) is the integer avalanche of [`hash`], **bit-exact** in WGSL (u32
/// wraps mod 2³² like `wrapping_mul`); the wave is the sibling `trig.rs`, byte-identical to the
/// force kernels' corrected parabolic sine (HR-5). `max` is not a kernel param — `count`
/// already carries the cap.
pub(crate) const GPU_KERNEL: GpuKernel = GpuKernel {
    wgsl: "\
        let em_id = (params.window_first + i) % 16777216u;\n\
        let em_seed = u32(max(params.seed, 0.0));\n\
        let em_jitter = em_rand01(em_seed, em_id, 0u) - 0.5;\n\
        let em_deg = params.angle + em_jitter * params.spread;\n\
        var em_cs = em_cos_sin_cycles(em_deg / 360.0);\n\
        let em_ax = em_axis(em_seed, em_id, params.shape_mode, params.shape_w,\n\
            params.shape_h, params.dir_mode);\n\
        if (dot(em_ax, em_ax) > 0.0) {\n\
            let em_rot = em_cos_sin_cycles(em_jitter * params.spread / 360.0);\n\
            em_cs = vec2<f32>(em_ax.x * em_rot.x - em_ax.y * em_rot.y,\n\
                em_ax.x * em_rot.y + em_ax.y * em_rot.x);\n\
        }\n\
        let em_sj = em_rand01(em_seed, em_id, 1u) - 0.5;\n\
        let em_spd = params.speed * (1.0 + em_sj * params.speed_random * 2.0);\n\
        write_P(i, em_birth(em_seed, em_id, params.shape_mode, params.shape_w,\n\
            params.shape_h, vec2<f32>(params.x, params.y)));\n\
        write_vel(i, vec2<f32>(em_cs.x * em_spd, em_cs.y * em_spd));\n\
        write_id(i, f32(em_id));\n\
        var em_step = f32(i) / params.rate;\n\
        if (i32(round(params.emit_mode)) == 1) {\n\
            let em_n = max(round(params.burst_count), 1.0);\n\
            let em_f = f32(params.window_first);\n\
            em_step = (floor((em_f + f32(i)) / em_n) - floor(em_f / em_n))\n\
                * params.burst_period;\n\
        }\n\
        write_age(i, params.window_age - em_step);\n\
        write_life(i, params.life);\n\
        write_Index(i, f32(i));\n\
        write_Count(i, f32(params.count));\n\
        let em_zj = em_rand01(em_seed, em_id, 4u) - 0.5;\n\
        let em_sz = max(params.size * (1.0 + em_zj * params.size_random * 2.0), 0.0);\n\
        write_size(i, vec2<f32>(em_sz, em_sz));\n",
    wgsl_lib: "\
        fn em_hash3(a: u32, b: u32, lane: u32) -> f32 {\n\
            var h: u32 = a * 0x9e3779b9u + b * 0x85ebca6bu + lane * 0xc2b2ae35u;\n\
            h = h ^ (h >> 16u);\n\
            h = h * 0x7feb352du;\n\
            h = h ^ (h >> 15u);\n\
            h = h * 0x846ca68bu;\n\
            h = h ^ (h >> 16u);\n\
            return f32(h >> 8u) / f32(16777216u);\n\
        }\n\
        fn em_rand01(seed: u32, id: u32, lane: u32) -> f32 {\n\
            return em_hash3(seed, id, lane);\n\
        }\n\
        fn em_sin_cycles(phase: f32) -> f32 {\n\
            let f = phase - floor(phase);\n\
            var p: f32;\n\
            if (f < 0.5) {\n\
                let u = f * 2.0;\n\
                p = 4.0 * u * (1.0 - u);\n\
            } else {\n\
                let u = (f - 0.5) * 2.0;\n\
                p = -4.0 * u * (1.0 - u);\n\
            }\n\
            return 0.225 * (p * abs(p) - p) + p;\n\
        }\n\
        fn em_cos_sin_cycles(phase: f32) -> vec2<f32> {\n\
            return vec2<f32>(em_sin_cycles(phase + 0.25), em_sin_cycles(phase));\n\
        }\n\
        fn em_axis(seed: u32, id: u32, shape: f32, w: f32, h: f32, dir: f32) -> vec2<f32> {\n\
            let d = i32(round(dir));\n\
            if (d < 1 || d > 2) { return vec2<f32>(0.0, 0.0); }\n\
            let off = em_birth(seed, id, shape, w, h, vec2<f32>(0.0, 0.0));\n\
            let l2 = dot(off, off);\n\
            if (l2 <= 0.0) { return vec2<f32>(0.0, 0.0); }\n\
            var sgn = 1.0;\n\
            if (d == 2) { sgn = -1.0; }\n\
            return off * (sgn / sqrt(l2));\n\
        }\n\
        fn em_birth(seed: u32, id: u32, mode: f32, w: f32, h: f32, o: vec2<f32>) -> vec2<f32> {\n\
            let m = i32(round(mode));\n\
            if (m < 1 || m > 3) { return o; }\n\
            let u = em_rand01(seed, id, 2u);\n\
            let v = em_rand01(seed, id, 3u);\n\
            if (m == 3) { return o + vec2<f32>((u - 0.5) * 2.0 * w, (v - 0.5) * 2.0 * h); }\n\
            if (m == 2) {\n\
                let cs = em_cos_sin_cycles(u);\n\
                return o + vec2<f32>(cs.x * w, cs.y * h);\n\
            }\n\
            let r = sqrt(u);\n\
            let cs = em_cos_sin_cycles(v);\n\
            return o + vec2<f32>(cs.x * w * r, cs.y * h * r);\n\
        }\n",
    bindings: &[
        ColumnBinding {
            column: "P",
            dim: Dim::Vec2,
            access: ColumnAccess::Write,
            identity: [0.0; 4],
            port: 0,
        },
        ColumnBinding {
            column: "vel",
            dim: Dim::Vec2,
            access: ColumnAccess::Write,
            identity: [0.0; 4],
            port: 0,
        },
        ColumnBinding {
            column: "id",
            dim: Dim::Scalar,
            access: ColumnAccess::Write,
            identity: [0.0; 4],
            port: 0,
        },
        ColumnBinding {
            column: "age",
            dim: Dim::Scalar,
            access: ColumnAccess::Write,
            identity: [0.0; 4],
            port: 0,
        },
        ColumnBinding {
            column: "life",
            dim: Dim::Scalar,
            access: ColumnAccess::Write,
            identity: [0.0; 4],
            port: 0,
        },
        ColumnBinding {
            column: "Index",
            dim: Dim::Scalar,
            access: ColumnAccess::Write,
            identity: [0.0; 4],
            port: 0,
        },
        ColumnBinding {
            column: "Count",
            dim: Dim::Scalar,
            access: ColumnAccess::Write,
            identity: [0.0; 4],
            port: 0,
        },
        ColumnBinding {
            column: "size",
            dim: Dim::Vec2,
            access: ColumnAccess::Write,
            identity: [0.0; 4],
            port: 0,
        },
    ],
    params: &[
        "rate",
        "life",
        "speed",
        "speed_random",
        "angle",
        "spread",
        "x",
        "y",
        "shape_mode",
        "shape_w",
        "shape_h",
        "dir_mode",
        "emit_mode",
        "burst_count",
        "burst_period",
        "seed",
        "size",
        "size_random",
    ],
    // Playhead-dependent (ADR-0130): the alive-window length `n(t)`, mirroring `emit`'s count.
    count_law: Some(|c| {
        window(
            super::Spawn::from_params(
                (c.param)("emit_mode"),
                (c.param)("rate"),
                (c.param)("burst_count"),
                (c.param)("burst_time"),
                (c.param)("burst_period"),
            ),
            (c.param)("life"),
            ((c.param)("max").max(0.0) as usize).min(MAX_ALIVE),
            c.playhead as f32,
        )
    }),
    variant_by_param: None,
    // ⚠️ **O device é RECUSADO quando a probabilidade morde**, e a razão é a `count_law`
    // logo acima: ela dá o tamanho do buffer a partir de aritmética (janela × cap), e um
    // portão por hash torna a contagem dependente de DADOS. Mapear a invocação `i` para o
    // i-ésimo sobrevivente exigiria um prefix-sum — o `motion.cull` tem a máquina para isso
    // (`KEEP_FLAG_COL` + `StreamOp` de compactação), e ligá-la a um GERADOR com
    // `count_law` é uma wave própria, não uma linha. Desligado (o default) nada recua, e é a
    // mesma porta que o `reindex` do `motion.combine` usa.
    applicable: Some(|p| p("probability") >= 1.0),
};
