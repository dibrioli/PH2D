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
        (ids::VECTOR_MODE_PENCIL, DrawMode::Pencil),
        (ids::VECTOR_MODE_NODE, DrawMode::Node),
        (ids::VECTOR_MODE_TEXT, DrawMode::Text),
        // O 5º pill: sem ele, desenhar uma forma deixava a fileira toda apagada.
        (ids::VECTOR_MODE_SHAPE, DrawMode::Shape),
        // O 8º pill (Pick Shapes / Blend): coleta formas na ordem de clique.
        (ids::VECTOR_MODE_PICKBLEND, DrawMode::PickBlend),
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
    t.adopt_markers(
        Marker::Circle,
        Marker::Open,
        crate::params::DEFAULT_MARKER_SCALE,
        crate::params::DEFAULT_MARKER_ROUND,
    );
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

/// **O Head Size e o Head Round chegam à tool** (e saturam na faixa que o painel registra na
/// caixa) — e marcam a seleção para reestilizar: são Style, como as pontas, então valem para
/// o traço que está na TELA, não só para o próximo desenhado.
#[test]
fn the_head_size_and_round_reach_the_tool_and_clamp_to_their_range() {
    let mut t = VectorTool::new();
    assert!(
        !t.take_apply_to_selected(),
        "a tool nova nasce sem restyle pendente"
    );
    Tool::handle_panel_event(&mut t, PanelEvent::SetValue(ids::VECTOR_MARKER_SCALE, 2.5));
    assert!((t.marker_scale() - 2.5).abs() < 1e-9);
    assert!(
        t.take_apply_to_selected(),
        "mudar o tamanho da cabeca tem de reestilizar o traco SELECIONADO — senao o numero \
         muda no painel e a seta continua igual na tela"
    );

    Tool::handle_panel_event(&mut t, PanelEvent::SetValue(ids::VECTOR_MARKER_ROUND, 0.75));
    assert!((t.marker_round() - 0.75).abs() < 1e-9);
    assert!(t.take_apply_to_selected());

    // A faixa satura nos dois extremos (o mesmo clamp que o `set_number_range` da caixa).
    Tool::handle_panel_event(
        &mut t,
        PanelEvent::SetValue(ids::VECTOR_MARKER_SCALE, 999.0),
    );
    assert!((t.marker_scale() - crate::params::MARKER_SCALE.max).abs() < 1e-9);
    Tool::handle_panel_event(&mut t, PanelEvent::SetValue(ids::VECTOR_MARKER_SCALE, -5.0));
    assert!(
        (t.marker_scale() - crate::params::MARKER_SCALE.min).abs() < 1e-9,
        "uma cabeca de tamanho zero (ou negativo) e uma seta invisivel"
    );
    Tool::handle_panel_event(&mut t, PanelEvent::SetValue(ids::VECTOR_MARKER_ROUND, 9.0));
    assert!((t.marker_round() - crate::params::MARKER_ROUND.max).abs() < 1e-9);
}

/// **A dupla via alterna nos DOIS sentidos, e o estado é derivado.**
///
/// Sem flag próprio: `both_ends()` é `start != None && end != None`. O gate percorre o ciclo
/// inteiro — inclusive o caso que o Enio nomeou (as duas pontas `None`), em que o botão
/// precisa NASCER uma seta, senão ele "acende" sem desenhar nada.
#[test]
fn both_ends_toggles_in_both_directions_and_stays_derived() {
    let mut t = VectorTool::new();
    let click = |t: &mut VectorTool| {
        Tool::handle_panel_event(t, PanelEvent::Click(ids::VECTOR_MARKER_BOTH));
    };

    // 1. Nasce sem ponta nenhuma ⇒ desligado.
    assert_eq!(t.marker_start(), Marker::None);
    assert_eq!(t.marker_end(), Marker::None);
    assert!(!t.both_ends());

    // 2. Desligado + as DUAS vazias ⇒ acende a seta canônica nos dois extremos.
    click(&mut t);
    assert!(t.both_ends(), "o clique tinha de ligar a dupla via");
    assert_eq!(t.marker_start(), crate::params::DEFAULT_BOTH_ENDS_MARKER);
    assert_eq!(t.marker_end(), crate::params::DEFAULT_BOTH_ENDS_MARKER);
    assert!(
        t.take_apply_to_selected(),
        "a dupla via e Style: tem de reestilizar o traco selecionado"
    );

    // 3. Ligado ⇒ volta a VIA ÚNICA: limpa o COMEÇO (a seta fica no fim, que e o que "uma
    //    via" significa num diagrama).
    click(&mut t);
    assert!(!t.both_ends());
    assert_eq!(t.marker_start(), Marker::None, "a via unica limpa o COMECO");
    assert_eq!(
        t.marker_end(),
        crate::params::DEFAULT_BOTH_ENDS_MARKER,
        "a ponta do FIM sobrevive — senao a linha perderia a seta inteira"
    );

    // 4. Desligado com uma ponta ESCOLHIDA no fim ⇒ ela e copiada para o comeco (o fim manda;
    //    o usuario nao perde o losango que escolheu).
    Tool::handle_panel_event(
        &mut t,
        PanelEvent::SetValue(
            ids::VECTOR_MARKER_END_DD,
            f64::from(Marker::Diamond.as_u8()),
        ),
    );
    assert!(!t.both_ends());
    click(&mut t);
    assert!(t.both_ends());
    assert_eq!(t.marker_start(), Marker::Diamond);
    assert_eq!(t.marker_end(), Marker::Diamond);

    // 5. E o snapshot que o painel pinta concorda com a tool (uma verdade so).
    assert!(t.ui_snapshot().both_ends());
    click(&mut t);
    assert!(!t.ui_snapshot().both_ends());
}

/// Só o COMEÇO tem ponta (o usuário escolheu no chip de Start): ligar a dupla via espelha
/// ESSA ponta no fim, em vez de clobberá-la com a seta default.
#[test]
fn both_ends_mirrors_a_lone_start_marker_instead_of_clobbering_it() {
    let mut t = VectorTool::new();
    Tool::handle_panel_event(
        &mut t,
        PanelEvent::SetValue(ids::VECTOR_MARKER_START_DD, f64::from(Marker::Bar.as_u8())),
    );
    assert!(!t.both_ends());
    Tool::handle_panel_event(&mut t, PanelEvent::Click(ids::VECTOR_MARKER_BOTH));
    assert!(t.both_ends());
    assert_eq!(t.marker_start(), Marker::Bar);
    assert_eq!(
        t.marker_end(),
        Marker::Bar,
        "a ponta escolhida foi espelhada"
    );
}

/// **Adotar as pontas do caminho selecionado leva TAMBÉM o tamanho e o arredondamento** — o
/// painel pinta a partir da tool, então sem isso os campos mostrariam o último valor autorado
/// em vez do do traço que está na tela. E adotar é LER: não pode marcar `apply_to_selected`
/// (o próprio ato de selecionar reescreveria o caminho).
#[test]
fn adopting_a_paths_markers_also_adopts_its_head_size_and_round() {
    let mut t = VectorTool::new();
    let _ = t.take_apply_to_selected();
    t.adopt_markers(Marker::None, Marker::Triangle, 3.0, 0.5);
    assert_eq!(t.marker_end(), Marker::Triangle);
    assert!((t.marker_scale() - 3.0).abs() < 1e-9);
    assert!((t.marker_round() - 0.5).abs() < 1e-9);
    assert!(
        !t.take_apply_to_selected(),
        "adotar e LER o documento — se marcasse, selecionar um caminho ja o reescreveria"
    );
    // Um save corrompido (escala absurda) e clampado na adocao, nao propagado.
    t.adopt_markers(Marker::None, Marker::Triangle, 1e9, -1.0);
    assert!((t.marker_scale() - crate::params::MARKER_SCALE.max).abs() < 1e-9);
    assert!((t.marker_round() - crate::params::MARKER_ROUND.min).abs() < 1e-9);
}

/// **Cada chip da FONTE de largura escolhe a sua, e ela chega ao espelho que a shell lê** (W1d).
///
/// O oráculo é o `draw_config()` — o que o laço de frame de facto consome — e não o campo
/// privado: um gate que lesse o campo provaria que a atribuição aconteceu, não que ela chega a
/// quem desenha.
#[test]
fn each_width_source_chip_reaches_the_draw_config() {
    use ph2d_vec_edit::pencil_width::WidthSource as Ws;
    let mut t = VectorTool::default();
    assert_eq!(
        t.draw_config().pencil_width_source,
        Ws::Uniform,
        "o default tem de ser a fonte que não inventa geometria nenhuma"
    );
    for (id, want) in [
        (ids::VECTOR_PENCIL_W_SPEED, Ws::Speed),
        (ids::VECTOR_PENCIL_W_PRESSURE, Ws::Pressure),
        (ids::VECTOR_PENCIL_W_UNIFORM, Ws::Uniform),
    ] {
        Tool::handle_panel_event(&mut t, PanelEvent::Click(id));
        assert_eq!(t.draw_config().pencil_width_source, want, "chip {id:?}");
        assert_eq!(
            t.ui_snapshot().pencil_width_source,
            want,
            "o painel pinta a partir do snapshot — ele tem de concordar com o config"
        );
    }
}

/// **Escolher uma fonte NÃO arranca o artista do modo em que ele está.** A seção só é pintada no
/// Pencil, então já se está lá; e um chip que trocasse o modo tornaria impossível ajustar a fonte
/// vindo de qualquer outro caminho.
#[test]
fn picking_a_width_source_does_not_change_the_draw_mode() {
    let mut t = VectorTool::default();
    Tool::handle_panel_event(&mut t, PanelEvent::Click(ids::VECTOR_MODE_NODE));
    let before = t.draw_config().mode;
    Tool::handle_panel_event(&mut t, PanelEvent::Click(ids::VECTOR_PENCIL_W_SPEED));
    assert_eq!(t.draw_config().mode, before);
}
