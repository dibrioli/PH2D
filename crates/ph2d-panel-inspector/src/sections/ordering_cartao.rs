//! O cartão Ordering (ficha da Sprite, 05/11): Z Index e Y-Sort à vista; as outras seis por «+ Order option…», e à
//! vista se em uso. Revê a ordem de 04/11.

use super::*;
use ph2d_editor_core::panel::cards::corpo::{self, Coluna};
use ph2d_editor_core::widget::card_ink::CardInk;
use ph2d_editor_core::widget::key_mark::Eixos;
use ph2d_tool_registry::hash_node_id;

/// A lista suspensa «+ Order option…».
pub(crate) const DD_OPCOES: NodeId = hash_node_id("insp_order_options.dd");
/// As seis opções raras, pela ordem da lista.
const OPCOES: [(NodeId, &str); 6] = [
    (
        hash_node_id("insp_order_opt_layer_order"),
        "component.field.order_in_layer.1",
    ),
    (
        hash_node_id("insp_order_opt_sorting_layer"),
        "component.field.sorting_layer.1",
    ),
    (
        hash_node_id("insp_order_opt_top_level"),
        "component.top_level.name",
    ),
    (
        hash_node_id("insp_order_opt_z_relative"),
        "component.field.z_as_relative.1",
    ),
    (
        hash_node_id("insp_order_opt_behind"),
        "card.insp.behind_parent",
    ),
    (
        hash_node_id("insp_order_opt_group"),
        "component.sorting_group.name",
    ),
];

thread_local! {
    /// As opções que a mão revelou nesta sessão, por objecto (um bit por linha de [`OPCOES`]).
    static REVELADAS: std::cell::RefCell<std::collections::BTreeMap<u64, u8>> =
        const { std::cell::RefCell::new(std::collections::BTreeMap::new()) };
}

/// A opção `i` está em uso neste objecto? Em uso nunca se esconde.
fn em_uso(i: usize, info: &InspectorOrderingInfo) -> bool {
    match i {
        0 => info.order_in_layer != 0,
        1 => info.sorting_layer != 0,
        2 => info.top_level,
        3 => !info.z_as_relative,
        4 => info.show_behind_parent,
        _ => info.sorting_group,
    }
}

fn a_mostra(i: usize, info: &InspectorOrderingInfo) -> bool {
    // Numa seleção mista o valor do primário não diz o dos outros: nada se esconde.
    info.selected_count > 1
        || em_uso(i, info)
        || REVELADAS.with(|r| r.borrow().get(&info.entity_bits).copied().unwrap_or(0)) & (1 << i)
            != 0
}

/// As opções ainda por revelar, para a lista suspensa: `(id, nome)`.
pub(crate) fn opcoes_por_revelar(info: &InspectorOrderingInfo) -> Vec<(NodeId, &'static str)> {
    OPCOES
        .iter()
        .enumerate()
        .filter(|(i, _)| !a_mostra(*i, info))
        .map(|(_, (id, k))| (*id, tr(k)))
        .collect()
}

/// Os botões da lista: uma vez por quadro, antes da pintura.
pub(crate) fn prepara_opcoes(store: &mut WidgetStore) {
    for id in std::iter::once(DD_OPCOES).chain(OPCOES.iter().map(|(id, _)| *id)) {
        store.register_if_absent(
            id,
            ph2d_editor_core::interaction::InteractiveState::Button {
                state: ButtonState::Normal,
            },
        );
    }
    ph2d_editor_core::panel::cards::dropdown::declare(DD_OPCOES);
}

/// Escolher uma opção revela a linha dela. `true` = consumido.
pub(crate) fn evento_das_opcoes(ev: ph2d_editor_core::interaction::WidgetEvent) -> bool {
    let ph2d_editor_core::interaction::WidgetEvent::Click(id) = ev else {
        return false;
    };
    let Some(i) = OPCOES.iter().position(|(o, _)| *o == id) else {
        return false;
    };
    if let Some(info) = crate::state::current_inspector_ordering() {
        REVELADAS.with(|r| *r.borrow_mut().entry(info.entity_bits).or_default() |= 1 << i);
    }
    true
}

#[allow(clippy::too_many_arguments)]
pub(super) fn em_cartao(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    ink: CardInk,
    x: f32,
    w: f32,
    y: f32,
    info: &InspectorOrderingInfo,
    sec: ph2d_editor_core::property_row::Seccao,
) -> f32 {
    let z = [ids::INSP_ORDER_Z_INDEX];
    let [(xa, wa), (xb, wb)] = corpo::colunas_iguais(x, w);
    let coluna_z = [Coluna {
        rotulo: tr("component.field.z_index.1"),
        eixos: Eixos::Letra(tr("card.glyph.z")),
        unidade: None,
        ids: &z,
    }];
    let mut yy = corpo::columns(
        scene,
        text_system,
        store,
        hit_index,
        ink,
        xa,
        wa,
        y,
        &coluna_z,
        1.0,
    );
    yy = yy.max(check_row(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        xb,
        wb,
        y,
        ids::INSP_ORDER_YSORT_ENABLED,
        TextKey::new("component.field.y_sort.1"),
        sec,
    ));
    let live = |id: NodeId, snap: bool| match store.checkbox(id).map(|(_, v)| v) {
        Some(CheckboxValue::Checked) => true,
        Some(CheckboxValue::Unchecked) => false,
        _ => snap,
    };
    if live(ids::INSP_ORDER_YSORT_ENABLED, info.y_sort_enabled) {
        yy = ysort_point_rows(
            scene,
            text_system,
            theme,
            hit_index,
            store,
            x,
            w,
            yy,
            info,
            sec,
        );
    }
    let (com_ordem, com_camada) = (a_mostra(0, info), a_mostra(1, info));
    let topo = yy;
    let (xl, wl) = if com_camada { (xa, wa) } else { (x, w) };
    if com_ordem {
        let l = [ids::INSP_ORDER_ORDER_IN_LAYER];
        let coluna = [Coluna {
            rotulo: tr("component.field.order_in_layer.1"),
            eixos: Eixos::Xyzw,
            unidade: None,
            ids: &l,
        }];
        yy = corpo::columns(
            scene,
            text_system,
            store,
            hit_index,
            ink,
            xl,
            wl,
            yy,
            &coluna,
            1.0,
        );
    }
    if com_camada {
        // Irmãs, na mesma linha: a lei da compactação.
        let (xc, wc, yc) = if com_ordem {
            (xb, wb, topo)
        } else {
            (x, w, topo)
        };
        yy = yy.max(layer_row(
            scene,
            text_system,
            theme,
            hit_index,
            store,
            xc,
            wc,
            yc,
            info.sorting_layer as usize,
            sec,
        ));
    }
    macro_rules! cb {
        ($id:expr, $l:expr) => {
            yy = check_row(
                scene,
                text_system,
                theme,
                hit_index,
                store,
                x,
                w,
                yy,
                $id,
                $l,
                sec,
            );
        };
    }
    if a_mostra(2, info) {
        cb!(ids::INSP_ORDER_TOP_LEVEL, tr("component.top_level.name"));
    }
    if a_mostra(3, info) {
        cb!(
            ids::INSP_ORDER_Z_RELATIVE,
            TextKey::new("component.field.z_as_relative.1")
        );
    }
    if a_mostra(4, info) {
        cb!(ids::INSP_ORDER_SHOW_BEHIND, tr("card.insp.behind_parent"));
    }
    if a_mostra(5, info) {
        cb!(
            ids::INSP_ORDER_SORTING_GROUP,
            tr("component.sorting_group.name")
        );
        if live(ids::INSP_ORDER_SORTING_GROUP, info.sorting_group) {
            cb!(
                ids::INSP_ORDER_SORT_AT_ROOT,
                TextKey::new("component.field.sorting_group.1")
            );
        }
    }
    yy
}
