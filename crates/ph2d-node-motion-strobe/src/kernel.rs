//! **O KERNEL WGSL do `motion.strobe`** (ciclo 7, W1c — doc 112) — o `step` da CPU, no
//! dispositivo. Cortado do `lib.rs` por RESPONSABILIDADE: o `lib.rs` responde *o que o flash É*,
//! este responde *o que o dispositivo corre*.
//!
//! ## Porque ele existe
//!
//! O ciclo 6 pôs os nove `pulse.*` no dispositivo **sem consumidor** (doc 110 §8.6), e este é o
//! consumidor: sem kernel, `grid → scale → strobe ← pulse.beat → output` levava a cadeia INTEIRA
//! para a CPU e subia `102 400` elementos por quadro (doc 112 §3).
//!
//! ## A forma: a do `pulse.adsr`
//!
//! O estado (`glow` · `glow_age` · `glow_seq`) atravessa o `pre` e chega pela porta `state`
//! (`ReadWrite` na porta 2: lê-se o do tique anterior, escreve-se o deste). O `pulse` lê-se pela
//! porta 1, emparelhado por LINHA como na CPU. O resto das colunas monta a base da porta 0 — que é
//! o `out.set(name, col.clone())` da CPU.
//!
//! ⚠️⚠️ **A queda é o MESMO `f32` da CPU, e não um `pow` do WGSL.** A duração do flash vira taxa
//! por `libm::powf` ([`decay_per_tick`]) e a recorrência multiplica o brilho por ela a cada tique:
//! portada como `pow`, cada fabricante arredondaria à sua maneira e o erro COMPUNHA-SE. O slot do
//! `decay` no uniform leva a taxa JÁ derivada ([`DerivedUniform`], o canal que esta wave abriu),
//! e com ele a recorrência sai igual ao bit. Os três clamps de documento (`attack`/`hold` com
//! `f32::max`, que engole um `NaN`; a `probability` com `clamp`) passam pelo mesmo canal — são a
//! MESMA função da CPU, não uma guarda reescrita.
//!
//! ⚠️ **A curva é uma LUT** (256 amostras, o canal A1-gpu) e é a única aproximação: sem curva a
//! tabela é a identidade e o `mix` devolve o brilho a poucos ULP; com curva, o erro é o declive ÷
//! a resolução. Ela pesa só o LOOK — a coluna `glow` que atravessa o `pre` nunca a vê, logo o erro
//! não entra na recorrência.
//!
//! ⚠️ **DUAS variantes, pelo `flash_blend`** — o molde do `fx.drop_shadow`: no `Sink` a CPU não
//! escreve a coluna `blend` (a de montante atravessa com a base); com um modo, escreve a tag em
//! TODAS as linhas, por cima da de montante.

use super::{
    AGE_COL, BLEND_COLUMN, CURVE_KEY, FLASH_BLEND, GLOW_COL, MANIFEST, NEVER, PULSE_COL, SEQ_COL,
    decay_per_tick, flash_blend_tag,
};
use ph2d_curve::Curve;
use ph2d_node_registry::NodeRegistry;
use ph2d_nodegraph::gpu::{ColumnAccess, ColumnBinding, DerivedUniform, GpuKernel, LutSpec};
use ph2d_nodegraph::port::Dim;

/// O corpo. `$blend` é a linha que escreve a coluna `blend` (vazia no `Sink`).
///
/// ⚠️ **A ordem dos ramos é a do `glow_of`**, e a queda é calculada ANTES do `if` (o mesmo valor que
/// a CPU calcula no `else`: uma multiplicação sem efeitos).
macro_rules! corpo {
    ($blend:literal) => {
        concat!(
            "let sb_arrived = read_pulse_pulse(i) > 0.5;\n",
            "let sb_seq = read_state_glow_seq(i);\n",
            "var sb_lit = false;\n",
            "if (sb_arrived) {\n",
            "    sb_lit = params.probability >= 1.0 || sb_rand01(i, sb_seq) < params.probability;\n",
            "}\n",
            "var sb_age = read_state_glow_age(i) + 1.0;\n",
            "if (sb_lit) { sb_age = 0.0; }\n",
            "var sb_g = read_state_glow(i) * params.decay;\n",
            "if (sb_age < params.attack) {\n",
            "    sb_g = sb_age / params.attack;\n",
            "} else if (sb_age <= params.attack + params.hold) {\n",
            "    sb_g = 1.0;\n",
            "}\n",
            "sb_g = clamp(sb_g, 0.0, 1.0);\n",
            "write_glow(i, sb_g);\n",
            "write_glow_age(i, sb_age);\n",
            "write_glow_seq(i, sb_seq + select(0.0, 1.0, sb_arrived));\n",
            "// O LOOK: a curva primeiro (o tempo), a máscara depois (o espaço).\n",
            "let sb_look = sb_curve_sample(sb_g) * clamp(read_in_falloff(i), 0.0, 1.0);\n",
            "let sb_k = 1.0 + params.size_boost * sb_look;\n",
            "let sb_s = read_in_size(i);\n",
            "write_size(i, vec2<f32>(sb_s.x * sb_k, sb_s.y * sb_k));\n",
            "let sb_a = params.flash_amount * sb_look;\n",
            "let sb_t = read_in_tint(i);\n",
            "write_tint(i, vec4<f32>(\n",
            "    sb_t.r + (params.flash_r - sb_t.r) * sb_a,\n",
            "    sb_t.g + (params.flash_g - sb_t.g) * sb_a,\n",
            "    sb_t.b + (params.flash_b - sb_t.b) * sb_a,\n",
            "    sb_t.a));\n",
            $blend,
        )
    };
}

/// O sorteio da probabilidade — o `hash.rs` deste nó, operação a operação (a aritmética de `u32`
/// do WGSL dá a volta como o `wrapping_*` do Rust). `prev_seq.max(0.0) as u32` é o `select`: um
/// `NaN` e um negativo dão `0`.
const BIBLIOTECA: &str = concat!(
    "fn sb_rand01(row: u32, seq: f32) -> f32 {\n",
    "    var ord = 0u;\n",
    "    if (seq > 0.0) { ord = u32(seq); }\n",
    "    var h = row * 0x9e3779b9u + ord * 0x85ebca6bu;\n",
    "    h = h ^ (h >> 16u);\n",
    "    h = h * 0x7feb352du;\n",
    "    h = h ^ (h >> 15u);\n",
    "    h = h * 0x846ca68bu;\n",
    "    h = h ^ (h >> 16u);\n",
    "    return f32(h >> 8u) / 16777216.0;\n",
    "}\n",
);

/// A identidade de um `size` ausente — o `vec2_col(.., [1, 1])` da CPU.
const UM: [f32; 4] = [1.0; 4];

const LIGACOES: [ColumnBinding; 7] = [
    ColumnBinding {
        column: "size",
        dim: Dim::Vec2,
        access: ColumnAccess::ReadWrite,
        identity: UM,
        port: 0,
    },
    ColumnBinding {
        column: "tint",
        dim: Dim::Vec4,
        access: ColumnAccess::ReadWrite,
        identity: UM,
        port: 0,
    },
    ColumnBinding {
        column: "falloff",
        dim: Dim::Scalar,
        access: ColumnAccess::Read,
        identity: UM,
        port: 0,
    },
    // O disparo, emparelhado por LINHA (o `scalar_col(pulse, …, 0.0)` da CPU).
    ColumnBinding {
        column: PULSE_COL,
        dim: Dim::Scalar,
        access: ColumnAccess::Read,
        identity: [0.0; 4],
        port: 1,
    },
    ColumnBinding {
        column: GLOW_COL,
        dim: Dim::Scalar,
        access: ColumnAccess::ReadWrite,
        identity: [0.0; 4],
        port: 2,
    },
    // ⚠️ Ausente ⇒ [`NEVER`], e não `0` — `0` quer dizer *«um pulso acabou de acender»*.
    ColumnBinding {
        column: AGE_COL,
        dim: Dim::Scalar,
        access: ColumnAccess::ReadWrite,
        identity: [NEVER, 0.0, 0.0, 0.0],
        port: 2,
    },
    ColumnBinding {
        column: SEQ_COL,
        dim: Dim::Scalar,
        access: ColumnAccess::ReadWrite,
        identity: [0.0; 4],
        port: 2,
    },
];

const PARAMS: &[&str] = &[
    "decay",
    "size_boost",
    "flash_r",
    "flash_g",
    "flash_b",
    "flash_amount",
    "attack",
    "hold",
    "probability",
    FLASH_BLEND,
];

/// `Sink` — a coluna `blend` não é tocada.
static SINK: GpuKernel = GpuKernel {
    wgsl: corpo!(""),
    wgsl_lib: BIBLIOTECA,
    bindings: &LIGACOES,
    params: PARAMS,
    count_law: None,
    variant_by_param: None,
    applicable: None,
};

/// Um modo autorado — a tag em todas as linhas. `floor(v + ½)` é o `round` da CPU para `v ≥ ½`
/// (⛔ **não** o `round` do WGSL, que arredonda o meio para o PAR).
static COM_MODO: GpuKernel = GpuKernel {
    wgsl: corpo!("write_blend(i, clamp(floor(params.flash_blend + 0.5), 1.0, 6.0));\n"),
    wgsl_lib: BIBLIOTECA,
    bindings: &[
        LIGACOES[0],
        LIGACOES[1],
        LIGACOES[2],
        LIGACOES[3],
        LIGACOES[4],
        LIGACOES[5],
        LIGACOES[6],
        ColumnBinding {
            column: BLEND_COLUMN,
            dim: Dim::Scalar,
            access: ColumnAccess::Write,
            identity: [0.0; 4],
            port: 0,
        },
    ],
    params: PARAMS,
    count_law: None,
    variant_by_param: None,
    applicable: None,
};

/// O kernel registado: o despachante (o molde do `motion.move`). A forma de topo **é** a `SINK`.
pub(crate) static GPU_KERNEL: GpuKernel = GpuKernel {
    wgsl: SINK.wgsl,
    wgsl_lib: BIBLIOTECA,
    bindings: &LIGACOES,
    params: PARAMS,
    count_law: None,
    variant_by_param: Some(|param| {
        if flash_blend_tag(param(FLASH_BLEND)).is_some() {
            &COM_MODO
        } else {
            &SINK
        }
    }),
    applicable: None,
};

/// Os slots do uniform que levam a lei da CPU JÁ aplicada — as MESMAS expressões do `eval`.
static DERIVADOS: &[DerivedUniform] = &[
    DerivedUniform {
        param: "decay",
        derive: |c| decay_per_tick((c.param)("decay")),
    },
    DerivedUniform {
        param: "attack",
        derive: |c| (c.param)("attack").max(0.0),
    },
    DerivedUniform {
        param: "hold",
        derive: |c| (c.param)("hold").max(0.0),
    },
    DerivedUniform {
        param: "probability",
        derive: |c| (c.param)("probability").clamp(0.0, 1.0), // CLAMP-OK: a mesma do `eval`
    },
];

/// A resolução da tabela da curva — a mesma do `field.remap` (o precedente medido do canal).
const LUT_RESOLUTION: u32 = 256;

static LUTS: &[LutSpec] = &[LutSpec {
    name: "sb_curve",
    text_key: CURVE_KEY,
    resolution: LUT_RESOLUTION,
    fill: preenche_a_curva,
}];

/// A curva amostrada em `t = k/(n−1)`. Sem curva (ou ilegível) é a IDENTIDADE — o `map_or(glow, …)`
/// da CPU, que nem chega a construí-la.
fn preenche_a_curva(text: &str, out: &mut [f32]) {
    let curve = ph2d_curve::parse(text).unwrap_or_else(Curve::identity);
    let n = out.len();
    for (k, slot) in out.iter_mut().enumerate() {
        #[expect(clippy::cast_precision_loss, reason = "256 amostras")]
        let t = if n <= 1 {
            0.0
        } else {
            k as f32 / (n - 1) as f32
        };
        *slot = curve.eval(t);
    }
}

/// **Regista o caminho do dispositivo** — o kernel, a tabela da curva e os uniforms derivados.
pub(crate) fn regista(reg: &mut NodeRegistry) {
    reg.register_gpu_kernel(MANIFEST.id, GPU_KERNEL);
    reg.register_luts(MANIFEST.id, LUTS);
    reg.register_derived_uniforms(MANIFEST.id, DERIVADOS);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::FLASH_BLEND_LABELS;

    /// ⚠️ **O topo dos modos no WGSL é o da CPU**, e está DE FACTO no texto compilado.
    #[test]
    fn the_kernels_mode_ceiling_is_the_cpus() {
        #[expect(clippy::cast_precision_loss, reason = "7 rótulos")]
        let topo = (FLASH_BLEND_LABELS.len() - 1) as f32;
        assert_eq!(topo, 6.0);
        assert!(COM_MODO.wgsl.contains("1.0, 6.0)"));
    }

    /// **A escolha da variante é a lei da CPU**, com as duas metades.
    #[test]
    fn the_blend_variant_follows_the_cpus_tag() {
        let liga = |k: &GpuKernel| k.bindings.iter().any(|b| b.column == BLEND_COLUMN);
        for (v, com_modo) in [
            (0.0, false),
            (0.49, false),
            (f32::NAN, false),
            (f32::INFINITY, false),
            (0.5, true),
            (4.0, true),
        ] {
            let k = GPU_KERNEL.resolve(&|name| if name == FLASH_BLEND { v } else { 0.0 });
            assert_eq!(liga(k), com_modo, "flash_blend {v}");
            assert_eq!(flash_blend_tag(v).is_some(), com_modo, "flash_blend {v}");
        }
    }

    /// **Os derivados são a lei do `eval`** — incluindo as entradas de lixo que a CPU engole.
    #[test]
    fn the_derived_uniforms_are_the_evals_expressions() {
        let deriva = |nome: &str, v: f32| {
            let d = DERIVADOS.iter().find(|d| d.param == nome).unwrap();
            let param = |n: &str| if n == nome { v } else { f32::NAN };
            (d.derive)(&ph2d_nodegraph::gpu::CountLawCtx {
                inputs: &[],
                param: &param,
                playhead: 0.0,
                dt: 0.0,
            })
        };
        for ticks in [34.0, 1.0, 0.0, -3.0, f32::NAN, 500.0] {
            assert_eq!(
                deriva("decay", ticks).to_bits(),
                decay_per_tick(ticks).to_bits(),
                "decay {ticks}"
            );
        }
        assert_eq!(
            deriva("attack", f32::NAN),
            0.0,
            "um NaN engolido, como no `eval`"
        );
        assert_eq!(deriva("hold", -2.0), 0.0);
        assert_eq!(deriva("probability", 3.0), 1.0);
        assert_eq!(deriva("probability", -1.0), 0.0);
        // ⚠️ E os QUATRO slots têm de ser params do kernel — um derivado com um nome que o
        // corpo não lê seria uma lei que ninguém aplica.
        for d in DERIVADOS {
            assert!(
                PARAMS.contains(&d.param),
                "`{}` não é param do kernel",
                d.param
            );
        }
    }

    /// **A tabela sem curva é a identidade** (o `map_or(glow, …)` da CPU).
    #[test]
    fn the_unset_curve_fills_the_identity() {
        let mut t = [0.0f32; LUT_RESOLUTION as usize];
        preenche_a_curva("", &mut t);
        assert_eq!(t[0], 0.0);
        assert_eq!(t[255], 1.0);
        assert!((t[128] - 128.0 / 255.0).abs() < 1e-6);
    }
}
