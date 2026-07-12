//! Testes do [`VectorTool`] — arquivo irmão de `tool.rs` (teto de 700 LOC por arquivo,
//! HR-18). Mesmo padrao de `shapes_tests.rs`: o modulo e declarado la com `#[path]`, entao
//! `super::*` continua sendo o modulo `tool`.

use super::*;
use crate::params::{WIDTH_MAX_PX, WIDTH_MIN_PX};
use ph2d_a11y::NodeId;

#[test]
fn fresh_tool_defaults() {
    let t = VectorTool::new();
    assert_eq!(t.stroke_rgba(), [240, 240, 245, 255]); // white
    assert_eq!(t.fill_rgba(), [90, 150, 230, 255]); // blue
    assert_eq!(t.stroke_width_px(), DEFAULT_STROKE_WIDTH_PX);
}

#[test]
fn width_slider_maps_normalized_to_px() {
    let mut t = VectorTool::new();
    Tool::handle_panel_event(&mut t, PanelEvent::SetValue(ids::VECTOR_WIDTH, 0.0));
    assert_eq!(t.stroke_width_px(), WIDTH_MIN_PX);
    Tool::handle_panel_event(&mut t, PanelEvent::SetValue(ids::VECTOR_WIDTH, 1.0));
    assert_eq!(t.stroke_width_px(), WIDTH_MAX_PX);
}

#[test]
fn set_stroke_sets_colour_and_flags_apply() {
    let mut t = VectorTool::new();
    assert!(!t.take_apply_to_selected());
    t.set_stroke_rgba([220, 60, 60, 255]);
    assert_eq!(t.stroke_rgba(), [220, 60, 60, 255]);
    assert!(t.take_apply_to_selected());
    assert!(!t.take_apply_to_selected(), "drained");
}

#[test]
fn set_fill_sets_colour_and_flags_apply() {
    let mut t = VectorTool::new();
    t.set_fill_rgba([70, 190, 90, 255]);
    assert_eq!(t.fill_rgba(), [70, 190, 90, 255]);
    assert!(t.take_apply_to_selected());
}

#[test]
fn opacity_sliders_set_fill_and_stroke_alpha_and_flag_apply() {
    let mut t = VectorTool::new();
    // Fill Opacity → 0 % = invisible (replaces the old "None" button).
    Tool::handle_panel_event(&mut t, PanelEvent::SetValue(ids::VECTOR_FILL_OPACITY, 0.0));
    assert_eq!(t.fill_rgba()[3], 0);
    assert!(t.take_apply_to_selected());
    // Fill Opacity → 100 %.
    Tool::handle_panel_event(&mut t, PanelEvent::SetValue(ids::VECTOR_FILL_OPACITY, 1.0));
    assert_eq!(t.fill_rgba()[3], 255);
    // Stroke Opacity → 50 % ≈ 128.
    Tool::handle_panel_event(
        &mut t,
        PanelEvent::SetValue(ids::VECTOR_STROKE_OPACITY, 0.5),
    );
    assert_eq!(t.stroke_rgba()[3], 128);
    assert!(t.take_apply_to_selected());
}

#[test]
fn foreign_node_id_ignored() {
    let mut t = VectorTool::new();
    let before = t.clone();
    Tool::handle_panel_event(&mut t, PanelEvent::SetValue(NodeId(999), 0.5));
    Tool::handle_panel_event(&mut t, PanelEvent::Click(NodeId(999)));
    assert_eq!(t, before);
}

#[test]
fn mode_buttons_switch_the_draw_mode() {
    let mut t = VectorTool::new();
    assert_eq!(t.mode(), DrawMode::Select); // default
    for (id, want) in [
        (ids::VECTOR_MODE_PEN, DrawMode::Pen),
        (ids::VECTOR_MODE_NODE, DrawMode::Node),
        (ids::VECTOR_MODE_TEXT, DrawMode::Text),
        // O 5º pill: sem ele, desenhar uma forma deixava a fileira toda apagada.
        (ids::VECTOR_MODE_SHAPE, DrawMode::Shape),
        (ids::VECTOR_MODE_SELECT, DrawMode::Select),
    ] {
        Tool::handle_panel_event(&mut t, PanelEvent::Click(id));
        assert_eq!(t.mode(), want);
    }
    // Trocar de modo NAO e edicao de Style -> nunca marca recolor.
    assert!(!t.take_apply_to_selected());
}

/// **Gate do seam do catálogo:** o botão de CADA forma escolhe aquela forma E arma o
/// gesto (modo Shape). Uma forma nova entra no catálogo e este teste já a cobre —
/// nenhum botão pode nascer morto.
#[test]
fn every_catalog_button_selects_its_shape_and_arms_the_gesture() {
    let mut t = VectorTool::new();
    for (i, d) in crate::shapes::SHAPES.iter().enumerate() {
        Tool::handle_panel_event(&mut t, PanelEvent::Click(ids::vector_shape_id(i)));
        assert_eq!(t.shape(), d.kind, "botao {i} nao escolheu {:?}", d.kind);
        assert_eq!(t.mode(), DrawMode::Shape, "escolher a forma arma o desenho");
        assert_eq!(t.draw_config().shape, d.kind, "o cfg espelha a forma");
    }
}

/// **Gate do campo genérico:** o campo `i` escreve o parâmetro `i` da forma ATIVA,
/// clampado à faixa do catálogo — e os valores são POR-FORMA (mexer no raio da
/// estrela não mexe no do retângulo: é o "último usado" de cada uma).
#[test]
fn a_shape_field_writes_the_active_shapes_parameter_and_is_per_shape() {
    let mut t = VectorTool::new();
    t.set_shape(ShapeKind::Star);
    // Campo 0 da estrela = Points (contagem: clampa e arredonda).
    Tool::handle_panel_event(
        &mut t,
        PanelEvent::SetValue(ids::vector_shape_field_id(0), 7.4),
    );
    assert!(
        (t.draw_config().values[0] - 7.0).abs() < 1e-9,
        "7.4 pontas -> 7"
    );
    Tool::handle_panel_event(
        &mut t,
        PanelEvent::SetValue(ids::vector_shape_field_id(0), 9_999.0),
    );
    assert!(
        (t.draw_config().values[0] - 60.0).abs() < 1e-9,
        "clampa no teto"
    );
    // Campo 2 da estrela = raio da ponta (px).
    Tool::handle_panel_event(
        &mut t,
        PanelEvent::SetValue(ids::vector_shape_field_id(2), 30.0),
    );
    assert!((t.shape_values(ShapeKind::Star)[2] - 30.0).abs() < 1e-9);
    // O poligono NAO foi tocado (os valores sao por-forma).
    assert!(
        (t.shape_values(ShapeKind::Polygon)[1] - 0.0).abs() < 1e-9,
        "o raio do poligono ficou onde estava"
    );
}

#[test]
fn stroke_cap_join_dash_arms() {
    use crate::params::{DASH_MAX, GAP_MAX, StrokeCap, StrokeJoin};
    let mut t = VectorTool::new();
    Tool::handle_panel_event(&mut t, PanelEvent::Click(ids::VECTOR_CAP_ROUND));
    assert_eq!(t.cap(), StrokeCap::Round);
    assert!(
        t.take_apply_to_selected(),
        "cap change restyles the selection"
    );
    Tool::handle_panel_event(&mut t, PanelEvent::Click(ids::VECTOR_JOIN_BEVEL));
    assert_eq!(t.join(), StrokeJoin::Bevel);
    Tool::handle_panel_event(&mut t, PanelEvent::SetValue(ids::VECTOR_DASH, 1.0));
    assert!((t.dash() - DASH_MAX).abs() < 1e-6);
    assert!(t.take_apply_to_selected());
    Tool::handle_panel_event(&mut t, PanelEvent::SetValue(ids::VECTOR_GAP, 1.0));
    assert!((t.gap() - GAP_MAX).abs() < 1e-6);
    assert!(
        t.take_apply_to_selected(),
        "gap change restyles the selection"
    );
    // Snapshot carries them.
    let s = t.ui_snapshot();
    assert_eq!(s.cap, StrokeCap::Round);
    assert_eq!(s.join, StrokeJoin::Bevel);
    assert!((s.dash - DASH_MAX).abs() < 1e-6);
    assert!((s.gap - GAP_MAX).abs() < 1e-6);
}

/// **As pontas chegam ao Style, e voltam no snapshot.** O painel manda o
/// discriminante da ponta no id do chip; a tool a guarda e marca a seleção para
/// reestilizar (uma ponta é Style, como a cor — vale para o traço que está na tela).
#[test]
fn markers_reach_the_style_and_flag_the_selection() {
    let mut t = VectorTool::new();
    assert_eq!(t.marker_start(), Marker::None, "uma linha nasce sem ponta");
    assert_eq!(t.marker_end(), Marker::None);

    Tool::handle_panel_event(
        &mut t,
        PanelEvent::SetValue(
            ids::VECTOR_MARKER_END_DD,
            f64::from(Marker::Triangle.as_u8()),
        ),
    );
    assert_eq!(t.marker_end(), Marker::Triangle);
    assert_eq!(t.marker_start(), Marker::None, "o outro seletor nao mexeu");
    assert!(
        t.take_apply_to_selected(),
        "a ponta e Style: reestiliza a selecao"
    );

    Tool::handle_panel_event(
        &mut t,
        PanelEvent::SetValue(ids::VECTOR_MARKER_START_DD, f64::from(Marker::Bar.as_u8())),
    );
    assert_eq!(t.marker_start(), Marker::Bar);

    let s = t.ui_snapshot();
    assert_eq!(s.marker_start, Marker::Bar);
    assert_eq!(s.marker_end, Marker::Triangle);
}

/// TODA ponta do catálogo chega à tool pelo discriminante — uma ponta nova entra em
/// `ALL_MARKERS` e já está coberta. Um valor de fora da tabela (save/versão futura)
/// resolve para "sem ponta", nunca em pânico.
#[test]
fn every_marker_in_the_catalog_reaches_the_tool_and_junk_is_none() {
    let mut t = VectorTool::new();
    for &m in ph2d_vec_scene::ALL_MARKERS {
        Tool::handle_panel_event(
            &mut t,
            PanelEvent::SetValue(ids::VECTOR_MARKER_START_DD, f64::from(m.as_u8())),
        );
        assert_eq!(t.marker_start(), m, "{m:?} nao chegou na tool");
    }
    Tool::handle_panel_event(
        &mut t,
        PanelEvent::SetValue(ids::VECTOR_MARKER_START_DD, 250.0),
    );
    assert_eq!(t.marker_start(), Marker::None, "discriminante desconhecido");
}

/// Adotar as pontas do caminho SELECIONADO é leitura do documento: muda o Style (o
/// painel passa a mostrar as pontas daquele caminho) e **não** arma o restyle — se
/// armasse, selecionar um caminho o reescreveria.
#[test]
fn adopting_markers_does_not_arm_a_restyle() {
    let mut t = VectorTool::new();
    t.adopt_markers(Marker::Circle, Marker::Open);
    assert_eq!(t.marker_start(), Marker::Circle);
    assert_eq!(t.marker_end(), Marker::Open);
    assert!(
        !t.take_apply_to_selected(),
        "selecionar nao pode reescrever o caminho"
    );
}

#[test]
fn ui_snapshot_round_trips_style() {
    let mut t = VectorTool::new();
    t.set_stroke_rgba([1, 2, 3, 255]);
    t.set_fill_rgba([4, 5, 6, 255]);
    Tool::handle_panel_event(&mut t, PanelEvent::SetValue(ids::VECTOR_WIDTH, 0.5));
    t.set_shape(ShapeKind::Polygon);
    Tool::handle_panel_event(
        &mut t,
        PanelEvent::SetValue(ids::vector_shape_field_id(0), 7.0),
    );
    let s = t.ui_snapshot();
    assert_eq!(s.stroke, [1, 2, 3, 255]);
    assert_eq!(s.fill, [4, 5, 6, 255]);
    assert_eq!(s.stroke_width_px, t.stroke_width_px());
    assert_eq!(s.mode, DrawMode::Shape, "escolher a forma arma o desenho");
    assert_eq!(s.shape, ShapeKind::Polygon);
    assert!(
        (s.values[0] - 7.0).abs() < 1e-9,
        "o snapshot leva os valores"
    );
}

#[test]
fn empty_panel_has_no_controls() {
    let t = VectorTool::new();
    let panel = t.build_panel();
    assert!(panel.controls.is_empty());
}

#[test]
fn id_label_icon_stable() {
    let t = VectorTool::new();
    assert_eq!(t.id(), ToolId::new("vector"));
    assert_eq!(t.label(), "Vector");
    assert_eq!(t.icon_slug(), "vector");
}
