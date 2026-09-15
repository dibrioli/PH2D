//! **O QUE A §14 OFERECE** — os nove cards e os números dentro deles.
//!
//! ⚠️ **Corte por RESPONSABILIDADE:** o pai (`player.rs`) responde *como a seção
//! se desenha* e este filho *o que ela oferece*. Os dois crescem por motivos
//! diferentes — o pintor por mudança de layout, a tabela por wave —, e foi a
//! `W-Swim` (o nono card) que cruzou o teto de 600 LOC do painel.
//!
//! ⚠️ **UMA tabela, TRÊS consumidores** (o molde do `SECTIONS` do painel de
//! física): o **pintor** desenha, o **`populate`** registra a dica de hover de
//! cada id, e a **varredura de seam** clica tudo. Uma row nova nasce pintada,
//! com dica e varrida, ou não nasce.
//!
//! Re-exportada pelo pai, então nenhum caminho de chamador muda.

use ph2d_editor_core::widget::Unit;

/// ⭐⭐ **Uma row da §14: rótulo · id · dica · UNIDADE.**
///
/// ⚠️ A dica entra na MESMA tupla e não numa segunda tabela, e a razão é a que este módulo já paga
/// em toda lista: uma row nova nasce com dica, ou não nasce. Uma tabela paralela de tooltips é a
/// que fica incompleta em silêncio — o controlo continua pintado e o artista continua sem saber o
/// que ele faz. ⭐ **A unidade entrou pela mesma porta, em 2026-09-14, e pela mesma razão.**
///
/// ⛔⛔ **A unidade entrou aqui em 2026-09-14, e ela SAIU do rótulo.** Medido com o sistema de
/// texto real, à largura de omissão do Inspector: **20 de 39** rótulos do painel eram CORTADOS, e
/// sem a unidade **fica 1**. Um `"Float Height (m)"` mede `92,1 px` numa coluna de `91,2`.
///
/// ⚠️ **A chave de i18n MANTÉM o sufixo** (`…float_height_m`): ela é um ENDEREÇO, não o texto.
/// Renomeá-la mudaria 31 sítios para dizer a mesma coisa — *um `rename` não distingue um endereço
/// de uma memória*.
pub(crate) type PlayerRow = (
    TextKey,
    ph2d_a11y::NodeId,
    TextKey,
    Option<ph2d_editor_core::widget::Unit>,
);

use ph2d_i18n::TextKey;

/// **A PERNA** — o que faz o personagem pairar em vez de encostar.
const LEG_ROWS: [PlayerRow; 6] = [
    (
        TextKey::new("panel.inspector.player.float_height_m"),
        crate::ids::INSP_PLAYER_FLOAT,
        TextKey::new("panel.inspector.player.how_high_the_character_hovers"),
        Some(Unit::Meters),
    ),
    (
        TextKey::new("panel.inspector.player.cling_distance_m"),
        crate::ids::INSP_PLAYER_CLING,
        TextKey::new("panel.inspector.player.how_far_above_rest_the"),
        Some(Unit::Meters),
    ),
    (
        TextKey::new("panel.inspector.player.leg_stiffness"),
        crate::ids::INSP_PLAYER_STIFFNESS,
        TextKey::new("panel.inspector.player.how_hard_the_leg_pushes"),
        None,
    ),
    // ⚠️ **A dica nomeia o TERCEIRO eixo, e ele foi medido** (W26): baixar este
    // número devolve o quique do pouso E uma subida lenta em rampa, mas só a
    // subida escala com os `Sub-steps` do painel de mundo (`∝ 1/n`) — o quique é
    // independente deles. Sem esta frase o artista baixa o knob, vê o
    // personagem andar sozinho, e não tem como saber que o outro knob paga.
    (
        TextKey::new("panel.inspector.player.leg_damping"),
        crate::ids::INSP_PLAYER_DAMPING,
        TextKey::new("panel.inspector.player.how_fast_the_bounce_dies"),
        None,
    ),
    // ⚠️ **Os dois de baixo são o SENSOR, não a mola** — e ficam neste card
    // porque a pergunta que respondem é *onde a perna procura chão*, que é o que
    // os quatro de cima consomem. Um card próprio separaria a geometria do
    // efeito dela, e o artista teria de saber que os dois se falam.
    (
        TextKey::new("panel.inspector.player.foot_rays"),
        crate::ids::INSP_PLAYER_FOOT_SAMPLES,
        TextKey::new("panel.inspector.player.how_many_rays_the_leg"),
        None,
    ),
    (
        TextKey::new("panel.inspector.player.foot_ray_spread"),
        crate::ids::INSP_PLAYER_FOOT_SPREAD,
        TextKey::new("panel.inspector.player.where_the_outer_feet_sit"),
        None,
    ),
];

/// **ANDAR** — a velocidade, e o que conta como chão.
const WALK_ROWS: [PlayerRow; 5] = [
    (
        TextKey::new("panel.inspector.player.speed_m_s"),
        crate::ids::INSP_PLAYER_SPEED,
        TextKey::new("panel.inspector.player.cruising_speed_measured_relative_to"),
        Some(Unit::MetersPerSecond),
    ),
    (
        TextKey::new("panel.inspector.player.acceleration"),
        crate::ids::INSP_PLAYER_ACCEL,
        TextKey::new("panel.inspector.player.how_quickly_he_reaches_cruising"),
        None,
    ),
    (
        TextKey::new("panel.inspector.player.air_acceleration"),
        crate::ids::INSP_PLAYER_AIR_ACCEL,
        TextKey::new("panel.inspector.player.steering_while_airborne_0_keeps"),
        None,
    ),
    (
        TextKey::new("panel.inspector.player.brake"),
        crate::ids::INSP_PLAYER_BRAKE,
        TextKey::new("panel.inspector.player.how_much_of_that_acceleration"),
        None,
    ),
    (
        TextKey::new("panel.inspector.player.max_slope_deg"),
        crate::ids::INSP_PLAYER_MAX_SLOPE,
        TextKey::new("panel.inspector.player.steepest_ramp_he_stands_on"),
        Some(Unit::Degrees),
    ),
];

/// **PULAR** (W4) — ⚠️ o primeiro é o único que o artista pensa; os seis
/// multiplicadores são o TATO, e o `1.0` de cada um é a gravidade do mundo.
const JUMP_ROWS: [PlayerRow; 9] = [
    (
        TextKey::new("panel.inspector.player.jump_height_m"),
        crate::ids::INSP_PLAYER_JUMP_HEIGHT,
        TextKey::new("panel.inspector.player.how_high_a_full_jump"),
        Some(Unit::Meters),
    ),
    // ⚠️ **Os dois do ar ficam LOGO ABAIXO do primeiro pulo, e não no fim do
    // card:** a pergunta que eles respondem é *quantos pulos, e de que altura*,
    // que é a mesma pergunta da linha de cima. Enterrá-los depois dos seis
    // multiplicadores de tato faria o artista procurá-los no card do perdão.
    (
        TextKey::new("panel.inspector.player.air_jumps"),
        crate::ids::INSP_PLAYER_AIR_JUMPS,
        TextKey::new("panel.inspector.player.extra_jumps_after_leaving_the"),
        None,
    ),
    (
        TextKey::new("panel.inspector.player.air_jump_height_m"),
        crate::ids::INSP_PLAYER_AIR_JUMP_H,
        TextKey::new("panel.inspector.player.how_high_an_air_jump"),
        Some(Unit::Meters),
    ),
    (
        TextKey::new("panel.inspector.player.takeoff_gravity"),
        crate::ids::INSP_PLAYER_TAKEOFF_G,
        TextKey::new("panel.inspector.player.gravity_while_rising_fast_1"),
        None,
    ),
    (
        TextKey::new("panel.inspector.player.takeoff_above_m_s"),
        crate::ids::INSP_PLAYER_TAKEOFF_SPEED,
        TextKey::new("panel.inspector.player.rising_faster_than_this_uses"),
        Some(Unit::MetersPerSecond),
    ),
    (
        TextKey::new("panel.inspector.player.peak_gravity"),
        crate::ids::INSP_PLAYER_PEAK_G,
        TextKey::new("panel.inspector.player.gravity_near_the_top_below"),
        None,
    ),
    (
        TextKey::new("panel.inspector.player.peak_window_m_s"),
        crate::ids::INSP_PLAYER_PEAK_SPEED,
        TextKey::new("panel.inspector.player.how_wide_that_slow_top"),
        Some(Unit::MetersPerSecond),
    ),
    (
        TextKey::new("panel.inspector.player.fall_gravity"),
        crate::ids::INSP_PLAYER_FALL_G,
        TextKey::new("panel.inspector.player.gravity_while_falling_above_1"),
        None,
    ),
    (
        TextKey::new("panel.inspector.player.cut_gravity"),
        crate::ids::INSP_PLAYER_CUT_G,
        TextKey::new("panel.inspector.player.gravity_while_rising_with_the"),
        None,
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
const FORGIVE_ROWS: [PlayerRow; 6] = [
    (
        TextKey::new("panel.inspector.player.coyote_time_s"),
        crate::ids::INSP_PLAYER_COYOTE,
        TextKey::new("panel.inspector.player.grace_after_leaving_the_ground"),
        Some(Unit::Seconds),
    ),
    (
        TextKey::new("panel.inspector.player.jump_buffer_s"),
        crate::ids::INSP_PLAYER_BUFFER,
        TextKey::new("panel.inspector.player.a_press_this_early_still"),
        Some(Unit::Seconds),
    ),
    (
        TextKey::new("panel.inspector.player.corner_reach_m"),
        crate::ids::INSP_PLAYER_CORNER,
        TextKey::new("panel.inspector.player.slide_sideways_up_to_this"),
        Some(Unit::Meters),
    ),
    (
        TextKey::new("panel.inspector.player.corner_rays"),
        crate::ids::INSP_PLAYER_CORNER_SAMPLES,
        TextKey::new("panel.inspector.player.how_many_rays_scan_the"),
        None,
    ),
    (
        TextKey::new("panel.inspector.player.corner_look_ahead"),
        crate::ids::INSP_PLAYER_CORNER_AHEAD,
        TextKey::new("panel.inspector.player.how_many_ticks_ahead_the"),
        None,
    ),
    (
        TextKey::new("panel.inspector.player.lift_momentum_s"),
        crate::ids::INSP_PLAYER_LIFT,
        TextKey::new("panel.inspector.player.keep_a_moving_platform_s"),
        Some(Unit::Seconds),
    ),
];

/// **A REAÇÃO** (W6) — ⚠️ os defaults são OPOSTOS de propósito: o peso volta
/// inteiro (é a física) e o tapete nasce desligado (é de produto).
const REACT_ROWS: [PlayerRow; 3] = [
    (
        TextKey::new("panel.inspector.player.weight_on_ground"),
        crate::ids::INSP_PLAYER_REACT_SUPPORT,
        TextKey::new("panel.inspector.player.how_much_of_his_weight"),
        None,
    ),
    (
        TextKey::new("panel.inspector.player.push_on_ground"),
        crate::ids::INSP_PLAYER_REACT_MOVEMENT,
        TextKey::new("panel.inspector.player.how_much_of_his_walking"),
        None,
    ),
    (
        TextKey::new("panel.inspector.player.push_on_bodies"),
        crate::ids::INSP_PLAYER_REACT_PUSH,
        TextKey::new("panel.inspector.player.how_hard_he_shoves_what"),
        None,
    ),
];

/// **AS PAREDES** (W13) — ⚠️ card PRÓPRIO, e não uma extensão do de PULO: o
/// escorregamento não é um pulo, e o que agrupa estes cinco números é a
/// superfície, não o gesto. As duas primeiras rows nascem em ZERO porque a
/// capacidade é opt-in (ver `WallConfig::STARTING_POINT`).
const WALL_ROWS: [PlayerRow; 8] = [
    (
        TextKey::new("panel.inspector.player.wall_slide_m_s"),
        crate::ids::INSP_PLAYER_WALL_SLIDE,
        TextKey::new("panel.inspector.player.slide_down_a_wall_at"),
        Some(Unit::MetersPerSecond),
    ),
    (
        TextKey::new("panel.inspector.player.wall_jump_m"),
        crate::ids::INSP_PLAYER_WALL_JUMP,
        TextKey::new("panel.inspector.player.how_high_a_jump_off"),
        Some(Unit::Meters),
    ),
    (
        TextKey::new("panel.inspector.player.wall_push_m_s"),
        crate::ids::INSP_PLAYER_WALL_PUSH,
        TextKey::new("panel.inspector.player.how_hard_a_wall_jump"),
        Some(Unit::MetersPerSecond),
    ),
    (
        TextKey::new("panel.inspector.player.wall_lockout_s"),
        crate::ids::INSP_PLAYER_WALL_LOCK,
        TextKey::new("panel.inspector.player.air_control_stays_quiet_this"),
        Some(Unit::Seconds),
    ),
    (
        TextKey::new("panel.inspector.player.wall_reach_m"),
        crate::ids::INSP_PLAYER_WALL_REACH,
        TextKey::new("panel.inspector.player.how_far_past_your_own"),
        Some(Unit::Meters),
    ),
    (
        TextKey::new("panel.inspector.player.wall_rays"),
        crate::ids::INSP_PLAYER_WALL_SAMPLES,
        TextKey::new("panel.inspector.player.how_many_rays_the_flank"),
        None,
    ),
    (
        TextKey::new("panel.inspector.player.wall_ray_spread"),
        crate::ids::INSP_PLAYER_WALL_SPREAD,
        TextKey::new("panel.inspector.player.where_the_outer_rays_sit"),
        None,
    ),
    (
        TextKey::new("panel.inspector.player.wall_grab_s"),
        crate::ids::INSP_PLAYER_WALL_GRAB,
        TextKey::new("panel.inspector.player.hold_r_against_a_wall"),
        Some(Unit::Seconds),
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
        TextKey::new("panel.inspector.player.dash_speed_m_s"),
        crate::ids::INSP_PLAYER_DASH_SPEED,
        TextKey::new("panel.inspector.player.how_fast_the_dash_carries"),
        Some(Unit::MetersPerSecond),
    ),
    (
        TextKey::new("panel.inspector.player.dash_time_s"),
        crate::ids::INSP_PLAYER_DASH_TIME,
        TextKey::new("panel.inspector.player.how_long_it_lasts_speed"),
        Some(Unit::Seconds),
    ),
    (
        TextKey::new("panel.inspector.player.dash_cooldown_s"),
        crate::ids::INSP_PLAYER_DASH_COOL,
        TextKey::new("panel.inspector.player.recovery_after_it_ends_before"),
        Some(Unit::Seconds),
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
        TextKey::new("panel.inspector.player.crouch_height_m"),
        crate::ids::INSP_PLAYER_CROUCH_HEIGHT,
        TextKey::new("panel.inspector.player.how_low_he_floats_while"),
        Some(Unit::Meters),
    ),
    (
        TextKey::new("panel.inspector.player.crouch_speed_m_s"),
        crate::ids::INSP_PLAYER_CROUCH_SPEED,
        TextKey::new("panel.inspector.player.how_fast_he_walks_while"),
        Some(Unit::MetersPerSecond),
    ),
];

/// **O NADO** (W-Swim) — ⚠️ card próprio, e a primeira row nasce em ZERO porque
/// a capacidade é opt-in, como o arranque e o agachar ao lado.
///
/// ⚠️ **A terceira row conta PESOS, não metros**, e a dica dela diz isso: a
/// mesma altura de água significa coisas diferentes em cada poça (depende das
/// duas densidades), enquanto *"a água sozinha me sustenta"* significa a mesma
/// coisa em todas. É o único número desta seção cuja UNIDADE não é m, s ou m/s.
///
/// ⚠️ **E ela responde a DUAS perguntas de propósito** — a porta (*quando começo
/// a nadar?*) e o repouso (*onde fico quando solto os controlos?*). Separá-las
/// em duas rows daria um par cujo valor certo de uma é função da outra
/// (`feedback_ergonomics_verdict_is_a_design_bug`) e cujo caso degenerado — uma
/// linha que o fluido não alcança — faria o nadador remar para baixo para
/// sempre. Com um número só esse estado é **inexprimível**.
const SWIM_ROWS: [PlayerRow; 3] = [
    (
        TextKey::new("panel.inspector.player.swim_speed_m_s"),
        crate::ids::INSP_PLAYER_SWIM_SPEED,
        TextKey::new("panel.inspector.player.how_fast_he_swims_in"),
        Some(Unit::MetersPerSecond),
    ),
    (
        TextKey::new("panel.inspector.player.swim_accel_m_s2"),
        crate::ids::INSP_PLAYER_SWIM_ACCEL,
        TextKey::new("panel.inspector.player.authority_against_the_water_low"),
        Some(Unit::MetersPerSecondSquared),
    ),
    (
        TextKey::new("panel.inspector.player.swim_line_weights"),
        crate::ids::INSP_PLAYER_SWIM_ENTER,
        TextKey::new("panel.inspector.player.buoyancy_he_swims_at_and"),
        None,
    ),
];

/// **A BEIRADA** (W-Ledge) — ⚠️ card próprio, e a primeira row nasce em ZERO
/// porque a capacidade é opt-in, como o nado e o arranque ao lado.
///
/// ⚠️ **Eram DUAS, e a metade sobre o alcance foi REFUTADA** (`W-LedgeSensor`).
/// Ela dizia *"o alcance é uma grandeza só — a janela acima da cabeça E a
/// distância à frente"*, e os motores 2D que shipam separam-nos: o **GDevelop**
/// expõe `Grab tolerance` (X) **e** `Grab offset` (Y, *"to match character
/// animation"*), o **Corgi Engine** expõe *origem e comprimento* do raycast. O
/// eixo Y é independente **pela ARTE**, e o `Span` existe porque um raio único
/// acha o lábio num `x` só.
///
/// ⚠️ **A metade sobre a VELOCIDADE continua de pé, e por isso são três e não
/// quatro:** acomodar e subir são o mesmo gesto de braço, e dois números
/// seriam um par cujo valor certo de um é função do outro — o defeito de
/// ergonomia que este módulo já nomeou no `swim_enter`.
const LEDGE_ROWS: [PlayerRow; 5] = [
    (
        TextKey::new("panel.inspector.player.ledge_grab_m"),
        crate::ids::INSP_PLAYER_LEDGE_GRAB,
        TextKey::new("panel.inspector.player.how_far_ahead_the_sensor"),
        Some(Unit::Meters),
    ),
    (
        TextKey::new("panel.inspector.player.grab_window_m"),
        crate::ids::INSP_PLAYER_LEDGE_REACH_Y,
        TextKey::new("panel.inspector.player.how_tall_the_catch_window"),
        Some(Unit::Meters),
    ),
    (
        TextKey::new("panel.inspector.player.grab_span_m"),
        crate::ids::INSP_PLAYER_LEDGE_SPAN,
        TextKey::new("panel.inspector.player.how_wide_the_sensor_is"),
        Some(Unit::Meters),
    ),
    (
        TextKey::new("panel.inspector.player.grab_offset_y_m"),
        crate::ids::INSP_PLAYER_LEDGE_OFFSET_Y,
        TextKey::new("panel.inspector.player.slides_the_sensor_up_or"),
        Some(Unit::Meters),
    ),
    (
        TextKey::new("panel.inspector.player.ledge_speed_m_s"),
        crate::ids::INSP_PLAYER_LEDGE_SPEED,
        TextKey::new("panel.inspector.player.how_fast_he_settles_into"),
        Some(Unit::MetersPerSecond),
    ),
];

/// **O PLANEIO** (`W-Glide`) — uma row, e a razão de ser uma só.
///
/// ⚠️ **Um TETO, e nenhum knob de "quanto tempo"**: o planeio dura enquanto o
/// dedo dura (é um regime, como o agarrar-se), então um segundo número que o
/// limitasse seria uma resposta a uma pergunta que o botão já responde.
///
/// ⚠️ **E não há knob de BOTÃO**: é o pulo, segurado na queda, que é o idioma de
/// Kirby e Yoshi — e ele COMPÕE com o pulo do ar em vez de brigar, porque um é
/// borda e o outro é nível.
const GLIDE_ROWS: [PlayerRow; 1] = [(
    TextKey::new("panel.inspector.player.glide_fall_m_s"),
    crate::ids::INSP_PLAYER_GLIDE_FALL,
    TextKey::new("panel.inspector.player.top_descent_speed_while_holding"),
    Some(Unit::MetersPerSecond),
)];

/// **O TETO DE QUEDA** (`W-Fall`) — a velocidade TERMINAL do personagem.
///
/// ⚠️ **Card PRÓPRIO, e não uma row dentro do GLIDE**, ainda que os dois sejam
/// tetos da mesma velocidade e componham por uma porta só
/// (`ph2d_platformer::descent_ceiling`): o planeio dura **enquanto o dedo
/// dura** e este vale **sempre**, e o card é onde o artista lê *o que está a
/// autorar*. Juntá-los pediria que ele descobrisse a diferença lendo a dica.
///
/// ⚠️ **Sem teto no valor DIGITÁVEL, e o §0 é o motivo:** medido pela porta do
/// produto (`ph2d-physics-ecs/tests/it/measure_terminal.rs`), uma queda livre de
/// mil metros chega a **142,57 m/s aos 8 s** e continua a crescer — o número
/// que a faixa do slider tem de conseguir descrever é o da MEDIÇÃO, não um
/// redondo confortável.
const FALL_ROWS: [PlayerRow; 1] = [(
    TextKey::new("panel.inspector.player.max_fall_m_s"),
    crate::ids::INSP_PLAYER_MAX_FALL,
    TextKey::new("panel.inspector.player.terminal_speed_the_fall_never"),
    Some(Unit::MetersPerSecond),
)];

/// **A TABELA da §14** — onze cards, e os números dentro deles.
///
/// ⚠️ **A contagem NÃO está escrita aqui**, e é uma correção: o doc dizia *"oito
/// cards, e os vinte e quatro números"* enquanto a tabela já pintava **33** — um
/// número à mão ao lado da lista que o produz só sabe envelhecer. Quem conta é a
/// [`player_row_count`], e quem a afirma é o gate de varredura do seam.
///
/// ⚠️ **UMA tabela, TRÊS consumidores** (o molde do `SECTIONS` do painel de
/// física): o **pintor** desenha, o **`populate`** registra a dica de hover de
/// cada id, e a **varredura de seam** clica tudo. Uma row nova nasce pintada,
/// com dica e varrida, ou não nasce.
///
/// ⚠️ **Os títulos são os módulos da lei** (`ride` · `walk` · `jump` · o perdão · `react` · `wall`
/// · `dash` · `crouch` · `swim` · `ledge` · `glide` · `fall`), não uma arrumação de gosto: quando o
/// artista
/// pergunta *"o que este número faz?"*, a primeira metade da resposta é *"a que
/// pergunta ele pertence"*, e ela passou a estar escrita na tela (Enio,
/// 2026-08-04: *"esse tanto de parâmetros juntos não fica bem; organize-os em
/// cards com um título que facilite o entendimento"*).
pub(crate) const PLAYER_CARDS: [(TextKey, ph2d_a11y::NodeId, &[PlayerRow]); 12] = [
    (
        TextKey::new("panel.inspector.player.leg"),
        crate::ids::INSP_PLAYER_CARD_LEG,
        &LEG_ROWS,
    ),
    (
        TextKey::new("panel.inspector.player.walk"),
        crate::ids::INSP_PLAYER_CARD_WALK,
        &WALK_ROWS,
    ),
    (
        TextKey::new("panel.inspector.player.jump"),
        crate::ids::INSP_PLAYER_CARD_JUMP,
        &JUMP_ROWS,
    ),
    (
        TextKey::new("panel.inspector.player.forgiveness"),
        crate::ids::INSP_PLAYER_CARD_FORGIVE,
        &FORGIVE_ROWS,
    ),
    (
        TextKey::new("panel.inspector.player.reaction"),
        crate::ids::INSP_PLAYER_CARD_REACT,
        &REACT_ROWS,
    ),
    (
        TextKey::new("panel.inspector.player.walls"),
        crate::ids::INSP_PLAYER_CARD_WALL,
        &WALL_ROWS,
    ),
    (
        TextKey::new("panel.inspector.player.dash"),
        crate::ids::INSP_PLAYER_CARD_DASH,
        &DASH_ROWS,
    ),
    (
        TextKey::new("panel.inspector.player.crouch"),
        crate::ids::INSP_PLAYER_CARD_CROUCH,
        &CROUCH_ROWS,
    ),
    (
        TextKey::new("panel.inspector.player.swim"),
        crate::ids::INSP_PLAYER_CARD_SWIM,
        &SWIM_ROWS,
    ),
    (
        TextKey::new("panel.inspector.player.ledge"),
        crate::ids::INSP_PLAYER_CARD_LEDGE,
        &LEDGE_ROWS,
    ),
    (
        TextKey::new("panel.inspector.player.glide"),
        crate::ids::INSP_PLAYER_CARD_GLIDE,
        &GLIDE_ROWS,
    ),
    (
        TextKey::new("panel.inspector.player.fall"),
        crate::ids::INSP_PLAYER_CARD_FALL,
        &FALL_ROWS,
    ),
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
