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

use super::rows::{card_frame, num_row, seg_row};
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
    // ⚠️ **A dica nomeia o TERCEIRO eixo, e ele foi medido** (W26): baixar este
    // número devolve o quique do pouso E uma subida lenta em rampa, mas só a
    // subida escala com os `Sub-steps` do painel de mundo (`∝ 1/n`) — o quique é
    // independente deles. Sem esta frase o artista baixa o knob, vê o
    // personagem andar sozinho, e não tem como saber que o outro knob paga.
    (
        "Leg Damping",
        ids::INSP_PLAYER_DAMPING,
        "How fast the bounce dies out. Above 1 he pops. Lower it for a bouncier \
         landing, then raise World > Sub-steps to stop him creeping up ramps.",
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
const REACT_ROWS: [PlayerRow; 3] = [
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
    (
        "Push on Bodies",
        ids::INSP_PLAYER_REACT_PUSH,
        "How hard he shoves what he walks into. KINEMATIC only: a dynamic body already pushes.",
    ),
];

/// **AS PAREDES** (W13) — ⚠️ card PRÓPRIO, e não uma extensão do de PULO: o
/// escorregamento não é um pulo, e o que agrupa estes cinco números é a
/// superfície, não o gesto. As duas primeiras rows nascem em ZERO porque a
/// capacidade é opt-in (ver `WallConfig::STARTING_POINT`).
const WALL_ROWS: [PlayerRow; 6] = [
    (
        "Wall Slide (m/s)",
        ids::INSP_PLAYER_WALL_SLIDE,
        "Slide DOWN a wall at this speed while pushing into it. 0 = off.",
    ),
    (
        "Wall Jump (m)",
        ids::INSP_PLAYER_WALL_JUMP,
        "How high a jump off a wall goes. 0 = off.",
    ),
    (
        "Wall Push (m/s)",
        ids::INSP_PLAYER_WALL_PUSH,
        "How hard a wall jump throws you AWAY from the wall.",
    ),
    (
        "Wall Lockout (s)",
        ids::INSP_PLAYER_WALL_LOCK,
        "Air control stays quiet this long after a wall jump.",
    ),
    (
        "Wall Reach (m)",
        ids::INSP_PLAYER_WALL_REACH,
        "How far past your own width the wall sensor looks.",
    ),
    (
        "Wall Grab (s)",
        ids::INSP_PLAYER_WALL_GRAB,
        "Hold R against a wall to stick instead of sliding, for this long. 0 = off.",
    ),
];

/// **O ARRANQUE** (W14) — ⚠️ card próprio pela mesma razão do das paredes, e a
/// primeira row nasce em ZERO porque a capacidade é opt-in.
///
/// ⚠️ **Três números, e o que impede voar NÃO é nenhum deles:** a carga (um
/// arranque por tempo-de-voo, reposta pelo pé no chão) é lei, não knob — expô-la
/// seria oferecer ao artista a escolha de fazer o personagem voar, que não é uma
/// escolha, é um bug com um slider.
const DASH_ROWS: [PlayerRow; 3] = [
    (
        "Dash Speed (m/s)",
        ids::INSP_PLAYER_DASH_SPEED,
        "How fast the dash carries him. 0 = off.",
    ),
    (
        "Dash Time (s)",
        ids::INSP_PLAYER_DASH_TIME,
        "How long it lasts. Speed x Time is the DISTANCE it covers.",
    ),
    (
        "Dash Cooldown (s)",
        ids::INSP_PLAYER_DASH_COOL,
        "Recovery after it ENDS, before he can dash again.",
    ),
];

/// **O AGACHAR** (W15) — ⚠️ card próprio, e a primeira row nasce em ZERO porque
/// a capacidade é opt-in.
///
/// ⚠️ **Dois números, e o zero significa coisas DIFERENTES em cada um** — a
/// altura desliga a capacidade, a velocidade não. É por isso que o hover de cada
/// um diz o que o SEU zero faz: quem lê "0 = off" numa row e o supõe na outra
/// autoraria um agachar que não existe julgando ter feito um agachar parado.
///
/// ⚠️ **E o que NÃO está aqui:** nenhuma dimensão de collider. Agachar é uma
/// perna mais CURTA, e a forma do corpo não é reescrita — ver o topo de
/// `ph2d_platformer::crouch`.
const CROUCH_ROWS: [PlayerRow; 2] = [
    (
        "Crouch Height (m)",
        ids::INSP_PLAYER_CROUCH_HEIGHT,
        "How low he floats while holding DOWN. 0 = off.",
    ),
    (
        "Crouch Speed (m/s)",
        ids::INSP_PLAYER_CROUCH_SPEED,
        "How fast he walks while crouched. 0 means duck in place.",
    ),
];

/// **A TABELA da §14** — oito cards, e os vinte e quatro números dentro deles.
///
/// ⚠️ **UMA tabela, TRÊS consumidores** (o molde do `SECTIONS` do painel de
/// física): o **pintor** desenha, o **`populate`** registra a dica de hover de
/// cada id, e a **varredura de seam** clica tudo. Uma row nova nasce pintada,
/// com dica e varrida, ou não nasce.
///
/// ⚠️ **Os títulos são os módulos da lei** (`ride` · `walk` · `jump` · o perdão
/// · `react` · `wall` · `dash` · `crouch`), não uma arrumação de gosto: quando o artista
/// pergunta *"o que este número faz?"*, a primeira metade da resposta é *"a que
/// pergunta ele pertence"*, e ela passou a estar escrita na tela (Enio,
/// 2026-08-04: *"esse tanto de parâmetros juntos não fica bem; organize-os em
/// cards com um título que facilite o entendimento"*).
pub(crate) const PLAYER_CARDS: [(&str, ph2d_a11y::NodeId, &[PlayerRow]); 8] = [
    ("LEG", ids::INSP_PLAYER_CARD_LEG, &LEG_ROWS),
    ("WALK", ids::INSP_PLAYER_CARD_WALK, &WALK_ROWS),
    ("JUMP", ids::INSP_PLAYER_CARD_JUMP, &JUMP_ROWS),
    ("FORGIVENESS", ids::INSP_PLAYER_CARD_FORGIVE, &FORGIVE_ROWS),
    ("REACTION", ids::INSP_PLAYER_CARD_REACT, &REACT_ROWS),
    ("WALLS", ids::INSP_PLAYER_CARD_WALL, &WALL_ROWS),
    ("DASH", ids::INSP_PLAYER_CARD_DASH, &DASH_ROWS),
    ("CROUCH", ids::INSP_PLAYER_CARD_CROUCH, &CROUCH_ROWS),
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

/// As dicas dos cinco BOTÕES da seção — a mesma lei das rows, num lugar onde não
/// cabe uma tupla de row.
pub(crate) const PLAYER_BUTTON_TIPS: [(ph2d_a11y::NodeId, &str); 5] = [
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
    (
        ids::INSP_PLAYER_CLEAR_RUN,
        "Throw away the recorded run. Playing with Physics on records a new one.",
    ),
    (
        ids::INSP_PLAYER_FIT_CROUCH,
        "Set Crouch Height to the lowest this body can really float at.",
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

    // **COMO ele é movido** (W-KinMove) — a primeira coisa da seção, porque toda
    // row abaixo dela é interpretada por este modo (a `LEG` inteira é a mola, e
    // sob Snap não há mola).
    yy = seg_row(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        x,
        w,
        yy,
        "Body",
        ids::INSP_PLAYER_MODE,
        &ids::INSP_PLAYER_MODE_IDS,
        &["Dynamic", "Kinematic"],
        info.mode_tag,
    );

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

    // **O MESMO piso, uma perna abaixo** (W18) — o espelho exato do botão acima,
    // e ele existe porque o card do AGACHAR não tinha controle nenhum que
    // resolvesse o número.
    //
    // ⚠️ **O defeito é medido e não é o que a nota da W15 previa.** Ela dizia *"o
    // corpo enterrado"*; ele **não enterra, ele SATURA** — o solver o segura
    // tangente com 1 mm de folga, a pose fica perfeitamente estável, e o que
    // acontece é o slider ficar **MORTO**: numa rampa de 45° (piso `0,583`)
    // autorar `0,50` dá folga `0,059` e autorar `0,30` dá `0,058`. Duzentos
    // milímetros de curso, um milímetro de resposta, e nada na tela.
    //
    // ⚠️ **Só com o agachar ARMADO:** em zero a capacidade está desligada e não há
    // defeito nenhum — e um botão que a ligasse pelas costas conflataria *dar um
    // agachar* com *consertar o que ele mede*.
    if info.min_float_known && info.crouch_height > 0.0 {
        let label = if info.crouch_height <= info.min_float_height {
            format!(
                "Fit Crouch to Collider (needs > {:.2} m)",
                info.min_float_height
            )
        } else {
            "Fit Crouch to Collider".to_string()
        };
        let rect = Rect::new(x, yy, w, h);
        let btn = Button::new(ids::INSP_PLAYER_FIT_CROUCH, &label)
            .kind(ButtonKind::Default)
            .state(
                store
                    .button_state(ids::INSP_PLAYER_FIT_CROUCH)
                    .unwrap_or(ButtonState::Normal),
            );
        paint_button(&btn, rect, scene, text_system, theme);
        hit_index.register(ids::INSP_PLAYER_FIT_CROUCH, rect);
        yy += h + Spacing::Sm.px();
    }

    // **A CORRIDA GRAVADA** (W17) — o mesmo desenho do botão acima: *o aviso mora
    // no rótulo do próprio controle*, então o número de segundos viaja no texto e
    // não num readout ao lado.
    //
    // ⚠️ **A AUSÊNCIA dele é o outro readout.** Sem corrida não há o que
    // descartar, e um botão pintado sobre nada seria um controle que não faz nada
    // — a lei do knob morto que esta seção honra em toda row opt-in.
    //
    // ⚠️ **E DESCARTAR TEM VOLTA** (W24): a corrida some do documento mas fica
    // guardada na sessão, e o mesmo lugar da tela passa a oferecer o caminho de
    // volta. Os dois nunca aparecem juntos — *há corrida viva* e *há corrida
    // descartada com a fita vazia* são estados mutuamente exclusivos por
    // construção, e é isso que dispensa qualquer coordenação entre eles.
    let run_button = if info.recorded_run_seconds > 0.0 {
        Some((
            ids::INSP_PLAYER_CLEAR_RUN,
            format!("Clear Recorded Run ({:.1} s)", info.recorded_run_seconds),
        ))
    } else if info.discarded_run_seconds > 0.0 {
        Some((
            ids::INSP_PLAYER_RESTORE_RUN,
            format!(
                "Restore Discarded Run ({:.1} s)",
                info.discarded_run_seconds
            ),
        ))
    } else {
        None
    };
    if let Some((id, label)) = run_button {
        let rect = Rect::new(x, yy, w, h);
        let btn = Button::new(id, &label)
            .kind(ButtonKind::Default)
            .state(store.button_state(id).unwrap_or(ButtonState::Normal));
        paint_button(&btn, rect, scene, text_system, theme);
        hit_index.register(id, rect);
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
