//! ⭐⭐ **O Inspector principal em CARTÕES** (`docs/CartoesUI/02_especificacao.md` §7, maquete C):
//! os MESMOS ids do Inspector de hoje (Transform, nome, visibilidade, ordem, mistura) e os do painel
//! Vector para a aparência e a pilha Fill/Stroke — o despacho de cada painel não muda (o evento
//! chega a todos os painéis instalados, visíveis ou não). Só a pintura. `PH2D_CARDS=0` volta ao
//! Inspector de hoje.

use crate::cards_pilha::hex;
use crate::ids;
use ph2d_a11y::NodeId;
use ph2d_editor_core::ids as core_ids;
use ph2d_editor_core::interaction::{InteractiveState, WidgetStore};
use ph2d_editor_core::panel::cards::{dropdown, estilos};
use ph2d_editor_core::project::{DisplayAngle, DisplayUnit};
use ph2d_editor_core::widget::card_field::CardFieldSpec;
use ph2d_i18n::tr;
use ph2d_tool_registry::hash_node_id;
use ph2d_tool_vector::appearance;
use ph2d_tool_vector::ids as vids;

pub const CARD_OBJECT: NodeId = hash_node_id("cards.insp.object");
pub const CARD_TRANSFORM: NodeId = hash_node_id("cards.insp.transform");
pub const CARD_APPEARANCE: NodeId = hash_node_id("cards.insp.appearance");
pub const CARD_FILL: NodeId = hash_node_id("cards.insp.fill");
pub const CARD_STROKE: NodeId = hash_node_id("cards.insp.stroke");
pub const CARD_ORDERING: NodeId = hash_node_id("cards.insp.ordering");
pub const CARD_MATERIAL: NodeId = hash_node_id("cards.insp.material");
/// As duas listas suspensas (caixas próprias; as opções são os ids do painel Vector).
pub const DD_BLEND: NodeId = hash_node_id("cards.insp.blend.dd");
pub const DD_STROKE_ALIGN: NodeId = hash_node_id("cards.insp.stroke_align.dd");
/// A corrente do Scale (manter a proporção).
pub const SCALE_LOCK: NodeId = hash_node_id("cards.insp.scale_lock");
const HEX_BASE: NodeId = hash_node_id("cards.insp.hex");

/// A caixa do código hex da camada `i` da pilha (o código editável do cartão Fill/Stroke).
#[must_use]
pub fn hex_id(i: usize) -> NodeId {
    NodeId(HEX_BASE.0.wrapping_add(i as u64))
}

/// De que camada é esta caixa de hex?
#[must_use]
pub(crate) fn hex_layer(id: NodeId) -> Option<usize> {
    (0..=HEX_STROKE_DE_BASE).find(|&i| hex_id(i) == id)
}

/// As caixas de hex das duas tintas de base, depois das camadas.
pub(crate) const HEX_FILL_DE_BASE: usize = ph2d_vec_scene::MAX_PAINT_LAYERS;
pub(crate) const HEX_STROKE_DE_BASE: usize = HEX_FILL_DE_BASE + 1;

/// A amostra da linha `i` (camada, ou tinta de base).
#[must_use]
pub(crate) fn amostra_de(i: usize) -> NodeId {
    match i {
        HEX_FILL_DE_BASE => vids::VECTOR_FILL_SWATCH,
        HEX_STROKE_DE_BASE => vids::VECTOR_STROKE_SWATCH,
        _ => vids::vector_paint_swatch_id(i),
    }
}

/// A cor que a forma mostra na linha `i`.
fn cor_de(i: usize) -> Option<[u8; 4]> {
    let snap = appearance::current_vector_style();
    match i {
        HEX_FILL_DE_BASE => snap.map(|s| s.fill),
        HEX_STROKE_DE_BASE => snap.map(|s| s.stroke),
        _ => appearance::current_appearance().and_then(|a| a.layers.get(i).map(|r| r.color)),
    }
}

/// A composição liga os cartões na thread da interface (`PH2D_CARDS=0` não liga); sem isto o painel
/// pinta a coluna clássica, que é a que os testes de costura das secções medem.
pub fn set_cards_on(on: bool) {
    LIGADO.with(|c| c.set(on));
}

/// O Inspector pinta-se em cartões? (a sprite fica no de hoje — as secções dela ainda não têm cartão).
#[must_use]
pub(crate) fn active() -> bool {
    LIGADO.with(std::cell::Cell::get) && crate::state::current_inspector_sprite().is_none()
}

thread_local! {
    static LIGADO: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
    static PROPORCAO: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
    /// O hex digitado à espera de a forma o mostrar: `(camada, cor, quadros)`; depois o seletor fecha.
    static HEX_PENDENTE: std::cell::Cell<Option<(usize, [u8; 4], u8)>> =
        const { std::cell::Cell::new(None) };
}

/// Quantos quadros o seletor fica aberto, no máximo, à espera de a forma mostrar o hex digitado.
const HEX_QUADROS: u8 = 30;

/// ⭐ **Digitar o hex é ESCOLHER a cor**: a escolha vai pela porta do seletor partilhado (a mesma
/// da amostra — `apply_picker` na `ph2d-app-vec`, que já sabe a lei da seleção mista); o seletor
/// abre na amostra da camada e fecha quando a forma mostra a cor. A alfa fica a da camada salvo um
/// hex de 8 (ou 4) dígitos — a opacidade é o campo ao lado.
pub(crate) fn commit_hex(store: &mut WidgetStore, i: usize) -> bool {
    use ph2d_editor_core::interaction::InteractiveState;
    let Some(InteractiveState::TextInput { text, .. }) = store.get(hex_id(i)) else {
        return false;
    };
    let digitos = text.trim().trim_start_matches('#').len();
    let Some(nova) = ph2d_editor_core::widget::parse_hex(text) else {
        return false;
    };
    let Some(atual) = cor_de(i) else {
        return false;
    };
    let mut c = nova.rgba;
    if digitos == 3 || digitos == 6 {
        c[3] = atual[3];
    }
    let picker = ph2d_editor_core::ids::INSP_BLENDER_PICKER;
    store.open_picker_on(amostra_de(i));
    store.set_blender_value(
        picker,
        ph2d_tokens::ColorValue::from_rgba8(c[0], c[1], c[2], c[3]),
    );
    store.note_colour_pick(picker);
    HEX_PENDENTE.with(|p| p.set(Some((i, c, 0))));
    true
}

/// Fecha o seletor que o hex abriu, quando a forma já mostra a cor (ou o prazo acabou).
fn fecha_seletor_do_hex(store: &mut WidgetStore) {
    let Some((i, c, n)) = HEX_PENDENTE.with(std::cell::Cell::get) else {
        return;
    };
    let alvo = amostra_de(i);
    let chegou = cor_de(i) == Some(c);
    if store.picker_target() != Some(alvo) {
        HEX_PENDENTE.with(|p| p.set(None));
    } else if chegou || n >= HEX_QUADROS {
        store.set_picker_target(None);
        HEX_PENDENTE.with(|p| p.set(None));
    } else {
        HEX_PENDENTE.with(|p| p.set(Some((i, c, n + 1))));
    }
}

/// O código e os números de cada linha de tinta: o que a forma mostra, salvo na caixa que a mão usa.
/// O painel Vector lê-os do publicado ao pintar; o cartão lê as caixas.
fn semeia_as_linhas(store: &mut WidgetStore) {
    let pct = |alfa: u8| f64::from(alfa) * 100.0 / 255.0; // LITERAL-PX-OK: byte→percent
    let mut codigos = Vec::new();
    let mut numeros = Vec::new();
    if let Some(a) = appearance::current_appearance() {
        for (i, r) in a.layers.iter().enumerate() {
            codigos.push((hex_id(i), r.color));
            let fracao = f64::from(r.opacity) * 100.0; // LITERAL-PX-OK: fraction→percent
            numeros.push((vids::vector_paint_row_opacity_id(i), fracao));
            numeros.push((vids::vector_paint_row_width_id(i), r.width));
        }
    }
    if let Some(s) = appearance::current_vector_style() {
        codigos.push((hex_id(HEX_FILL_DE_BASE), s.fill));
        codigos.push((hex_id(HEX_STROKE_DE_BASE), s.stroke));
        numeros.push((vids::VECTOR_FILL_OPACITY_NUM, pct(s.fill[3])));
        numeros.push((vids::VECTOR_STROKE_OPACITY_NUM, pct(s.stroke[3])));
    }
    let foco = store.focus_id();
    let mao = store.number_input_drag().map(|d| d.id);
    for (id, cor) in codigos {
        store.register_if_absent(
            id,
            InteractiveState::TextInput {
                state: ph2d_editor_core::widget::TextInputState::Normal,
                text: String::new(),
                caret: 0,
                selection_anchor: None,
            },
        );
        store.mark_cancel_on_escape(id);
        if foco != Some(id)
            && let Some(InteractiveState::TextInput {
                text,
                caret,
                selection_anchor,
                ..
            }) = store.get_mut(id)
        {
            let h = hex(cor);
            if *text != h {
                *text = h;
                *caret = text.len();
                *selection_anchor = None;
            }
        }
    }
    for (id, valor) in numeros {
        if foco != Some(id) && mao != Some(id) && store.number_value(id) != Some(valor) {
            store.set_number_value(id, valor);
        }
    }
}

/// A corrente do Scale está fechada?
#[must_use]
pub(crate) fn scale_locked() -> bool {
    PROPORCAO.with(std::cell::Cell::get)
}

pub(crate) fn toggle_scale_lock() {
    PROPORCAO.with(|c| c.set(!c.get()));
}

/// Com a corrente fechada, mudar um eixo do Scale leva o outro na mesma proporção de antes.
pub(crate) fn keep_proportion(
    host: &mut dyn ph2d_editor_core::panel::PanelHostInternal,
    id: NodeId,
    info: &ph2d_editor_core::screens::hero::InspectorTransformInfo,
) {
    let (este, outro, i, j) = if id == ids::INSP_TRANSFORM_SCALE_X {
        (id, ids::INSP_TRANSFORM_SCALE_Y, 0, 1)
    } else if id == ids::INSP_TRANSFORM_SCALE_Y {
        (id, ids::INSP_TRANSFORM_SCALE_X, 1, 0)
    } else {
        return;
    };
    if !scale_locked() || info.scale[i].abs() <= f32::EPSILON {
        return;
    }
    let razao = f64::from(info.scale[j] / info.scale[i]);
    if let Some(v) = host.store().number_value(este) {
        host.store_mut().set_number_value(outro, v * razao);
    }
}

fn pos_step() -> f64 {
    match crate::state::current_display_unit() {
        DisplayUnit::Meters => 0.01, // LITERAL-PX-OK: passo em metros
        DisplayUnit::Pixels => 1.0,
    }
}

pub(crate) fn graus() -> bool {
    matches!(crate::state::current_display_angle(), DisplayAngle::Degrees)
}

/// Marca os campos, declara as listas suspensas e semeia as dobras — com o store MUTÁVEL.
pub(crate) fn prepare(store: &mut WidgetStore) {
    let p = pos_step();
    let a = if graus() { 1.0 } else { 0.01 };
    let campos: [(NodeId, f64, f64, Option<f64>); 12] = [
        (vids::VECTOR_STROKE_OPACITY_NUM, 1.0, 1.0, Some(100.0)),
        (ids::INSP_TRANSFORM_POS_X, p, p, None),
        (ids::INSP_TRANSFORM_POS_Y, p, p, None),
        (ids::INSP_TRANSFORM_ROT, a, a, Some(0.0)),
        (ids::INSP_TRANSFORM_SCALE_X, 0.1, 0.01, Some(1.0)),
        (ids::INSP_TRANSFORM_SCALE_Y, 0.1, 0.01, Some(1.0)),
        (ids::INSP_TRANSFORM_SKEW_X, a, a, Some(0.0)),
        (ids::INSP_TRANSFORM_SKEW_Y, a, a, Some(0.0)),
        (vids::VECTOR_OBJ_OPACITY_NUM, 1.0, 1.0, Some(100.0)),
        (vids::VECTOR_FILL_OPACITY_NUM, 1.0, 1.0, Some(100.0)),
        (vids::VECTOR_WIDTH_NUM, 1.0, 0.1, None),
        (ids::INSP_ORDER_Z_INDEX, 1.0, 0.1, Some(0.0)),
    ];
    for (id, step, scrub, default) in campos {
        store.mark_card_field(
            id,
            CardFieldSpec {
                step,
                scrub,
                default,
            },
        );
    }
    for id in [DD_BLEND, DD_STROKE_ALIGN, SCALE_LOCK] {
        store.register_if_absent(
            id,
            InteractiveState::Button {
                state: ph2d_editor_core::widget::ButtonState::Normal,
            },
        );
    }
    dropdown::declare(DD_BLEND);
    dropdown::declare(DD_STROKE_ALIGN);
    if let Some((linha, _)) = crate::state::current_inspector_transform()
        .and_then(|t| store.hierarchy_row_of_entity(t.entity_bits))
    {
        store.set_tooltip(core_ids::hier_lock_companion(linha), tr("card.insp.lock"));
    }
    for i in 0..ph2d_vec_scene::MAX_PAINT_LAYERS {
        for id in [
            vids::vector_paint_row_opacity_id(i),
            vids::vector_paint_row_width_id(i),
        ] {
            let (step, scrub) = (1.0, 1.0);
            let default = (id == vids::vector_paint_row_opacity_id(i)).then_some(100.0);
            store.mark_card_field(
                id,
                CardFieldSpec {
                    step,
                    scrub,
                    default,
                },
            );
        }
        for (id, chave) in [
            (vids::vector_paint_eye_id(i), "card.insp.layer_visible"),
            (vids::vector_paint_del_id(i), "card.insp.layer_remove"),
            (vids::VECTOR_FILL_EYE, "card.insp.layer_visible"),
            (vids::VECTOR_STROKE_EYE, "card.insp.layer_visible"),
            (vids::VECTOR_FILL_PRESENT, "card.insp.layer_remove"),
            (vids::VECTOR_STROKE_PRESENT, "card.insp.layer_remove"),
        ] {
            if store.tooltip_for(id).is_none() {
                store.set_tooltip(id, tr(chave));
            }
        }
    }
    fecha_seletor_do_hex(store);
    for card in [CARD_FILL, CARD_STROKE] {
        let b = estilos::botao_de(card);
        if store.tooltip_for(b).is_none() {
            store.set_tooltip(b, tr("card.styles.open"));
        }
    }
    for i in 0..ph2d_vec_scene::MAX_PAINT_LAYERS {
        let d = estilos::solta_de(i);
        if store.tooltip_for(d).is_none() {
            store.set_tooltip(d, tr("card.styles.detach"));
        }
    }
    semeia_as_linhas(store);
    store.set_collapsed_if_unchosen(CARD_ORDERING, true);
    store.set_collapsed_if_unchosen(CARD_MATERIAL, true);
    for (id, chave) in [
        (ids::INSP_TRANSFORM_RESET, "card.insp.reset"),
        (SCALE_LOCK, "card.insp.scale_lock"),
        (ids::INSP_VISIBILITY_CHECK, "card.insp.visible"),
        (vids::VECTOR_PAINT_ADD_FILL, "card.insp.add_fill"),
        (vids::VECTOR_PAINT_ADD_STROKE, "card.insp.add_stroke"),
        (vids::VECTOR_CAP_BUTT, "card.insp.cap_butt"),
        (vids::VECTOR_CAP_ROUND, "card.insp.cap_round"),
        (vids::VECTOR_CAP_SQUARE, "card.insp.cap_square"),
        (vids::VECTOR_JOIN_MITER, "card.insp.join_miter"),
        (vids::VECTOR_JOIN_ROUND, "card.insp.join_round"),
        (vids::VECTOR_JOIN_BEVEL, "card.insp.join_bevel"),
        (
            vids::VECTOR_ARRANGE_ROTATE_CW,
            "panel.vector.arrange.rotate_cw",
        ),
        (vids::VECTOR_ARRANGE_FLIP_H, "panel.vector.arrange.flip_h"),
        (vids::VECTOR_ARRANGE_FLIP_V, "panel.vector.arrange.flip_v"),
    ] {
        if store.tooltip_for(id).is_none() {
            store.set_tooltip(id, tr(chave));
        }
    }
}

pub(crate) use crate::cards_corpos::cards;

#[cfg(test)]
#[path = "cards_tests.rs"]
mod tests;
