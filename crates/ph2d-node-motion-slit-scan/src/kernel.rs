//! **O KERNEL WGSL do `motion.slit_scan`** (ciclo 7, W1c — doc 112) — o `step` da CPU, no
//! dispositivo. Cortado do `lib.rs` por RESPONSABILIDADE: o `lib.rs` responde *o que o slit É*,
//! este responde *o que o dispositivo corre*.
//!
//! ## Porque ele existe
//!
//! Sem kernel, `grid → scale → slit_scan → output` levava a cadeia INTEIRA para a CPU e subia
//! `102 400` elementos por quadro (doc 112 §3).
//!
//! ## ⚠️⚠️ O anel do dispositivo NÃO é o da CPU — e é essa a decisão
//!
//! A CPU guarda a linha de atraso em **32 colunas `vec2`** (`ss_1`..`ss_32`). Portadas tal e qual,
//! o passe ligaria 32 leituras + 32 escritas + `P` e `falloff` = **67 ligações de armazenamento** —
//! e um Metal pára em 31. O dispositivo guarda o MESMO anel em **quatro `mat4x4`** por elemento
//! ([`ANEL`]: 64 números = 32 posições, a posição `k` nas faixas `2(k−1)` e `2(k−1)+1`): 11
//! ligações, a conta do `motion.integrate`.
//!
//! ⚠️ **Isto só é legítimo porque o estado nunca atravessa a costura.** O planeador RECUA um nó cujo
//! `pre` viria da CPU (*«este motor não tem costura que lho entregue»*, `plan.rs`), e a descida só
//! lê o sink: o anel de cada rota vive e morre na rota dele. Um slit-scan da CPU a montante de um
//! do dispositivo sobe as `ss_*` dele pela fronteira e elas atravessam como qualquer coluna — o
//! nó do dispositivo lê o SEU anel pela porta `state`.
//!
//! ⚠️ **Um anel AUSENTE é a pose VIVA, e não uma constante** — o `past` da CPU re-semeia um slot
//! que falta com as posições de agora (a cena forma-se nos próximos `lag` tiques em vez de saltar
//! para lixo). Uma identidade de ligação é uma constante, logo o corpo ramifica no `HAS_*`.

use super::{MANIFEST, lag_ticks};
use ph2d_node_registry::NodeRegistry;
use ph2d_nodegraph::gpu::{ColumnAccess, ColumnBinding, DerivedUniform, GpuKernel};
use ph2d_nodegraph::port::Dim;

/// As quatro colunas do anel no dispositivo (`Dim::Mat4`, 16 números cada).
pub(crate) const ANEL: [&str; 4] = ["ss_ring0", "ss_ring1", "ss_ring2", "ss_ring3"];

const CORPO: &str = "\
let ss_live = read_in_P(i);\n\
let ss_has = HAS_state_ss_ring0 && HAS_state_ss_ring1 && HAS_state_ss_ring2 && HAS_state_ss_ring3;\n\
let ss_old = array<mat4x4<f32>, 4>(read_state_ss_ring0(i), read_state_ss_ring1(i),\n\
\x20   read_state_ss_ring2(i), read_state_ss_ring3(i));\n\
// O atraso DESTE elemento — o `delay_of` da CPU. O `lag` do uniform ja' vem por `lag_ticks`.\n\
var ss_d = 0.0;\n\
let ss_f_raw = read_in_falloff(i);\n\
var ss_f = 0.0;\n\
if (abs(ss_f_raw) <= 3.4028235e38) { ss_f = clamp(ss_f_raw, 0.0, 1.0); }\n\
if (params.ramp >= 0.5) {\n\
\x20   // `Delay By = Field`: o campo sozinho (ciclo 7, W3).\n\
\x20   if (params.lag > 0.0) { ss_d = clamp(params.lag * ss_f, 0.0, 32.0); }\n\
} else if (params.count >= 2u && params.lag > 0.0) {\n\
\x20   let ss_rank = f32(i) / f32(params.count - 1u);\n\
\x20   ss_d = clamp(params.lag * ss_rank * ss_f, 0.0, 32.0);\n\
}\n\
let ss_lo = u32(ss_d);\n\
let ss_hi = min(ss_lo + 1u, 32u);\n\
let ss_frac = ss_d - f32(ss_lo);\n\
let ss_a = ss_at(ss_old, ss_has, ss_live, ss_lo);\n\
let ss_b = ss_at(ss_old, ss_has, ss_live, ss_hi);\n\
write_P(i, vec2<f32>(ss_a.x + (ss_b.x - ss_a.x) * ss_frac, ss_a.y + (ss_b.y - ss_a.y) * ss_frac));\n\
// O anel avanca: a pose viva vira `1 tique atras`, e cada posicao desce uma.\n\
var ss_new = array<mat4x4<f32>, 4>();\n\
for (var ss_k = 1u; ss_k <= 32u; ss_k = ss_k + 1u) {\n\
\x20   let ss_v = ss_at(ss_old, ss_has, ss_live, ss_k - 1u);\n\
\x20   let ss_l = 2u * (ss_k - 1u);\n\
\x20   let ss_e0 = ss_l % 16u;\n\
\x20   let ss_e1 = (ss_l + 1u) % 16u;\n\
\x20   ss_new[ss_l / 16u][ss_e0 / 4u][ss_e0 % 4u] = ss_v.x;\n\
\x20   ss_new[(ss_l + 1u) / 16u][ss_e1 / 4u][ss_e1 % 4u] = ss_v.y;\n\
}\n\
write_ss_ring0(i, ss_new[0]);\n\
write_ss_ring1(i, ss_new[1]);\n\
write_ss_ring2(i, ss_new[2]);\n\
write_ss_ring3(i, ss_new[3]);\n";

/// A faixa `l` do anel, e a posição `k` tiques atrás (`0` = a viva; um anel ausente = a viva).
const BIBLIOTECA: &str = "\
fn ss_lane(m: array<mat4x4<f32>, 4>, l: u32) -> f32 {\n\
\x20   let e = l % 16u;\n\
\x20   return m[l / 16u][e / 4u][e % 4u];\n\
}\n\
fn ss_at(m: array<mat4x4<f32>, 4>, has: bool, live: vec2<f32>, k: u32) -> vec2<f32> {\n\
\x20   if (k == 0u || !has) { return live; }\n\
\x20   let l = 2u * (k - 1u);\n\
\x20   return vec2<f32>(ss_lane(m, l), ss_lane(m, l + 1u));\n\
}\n";

const fn anel(k: usize) -> ColumnBinding {
    ColumnBinding {
        column: ANEL[k],
        dim: Dim::Mat4,
        access: ColumnAccess::ReadWrite,
        identity: [0.0; 4],
        port: 1,
    }
}

/// O kernel. ⚠️ `P` ausente é a origem (o `positions` da CPU) e `falloff` ausente é `1`.
pub(crate) const GPU_KERNEL: GpuKernel = GpuKernel {
    wgsl: CORPO,
    wgsl_lib: BIBLIOTECA,
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
        anel(0),
        anel(1),
        anel(2),
        anel(3),
    ],
    params: &["lag", super::RAMP],
    count_law: None,
    variant_by_param: None,
    applicable: None,
};

/// O `lag` do uniform é o `lag_ticks` da CPU — o clamp a `MAX_LAG` e o não-finito como `0`.
static DERIVADOS: &[DerivedUniform] = &[DerivedUniform {
    param: "lag",
    derive: |c| lag_ticks((c.param)("lag")),
}];

/// **Regista o caminho do dispositivo.**
pub(crate) fn regista(reg: &mut NodeRegistry) {
    reg.register_gpu_kernel(MANIFEST.id, GPU_KERNEL);
    reg.register_derived_uniforms(MANIFEST.id, DERIVADOS);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ring::MAX_LAG;

    /// ⚠️ **O `32` do WGSL é o `MAX_LAG` da CPU, e as quatro matrizes guardam-no inteiro.**
    #[test]
    fn the_device_ring_holds_the_cpus_delay_line() {
        assert_eq!(MAX_LAG, 32);
        assert!(CORPO.contains("clamp(params.lag * ss_rank * ss_f, 0.0, 32.0)"));
        assert!(CORPO.contains("min(ss_lo + 1u, 32u)"));
        assert!(CORPO.contains("ss_k <= 32u"));
        assert_eq!(ANEL.len() * 16, 2 * MAX_LAG, "64 faixas = 32 posições vec2");
    }

    /// **O derivado é o `lag_ticks`**, com o lixo de documento.
    #[test]
    fn the_derived_lag_is_the_cpus() {
        for v in [12.0, -1.0, 99.0, f32::NAN, f32::INFINITY, 3.4] {
            let param = |_: &str| v;
            let d = (DERIVADOS[0].derive)(&ph2d_nodegraph::gpu::CountLawCtx {
                inputs: &[],
                param: &param,
                playhead: 0.0,
                dt: 0.0,
            });
            assert_eq!(d.to_bits(), lag_ticks(v).to_bits(), "lag {v}");
        }
    }
}
