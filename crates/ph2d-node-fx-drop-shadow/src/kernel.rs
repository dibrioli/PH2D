//! **O KERNEL WGSL do `fx.drop_shadow`** (ciclo 7, W1b — doc 112) — cortado do `lib.rs` por
//! RESPONSABILIDADE: o `lib.rs` responde *o que a sombra É*, este responde *o que o dispositivo
//! corre*. Nada aqui é lido pela CPU.
//!
//! ## Porque ele existe
//!
//! Sem kernel, o nó levava a cadeia INTEIRA para a CPU: `grid 320² → scale → drop_shadow → output`
//! passava de `3` estágios no dispositivo para `1`, com `204 800` elementos a subir por quadro —
//! e uma sombra é, por natureza, o último nó de um grafo (doc 112 §3).
//!
//! ## A forma: o molde do `fx.rgb_split`
//!
//! Um `StreamOp::SourceRows` (ADR-0136): a lei de contagem dimensiona o passe (`n × (taps + 1)`),
//! o corpo escreve de que linha-fonte cada saída nasce (`cp_rows`) e LÊ a fonte nessa linha, e o
//! sequenciador reúne as OUTRAS colunas — o `tile` da CPU, em BLOCOS (saída `i` = cópia `i / n`,
//! fonte `i % n`): todo o tap 0, depois todo o tap 1, …, e os elementos por último, por cima.
//!
//! ⚠️ **DUAS variantes, escolhidas pelo `shadow_blend`**, e não uma com a coluna sempre escrita. No
//! `Sink` a CPU **não toca** na coluna `blend` (o `tile` já copiou a de montante, ou ela não
//! existe); a variante `SINK` não a liga, e o sequenciador reúne-a como reúne o `id`. Escrevê-la a
//! `0` daria a MESMA imagem (a descida lê `0` como *o modo do sink*), mas pagaria uma coluna por
//! quadro para não mudar um pixel.
//!
//! ⚠️ **O caso APAGADO não é um `PASSTHROUGH`** (a mesma razão do `fx.rgb_split`): um nó
//! `SourceRows` arranca de uma base VAZIA. A lei de contagem devolve `n` e o corpo copia.
//!
//! ⚠️ **A trigonometria é a folha parabólica, portada** — `ds_sin_cycles` é o `trig.rs` operação a
//! operação, e o disco de Vogel é o `soft::disc`. Os literais que atravessam a fronteira (o ângulo
//! de ouro, os taps, o topo dos modos) têm gate contra as consts da CPU, abaixo.

use super::{MANIFEST, MAX_INSTANCES, SHADOW_BLEND, shadow_blend_tag, soft};
use ph2d_node_registry::NodeRegistry;
use ph2d_nodegraph::gpu::{
    ColumnAccess, ColumnBinding, CountLawCtx, GpuKernel, ROWS_COL, SourceWindow, StreamOp,
};
use ph2d_nodegraph::port::Dim;

/// `true` quando a CPU devolve a entrada tal e qual por causa do ALFA da cor (o `dead` do `cast`).
fn apagada(a: f32) -> bool {
    !a.is_finite() || a <= 0.0
}

/// Quantos taps a CPU vai emitir — o `taps(..).len()` do `cast`, sem construir o disco.
fn taps(softness: f32) -> usize {
    if softness.is_finite() && softness > 0.0 {
        soft::TAPS
    } else {
        1
    }
}

/// **Quantas cópias por elemento saem** — a MESMA expressão que o `cast`: `taps + 1`, ou a entrada
/// tal e qual quando não há o que sombrear, quando o alfa está apagado, ou acima do tecto.
///
/// ⚠️ *Duas leis que discordam não rebentam: desenham outro número de coisas* — há gate a pô-la
/// ao lado do `cast` em toda a grelha de casos.
fn copias(n: usize, a: f32, softness: f32) -> usize {
    let k = taps(softness) + 1;
    if n == 0 || n.saturating_mul(k) > MAX_INSTANCES || apagada(a) {
        1
    } else {
        k
    }
}

fn lei_de_contagem(c: &CountLawCtx<'_>) -> SourceWindow {
    let n = c.inputs.first().copied().unwrap_or(0) as usize;
    SourceWindow::of_count(n * copias(n, (c.param)("a"), (c.param)("softness")))
}

/// O corpo, até ao ramo de cada linha. ⚠️ `k_copias > 2` **é** o `split` da CPU: a lei de contagem
/// só devolve `17` quando o `softness` pede o disco.
macro_rules! corpo {
    ($verbatim_blend:literal, $sombra_blend:literal) => {
        concat!(
            "let k_n = max(params.window_src_n, 1u);\n",
            "let k_copias = params.count / k_n;\n",
            "let k_row = i % k_n;\n",
            "let k_c = i / k_n;\n",
            "write_cp_rows(i, f32(k_row));\n",
            "let k_p = read_P(k_row);\n",
            "let k_t = read_tint(k_row);\n",
            "if (k_copias < 2u || k_c == k_copias - 1u) {\n",
            "    write_P(i, k_p);\n",
            "    write_tint(i, k_t);\n",
            $verbatim_blend,
            "} else {\n",
            "    let k_cs = ds_cos_sin(params.direction / 360.0);\n",
            "    let k_off = vec2<f32>(k_cs.x * params.distance, k_cs.y * params.distance);\n",
            "    var k_q = k_off;\n",
            "    var k_a = params.a * k_t.a * read_falloff(k_row);\n",
            "    if (k_copias > 2u) {\n",
            "        let k_o = ds_disc(params.softness, k_c);\n",
            "        k_q = vec2<f32>(k_off.x + k_o.x, k_off.y + k_o.y);\n",
            "        k_a = ds_per_tap_alpha(k_a);\n",
            "    }\n",
            "    write_P(i, vec2<f32>(k_p.x + k_q.x, k_p.y + k_q.y));\n",
            "    write_tint(i, vec4<f32>(params.r, params.g, params.b, k_a));\n",
            $sombra_blend,
            "}\n",
        )
    };
}

/// As funções da folha — `trig.rs` e `soft.rs` portados.
const BIBLIOTECA: &str = concat!(
    "// O seno parabólico corrigido em CICLOS — o `trig.rs` deste nó.\n",
    "fn ds_sin_cycles(phase: f32) -> f32 {\n",
    "    let f = phase - floor(phase);\n",
    "    var p: f32;\n",
    "    if (f < 0.5) {\n",
    "        let u = f * 2.0;\n",
    "        p = 4.0 * u * (1.0 - u);\n",
    "    } else {\n",
    "        let u = (f - 0.5) * 2.0;\n",
    "        p = -4.0 * u * (1.0 - u);\n",
    "    }\n",
    "    return 0.225 * (p * abs(p) - p) + p;\n",
    "}\n",
    "fn ds_cos_sin(phase: f32) -> vec2<f32> {\n",
    "    return vec2<f32>(ds_sin_cycles(phase + 0.25), ds_sin_cycles(phase));\n",
    "}\n",
    "// O tap `k` do disco de Vogel de raio `r` — o `soft::disc`.\n",
    "fn ds_disc(r: f32, k: u32) -> vec2<f32> {\n",
    "    let t = (f32(k) + 0.5) / 16.0;\n",
    "    let rad = r * sqrt(t);\n",
    "    let cs = ds_cos_sin(f32(k) * 0.38196602);\n",
    "    return vec2<f32>(rad * cs.x, rad * cs.y);\n",
    "}\n",
    "// O alfa de cada tap para a UNIÃO valer `a` — o `soft::per_tap_alpha`, quatro `sqrt`.\n",
    "fn ds_per_tap_alpha(a: f32) -> f32 {\n",
    "    if (!(a >= 0.0 && a <= 1.0)) { return a; }\n",
    "    var x = 1.0 - a;\n",
    "    x = sqrt(x);\n",
    "    x = sqrt(x);\n",
    "    x = sqrt(x);\n",
    "    x = sqrt(x);\n",
    "    return 1.0 - x;\n",
    "}\n",
);

const LIDAS: [ColumnBinding; 3] = [
    ColumnBinding {
        column: "P",
        dim: Dim::Vec2,
        access: ColumnAccess::SourceRead,
        identity: [0.0; 4],
        port: 0,
    },
    // ⚠️ Ausente ⇒ BRANCO OPACO (o `tints` da CPU) — o alfa da sombra herda o do elemento.
    ColumnBinding {
        column: "tint",
        dim: Dim::Vec4,
        access: ColumnAccess::SourceRead,
        identity: [1.0; 4],
        port: 0,
    },
    // Ausente ⇒ `1` (o `falloff_at` da CPU).
    ColumnBinding {
        column: "falloff",
        dim: Dim::Scalar,
        access: ColumnAccess::SourceRead,
        identity: [1.0, 0.0, 0.0, 0.0],
        port: 0,
    },
];

const ESCRITAS: [ColumnBinding; 3] = [
    ColumnBinding {
        column: "P",
        dim: Dim::Vec2,
        access: ColumnAccess::Write,
        identity: [0.0; 4],
        port: 0,
    },
    ColumnBinding {
        column: "tint",
        dim: Dim::Vec4,
        access: ColumnAccess::Write,
        identity: [1.0; 4],
        port: 0,
    },
    ColumnBinding {
        column: ROWS_COL,
        dim: Dim::Scalar,
        access: ColumnAccess::Write,
        identity: [0.0; 4],
        port: 0,
    },
];

/// Ausente ⇒ `0`, *o modo do sink* — o mesmo que a CPU põe nas linhas dos elementos.
const BLEND: [ColumnBinding; 2] = [
    ColumnBinding {
        column: super::BLEND_COLUMN,
        dim: Dim::Scalar,
        access: ColumnAccess::SourceRead,
        identity: [0.0; 4],
        port: 0,
    },
    ColumnBinding {
        column: super::BLEND_COLUMN,
        dim: Dim::Scalar,
        access: ColumnAccess::Write,
        identity: [0.0; 4],
        port: 0,
    },
];

const PARAMS: &[&str] = &[
    "direction",
    "distance",
    "r",
    "g",
    "b",
    "softness",
    "a",
    SHADOW_BLEND,
];

/// `Sink` — a coluna `blend` não é ligada, e o sequenciador reúne a de montante.
static SINK: GpuKernel = GpuKernel {
    wgsl: corpo!("", ""),
    wgsl_lib: BIBLIOTECA,
    bindings: &[
        LIDAS[0],
        LIDAS[1],
        LIDAS[2],
        ESCRITAS[0],
        ESCRITAS[1],
        ESCRITAS[2],
    ],
    params: PARAMS,
    count_law: Some(lei_de_contagem),
    variant_by_param: None,
    applicable: None,
};

/// Um modo autorado — as sombras levam a tag (`floor(v + ½)` é o `round` da CPU para `v ≥ ½`, e
/// ⛔ **não** o `round` do WGSL, que arredonda o meio para o PAR), os elementos a sua.
static COM_MODO: GpuKernel = GpuKernel {
    wgsl: corpo!(
        "    write_blend(i, read_blend(k_row));\n",
        "    write_blend(i, clamp(floor(params.shadow_blend + 0.5), 1.0, 6.0));\n"
    ),
    wgsl_lib: BIBLIOTECA,
    bindings: &[
        LIDAS[0],
        LIDAS[1],
        LIDAS[2],
        BLEND[0],
        ESCRITAS[0],
        ESCRITAS[1],
        ESCRITAS[2],
        BLEND[1],
    ],
    params: PARAMS,
    count_law: Some(lei_de_contagem),
    variant_by_param: None,
    applicable: None,
};

/// O kernel registado: o despachante. A forma de topo **é** a `SINK` (o molde do `motion.move`).
pub(crate) static GPU_KERNEL: GpuKernel = GpuKernel {
    wgsl: SINK.wgsl,
    wgsl_lib: BIBLIOTECA,
    bindings: SINK.bindings,
    params: PARAMS,
    count_law: Some(lei_de_contagem),
    variant_by_param: Some(|param| {
        if shadow_blend_tag(param(SHADOW_BLEND)).is_some() {
            &COM_MODO
        } else {
            &SINK
        }
    }),
    applicable: None,
};

/// **Regista o caminho do dispositivo** — o kernel e a reunião de molde.
pub(crate) fn regista(reg: &mut NodeRegistry) {
    reg.register_gpu_kernel(MANIFEST.id, GPU_KERNEL);
    reg.register_stream_op(MANIFEST.id, StreamOp::SourceRows { port: 0 });
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::SHADOW_BLEND_LABELS;
    use ph2d_nodegraph::attr::{Column, Stream};

    /// O topo da escada dos modos (`SHADOW_BLEND_LABELS.len() − 1`), como está no WGSL.
    const TOPO_WGSL: &str = "6.0";
    /// O ângulo de ouro em ciclos, como está no WGSL.
    const OURO_WGSL: &str = "0.38196602";
    /// `soft::TAPS`, como está no WGSL.
    const TAPS_WGSL: &str = "16.0";

    /// ⚠️ **Os literais do WGSL são as consts da CPU.** Um ângulo de ouro com um dígito a menos
    /// rodava o disco inteiro em silêncio, e a paridade só o via com a maciez ligada.
    #[test]
    fn the_kernels_literals_are_the_cpus_constants() {
        assert_eq!(
            OURO_WGSL.parse::<f32>().unwrap().to_bits(),
            soft::GOLDEN_TURN.to_bits()
        );
        #[expect(clippy::cast_precision_loss, reason = "TAPS = 16")]
        let taps = soft::TAPS as f32;
        assert_eq!(TAPS_WGSL.parse::<f32>().unwrap(), taps);
        #[expect(clippy::cast_precision_loss, reason = "7 rótulos")]
        let topo = (SHADOW_BLEND_LABELS.len() - 1) as f32;
        assert_eq!(TOPO_WGSL.parse::<f32>().unwrap(), topo);
        // E cada literal está DE FACTO no texto que o dispositivo compila — senão o gate mede
        // uma const que ninguém usa.
        assert!(BIBLIOTECA.contains(&format!("* {OURO_WGSL})")));
        assert!(BIBLIOTECA.contains(&format!(") / {TAPS_WGSL};")));
        assert!(COM_MODO.wgsl.contains(&format!("1.0, {TOPO_WGSL})")));
    }

    /// **A lei de contagem é a do `cast`**, em toda a grelha de casos que a movem — o alfa (vivo,
    /// zero, negativo, `NaN`), a maciez (desligada, ligada, `NaN`, `∞`) e o tecto (os dois lados
    /// dele, com e sem disco).
    #[test]
    fn the_count_law_is_the_cpus_count() {
        let stream = |n: usize| {
            let mut s = Stream::new(n);
            s.set("P", Column::Vec2(vec![[0.0, 0.0]; n]));
            s
        };
        let hard_edge = MAX_INSTANCES / 2;
        let soft_edge = MAX_INSTANCES / (soft::TAPS + 1);
        let mut checked = 0;
        for n in [0, 1, 7, soft_edge, soft_edge + 1, hard_edge, hard_edge + 1] {
            let input = stream(n);
            for a in [0.35, 0.0, -1.0, f32::NAN] {
                for softness in [0.0, 0.3, f32::NAN, f32::INFINITY, -2.0] {
                    let cpu =
                        super::super::cast(&input, 315.0, 0.2, [0.0, 0.0, 0.0, a], softness, 0.0);
                    assert_eq!(
                        n * copias(n, a, softness),
                        cpu.count(),
                        "n {n}, a {a}, softness {softness}"
                    );
                    checked += 1;
                }
            }
        }
        assert_eq!(checked, 7 * 4 * 5, "a grelha correu inteira");
    }

    /// ⚠️ **A escolha da variante é a lei da CPU** (`shadow_blend_tag`) — e as duas metades: o
    /// `Sink` não liga a coluna, um modo liga-a.
    #[test]
    fn the_blend_variant_follows_the_cpus_tag() {
        let liga = |k: &GpuKernel| {
            k.bindings
                .iter()
                .any(|b| b.column == super::super::BLEND_COLUMN)
        };
        for (v, com_modo) in [
            (0.0, false),
            (0.49, false),
            (f32::NAN, false),
            (f32::INFINITY, false),
            (0.5, true),
            (4.0, true),
            (99.0, true),
        ] {
            let k = GPU_KERNEL.resolve(&|name| if name == SHADOW_BLEND { v } else { 0.0 });
            assert_eq!(liga(k), com_modo, "shadow_blend {v}");
            assert_eq!(shadow_blend_tag(v).is_some(), com_modo, "shadow_blend {v}");
        }
    }
}
