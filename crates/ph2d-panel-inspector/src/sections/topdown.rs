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

const CHECK_H: f32 = 18.0; // LITERAL-PX-OK: altura visual do Checkbox, igual à das irmãs

/// Uma linha de aviso. Devolve o `y` seguinte. (Gémea da da fábrica — ver a irmã.)
#[allow(clippy::too_many_arguments)]
fn warn(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    x: f32,
    w: f32,
    y: f32,
    texto: &str,
    token: ColorToken,
) -> f32 {
    let font = TypeToken::Sm.px();
    paint_text(
        text_system,
        scene,
        texto,
        x,
        y,
        font,
        w,
        resolve(token, theme),
    );
    y + font + ph2d_tokens::control_gap_px()
}

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
    let gap = Spacing::Xs.px();
    let n = ids.len() as f32;
    let cw = ((w - gap * (n - 1.0)) / n).max(0.0);
    for (i, &id) in ids.iter().enumerate() {
        let rect = Rect::new(x + (cw + gap) * i as f32, row_y, cw, ph2d_tokens::ROW_H_PX);
        hit_index.register(id, rect);
        let kind = if i == escolhido {
            ButtonKind::Accent
        } else {
            ButtonKind::Default
        };
        paint_button(
            &Button::new(id, rotulos.get(i).copied().unwrap_or(""))
                .kind(kind)
                .visual(store.button_visual(id)),
            rect,
            scene,
            text_system,
            theme,
        );
    }
    row_y + ph2d_tokens::row_pitch_px()
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
        cur_y = warn(
            scene,
            text_system,
            theme,
            x,
            w,
            cur_y,
            "No body \u{2014} add a Rigid Body for this to move anything.",
            ColorToken::Danger,
        );
    } else if !i.body_is_kinematic {
        cur_y = warn(
            scene,
            text_system,
            theme,
            x,
            w,
            cur_y,
            "The body must be Kinematic \u{2014} a dynamic body belongs to the solver.",
            ColorToken::Danger,
        );
    }
    if i.conflicts_with_platformer {
        cur_y = warn(
            scene,
            text_system,
            theme,
            x,
            w,
            cur_y,
            "A Platform Player on this object wins \u{2014} remove one of the two.",
            ColorToken::Warn,
        );
    } else if !i.clock_playing {
        cur_y = warn(
            scene,
            text_system,
            theme,
            x,
            w,
            cur_y,
            "The clock is stopped \u{2014} it moves while the clock plays.",
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
    for (label, id, step) in [
        ("Speed (m/s)", crate::ids::INSP_TD_SPEED, 0.1), // LITERAL-PX-OK: m/s
        ("Acceleration (0 = instant)", crate::ids::INSP_TD_ACCEL, 0.5), // LITERAL-PX-OK: m/s²
        ("Deceleration (0 = instant)", crate::ids::INSP_TD_DECEL, 0.5), // LITERAL-PX-OK: m/s²
    ] {
        cur_y = super::anchors::field_row(
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
        "Directions",
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
        "Viewpoint",
        &crate::ids::INSP_TD_VIEWPOINT,
        &rotulos,
        usize::from(i.viewpoint.tag()),
    );
    // ⭐ **Só em `Custom`** — ver o cabeçalho.
    if i.viewpoint.uses_angle() {
        cur_y = super::anchors::field_row(
            scene,
            text_system,
            theme,
            hit_index,
            store,
            x,
            w,
            cur_y,
            "Board Angle (deg)",
            &[crate::ids::INSP_TD_VIEW_ANGLE],
            0.5, // LITERAL-PX-OK: graus
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
        "Facing",
        &crate::ids::INSP_TD_FACING,
        &rotulos,
        usize::from(i.facing.tag()),
    );
    // ⭐ **Só quando ele roda.**
    if i.facing.uses_turn_speed() {
        cur_y = super::anchors::field_row(
            scene,
            text_system,
            theme,
            hit_index,
            store,
            x,
            w,
            cur_y,
            "Turn Speed (deg/s, 0 = instant)",
            &[crate::ids::INSP_TD_TURN_SPEED],
            10.0, // LITERAL-PX-OK: graus/s
        );
    }

    for (label, id, step) in [
        ("Min Slide Angle (deg)", crate::ids::INSP_TD_MIN_SLIDE, 1.0), // LITERAL-PX-OK: graus
        ("Max Slides", crate::ids::INSP_TD_MAX_SLIDES, 1.0),           // LITERAL-PX-OK: contagem
    ] {
        cur_y = super::anchors::field_row(
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
        );
    }

    let rect = Rect::new(x, cur_y, w, CHECK_H);
    hit_index.register(crate::ids::INSP_TD_DEFAULT_CONTROLS, rect);
    paint_checkbox(
        &Checkbox::new(crate::ids::INSP_TD_DEFAULT_CONTROLS, "Default Controls")
            .visual(store.checkbox_visual(crate::ids::INSP_TD_DEFAULT_CONTROLS))
            .value(if i.default_controls {
                CheckboxValue::Checked
            } else {
                CheckboxValue::Unchecked
            }),
        rect,
        scene,
        text_system,
        theme,
    );
    cur_y += CHECK_H + ph2d_tokens::control_gap_px();
    if !i.default_controls {
        cur_y = warn(
            scene,
            text_system,
            theme,
            x,
            w,
            cur_y,
            "Off \u{2014} the keys don't reach it; something else must drive it.",
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
        "Top-Down Player",
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
    if info.selected_count > 1 {
        cur_y = warn(
            scene,
            text_system,
            theme,
            x,
            w,
            cur_y,
            "Editing the primary selection only.",
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
