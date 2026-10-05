//! Os gates da ferramenta de osso — vieram do `tool_weight_tests.rs` da `ph2d-tool-vector` (A14),
//! mais o verbo.

use super::*;

fn clica(t: &mut BoneTool, id: ph2d_a11y::NodeId) {
    Tool::handle_panel_event(t, PanelEvent::Click(id));
}

/// ⭐ Os três segmentos armam o verbo; controlo: nasce em *Create*.
#[test]
fn the_three_segments_arm_the_verb() {
    let mut t = BoneTool::default();
    assert_eq!(
        t.config().action,
        BoneAction::Create,
        "controlo: o nascimento"
    );
    for (id, verbo) in [
        (ids::VECTOR_BONE_ACT_WEIGHT, BoneAction::Weight),
        (ids::VECTOR_BONE_ACT_TRANSFORM, BoneAction::Transform),
        (ids::VECTOR_BONE_ACT_CREATE, BoneAction::Create),
    ] {
        clica(&mut t, id);
        assert_eq!(
            t.config().action,
            verbo,
            "{verbo:?} não chegou à ferramenta"
        );
    }
}

/// ⭐ Os dois lados chegam à ferramenta e NÃO armam o verbo.
#[test]
fn the_two_sides_reach_the_tool_and_do_not_arm_the_verb() {
    for (id, esperado) in [
        (ids::VECTOR_BONE_WEIGHT_SUB, WeightDirection::Subtract),
        (ids::VECTOR_BONE_WEIGHT_ADD, WeightDirection::Add),
    ] {
        let mut t = BoneTool::default();
        clica(&mut t, ids::VECTOR_BONE_ACT_TRANSFORM);
        clica(&mut t, id);
        assert_eq!(t.config().weight_direction, esperado);
        assert_eq!(
            t.config().action,
            BoneAction::Transform,
            "escolher o lado do pincel ARRANCOU o artista do verbo em que ele estava"
        );
    }
    let mut t = BoneTool::default();
    clica(&mut t, ids::VECTOR_BONE_WEIGHT_ABS);
    assert_eq!(t.config().weight_mode, WeightMode::Absolute);
    clica(&mut t, ids::VECTOR_BONE_WEIGHT_CUMUL);
    assert_eq!(t.config().weight_mode, WeightMode::Cumulative);
}

/// ⭐ A força é uma MAGNITUDE (um negativo não a zera, o tecto é `1`) e o raio tem piso.
#[test]
fn the_strength_is_a_magnitude_and_the_radius_has_a_floor() {
    let mut t = BoneTool::default();
    Tool::handle_panel_event(
        &mut t,
        PanelEvent::SetValue(ids::VECTOR_BONE_WEIGHT_AMOUNT, -0.4),
    );
    assert!((t.config().weight_amount - 0.4).abs() < 1e-12);
    Tool::handle_panel_event(
        &mut t,
        PanelEvent::SetValue(ids::VECTOR_BONE_WEIGHT_AMOUNT, 3.0),
    );
    assert!((t.config().weight_amount - 1.0).abs() < 1e-12);
    assert_eq!(
        t.config().weight_direction,
        WeightDirection::Add,
        "o campo do número mexeu na DIRECÇÃO"
    );
    Tool::handle_panel_event(
        &mut t,
        PanelEvent::SetValue(ids::VECTOR_BONE_WEIGHT_RADIUS, 0.0),
    );
    assert!((t.config().weight_radius - WEIGHT_RADIUS_MIN).abs() < 1e-12);
}
