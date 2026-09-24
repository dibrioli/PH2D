//! **Os parâmetros do TIPO escolhido** — limites, mola e comprimento: as rows que cada tipo de
//! junta pinta na §12.
//!
//! ⚠️ **Corte por RESPONSABILIDADE** (2026-09-13), o terceiro irmão `#[path]` do `joint.rs` depois
//! de `joint_cards.rs` (motor · ruptura) e `joint_custom.rs` (eixos): o pai responde *como a secção
//! se desenha*, este *o que cada TIPO oferece* — e é este que cresce, um tipo de cada vez. O pai
//! passou o tecto de 600 LOC quando as tabelas de rótulos viraram `TextKey`.

use super::rows::{num_row, num_row_unit, seg_row};
use super::*;
use ph2d_editor_core::screens::hero::InspectorJointInfo;
use ph2d_i18n::TextKey;
use ph2d_i18n::tr;
use ph2d_i18n::tr_with;

/// **Os parâmetros do TIPO escolhido** — limites, mola e comprimento.
///
/// Fn própria pelo cap de 200 LOC da seção e porque a família cresce por tipo:
/// o Wheel trouxe a primeira combinação de DUAS famílias (curso + mola), que foi
/// o que transformou a cadeia `else if` daqui em perguntas independentes.
#[allow(clippy::too_many_arguments)]
pub(super) fn paint_kind_params(
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
    let mut yy = y;
    // **O Custom descreve os EIXOS**, e essa é a família inteira dele: ele não
    // usa o par de limites único (a unidade seria de qual eixo?) nem um
    // comprimento. Módulo irmão pelo cap de LOC.
    if info.kind_tag == KIND_CUSTOM {
        yy = paint_axis_rows(scene, text_system, theme, hit_index, store, x, w, yy, info);
    }
    // ⚠️ **Perguntas INDEPENDENTES, não uma cadeia `else if`** — o
    // [`KIND_WHEEL`] é o primeiro tipo que quer DUAS famílias de linha (o curso,
    // que era do Pin/Slider, e a mola, que era da Spring), e numa cadeia ele
    // teria de escolher uma. A cadeia também já era frágil pelo outro lado: o
    // comentário do [`KIND_ROPE`] registra que um Weld herdaria o "Max Length"
    // de um `else` nu. Cada família agora se oferece sozinha.
    if kind_has_limits(info.kind_tag) {
        yy = seg_row(
            scene,
            text_system,
            theme,
            hit_index,
            store,
            x,
            w,
            yy,
            limits_label(info.kind_tag),
            ids::INSP_JOINT_LIMITS_GROUP,
            &ids::INSP_JOINT_LIMITS,
            &SWITCH_LABELS.map(TextKey::tr),
            u8::from(info.limits_enabled),
        );
        if info.limits_enabled {
            let unit = limit_unit(info.kind_tag);
            for (label, id) in [
                (
                    tr("panel.inspector.joint.min_unit"),
                    ids::INSP_JOINT_LIMIT_MIN,
                ),
                (
                    tr("panel.inspector.joint.max_unit"),
                    ids::INSP_JOINT_LIMIT_MAX,
                ),
            ] {
                yy = num_row_unit(
                    scene,
                    text_system,
                    theme,
                    hit_index,
                    store,
                    x,
                    w,
                    yy,
                    label,
                    id,
                    Some(unit),
                    None,
                );
            }
        }
    }
    if info.kind_tag == KIND_SPRING {
        yy = num_row_unit(
            scene,
            text_system,
            theme,
            hit_index,
            store,
            x,
            w,
            yy,
            tr("panel.inspector.joint.rest_length_m"),
            ids::INSP_JOINT_REST_LENGTH,
            Some(ph2d_editor_core::widget::Unit::Meters),
            None,
        );
    }
    // A solda que CEDE (W-SoftWeld). A chave vem ANTES da mola porque é ela quem
    // a revela — a mesma ordem do `Limits` e do seu Min/Max.
    if info.kind_tag == KIND_WELD {
        yy = seg_row(
            scene,
            text_system,
            theme,
            hit_index,
            store,
            x,
            w,
            yy,
            tr("panel.inspector.joint.weld"),
            ids::INSP_JOINT_SOFT_GROUP,
            &ids::INSP_JOINT_SOFT,
            &SOFT_LABELS.map(TextKey::tr),
            u8::from(info.soft),
        );
    }
    // A mola: da Spring (que PENDURA um corpo), do Wheel (cuja suspensão
    // SUSTENTA um) e da solda MOLE (cujo ÂNGULO cede). Mesmos dois campos,
    // mesmos dois ids — é a mesma coisa física, e por isso a troca de tipo
    // re-semeia a ESCALA deles.
    if kind_has_spring(info.kind_tag, info.soft) {
        for (label, id) in [
            (
                tr("panel.inspector.joint.stiffness"),
                ids::INSP_JOINT_STIFFNESS,
            ),
            (tr("panel.inspector.joint.damping"), ids::INSP_JOINT_DAMPING),
        ] {
            yy = num_row(
                scene,
                text_system,
                theme,
                hit_index,
                store,
                x,
                w,
                yy,
                label,
                id,
            );
        }
    }
    if info.kind_tag == KIND_ROPE || info.kind_tag == KIND_ROD || info.kind_tag == KIND_PULLEY {
        // O MESMO id, rótulo diferente: numa corda o número é um TETO, numa
        // barra é o comprimento em si, e numa polia é a corda INTEIRA (a soma
        // dos dois ramos). Um segundo id seria um segundo lugar para o mesmo
        // campo do componente.
        let label = match info.kind_tag {
            KIND_ROD => tr("panel.inspector.joint.length_m"),
            KIND_PULLEY => tr("panel.inspector.joint.rope_length_m"),
            _ => tr("panel.inspector.joint.max_length_m"),
        };
        yy = num_row(
            scene,
            text_system,
            theme,
            hit_index,
            store,
            x,
            w,
            yy,
            label,
            ids::INSP_JOINT_MAX_LENGTH,
        );
    }
    if info.kind_tag == KIND_PULLEY {
        // **Acrescentar uma roldana** (pedido 4). O botão mora aqui — na seção da
        // CORDA — porque é a corda que possui a lista, e porque é onde o artista
        // está quando pensa *"esta corda precisa de mais uma"*. A contagem no
        // rótulo é o que torna o clique VISÍVEL: a roldana nova nasce SOBRE a
        // corda, para não dar um puxão, e ali o desenho quase não muda.
        let rect = ph2d_editor_core::property_row::caixa_do_botao(
            text_system,
            x,
            w,
            yy,
            &tr_with(
                "panel.inspector.joint.add_wheel",
                &[("n", &info.wheel_count)],
            ),
        );
        let btn = Button::new(
            ids::INSP_JOINT_ADD_WHEEL,
            tr_with(
                "panel.inspector.joint.add_wheel",
                &[("n", &info.wheel_count)],
            ),
        )
        .kind(ButtonKind::Default)
        .visual(store.button_visual(ids::INSP_JOINT_ADD_WHEEL));
        paint_button(&btn, rect, scene, text_system, theme);
        hit_index.register(ids::INSP_JOINT_ADD_WHEEL, rect);
        yy = ph2d_editor_core::property_row::abaixo_do_botao(rect);
    }
    yy
}
