//! **As guardas de COMPOSIÇÃO do planeio** (`W-Glide`) — quem é dono do eixo
//! vertical.
//!
//! ⚠️ **Estes gates existem porque duas mutações SOBREVIVERAM a tudo o resto:**
//! tirar o `!swimming` e tirar o `standing.is_none()` da guarda do planeio
//! deixava os gates da lei, os do produto e os da cena **VERDES**. A lei do
//! planeio está certa nos dois casos — o que se perde é a decisão de **QUEM
//! escreve o eixo vertical neste tique**, e essa decisão não vive na lei: vive
//! no [`crate::player_motor`].
//!
//! ⚠️ **E o nível certo para a testar é ESTE, e não uma poça.** Duas fixtures de
//! água inteiras foram construídas e as duas mediram a coisa errada — a primeira
//! deixava o nadador **sair da água** (o planeio agia legitimamente acima dela),
//! e a segunda mediu o MERGULHO, onde o que muda é a **velocidade de chegada**
//! (2,00 m/s em vez de 2,83), que também é o planeio a fazer o seu trabalho.
//! Uma guarda de composição pergunta *quem escreve o eixo*, e quem responde é a
//! porta que compõe.

use super::*;

const UP: Vec2 = [0.0, 1.0];
const G: Vec2 = [0.0, -9.81];
const DT: f32 = 1.0 / 60.0;
const SPRING: Support = Support::Spring;
const DRY: Buoyed = Buoyed::DRY;
/// Fundo o bastante para armar o nado com o limiar default.
const DEEP: Buoyed = Buoyed(2.0);

/// Uma queda rápida — mais depressa do que qualquer teto destes gates.
const FAST_FALL: Vec2 = [0.0, -12.0];

/// O planeio armado, e mais nada.
fn gliding_cfg() -> PlayerConfig {
    PlayerConfig {
        glide: GlideConfig { fall_speed: 2.0 },
        ..PlayerConfig::STARTING_POINT
    }
}

/// O planeio armado **e o nado também** — o par que a guarda separa.
fn gliding_and_swimming_cfg() -> PlayerConfig {
    PlayerConfig {
        glide: GlideConfig { fall_speed: 2.0 },
        swim: SwimConfig {
            speed: 4.0,
            acceleration: 12.0,
            ..SwimConfig::STARTING_POINT
        },
        ..PlayerConfig::STARTING_POINT
    }
}

/// ⚠️ **A mola COMPRIMIDA, e não o raio à altura de repouso.** A primeira
/// versão desta fixture usava `distance = 0.9` (exatamente a `float_height`), e
/// ali a perna produz correção **zero** — o gate media um tique em que ela não
/// tinha escrito nada, e concluía que o planeio estava a agir por cima dela. O
/// que a guarda protege é o eixo que a mola ESCREVE, então a fixture tem de a
/// pôr a escrever.
fn on_ground() -> GroundSample {
    GroundSample {
        distance: 0.5,
        normal: UP,
        ground_velocity: [0.0, 0.0],
        one_way: false,
    }
}

/// O dedo no pulo, que é o que arma o planeio.
fn held() -> PlayerInput {
    PlayerInput {
        jump: true,
        ..PlayerInput::default()
    }
}

fn step(
    cfg: &PlayerConfig,
    ground: Option<&GroundSample>,
    velocity: Vec2,
    buoyed: Buoyed,
) -> PlayerStep {
    player_motor(
        cfg,
        ground,
        None,
        None,
        None,
        None,
        held(),
        PlayerState::default(),
        velocity,
        G,
        UP,
        DT,
        buoyed,
        SPRING,
    )
}

/// **⚠️ O PLANEIO NÃO ALCANÇA UM NADADOR.**
///
/// ⚠️ **O defeito que a guarda impede é dois-donos, não uma sobreposição
/// inofensiva:** dentro d'água o botão de pulo **já significa subir** — o
/// `swim_rise` lê o MESMO `input.jump` —, então sem ela o mesmo dedo pede as
/// duas coisas e o eixo vertical ganha um freio que ninguém autorou.
///
/// O oráculo é o motor: com o nado a agir, ligar ou desligar o teto de planeio
/// tem de dar **exatamente** o mesmo tique.
#[test]
fn the_glide_never_reaches_a_swimmer() {
    let with_glide = step(&gliding_and_swimming_cfg(), None, FAST_FALL, DEEP);
    let without = step(
        &PlayerConfig {
            glide: GlideConfig::STARTING_POINT,
            ..gliding_and_swimming_cfg()
        },
        None,
        FAST_FALL,
        DEEP,
    );
    assert_eq!(
        with_glide.motor, without.motor,
        "a nadar, o teto de planeio nao pode mudar um bit do motor -- o mesmo \
         dedo estaria a pedir subir E a travar"
    );
}

/// **E o CONTROLE: fora d'água, no mesmo tique, ele age.**
///
/// ⚠️ **Sem este par o gate acima ficaria verde por vácuo** — um planeio que
/// nunca agisse em lado nenhum também passaria nele.
#[test]
fn and_the_same_tick_out_of_the_water_does_glide() {
    let wet = step(&gliding_and_swimming_cfg(), None, FAST_FALL, DEEP);
    let dry = step(&gliding_and_swimming_cfg(), None, FAST_FALL, DRY);
    assert_ne!(
        wet.motor, dry.motor,
        "no ar o mesmo tique TEM de planar -- senao o gate irmao nao prova nada"
    );
}

/// **⚠️ NO TIQUE DA DECOLAGEM, O PULO MANDA.**
///
/// ⚠️ **Este gate nasceu VERMELHO e a wave tinha um defeito real:** no tique em
/// que um pulo sai, a `standing` já é `None` **de propósito** — a perna cala-se
/// para não disputar o eixo com o `boost` do pulo —, então a guarda
/// `standing.is_none()` **deixava o planeio passar**. Medido: com o pé no chão e
/// o botão apertado, o motor levava `+18,26` do pulo **e mais `+10,00` do
/// planeio`. Dez metros por segundo que o jogador não pediu.
///
/// ⚠️ **A cura foi uma sexta guarda (`!jump.takeoff()`), e não afrouxar a lei:** o
/// planeio está certo, o que faltava era a decisão de *quem escreve o eixo neste
/// tique*.
#[test]
fn on_the_takeoff_tick_the_jump_owns_the_axis() {
    let ground = on_ground();
    let with_glide = step(&gliding_cfg(), Some(&ground), FAST_FALL, DRY);
    let without = step(&PlayerConfig::STARTING_POINT, Some(&ground), FAST_FALL, DRY);
    // ⚠️ **A prova de que houve decolagem é o ESTADO**, e não um campo do passo:
    // quem sai do chão fica `airborne`.
    assert!(
        with_glide.state.jump.airborne,
        "a fixture TEM de conter uma decolagem, senao este gate mede outra coisa"
    );
    assert_eq!(
        with_glide.motor, without.motor,
        "no tique da decolagem o pulo ja' escreve o eixo vertical: o planeio nao \
         pode acrescentar um segundo termo"
    );
}

/// **⚠️ COM O PÉ NO CHÃO E SEM DECOLAGEM, A PERNA MANDA.**
///
/// ⚠️ **É preciso um estado com o botão JÁ segurado** (`was_held`), senão a
/// fixture cai no tique da decolagem — que é o gate irmão, e outra pergunta. Sem
/// borda não há pulo, a perna fica armada, e o que se mede é a guarda
/// `standing.is_none()`.
#[test]
fn with_a_foot_on_the_ground_and_no_takeoff_the_leg_owns_the_axis() {
    let ground = on_ground();
    let already_held = PlayerState {
        jump: JumpState {
            was_held: true,
            ..JumpState::default()
        },
        ..PlayerState::default()
    };
    let go = |cfg: &PlayerConfig| {
        player_motor(
            cfg,
            Some(&ground),
            None,
            None,
            None,
            None,
            held(),
            already_held,
            FAST_FALL,
            G,
            UP,
            DT,
            DRY,
            SPRING,
        )
    };
    let with_glide = go(&gliding_cfg());
    let without = go(&PlayerConfig::STARTING_POINT);
    assert!(
        !with_glide.state.jump.airborne,
        "a fixture NAO pode conter uma decolagem -- isso e' o gate irmao"
    );
    assert_eq!(
        with_glide.motor, without.motor,
        "com o pe' no chao a mola ja' escreve o eixo vertical: o planeio nao pode \
         acrescentar um segundo termo"
    );
}

/// **E o CONTROLE: o MESMO tique sem chão plana.**
#[test]
fn and_the_same_tick_without_ground_does_glide() {
    let ground = on_ground();
    let grounded = step(&gliding_cfg(), Some(&ground), FAST_FALL, DRY);
    let airborne = step(&gliding_cfg(), None, FAST_FALL, DRY);
    assert_ne!(
        grounded.motor, airborne.motor,
        "no ar o mesmo tique TEM de planar -- senao o gate irmao nao prova nada"
    );
}
