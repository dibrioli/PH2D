//! B05 — o censo: todo setter de camada e de ajuste marca o documento «não assado», sobe a revisão do
//! painel e cria UM passo de undo que devolve a pilha de antes; um arrasto inteiro é um passo só.

use super::super::*;
use crate::layers::ReliefComposite;
use ph2d_painter_effects::adjustments::{AdjustmentKind as K, AdjustmentParams};

const LADO: u32 = 32;
const N: usize = (LADO as usize) * (LADO as usize);

struct Ids {
    a: RtLayerId,
    b: RtLayerId,
    adj: RtLayerId,
}

fn doc(kind: K) -> (PainterTool, Ids) {
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; N * 4], LADO, LADO);
    let a = t
        .add_raster_layer_with_pixels("A", vec![90u8; N * 4])
        .expect("A");
    let b = t
        .add_raster_layer_with_pixels("B", vec![160u8; N * 4])
        .expect("B");
    let adj = t.add_adjustment_layer(kind).expect("o ajuste");
    (t, Ids { a, b, adj })
}

type Acto = fn(&mut PainterTool, &Ids);

fn nada(_: &mut PainterTool, _: &Ids) {}

/// (nome, ajuste do documento, preparo fora do passo, o setter, marca «não assado»?)
fn censo() -> Vec<(&'static str, K, Acto, Acto, bool)> {
    vec![
        (
            "set_adjustment_param",
            K::HueSaturationBrightness,
            nada,
            |t, i| t.set_adjustment_param(i.adj, 0, 0.9),
            true,
        ),
        (
            "flip_adjustment_toggle",
            K::PhotoFilter,
            nada,
            |t, i| t.flip_adjustment_toggle(i.adj, 0),
            true,
        ),
        (
            "set_adjustment_segment",
            K::ColorBalance,
            nada,
            |t, i| t.set_adjustment_segment(i.adj, 2),
            true,
        ),
        (
            "set_channel_mixer_weight",
            K::ChannelMixer,
            nada,
            |t, i| t.set_channel_mixer_weight(i.adj, 0, 1, 0.9),
            true,
        ),
        (
            "set_selective_color_value",
            K::SelectiveColor,
            nada,
            |t, i| t.set_selective_color_value(i.adj, 0, 0, 0.9),
            true,
        ),
        (
            "set_curve_point",
            K::Curves,
            nada,
            |t, i| t.set_curve_point(i.adj, 0, 2, 0.5, 0.8),
            true,
        ),
        (
            "add_curve_point",
            K::Curves,
            |t, i| {
                t.remove_curve_point(i.adj, 0, 2);
            },
            |t, i| {
                t.add_curve_point(i.adj, 0);
            },
            true,
        ),
        (
            "remove_curve_point",
            K::Curves,
            nada,
            |t, i| t.remove_curve_point(i.adj, 0, 2),
            true,
        ),
        (
            "set_gradient_stop_offset",
            K::GradientMap,
            nada,
            |t, i| t.set_gradient_stop_offset(i.adj, 0, 0.2),
            true,
        ),
        (
            "add_gradient_stop",
            K::GradientMap,
            nada,
            |t, i| {
                t.add_gradient_stop(i.adj);
            },
            true,
        ),
        (
            "remove_gradient_stop",
            K::GradientMap,
            |t, i| {
                t.add_gradient_stop(i.adj);
            },
            |t, i| t.remove_gradient_stop(i.adj, 1),
            true,
        ),
        (
            "set_gradient_stop_color",
            K::GradientMap,
            nada,
            |t, i| t.set_gradient_stop_color(i.adj, 0, 0, 0.5),
            true,
        ),
        (
            "set_layer_visible",
            K::Invert,
            nada,
            |t, i| t.set_layer_visible(i.a, false),
            true,
        ),
        (
            "set_layer_name",
            K::Invert,
            nada,
            |t, i| t.set_layer_name(i.a, "Z"),
            false,
        ),
        (
            "set_layer_opacity",
            K::Invert,
            nada,
            |t, i| t.set_layer_opacity(i.a, 0.5),
            true,
        ),
        (
            "set_layer_blend_mode",
            K::Invert,
            nada,
            |t, i| t.set_layer_blend_mode(i.a, BlendMode::Multiply),
            true,
        ),
        (
            "set_layer_impasto_depth",
            K::Invert,
            nada,
            |t, i| t.set_layer_impasto_depth(i.a, 0.5),
            true,
        ),
        (
            "set_layer_impasto_depth_norm",
            K::Invert,
            nada,
            |t, i| t.set_layer_impasto_depth_norm(i.a, 0.9),
            true,
        ),
        (
            "toggle_layer_impasto_composite",
            K::Invert,
            nada,
            |t, i| t.toggle_layer_impasto_composite(i.a),
            true,
        ),
        (
            "set_layer_impasto_composite",
            K::Invert,
            |t, i| t.set_layer_impasto_composite(i.a, ReliefComposite::Add),
            |t, i| t.set_layer_impasto_composite(i.a, ReliefComposite::Level),
            true,
        ),
        (
            "set_layer_clipping",
            K::Invert,
            nada,
            |t, i| t.set_layer_clipping(i.a, true),
            true,
        ),
        (
            "set_layer_alpha_locked",
            K::Invert,
            nada,
            |t, i| t.set_layer_alpha_locked(i.a, true),
            true,
        ),
        (
            "set_layer_reference",
            K::Invert,
            nada,
            |t, i| t.set_layer_reference(i.a, true),
            true,
        ),
        (
            "move_layer_up",
            K::Invert,
            nada,
            |t, i| t.move_layer_up(i.a),
            true,
        ),
        (
            "move_layer_down",
            K::Invert,
            nada,
            |t, i| t.move_layer_down(i.b),
            true,
        ),
        (
            "add_group",
            K::Invert,
            nada,
            |t, _| {
                t.add_group();
            },
            true,
        ),
        (
            "group_active",
            K::Invert,
            nada,
            |t, _| {
                t.group_active();
            },
            true,
        ),
        (
            "toggle_mask_inverted",
            K::Invert,
            |t, _| {
                t.add_mask_to_active();
            },
            |t, _| {
                let m = t.layers.active().expect("a máscara");
                t.toggle_mask_inverted(m);
            },
            true,
        ),
    ]
}

/// B05.1 + B05.3 + B05.A1 — cada setter: pilha mudou (controlo), não assado, revisão sobe, +1 passo, e o
/// Ctrl+Z devolve a pilha de antes.
#[test]
fn todo_setter_de_camada_e_de_ajuste_marca_sobe_a_revisao_e_cria_um_passo() {
    let mut falhas = Vec::new();
    for (nome, kind, prep, acto, assa) in censo() {
        let (mut t, ids) = doc(kind);
        prep(&mut t, &ids);
        t.mark_baked();
        let (rev, fundo, pilha) = (t.layers_revision(), t.undo.undo_depth(), t.layers.clone());
        acto(&mut t, &ids);
        if t.layers == pilha {
            falhas.push(format!("{nome}: controlo — a pilha não mudou"));
            continue;
        }
        if assa && !t.has_unbaked_edits() {
            falhas.push(format!("{nome}: não marcou «não assado»"));
        }
        if t.layers_revision() == rev {
            falhas.push(format!("{nome}: a revisão não subiu"));
        }
        if t.undo.undo_depth() != fundo + 1 {
            falhas.push(format!(
                "{nome}: {} passos (queria 1)",
                t.undo.undo_depth() - fundo
            ));
            continue;
        }
        t.undo_last();
        if t.layers != pilha {
            falhas.push(format!("{nome}: o Ctrl+Z não devolveu a pilha de antes"));
        }
    }
    assert!(
        falhas.is_empty(),
        "{} de {}:\n{}",
        falhas.len(),
        censo().len(),
        falhas.join("\n")
    );
}

fn hsb(t: &PainterTool, id: RtLayerId) -> AdjustmentParams {
    t.layers
        .get(id)
        .and_then(|l| match &l.kind {
            LayerKind::Adjustment(a) => Some(a.params.clone()),
            _ => None,
        })
        .expect("o ajuste")
}

/// B05.2 — um arrasto de slider (N quadros) é UM passo: o Ctrl+Z devolve o valor de antes do arrasto e não
/// desfaz a acção anterior (a criação do ajuste fica).
#[test]
fn desfazer_um_parametro_de_ajuste_devolve_o_valor_de_antes() {
    let (mut t, ids) = doc(K::HueSaturationBrightness);
    let antes = hsb(&t, ids.adj);
    for k in 1..=10 {
        t.set_adjustment_param(ids.adj, 0, 0.5 + 0.04 * k as f32);
    }
    assert!(
        hsb(&t, ids.adj) != antes,
        "controlo: o arrasto mudou o parâmetro"
    );
    assert!(t.undo_last());
    assert!(
        t.layers.get(ids.adj).is_some(),
        "o Ctrl+Z desfez a criação do ajuste"
    );
    assert!(
        hsb(&t, ids.adj) == antes,
        "o Ctrl+Z não devolveu o valor de antes do arrasto"
    );
}
