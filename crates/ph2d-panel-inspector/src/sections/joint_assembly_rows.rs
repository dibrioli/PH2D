//! **O cartão Joint fala pelo papel e pela unidade** (Joint Editor W2): o título «Front wheel · Car 1»,
//! a caixa «Same as …» que edita o grupo da montagem, a caixa «Solve as one chain» do Pin, e os
//! rótulos que mudam com o tipo (a Wheel diz Suspension, Travel down/up, Drive, Spin speed).
//!
//! Caixas e nunca toggles (dono 10/11). Cada caixa só aparece onde o leitor a decide: «Same as» só
//! numa junta com par no grupo; «one chain» só num Pin sem motor e não quebrável (o motor da física
//! ignora o pedido nos outros — `ph2d_physics::world::chains::takes_one_chain`).

use super::super::rows::caixa_row;
use super::KIND_PIN;
use super::KIND_WHEEL;
use super::*;
use ph2d_editor_core::screens::hero::InspectorJointInfo;
use ph2d_editor_core::widget::key_mark::Rotulo;
use ph2d_i18n::{TextKey, tr, tr_with};

/// O título do cartão: o papel na montagem, ou «Joint» numa junta avulsa.
pub(super) fn title(info: &InspectorJointInfo) -> &str {
    if info.title.is_empty() {
        tr("component.physics_joint.name")
    } else {
        &info.title
    }
}

fn caixa(on: bool) -> CheckboxValue {
    if on {
        CheckboxValue::Checked
    } else {
        CheckboxValue::Unchecked
    }
}

/// «Same as …» — só quando a junta tem par no grupo. Pinta-se logo abaixo do Active: ela diz se
/// tudo o que vem abaixo vale só para esta junta ou para o grupo.
#[allow(clippy::too_many_arguments)]
pub(super) fn paint_linked_row(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    x: f32,
    w: f32,
    y: f32,
    info: &InspectorJointInfo,
) -> f32 {
    if info.same_as.is_empty() {
        return y;
    }
    let label = tr_with("panel.inspector.joint.same_as", &[("names", &info.same_as)]);
    let sec = ph2d_editor_core::property_row::Seccao::medida(text_system, 1, &[label.as_str()]);
    caixa_row(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        x,
        w,
        y,
        (ids::INSP_JOINT_LINKED, label.as_str(), caixa(info.linked)),
        sec,
    )
}

/// A caixa existe? Pin sem motor e não quebrável — o mesmo que o motor da física aceita.
#[must_use]
pub(super) fn offers_one_chain(info: &InspectorJointInfo) -> bool {
    info.kind_tag == KIND_PIN && !info.motor_enabled && !info.break_enabled
}

/// «Solve as one chain» — depois dos parâmetros do Pin.
#[allow(clippy::too_many_arguments)]
pub(super) fn paint_one_chain_row(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    x: f32,
    w: f32,
    y: f32,
    info: &InspectorJointInfo,
) -> f32 {
    if !offers_one_chain(info) {
        return y;
    }
    let label = tr("component.field.joint.28");
    let sec = ph2d_editor_core::property_row::Seccao::medida(text_system, 1, &[label]);
    caixa_row(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        x,
        w,
        y,
        (ids::INSP_JOINT_ONE_CHAIN, label, caixa(info.one_chain)),
        sec,
    )
}

/// Os rótulos que o TIPO muda — a Wheel é uma suspensão e um motor de roda.
pub(super) fn limit_min_label(kind_tag: u8) -> Rotulo<'static> {
    if kind_tag == KIND_WHEEL {
        tr("panel.inspector.joint.travel_down").into()
    } else {
        TextKey::new("component.field.joint.5").into()
    }
}

pub(super) fn limit_max_label(kind_tag: u8) -> Rotulo<'static> {
    if kind_tag == KIND_WHEEL {
        tr("panel.inspector.joint.travel_up").into()
    } else {
        TextKey::new("component.field.joint.6").into()
    }
}

pub(super) fn stiffness_label(kind_tag: u8) -> Rotulo<'static> {
    if kind_tag == KIND_WHEEL {
        tr("panel.inspector.joint.suspension_stiffness").into()
    } else {
        TextKey::new("component.field.joint.11").into()
    }
}

pub(super) fn damping_label(kind_tag: u8) -> Rotulo<'static> {
    if kind_tag == KIND_WHEEL {
        tr("panel.inspector.joint.suspension_damping").into()
    } else {
        TextKey::new("component.field.joint.12").into()
    }
}

pub(super) fn motor_label(kind_tag: u8) -> Rotulo<'static> {
    if kind_tag == KIND_WHEEL {
        tr("panel.inspector.joint.drive").into()
    } else {
        TextKey::new("component.field.joint.7").into()
    }
}

pub(super) fn speed_label(kind_tag: u8) -> Rotulo<'static> {
    if kind_tag == KIND_WHEEL {
        tr("panel.inspector.joint.spin_speed").into()
    } else {
        TextKey::new("component.field.joint.8").into()
    }
}

/// O texto pintado não é observável pelo `MockPanelHost`; os rótulos e o título são estas fns, e o
/// pintor da §12 chama-as (`joint_kind_rows`, `joint_cards`).
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_wheel_says_suspension_travel_and_drive_and_the_others_keep_theirs() {
        let wheel = [
            limit_min_label(KIND_WHEEL).texto,
            limit_max_label(KIND_WHEEL).texto,
            stiffness_label(KIND_WHEEL).texto,
            damping_label(KIND_WHEEL).texto,
            motor_label(KIND_WHEEL).texto,
            speed_label(KIND_WHEEL).texto,
        ];
        assert_eq!(
            wheel,
            [
                "Travel down",
                "Travel up",
                "Suspension stiffness",
                "Suspension damping",
                "Drive",
                "Spin speed"
            ]
        );
        for plain in [
            tr("component.field.joint.11"),
            tr("component.field.joint.5"),
        ] {
            assert!(!wheel.contains(&plain), "a wheel still says {plain:?}");
        }
        assert_eq!(
            stiffness_label(KIND_PIN).texto,
            tr("component.field.joint.11")
        );
        assert_eq!(
            limit_min_label(KIND_PIN).texto,
            tr("component.field.joint.5")
        );
    }

    #[test]
    fn the_card_title_is_the_role_or_joint() {
        let mut info = InspectorJointInfo {
            entity_bits: 1,
            kind_tag: KIND_WHEEL,
            body_a_name: String::new(),
            body_b_name: String::new(),
            bound: true,
            world_anchored: false,
            limits_enabled: false,
            limit_min_ui: 0.0,
            limit_max_ui: 0.0,
            motor_enabled: false,
            motor_mode_tag: 0,
            motor_speed_ui: 0.0,
            motor_target_ui: 0.0,
            motor_max_force: 0.0,
            rest_length: 0.0,
            stiffness: 0.0,
            damping: 0.0,
            max_length: 0.0,
            soft: false,
            pick_armed: 0,
            wheel_count: 0,
            break_enabled: false,
            break_force: 0.0,
            break_torque: 0.0,
            breaks_on_torque: false,
            active: true,
            collide_connected: false,
            paste_targets: 0,
            title: "Front wheel · Car 1".into(),
            one_chain: false,
            same_as: String::new(),
            linked: false,
            axis_mode_tag: [0; 3],
            axis_min_ui: [0.0; 3],
            axis_max_ui: [0.0; 3],
            motor_axis_tag: 0,
        };
        assert_eq!(title(&info), "Front wheel · Car 1");
        info.title.clear();
        assert_eq!(title(&info), "Joint");
    }
}
