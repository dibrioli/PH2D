//! **O KERNEL WGSL do `motion.spherize` e as reduções dele** — cortado do `lib.rs` no teto de
//! LOC do HR-18 (700 para `crates/`), pela mesma costura que o `motion.drive`, o `motion.noise`
//! e o `motion.kaleidoscope` já usam.
//!
//! O corte é por RESPONSABILIDADE: o `lib.rs` responde *o que uma lente É* (o manifesto, o
//! `eval`, o registo) e este responde *o que o dispositivo corre e o que ele mede antes*.

use super::VALUE_COL;
use ph2d_nodegraph::gpu::{ColumnAccess, ColumnBinding, GpuKernel, ReduceOp, ReduceSpec};
use ph2d_nodegraph::port::Dim;

/// The whole-stream reductions this deformer needs: the layout **centroid**, as
/// two `Sum`s over `P.x` and `P.y` (GPU/M5, the deformer channel —
/// `ph2d_nodegraph::reduce_meta`). The kernel divides each by `params.count` to
/// recover the mean — the same `c = Σp / n` at the top of [`spherize`].
///
/// ⚠️ **This is the first node with MORE THAN ONE reduction**, which is why they
/// carry names (`cx`/`cy`): the kernel reads `reduce_cx()` / `reduce_cy()`, and a
/// single unnamed `reduce_result()` could not have served both. The channel was
/// declared plural for exactly this node.
///
/// ⚠️ **An ε, and a wider one than the `Max` deformers** — float addition is not
/// associative, so a tree sum and a sequential sum differ, and the partial sums
/// here reach the magnitude of the whole layout (`Σx` over 490k coordinates)
/// before dividing back down by `n`. The `bend`/`twist` folds were bit-exact
/// because `Max` is exact in any order; a `Sum` never is. The bound is measured
/// on the device, not borrowed (see the parity gate).
///
/// ⚠️⚠️ **E «nos últimos ulps», que era o que esta nota dizia, estava errado por
/// quatro ordens de grandeza** (ciclo 3, W1 — doc 106 §4): medido a 409 600
/// elementos com o layout a `4` da origem, a divergência era **54 365 % da barra
/// de `2e-4`** — e **535 % já no tamanho do gate** (16 384), que passava só porque
/// a lente dele tem raio `6`. O erro era da CPU, não do kernel: hoje a média vem
/// de [`ph2d_nodegraph::reduce_meta::centroid_of`], que acumula em `f64`, e a
/// pior célula da varredura lê **30,5 %**.
pub(crate) static REDUCES: &[ReduceSpec] = &[
    ReduceSpec {
        name: "cx",
        column: "P",
        dim: Dim::Vec2,
        port: 0,
        op: ReduceOp::Sum,
        value: "v.x",
        params: &[],
        // Absent `P` → origin on both paths (`vec![[0,0]; n]`); Σ 0 = 0, so the
        // centroid is the origin, which is what the CPU computes.
        identity: [0.0; 4],
    },
    ReduceSpec {
        name: "cy",
        column: "P",
        dim: Dim::Vec2,
        port: 0,
        op: ReduceOp::Sum,
        value: "v.y",
        params: &[],
        identity: [0.0; 4],
    },
];

/// The device form of [`spherize`] (GPU/M5). One invocation per element, reading
/// the centroid from the two `Sum` reductions above and dividing by the count.
///
/// ⚠️ **`amount` is read at index 0, ALWAYS** — `read_amount_v(0u)`, not
/// `read_amount_v(i)`. Unlike `bend`/`twist`, the CPU node reads `vals.first()`
/// and broadcasts it: `amount` is one number for the whole lens even when a
/// per-element field is wired. `ReadBroadcast` makes a length-1 input *present*
/// (so a bare `value.lfo` reads its value rather than the identity), and reading
/// index 0 makes a length-N input behave like the CPU's `first()`.
pub(crate) const GPU_KERNEL: GpuKernel = GpuKernel {
    wgsl: "\
        let sp_c = vec2<f32>(reduce_cx(), reduce_cy()) / f32(params.count)\n\
        \x20   + vec2<f32>(params.offset_x, params.offset_y);\n\
        let sp_p = read_in_P(i);\n\
        let sp_d = sp_p - sp_c;\n\
        let sp_r = sqrt(sp_d.x * sp_d.x + sp_d.y * sp_d.y);\n\
        let sp_rmax = max(params.radius, 1e-6);\n\
        // O raio na metrica da lente; circulo => o caminho literal da CPU.\n\
        var sp_t = sp_r / sp_rmax;\n\
        if (params.radius_y > 0.0) {\n\
        \x20   let sp_q = vec2<f32>(sp_d.x / sp_rmax, sp_d.y / params.radius_y);\n\
        \x20   sp_t = sqrt(sp_q.x * sp_q.x + sp_q.y * sp_q.y);\n\
        }\n\
        var sp_warped = sp_p;\n\
        if (sp_r >= 1e-6 && sp_t < 1.0) {\n\
        \x20   let sp_scale = 1.0 + read_amount_v(0u) * (1.0 - sp_t * sp_t);\n\
        \x20   sp_warped = vec2<f32>(sp_c.x + sp_d.x * sp_scale, sp_c.y + sp_d.y * sp_scale);\n\
        }\n\
        let sp_f = clamp(read_in_falloff(i), 0.0, 1.0);\n\
        write_P(i, vec2<f32>(\n\
        \x20   sp_p.x + (sp_warped.x - sp_p.x) * sp_f,\n\
        \x20   sp_p.y + (sp_warped.y - sp_p.y) * sp_f));\n",
    wgsl_lib: "",
    bindings: &[
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
            identity: [1.0, 0.0, 0.0, 0.0],
            port: 0,
        },
        ColumnBinding {
            column: VALUE_COL,
            dim: Dim::Scalar,
            // Unconnected reads 0.5 (a gentle bulge, the CPU's `unwrap_or(0.5)`).
            access: ColumnAccess::ReadBroadcast,
            identity: [0.5, 0.0, 0.0, 0.0],
            port: 1,
        },
    ],
    params: &["radius", "radius_y", "offset_x", "offset_y"],
    count_law: None,
    variant_by_param: None,
    applicable: None,
};
