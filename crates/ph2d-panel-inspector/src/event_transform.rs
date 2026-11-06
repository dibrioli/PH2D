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
    campos: &[ph2d_a11y::NodeId],
) {
    use crate::ids::{
        INSP_TRANSFORM_POS_X, INSP_TRANSFORM_POS_Y, INSP_TRANSFORM_ROT, INSP_TRANSFORM_SCALE_X,
        INSP_TRANSFORM_SCALE_Y, INSP_TRANSFORM_SKEW_X, INSP_TRANSFORM_SKEW_Y,
    };
    let unit = host.project().display_unit;
    let ppm = host.project().pixels_per_meter;
    let ang = host.project().display_angle;
    // ⛔ Só o campo que a mão mexeu vem da caixa: as caixas são uma por id, e dois cartões presos partilham-nas. O resto
    // é do objecto do gesto, ou o Y do outro objecto vem junto (dono 05/11).
    let da_mao = |id| {
        campos
            .contains(&id)
            .then(|| host.store().number_value(id))
            .flatten()
            .map(|v| v as f32)
    };
    let edit = InspectorTransformInfo {
        entity_bits: info.entity_bits,
        translation: [
            da_mao(INSP_TRANSFORM_POS_X).map_or(info.translation[0], |v| unit.to_meters(v, ppm)),
            da_mao(INSP_TRANSFORM_POS_Y).map_or(info.translation[1], |v| unit.to_meters(v, ppm)),
        ],
        rotation_rad: da_mao(INSP_TRANSFORM_ROT).map_or(info.rotation_rad, |v| ang.to_radians(v)),
        scale: [
            da_mao(INSP_TRANSFORM_SCALE_X).unwrap_or(info.scale[0]),
            da_mao(INSP_TRANSFORM_SCALE_Y).unwrap_or(info.scale[1]),
        ],
        skew_rad: [
            da_mao(INSP_TRANSFORM_SKEW_X).map_or(info.skew_rad[0], |v| ang.to_radians(v)),
            da_mao(INSP_TRANSFORM_SKEW_Y).map_or(info.skew_rad[1], |v| ang.to_radians(v)),
        ],
    };
    host.bus_mut()
        .push(EditorAction::InspectorTransformEdit(edit));
}

/// ⭐ **Girar 90° e espelhar o objecto que não é vetor** (dono 04/11): soma um quarto de volta à
/// rotação, ou troca o sinal de `scale`, e publica UM `InspectorTransformEdit`, um passo de desfazer.
/// A física lê o sinal: forma pelo valor absoluto, `offset` espelhado (`ph2d_physics_ecs::scale`).
pub(crate) fn giro_click(
    host: &mut dyn PanelHostInternal,
    id: ph2d_a11y::NodeId,
    info: InspectorTransformInfo,
) -> bool {
    use crate::ids::{
        INSP_TRANSFORM_FLIP_H, INSP_TRANSFORM_FLIP_V, INSP_TRANSFORM_ROT, INSP_TRANSFORM_ROTATE_CW,
        INSP_TRANSFORM_SCALE_X, INSP_TRANSFORM_SCALE_Y,
    };
    // Keep in place: o espelho passa pela porta da imagem, que sabe compensar o pivô.
    if crate::cards_sprite_pivot::mirror_no_lugar()
        && crate::state::current_inspector_sprite().is_some()
        && (id == INSP_TRANSFORM_FLIP_H || id == INSP_TRANSFORM_FLIP_V)
    {
        let eixo = u8::from(id == INSP_TRANSFORM_FLIP_V);
        crate::cards::scale_espelhado(info.entity_bits, usize::from(eixo));
        host.bus_mut()
            .push(ph2d_editor_core::action_bus::EditorAction::ImageUseEdit {
                entity_bits: info.entity_bits,
                edit: ph2d_editor_core::ImageUseEdit::MirrorInPlace(eixo),
            });
        return true;
    }
    let troca_sinal = |host: &mut dyn PanelHostInternal, campo, atual: f32| {
        host.store_mut().set_number_value(campo, -f64::from(atual));
    };
    if id == INSP_TRANSFORM_ROTATE_CW {
        let ang = host.project().display_angle;
        let agora = info.rotation_rad;
        let volta = std::f32::consts::TAU;
        let novo = (agora + std::f32::consts::FRAC_PI_2 + std::f32::consts::PI).rem_euclid(volta)
            - std::f32::consts::PI;
        host.store_mut()
            .set_number_value(INSP_TRANSFORM_ROT, ang.from_radians_f64(f64::from(novo)));
    } else if id == INSP_TRANSFORM_FLIP_H {
        troca_sinal(host, INSP_TRANSFORM_SCALE_X, info.scale[0]);
        crate::cards::scale_espelhado(info.entity_bits, 0);
    } else if id == INSP_TRANSFORM_FLIP_V {
        troca_sinal(host, INSP_TRANSFORM_SCALE_Y, info.scale[1]);
        crate::cards::scale_espelhado(info.entity_bits, 1);
    } else {
        return false;
    }
    let campo = if id == INSP_TRANSFORM_ROTATE_CW {
        INSP_TRANSFORM_ROT
    } else if id == INSP_TRANSFORM_FLIP_H {
        INSP_TRANSFORM_SCALE_X
    } else {
        INSP_TRANSFORM_SCALE_Y
    };
    commit_transform_edit(host, info, &[campo]);
    true
}
