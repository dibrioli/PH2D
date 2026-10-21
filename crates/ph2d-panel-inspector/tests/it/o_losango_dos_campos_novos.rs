//! ⭐ **O losango nos campos do Inspector que ganharam canal** (plano 35 §3.0, famílias OBJETO e
//! JUNTA): o skew, a emissão, o tint, o self tint, os cantos, a silhueta, e da junta o motor, a
//! quebra, os limites, a mola e os eixos de um Custom — cada linha pinta o losango na coluna do
//! decorator e regista o alvo do clique. A shell publica o mapa (`marcas_inspector`).

use std::collections::BTreeMap;

use ph2d_a11y::NodeId;
use ph2d_editor_core::screens::hero::{
    InspectorJointInfo, InspectorSpriteInfo, InspectorSpriteMixed, InspectorSpriteSource,
    InspectorTransformInfo,
};
use ph2d_editor_core::widget::key_mark::{
    KeyMark, key_mark_id, set_key_marks, take_key_mark_click,
};
use ph2d_editor_core::zones::Rect;
use ph2d_host::{PointerButton, PointerEvent, PointerKind, PointerSource};
use ph2d_panel_inspector::ids as insp;
use ph2d_panel_inspector::{
    InspectorPanel, InspectorState, set_current_inspector_joint, set_current_inspector_sprite,
    set_current_inspector_transform,
};
use ph2d_ui_testkit::MockPanelHost;

const ENTITY: u64 = 0x7E57_0099;
const VIEWPORT: Rect = Rect {
    x: 0.0,
    y: 0.0,
    w: 320.0,
    h: 4000.0,
};

fn sprite() -> InspectorSpriteInfo {
    InspectorSpriteInfo {
        emissive: 2.0,
        entity_bits: ENTITY,
        world_size: [1.0, 1.0],
        source_kind: InspectorSpriteSource::Atlas { key: 3 },
        source_precision: Some(ph2d_editor_core::Precision::Rgba8),
        sheet_label: None,
        source_pixels: Some((256, 256)),
        can_reimport: true,
        flip_x: false,
        flip_y: false,
        opacity: 1.0,
        tint_fill: false,
        hframes: 1,
        vframes: 1,
        frame: 0,
        tint: [1.0; 4],
        self_tint: [1.0; 4],
        per_corner_tint: [[1.0; 4]; 4],
        region_enabled: false,
        region_rect: [0.0; 4],
        region_filter_clip: true,
        centered: true,
        offset: [0.0, 0.0],
        selected_count: 1,
        mixed: InspectorSpriteMixed::default(),
    }
}

/// Uma junta do tipo `kind_tag` com todos os interruptores LIGADOS (cada linha na tela), e o eixo X
/// de um Custom em `Limited` (o par min/max só existe nesse modo).
fn junta(kind_tag: u8) -> InspectorJointInfo {
    InspectorJointInfo {
        entity_bits: ENTITY,
        kind_tag,
        body_a_name: "Hook".into(),
        body_b_name: "Plank".into(),
        bound: true,
        world_anchored: false,
        limits_enabled: true,
        limit_min_ui: -45.0,
        limit_max_ui: 45.0,
        motor_enabled: true,
        motor_mode_tag: 0,
        motor_speed_ui: 1.0,
        motor_target_ui: 0.0,
        motor_max_force: 10.0,
        rest_length: 1.0,
        stiffness: 30.0,
        damping: 0.5,
        max_length: 2.0,
        soft: true,
        pick_armed: 0,
        wheel_count: 0,
        break_enabled: true,
        break_force: 100.0,
        break_torque: 50.0,
        breaks_on_torque: kind_tag == 0,
        active: true,
        collide_connected: false,
        paste_targets: 0,
        axis_mode_tag: [1, 0, 2],
        axis_min_ui: [-1.0, -1.0, -45.0],
        axis_max_ui: [1.0, 1.0, 45.0],
        motor_axis_tag: 0,
    }
}

/// Pinta com `marcadas` em `Chave` e devolve os alvos de losango registados.
fn losangos(host: &mut MockPanelHost, marcadas: &[NodeId]) -> Vec<(NodeId, Rect)> {
    set_key_marks(marcadas.iter().map(|&id| (id, KeyMark::Chave)).collect());
    let mut st = InspectorState::default();
    host.settle_section_folds();
    host.paint::<InspectorPanel>(&mut st, VIEWPORT)
}

#[track_caller]
fn tem(rects: &[(NodeId, Rect)], primeiro: NodeId, o_que: &str) {
    assert!(
        rects.iter().any(|(id, _)| *id == key_mark_id(primeiro)),
        "a linha {o_que} não pintou o losango (ou não registou o alvo)"
    );
}

/// ⭐ Skew, emissão, tint, self tint, cantos e silhueta pintam o losango; o clique no dos cantos
/// pede a chave dos QUATRO (um grupo). (Mutação: o `paint_decorator_dot` de volta nos cantos ⇒ sem
/// alvo; a silhueta sem a coluna ⇒ sem alvo.)
#[test]
fn os_campos_da_sprite_pintam_o_losango() {
    ph2d_editor_core::paint::set_ui_look(ph2d_tokens::UiLook::Redesign);
    set_current_inspector_transform(Some(InspectorTransformInfo {
        entity_bits: ENTITY,
        translation: [0.0, 0.0],
        rotation_rad: 0.0,
        scale: [1.0, 1.0],
        skew_rad: [0.1, 0.0],
    }));
    set_current_inspector_sprite(Some(sprite()));
    let linhas = [
        (insp::INSP_TRANSFORM_SKEW_X, "Skew"),
        (insp::INSP_SPRITE_EMISSIVE, "Emissive"),
        (insp::INSP_SPRITE_TINT_SWATCH, "Tint"),
        (insp::INSP_SPRITE_SELF_TINT_SWATCH, "Self Tint"),
        (insp::INSP_SPRITE_CORNER_TL, "Per-Corner"),
        (insp::INSP_SPRITE_TINT_FILL, "Tint Fill"),
    ];
    let ids: Vec<NodeId> = linhas.iter().map(|&(id, _)| id).collect();
    let mut host = MockPanelHost::with_panel::<InspectorPanel>();
    let rects = losangos(&mut host, &ids);
    for (id, o_que) in linhas {
        tem(&rects, id, o_que);
    }
    let r = rects
        .iter()
        .find(|(id, _)| *id == key_mark_id(insp::INSP_SPRITE_CORNER_TL))
        .map(|(_, r)| *r)
        .expect("o losango dos cantos");
    let _ = take_key_mark_click();
    let ev = |kind, t| PointerEvent {
        kind,
        x: r.x + r.w * 0.5,
        y: r.y + r.h * 0.5,
        button: PointerButton::Primary,
        source: PointerSource::Mouse,
        pressure: 1.0,
        timestamp_ns: t,
    };
    let _ = host.dispatch_pointer_event(ev(PointerKind::Down, 1_000_000_000));
    let _ = host.dispatch_pointer_event(ev(PointerKind::Up, 1_010_000_000));
    let c = take_key_mark_click().expect("o clique no losango dos cantos não chegou à shell");
    assert_eq!(
        c.fields,
        vec![
            insp::INSP_SPRITE_CORNER_TL,
            insp::INSP_SPRITE_CORNER_TR,
            insp::INSP_SPRITE_CORNER_BL,
            insp::INSP_SPRITE_CORNER_BR,
        ]
    );
    set_key_marks(BTreeMap::new());
    set_current_inspector_sprite(None);
    set_current_inspector_transform(None);
}

/// ⭐ Motor (interruptor, modo, força), quebra (interruptor, força, torque), limites (interruptor,
/// min, max) num Pin; a solda mole e a mola numa Weld; os eixos e o eixo do motor num Custom.
#[test]
fn os_campos_da_junta_pintam_o_losango() {
    ph2d_editor_core::paint::set_ui_look(ph2d_tokens::UiLook::Redesign);
    let casos: [(u8, &[(NodeId, &str)]); 3] = [
        (
            0,
            &[
                (insp::INSP_JOINT_MOTOR[0], "Motor"),
                (insp::INSP_JOINT_MOTOR_MODE[0], "Mode"),
                (insp::INSP_JOINT_MOTOR_FORCE, "Max Force"),
                (insp::INSP_JOINT_BREAK[0], "Breakable"),
                (insp::INSP_JOINT_BREAK_FORCE, "Break Force"),
                (insp::INSP_JOINT_BREAK_TORQUE, "Break Torque"),
                (insp::INSP_JOINT_LIMITS[0], "Limits"),
                (insp::INSP_JOINT_LIMIT_MIN, "Min"),
                (insp::INSP_JOINT_LIMIT_MAX, "Max"),
            ],
        ),
        (
            3,
            &[
                (insp::INSP_JOINT_SOFT[0], "Weld"),
                (insp::INSP_JOINT_STIFFNESS, "Stiffness"),
                (insp::INSP_JOINT_DAMPING, "Damping"),
            ],
        ),
        (
            8,
            &[
                (insp::INSP_JOINT_AXIS_MODE[0][0], "Axis X"),
                (insp::INSP_JOINT_AXIS_MIN[0], "Axis X Min"),
                (insp::INSP_JOINT_AXIS_MAX[0], "Axis X Max"),
                (insp::INSP_JOINT_AXIS_MODE[2][0], "Rotation"),
                (insp::INSP_JOINT_MOTOR_AXIS[0], "Motor Axis"),
            ],
        ),
    ];
    for (kind, linhas) in casos {
        set_current_inspector_joint(Some(junta(kind)));
        let ids: Vec<NodeId> = linhas.iter().map(|&(id, _)| id).collect();
        let mut host = MockPanelHost::with_panel::<InspectorPanel>();
        let rects = losangos(&mut host, &ids);
        for &(id, o_que) in linhas {
            tem(&rects, id, o_que);
        }
    }
    set_key_marks(BTreeMap::new());
    set_current_inspector_joint(None);
}
