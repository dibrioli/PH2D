//! Gates for the sink STYLE door (doc 89, folha 17).
//!
//! Two properties carry the wave, and valem para os quatro params: the DEFAULT is
//! byte-identical to every frame this app drew before they existed, and a chosen
//! value lands in the field the renderer actually reads.

use super::{
    SINK_BLEND_PARAM, SINK_FILTER_PARAM, SINK_PIVOT_LIMIT, SINK_PIVOT_X_PARAM, SINK_PIVOT_Y_PARAM,
    SINK_SORT_PARAM, sink_blend_tag,
};
use crate::lower::lower_to_instances_onto;
use crate::{Column, RenderInstance, Stream};
use ph2d_nodegraph::graph::Graph;
use ph2d_render::SinkStyle;

const UV: [f32; 4] = [0.0, 0.0, 1.0, 1.0];
const SZ: [f32; 2] = [1.0, 1.0];

fn a_stream() -> Stream {
    Stream::new(3).with("P", Column::Vec2(vec![[0.0, 0.0], [1.0, 0.0], [2.0, 0.0]]))
}

/// **The default is the world that already shipped.** A graph nobody has touched
/// answers `Mix`, and `Mix` lowers to `flip_uv == 0` — the literal both lowerings
/// hardcoded. FALSIFIED by a door that defaults to anything else, or by a packer
/// that writes bits for tag 0.
#[test]
fn an_untouched_sink_lowers_to_the_flip_uv_this_app_always_wrote() {
    let mut g = Graph::new();
    let sink = g.add_node("motion.output");
    assert_eq!(sink_blend_tag(&g, sink), 0, "untouched sink must be Mix");

    let mut out: Vec<RenderInstance> = Vec::new();
    lower_to_instances_onto(&a_stream(), UV, SZ, crate::sink_style(&g, sink), &mut out);
    assert_eq!(out.len(), 3);
    for inst in &out {
        assert_eq!(inst.flip_uv, 0, "the neutral tag must write a zero word");
    }
}

/// **A chosen tag reaches the field the RENDERER reads.** The oracle is
/// `RenderInstance::unpack_blend` — the renderer's own accessor, the one
/// `compute_runs` keys draw runs on — not a re-implementation of the shift.
/// FALSIFIED by a lowering that drops the tag, or packs it into other bits.
#[test]
fn the_authored_tag_arrives_in_the_bits_the_renderer_keys_runs_on() {
    for tag in 0..ph2d_render::pipeline::BLEND_PIPELINE_COUNT as u8 {
        let mut g = Graph::new();
        let sink = g.add_node("motion.output");
        g.set_param(sink, SINK_BLEND_PARAM, f32::from(tag));
        assert_eq!(sink_blend_tag(&g, sink), tag);

        let mut out: Vec<RenderInstance> = Vec::new();
        lower_to_instances_onto(
            &a_stream(),
            UV,
            SZ,
            SinkStyle {
                blend: tag,
                ..SinkStyle::PLAIN
            },
            &mut out,
        );
        for inst in &out {
            assert_eq!(
                RenderInstance::unpack_blend(inst.flip_uv),
                tag,
                "tag {tag} did not survive the lowering"
            );
            // The blend bits are 5-7; the flip/repeat/tint_fill bits below them
            // are still nobody's business in a Motion stream. A packer that
            // shifted wrong would show up here as a stray low bit.
            assert_eq!(inst.flip_uv & 0b1_1111, 0, "tag {tag} smeared low bits");
        }
    }
}

/// **The tag is per SINK, not per document.** Two Output nodes in one graph may
/// draw the same scene in two modes. FALSIFIED by a door that reads a document
/// -level value, or that caches the first answer.
#[test]
fn two_sinks_in_one_graph_answer_independently() {
    let mut g = Graph::new();
    let a = g.add_node("motion.output");
    let b = g.add_node("motion.output");
    g.set_param(a, SINK_BLEND_PARAM, 1.0);
    assert_eq!(sink_blend_tag(&g, a), 1);
    assert_eq!(sink_blend_tag(&g, b), 0, "the untouched sink is still Mix");
}

/// **A corrupt value composites normally instead of selecting a mode nobody
/// authored.** Out of range CLAMPS — it never wraps, because wrapping would turn
/// a stray `6` into `Mix` and a stray `7` into `Add`, and both of those LOOK
/// authored.
///
/// ⚠️ The ceiling is read from the renderer's pipeline array, so this stays true
/// on the day a seventh mode lands.
///
/// ⚠️ **The non-finite arm is a SECOND layer and is documented, not gated:**
/// `Graph::set_param` already `debug_assert`s finiteness, so an `inf`/`NaN` tag
/// cannot reach this door through the public API in a debug build — a fixture
/// that tried would panic in the setter, one frame before the thing under test.
/// The arm stays for release builds (where that assert is compiled out) and is
/// two lines; a gate for it would have to poison the map behind the setter's
/// back, which tests a graph this app cannot construct.
#[test]
fn an_out_of_range_value_clamps_rather_than_wrapping() {
    let top = (ph2d_render::pipeline::BLEND_PIPELINE_COUNT - 1) as u8;
    for (value, want) in [
        (-3.0, 0),
        (-0.4, 0),
        (0.5, 1),  // round-half-away-from-zero, like `f32::round`
        (1.49, 1), //
        (99.0, top),
    ] {
        let mut g = Graph::new();
        let sink = g.add_node("motion.output");
        g.set_param(sink, SINK_BLEND_PARAM, value);
        assert_eq!(sink_blend_tag(&g, sink), want, "value {value} mis-resolved");
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// **O RESTO DO ESTILO** (doc 89, folha 17): pivô · filtro · ordem.
// ─────────────────────────────────────────────────────────────────────────────

/// Uma fileira com tamanhos DIFERENTES — a fixtura que separa «o pivô é uma
/// fracção» de «o pivô é uma distância».
fn a_stream_of_mixed_sizes() -> Stream {
    a_stream().with(
        "size",
        Column::Vec2(vec![[2.0, 4.0], [8.0, 4.0], [1.0, 1.0]]),
    )
}

/// ⭐ **UM SINK INTOCADO DESENHA O QUADRO DE ANTES, nos QUATRO campos.**
///
/// ⚠️ Este é o gate que uma feature de estilo pode partir sem que nada dê erro:
/// um default trocado num dos quatro muda **toda cena de Motion que já existe**,
/// e o sintoma é «alguma coisa ficou diferente» sem uma linha de log.
#[test]
fn an_untouched_sink_lowers_to_the_instance_this_app_always_wrote() {
    let mut g = Graph::new();
    let sink = g.add_node("motion.output");
    assert!(
        crate::sink_style(&g, sink).is_plain(),
        "um sink sem overrides tem de ser o estilo de sempre"
    );

    let mut out: Vec<RenderInstance> = Vec::new();
    lower_to_instances_onto(
        &a_stream_of_mixed_sizes(),
        UV,
        SZ,
        crate::sink_style(&g, sink),
        &mut out,
    );
    for inst in &out {
        assert_eq!(inst.flip_uv, 0);
        assert_eq!(inst.anchor, [0.0, 0.0], "o anchor cravado era [0,0]");
        assert_eq!(inst.sampling, 0, "o sampling cravado era 0");
        assert_eq!(inst.sub_order, 0, "o sub_order novo nasce a 0");
        assert_eq!(inst.uv_xform, RenderInstance::IDENTITY_UV_XFORM);
    }
}

/// ⭐⭐ **O PIVÔ ESCALA COM O TAMANHO DE CADA LINHA.**
///
/// ⚠️ **O controle é a terceira peça da fixtura**: as duas primeiras linhas têm
/// larguras diferentes e a MESMA altura. Um lowering que tratasse o pivô como uma
/// distância daria o mesmo `anchor.x` às duas — e a diferença de `y` continuaria
/// certa, então metade das asserções passaria. *Uma fixtura de tamanho uniforme
/// não distingue as duas leis.*
#[test]
fn the_pivot_is_a_fraction_of_each_rows_own_size() {
    let mut g = Graph::new();
    let sink = g.add_node("motion.output");
    g.set_param(sink, SINK_PIVOT_X_PARAM, 0.5);
    g.set_param(sink, SINK_PIVOT_Y_PARAM, -0.25);

    let mut out: Vec<RenderInstance> = Vec::new();
    lower_to_instances_onto(
        &a_stream_of_mixed_sizes(),
        UV,
        SZ,
        crate::sink_style(&g, sink),
        &mut out,
    );
    assert_eq!(out[0].anchor, [1.0, -1.0]);
    assert_eq!(out[1].anchor, [4.0, -1.0]);
    assert_eq!(out[2].anchor, [0.5, -0.25]);
    assert_ne!(
        out[0].anchor[0], out[1].anchor[0],
        "CONTROLE: duas larguras diferentes tem de dar deslocamentos diferentes"
    );
}

/// **O pivô é CLAMPADO, nunca embrulhado** — e a cerca diz de que recurso é.
#[test]
fn the_pivot_is_clamped_into_the_frame_the_renderer_culls_by() {
    let mut g = Graph::new();
    let sink = g.add_node("motion.output");
    g.set_param(sink, SINK_PIVOT_X_PARAM, 50.0);
    g.set_param(sink, SINK_PIVOT_Y_PARAM, -50.0);
    let st = crate::sink_style(&g, sink);
    assert_eq!(st.pivot, [SINK_PIVOT_LIMIT, -SINK_PIVOT_LIMIT]);

    // ⚠️ **A metade `NaN` NÃO é alcançável daqui, e a ausência é o achado:** o
    // `Graph::set_param` tem um `debug_assert!(value.is_finite())` e o parser
    // textual recusa um override não-finito ao carregar. O guarda `is_finite` da
    // porta é herdado do `sink_blend_tag` e continua a ser defesa em profundidade
    // contra um ficheiro corrompido — *mas nenhum teste pode chegar-lhe pela API,
    // e um gate que fingisse chegar estaria a medir o `debug_assert`.*
}

/// ⭐ **O FILTRO CHEGA À CHAVE QUE O RENDERER USA PARA ESCOLHER O SAMPLER**, e o
/// oráculo é o `unpack_sampling` do renderer, não uma re-implementação do shift.
///
/// ⚠️ E o teto é o do `FilterMode::from_tag`: um item acima dele seria um modo de
/// menu que o `sink_style` devolve como `Project`.
#[test]
fn the_authored_filter_arrives_in_the_key_the_renderer_binds_by() {
    for tag in 0..=ph2d_render::image_filter::FILTER_TAG_MAX {
        let mut g = Graph::new();
        let sink = g.add_node("motion.output");
        g.set_param(sink, SINK_FILTER_PARAM, f32::from(tag));
        let st = crate::sink_style(&g, sink);
        let (filter, repeat) = RenderInstance::unpack_sampling(st.sampling);
        assert_eq!(filter, tag, "o tag {tag} nao sobreviveu a porta");
        assert_eq!(repeat, 0, "o repeat fica em Inherit ate' alguem o autorar");
    }
    // Acima do teto, clampa — nunca embrulha para um modo que ninguem escolheu.
    let mut g = Graph::new();
    let sink = g.add_node("motion.output");
    g.set_param(sink, SINK_FILTER_PARAM, 99.0);
    let (filter, _) = RenderInstance::unpack_sampling(crate::sink_style(&g, sink).sampling);
    assert_eq!(filter, ph2d_render::image_filter::FILTER_TAG_MAX);
}

/// ⭐⭐⭐ **A ORDEM DAS LINHAS SOBREVIVE AO DESEMPATE POR TEXTURA** — a célula
/// inteira, medida pela porta do produto (`sort_render_order`).
///
/// A fixtura é a mídia MISTA que a folha 17 nomeia: as linhas alternam
/// `texture_id` 7,3,7,3. Em `Texture` a ordenação REAGRUPA-as por textura (é o que
/// forma runs de desenho, e é o de sempre); em `Stream` a ordem das linhas ganha.
///
/// ⚠️ **A metade que quase ficou de fora é a PRIMEIRA asserção**: sem ela o gate
/// passaria com um lowering que pusesse `sub_order = i` SEMPRE — e aí `Texture`
/// deixaria de agrupar, o que é uma regressão de draw calls que nenhum gate de
/// pixels vê.
#[test]
fn the_row_order_only_beats_the_texture_tiebreak_when_the_sink_asks() {
    let s = a_stream()
        .with("texture_id", Column::Scalar(vec![7.0, 3.0, 7.0]))
        .with("size", Column::Vec2(vec![[1.0, 1.0]; 3]));
    let order = |stream_order: bool| {
        let mut g = Graph::new();
        let sink = g.add_node("motion.output");
        g.set_param(sink, SINK_SORT_PARAM, f32::from(u8::from(stream_order)));
        let mut out: Vec<RenderInstance> = Vec::new();
        lower_to_instances_onto(&s, UV, SZ, crate::sink_style(&g, sink), &mut out);
        ph2d_render::sort_render_order(&mut out);
        out.iter().map(|i| i.world_pos[0]).collect::<Vec<_>>()
    };
    assert_eq!(
        order(false),
        vec![1.0, 0.0, 2.0],
        "Texture: a textura 3 vem antes da 7 — e e' isto que agrupa os runs"
    );
    assert_eq!(
        order(true),
        vec![0.0, 1.0, 2.0],
        "Stream: a ordem das LINHAS ganha, que e' o que o motion.sort autorou"
    );
}

/// **A célula do FLIPBOOK: a coluna `uv_cell` é RELATIVA e compõe com o ladrilho.**
///
/// ⚠️ O gate mede as duas metades: que a coluna chega ao `uv_xform` (o campo que o
/// shader lê) e que ela é **independente do ladrilho** — a mesma célula sobre dois
/// `default_uv_rect` diferentes escreve o MESMO `uv_xform`. Uma implementação que
/// tivesse escrito UVs absolutas passaria a 1.ª asserção e falharia a 2.ª.
#[test]
fn the_uv_cell_column_is_relative_so_it_composes_with_whatever_tile_the_row_has() {
    let s = a_stream().with(
        "uv_cell",
        Column::Vec4(vec![
            [0.25, 0.5, 0.0, 0.0],
            [0.25, 0.5, 0.25, 0.5],
            [0.25, 0.5, 0.75, 0.5],
        ]),
    );
    let lower = |tile: [f32; 4]| {
        let mut out: Vec<RenderInstance> = Vec::new();
        lower_to_instances_onto(&s, tile, SZ, SinkStyle::PLAIN, &mut out);
        out
    };
    let a = lower(UV);
    assert_eq!(
        a[1].uv_xform,
        [0.25, 0.5, 0.25, 0.5],
        "a celula chega ao campo"
    );
    let b = lower([0.5, 0.5, 0.75, 1.0]);
    for (x, y) in a.iter().zip(b.iter()) {
        assert_eq!(
            x.uv_xform, y.uv_xform,
            "a celula NAO pode depender do ladrilho — senao ela e' UV absoluta"
        );
    }
    assert_ne!(
        a[0].atlas_uv, b[0].atlas_uv,
        "CONTROLE: os ladrilhos diferem"
    );
}
