//! **How a render sink DRAWS** — o estilo do sink (doc 89, folha 17).
//!
//! Split out of `lib.rs` at the HR-18 LOC cap along the seam that was already
//! there: `lib.rs` runs the CLOCK (`MotionCookPump`) and `lower.rs` answers *what
//! does a cooked stream look like on screen*; this file answers the one question
//! that is neither — *in what STYLE does this sink draw?* — and is the door both
//! render routes ask.
//!
//! The reference is unanimous and it decided the shape: Niagara puts blend on the
//! Sprite Renderer's material, Cavalry on the layer/shader, AE and Stardust on the
//! layer. Blend belongs to the RENDERER, not to a particle — so it is a param of
//! `motion.output` and a scalar of the lowering, never a per-element column.
//!
//! ⚠️ **E a MESMA leitura decide os outros três** (pivô · filtro · ordem): as três
//! referências põem cada um deles no renderer / no material / na camada, nunca na
//! partícula. É por isso que este ficheiro devolve um [`SinkStyle`] e não quatro
//! respostas soltas — *quatro perguntas com a mesma resposta estrutural são uma
//! pergunta*, e um segundo leitor de qualquer uma delas seria livre de arredondar
//! diferente.
//!
//! ⚠️ **The tag cannot travel as a stream column, and the reason is structural.**
//! On the device `motion.output` is `GpuKernel::PASSTHROUGH`: the sequencer emits
//! no pass for it, so anything its `eval` wrote would never reach the device
//! lowering. Both lowerings take the style as an argument instead — the CPU pump
//! asks this door inside its sink loop, the shell asks it for the single sink the
//! GPU route accepts, and `ph2d-gpu-cook` receives it (that crate keeps
//! `ph2d-eval-motion` a DEV dependency on purpose, so [`SinkStyle`] lives in
//! `ph2d-render` — the crate that owns the struct every field of it belongs to).

use ph2d_nodegraph::graph::{Graph, NodeId};
use ph2d_render::{RenderInstance, SinkStyle};

/// The sink param that names a render sink's blend mode.
///
/// It is the SAME string `ph2d-node-motion-output` declares as `BLEND_PARAM`.
/// Neither crate may depend on the other (both are leaves of the node system), so
/// the agreement is pinned by a gate in the shell — the one place that sees both.
/// This substrate already knows a vocabulary of well-known NAMES (`"P"`, `"size"`,
/// `"rot"`, `"tint"`, `"uv_rect"`, `"texture_id"`, `"geometry_id"`); this joins it
/// as the first well-known *param* name rather than column.
pub const SINK_BLEND_PARAM: &str = "blend";

/// Os outros três params do sink, com os mesmos nomes que
/// `ph2d-node-motion-output` declara (e o mesmo gate da shell a pinar que
/// concordam — nenhuma das duas folhas alcança a outra).
pub const SINK_PIVOT_X_PARAM: &str = "pivot_x";
/// Ver [`SINK_PIVOT_X_PARAM`].
pub const SINK_PIVOT_Y_PARAM: &str = "pivot_y";
/// Ver [`SINK_PIVOT_X_PARAM`].
pub const SINK_FILTER_PARAM: &str = "filter";
/// Ver [`SINK_PIVOT_X_PARAM`].
pub const SINK_SORT_PARAM: &str = "sort";

/// Quão longe do centro o pivô pode ir, em fracções do tamanho — o mesmo número
/// que o nó publica como `PIVOT_LIMIT`, e o gate da shell pina que são iguais.
pub const SINK_PIVOT_LIMIT: f32 = 1.0;

/// O valor de um param do sink, ou `0.0` se ele não foi autorado.
///
/// ⚠️ **`NaN`/`inf` caem para `0.0` e não para o clamp**: um documento corrompido
/// desenha como o de sempre em vez de escolher um extremo que ninguém autorou.
fn param(graph: &Graph, sink: NodeId, name: &str) -> f32 {
    let v = graph
        .node_param_overrides(sink)
        .and_then(|p| p.get(name))
        .copied()
        .unwrap_or(0.0);
    if v.is_finite() { v } else { 0.0 }
}

/// Um param que é um TAG: arredondado meio-para-longe-de-zero e **clampado** para
/// dentro da faixa, nunca embrulhado.
fn tag(graph: &Graph, sink: NodeId, name: &str, top: u8) -> u8 {
    param(graph, sink, name).round().clamp(0.0, f32::from(top)) as u8
}

/// **O estilo com que um sink desenha.**
///
/// **A porta única.** O pump da CPU pergunta-a por sink (ele percorre muitos); a
/// shell pergunta-a pelo único sink que a rota da GPU aceita. Dois chamadores, uma
/// resposta — um segundo leitor seria livre de arredondar ou clampar de outra
/// maneira, e as duas rotas desenhariam o mesmo documento de maneiras diferentes,
/// que nenhum gate a olhar para uma rota consegue ver.
///
/// Um nó sem overrides — e todo nó que não é sink — devolve [`SinkStyle::PLAIN`],
/// que é exactamente o que os dois lowerings cravavam antes destes params
/// existirem.
#[must_use]
pub fn sink_style(graph: &Graph, sink: NodeId) -> SinkStyle {
    // O tecto do blend é o array de pipelines, lido DO RENDERER — um literal `5`
    // aqui continuaria a compilar no dia em que um sexto modo aterrasse e
    // recusaria silenciosamente escolhê-lo.
    let top = (ph2d_render::pipeline::BLEND_PIPELINE_COUNT - 1) as u8;
    let pivot_lim = SINK_PIVOT_LIMIT;
    SinkStyle {
        blend: tag(graph, sink, SINK_BLEND_PARAM, top),
        pivot: [
            param(graph, sink, SINK_PIVOT_X_PARAM).clamp(-pivot_lim, pivot_lim),
            param(graph, sink, SINK_PIVOT_Y_PARAM).clamp(-pivot_lim, pivot_lim),
        ],
        // ⚠️ O `repeat` fica em `Inherit` de propósito: com o `uv_xform` na
        // identidade as três leis de wrap concordam dentro de `[0,1]`, então um
        // knob de repetição neste sink seria **morto** até alguém escrever a
        // coluna `uv_cell` — e é o `motion.sub_uv` que a escreve, com o wrap dele.
        sampling: RenderInstance::pack_sampling(
            tag(
                graph,
                sink,
                SINK_FILTER_PARAM,
                ph2d_render::image_filter::FILTER_TAG_MAX,
            ),
            0,
        ),
        stream_order: param(graph, sink, SINK_SORT_PARAM) >= 0.5,
    }
}

/// O tag de blend de um sink — o atalho que os chamadores de sempre usam.
#[must_use]
pub fn sink_blend_tag(graph: &Graph, sink: NodeId) -> u8 {
    sink_style(graph, sink).blend
}

#[cfg(test)]
#[path = "sink_style_tests.rs"]
mod tests;
