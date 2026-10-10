//! ⭐⭐ **As NOVE listas que rolam deste painel ROLAM de ponta a ponta** — a prova partilhada
//! (`ph2d_ui_testkit::prova_que_a_lista_longa_rola`) sobre a função real de cada passe diferido,
//! numa banda de painéis baixa onde até a lista mais curta (as sete famílias do catálogo)
//! transborda. O picker de ícone e o de fonte pintam a linha deles pela mesma porta.

use ph2d_a11y::NodeId;
use ph2d_editor_core::panel::PaintCtx;
use ph2d_editor_core::screens::HeroLayout;
use ph2d_editor_core::zones::Rect;
use ph2d_tokens::Theme;
use ph2d_ui_testkit::{MockPanelHost, prova_que_a_lista_longa_rola};

use crate::ids;

const JANELA: Rect = Rect {
    x: 0.0,
    y: 0.0,
    w: 1600.0,
    h: 900.0,
};
/// A banda de painéis por onde a lista se clampa — baixa de propósito.
const BANDA: Rect = Rect {
    x: 1200.0,
    y: 100.0,
    w: 400.0,
    h: 120.0,
};
const CHIP: Rect = Rect {
    x: 1300.0,
    y: 110.0,
    w: 200.0,
    h: 24.0,
};
const T: Theme = Theme::Forge;

/// Rótulos sem palavra nenhuma: o que se prova é a rolagem, não o texto.
const NOMES: [&str; 20] = [
    "0", "1", "2", "3", "4", "5", "6", "7", "8", "9", "10", "11", "12", "13", "14", "15", "16",
    "17", "18", "19",
];

fn prova(dd: NodeId, opcoes: &[NodeId], pinta: fn(&mut PaintCtx)) {
    let mut layout = HeroLayout::for_viewport(JANELA);
    layout.inspector = BANDA;
    let mut host = MockPanelHost::new();
    prova_que_a_lista_longa_rola(&mut host, dd, opcoes, &mut |h| h.paint_fn(&layout, pinta));
}

#[test]
fn a_mistura_do_objecto_rola() {
    let n = ph2d_vec_render::blend::offered().count();
    let opcoes: Vec<_> = (0..n).map(ids::vector_obj_blend_option_id).collect();
    prova(ids::VECTOR_OBJ_BLEND, &opcoes, |ctx| {
        crate::paint_appearance::paint_blend_popover(ctx, CHIP, T);
    });
}

#[test]
fn a_mistura_de_uma_camada_rola() {
    let n = ph2d_vec_render::blend::offered().count();
    let opcoes: Vec<_> = (0..n).map(ids::vector_paint_blend_option_id).collect();
    prova(ids::VECTOR_PAINT_BLEND, &opcoes, |ctx| {
        crate::paint_stack_rows::paint_layer_blend_popover(ctx, CHIP, T);
    });
}

#[test]
fn a_mistura_de_um_filtro_rola() {
    crate::state::filters::set_filter_blend_names(NOMES.to_vec());
    let opcoes: Vec<_> = (0..NOMES.len())
        .map(|i| ids::filter_blend_option_id(0, i))
        .collect();
    prova(ids::filter_blend_id(0), &opcoes, |ctx| {
        crate::paint_sections::filters_blend::paint_blend_popover(ctx, 0, CHIP, T);
    });
    crate::state::filters::set_filter_blend_names(Vec::new());
}

#[test]
fn as_familias_do_catalogo_rolam() {
    let opcoes: Vec<_> = (0..ph2d_tool_vector::shapes::ALL_GROUPS.len())
        .map(ph2d_tool_vector::ids::vector_shape_group_id)
        .collect();
    prova(
        ph2d_tool_vector::ids::VECTOR_SHAPE_GROUP_DD,
        &opcoes,
        |ctx| {
            crate::paint_catalog::paint_group_popover(ctx, CHIP, T);
        },
    );
}

#[test]
fn as_pontas_rolam() {
    let opcoes: Vec<_> = (0..ph2d_vec_scene::ALL_MARKERS.len())
        .map(|i| ph2d_tool_vector::ids::vector_marker_option_id(0, i))
        .collect();
    prova(crate::paint_markers::marker_dd_id(0), &opcoes, |ctx| {
        crate::paint_markers::paint_marker_popover(ctx, 0, CHIP, T);
    });
}

#[test]
fn a_tecla_de_uma_forma_do_morph_rola() {
    crate::state::set_morph_states_state(Some(crate::state::MorphStatesState {
        rows: vec![crate::state::MorphShapeRow {
            to: NOMES[0].into(),
            when: String::new(),
            live: false,
        }],
        actions: NOMES.iter().map(|s| (*s).to_string()).collect(),
        ..Default::default()
    }));
    let n = (NOMES.len() + 1).min(ids::MAX_MORPH_ACTIONS);
    let opcoes: Vec<_> = (0..n)
        .map(|i| ids::morph_shape_key_option_id(0, i))
        .collect();
    prova(ids::morph_shape_key_button_id(0), &opcoes, |ctx| {
        crate::paint_sections::morph_arrows::paint_when_popover(ctx, 0, CHIP, T);
    });
    crate::state::set_morph_states_state(None);
}

#[test]
fn os_tokens_de_um_slot_rolam() {
    let slot = ids::TOKEN_SLOTS[0];
    let n = 1 + slot.table.len();
    let opcoes: Vec<_> = (0..n)
        .map(|i| ids::vector_token_option_id(slot.code, i))
        .collect();
    prova(slot.chip, &opcoes, |ctx| {
        crate::paint_tokens::paint_token_popover(ctx, ids::TOKEN_SLOTS[0].code, CHIP, T);
    });
}

#[test]
fn o_picker_de_icone_rola() {
    let n = ph2d_editor_core::icons::IconId::all().len() + 1;
    let opcoes: Vec<_> = (0..n).map(ids::vector_widget_icon_option_id).collect();
    prova(ids::VECTOR_WIDGET_ICON_DD, &opcoes, |ctx| {
        crate::icon_dropdown::paint(ctx, CHIP, T);
    });
}

#[test]
fn o_picker_de_fonte_rola() {
    crate::state::set_current_text_font_previews(
        NOMES
            .iter()
            .map(|s| crate::state::FontPreview {
                family: None,
                display: (*s).to_string(),
                outline: ph2d_vector::BezPath::new(),
                advance_em: 0.0,
            })
            .collect(),
    );
    let opcoes: Vec<_> = (0..NOMES.len())
        .map(ids::vector_text_font_option_id)
        .collect();
    prova(ids::VECTOR_TEXT_FONT_DD, &opcoes, |ctx| {
        crate::font_dropdown::paint(ctx, CHIP, T);
    });
    crate::state::set_current_text_font_previews(Vec::new());
}
