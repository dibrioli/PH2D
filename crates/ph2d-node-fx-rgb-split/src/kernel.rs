//! **O KERNEL WGSL do `fx.rgb_split`** (ciclo 7, W1b — doc 112) — cortado do `lib.rs` pelo tecto de
//! LOC e por RESPONSABILIDADE: o `lib.rs` responde *o que a separação É*, este responde *o que o
//! dispositivo corre*. Nada aqui é lido pela CPU.
//!
//! ## Porque ele existe
//!
//! Sem kernel, o nó levava a cadeia INTEIRA para a CPU: `grid 320² → scale → rgb_split → output`
//! passava de `3` estágios no dispositivo para `1`, com o stream inteiro a subir por quadro — e um
//! nó de aparência é, por natureza, o último de um grafo (doc 112 §3).
//!
//! ## A forma: o molde do `motion.kaleidoscope`
//!
//! Um nó que MULTIPLICA as linhas é um `StreamOp::SourceRows` (ADR-0136): a lei de contagem
//! dimensiona o passe (`n × 3`), o corpo escreve de que linha-fonte cada saída nasce (`cp_rows`)
//! e LÊ a fonte nessa linha (`ColumnAccess::SourceRead`), e o sequenciador reúne todas as OUTRAS
//! colunas (`size`, `rot`, `id`, `falloff`…) — exactamente o `tile` da CPU, que repete em BLOCOS
//! (saída `i` = cópia `i / n`, fonte `i % n`), e é essa ordem que põe as franjas ATRÁS.
//!
//! ⚠️ **O caso APAGADO não é um `PASSTHROUGH`**, e parece que devia: com a opacidade a `0` a CPU
//! devolve a entrada tal e qual. ⛔ Mas um nó `SourceRows` arranca de uma base VAZIA (o sequenciador
//! troca-a antes de olhar para o kernel), e uma variante sem corpo emitiria **zero** elementos. ⇒ a
//! lei de contagem devolve `n` (uma cópia) e o corpo copia `P` e `tint` da fonte. A imagem é a da
//! CPU; a única diferença é de FORMA e invisível: sem `tint` na entrada, o dispositivo escreve-o a
//! branco opaco, que é a identidade que o desenho já lhe daria.

use super::{CENTER_X, CENTER_Y, COPIES, MANIFEST, MAX_INSTANCES, START};
use ph2d_node_registry::NodeRegistry;
use ph2d_nodegraph::gpu::{
    ColumnAccess, ColumnBinding, GpuKernel, ROWS_COL, SourceWindow, StreamOp,
};
use ph2d_nodegraph::port::Dim;

/// `true` quando a CPU devolve a entrada tal e qual por causa da OPACIDADE (o `dead` do `split`).
fn apagada(opacity: f32) -> bool {
    !opacity.is_finite() || opacity <= 0.0
}

/// **Quantas linhas saem** — a MESMA expressão que o `split` da CPU: três cópias, ou a entrada tal
/// e qual quando não há o que franjar, quando a opacidade está apagada, ou acima do tecto.
///
/// ⚠️ *Duas leis que discordam não rebentam: desenham outro número de coisas.* É por isso que ela
/// lê o mesmo `MAX_INSTANCES` e a mesma regra do `dead`.
fn copias(n: usize, opacity: f32) -> usize {
    if n == 0 || n.saturating_mul(COPIES) > MAX_INSTANCES || apagada(opacity) {
        1
    } else {
        COPIES
    }
}

/// O kernel. ⚠️ **A aritmética é a do `split`, operação a operação** — o centroide é `Σ/n` (a
/// redução `Sum` do pivô), o raio limpo é o braço LITERAL quando o `start` não é positivo e finito
/// (senão a divisão por `r` mexia nos elementos em cima do eixo), e o alfa é
/// `a · opacity · falloff`, nessa ordem.
pub(crate) const GPU_KERNEL: GpuKernel = GpuKernel {
    wgsl: "\
        let k_n = max(params.window_src_n, 1u);\n\
        let k_copias = params.count / k_n;\n\
        let k_row = i % k_n;\n\
        let k_c = i / k_n;\n\
        write_cp_rows(i, f32(k_row));\n\
        let k_p = read_P(k_row);\n\
        let k_t = read_tint(k_row);\n\
        // O deslocamento: `Split` é o mesmo `(x, y)` para todos; `Aberration` é radial a partir\n\
        // do centroide MAIS o eixo autorado. ⚠️ `!(mode < 0.5)` e não `mode >= 0.5`: com um\n\
        // `NaN` a CPU cai na aberração, e a comparação directa cairia no `Split`.\n\
        var k_off = vec2<f32>(params.x, params.y);\n\
        if (!(params.mode < 0.5)) {\n\
        \x20   let k_cx = reduce_cx() / f32(k_n) + params.center_x;\n\
        \x20   let k_cy = reduce_cy() / f32(k_n) + params.center_y;\n\
        \x20   let k_dx = k_p.x - k_cx;\n\
        \x20   let k_dy = k_p.y - k_cy;\n\
        \x20   if (params.start > 0.0 && params.start <= 3.4028235e38) {\n\
        \x20       let k_r = sqrt(k_dx * k_dx + k_dy * k_dy);\n\
        \x20       if (k_r <= params.start) {\n\
        \x20           k_off = vec2<f32>(0.0, 0.0);\n\
        \x20       } else {\n\
        \x20           let k_k = (k_r - params.start) / k_r;\n\
        \x20           k_off = vec2<f32>(k_dx * params.strength * k_k, k_dy * params.strength * k_k);\n\
        \x20       }\n\
        \x20   } else {\n\
        \x20       k_off = vec2<f32>(k_dx * params.strength, k_dy * params.strength);\n\
        \x20   }\n\
        }\n\
        let k_a = k_t.a * params.opacity * read_falloff(k_row);\n\
        // Bloco 0 = o fantasma R (`+off`), bloco 1 = o G+B (`−off`), bloco 2 = o próprio elemento,\n\
        // POR CIMA. Com uma cópia só (apagado / acima do tecto) é a entrada tal e qual.\n\
        if (k_copias < 3u || k_c == 2u) {\n\
        \x20   write_P(i, k_p);\n\
        \x20   write_tint(i, k_t);\n\
        } else if (k_c == 0u) {\n\
        \x20   write_P(i, vec2<f32>(k_p.x + k_off.x, k_p.y + k_off.y));\n\
        \x20   write_tint(i, vec4<f32>(k_t.r, 0.0, 0.0, k_a));\n\
        } else {\n\
        \x20   write_P(i, vec2<f32>(k_p.x - k_off.x, k_p.y - k_off.y));\n\
        \x20   write_tint(i, vec4<f32>(0.0, k_t.g, k_t.b, k_a));\n\
        }\n",
    wgsl_lib: "",
    bindings: &[
        // A fonte, lida na linha mapeada `i % n` — desacoplada do comprimento do passe.
        ColumnBinding {
            column: "P",
            dim: Dim::Vec2,
            access: ColumnAccess::SourceRead,
            identity: [0.0; 4],
            port: 0,
        },
        // ⚠️ Ausente ⇒ BRANCO OPACO, a identidade multiplicativa (o `tints` da CPU) — zeros
        // tornariam toda cópia preta e invisível.
        ColumnBinding {
            column: "tint",
            dim: Dim::Vec4,
            access: ColumnAccess::SourceRead,
            identity: [1.0; 4],
            port: 0,
        },
        // O peso da convenção do módulo — ausente ⇒ `1` (o `falloff_at` da CPU).
        ColumnBinding {
            column: "falloff",
            dim: Dim::Scalar,
            access: ColumnAccess::SourceRead,
            identity: [1.0, 0.0, 0.0, 0.0],
            port: 0,
        },
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
        // A linha-fonte de cada saída — o sequenciador reúne por ela e larga esta coluna.
        ColumnBinding {
            column: ROWS_COL,
            dim: Dim::Scalar,
            access: ColumnAccess::Write,
            identity: [0.0; 4],
            port: 0,
        },
    ],
    params: &[
        "mode", "x", "y", "strength", "opacity", CENTER_X, CENTER_Y, START,
    ],
    count_law: Some(|c| {
        let n = c.inputs.first().copied().unwrap_or(0) as usize;
        SourceWindow::of_count(n * copias(n, (c.param)("opacity")))
    }),
    variant_by_param: None,
    applicable: None,
};

/// **Regista o caminho do dispositivo** — o kernel, a reunião de molde e as duas reduções do
/// centroide (o par `Sum(P.x)`/`Sum(P.y)` do pivô, que correm antes do passe por elemento).
pub(crate) fn regista(reg: &mut NodeRegistry) {
    reg.register_gpu_kernel(MANIFEST.id, GPU_KERNEL);
    reg.register_stream_op(MANIFEST.id, StreamOp::SourceRows { port: 0 });
    reg.register_reduces(MANIFEST.id, ph2d_nodegraph::pivot::CENTROID_REDUCES);
}
