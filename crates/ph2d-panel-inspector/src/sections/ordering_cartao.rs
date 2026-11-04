//! O cartão Ordering na ordem do dono (04/11): Z + Order in Layer · Sorting Layer + Top Level ·
//! Z as Relative + Behind Parent · Y-Sort + Sorting Group; o que cada caixa abre vem por baixo.

use super::*;
use ph2d_editor_core::panel::cards::corpo::{self, Coluna};
use ph2d_editor_core::widget::card_ink::CardInk;
use ph2d_editor_core::widget::key_mark::Eixos;

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
    let (z, l) = ([ids::INSP_ORDER_Z_INDEX], [ids::INSP_ORDER_ORDER_IN_LAYER]);
    let colunas = [
        Coluna {
            rotulo: tr("component.field.z_index.1"),
            eixos: Eixos::Letra(tr("card.glyph.z")),
            unidade: None,
            ids: &z,
        },
        Coluna {
            rotulo: tr("component.field.order_in_layer.1"),
            eixos: Eixos::Xyzw,
            unidade: None,
            ids: &l,
        },
    ];
    let y = corpo::columns(
        scene,
        text_system,
        store,
        hit_index,
        ink,
        x,
        w,
        y,
        &colunas,
        1.0,
    );
    let [(xa, wa), (xb, wb)] = corpo::colunas_iguais(x, w);
    let fim = layer_row(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        xa,
        wa,
        y,
        info.sorting_layer as usize,
        sec,
    );
    check_row(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        xb,
        wb,
        corpo::caixa_ao_lado_do_rotulo(y),
        ids::INSP_ORDER_TOP_LEVEL,
        tr("component.top_level.name"),
        sec,
    );
    let mut yy = fim;
    let live = |id: NodeId, snap: bool| match store.checkbox(id).map(|(_, v)| v) {
        Some(CheckboxValue::Checked) => true,
        Some(CheckboxValue::Unchecked) => false,
        _ => snap,
    };
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
    cb!(
        ids::INSP_ORDER_Z_RELATIVE,
        TextKey::new("component.field.z_as_relative.1")
    );
    cb!(ids::INSP_ORDER_SHOW_BEHIND, tr("card.insp.behind_parent"));
    cb!(
        ids::INSP_ORDER_YSORT_ENABLED,
        TextKey::new("component.field.y_sort.1")
    );
    cb!(
        ids::INSP_ORDER_SORTING_GROUP,
        tr("component.sorting_group.name")
    );
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
    if live(ids::INSP_ORDER_SORTING_GROUP, info.sorting_group) {
        cb!(
            ids::INSP_ORDER_SORT_AT_ROOT,
            TextKey::new("component.field.sorting_group.1")
        );
    }
    yy
}
