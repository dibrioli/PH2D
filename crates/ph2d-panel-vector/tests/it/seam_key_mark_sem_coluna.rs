//! ⭐ **F24d — os campos que não tinham a coluna do losango ganham-na** (handoff F24c §6.3): a caixa
//! de marcar, o menu, o seletor do Fill Type, o olho da entrada da pilha e o do efeito, a caixinha
//! de um parâmetro do efeito. Cada losango é pintado, registado, o controlo acaba ANTES dele, e o
//! clique REAL chega à shell com os campos da linha.

use std::collections::BTreeMap;

use ph2d_a11y::NodeId;
use ph2d_editor_core::widget::key_mark::{
    KeyMark, key_mark_id, set_key_marks, take_key_mark_click,
};
use ph2d_editor_core::zones::Rect;
use ph2d_host::{PointerButton, PointerEvent, PointerKind, PointerSource};
use ph2d_panel_vector::state::{self, Appearance, PaintRow, VectorPanelState};
use ph2d_panel_vector::{FxParamView, FxRowView, VectorPanel, ids};

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

/// O losango do campo `campo` (com a shell a dizer `Chave`), o rect do CONTROLO `controlo`, e os
/// campos que o clique real nele entrega.
fn losango(campo: NodeId, controlo: NodeId, o_que: &str) -> (Rect, Rect, Vec<NodeId>) {
    ph2d_editor_core::paint::set_ui_look(ph2d_tokens::UiLook::Redesign);
    set_key_marks(BTreeMap::from([(campo, KeyMark::Chave)]));
    let _ = take_key_mark_click();
    let mut host = crate::host_aberto();
    let mut st = VectorPanelState;
    let r = host
        .painted_rect::<VectorPanel>(&mut st, VIEWPORT, key_mark_id(campo))
        .unwrap_or_else(|| panic!("o losango de {o_que} foi pintado mas não registado"));
    let c = host
        .painted_rect::<VectorPanel>(&mut st, VIEWPORT, controlo)
        .unwrap_or_else(|| panic!("o controlo de {o_que} não foi pintado"));
    let _ = host.dispatch_pointer_event(ev(PointerKind::Down, r, 1_000_000_000));
    let _ = host.dispatch_pointer_event(ev(PointerKind::Up, r, 1_010_000_000));
    let f = take_key_mark_click()
        .unwrap_or_else(|| panic!("o clique no losango de {o_que} não chegou à shell"))
        .fields;
    set_key_marks(BTreeMap::new());
    (r, c, f)
}

fn antes(c: Rect, r: Rect) -> bool {
    c.x + c.w <= r.x + 0.5
}

fn aparencia(layers: Vec<PaintRow>) {
    state::set_current_appearance(Some(Appearance {
        opacity: 1.0,
        blend: ph2d_vec_scene::BlendMode::Normal,
        layers,
        mixed_opacity: false,
        mixed_blend: false,
        layers_differ: false,
    }));
}

fn entrada() -> PaintRow {
    PaintRow {
        is_fill: true,
        color: [0xCC, 0x33, 0x33, 0xFF],
        width: 0.0,
        enabled: true,
        opacity: 1.0,
        blend: ph2d_vec_scene::BlendMode::Normal,
        offset: [0.0, 0.0],
        dilate: 0.0,
        dilate_join: 1,
        mixed: Default::default(),
    }
}

/// A caixa «tem traço» acaba antes do losango, e o clique pede a chave dela. (Mutação: o
/// `checkbox_row` sem o `decorator` ⇒ o losango não existe.)
#[test]
fn a_caixa_de_marcar_tem_losango() {
    state::set_stroke_present(Some(true));
    let (r, c, f) = losango(
        ids::VECTOR_STROKE_PRESENT,
        ids::VECTOR_STROKE_PRESENT,
        "Stroke",
    );
    assert!(
        antes(c, r),
        "a caixa entra na coluna do losango: {c:?} × {r:?}"
    );
    assert_eq!(f, vec![ids::VECTOR_STROKE_PRESENT]);
    state::set_stroke_present(None);
}

/// O menu de mistura do OBJECTO encolhe para fora da coluna; o clique pede a chave da mistura.
#[test]
fn o_menu_de_mistura_tem_losango() {
    aparencia(Vec::new());
    let (r, c, f) = losango(ids::VECTOR_OBJ_BLEND, ids::VECTOR_OBJ_BLEND, "Blend");
    assert!(
        antes(c, r),
        "o chip entra na coluna do losango: {c:?} × {r:?}"
    );
    assert_eq!(f, vec![ids::VECTOR_OBJ_BLEND]);
    state::set_current_appearance(None);
}

/// O seletor Solid/Linear/… do Fill Type: um losango para a fileira (a tinta inteira).
#[test]
fn o_fill_type_tem_losango() {
    use ph2d_tool_vector::ids::{VECTOR_FILL_KIND_PATTERN, VECTOR_FILL_KIND_SOLID};
    state::set_current_fill(Some(ph2d_panel_vector::FillKind::Solid), None);
    let (r, c, f) = losango(
        VECTOR_FILL_KIND_SOLID,
        VECTOR_FILL_KIND_PATTERN,
        "Fill Type",
    );
    assert!(
        c.x + c.w <= r.x + 0.5 || c.y > r.y + 0.5,
        "um segmento entra na coluna do losango: {c:?} × {r:?}"
    );
    assert_eq!(f, vec![VECTOR_FILL_KIND_SOLID]);
    state::set_current_fill(None, None);
}

/// O OLHO da entrada tem o seu losango colado a ele — outro que o da tinta, à direita — e o rótulo
/// começa depois dele.
#[test]
fn o_olho_da_entrada_tem_losango() {
    aparencia(vec![entrada()]);
    let olho = ids::vector_paint_eye_id(0);
    let (r, c, f) = losango(olho, olho, "o olho");
    assert!(antes(c, r), "o olho por cima do losango: {c:?} × {r:?}");
    assert_ne!(
        key_mark_id(olho),
        key_mark_id(ids::vector_paint_swatch_id(0))
    );
    assert_eq!(f, vec![olho]);
    let mut host = crate::host_aberto();
    let rotulo = host
        .painted_rect::<VectorPanel>(&mut VectorPanelState, VIEWPORT, ids::vector_paint_row_id(0))
        .expect("o rótulo da entrada");
    assert!(antes(r, rotulo), "o rótulo por cima do losango do olho");
    state::set_current_appearance(None);
}

/// O olho do efeito (no cabeçalho do card) e a caixinha de um parâmetro dele têm losango cada um.
#[test]
fn o_efeito_tem_losango_no_olho_e_na_caixinha() {
    use ph2d_tool_vector::ids::{vector_fx_hide_id, vector_fx_remove_id, vector_fx_toggle_id};
    let p = |name, toggle| FxParamView {
        name,
        min: 0.0,
        max: 1.0,
        toggle,
        integer: false,
        value: 0.0,
    };
    ph2d_panel_vector::set_current_effects(
        true,
        false,
        &["Trim Path"],
        vec![FxRowView {
            label: "Trim Path",
            enabled: true,
            params: vec![p("Start", false), p("Smooth", true)],
            falloff_role: ph2d_panel_vector::FalloffRole::NotFalloff,
        }],
    );
    let (r, c, f) = losango(
        vector_fx_hide_id(0),
        vector_fx_remove_id(0),
        "o olho do efeito",
    );
    assert!(
        antes(c, r),
        "o ✕ do card por cima do losango: {c:?} × {r:?}"
    );
    assert_eq!(f, vec![vector_fx_hide_id(0)]);
    let t = vector_fx_toggle_id(0, 1);
    let (r, c, f) = losango(t, t, "a caixinha do efeito");
    assert!(antes(c, r), "a caixinha por cima do losango: {c:?} × {r:?}");
    assert_eq!(f, vec![t]);
    ph2d_panel_vector::set_current_effects(false, false, &[], Vec::new());
}
