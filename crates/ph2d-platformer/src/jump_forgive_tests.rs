//! **O PERDÃO e a MEMÓRIA** — os gates das duas assistências que a lei do pulo
//! carrega, irmão de `jump_tests.rs` pelo teto de 700 LOC.
//!
//! ⚠️ **O corte é por RESPONSABILIDADE, não por tamanho:** o pai fica com *o que
//! um pulo É* (a altura autorada, as fases da gravidade, o corte, o pouso), e
//! este filho com *o que o jogo PERDOA* (coyote, buffer) e *o que ele CARREGA*
//! (a memória do chão que se deixou). São perguntas diferentes, e cada uma tem
//! a sua fixture.
//!
//! ⚠️ Módulo FILHO por `#[path]`, como o pai: é isso que mantém `use super::*` a
//! alcançar o que não é `pub`.
use super::*;

const UP: Vec2 = [0.0, 1.0];
const G: Vec2 = [0.0, -9.81];
/// O tique de 60 Hz, o mesmo do produto.
const DT: f32 = 1.0 / 60.0;

fn ground() -> GroundSample {
    GroundSample {
        distance: 0.9,
        normal: [0.0, 1.0],
        ground_velocity: [0.0, 0.0],
        one_way: false,
    }
}

// ═══ W8 — O PERDÃO ═══════════════════════════════════════════════════════

/// Um estado no chão, pronto para o próximo tique.
fn standing(cfg: &JumpConfig) -> JumpState {
    jump_step(
        cfg,
        JumpState::default(),
        Some(&ground()),
        0.0,
        false,
        false,
        None,
        G,
        UP,
        DT,
    )
    .state
}

/// **Sair da borda e pular DEPOIS ainda sai — e para de sair.**
///
/// As duas metades num gate só porque são a MESMA janela vista dos dois
/// lados: uma tolerância sem fim não é perdão, é voo.
#[test]
fn walking_off_a_ledge_still_jumps_inside_the_window_and_not_after() {
    // ⚠️ **A janela é de DOIS tiques de propósito, e a mutação exigiu isso:**
    // com os 0,1 s do perfil de partida (seis tiques) uma fixture tem folga
    // de sobra, e o deslocamento de UM tique — que é o que sai de correr os
    // relógios DEPOIS da decolagem em vez de antes — cabia dentro dela sem
    // ninguém ver. A dois tiques, um tique é metade da lei.
    let cfg = JumpConfig {
        coyote_time: 3.5 * DT,
        ..JumpConfig::STARTING_POINT
    };
    // ⚠️ **3,5 e não 3: a fronteira não pode cair num EMPATE.** Numa janela
    // de N tiques exatos o último passo compara `0.0 > 0.0`, e quem decide
    // é o resíduo de `f32` de somar `1/60` — um gate cujo veredito sai da
    // última casa decimal não está medindo a lei.
    for (ticks, want) in [(2, true), (3, false)] {
        let mut st = standing(&cfg);
        // Anda para fora: sem chão, sem apertar, caindo.
        for _ in 0..ticks {
            st = jump_step(&cfg, st, None, -0.5, false, false, None, G, UP, DT).state;
        }
        let s = jump_step(&cfg, st, None, -0.5, true, false, None, G, UP, DT);
        assert_eq!(
            s.takeoff,
            want,
            "coyote apos {ticks} tiques ({:.3} s) devia ser {want}",
            ticks as f32 * DT
        );
    }
}

/// ⚠️ **O coyote é CONSUMIDO pelo pulo que ele perdoa.** Sem isto ele seria
/// um pulo duplo com outro nome: sair da borda, pular, e ainda ter janela.
#[test]
fn the_coyote_window_is_spent_by_the_jump_it_forgives() {
    let cfg = JumpConfig::STARTING_POINT;
    let mut st = standing(&cfg);
    st = jump_step(&cfg, st, None, -0.5, false, false, None, G, UP, DT).state;
    let first = jump_step(&cfg, st, None, -0.5, true, false, None, G, UP, DT);
    assert!(first.takeoff, "o pulo de coyote tem de sair");
    assert_eq!(first.state.coyote, 0.0, "e a janela tem de ser GASTA");
    // Solta e aperta de novo, ainda no ar, dentro do que SERIA a janela.
    let released = jump_step(&cfg, first.state, None, 3.0, false, false, None, G, UP, DT);
    let again = jump_step(
        &cfg,
        released.state,
        None,
        3.0,
        true,
        false,
        None,
        G,
        UP,
        DT,
    );
    assert!(!again.takeoff, "nao pode haver um segundo pulo no ar");
}

/// **Apertar CEDO demais ainda pula — no tique em que o pé toca.**
///
/// ⚠️ O oráculo é *no MESMO tique*, não *em algum tique*: a ordem
/// pouso-antes-de-decolagem é a razão de o buffer existir, e um gate que só
/// pedisse "acabou pulando" ficaria verde com os 16 ms de atraso de volta.
#[test]
fn pressing_early_jumps_on_the_very_tick_the_foot_lands() {
    let cfg = JumpConfig::STARTING_POINT;
    // Em queda depois de um pulo: `airborne`, descendo, sem chão.
    let flying = JumpState {
        airborne: true,
        ..JumpState::default()
    };
    // O aperto acontece no ar, 3 tiques antes de tocar.
    let mut st = jump_step(&cfg, flying, None, -4.0, true, false, None, G, UP, DT).state;
    assert!(st.buffer > 0.0, "o aperto tem de ser GUARDADO");
    for _ in 0..2 {
        st = jump_step(&cfg, st, None, -4.0, true, false, None, G, UP, DT).state;
    }
    // O pé toca — segurando ainda, que é o gesto real (ninguém solta a
    // tecla no ar de propósito).
    let land = jump_step(
        &cfg,
        st,
        Some(&ground()),
        -4.0,
        true,
        false,
        None,
        G,
        UP,
        DT,
    );
    assert!(
        land.takeoff,
        "o aperto guardado tem de disparar NO tique do pouso"
    );
    assert_eq!(land.state.buffer, 0.0, "e ser gasto");

    // ⚠️ **A metade que FALTAVA, e a mutação provou:** sem ela, um buffer
    // que nunca escorre passa no gate — o aperto chega ao pouso porque a
    // fixture pousava dentro da janela de qualquer jeito. Um perdão sem fim
    // é um pulo agendado para sempre.
    let mut far = jump_step(&cfg, flying, None, -4.0, true, false, None, G, UP, DT).state;
    let past = (cfg.jump_buffer / DT).ceil() as i32 + 2;
    for _ in 0..past {
        far = jump_step(&cfg, far, None, -4.0, true, false, None, G, UP, DT).state;
    }
    let late = jump_step(
        &cfg,
        far,
        Some(&ground()),
        -4.0,
        true,
        false,
        None,
        G,
        UP,
        DT,
    );
    assert!(
        !late.takeoff,
        "um aperto de {:.3} s atras nao pode sobreviver a janela de {:.3} s",
        past as f32 * DT,
        cfg.jump_buffer
    );
}

/// **Um aperto é UM pulo.** Sem o consumo, o buffer sobrevive à própria
/// decolagem e re-dispara no tique seguinte, com o pé ainda em contato.
#[test]
fn one_press_is_one_jump() {
    let cfg = JumpConfig::STARTING_POINT;
    let st = standing(&cfg);
    let first = jump_step(&cfg, st, Some(&ground()), 0.0, true, false, None, G, UP, DT);
    assert!(first.takeoff);
    // O tique seguinte: ainda segurando, o raio ainda vê o chão (a decolagem
    // não teleporta ninguém).
    let second = jump_step(
        &cfg,
        first.state,
        Some(&ground()),
        3.0,
        true,
        false,
        None,
        G,
        UP,
        DT,
    );
    assert!(!second.takeoff, "um aperto nao pode dar dois pulos");
}

/// ⚠️ **Com as duas janelas em ZERO a lei é a de antes desta wave, AO BIT.**
///
/// É o que torna a W8 uma adição honesta: quem não quer assistência a
/// desliga e recebe exatamente o pulo que o W4 shipou.
#[test]
fn zero_windows_are_the_law_of_before_this_wave() {
    let cfg = JumpConfig {
        coyote_time: 0.0,
        jump_buffer: 0.0,
        ..JumpConfig::STARTING_POINT
    };
    // Fora da borda: nao pula.
    let mut st = standing(&cfg);
    st = jump_step(&cfg, st, None, -0.5, false, false, None, G, UP, DT).state;
    assert!(
        !jump_step(&cfg, st, None, -0.5, true, false, None, G, UP, DT).takeoff,
        "sem coyote, sair da borda tira o pulo no tique seguinte"
    );
    // Apertar no ar nao sobrevive ate' o pouso.
    let flying = JumpState {
        airborne: true,
        ..JumpState::default()
    };
    let pressed = jump_step(&cfg, flying, None, -4.0, true, false, None, G, UP, DT).state;
    let land = jump_step(
        &cfg,
        pressed,
        Some(&ground()),
        -4.0,
        true,
        false,
        None,
        G,
        UP,
        DT,
    );
    assert!(
        !land.takeoff,
        "sem buffer, um aperto no ar morre com o tique em que foi feito"
    );
}

// ── W10: A MEMÓRIA DO CHÃO QUE SE DEIXOU ─────────────────────────────────────

/// Um vagão a 4 m/s sob os pés.
fn wagon() -> GroundSample {
    GroundSample {
        ground_velocity: [4.0, 0.0],
        ..ground()
    }
}

/// **A memória enche no chão e escorre no ar** — o irmão exato do coyote, e por
/// isso o gate mede as duas metades.
///
/// **Mutação que deve sangrar:** não drenar `lift_time` no ramo do ar.
#[test]
fn the_lift_memory_fills_on_the_ground_and_drains_in_the_air() {
    let cfg = JumpConfig::STARTING_POINT;
    let on = jump_step(
        &cfg,
        JumpState::default(),
        Some(&wagon()),
        0.0,
        false,
        false,
        None,
        G,
        UP,
        DT,
    );
    assert_eq!(on.state.lift, [4.0, 0.0], "no chao ela guarda o VAGAO");
    assert!(
        (on.state.lift_time - cfg.lift_momentum).abs() < 1.0e-6,
        "e a janela nasce cheia: {}",
        on.state.lift_time
    );

    // No ar ela escorre, e o VALOR lembrado não se apaga com ela.
    let air = jump_step(&cfg, on.state, None, -1.0, false, false, None, G, UP, DT);
    assert_eq!(air.state.lift, [4.0, 0.0], "o que se lembra nao muda no ar");
    assert!(
        (air.state.lift_time - (cfg.lift_momentum - DT)).abs() < 1.0e-6,
        "a janela escorre por dt: {}",
        air.state.lift_time
    );
}

/// **O referencial SEGURA o valor cheio e depois SOLTA** — a lei que a medição
/// escolheu contra o desvanecimento (ver [`JumpConfig::lift_momentum`]).
///
/// **Mutação que deve sangrar:** devolver `lift · (lift_time / lift_momentum)`,
/// que é a primeira versão desta lei.
#[test]
fn the_carried_frame_holds_full_and_then_releases() {
    let cfg = JumpConfig::STARTING_POINT;
    let mut st = jump_step(
        &cfg,
        JumpState::default(),
        Some(&wagon()),
        0.0,
        false,
        false,
        None,
        G,
        UP,
        DT,
    )
    .state;

    // Meio da janela: ainda CHEIO — é isso que "preserva" quer dizer.
    for _ in 0..40 {
        st = jump_step(&cfg, st, None, -1.0, false, false, None, G, UP, DT).state;
        assert_eq!(
            carried_frame(&cfg, &st),
            [4.0, 0.0],
            "dentro da janela o referencial e' o do vagao, INTEIRO (restam {})",
            st.lift_time
        );
    }

    // Passada a janela: o referencial volta a ser o do mundo.
    for _ in 0..100 {
        st = jump_step(&cfg, st, None, -1.0, false, false, None, G, UP, DT).state;
    }
    assert_eq!(st.lift_time, 0.0, "a janela fechou");
    assert_eq!(
        carried_frame(&cfg, &st),
        [0.0, 0.0],
        "e o referencial volta a ser o do mundo"
    );
}

/// **`lift_momentum = 0` é inerte, e o chão PARADO também** — as duas metades
/// que tornam o default ligado honesto.
#[test]
fn a_still_ground_and_a_zero_window_both_carry_nothing() {
    let mut off = JumpConfig::STARTING_POINT;
    off.lift_momentum = 0.0;
    let s = jump_step(
        &off,
        JumpState::default(),
        Some(&wagon()),
        0.0,
        false,
        false,
        None,
        G,
        UP,
        DT,
    )
    .state;
    assert_eq!(
        carried_frame(&off, &s),
        [0.0, 0.0],
        "janela zero nao carrega nada, nem de um vagao"
    );

    let cfg = JumpConfig::STARTING_POINT;
    let still = jump_step(
        &cfg,
        JumpState::default(),
        Some(&ground()),
        0.0,
        false,
        false,
        None,
        G,
        UP,
        DT,
    )
    .state;
    assert_eq!(
        carried_frame(&cfg, &still),
        [0.0, 0.0],
        "e um chao parado lembra [0, 0] — o default ligado e' inerte ali"
    );
}
