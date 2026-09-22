//! ⭐⭐⭐ **O que o Inspector mostra do PROJÉCTIL** (TOP-20 #14, W3).
//!
//! # ⭐⭐ O que o painel DIZ que os campos sozinhos não diriam
//!
//! | aviso | o que se passa |
//! |---|---|
//! | `No body` | sem `RigidBody` não há o que mover nem em que bater |
//! | `The body must be Kinematic` | um corpo dinâmico é do **solver**, e o projéctil não tem pose para escrever |
//! | `The flight is over` | o alcance ou os ricochetes acabaram |
//! | `The clock is stopped` | a ponte corre no passo fixo |
//!
//! ⚠️ **O terceiro é o desta wave, e sem ele um projéctil que fez exactamente o que devia lê-se
//! como partido:** ele está parado no ar, com todos os números certos no ecrã.
//!
//! # ⭐ E duas linhas SOMEM conforme os números
//!
//! O *Bounciness* só existe com saltos, e o *Target* só existe com perseguição — é a lei do
//! `SignalVerb::uses_arg`, e a alternativa (mostrar sempre) entrega um controlo morto.

use super::*;
use ph2d_editor_core::projectile_edits::InspectorProjectileInfo;
use ph2d_editor_core::widget::SectionFold;
use ph2d_i18n::tr;

/// **Os AVISOS** — a metade que responde a *«pus o componente e ele não faz nada»*.
///
/// ⚠️ Eles vêm ANTES dos números, e por isso são uma função própria: quem não vê nada mexer não
/// quer afinar um ricochete.
#[allow(clippy::too_many_arguments)]
fn avisos(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    x: f32,
    w: f32,
    y: f32,
    i: &InspectorProjectileInfo,
) -> f32 {
    let mut cur_y = y;
    if !i.has_body {
        cur_y = super::rows::aviso(
            scene,
            text_system,
            theme,
            x,
            w,
            cur_y,
            tr("panel.inspector.projectile.no_body_u_add_a_rigid_body_for_this_to_move_anything"),
            ColorToken::Danger,
        );
    } else if !i.body_is_kinematic {
        cur_y = super::rows::aviso(
            scene,
            text_system,
            theme,
            x,
            w,
            cur_y,
            tr(
                "panel.inspector.projectile.the_body_must_be_kinematic_u_a_dynamic_body_belongs_to_the_solver",
            ),
            ColorToken::Danger,
        );
    }
    if i.flight_over {
        cur_y = super::rows::aviso(
            scene,
            text_system,
            theme,
            x,
            w,
            cur_y,
            tr("panel.inspector.projectile.the_flight_is_over_u_rewind_to_launch_it_again"),
            ColorToken::Text3,
        );
    } else if !i.clock_playing {
        cur_y = super::rows::aviso(
            scene,
            text_system,
            theme,
            x,
            w,
            cur_y,
            tr("panel.inspector.projectile.the_clock_is_stopped_u_it_flies_while_the_clock_plays"),
            ColorToken::Text3,
        );
    }
    cur_y
}

/// O corpo da secção — os CONTROLOS.
#[allow(clippy::too_many_arguments, clippy::too_many_lines)]
fn corpo(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    x: f32,
    w: f32,
    y: f32,
    i: &InspectorProjectileInfo,
) -> f32 {
    let mut cur_y = avisos(scene, text_system, theme, x, w, y, i);
    // ⭐⭐ **A coluna do nome é da SECÇÃO** (`line/UIUX`, 2026-09-15): esta secção nasceu
    //    contra a porta antiga (`anchors::field_row`, o nome POR CIMA do campo) e passa à
    //    única que existe. ⚠️ Os nomes são os da secção INTEIRA, inclusive os das linhas que
    //    este quadro não pinta — *uma coluna que salta quando uma linha aparece é uma coluna
    //    por linha com outro nome.*
    let seccao = ph2d_editor_core::property_row::Seccao::medida(
        text_system,
        1,
        &[
            tr("panel.inspector.projectile.speed_m_s"),
            tr("panel.inspector.projectile.acceleration"),
            tr("panel.inspector.projectile.max_speed_0_no_cap"),
            tr("panel.inspector.projectile.gravity_0_straight"),
            tr("panel.inspector.projectile.max_bounces"),
            tr("panel.inspector.projectile.bounciness_1_perfect"),
            tr("panel.inspector.projectile.range_m_0_forever"),
            tr("panel.inspector.projectile.homing_0_none"),
        ],
    );
    for (label, id, step, unidade) in [
        (
            tr("panel.inspector.projectile.speed_m_s"),
            crate::ids::INSP_PJ_SPEED,
            0.5,
            Some(ph2d_editor_core::widget::Unit::MetersPerSecond),
        ), // LITERAL-PX-OK: m/s
        (
            tr("panel.inspector.projectile.acceleration"),
            crate::ids::INSP_PJ_ACCEL,
            0.5,
            None,
        ), // LITERAL-PX-OK: m/s²
        (
            tr("panel.inspector.projectile.max_speed_0_no_cap"),
            crate::ids::INSP_PJ_MAX_SPEED,
            0.5,
            None,
        ), // LITERAL-PX-OK: m/s
        (
            tr("panel.inspector.projectile.gravity_0_straight"),
            crate::ids::INSP_PJ_GRAVITY,
            0.5,
            None,
        ), // LITERAL-PX-OK: m/s²
        (
            tr("panel.inspector.projectile.max_bounces"),
            crate::ids::INSP_PJ_MAX_BOUNCES,
            1.0,
            None,
        ), // LITERAL-PX-OK: contagem
    ] {
        cur_y = super::rows::fields_row(
            scene,
            text_system,
            theme,
            hit_index,
            store,
            x,
            w,
            cur_y,
            label,
            &[id],
            step,
            unidade,
            seccao,
        );
    }
    // ⭐ **Só com saltos** — sem ricochete nenhum, a perda por salto não tem sujeito.
    if i.max_bounces > 0 {
        cur_y = super::rows::fields_row(
            scene,
            text_system,
            theme,
            hit_index,
            store,
            x,
            w,
            cur_y,
            tr("panel.inspector.projectile.bounciness_1_perfect"),
            &[crate::ids::INSP_PJ_BOUNCINESS],
            0.05, // LITERAL-PX-OK: fracção
            None,
            seccao,
        );
    }
    cur_y = super::rows::fields_row(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        x,
        w,
        cur_y,
        tr("panel.inspector.projectile.range_m_0_forever"),
        &[crate::ids::INSP_PJ_RANGE],
        1.0, // LITERAL-PX-OK: metros
        Some(ph2d_editor_core::widget::Unit::Meters),
        seccao,
    );
    cur_y = super::rows::fields_row(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        x,
        w,
        cur_y,
        tr("panel.inspector.projectile.homing_0_none"),
        &[crate::ids::INSP_PJ_HOMING_ACCEL],
        10.0, // LITERAL-PX-OK: m/s²
        None,
        seccao,
    );
    // ⭐ **Só com perseguição** — um campo de alvo sem aceleração é um controlo morto.
    if i.homing_accel > 0.0 {
        cur_y = super::anim_rows::text_row(
            scene,
            text_system,
            theme,
            hit_index,
            store,
            x,
            w,
            cur_y,
            tr("panel.inspector.projectile.homing_target_label"),
            crate::ids::INSP_PJ_HOMING_TARGET,
            TextInput::new(crate::ids::INSP_PJ_HOMING_TARGET, "")
                .placeholder(tr("panel.inspector.projectile.target_object_name_u")),
            seccao,
        );
        // ⚠️ **Um nome escrito que ninguém tem** não é o mesmo que nenhum nome, e o painel diz a
        // diferença — senão um alvo apagado lê-se como uma perseguição partida.
        if i.homing_target_missing {
            cur_y = super::rows::aviso(
                scene,
                text_system,
                theme,
                x,
                w,
                cur_y,
                tr("panel.inspector.projectile.no_object_in_the_scene_has_that_name"),
                ColorToken::Warn,
            );
        }
    }

    ph2d_editor_core::property_row::paint_check_row(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        x,
        w,
        cur_y,
        (
            crate::ids::INSP_PJ_FACE_VELOCITY,
            tr("panel.inspector.projectile.face_velocity"),
            i.face_velocity,
        ),
        seccao,
    )
}

/// Pinta a secção. Devolve o `y` seguinte.
#[allow(clippy::too_many_arguments)]
pub(crate) fn paint_projectile_section(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    x: f32,
    w: f32,
    y: f32,
    info: &InspectorProjectileInfo,
) -> f32 {
    let header_h = TypeToken::Md.px() + Spacing::Md.px(); // LITERAL-PX-OK: banda do cabeçalho
    let header = section_header(
        store,
        ph2d_editor_core::ids::INSP_LIVE_PROJECTILE_SECTION,
        tr("panel.inspector.projectile.projectile_motion"),
    );
    paint_section_header(
        &header,
        Rect::new(x, y, w, header_h),
        scene,
        text_system,
        theme,
    );
    let Some(fold) = SectionFold::begin(
        store,
        ph2d_editor_core::ids::INSP_LIVE_PROJECTILE_SECTION,
        x,
        w,
        y + header_h,
        scene,
        hit_index,
    ) else {
        return y + header_h;
    };
    let mut cur_y = y + header_h;
    if info.selected_count > 1 {
        cur_y = super::rows::aviso(
            scene,
            text_system,
            theme,
            x,
            w,
            cur_y,
            tr("panel.inspector.projectile.editing_the_primary_selection_only"),
            ColorToken::Text3,
        );
    }
    cur_y = corpo(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        x,
        w,
        cur_y,
        info,
    );
    fold.finish(store, scene, hit_index, cur_y)
}
