//! **AS PAREDES** (W13) — os gates de comportamento, com o rapier de verdade.
//!
//! Duas metades que partilham UMA pergunta (*estou agarrado?*): o **freio** da
//! queda e o **pulo** que sai dali. O plano 06 §4 listava-as como *"cada uma é
//! uma wave própria"*, e a construção mostrou que são a mesma: separá-las daria
//! duas respostas para *o que conta como parede*, e a segunda divergiria da
//! primeira no dia em que alguém mexesse num limiar.
//!
//! # ⚠️ O CONTROLE desta cena é a queda LIVRE (desde o Bug #14)
//!
//! Até 13/11 o corpo do player raspava na parede (atrito de Coulomb contra a normal que o
//! controle aéreo sustenta) e, sem a capacidade, descia só 9 cm em 1 s — a «cola». O corpo do
//! player agora não tem atrito com nada (`player_sem_atrito`): sem a capacidade ele CAI, e a
//! única coisa que o segura numa parede é a lei autorada.
//!
//! A tabela está no `measure_wall`.

use crate::platform_wall_rig as rig_fixture;

use ph2d_core::Vec2;
use ph2d_ecs::{Name, Transform};
use ph2d_physics_ecs::PlayerInput;
use ph2d_physics_ecs::{BodyKind, Collider, ColliderShape, PlatformPlayer, RigidBody};
use rig_fixture::{
    FLOAT_HEIGHT, GAP_CENTER, Rig, START_Y, WALL_FACE, into_wall, pose, rig, rig_gapped,
};

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
/// ⚠️ E o CONTROLE é a **queda livre**: com a capacidade desligada o personagem pressionado
/// contra a parede cai como solto (6,05 m em 1 s, medido) — nada de atrito o segura (Bug #14).
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
        dstuck > dfast,
        "o CONTROLE e' a queda livre: sem a capacidade a parede nao freia ({dstuck:.3} m) e \
         desce mais que o escorregar mais rapido ({dfast:.3} m)"
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
            drive_y: 0.0,
            drive: 1.0,
            jump: true,
            down: false,
            dash: false,
            grab: false,
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
            drive_y: 0.0,
            drive: 1.0,
            jump: true,
            down: false,
            dash: false,
            grab: false,
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

/// **O GATE DO FLANCO: uma parede que a CINTURA não vê continua a segurar.**
///
/// A cena é a parede de sempre com uma **fresta** de 0,8 m atravessada na
/// altura do peito. O corpo mede 1,0 m de caixa, então pé e ombro continuam com
/// 10 cm encostados na pedra — geometria que qualquer jogador lê como *"estou
/// contra a parede"*.
///
/// ⚠️ **O oráculo é o PULO, e não o escorregamento** — a escolha está medida no
/// `measure_wall_flank`. A descida quase não denuncia o defeito (0,0500 →
/// 0,0632 m/tique, medido quando ainda havia a «cola» do atrito, Bug #14) porque o
/// escorregar governa a descida de qualquer jeito; o pulo não: ou a lei vê
/// parede naquele tique, ou o aperto do botão não faz **nada**.
///
/// ⚠️ **E a fresta é 0,8 e não 0,4 por um motivo medido:** abaixo de ~0,70 m o
/// **buffer do pulo** mascara tudo — ele guarda o aperto até o bloco de baixo
/// reaparecer, e um gate escrito ali passaria com o sensor cego.
///
/// **Medido:** com um raio só, `0.000 m`; com o flanco, `2.295 m` — contra
/// `2.162 m` na parede sólida (fixtura da lei velha, `v0` 6,264; a 2,0 da lei nova a sólida sobe 2,06 m).
#[test]
fn the_wall_jump_survives_a_gap_the_waist_falls_into() {
    const GAP: f32 = 0.8;
    let solid = wall_jump_rise(0.0);
    let gapped = wall_jump_rise(GAP);
    assert!(
        solid > 1.5,
        "o CONTROLE: na parede solida o pulo sobe (subiu {solid:.3} m)"
    );
    assert!(
        gapped > solid * 0.8,
        "com {GAP:.1} m de fresta e 10 cm de pe/ombro na pedra, o pulo tem de \
         sair: subiu {gapped:.3} m contra {solid:.3} m na parede solida"
    );
}

/// Desce até a fresta e aperta pulo; devolve quanto subiu a partir dali.
fn wall_jump_rise(gap: f32) -> f32 {
    wall_jump_flight(2.0, gap).0
}

/// O voo de um pulo de parede de `height` m dado na fresta: `(subida, maior velocidade vertical)`.
fn wall_jump_flight(height: f32, gap: f32) -> (f32, f32) {
    let mut r = rig_gapped(3.0, height, gap);
    r.bridge.set_player_input(r.player, into_wall());
    let mut t = 0u64;
    while t < 400 && pose(&r.sim).1 > GAP_CENTER {
        t += 1;
        r.bridge.dispatch(&mut r.sim, true, t);
    }
    let from = pose(&r.sim).1;
    let mut input = into_wall();
    input.jump = true;
    r.bridge.set_player_input(r.player, input);
    let (rise, vy) = flight(&mut r, t);
    (rise - from, vy)
}

/// 90 tiques de pulo segurado contra a parede: `(pico, maior velocidade vertical)` — a
/// velocidade é o deslocamento do tique sobre `dt`.
fn flight(r: &mut Rig, mut t: u64) -> (f32, f32) {
    let mut input = into_wall();
    input.jump = true;
    r.bridge.set_player_input(r.player, input);
    let mut y = pose(&r.sim).1;
    let (mut peak, mut vy) = (y, f32::MIN);
    for _ in 0..90 {
        t += 1;
        r.bridge.dispatch(&mut r.sim, true, t);
        let now = pose(&r.sim).1;
        vy = vy.max((now - y) / DT);
        peak = peak.max(now);
        y = now;
    }
    (peak, vy)
}

const DT: f32 = 1.0 / 60.0;

/// Um pulo do CHÃO de `height` m, a `start_x` da parede, a correr contra ela: o mesmo par.
fn ground_jump_flight(height: f32, start_x: f32) -> (f32, f32) {
    let mut r = ground_rig(height, start_x);
    let t = run(&mut r, PlayerInput::default(), 60, 0);
    let from = pose(&r.sim).1;
    let (peak, vy) = flight(&mut r, t);
    (peak - from, vy)
}

/// A parede com um chão até à face dela (topo em y = 0) e o player pousado em `start_x`.
fn ground_rig(height: f32, start_x: f32) -> Rig {
    let mut r = rig(3.0, height);
    r.player_cfg(|p| p.jump_height = height);
    r.sim.world_mut().spawn((
        Name::new("Ledge floor"),
        RigidBody {
            kind: BodyKind::Static,
        },
        Collider {
            shape: ColliderShape::Cuboid {
                half_x: 5.0,
                half_y: 0.5,
            },
            ..Collider::default()
        },
        Transform::from_translation(Vec2::new(WALL_FACE - 5.0, -0.5)),
    ));
    let mut tr = r
        .sim
        .world_mut()
        .get_mut::<Transform>(r.player)
        .expect("o player tem Transform");
    tr.translation = Vec2::new(start_x, FLOAT_HEIGHT + 0.05);
    r
}

/// **O GATE DA VARREDURA (Bug #11): nenhum pulo, do chão ou da parede, sai mais rápido que o
/// próprio arranque nem sobe acima do próprio ápice** — Jump Height 1,5–2,5 m em passos de
/// 0,05, e o pulo do chão a dois afastamentos da parede.
///
/// ⚠️ Varredura, e não um ponto: o defeito vivia numa JANELA (o tique em que o corpo chega à
/// parede a 6 m/s, penetra 2 cm e o pé de fora da perna nasce dentro dela), e o gate de um ponto
/// só ficou verde a 0,1 m dela. Medido antes da cura: a 2,00 m a parede sólida subia ~11,9 m a
/// 27 m/s; depois, 2,06 m.
#[test]
fn no_jump_leaves_faster_than_its_launch_or_above_its_apex() {
    let g = -ph2d_physics::PhysicsWorld::DEFAULT_GRAVITY_Y;
    let cfg = PlatformPlayer::default().config();
    let mut bad = Vec::new();
    for k in 0..=20 {
        let h = 1.5 + 0.05 * k as f32;
        let v0 = ph2d_platformer::launch_speed(&cfg.jump, g, h, false, DT);
        let cap_v = v0 + g * DT + 0.05;
        let cap_rise = h + 0.25;
        let cases = [
            ("parede", wall_jump_flight(h, 0.0)),
            ("chao a 1,0 m", ground_jump_flight(h, WALL_FACE - 0.2 - 1.0)),
            ("chao a 1,6 m", ground_jump_flight(h, WALL_FACE - 0.2 - 1.6)),
        ];
        for (from, (rise, vy)) in cases {
            if vy > cap_v || rise > cap_rise || rise < h * 0.6 {
                bad.push(format!(
                    "h={h:.2} {from}: subiu {rise:.3} m (teto {cap_rise:.2}), vy max {vy:.2} m/s (teto {cap_v:.2})"
                ));
            }
        }
    }
    assert!(bad.is_empty(), "pulos fora da lei:\n{}", bad.join("\n"));
}

/// **O GATE DO AGARRAR-SE (W23): com o botão apertado ele PARA na parede.**
///
/// ⚠️ **O oráculo é a razão entre dois regimes da MESMA cena**, não uma distância
/// absoluta — a lição que o gate do escorregamento já carrega (nasceu sobre a
/// *cola* do atrito, que o Bug #14 tirou): qualquer número solto não diria nada. O que
/// diz é: **agarrado ele desce muito menos do que escorregando**, e depois de a
/// reserva acabar volta a escorregar.
///
/// **Medido:** com `wall_grab_stamina = 1,0` e `wall_slide_speed = 3,0`, em 60
/// tiques agarrado ele desce uma fração do que desce solto — e o mesmo rig com
/// o botão SOLTO é o controle.
#[test]
fn holding_the_grab_button_stops_him_on_the_wall() {
    let sliding = wall_descent(0.0, false);
    let held = wall_descent(2.0, true);
    let released = wall_descent(2.0, false);
    assert!(
        sliding > 2.0,
        "o CONTROLE: escorregando ele desce de facto ({sliding:.3} m)"
    );
    assert!(
        held < sliding * 0.25,
        "agarrado ele tem de PARAR: desceu {held:.3} m contra {sliding:.3} m \
         escorregando"
    );
    assert!(
        (released - sliding).abs() < 0.05,
        "e com a reserva armada mas o botao SOLTO nada muda: {released:.3} \
         contra {sliding:.3}"
    );
}

/// **E quando a reserva acaba ele volta a escorregar** — a outra metade, e a que
/// impede o agarrar-se de virar uma beirada permanente.
#[test]
fn when_the_reserve_runs_out_he_slides_again() {
    // Reserva curta: 0,5 s de 60 tiques (1 s) — metade agarrado, metade solto.
    let short = wall_descent(0.5, true);
    let long = wall_descent(2.0, true);
    assert!(
        short > long * 2.0,
        "com a reserva curta ele desce muito mais ({short:.3} contra \
         {long:.3} m): a reserva acaba e o escorregamento volta"
    );
}

/// Desce a parede por 60 tiques com a reserva dada; devolve quanto desceu.
fn wall_descent(stamina: f32, grab: bool) -> f32 {
    let mut r = rig(3.0, 0.0);
    r.player_cfg(|p| p.wall_grab_stamina = stamina);
    // Um tique para o sensor ver a parede antes de o botao contar.
    let mut input = into_wall();
    r.bridge.set_player_input(r.player, input);
    let mut t = 0u64;
    for _ in 0..30 {
        t += 1;
        r.bridge.dispatch(&mut r.sim, true, t);
    }
    let from = pose(&r.sim).1;
    input.grab = grab;
    r.bridge.set_player_input(r.player, input);
    for _ in 0..60 {
        t += 1;
        r.bridge.dispatch(&mut r.sim, true, t);
    }
    from - pose(&r.sim).1
}
