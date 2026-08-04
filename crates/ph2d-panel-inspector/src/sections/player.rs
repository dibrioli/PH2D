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

use super::rows::{card_frame, num_row};
use super::*;
use ph2d_editor_core::screens::hero::InspectorPlayerInfo;

/// Uma row: **rótulo · id · a dica de hover**.
///
/// ⚠️ A dica entra na MESMA tupla e não numa segunda tabela, e a razão é a que
/// este módulo já paga em toda lista: uma row nova nasce com dica, ou não nasce.
/// Uma tabela paralela de tooltips é a que fica incompleta em silêncio — o
/// controle continua pintado e o artista continua sem saber o que ele faz.
pub(crate) type PlayerRow = (&'static str, ph2d_a11y::NodeId, &'static str);

/// **A PERNA** — o que faz o personagem pairar em vez de encostar.
const LEG_ROWS: [PlayerRow; 4] = [
    (
        "Float Height (m)",
        ids::INSP_PLAYER_FLOAT,
        "How high the character hovers above the ground.",
    ),
    (
        "Cling Distance (m)",
        ids::INSP_PLAYER_CLING,
        "How far above rest the leg still grips: steps, not jumps.",
    ),
    (
        "Leg Stiffness",
        ids::INSP_PLAYER_STIFFNESS,
        "How hard the leg pushes back. Higher is a firmer stance.",
    ),
    (
        "Leg Damping",
        ids::INSP_PLAYER_DAMPING,
        "How fast the bounce dies out. Above 1 he pops.",
    ),
];

/// **ANDAR** — a velocidade, e o que conta como chão.
const WALK_ROWS: [PlayerRow; 4] = [
    (
        "Speed (m/s)",
        ids::INSP_PLAYER_SPEED,
        "Cruising speed, measured relative to the ground.",
    ),
    (
        "Acceleration",
        ids::INSP_PLAYER_ACCEL,
        "How quickly he reaches cruising speed on the ground.",
    ),
    (
        "Air Acceleration",
        ids::INSP_PLAYER_AIR_ACCEL,
        "Steering while airborne. 0 keeps the jump arc intact.",
    ),
    (
        "Max Slope (deg)",
        ids::INSP_PLAYER_MAX_SLOPE,
        "Steepest ramp he stands on and walks up, in DEGREES.",
    ),
];

/// **PULAR** (W4) — ⚠️ o primeiro é o único que o artista pensa; os seis
/// multiplicadores são o TATO, e o `1.0` de cada um é a gravidade do mundo.
const JUMP_ROWS: [PlayerRow; 7] = [
    (
        "Jump Height (m)",
        ids::INSP_PLAYER_JUMP_HEIGHT,
        "How high a full jump reaches, in metres.",
    ),
    (
        "Takeoff Gravity",
        ids::INSP_PLAYER_TAKEOFF_G,
        "Gravity while rising fast. 1 is the world's.",
    ),
    (
        "Takeoff Above (m/s)",
        ids::INSP_PLAYER_TAKEOFF_SPEED,
        "Rising faster than this uses Takeoff Gravity.",
    ),
    (
        "Peak Gravity",
        ids::INSP_PLAYER_PEAK_G,
        "Gravity near the top. Below 1 he hangs longer.",
    ),
    (
        "Peak Window (m/s)",
        ids::INSP_PLAYER_PEAK_SPEED,
        "How wide that slow top is, in m/s.",
    ),
    (
        "Fall Gravity",
        ids::INSP_PLAYER_FALL_G,
        "Gravity while falling. Above 1 he drops faster than he rose.",
    ),
    (
        "Cut Gravity",
        ids::INSP_PLAYER_CUT_G,
        "Gravity while rising with the button RELEASED.",
    ),
];

/// **O PERDÃO** (W8 + W10) — ⚠️ os dois primeiros são o MESMO erro visto dos
/// dois lados (um apertou tarde, o outro cedo); os dois da W10 perdoam coisas
/// diferentes, e o card os junta porque a **família** é a mesma: *o jogo faz o
/// que o jogador quis dizer*. `0` desliga cada um, e com os quatro em zero a lei
/// é a que o W4 shipou, ao bit.
///
/// ⚠️ **A unidade está no RÓTULO porque as quatro não são a mesma grandeza:**
/// três são segundos e o *Corner Reach* é METROS. Sem `(m)` ali, um artista que
/// leu as três de cima escreve `0.1` esperando um décimo de segundo e recebe dez
/// centímetros.
const FORGIVE_ROWS: [PlayerRow; 4] = [
    (
        "Coyote Time (s)",
        ids::INSP_PLAYER_COYOTE,
        "Grace after leaving the ground. 0 turns it off.",
    ),
    (
        "Jump Buffer (s)",
        ids::INSP_PLAYER_BUFFER,
        "A press this early still fires on landing.",
    ),
    (
        "Corner Reach (m)",
        ids::INSP_PLAYER_CORNER,
        "Slide sideways up to this to clear a ledge you clipped. In METRES.",
    ),
    (
        "Lift Momentum (s)",
        ids::INSP_PLAYER_LIFT,
        "Keep a moving platform's speed for this long after leaving it.",
    ),
];

/// **A REAÇÃO** (W6) — ⚠️ os defaults são OPOSTOS de propósito: o peso volta
/// inteiro (é a física) e o tapete nasce desligado (é de produto).
const REACT_ROWS: [PlayerRow; 2] = [
    (
        "Weight on Ground",
        ids::INSP_PLAYER_REACT_SUPPORT,
        "How much of his weight presses the ground down.",
    ),
    (
        "Push on Ground",
        ids::INSP_PLAYER_REACT_MOVEMENT,
        "How much of his walking shoves the ground back.",
    ),
];

/// **A TABELA da §14** — cinco cards, e os dezenove números dentro deles.
///
/// ⚠️ **UMA tabela, TRÊS consumidores** (o molde do `SECTIONS` do painel de
/// física): o **pintor** desenha, o **`populate`** registra a dica de hover de
/// cada id, e a **varredura de seam** clica tudo. Uma row nova nasce pintada,
/// com dica e varrida, ou não nasce.
///
/// ⚠️ **Os cinco títulos são os cinco módulos da lei** (`ride` · `walk` · `jump`
/// · o perdão · `react`), não uma arrumação de gosto: quando o artista pergunta
/// *"o que este número faz?"*, a primeira metade da resposta é *"a que pergunta
/// ele pertence"*, e ela passou a estar escrita na tela (Enio, 2026-08-04:
/// *"esse tanto de parâmetros juntos não fica bem; organize-os em cards com um
/// título que facilite o entendimento"*).
pub(crate) const PLAYER_CARDS: [(&str, ph2d_a11y::NodeId, &[PlayerRow]); 5] = [
    ("LEG", ids::INSP_PLAYER_CARD_LEG, &LEG_ROWS),
    ("WALK", ids::INSP_PLAYER_CARD_WALK, &WALK_ROWS),
    ("JUMP", ids::INSP_PLAYER_CARD_JUMP, &JUMP_ROWS),
    ("FORGIVENESS", ids::INSP_PLAYER_CARD_FORGIVE, &FORGIVE_ROWS),
    ("REACTION", ids::INSP_PLAYER_CARD_REACT, &REACT_ROWS),
];

/// Quantas rows numéricas a seção pinta — **contadas da tabela**, nunca escritas
/// à mão ao lado dela.
pub(crate) const fn player_row_count() -> usize {
    let mut n = 0;
    let mut i = 0;
    while i < PLAYER_CARDS.len() {
        n += PLAYER_CARDS[i].2.len();
        i += 1;
    }
    n
}

/// As dicas dos três BOTÕES da seção — a mesma lei das rows, num lugar onde não
/// cabe uma tupla de row.
pub(crate) const PLAYER_BUTTON_TIPS: [(ph2d_a11y::NodeId, &str); 3] = [
    (
        ids::INSP_PLAYER_ADD,
        "Turn this body into a walking, jumping character.",
    ),
    (
        ids::INSP_PLAYER_FIT,
        "Set Float Height from the collider, so he really hovers.",
    ),
    (
        ids::INSP_PLAYER_REMOVE,
        "Give the behaviour back: it becomes a plain body again.",
    ),
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

    for (title, _card_id, rows) in PLAYER_CARDS {
        let (ix, iw, mut ry, next_y) =
            card_frame(scene, text_system, theme, x, w, yy, title, rows.len());
        for (label, id, _tip) in rows {
            ry = num_row(
                scene,
                text_system,
                theme,
                hit_index,
                store,
                ix,
                iw,
                ry,
                label,
                *id,
            );
        }
        // ⚠️ O `ry` é DESCARTADO de propósito: quem manda no fluxo é a moldura
        // (`next_y`), medida pela MESMA régua com que as rows avançam. Somar as
        // rows aqui seria a segunda aritmética que discorda da caixa desenhada.
        let _ = ry;
        yy = next_y;
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
