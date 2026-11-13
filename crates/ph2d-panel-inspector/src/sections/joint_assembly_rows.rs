//! **O cartão Joint fala pelo papel e pela unidade** (Joint Editor W2): o título «Front wheel · Car 1»,
//! a caixa «Same as …» que edita o grupo da montagem, a caixa «Solve as one chain» do Pin, e os
//! rótulos que mudam com o tipo (a Wheel diz Suspension, Travel down/up, Drive, Spin speed).
//!
//! Caixas e nunca toggles (dono 10/11). Cada caixa só aparece onde o leitor a decide: «Same as» só
//! numa junta com par no grupo; «one chain» só num Pin sem motor e não quebrável (o motor da física
//! ignora o pedido nos outros — `ph2d_physics::world::chains::takes_one_chain`).

use super::KIND_PIN;
use super::KIND_WHEEL;
use super::super::rows::caixa_row;
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
