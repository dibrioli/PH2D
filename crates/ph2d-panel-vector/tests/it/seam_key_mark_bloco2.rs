//! ⭐ **Bloco 2 — o losango nos campos que eram dívida** (plano 35 §10): o olho e os números de um
//! filtro, o pincel do traço, a quebra e a fonte do texto, o perfil de largura, o Envelope, as
//! âncoras, o recorte, o auto layout, a pele do widget, o padrão e o texto no caminho. Cada losango
//! é pintado e registado na coluna do decorator, e o clique REAL chega à shell com os campos da
//! linha.

use std::collections::BTreeMap;

use ph2d_a11y::NodeId;
use ph2d_editor_core::widget::key_mark::{
    KeyMark, key_mark_id, set_key_marks, take_key_mark_click,
};
use ph2d_editor_core::zones::Rect;
use ph2d_host::{PointerButton, PointerEvent, PointerKind, PointerSource};
use ph2d_panel_vector::state::{self, VectorPanelState};
use ph2d_panel_vector::{FilterKindView, FilterRowView, VectorPanel, ids};
use ph2d_ui_testkit::MockPanelHost;

const VIEWPORT: Rect = Rect {
    x: 0.0,
    y: 0.0,
    w: 1600.0,
    h: 4000.0,
};

fn ev(kind: PointerKind, r: Rect, t: u128) -> PointerEvent {
    PointerEvent {
        kind,
        x: r.x + r.w * 0.5,
        y: r.y + r.h * 0.5,
        button: PointerButton::Primary,
        source: PointerSource::Mouse,
        pressure: 1.0,
        timestamp_ns: t,
    }
}

/// O losango da linha cujo PRIMEIRO campo é `campo` (com a shell a dizer `Chave`): pintado e
/// registado, à direita do controlo `controlo`, e o clique real nele entrega os campos da linha.
fn losango(mut host: MockPanelHost, campo: NodeId, controlo: NodeId, o_que: &str) {
    ph2d_editor_core::paint::set_ui_look(ph2d_tokens::UiLook::Redesign);
    set_key_marks(BTreeMap::from([(campo, KeyMark::Chave)]));
    let _ = take_key_mark_click();
    let mut st = VectorPanelState;
    let r = host
        .painted_rect::<VectorPanel>(&mut st, VIEWPORT, key_mark_id(campo))
        .unwrap_or_else(|| panic!("o losango de {o_que} não foi pintado ou não foi registado"));
    let c = host
        .painted_rect::<VectorPanel>(&mut st, VIEWPORT, controlo)
        .unwrap_or_else(|| panic!("o controlo de {o_que} não foi pintado"));
    // O controlo acaba antes da coluna, ou mora noutra linha (os chips de um segmentado descem
    // para debaixo do rótulo, a do losango).
    let cobre = c.x < r.x + r.w - 0.5
        && r.x < c.x + c.w - 0.5
        && c.y < r.y + r.h - 0.5
        && r.y < c.y + c.h - 0.5;
    assert!(!cobre, "{o_que}: o controlo cobre o losango: {c:?} × {r:?}");
    let _ = host.dispatch_pointer_event(ev(PointerKind::Down, r, 1_000_000_000));
    let _ = host.dispatch_pointer_event(ev(PointerKind::Up, r, 1_010_000_000));
    let f = take_key_mark_click()
        .unwrap_or_else(|| panic!("o clique no losango de {o_que} não chegou à shell"))
        .fields;
    assert!(f.contains(&campo), "{o_que}: o clique pediu {f:?}");
    set_key_marks(BTreeMap::new());
}

fn filtro() -> (FilterRowView, FilterKindView) {
    let row = FilterRowView {
        kind: 0,
        mode: 0,
        enabled: true,
        radius: 0.12,
        offx: 0.1,
        offy: -0.1,
        color: [0, 0, 0, 255],
        color_b: [255, 255, 255, 255],
        opacity: 1.0,
        blend: 0,
        scale: 0.25,
        detail: 3,
        seed: 0,
        grow: 0.0,
        hue: 0.0,
        sat: 0.0,
        bright: 0.0,
        stop_pos: [0.0; 8],
        stop_colors: [[0; 4]; 8],
        stop_count: 0,
        ramp_preview: [[128; 3]; ph2d_panel_vector::RAMP_PREVIEW_N],
    };
    let kind = FilterKindView {
        name: "Drop Shadow",
        radius_label: Some("Radius"),
        offset_labels: Some(("Offset X", "Offset Y")),
        color_label: Some("Color"),
        color_b_label: None,
        modes: &["Proximity", "Contour"],
        takes_blend: true,
        takes_ramp: false,
        noise_labels: None,
        grow_label: None,
        adjust_labels: None,
    };
    (row, kind)
}

/// O filtro: o OLHO tem o losango dele no cabeçalho do card (à direita do ✕), e o raio, a
/// mistura e o modo têm o do degrau.
#[test]
fn o_filtro_tem_losango_no_olho_e_nos_campos() {
    let (row, kind) = filtro();
    ph2d_panel_vector::set_current_filter_can_add(true);
    ph2d_panel_vector::set_filter_kinds(vec![kind]);
    ph2d_panel_vector::set_filter_blend_names(vec!["Normal", "Multiply"]);
    ph2d_panel_vector::set_current_filters(vec![row]);
    let olho = ids::filter_hide_id(0);
    losango(
        crate::host_aberto(),
        olho,
        ids::filter_remove_id(0),
        "o olho do filtro",
    );
    let raio = ids::filter_radius_id(0);
    losango(
        crate::host_aberto(),
        raio,
        ids::filter_radius_num_id(0),
        "o raio",
    );
    let mistura = ids::filter_blend_id(0);
    losango(
        crate::host_aberto(),
        mistura,
        mistura,
        "a mistura do filtro",
    );
    let modo = ids::filter_mode_id(0, 0);
    losango(
        crate::host_aberto(),
        modo,
        ids::filter_mode_id(0, 1),
        "o modo",
    );
    ph2d_panel_vector::set_current_filters(Vec::new());
}

/// O pincel do traço: a escala (o slider e o número) e o espelho.
#[test]
fn o_pincel_tem_losango() {
    ph2d_panel_vector::set_current_brush(Some(ph2d_panel_vector::BrushRow {
        has_art: true,
        spacing: 1.0,
        scale: 1.0,
        offset: 0.0,
        rotation_deg: 0.0,
        flip: false,
    }));
    losango(
        crate::host_aberto(),
        ids::VECTOR_BRUSH_SCALE,
        ids::VECTOR_BRUSH_SCALE_NUM,
        "a escala do pincel",
    );
    losango(
        crate::host_aberto(),
        ids::VECTOR_BRUSH_FLIP,
        ids::VECTOR_BRUSH_FLIP,
        "o espelho do pincel",
    );
    ph2d_panel_vector::set_current_brush(None);
}

/// O texto: o modo da quebra (Auto | Fixed) e o menu da fonte.
#[test]
fn a_quebra_e_a_fonte_tem_losango() {
    state::set_current_text_visible(true);
    state::set_current_text_wrap(Some(40.0));
    state::set_current_text_font(Some("Inter".to_owned()));
    losango(
        crate::host_aberto(),
        ids::VECTOR_TEXT_WRAP_AUTO,
        ids::VECTOR_TEXT_WRAP_FIXED,
        "o modo da quebra",
    );
    losango(
        crate::host_aberto(),
        ids::VECTOR_TEXT_FONT_DD,
        ids::VECTOR_TEXT_FONT_DD,
        "a fonte",
    );
    state::set_current_text_visible(false);
}

/// O perfil de largura (Expand): o catálogo e os sliders que o armam.
#[test]
fn o_perfil_tem_losango() {
    state::set_current_selection_count(1);
    let preset = ids::vector_width_preset_id(0);
    losango(
        crate::host_aberto(),
        preset,
        ids::vector_width_preset_id(1),
        "o catálogo de perfis",
    );
    losango(
        crate::host_aberto(),
        ids::VECTOR_EXPAND_W_MID,
        ph2d_tool_vector::ids::VECTOR_EXPAND_W_MID_NUM,
        "o W Mid",
    );
    state::set_current_selection_count(0);
}

/// O Envelope vivo: a gaiola (Perspective | Mesh | Pins) e a curvatura.
#[test]
fn o_envelope_tem_losango() {
    state::set_current_selection_count(1);
    state::set_current_has_envelope(true);
    state::set_current_envelope_mode(0);
    state::set_current_envelope_presets(&["Arc", "Bulge"], Some(0), 0.5);
    losango(
        crate::host_aberto(),
        ids::VECTOR_ENVELOPE_PERSPECTIVE,
        ids::VECTOR_ENVELOPE_PINS,
        "a gaiola",
    );
    losango(
        crate::host_aberto(),
        ids::VECTOR_ENVELOPE_BEND,
        ph2d_tool_vector::ids::VECTOR_ENVELOPE_BEND_NUM,
        "a curvatura",
    );
    state::set_current_has_envelope(false);
    state::set_current_selection_count(0);
}

/// A Interface: as âncoras, o recorte, o auto layout (direcção, distribuição, o absoluto do
/// filho) e a pele do widget com o ícone.
#[test]
fn a_interface_tem_losango() {
    use ph2d_panel_vector::state::{AnchorState, LayoutFlow, LayoutItem, WidgetSkinState};
    state::set_anchor_state(Some(AnchorState {
        h: Some(ids::VECTOR_ANCHOR_H_START),
        v: Some(ids::VECTOR_ANCHOR_V_END),
    }));
    losango(
        crate::host_na_interface(),
        ids::VECTOR_ANCHOR_H_START,
        ids::VECTOR_ANCHOR_H_STRETCH,
        "as âncoras",
    );
    state::set_anchor_state(None);

    state::set_frame_clip(Some(false));
    losango(
        crate::host_na_interface(),
        ids::VECTOR_FRAME_CLIP_OFF,
        ids::VECTOR_FRAME_CLIP_ON,
        "o recorte",
    );
    state::set_layout_flow(Some(LayoutFlow {
        dir: ids::VECTOR_LAYOUT_DIR_ROW,
        gap: [0.0; 2],
        pad: [0.0; 4],
        align: ids::VECTOR_LAYOUT_ALIGN_START,
        justify: ids::VECTOR_LAYOUT_JUSTIFY_START,
        size: [
            ids::VECTOR_LAYOUT_SIZE_W_FIXED,
            ids::VECTOR_LAYOUT_SIZE_H_FIXED,
        ],
        min: [0.0; 2],
        max: [0.0; 2],
        columns: 2.0,
    }));
    state::set_layout_item(Some(LayoutItem {
        grow: 0.0,
        shrink: 0.0,
        absolute: false,
        in_flow: true,
        parent_is_grid: false,
    }));
    losango(
        crate::host_na_interface(),
        ids::VECTOR_LAYOUT_DIR_OFF,
        ids::VECTOR_LAYOUT_DIR_GRID,
        "a direcção do layout",
    );
    losango(
        crate::host_na_interface(),
        ids::VECTOR_LAYOUT_JUSTIFY_START,
        ids::VECTOR_LAYOUT_JUSTIFY_AROUND,
        "a distribuição",
    );
    losango(
        crate::host_na_interface(),
        ids::VECTOR_LAYOUT_ITEM_ABSOLUTE,
        ids::VECTOR_LAYOUT_ITEM_ABSOLUTE,
        "o absoluto do filho",
    );
    state::set_layout_flow(None);
    state::set_layout_item(None);
    state::set_frame_clip(None);

    state::set_widget_skin_state(
        Some(WidgetSkinState {
            kinds: vec!["Button".to_owned(), "Toggle".to_owned()],
            selected: Some(0),
            drives: None,
            icon: Some(None),
            unknown: false,
        }),
        0,
    );
    let pele = ids::vector_widget_kind_id(0);
    losango(
        crate::host_na_interface(),
        pele,
        ids::vector_widget_kind_id(1),
        "a pele do widget",
    );
    losango(
        crate::host_na_interface(),
        ids::VECTOR_WIDGET_ICON_DD,
        ids::VECTOR_WIDGET_ICON_DD,
        "o ícone do widget",
    );
    state::set_widget_skin_state(None, 0);
}

/// O padrão e o texto no caminho: o espaçamento, o «Slide» e o lado; o início do texto e o lado.
#[test]
fn o_padrao_e_o_texto_no_caminho_tem_losango() {
    state::set_current_patternpath(true, 0.0, 1.0, 1.0, 0.0, false, 0.0);
    losango(
        crate::host_aberto(),
        ids::VECTOR_PATTERNPATH_SPACING,
        ids::VECTOR_PATTERNPATH_SPACING_NUM,
        "o espaçamento do padrão",
    );
    losango(
        crate::host_aberto(),
        ids::VECTOR_PATTERNPATH_FLIP_OFF,
        ids::VECTOR_PATTERNPATH_FLIP,
        "o lado do padrão",
    );
    state::set_current_patternpath(false, 0.0, 1.0, 1.0, 0.0, false, 0.0);

    state::set_current_text_visible(true);
    state::set_current_textpath(true, 0.25, false);
    losango(
        crate::host_aberto(),
        ids::VECTOR_TEXTPATH_OFFSET,
        ids::VECTOR_TEXTPATH_OFFSET_NUM,
        "o início do texto no caminho",
    );
    losango(
        crate::host_aberto(),
        ids::VECTOR_TEXTPATH_FLIP_OFF,
        ids::VECTOR_TEXTPATH_FLIP,
        "o lado do texto",
    );
    state::set_current_textpath(false, 0.0, false);
    state::set_current_text_visible(false);
}
