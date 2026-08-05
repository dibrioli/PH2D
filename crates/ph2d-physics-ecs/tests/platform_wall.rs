//! **AS PAREDES** (W13) — os gates de comportamento, com o rapier de verdade.
//!
//! Duas metades que partilham UMA pergunta (*estou agarrado?*): o **freio** da
//! queda e o **pulo** que sai dali. O plano 06 §4 listava-as como *"cada uma é
//! uma wave própria"*, e a construção mostrou que são a mesma: separá-las daria
//! duas respostas para *o que conta como parede*, e a segunda divergiria da
//! primeira no dia em que alguém mexesse num limiar.
//!
//! # ⚠️ O CONTROLE desta cena não é uma queda livre — é a COLA
//!
//! O primeiro corte destes gates supunha que, com a capacidade desligada, o
//! personagem cairia. **Ele não cai:** medido, desce 9 cm em um segundo inteiro,
//! porque o atrito contra a normal que o controle aéreo sustenta mais a
//! gravidade do ÁPICE (metade do peso) o seguram. O oráculo escrito sobre a
//! premissa errada nasceu vermelho sobre produto correto — e foi ele que
//! derrubou a primeira versão da LEI, que era um teto e nunca dispararia.
//!
//! A tabela está no `measure_wall`.

#[path = "platform_wall_rig.rs"]
mod rig_fixture;

use ph2d_physics_ecs::PlayerInput;
use rig_fixture::{Rig, START_Y, into_wall, pose, rig};

/// Corre `ticks` tiques com a entrada dada, a partir de `from`.
fn run(r: &mut Rig, input: PlayerInput, ticks: u64, from: u64) -> u64 {
    r.bridge.set_player_input(r.player, input);
    let mut t = from;
    for _ in 0..ticks {
        t += 1;
        r.bridge.dispatch(&mut r.sim, true, t);
    }
    t
}

/// **O GATE DA WAVE (metade 1): o escorregamento é a VELOCIDADE autorada.**
///
/// ⚠️ **O oráculo é a razão entre DOIS valores do knob**, não uma distância
/// absoluta: dobrar `wall_slide_speed` tem de dobrar a descida. É o que separa
/// *"a assistência faz alguma coisa"* de *"o número que o artista escreveu É o
/// número que o produto honra"* — e nenhum bug que apenas o faça cair satisfaz
/// isto.
///
/// ⚠️ E o CONTROLE é a **COLA**: com a capacidade desligada o personagem
/// pressionado contra a parede quase não desce (9 cm em 1 s, medido). É esse
/// estado que a wave existe para substituir, e é ele que torna um teto inerte.
#[test]
fn the_slide_descends_at_the_authored_speed() {
    let stuck = {
        let mut r = rig(0.0, 0.0);
        run(&mut r, into_wall(), 60, 0);
        pose(&r.sim).1
    };
    let slow = {
        let mut r = rig(3.0, 0.0);
        run(&mut r, into_wall(), 60, 0);
        pose(&r.sim).1
    };
    let fast = {
        let mut r = rig(6.0, 0.0);
        run(&mut r, into_wall(), 60, 0);
        pose(&r.sim).1
    };

    let (dstuck, dslow, dfast) = (START_Y - stuck, START_Y - slow, START_Y - fast);
    assert!(
        dstuck < 0.3,
        "o CONTROLE e' a cola: sem a capacidade ele quase nao desce, e desceu {dstuck:.3} m"
    );
    assert!(
        dslow > 2.0,
        "com 3 m/s ele tem de descer da ordem de 3 m em 1 s: desceu {dslow:.3} m"
    );
    let ratio = dfast / dslow;
    assert!(
        (1.7..2.3).contains(&ratio),
        "dobrar o knob tem de dobrar a descida: {dslow:.3} -> {dfast:.3} (razao {ratio:.2})"
    );
}

/// **Raspar não é agarrar-se** — sem empurrar contra a parede, a queda é a de
/// sempre.
///
/// ⚠️ É o gate que impede a assistência de virar uma armadilha: num pulo
/// horizontal o personagem passa raspando por paredes o tempo todo, e grudar em
/// cada uma leria como o controle a travar.
#[test]
fn falling_past_a_wall_without_pushing_is_a_normal_fall() {
    let mut passive = rig(3.0, 0.0);
    run(&mut passive, PlayerInput::default(), 60, 0);
    let (_, passive_y) = pose(&passive.sim);

    let mut falling = rig(0.0, 0.0);
    run(&mut falling, PlayerInput::default(), 60, 0);
    let (_, fell_y) = pose(&falling.sim);

    assert!(
        (passive_y - fell_y).abs() < 0.05,
        "sem empurrar, a capacidade tem de ser inerte: {passive_y:.3} contra {fell_y:.3}"
    );
}

/// **O GATE DA WAVE (metade 2): o pulo de parede sobe E afasta.**
///
/// ⚠️ **As duas metades num gate só**, porque um pulo de parede que sobe e não
/// afasta é indistinguível de um pulo normal dado no ar — e é justamente o que
/// um `away` esquecido produziria.
///
/// ⚠️ **O botão fica SEGURADO**, e o primeiro corte deste gate não o segurava:
/// solto ao terceiro tique, a altura variável cortava o próprio pulo
/// (`cut_gravity = 4`, medido em 39,6 m/s² de desaceleração) e o gate reprovava
/// um produto correto. Um jogador segura; a fixture tem de segurar também.
///
/// **Medido (2026-08-05, altura autorada 2,0 m, jogador a segurar a direção da
/// parede):** com o silêncio do controle aéreo, sobe **1,93 m** e afasta
/// **1,17 m**; sem ele, **1,53 m** e **0,44 m**.
#[test]
fn a_wall_jump_goes_up_and_away() {
    let mut r = rig(3.0, 2.0);
    // Agarra-se primeiro: a lei exige estar a DESCER contra a parede.
    let t = run(&mut r, into_wall(), 30, 0);
    let (x0, y0) = pose(&r.sim);

    r.bridge.set_player_input(
        r.player,
        PlayerInput {
            drive: 1.0,
            jump: true,
            down: false,
            dash: false,
        },
    );
    let (mut peak, mut far) = (y0, x0);
    let mut t = t;
    for k in 0..45 {
        t += 1;
        r.bridge.dispatch(&mut r.sim, true, t);
        // Solta o pulo já no fim da subida, e continua a empurrar para a parede
        // — que é o caso em que o controle aéreo desfaria o empurrão.
        if k == 24 {
            r.bridge.set_player_input(r.player, into_wall());
        }
        let (x, y) = pose(&r.sim);
        peak = peak.max(y);
        far = far.min(x);
    }

    assert!(
        peak > y0 + 1.5,
        "o pulo de parede tem de SUBIR perto da altura autorada: pico {peak:.3} contra {y0:.3}"
    );
    assert!(
        x0 - far > 1.0,
        "e tem de AFASTAR: chegou a {far:.3}, partindo de {x0:.3}"
    );
}

/// **Sem a capacidade armada, o mesmo aperto no ar não faz nada.**
///
/// ⚠️ O controle que prova que o pulo de parede não é um pulo duplo disfarçado:
/// com `wall_jump_height = 0` o personagem continua a cair, e é a parede que
/// está a oferecer o pulo, não o ar.
#[test]
fn without_the_capability_the_same_press_does_nothing() {
    let mut r = rig(3.0, 0.0);
    let t = run(&mut r, into_wall(), 30, 0);
    let (_, y0) = pose(&r.sim);

    let mut peak = y0;
    let mut t = run(
        &mut r,
        PlayerInput {
            drive: 1.0,
            jump: true,
            down: false,
            dash: false,
        },
        3,
        t,
    );
    for _ in 0..40 {
        t += 1;
        r.bridge.dispatch(&mut r.sim, true, t);
        let (_, y) = pose(&r.sim);
        peak = peak.max(y);
    }
    assert!(
        peak < y0 + 0.05,
        "sem altura autorada a parede nao oferece pulo nenhum: pico {peak:.3} contra {y0:.3}"
    );
}
