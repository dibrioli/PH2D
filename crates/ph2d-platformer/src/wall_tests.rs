//! Os gates das PAREDES (W13) — irmao de `wall.rs` pelo teto de LOC (HR-18).
//!
//! ⚠️ Continua sendo o mod FILHO (`#[path]`), entao o `use super::*` alcanca o
//! que e' privado — a mesma escolha do `crouch_tests`.

use super::*;
use crate::WalkConfig;
const UP: Vec2 = [0.0, 1.0];

fn cfg(slide: f32, height: f32) -> PlayerConfig {
    PlayerConfig {
        wall: WallConfig {
            slide_speed: slide,
            jump_height: height,
            ..WallConfig::STARTING_POINT
        },
        ..PlayerConfig::STARTING_POINT
    }
}

/// Um flanco em que só a CINTURA viu alguma coisa — a forma que o sensor
/// tinha antes de ele olhar o flanco todo, e a que os gates de sempre usam.
fn only_mid(normal: Vec2) -> WallProbe {
    WallProbe::from_hits(
        1.0,
        &[
            Some(WallHit {
                distance: 0.05,
                normal,
            }),
            None,
            None,
        ],
    )
}

/// Uma parede à direita, normal apontando para a esquerda.
fn right_wall() -> WallProbe {
    only_mid([-1.0, 0.0])
}

/// **A régua é a da PERNA**, e este gate é o que impede alguém de escrever
/// um segundo ângulo: uma rampa que a caminhada ACEITA nunca é parede.
#[test]
fn a_surface_the_leg_accepts_is_never_a_wall() {
    let c = cfg(4.0, 2.0);
    // 30°: dentro do `max_slope` default (45°), logo é chão, não parede.
    let ramp = only_mid([-0.5, 0.866]);
    assert!(cling(&c, Some(&ramp), 1.0, -5.0, UP).is_none());
    // E a vertical é parede.
    assert!(cling(&c, Some(&right_wall()), 1.0, -5.0, UP).is_some());
}

/// **Agarrar-se exige EMPURRAR contra ela** — raspar não conta.
#[test]
fn brushing_a_wall_is_not_clinging() {
    let c = cfg(4.0, 2.0);
    let w = right_wall();
    assert!(
        cling(&c, Some(&w), -1.0, -5.0, UP).is_none(),
        "empurrando para LONGE da parede"
    );
    assert!(
        cling(&c, Some(&w), 0.0, -5.0, UP).is_none(),
        "sem empurrar nada"
    );
    assert!(cling(&c, Some(&w), 1.0, -5.0, UP).is_some());
}

/// **E exige DESCER** — subindo, a parede não tem nada a fazer.
#[test]
fn rising_past_a_wall_is_not_clinging() {
    let c = cfg(4.0, 2.0);
    let w = right_wall();
    assert!(cling(&c, Some(&w), 1.0, 5.0, UP).is_none(), "subindo");
    assert!(cling(&c, Some(&w), 1.0, 0.0, UP).is_none(), "no apice");
    assert!(cling(&c, Some(&w), 1.0, -0.1, UP).is_some());
}

/// **O escorregamento DEFINE a velocidade, nas DUAS direções.**
///
/// ⚠️ A metade de baixo (`-1 → -3`) é a que a medição obrigou: com um teto
/// só, um personagem colado à parede por atrito nunca era solto, e o knob
/// era um número que não fazia nada. Ver o doc da função.
#[test]
fn the_slide_sets_the_speed_in_both_directions() {
    let c = WallConfig {
        slide_speed: 3.0,
        ..WallConfig::STARTING_POINT
    };
    let braked = wall_slide(&c, true, false, -9.0, UP);
    assert!(
        (braked.boost[1] - 6.0).abs() < 1.0e-5,
        "de -9 para -3 sao +6: {:?}",
        braked.boost
    );
    let released = wall_slide(&c, true, false, -1.0, UP);
    assert!(
        (released.boost[1] + 2.0).abs() < 1.0e-5,
        "de -1 para -3 sao -2 -- o COLADO tem de ser solto: {:?}",
        released.boost
    );
    assert_eq!(braked.accel, [0.0, 0.0], "e' um boost, nunca uma forca");
}

/// **Desligado é desligado, AO BIT** — o zero de `slide_speed` não é um teto
/// muito alto, é a ausência da assistência.
#[test]
fn a_zero_slide_speed_is_the_world_untouched() {
    let c = WallConfig {
        slide_speed: 0.0,
        ..WallConfig::STARTING_POINT
    };
    assert_eq!(wall_slide(&c, true, false, -40.0, UP), Motor::default());
}

/// **O empurrão sai pela NORMAL**, e é isso que faz uma parede inclinada
/// lançar para onde ela aponta.
#[test]
fn the_launch_leaves_along_the_normal() {
    let c = WallConfig {
        jump_height: 2.0,
        jump_push: 6.0,
        ..WallConfig::STARTING_POINT
    };
    let w = cling(&cfg(4.0, 2.0), Some(&right_wall()), 1.0, -5.0, UP).expect("agarrado");
    let l = wall_launch(&c, Some(&w)).expect("ha' pulo a oferecer");
    assert!(
        (l.away[0] + 6.0).abs() < 1.0e-5,
        "para a ESQUERDA: {:?}",
        l.away
    );
    assert!(l.away[1].abs() < 1.0e-5);
    assert_eq!(l.height, 2.0);
    assert_eq!(
        l.lockout,
        WallConfig::STARTING_POINT.jump_lockout,
        "a oferta carrega o silencio do controle aereo"
    );
}

/// **Sem altura autorada a parede não oferece pulo nenhum** — e o
/// escorregamento continua a valer sozinho.
#[test]
fn a_wall_with_no_jump_height_offers_nothing() {
    let c = WallConfig {
        jump_height: 0.0,
        ..WallConfig::STARTING_POINT
    };
    let w = cling(&cfg(4.0, 0.0), Some(&right_wall()), 1.0, -5.0, UP).expect("agarrado");
    assert!(wall_launch(&c, Some(&w)).is_none());
}

/// **O sensor não é castado onde não pode agir** — a porta única.
#[test]
fn the_probe_is_only_wanted_where_it_can_act() {
    let off = WallConfig::STARTING_POINT;
    assert!(!off.armed(), "o ponto de partida nasce DESLIGADO");
    assert!(!wall_probe_wanted(&off, false, 1.0));

    let on = WallConfig {
        slide_speed: 4.0,
        ..off
    };
    assert!(wall_probe_wanted(&on, false, 1.0));
    assert!(!wall_probe_wanted(&on, true, 1.0), "no chao, nao");
    assert!(!wall_probe_wanted(&on, false, 0.0), "sem empurrar, nao");
}

/// ⚠️ **Uma normal degenerada é RECUSADA**, ao contrário do chão que a trata
/// como plano: ali a suposição menos daninha empurra o personagem para fora,
/// aqui ela o agarraria ao que quer que ele esteja atravessado.
#[test]
fn a_degenerate_normal_is_refused() {
    let c = cfg(4.0, 2.0);
    let w = only_mid([0.0, 0.0]);
    assert!(cling(&c, Some(&w), 1.0, -5.0, UP).is_none());
}

/// O `max_slope` autorado MOVE a fronteira da parede — a prova de que a
/// régua é mesmo a da perna, e não uma cópia.
#[test]
fn the_authored_max_slope_moves_the_wall_boundary() {
    // 60° de inclinação: normal a 60° do `up`.
    let steep = only_mid([-0.866, 0.5]);
    let mut c = cfg(4.0, 2.0);
    c.walk = WalkConfig {
        max_slope_deg: 45.0,
        ..c.walk
    };
    assert!(
        cling(&c, Some(&steep), 1.0, -5.0, UP).is_some(),
        "com o limite em 45, uma rampa de 60 e' parede"
    );
    c.walk = WalkConfig {
        max_slope_deg: 70.0,
        ..c.walk
    };
    assert!(
        cling(&c, Some(&steep), 1.0, -5.0, UP).is_none(),
        "com o limite em 70 ela volta a ser CHAO, e chao nao se agarra"
    );
}

// ── O FLANCO (W13, o item que ficou aberto) ──────────────────────────────────

/// Um flanco montado à mão: `(distância, normal)` por altura, na ordem de
/// [`wall_offsets`] — cintura, pés, ombros.
fn flank(hits: [Option<(f32, Vec2)>; WALL_SAMPLES]) -> WallProbe {
    WallProbe::from_hits(
        1.0,
        &hits.map(|h| h.map(|(distance, normal)| WallHit { distance, normal })),
    )
}

/// **O sensor cobre o flanco INTEIRO, e a cintura é a PRIMEIRA.**
///
/// ⚠️ Duas afirmações, e a segunda é que torna a wave segura: a ordem decide o
/// desempate no [`cling`], então numa parede plana (as três distâncias iguais) a
/// resposta continua sendo a da cintura — que é a que sempre foi.
#[test]
fn the_flank_is_sampled_from_foot_to_shoulder_with_the_waist_first() {
    let all = wall_offsets(0.5, WALL_SAMPLES, 1.0);
    let offs = &all[..WALL_SAMPLES];
    assert_eq!(offs.len(), WALL_SAMPLES);
    assert!(
        (offs[0]).abs() < 1.0e-9,
        "a cintura tem de ser a PRIMEIRA (ela desempata): {offs:?}"
    );
    let lo = offs.iter().copied().fold(f32::INFINITY, f32::min);
    let hi = offs.iter().copied().fold(f32::NEG_INFINITY, f32::max);
    assert!(
        (lo + 0.5).abs() < 1.0e-6 && (hi - 0.5).abs() < 1.0e-6,
        "as amostras tem de alcancar as duas pontas da caixa: {offs:?}"
    );
}

/// **O GATE DA WAVE: uma parede que a CINTURA não vê continua a ser parede.**
///
/// ⚠️ Este é o defeito que estava aberto desde a W13 e cujo preço foi medido no
/// `measure_wall_flank`: com um raio só, uma fresta de 0,75 m num corpo de 1,0 m
/// **recusava o pulo de parede por inteiro** (0,000 m contra 2,162 m), com 12,5
/// cm de pé E de ombro ainda encostados.
#[test]
fn a_wall_the_waist_misses_is_still_a_wall() {
    let c = cfg(4.0, 2.0);
    let gap = flank([None, Some((0.05, [-1.0, 0.0])), Some((0.05, [-1.0, 0.0]))]);
    let got = cling(&c, Some(&gap), 1.0, -5.0, UP).expect("pes e ombros encostados = parede");
    assert_eq!(got.normal, [-1.0, 0.0]);
    assert_eq!(got.side, 1.0);
}

/// **Uma rampa aos pés não CEGA a parede no tronco** — o caso que um raio só não
/// tinha como resolver.
///
/// ⚠️ E ele é o argumento inteiro para a redução morar na LEI e não na ponte: a
/// rampa só é descartável porque quem escolhe já sabe o que a perna aceita. Uma
/// ponte que ficasse com *a mais próxima* devolveria a rampa e o personagem não
/// se agarraria a nada.
#[test]
fn a_ramp_at_the_feet_does_not_blind_the_wall_at_the_torso() {
    let c = cfg(4.0, 2.0);
    // Os pes veem a rampa de 30 graus, MAIS PERTO que a parede do tronco.
    let mixed = flank([Some((0.09, [-1.0, 0.0])), Some((0.01, [-0.5, 0.866])), None]);
    let got = cling(&c, Some(&mixed), 1.0, -5.0, UP).expect("o tronco encosta numa parede");
    assert_eq!(
        got.normal,
        [-1.0, 0.0],
        "a rampa e' mais perto, mas a perna a ACEITA — ela nao e' parede"
    );
}

/// **De duas paredes, a mais PRÓXIMA** — e num empate a cintura.
#[test]
fn the_nearest_wall_wins_and_a_tie_goes_to_the_waist() {
    let c = cfg(4.0, 2.0);
    // Empate: as tres normais sao distinguiveis, e a resposta tem de ser a
    // primeira da lista.
    let flat = flank([
        Some((0.05, [-1.0, 0.0])),
        Some((0.05, [-0.99, -0.141])),
        Some((0.05, [-0.99, 0.141])),
    ]);
    assert_eq!(
        cling(&c, Some(&flat), 1.0, -5.0, UP).map(|s| s.normal),
        Some([-1.0, 0.0]),
        "parede plana: a cintura desempata, e a resposta e' a de sempre"
    );
    // Sem empate: o ombro esta' mais perto.
    let jut = flank([
        Some((0.09, [-1.0, 0.0])),
        None,
        Some((0.02, [-0.99, 0.141])),
    ]);
    assert_eq!(
        cling(&c, Some(&jut), 1.0, -5.0, UP).map(|s| s.normal),
        Some([-0.99, 0.141]),
        "o que esta' mais perto e' o que o corpo encosta primeiro"
    );
}

/// **Uma amostra DEGENERADA é pulada, nunca escolhida.**
///
/// ⚠️ `distance == 0` significa que o raio nasceu dentro da forma, e ali a
/// normal pode ser o vetor nulo (é o contrato do `CastHit`). Sem esta regra o
/// pé um triz afundado venceria a corrida do *mais próximo* com uma medição que
/// não mede nada, e mataria um agarrar-se que a cintura concedia — o gate
/// dedicado que a camada de defesa pedia
/// ([[feedback_layered_defenses_need_per_layer_gates]]).
#[test]
fn a_degenerate_sample_is_skipped_not_chosen() {
    let c = cfg(4.0, 2.0);
    let sunk = flank([Some((0.05, [-1.0, 0.0])), Some((0.0, [0.0, 0.0])), None]);
    assert_eq!(
        cling(&c, Some(&sunk), 1.0, -5.0, UP).map(|s| s.normal),
        Some([-1.0, 0.0]),
        "o pe afundado nao mede superficie nenhuma; a cintura mede"
    );
    // E um flanco inteiro degenerado nao e' parede nenhuma.
    let all = flank([Some((0.0, [0.0, 0.0])), Some((0.0, [0.0, 0.0])), None]);
    assert!(cling(&c, Some(&all), 1.0, -5.0, UP).is_none());
}

// ── O AGARRAR-SE (W23) ───────────────────────────────────────────────────────

/// A config com o agarrar-se armado.
fn grab_cfg(stamina: f32) -> WallConfig {
    WallConfig {
        slide_speed: 4.0,
        grab_stamina: stamina,
        ..WallConfig::STARTING_POINT
    }
}

/// **O GATE DA WAVE: agarrado, ele NÃO desce** — e o mesmo `wall_slide` é quem
/// responde.
///
/// ⚠️ O oráculo é o alvo de velocidade, não uma distância: agarrado o alvo é
/// **zero** e solto é `−slide_speed`, e é essa diferença que o motor entrega.
/// Um segundo termo somado ao escorregamento daria dois donos do mesmo número e
/// o sintoma seria um personagem que *quase* para.
#[test]
fn gripping_holds_him_where_he_is_and_letting_go_slides() {
    let c = grab_cfg(2.0);
    // Solto: e' o escorregamento de sempre — de −9 para −4 sao +5.
    let sliding = wall_slide(&c, true, false, -9.0, UP);
    assert!(
        (sliding.boost[1] - 5.0).abs() < 1.0e-5,
        "{:?}",
        sliding.boost
    );
    // Agarrado: o alvo e' ZERO — de −9 para 0 sao +9.
    let gripping = wall_slide(&c, true, true, -9.0, UP);
    assert!(
        (gripping.boost[1] - 9.0).abs() < 1.0e-5,
        "agarrado o alvo e' zero: {:?}",
        gripping.boost
    );
    assert_eq!(gripping.accel, [0.0, 0.0], "e' um boost, nunca uma forca");
}

/// **A reserva ACABA, e quando acaba ele volta a escorregar.**
#[test]
fn the_reserve_runs_out_and_then_he_slides_again() {
    let c = grab_cfg(0.5);
    let mut st = GrabState::default();
    let mut held_for = 0;
    for _ in 0..60 {
        let (next, gripping) = grab_step(&c, st, true, true, false, 1.0 / 60.0);
        st = next;
        if gripping {
            held_for += 1;
        }
    }
    // 0,5 s a 60 Hz sao 30 tiques — e o 31º ja' nao segura.
    assert_eq!(
        held_for, 30,
        "a reserva autorada e' quanto tempo ele segura"
    );
    assert!(st.spent >= c.grab_stamina);
    // E dali em diante o escorregamento volta.
    let (_, gripping) = grab_step(&c, st, true, true, false, 1.0 / 60.0);
    assert!(!gripping, "sem reserva nao ha' agarrar-se");
}

/// **O CHÃO enche a reserva, de uma vez.**
///
/// ⚠️ Qualquer outra regra (recarga por segundo, recarga ao soltar) ensinaria o
/// jogador a esperar parado, que é exactamente o que a reserva existe para não
/// ser.
#[test]
fn the_ground_refills_the_reserve_at_once() {
    let c = grab_cfg(1.0);
    let spent = GrabState { spent: 0.9 };
    let (st, gripping) = grab_step(&c, spent, false, false, true, 1.0 / 60.0);
    assert_eq!(st, GrabState::default(), "o chao devolve a reserva inteira");
    assert!(!gripping, "no chao nao se agarra a parede nenhuma");
}

/// **Sem o botão não há agarrar-se, e sem parede também não** — as duas metades
/// que o `grab_step` acrescenta ao que o `cling` já respondeu.
#[test]
fn the_grip_needs_both_the_button_and_the_wall() {
    let c = grab_cfg(2.0);
    let st = GrabState::default();
    assert!(
        !grab_step(&c, st, true, false, false, 0.016).1,
        "parede sim, botao nao"
    );
    assert!(
        !grab_step(&c, st, false, true, false, 0.016).1,
        "botao sim, parede nao"
    );
    assert!(grab_step(&c, st, true, true, false, 0.016).1);
}

/// **Desligado é desligado**, e o zero não é um caso especial: segurar por zero
/// segundos **é** não ter agarrar-se.
#[test]
fn a_zero_reserve_is_no_grab_at_all() {
    let c = grab_cfg(0.0);
    let st = GrabState::default();
    let (next, gripping) = grab_step(&c, st, true, true, false, 0.016);
    assert!(!gripping);
    assert_eq!(next, st, "e nada e' gasto");
    // ⚠️ E o escorregamento continua o AUTORADO. O primeiro corte deste gate
    // comparava contra o `STARTING_POINT`, que tem `slide_speed: 0.0` — um
    // controle que descreve outra config, e que reprovava produto correto.
    let m = wall_slide(&c, true, false, -9.0, UP);
    assert!(
        (m.boost[1] - 5.0).abs() < 1.0e-5,
        "de -9 para -4 sao +5, o escorregamento de sempre: {:?}",
        m.boost
    );
}

/// **A reserva conta a capacidade como ARMADA** — senão o sensor lateral nunca
/// seria castado, e um agarrar-se sem escorregamento nem pulo de parede ficaria
/// morto sem nada dizer por quê.
#[test]
fn a_grab_alone_arms_the_wall_sensor() {
    let only_grab = WallConfig {
        slide_speed: 0.0,
        jump_height: 0.0,
        grab_stamina: 2.0,
        ..WallConfig::STARTING_POINT
    };
    assert!(only_grab.armed(), "so' o agarrar-se ja' arma a parede");
    assert!(wall_probe_wanted(&only_grab, false, 1.0));
}
