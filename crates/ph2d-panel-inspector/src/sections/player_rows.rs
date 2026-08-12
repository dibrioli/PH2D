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

use super::player::PlayerRow;
use ph2d_editor_core::ids;

/// **A PERNA** — o que faz o personagem pairar em vez de encostar.
const LEG_ROWS: [PlayerRow; 6] = [
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
    // ⚠️ **Os dois de baixo são o SENSOR, não a mola** — e ficam neste card
    // porque a pergunta que respondem é *onde a perna procura chão*, que é o que
    // os quatro de cima consomem. Um card próprio separaria a geometria do
    // efeito dela, e o artista teria de saber que os dois se falam.
    (
        "Foot Rays",
        ids::INSP_PLAYER_FOOT_SAMPLES,
        "How MANY rays the leg casts. Odd; the middle one breaks ties. \
         1 sinks over gaps your body could span.",
    ),
    (
        "Foot Ray Spread",
        ids::INSP_PLAYER_FOOT_SPREAD,
        "Where the OUTER feet sit, as a fraction of your half-width. 1 = the box edge, \
         0 collapses back to a single ray.",
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
const JUMP_ROWS: [PlayerRow; 9] = [
    (
        "Jump Height (m)",
        ids::INSP_PLAYER_JUMP_HEIGHT,
        "How high a full jump reaches, in metres.",
    ),
    // ⚠️ **Os dois do ar ficam LOGO ABAIXO do primeiro pulo, e não no fim do
    // card:** a pergunta que eles respondem é *quantos pulos, e de que altura*,
    // que é a mesma pergunta da linha de cima. Enterrá-los depois dos seis
    // multiplicadores de tato faria o artista procurá-los no card do perdão.
    (
        "Air Jumps",
        ids::INSP_PLAYER_AIR_JUMPS,
        "Extra jumps after leaving the ground. 0 turns it off; they refill on landing.",
    ),
    (
        "Air Jump Height (m)",
        ids::INSP_PLAYER_AIR_JUMP_H,
        "How high an AIR jump reaches, in metres. Same as above is the Celeste feel; \
         lower is Hollow Knight.",
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
const FORGIVE_ROWS: [PlayerRow; 6] = [
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
        "Corner Rays",
        ids::INSP_PLAYER_CORNER_SAMPLES,
        "How MANY rays scan the ceiling profile. More = a finer ledge edge.",
    ),
    (
        "Corner Look-ahead",
        ids::INSP_PLAYER_CORNER_AHEAD,
        "How many TICKS ahead the ceiling profile looks. 0 = no anticipation.",
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
        "How hard he shoves what he walks into. A dynamic body already pushes through the solver.",
    ),
];

/// **AS PAREDES** (W13) — ⚠️ card PRÓPRIO, e não uma extensão do de PULO: o
/// escorregamento não é um pulo, e o que agrupa estes cinco números é a
/// superfície, não o gesto. As duas primeiras rows nascem em ZERO porque a
/// capacidade é opt-in (ver `WallConfig::STARTING_POINT`).
const WALL_ROWS: [PlayerRow; 8] = [
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
        "Wall Rays",
        ids::INSP_PLAYER_WALL_SAMPLES,
        "How MANY rays the flank casts. Odd; the middle one breaks ties.",
    ),
    (
        "Wall Ray Spread",
        ids::INSP_PLAYER_WALL_SPREAD,
        "Where the OUTER rays sit, as a fraction of your half-height. 1 = the box edge.",
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
        "Swim Speed (m/s)",
        ids::INSP_PLAYER_SWIM_SPEED,
        "How fast he swims, in any direction. 0 = off.",
    ),
    (
        "Swim Accel (m/s2)",
        ids::INSP_PLAYER_SWIM_ACCEL,
        "Authority against the water. Low: he floats up on his own.",
    ),
    (
        "Swim Line (weights)",
        ids::INSP_PLAYER_SWIM_ENTER,
        "Buoyancy he swims at, and rests at. 1 = the water holds him.",
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
const LEDGE_ROWS: [PlayerRow; 4] = [
    (
        "Ledge Grab (m)",
        ids::INSP_PLAYER_LEDGE_GRAB,
        "How far ahead the sensor looks for a lip. 0 = off.",
    ),
    (
        "Grab Height (m)",
        ids::INSP_PLAYER_LEDGE_REACH_Y,
        "How far above his head a lip is still caught.",
    ),
    (
        "Grab Span (m)",
        ids::INSP_PLAYER_LEDGE_SPAN,
        "How wide the sensor is. 0 = a single ray.",
    ),
    (
        "Ledge Speed (m/s)",
        ids::INSP_PLAYER_LEDGE_SPEED,
        "How fast he settles into the hang, and climbs over.",
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
    "Glide Fall (m/s)",
    ids::INSP_PLAYER_GLIDE_FALL,
    "Top descent speed while holding jump in a fall. 0 = off.",
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
/// ⚠️ **Os títulos são os módulos da lei** (`ride` · `walk` · `jump` · o perdão
/// · `react` · `wall` · `dash` · `crouch` · `swim`), não uma arrumação de gosto: quando o artista
/// pergunta *"o que este número faz?"*, a primeira metade da resposta é *"a que
/// pergunta ele pertence"*, e ela passou a estar escrita na tela (Enio,
/// 2026-08-04: *"esse tanto de parâmetros juntos não fica bem; organize-os em
/// cards com um título que facilite o entendimento"*).
pub(crate) const PLAYER_CARDS: [(&str, ph2d_a11y::NodeId, &[PlayerRow]); 11] = [
    ("LEG", ids::INSP_PLAYER_CARD_LEG, &LEG_ROWS),
    ("WALK", ids::INSP_PLAYER_CARD_WALK, &WALK_ROWS),
    ("JUMP", ids::INSP_PLAYER_CARD_JUMP, &JUMP_ROWS),
    ("FORGIVENESS", ids::INSP_PLAYER_CARD_FORGIVE, &FORGIVE_ROWS),
    ("REACTION", ids::INSP_PLAYER_CARD_REACT, &REACT_ROWS),
    ("WALLS", ids::INSP_PLAYER_CARD_WALL, &WALL_ROWS),
    ("DASH", ids::INSP_PLAYER_CARD_DASH, &DASH_ROWS),
    ("CROUCH", ids::INSP_PLAYER_CARD_CROUCH, &CROUCH_ROWS),
    ("SWIM", ids::INSP_PLAYER_CARD_SWIM, &SWIM_ROWS),
    ("LEDGE", ids::INSP_PLAYER_CARD_LEDGE, &LEDGE_ROWS),
    ("GLIDE", ids::INSP_PLAYER_CARD_GLIDE, &GLIDE_ROWS),
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
