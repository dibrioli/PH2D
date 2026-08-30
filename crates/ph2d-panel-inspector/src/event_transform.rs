//! **A comissão do Transform** — irmão do [`crate::event`] pelo teto de LOC (HR-18), e o
//! corte é por assunto: aqui mora *como as sete caixas do Transform viram um
//! `InspectorTransformEdit`*.
//!
//! ⚠️ **É a metade de ESCRITA da fronteira de display**; a de leitura vive no
//! [`crate::sync`], e as duas lêem os MESMOS dois settings (`display_unit` para o
//! comprimento, `display_angle` para o ângulo). Separá-las em ficheiros não as separa em
//! leis — é a mesma porta, vista dos dois lados.
//!
//! Nasceu quando a unidade de ângulo (Enio, 2026-08-30) acrescentou linhas ao
//! `apply_event_impl` e o teto de LOC cobrou. ⭐ *O teto não pediu um `allow`: pediu o
//! corte que este ficheiro é* — e o `event_anchor` / `event_anim` / `event_joint` já
//! tinham aberto o caminho.

use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::ids;
use ph2d_editor_core::panel::PanelHostInternal;
use ph2d_editor_core::screens::hero::InspectorTransformInfo;

/// ⭐ **A comissão do Transform — a metade de ESCRITA da fronteira de display.**
///
/// # Por que é uma função própria
///
/// O corte é por responsabilidade, e foi o teto de LOC que o cobrou quando a unidade de
/// ângulo acrescentou linhas ao `apply_event_impl`: *ler as sete caixas, converter cada
/// uma da unidade em que o artista a vê para a unidade em que o documento a guarda, e
/// publicar UM `InspectorTransformEdit`* é um assunto só, e não tem nada a ver com o
/// despacho de eventos que o envolvia.
///
/// # ⚠️ As duas unidades entram pela MESMA porta que a leitura usa
///
/// `display_unit` para o comprimento e `display_angle` para o ângulo — os mesmos dois
/// que o [`crate::sync`] lê ao mostrar. ⛔ Se a leitura mostrasse graus e esta escrita
/// interpretasse radianos, digitar `90` numa caixa parada escreveria `90 rad` no
/// documento, e o objecto saltaria. *É por isso que a porta é uma só.*
///
/// ⚠️ **O `unwrap_or` de cada caixa tem de estar na unidade da CAIXA**, não na do
/// documento: ele é o valor de recurso para quando o `WidgetStore` ainda não tem a linha,
/// e um recurso em radianos numa caixa de graus seria lido como graus.
pub(crate) fn commit_transform_edit(
    host: &mut dyn PanelHostInternal,
    info: ph2d_editor_core::screens::hero::InspectorTransformInfo,
) {
    let unit = host.project().display_unit;
    let ppm = host.project().pixels_per_meter;
    let ang = host.project().display_angle;
    let x_disp = host
        .store()
        .number_value(ids::INSP_TRANSFORM_POS_X)
        .unwrap_or(unit.from_meters(info.translation[0], ppm) as f64) as f32;
    let y_disp = host
        .store()
        .number_value(ids::INSP_TRANSFORM_POS_Y)
        .unwrap_or(unit.from_meters(info.translation[1], ppm) as f64) as f32;
    let rot_disp = host
        .store()
        .number_value(ids::INSP_TRANSFORM_ROT)
        .unwrap_or(ang.from_radians_f64(f64::from(info.rotation_rad))) as f32;
    let sx = host
        .store()
        .number_value(ids::INSP_TRANSFORM_SCALE_X)
        .unwrap_or(info.scale[0] as f64) as f32;
    let sy = host
        .store()
        .number_value(ids::INSP_TRANSFORM_SCALE_Y)
        .unwrap_or(info.scale[1] as f64) as f32;
    // Skew authored in the ACTIVE angle unit, for parity with Rotation; the
    // ECS-commit boundary converts to radians and clamps to
    // Transform::SKEW_LIMIT (ADR-0025-amendment-1 §2.5).
    let skew_x_disp = host
        .store()
        .number_value(ids::INSP_TRANSFORM_SKEW_X)
        .unwrap_or(ang.from_radians_f64(f64::from(info.skew_rad[0]))) as f32;
    let skew_y_disp = host
        .store()
        .number_value(ids::INSP_TRANSFORM_SKEW_Y)
        .unwrap_or(ang.from_radians_f64(f64::from(info.skew_rad[1]))) as f32;
    host.bus_mut().push(EditorAction::InspectorTransformEdit(
        InspectorTransformInfo {
            entity_bits: info.entity_bits,
            translation: [unit.to_meters(x_disp, ppm), unit.to_meters(y_disp, ppm)],
            rotation_rad: ang.to_radians(rot_disp),
            scale: [sx, sy],
            skew_rad: [ang.to_radians(skew_x_disp), ang.to_radians(skew_y_disp)],
        },
    ));
}
