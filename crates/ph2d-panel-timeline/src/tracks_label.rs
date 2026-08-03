//! **Como este app NOMEIA um canal animado** — `prop_label` e `track_label`.
//!
//! Módulo irmão do `tracks.rs` pelo cap de 600 LOC, cortado pelo assunto que os
//! dois já separavam: *como o dope-sheet DESENHA* × *que palavra ele mostra*.
//!
//! ⚠️ **`track_label` é porta ÚNICA, e a razão é literal:** o card de Expressão
//! nasceu montando esta string por conta própria, e o doc-comment dele prometia
//! `"Ball · Position Y"` desde o primeiro dia enquanto o código escrevia
//! `#nnnn`. A row e o card do mesmo canal dizendo coisas diferentes é
//! exactamente o que uma segunda cópia produz.

use ph2d_timeline::PropKind;

/// The display label for a property (the panel's presentation of `PropKind`).
pub(crate) fn prop_label(p: PropKind) -> &'static str {
    match p {
        PropKind::TranslationX => ph2d_i18n::tr("panel.timeline.prop.translate_x"),
        PropKind::TranslationY => ph2d_i18n::tr("panel.timeline.prop.translate_y"),
        PropKind::Rotation => ph2d_i18n::tr("panel.timeline.prop.rotation"),
        PropKind::ScaleX => ph2d_i18n::tr("panel.timeline.prop.scale_x"),
        PropKind::ScaleY => ph2d_i18n::tr("panel.timeline.prop.scale_y"),
        PropKind::Opacity => ph2d_i18n::tr("panel.timeline.prop.opacity"),
        PropKind::TimeRemap => ph2d_i18n::tr("panel.timeline.prop.time"),
        PropKind::Morph => ph2d_i18n::tr("panel.timeline.prop.morph"),
        PropKind::Position => ph2d_i18n::tr("panel.timeline.prop.position"),
        PropKind::JointMotorTarget => ph2d_i18n::tr("panel.timeline.prop.motor_target"),
        PropKind::JointMotorSpeed => ph2d_i18n::tr("panel.timeline.prop.motor_speed"),
        PropKind::JointRestLength => ph2d_i18n::tr("panel.timeline.prop.rest_length"),
        PropKind::JointMaxLength => ph2d_i18n::tr("panel.timeline.prop.max_length"),
    }
}

/// **How this app names an animated channel** — `Ball · Position X`, or the short id
/// `Position X  #7294` when the object published no name (FASE C.3 do plano 12).
///
/// A porta ÚNICA, e a razão é literal: o card de Expressão nasceu montando esta string
/// por conta própria, e o doc-comment dele **prometia** `"Ball · Position Y"` desde o
/// primeiro dia enquanto o código escrevia `#nnnn` — a row e o card do mesmo canal
/// dizendo coisas diferentes é exactamente o que uma segunda cópia produz.
///
/// **O nome vem primeiro** porque é o que o artista varre numa coluna estreita: com
/// seis tracks do mesmo objeto, a propriedade é o que muda de linha para linha, e o
/// nome é a âncora que diz de quem é o bloco.
pub(crate) fn track_label(name: Option<&str>, entity: u64, prop: PropKind) -> String {
    match name {
        Some(n) => format!("{n} · {}", prop_label(prop)),
        // O fallback é o rótulo que o dope-sheet sempre teve. Ele NÃO é sinal de erro:
        // um objeto sem `Name` é transiente, e mostrar `#nnnn` continua distinguindo
        // duas rows do mesmo tipo em objetos diferentes.
        None => format!("{}  #{}", prop_label(prop), entity % 10_000),
    }
}
