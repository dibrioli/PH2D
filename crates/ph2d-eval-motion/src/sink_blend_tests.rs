//! Gates for the sink blend door (doc 89, folha 17).
//!
//! Two properties carry the wave: the DEFAULT is byte-identical to every frame
//! this app drew before the param existed, and a chosen tag lands in the field
//! the renderer actually keys its draw runs on.

use super::{SINK_BLEND_PARAM, sink_blend_tag};
use crate::lower::lower_to_instances_onto;
use crate::{Column, RenderInstance, Stream};
use ph2d_nodegraph::graph::Graph;

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
    lower_to_instances_onto(&a_stream(), UV, SZ, sink_blend_tag(&g, sink), &mut out);
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
        lower_to_instances_onto(&a_stream(), UV, SZ, tag, &mut out);
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
