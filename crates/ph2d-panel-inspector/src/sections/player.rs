//! **§14 Platform Player** — o comportamento de personagem (W5).
//!
//! ⚠️ **A face VAZIA é a metade importante**, e é a lição que a §11 do W2a já
//! pagou: antes dela não existia gesto nenhum no editor que tornasse um sprite
//! físico. Aqui é a mesma coisa um degrau acima — um corpo Dynamic sem o
//! componente vê **um botão**, e é ele que faz o comportamento existir.
//!
//! ⚠️ **Nada aqui é oferecido a um corpo que não é Dynamic**, e é FÍSICA: a mola
//! é um impulso, e um impulso não move massa infinita. A recusa mora no
//! construtor do info (a shell), que é quem sabe o kind — o pintor decide se
//! oferece a partir da MESMA resposta.
//!
//! ⚠️ **Os `hit_index.register` são escritos com id LITERAL, um por botão**, e
//! isso não é verbosidade: é a única forma que o `architecture_panel_wiring_parity`
//! consegue ver (ele coleta `.register(ids::<LITERAL>` e pula um primeiro
//! argumento variável). Dobrá-los num laço apagaria a cobertura de paridade dos
//! botões em silêncio — a cicatriz que a §11 já carrega escrita.

use super::rows::num_row;
use super::*;
use ph2d_editor_core::screens::hero::InspectorPlayerInfo;

/// Os quatro números da PERNA e os quatro da CAMINHADA, na ordem em que a lei
/// os usa. Tabela e não oito chamadas soltas: o pintor e o gate de seam iteram
/// a MESMA lista, então uma row nova nasce pintada e varrida.
pub(crate) const PLAYER_ROWS: [(&str, ph2d_a11y::NodeId); 14] = [
    ("Cling Distance (m)", ids::INSP_PLAYER_CLING),
    ("Leg Stiffness", ids::INSP_PLAYER_STIFFNESS),
    ("Leg Damping", ids::INSP_PLAYER_DAMPING),
    ("Speed (m/s)", ids::INSP_PLAYER_SPEED),
    ("Acceleration", ids::INSP_PLAYER_ACCEL),
    ("Air Acceleration", ids::INSP_PLAYER_AIR_ACCEL),
    ("Max Slope (deg)", ids::INSP_PLAYER_MAX_SLOPE),
    // O PULO (W4). ⚠️ O primeiro é o único que o artista pensa — os seis
    // multiplicadores são o TATO, e o `1.0` de cada um é gravidade do mundo.
    ("Jump Height (m)", ids::INSP_PLAYER_JUMP_HEIGHT),
    ("Takeoff Gravity", ids::INSP_PLAYER_TAKEOFF_G),
    ("Takeoff Above (m/s)", ids::INSP_PLAYER_TAKEOFF_SPEED),
    ("Peak Gravity", ids::INSP_PLAYER_PEAK_G),
    ("Peak Window (m/s)", ids::INSP_PLAYER_PEAK_SPEED),
    ("Fall Gravity", ids::INSP_PLAYER_FALL_G),
    ("Cut Gravity", ids::INSP_PLAYER_CUT_G),
];

#[allow(clippy::too_many_arguments)]
pub(crate) fn paint_player_section(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    x: f32,
    w: f32,
    y: f32,
    info: &InspectorPlayerInfo,
) -> f32 {
    let header_h = TypeToken::Md.px() + Spacing::Md.px(); // LITERAL-PX-OK: section header band height
    let h = TypeToken::Md.px() + Spacing::Sm.px(); // LITERAL-PX-OK: control row height
    let collapsed = store.is_collapsed(ids::INSP_LIVE_PLAYER_SECTION);
    let color_id = ids::INSP_LIVE_PLAYER_COLOR;
    let rgba = store
        .widget_color(color_id)
        .unwrap_or([0x88, 0x88, 0x88, 0xff]); // LITERAL-COLOR-OK: neutral default section accent
    let header = SectionHeader::new(ids::INSP_LIVE_PLAYER_SECTION, "Platform Player")
        .collapsible(!collapsed)
        .color(rgba);
    let header_rect = Rect::new(x, y, w, header_h);
    paint_section_header(&header, header_rect, scene, text_system, theme);
    if let Some(circle_rect) = ph2d_editor_core::widget::color_circle_hit_rect(&header, header_rect)
    {
        hit_index.register(color_id, circle_rect);
    }
    if collapsed {
        return y + header_h;
    }

    let mut yy = y + header_h;

    // A FACE VAZIA — um botão, e é ele que faz o comportamento existir.
    if !info.has_player {
        let rect = Rect::new(x, yy, w, h);
        let btn = Button::new(ids::INSP_PLAYER_ADD, "Make Platform Player")
            .kind(ButtonKind::Default)
            .state(
                store
                    .button_state(ids::INSP_PLAYER_ADD)
                    .unwrap_or(ButtonState::Normal),
            );
        paint_button(&btn, rect, scene, text_system, theme);
        hit_index.register(ids::INSP_PLAYER_ADD, rect);
        return yy + h + Spacing::Sm.px();
    }

    yy = num_row(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        x,
        w,
        yy,
        "Float Height (m)",
        ids::INSP_PLAYER_FLOAT,
    );
    for (label, id) in PLAYER_ROWS {
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

    // ⚠️ **O piso geométrico, dito em voz alta — e pelo controle que o resolve.**
    //
    // O sensor mede na VERTICAL e quem encosta na rampa é a cápsula ao longo da
    // NORMAL dela, então flutuar de verdade exige
    // `float_height > half_height + radius / cos(max_slope)`. Com o ponto de
    // partida (`0,5`) e a cápsula canônica o personagem fica **TANGENTE** ao
    // chão — ele não paira, e só uma rampa revela. Um número que o app SABE e
    // não mostra é um número que o artista descobre por acidente.
    //
    // O aviso mora no rótulo do próprio botão que o conserta: um controle, uma
    // mensagem. Um readout separado seria uma segunda superfície dizendo o mesmo
    // fato, e as duas divergiriam no dia em que a fórmula ganhasse uma forma.
    if info.min_float_known {
        let label = if info.float_height <= info.min_float_height {
            format!("Fit to Collider (needs > {:.2} m)", info.min_float_height)
        } else {
            "Fit to Collider".to_string()
        };
        let rect = Rect::new(x, yy, w, h);
        let btn = Button::new(ids::INSP_PLAYER_FIT, &label)
            .kind(ButtonKind::Default)
            .state(
                store
                    .button_state(ids::INSP_PLAYER_FIT)
                    .unwrap_or(ButtonState::Normal),
            );
        paint_button(&btn, rect, scene, text_system, theme);
        hit_index.register(ids::INSP_PLAYER_FIT, rect);
        yy += h + Spacing::Sm.px();
    }

    let rect = Rect::new(x, yy, w, h);
    let btn = Button::new(ids::INSP_PLAYER_REMOVE, "Remove Platform Player")
        .kind(ButtonKind::Default)
        .state(
            store
                .button_state(ids::INSP_PLAYER_REMOVE)
                .unwrap_or(ButtonState::Normal),
        );
    paint_button(&btn, rect, scene, text_system, theme);
    hit_index.register(ids::INSP_PLAYER_REMOVE, rect);
    yy + h + Spacing::Sm.px()
}
