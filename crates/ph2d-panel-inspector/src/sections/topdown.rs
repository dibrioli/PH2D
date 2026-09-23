//! ⭐⭐⭐ **O que o Inspector mostra do MOVER DE VISTA DE CIMA** (TOP-20 #13, W3).
//!
//! # ⭐⭐ O que o painel DIZ que os campos sozinhos não diriam
//!
//! Quatro avisos, e cada um responde a uma forma diferente de *«pus o componente e ele não anda»*:
//!
//! | aviso | o que se passa |
//! |---|---|
//! | `No body` | sem `RigidBody` não há o que mover |
//! | `The body must be Kinematic` | um corpo dinâmico é do **solver**, e o mover não tem pose para escrever |
//! | `A Platform Player on this object wins` | dois movers, um `Transform` |
//! | `The clock is stopped` | a ponte corre no passo fixo |
//!
//! ⚠️ **Os dois do meio são os que esta wave pagou a descobrir**, e sem eles um componente que está
//! a funcionar perfeitamente lê-se como partido **com todos os números certos no ecrã**.
//!
//! # ⭐ E duas linhas SOMEM conforme o modo
//!
//! O *Board Angle* só existe em `Custom` e a *Turn Speed* só existe quando ele roda — é a lei do
//! `SignalVerb::uses_arg`, e a alternativa (mostrar sempre) entrega um controlo morto em cada modo.

use super::*;
use ph2d_editor_core::topdown_edits::{
    InspectorFacing, InspectorMoveDirections, InspectorTopDownInfo, InspectorViewpoint,
};
use ph2d_editor_core::widget::SectionFold;
use ph2d_i18n::tr;

/// Um segmentado. ⚠️ **A selecção vem do SNAPSHOT, nunca do store** — ler o store faria o primeiro
/// clique depois de trocar de objecto mandar o valor do objecto anterior.
#[allow(clippy::too_many_arguments)]
fn seg_row(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    x: f32,
    w: f32,
    y: f32,
    titulo: &str,
    ids: &[ph2d_a11y::NodeId],
    rotulos: &[&'static str],
    escolhido: usize,
) -> f32 {
    let font = TypeToken::Sm.px();
    paint_text(
        text_system,
        scene,
        titulo,
        x,
        y,
        font,
        w,
        resolve(ColorToken::Text2, theme),
    );
    let row_y = y + font + Spacing::Xs.px();
    // ⭐⭐ **A disposição é a PORTA da casa** — ver o irmão em `sections/anim.rs` (2026-09-19): as
    //    quatro linhas que repartiam a coluna em partes IGUAIS cortavam `Top-Down` · `Custom` ·
    //    `Don't Turn` · `Face Move` a `48 px` cada, com as palavras a caberem de sobra na coluna.
    let segments: Vec<(&str, bool, ph2d_a11y::NodeId)> = ids
        .iter()
        .enumerate()
        .map(|(i, &id)| (rotulos.get(i).copied().unwrap_or(""), i == escolhido, id))
        .collect();
    let seg_h = ph2d_editor_core::widget::panel_chrome::paint_segmented_group_adaptive(
        Rect::new(x, row_y, w, ph2d_tokens::ROW_H_PX),
        &segments,
        scene,
        text_system,
        theme,
        store,
        hit_index,
    );
    row_y + seg_h + ph2d_tokens::control_gap_px()
}

/// **Os AVISOS** — a metade que responde a *«pus o componente e ele não anda»*.
///
/// ⚠️ Eles vêm ANTES dos números, e por isso são uma função própria: quem não vê nada mexer não
/// quer afinar uma rampa. O corte foi imposto pelo tecto de LOC e está certo por isto.
#[allow(clippy::too_many_arguments)]
fn avisos(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    x: f32,
    w: f32,
    y: f32,
    i: &InspectorTopDownInfo,
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
            tr("panel.inspector.topdown.no_body_u_add_a_rigid_body_for_this_to_move_anything"),
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
                "panel.inspector.topdown.the_body_must_be_kinematic_u_a_dynamic_body_belongs_to_the_solver",
            ),
            ColorToken::Danger,
        );
    }
    if i.conflicts_with_platformer {
        cur_y = super::rows::aviso(
            scene,
            text_system,
            theme,
            x,
            w,
            cur_y,
            tr(
                "panel.inspector.topdown.a_platform_player_on_this_object_wins_u_remove_one_of_the_two",
            ),
            ColorToken::Warn,
        );
    } else if !i.clock_playing {
        cur_y = super::rows::aviso(
            scene,
            text_system,
            theme,
            x,
            w,
            cur_y,
            tr("panel.inspector.topdown.the_clock_is_stopped_u_it_moves_while_the_clock_plays"),
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
    i: &InspectorTopDownInfo,
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
            tr("panel.inspector.topdown.speed_m_s"),
            tr("panel.inspector.topdown.acceleration_0_instant"),
            tr("panel.inspector.topdown.deceleration_0_instant"),
            tr("panel.inspector.topdown.board_angle_deg"),
            tr("panel.inspector.topdown.turn_speed_deg_s_0_instant"),
            tr("panel.inspector.topdown.min_slide_angle_deg"),
            tr("panel.inspector.topdown.max_slides"),
        ],
    );
    for (label, id, step, unidade) in [
        (
            tr("panel.inspector.topdown.speed_m_s"),
            crate::ids::INSP_TD_SPEED,
            0.1, // LITERAL-PX-OK: passo de scrub em m/s
            Some(ph2d_editor_core::widget::Unit::MetersPerSecond),
        ), // LITERAL-PX-OK: m/s
        (
            tr("panel.inspector.topdown.acceleration_0_instant"),
            crate::ids::INSP_TD_ACCEL,
            0.5,
            None,
        ), // LITERAL-PX-OK: m/s²
        (
            tr("panel.inspector.topdown.deceleration_0_instant"),
            crate::ids::INSP_TD_DECEL,
            0.5,
            None,
        ), // LITERAL-PX-OK: m/s²
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

    let rotulos: Vec<&'static str> = InspectorMoveDirections::ALL
        .iter()
        .map(|m| m.label())
        .collect();
    cur_y = seg_row(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        x,
        w,
        cur_y,
        tr("panel.inspector.topdown.directions"),
        &crate::ids::INSP_TD_DIRECTIONS,
        &rotulos,
        usize::from(i.directions.tag()),
    );

    let rotulos: Vec<&'static str> = InspectorViewpoint::ALL.iter().map(|m| m.label()).collect();
    cur_y = seg_row(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        x,
        w,
        cur_y,
        tr("panel.inspector.topdown.viewpoint"),
        &crate::ids::INSP_TD_VIEWPOINT,
        &rotulos,
        usize::from(i.viewpoint.tag()),
    );
    // ⭐ **Só em `Custom`** — ver o cabeçalho.
    if i.viewpoint.uses_angle() {
        cur_y = super::rows::fields_row(
            scene,
            text_system,
            theme,
            hit_index,
            store,
            x,
            w,
            cur_y,
            tr("panel.inspector.topdown.board_angle_deg"),
            &[crate::ids::INSP_TD_VIEW_ANGLE],
            0.5, // LITERAL-PX-OK: graus
            Some(ph2d_editor_core::widget::Unit::Degrees),
            seccao,
        );
    }

    let rotulos: Vec<&'static str> = InspectorFacing::ALL.iter().map(|m| m.label()).collect();
    cur_y = seg_row(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        x,
        w,
        cur_y,
        tr("panel.inspector.topdown.facing"),
        &crate::ids::INSP_TD_FACING,
        &rotulos,
        usize::from(i.facing.tag()),
    );
    // ⭐ **Só quando ele roda.**
    if i.facing.uses_turn_speed() {
        cur_y = super::rows::fields_row(
            scene,
            text_system,
            theme,
            hit_index,
            store,
            x,
            w,
            cur_y,
            tr("panel.inspector.topdown.turn_speed_deg_s_0_instant"),
            &[crate::ids::INSP_TD_TURN_SPEED],
            10.0, // LITERAL-PX-OK: graus/s
            Some(ph2d_editor_core::widget::Unit::DegreesPerSecond),
            seccao,
        );
    }

    cur_y = deslize(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        x,
        w,
        cur_y,
        seccao,
        i,
    );
    cur_y
}

/// **O DESLIZE numa parede** — irmão por tecto de LOC (a função passou a `210/200` quando a
/// coluna da secção entrou). ⚠️ Corte por RESPONSABILIDADE: é a metade que o TOP-20 #13 existe
/// para trazer (deslizar à velocidade CHEIA), e ela tem os dois números que só ela lê.
#[allow(clippy::too_many_arguments)]
fn deslize(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    x: f32,
    w: f32,
    y: f32,
    seccao: ph2d_editor_core::property_row::Seccao,
    i: &InspectorTopDownInfo,
) -> f32 {
    let mut cur_y = y;
    for (label, id, step, unidade) in [
        (
            tr("panel.inspector.topdown.min_slide_angle_deg"),
            crate::ids::INSP_TD_MIN_SLIDE,
            1.0,
            Some(ph2d_editor_core::widget::Unit::Degrees),
        ), // LITERAL-PX-OK: graus
        (
            tr("panel.inspector.topdown.max_slides"),
            crate::ids::INSP_TD_MAX_SLIDES,
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

    cur_y = ph2d_editor_core::property_row::paint_check_row(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        x,
        w,
        cur_y,
        (
            crate::ids::INSP_TD_DEFAULT_CONTROLS,
            tr("panel.inspector.topdown.default_controls"),
            i.default_controls,
        ),
        seccao,
    );
    if !i.default_controls {
        cur_y = super::rows::aviso(
            scene,
            text_system,
            theme,
            x,
            w,
            cur_y,
            tr(
                "panel.inspector.topdown.off_u_the_keys_don_t_reach_it_something_else_must_drive_it",
            ),
            ColorToken::Text3,
        );
    }
    cur_y
}

/// Pinta a secção. Devolve o `y` seguinte.
#[allow(clippy::too_many_arguments)]
pub(crate) fn paint_topdown_section(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    x: f32,
    w: f32,
    y: f32,
    info: &InspectorTopDownInfo,
) -> f32 {
    let header_h = TypeToken::Md.px() + Spacing::Md.px(); // LITERAL-PX-OK: banda do cabeçalho
    let header = section_header(
        store,
        ph2d_editor_core::ids::INSP_LIVE_TOPDOWN_SECTION,
        tr("panel.inspector.topdown.top_down_player"),
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
        ph2d_editor_core::ids::INSP_LIVE_TOPDOWN_SECTION,
        x,
        w,
        y + header_h,
        scene,
        hit_index,
    ) else {
        return y + header_h;
    };
    let mut cur_y = y + header_h;
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
