//! Os `NodeId` de widget do painel da timeline, e a tabela dos botões «+Track».
//!
//! ⚠️ **Uma definição e ZERO re-exportações** (auditoria A5b, 2026-09-12). Os ids que só este painel
//! lê moram aqui (`timeline.rs`, `menus_timeline.rs`); os que a `ph2d-editor-core` também lê (o corpo
//! do painel, que o *z-order walk* percorre, e o que o despacho compara) ficaram lá, e quem os usa
//! nomeia `ph2d_editor_core::ids::X`. Até aqui este módulo re-exportava a
//! `ph2d-editor-core::ids::chrome::timeline` inteira «para o despacho e a pintura partilharem os
//! ids» — com uma definição só, partilham-na sem re-exportação nenhuma.

/// The "+Track" property buttons paired with their [`ph2d_timeline::PropKind`]:
/// the six scene properties in `PropKind::ALL` order, then **Time Remap** (the
/// per-object clock — not a pose prop, so it lives outside `ALL`). Used by
/// paint (the label comes from `prop_label`, the single i18n source) + the
/// hit/populate wiring (which ignores the second field).
///
/// **Position** fecha a lista (ADR-0141): é o modo de TRAJETÓRIA, a alternativa a
/// TranslationX + TranslationY, e por isso fica fora de `ALL` como o Time Remap. Entra
/// no FIM porque a ordem desta tabela é a ordem que o artista lê, e as seis da pose são
/// o que ele procura primeiro.
pub const ADDPROP_BUTTONS: [(ph2d_a11y::NodeId, ph2d_timeline::PropKind); 18] = [
    (TIMELINE_ADDPROP_TX, ph2d_timeline::PropKind::TranslationX),
    (TIMELINE_ADDPROP_TY, ph2d_timeline::PropKind::TranslationY),
    (TIMELINE_ADDPROP_ROT, ph2d_timeline::PropKind::Rotation),
    (TIMELINE_ADDPROP_SX, ph2d_timeline::PropKind::ScaleX),
    (TIMELINE_ADDPROP_SY, ph2d_timeline::PropKind::ScaleY),
    (TIMELINE_ADDPROP_OPACITY, ph2d_timeline::PropKind::Opacity),
    (TIMELINE_ADDPROP_TIME, ph2d_timeline::PropKind::TimeRemap),
    (TIMELINE_ADDPROP_POS, ph2d_timeline::PropKind::Position),
    (TIMELINE_ADDPROP_MORPH, ph2d_timeline::PropKind::Morph),
    (
        TIMELINE_ADDPROP_JOINT_MOTOR_TARGET,
        ph2d_timeline::PropKind::JointMotorTarget,
    ),
    (
        TIMELINE_ADDPROP_JOINT_MOTOR_SPEED,
        ph2d_timeline::PropKind::JointMotorSpeed,
    ),
    (
        TIMELINE_ADDPROP_JOINT_REST_LENGTH,
        ph2d_timeline::PropKind::JointRestLength,
    ),
    (
        TIMELINE_ADDPROP_JOINT_MAX_LENGTH,
        ph2d_timeline::PropKind::JointMaxLength,
    ),
    (
        TIMELINE_ADDPROP_BONE_BEND_IN_X,
        ph2d_timeline::PropKind::BoneBendInX,
    ),
    (
        TIMELINE_ADDPROP_BONE_BEND_IN_Y,
        ph2d_timeline::PropKind::BoneBendInY,
    ),
    (
        TIMELINE_ADDPROP_BONE_BEND_OUT_X,
        ph2d_timeline::PropKind::BoneBendOutX,
    ),
    (
        TIMELINE_ADDPROP_BONE_BEND_OUT_Y,
        ph2d_timeline::PropKind::BoneBendOutY,
    ),
    (
        TIMELINE_ADDPROP_IK_BEND_SIDE,
        ph2d_timeline::PropKind::IkBendSide,
    ),
];

mod menus_timeline;
pub use menus_timeline::*;
mod timeline;
pub use timeline::*;
