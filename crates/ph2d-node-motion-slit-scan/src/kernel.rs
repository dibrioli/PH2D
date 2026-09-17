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
// As 16 COLUNAS do anel (4 faixas cada), lidas UMA vez. Um anel ausente e' a pose viva.\n\
var ss_c = array<vec4<f32>, 16>();\n\
if (ss_has) {\n\
\x20   let ss_m = array<mat4x4<f32>, 4>(read_state_ss_ring0(i), read_state_ss_ring1(i),\n\
\x20       read_state_ss_ring2(i), read_state_ss_ring3(i));\n\
\x20   for (var ss_j = 0u; ss_j < 16u; ss_j = ss_j + 1u) { ss_c[ss_j] = ss_m[ss_j / 4u][ss_j % 4u]; }\n\
} else {\n\
\x20   let ss_v = vec4<f32>(ss_live.x, ss_live.y, ss_live.x, ss_live.y);\n\
\x20   for (var ss_j = 0u; ss_j < 16u; ss_j = ss_j + 1u) { ss_c[ss_j] = ss_v; }\n\
}\n\
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
let ss_a = ss_pick(ss_c[ss_col(ss_lo)], ss_live, ss_lo);\n\
let ss_b = ss_pick(ss_c[ss_col(ss_hi)], ss_live, ss_hi);\n\
write_P(i, vec2<f32>(ss_a.x + (ss_b.x - ss_a.x) * ss_frac, ss_a.y + (ss_b.y - ss_a.y) * ss_frac));\n\
// O anel avanca DUAS faixas: a pose viva vira `1 tique atras`, e cada posicao desce uma.\n\
var ss_n = array<vec4<f32>, 16>();\n\
ss_n[0] = vec4<f32>(ss_live.x, ss_live.y, ss_c[0].x, ss_c[0].y);\n\
for (var ss_j = 1u; ss_j < 16u; ss_j = ss_j + 1u) {\n\
\x20   ss_n[ss_j] = vec4<f32>(ss_c[ss_j - 1u].z, ss_c[ss_j - 1u].w, ss_c[ss_j].x, ss_c[ss_j].y);\n\
}\n\
write_ss_ring0(i, mat4x4<f32>(ss_n[0], ss_n[1], ss_n[2], ss_n[3]));\n\
write_ss_ring1(i, mat4x4<f32>(ss_n[4], ss_n[5], ss_n[6], ss_n[7]));\n\
write_ss_ring2(i, mat4x4<f32>(ss_n[8], ss_n[9], ss_n[10], ss_n[11]));\n\
write_ss_ring3(i, mat4x4<f32>(ss_n[12], ss_n[13], ss_n[14], ss_n[15]));\n";

/// A posição `k` tiques atrás, dada a COLUNA que a guarda (`0` = a viva). A posição `k ≥ 1` mora
/// nas faixas `2(k−1)` e `2(k−1)+1`, logo na coluna `(k−1)/2` — em `.xy` se `k−1` é par, `.zw` se
/// é ímpar.
///
/// ⛔⛔ **A 1.ª redacção passava o anel INTEIRO por valor** (`array<mat4x4<f32>, 4>`, 64 números)
/// a uma função chamada 34 vezes por elemento: o slit-scan custava **17,6 ns por linha** no
/// dispositivo contra `~2` dos nove irmãos, e a um milhão de objectos ocupava um quadro inteiro
/// sozinho (`16,4 ms`, doc 112 §4-septies). Hoje o anel é lido UMA vez e o avanço é uma
/// translação de colunas.
const BIBLIOTECA: &str = "\
fn ss_col(k: u32) -> u32 {\n\
\x20   return (max(k, 1u) - 1u) / 2u;\n\
}\n\
fn ss_pick(col: vec4<f32>, live: vec2<f32>, k: u32) -> vec2<f32> {\n\
\x20   if (k == 0u) { return live; }\n\
\x20   if (((k - 1u) & 1u) == 0u) { return col.xy; }\n\
\x20   return col.zw;\n\
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
        // 16 colunas de 4 faixas = 64 faixas = 32 posições `vec2`.
        assert!(CORPO.contains("ss_j < 16u"));
        assert_eq!(ANEL.len() * 16, 2 * MAX_LAG, "64 faixas = 32 posições vec2");
        assert_eq!(ANEL.len() * 4, 16, "quatro matrizes = dezasseis colunas");
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
