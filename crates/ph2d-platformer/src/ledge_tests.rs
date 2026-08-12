//! Os gates da BEIRADA (`W-Ledge`) — a lei pura, sem mundo.

use super::*;

const UP: Vec2 = [0.0, 1.0];
const DT: f32 = 1.0 / 60.0;

fn cfg() -> LedgeConfig {
    LedgeConfig {
        grab: 0.4,
        speed: 3.0,
    }
}

/// Um lábio à DIREITA, `rise` acima da cabeça.
fn lip(rise: f32) -> LedgeProbe {
    LedgeProbe {
        lip_rise: rise,
        side: 1.0,
        across: 0.8,
        rise: 1.4,
    }
}

/// Um tique com o dedo a empurrar para a direita e mais nada.
fn push(c: &LedgeConfig, s: LedgeState, p: Option<&LedgeProbe>, v: Vec2, jump: bool) -> LedgeStep {
    ledge_step(c, s, p, 1.0, jump, false, false, v, UP, DT)
}

/// **O NEUTRO** — desligada, a beirada é o mundo de antes desta wave.
#[test]
fn a_disabled_ledge_is_the_world_before_this_wave() {
    let off = LedgeConfig { grab: 0.0, ..cfg() };
    let out = push(
        &off,
        LedgeState::default(),
        Some(&lip(0.2)),
        [0.0, -5.0],
        false,
    );
    assert!(!out.active, "desligada nao age");
    assert_eq!(out.motor, Motor::default(), "e nao escreve motor nenhum");
    assert_eq!(out.state, LedgeState::default());
    assert!(
        !ledge_probe_wanted(&off, false, 1.0, LedgeState::default()),
        "e nem sequer casta o raio"
    );
}

/// **Agarrar-se PARA a queda** — a razão de existir da metade *hang*.
#[test]
fn catching_a_ledge_stops_the_fall() {
    let c = cfg();
    let out = push(
        &c,
        LedgeState::default(),
        Some(&lip(0.2)),
        [0.0, -8.0],
        false,
    );
    assert!(out.active && out.state.hanging, "agarrou");
    // O `boost` tem de anular a queda inteira e ainda subir para o lábio.
    let after = -8.0 + out.motor.boost[1];
    assert!(
        after > 0.0,
        "depois do boost ele SOBE em direcao ao labio, e nao cai: {after}"
    );
    assert!(
        after <= c.speed + 1e-4,
        "e nunca mais depressa que a `speed`: {after}"
    );
}

/// **O SERVO mira o lábio, e o alvo é o ZERO** — é isso que o põe na pose.
#[test]
fn the_servo_drives_the_top_of_the_body_to_the_lip() {
    let c = cfg();
    // Já alinhado: nada a fazer no eixo vertical.
    let out = push(
        &c,
        LedgeState {
            hanging: true,
            climb: [0.0; 2],
        },
        Some(&lip(0.0)),
        [0.0, 0.0],
        false,
    );
    assert!(out.state.hanging);
    assert_eq!(out.motor.boost[1], 0.0, "alinhado, ele fica");
    // Abaixo do lábio: sobe. Acima dele: desce.
    let below = push(
        &c,
        LedgeState {
            hanging: true,
            climb: [0.0; 2],
        },
        Some(&lip(0.1)),
        [0.0, 0.0],
        false,
    );
    let above = push(
        &c,
        LedgeState {
            hanging: true,
            climb: [0.0; 2],
        },
        Some(&lip(-0.1)),
        [0.0, 0.0],
        false,
    );
    assert!(below.motor.boost[1] > 0.0, "labio acima => sobe");
    assert!(above.motor.boost[1] < 0.0, "labio abaixo => desce");
}

/// **As DUAS soleiras** — agarrar pede o lábio acima; continuar agarrado não.
///
/// ⚠️ **É o gate que impede o sucesso de desligar a lei:** o servo leva o topo
/// ao lábio (`lip_rise → 0`), e uma soleira única recusaria exactamente a pose
/// que ela procura.
#[test]
fn the_grab_needs_the_lip_overhead_but_the_hold_does_not() {
    let c = cfg();
    let fresh = LedgeState::default();
    let held = LedgeState {
        hanging: true,
        climb: [0.0; 2],
    };
    assert!(
        !push(&c, fresh, Some(&lip(0.0)), [0.0, -1.0], false).active,
        "com o labio a' altura da cabeca, ainda nao ha' o que alcancar"
    );
    assert!(
        push(&c, held, Some(&lip(0.0)), [0.0, 0.0], false)
            .state
            .hanging,
        "mas quem ja' esta' agarrado FICA — e' a pose que o servo procura"
    );
    assert!(
        push(&c, held, Some(&lip(-0.2)), [0.0, 0.0], false)
            .state
            .hanging,
        "e continua agarrado um pouco abaixo dela"
    );
}

/// **Soltar-se é não fazer nada** — parar de empurrar, ou apertar baixo.
#[test]
fn letting_go_is_the_absence_of_the_gesture() {
    let c = cfg();
    let held = LedgeState {
        hanging: true,
        climb: [0.0; 2],
    };
    let p = lip(0.1);
    // Sem `drive`.
    assert!(
        !ledge_step(
            &c,
            held,
            Some(&p),
            0.0,
            false,
            false,
            false,
            [0.0; 2],
            UP,
            DT
        )
        .active
    );
    // Empurrando para o OUTRO lado.
    assert!(
        !ledge_step(
            &c,
            held,
            Some(&p),
            -1.0,
            false,
            false,
            false,
            [0.0; 2],
            UP,
            DT
        )
        .active
    );
    // Apertando baixo.
    assert!(
        !ledge_step(
            &c,
            held,
            Some(&p),
            1.0,
            false,
            true,
            false,
            [0.0; 2],
            UP,
            DT
        )
        .active
    );
    // E no CHÃO: quem pousou nao esta' pendurado.
    assert!(
        !ledge_step(
            &c,
            held,
            Some(&p),
            1.0,
            false,
            false,
            true,
            [0.0; 2],
            UP,
            DT
        )
        .active
    );
}

/// **O mantle é um L, e a ORDEM é a correção** — sobe primeiro, atravessa
/// depois.
///
/// ⚠️ **A diagonal cortaria a quina do patamar**, e o solver responderia
/// empurrando o corpo de volta para fora.
#[test]
fn the_mantle_rises_before_it_crosses() {
    let c = cfg();
    let p = lip(0.05);
    let start = push(&c, LedgeState::default(), Some(&p), [0.0, 0.0], true);
    assert!(start.state.climbing(), "o pulo pendurado COMECA a subida");
    assert_eq!(start.state.climb, [p.across * p.side, p.rise]);
    assert!(!start.state.hanging, "e ela nao e' um pendurar");

    // Enquanto sobra subida, o movimento é para CIMA e mais nada.
    let mut s = start.state;
    let mut ticks = 0;
    while s.climb[1] > 0.0 {
        let out = ledge_step(&c, s, None, 0.0, false, false, false, [0.0; 2], UP, DT);
        assert!(out.active, "a subida nao precisa do sensor nem do dedo");
        assert!(
            out.motor.boost[1] > 0.0 && out.motor.boost[0] == 0.0,
            "so' para cima"
        );
        s = out.state;
        ticks += 1;
        assert!(ticks < 1000, "a subida tem de acabar");
    }
    // Depois dela, o movimento é para o LADO e mais nada.
    let across = ledge_step(&c, s, None, 0.0, false, false, false, [0.0; 2], UP, DT);
    assert!(
        across.motor.boost[0] > 0.0,
        "para a direita, o lado do labio"
    );
    assert_eq!(across.motor.boost[1], 0.0, "e ja' nao sobe");
}

/// **A subida ACABA, e acaba zerada** — nenhuma sobra de `f32` a arrastar.
#[test]
fn the_mantle_finishes_and_leaves_nothing_behind() {
    let c = cfg();
    let mut s = push(&c, LedgeState::default(), Some(&lip(0.05)), [0.0; 2], true).state;
    let mut ticks = 0;
    while s.busy() {
        s = ledge_step(&c, s, None, 0.0, false, false, false, [0.0; 2], UP, DT).state;
        ticks += 1;
        assert!(ticks < 1000, "tem de acabar");
    }
    assert_eq!(s, LedgeState::default(), "e o estado volta ao neutro");
    // ⚠️ O percurso total é o que o sensor mediu, e o tempo sai da `speed`.
    let want = (1.4 + 0.8) / c.speed / DT;
    assert!(
        (ticks as f32 - want).abs() < 3.0,
        "{ticks} tiques contra os ~{want:.0} que a `speed` promete"
    );
}

/// **A subida é um gesto COMPROMETIDO** — nada a interrompe.
///
/// ⚠️ Interrompê-la a meio deixaria o personagem no ar, à altura de um patamar
/// em que ele não está.
#[test]
fn nothing_interrupts_a_mantle_in_progress() {
    let c = cfg();
    let s = push(&c, LedgeState::default(), Some(&lip(0.05)), [0.0; 2], true).state;
    // Sem sensor, sem dedo, a empurrar para o lado errado, com baixo apertado,
    // e até com chão debaixo dos pés.
    let out = ledge_step(&c, s, None, -1.0, false, true, true, [0.0; 2], UP, DT);
    assert!(out.active && out.state.climbing(), "a subida continua");
}

/// **O motor SUBSTITUI a velocidade** — é o desenho do arranque.
#[test]
fn the_ledge_replaces_the_velocity_it_does_not_add_to_it() {
    let c = cfg();
    for v in [[3.0_f32, -9.0], [-2.0, 4.0], [0.0, 0.0]] {
        let out = push(&c, LedgeState::default(), Some(&lip(0.2)), v, false);
        let after = [v[0] + out.motor.boost[0], v[1] + out.motor.boost[1]];
        assert_eq!(after[0], 0.0, "o lateral morre, venha ele de onde vier");
        assert!(
            after[1] > 0.0 && after[1] <= c.speed + 1e-4,
            "e o vertical e' o servo, nao a velocidade que havia: {after:?}"
        );
        assert_eq!(out.motor.accel, [0.0, 0.0], "e nada disto e' forca");
    }
}

/// **A porta do sensor** — e a subida não pergunta nada ao mundo.
#[test]
fn the_probe_is_only_wanted_where_the_law_can_act() {
    let c = cfg();
    let fresh = LedgeState::default();
    assert!(
        ledge_probe_wanted(&c, false, 1.0, fresh),
        "no ar, a empurrar"
    );
    assert!(!ledge_probe_wanted(&c, true, 1.0, fresh), "no chao nao");
    assert!(
        !ledge_probe_wanted(&c, false, 0.0, fresh),
        "sem direcao nao"
    );
    let climbing = LedgeState {
        hanging: false,
        climb: [0.5, 0.5],
    };
    assert!(
        !ledge_probe_wanted(&c, false, 1.0, climbing),
        "a subida ja' sabe o que falta — o mundo nao lhe diz nada"
    );
}
