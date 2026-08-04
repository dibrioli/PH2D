//! Os gates da lei do PULO — ver `jump.rs`.
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
    }
}

/// **A altura vira velocidade UMA vez, pela fórmula.**
///
/// `v₀ = √(2·g·h)`: com `h = 2` e `g = 9,81` são 6,264 m/s.
#[test]
fn the_takeoff_speed_comes_from_the_authored_height() {
    let cfg = JumpConfig::STARTING_POINT;
    let s = jump_step(
        &cfg,
        JumpState::default(),
        Some(&ground()),
        0.0,
        true,
        G,
        UP,
        DT,
    );
    let expected = (2.0 * 9.81 * 2.0_f32).sqrt();
    assert!(
        (s.motor.boost[1] - expected).abs() < 1.0e-3,
        "o boost tem de ser v0 = sqrt(2gh) = {expected:.4}: {:?}",
        s.motor.boost
    );
    assert!(s.state.airborne, "decolou");
    assert!(!s.spring_armed, "e a perna CALA");
}

/// ⚠️ **Pular já subindo dá a MESMA altura** — o boost leva AO valor, não
/// soma a ele.
///
/// É a frase inteira do *"parametrizado por altura"*: sem isto, pular de uma
/// plataforma que sobe daria um pulo mais alto do que o autorado, e o número
/// na row deixaria de descrever o que acontece.
#[test]
fn jumping_while_already_rising_reaches_the_same_height() {
    let cfg = JumpConfig::STARTING_POINT;
    let v0 = (2.0 * 9.81 * 2.0_f32).sqrt();
    for start in [0.0_f32, 2.0, -1.0] {
        let s = jump_step(
            &cfg,
            JumpState::default(),
            Some(&ground()),
            start,
            true,
            G,
            UP,
            DT,
        );
        let after = start + s.motor.boost[1];
        assert!(
            (after - v0).abs() < 1.0e-3,
            "partindo de {start}: a velocidade final tem de ser {v0:.4}, deu {after:.4}"
        );
    }
}

/// ⚠️ **Segurar a tecla NÃO re-pula** — a decolagem é na BORDA.
///
/// Sem isto, um dedo apoiado no botão daria um impulso por tick e o
/// personagem subiria para sempre.
#[test]
fn holding_the_button_does_not_re_jump() {
    let cfg = JumpConfig::STARTING_POINT;
    let first = jump_step(
        &cfg,
        JumpState::default(),
        Some(&ground()),
        0.0,
        true,
        G,
        UP,
        DT,
    );
    assert!(first.motor.boost[1] > 0.0);
    // O tick seguinte, ainda com a tecla presa e ainda vendo o chão.
    let second = jump_step(&cfg, first.state, Some(&ground()), 3.0, true, G, UP, DT);
    assert_eq!(second.motor.boost, [0.0, 0.0], "nada de segundo impulso");
}

/// **Nem pular no AR** — o pulo duplo é uma feature, não um acidente.
#[test]
fn a_second_press_in_mid_air_does_nothing() {
    let cfg = JumpConfig::STARTING_POINT;
    let flying = JumpState {
        airborne: true,
        cut: false,
        was_held: false,
        ..JumpState::default()
    };
    let s = jump_step(&cfg, flying, None, 3.0, true, G, UP, DT);
    assert_eq!(s.motor.boost, [0.0, 0.0]);
}

/// **Soltar durante a subida ARMA o corte, e ele não desarma.**
///
/// Segurar de novo no ar não devolve a altura — senão ela seria função de
/// quantas vezes o jogador tamborilou o dedo.
#[test]
fn releasing_arms_the_cut_and_re_holding_does_not_undo_it() {
    let cfg = JumpConfig::STARTING_POINT;
    let flying = JumpState {
        airborne: true,
        cut: false,
        was_held: true,
        ..JumpState::default()
    };
    let released = jump_step(&cfg, flying, None, 4.0, false, G, UP, DT);
    assert!(released.state.cut, "soltar arma o corte");
    assert!(
        (released.motor.accel[1] - G[1] * (cfg.cut_gravity - 1.0)).abs() < 1.0e-4,
        "e o corte pesa: {:?}",
        released.motor.accel
    );

    let re_held = jump_step(&cfg, released.state, None, 3.0, true, G, UP, DT);
    assert!(re_held.state.cut, "segurar de novo NAO desfaz o corte");
}

/// **As quatro fases dão quatro gravidades**, e a do ápice é a única ABAIXO
/// de 1 — a decisão do módulo.
#[test]
fn each_phase_gets_its_own_gravity() {
    let cfg = JumpConfig::STARTING_POINT;
    let flying = JumpState {
        airborne: true,
        cut: false,
        was_held: true,
        ..JumpState::default()
    };
    let scale = |rel: f32, st: JumpState, held: bool| {
        let s = jump_step(&cfg, st, None, rel, held, G, UP, DT);
        s.motor.accel[1] / G[1] + 1.0
    };
    let rising = scale(5.0, flying, true);
    let peak = scale(0.0, flying, true);
    let falling = scale(-5.0, flying, true);
    let cut = scale(
        5.0,
        JumpState {
            cut: true,
            ..flying
        },
        true,
    );
    assert!(
        (rising - 1.0).abs() < 1.0e-4,
        "subindo (takeoff inerte): {rising}"
    );
    assert!((peak - cfg.peak_gravity).abs() < 1.0e-4, "apice: {peak}");
    assert!(
        (falling - cfg.fall_gravity).abs() < 1.0e-4,
        "queda: {falling}"
    );
    assert!((cut - cfg.cut_gravity).abs() < 1.0e-4, "corte: {cut}");
    assert!(
        peak < 1.0,
        "⚠️ o apice ALONGA (Celeste), nao encurta (tnua): {peak}"
    );
}

/// ⚠️ **Multiplicadores em `1.0` são gravidade extra ZERO** — o mundo sem
/// pulo, byte a byte. É o que torna cada knob uma escolha e não um imposto.
#[test]
fn neutral_multipliers_add_exactly_nothing() {
    let cfg = JumpConfig {
        takeoff_gravity: 1.0,
        peak_gravity: 1.0,
        fall_gravity: 1.0,
        cut_gravity: 1.0,
        ..JumpConfig::STARTING_POINT
    };
    let flying = JumpState {
        airborne: true,
        cut: true,
        was_held: false,
        ..JumpState::default()
    };
    for rel in [-9.0_f32, -1.0, 0.0, 1.0, 9.0] {
        let s = jump_step(&cfg, flying, None, rel, false, G, UP, DT);
        assert_eq!(s.motor.accel, [0.0, 0.0], "a {rel} m/s");
    }
}

/// **O pouso pede as DUAS metades** — descendo E com chão ao alcance.
///
/// Só "achou chão" pousaria no tick da decolagem (o raio ainda vê o chão) e
/// a mola puxaria de volta o pulo inteiro; só "descendo" nunca pousaria.
#[test]
fn landing_needs_both_halves() {
    let cfg = JumpConfig::STARTING_POINT;
    let flying = JumpState {
        airborne: true,
        cut: false,
        was_held: false,
        ..JumpState::default()
    };
    // Subindo COM chão ao alcance (o tick logo após a decolagem): não pousa.
    let rising = jump_step(&cfg, flying, Some(&ground()), 5.0, false, G, UP, DT);
    assert!(rising.state.airborne, "ainda subindo, nao pousou");
    assert!(!rising.spring_armed, "e a perna segue CALADA");

    // Descendo SEM chão: também não.
    let falling = jump_step(&cfg, flying, None, -5.0, false, G, UP, DT);
    assert!(falling.state.airborne);

    // Descendo COM chão: pousou, e a perna volta.
    let landed = jump_step(&cfg, flying, Some(&ground()), -5.0, false, G, UP, DT);
    assert!(!landed.state.airborne, "pousou");
    assert!(landed.spring_armed, "e a perna volta a agir");
}

/// ⚠️ **O pouso limpa o CORTE também, e ele é alcançável.**
///
/// O corte é lido num sítio só — subindo acima do `peak_speed` — e no chão a
/// perna arma primeiro, então um corte esquecido parece inofensivo. Não é: se
/// o personagem **sai do chão SUBINDO sem pular** (andar para fora da borda de
/// uma plataforma kinematic que ASCENDE, que este produto já tem desde o W4),
/// não há decolagem para zerá-lo — e a subida ganha `cut_gravity` (4×) onde
/// devia ganhar `takeoff_gravity` (1×), puxando o personagem para baixo por um
/// corte de um pulo que acabou.
///
/// ⚠️ Este gate nasceu de uma mutação que **sobreviveu à suíte inteira** (a de
/// comportamento inclusive): tirar o `next.cut = false` do pouso não move um
/// milímetro em nenhuma cena que só pula e cai.
#[test]
fn landing_clears_the_cut_so_the_next_rise_is_not_punished() {
    let cfg = JumpConfig::STARTING_POINT;
    let cut_in_flight = JumpState {
        airborne: true,
        cut: true,
        was_held: false,
        ..JumpState::default()
    };

    // Pousa com o corte armado.
    let landed = jump_step(&cfg, cut_in_flight, Some(&ground()), -5.0, false, G, UP, DT);
    assert!(!landed.state.airborne, "pousou");
    assert!(!landed.state.cut, "e o corte MORREU com o pouso");

    // Agora sai do chão SUBINDO sem pular (a plataforma o levou).
    let rising = jump_step(&cfg, landed.state, None, 5.0, false, G, UP, DT);
    let scale = rising.motor.accel[1] / G[1] + 1.0;
    assert!(
        (scale - cfg.takeoff_gravity).abs() < 1.0e-4,
        "subir sem pular usa a gravidade de SAIDA, nao a do corte: {scale}"
    );
}

/// Sem pulo nenhum, andar para fora de uma borda ainda ganha a gravidade de
/// QUEDA — ela é sobre cair, não sobre pular.
#[test]
fn walking_off_a_ledge_still_falls_faster() {
    let cfg = JumpConfig::STARTING_POINT;
    let s = jump_step(&cfg, JumpState::default(), None, -5.0, false, G, UP, DT);
    assert!(!s.state.airborne, "ele nao pulou");
    assert!(
        (s.motor.accel[1] - G[1] * (cfg.fall_gravity - 1.0)).abs() < 1.0e-4,
        "mas cai com a gravidade de queda: {:?}",
        s.motor.accel
    );
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
            st = jump_step(&cfg, st, None, -0.5, false, G, UP, DT).state;
        }
        let s = jump_step(&cfg, st, None, -0.5, true, G, UP, DT);
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
    st = jump_step(&cfg, st, None, -0.5, false, G, UP, DT).state;
    let first = jump_step(&cfg, st, None, -0.5, true, G, UP, DT);
    assert!(first.takeoff, "o pulo de coyote tem de sair");
    assert_eq!(first.state.coyote, 0.0, "e a janela tem de ser GASTA");
    // Solta e aperta de novo, ainda no ar, dentro do que SERIA a janela.
    let released = jump_step(&cfg, first.state, None, 3.0, false, G, UP, DT);
    let again = jump_step(&cfg, released.state, None, 3.0, true, G, UP, DT);
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
    let mut st = jump_step(&cfg, flying, None, -4.0, true, G, UP, DT).state;
    assert!(st.buffer > 0.0, "o aperto tem de ser GUARDADO");
    for _ in 0..2 {
        st = jump_step(&cfg, st, None, -4.0, true, G, UP, DT).state;
    }
    // O pé toca — segurando ainda, que é o gesto real (ninguém solta a
    // tecla no ar de propósito).
    let land = jump_step(&cfg, st, Some(&ground()), -4.0, true, G, UP, DT);
    assert!(
        land.takeoff,
        "o aperto guardado tem de disparar NO tique do pouso"
    );
    assert_eq!(land.state.buffer, 0.0, "e ser gasto");

    // ⚠️ **A metade que FALTAVA, e a mutação provou:** sem ela, um buffer
    // que nunca escorre passa no gate — o aperto chega ao pouso porque a
    // fixture pousava dentro da janela de qualquer jeito. Um perdão sem fim
    // é um pulo agendado para sempre.
    let mut far = jump_step(&cfg, flying, None, -4.0, true, G, UP, DT).state;
    let past = (cfg.jump_buffer / DT).ceil() as i32 + 2;
    for _ in 0..past {
        far = jump_step(&cfg, far, None, -4.0, true, G, UP, DT).state;
    }
    let late = jump_step(&cfg, far, Some(&ground()), -4.0, true, G, UP, DT);
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
    let first = jump_step(&cfg, st, Some(&ground()), 0.0, true, G, UP, DT);
    assert!(first.takeoff);
    // O tique seguinte: ainda segurando, o raio ainda vê o chão (a decolagem
    // não teleporta ninguém).
    let second = jump_step(&cfg, first.state, Some(&ground()), 3.0, true, G, UP, DT);
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
    st = jump_step(&cfg, st, None, -0.5, false, G, UP, DT).state;
    assert!(
        !jump_step(&cfg, st, None, -0.5, true, G, UP, DT).takeoff,
        "sem coyote, sair da borda tira o pulo no tique seguinte"
    );
    // Apertar no ar nao sobrevive ate' o pouso.
    let flying = JumpState {
        airborne: true,
        ..JumpState::default()
    };
    let pressed = jump_step(&cfg, flying, None, -4.0, true, G, UP, DT).state;
    let land = jump_step(&cfg, pressed, Some(&ground()), -4.0, true, G, UP, DT);
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
    let air = jump_step(&cfg, on.state, None, -1.0, false, G, UP, DT);
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
        G,
        UP,
        DT,
    )
    .state;

    // Meio da janela: ainda CHEIO — é isso que "preserva" quer dizer.
    for _ in 0..40 {
        st = jump_step(&cfg, st, None, -1.0, false, G, UP, DT).state;
        assert_eq!(
            carried_frame(&cfg, &st),
            [4.0, 0.0],
            "dentro da janela o referencial e' o do vagao, INTEIRO (restam {})",
            st.lift_time
        );
    }

    // Passada a janela: o referencial volta a ser o do mundo.
    for _ in 0..100 {
        st = jump_step(&cfg, st, None, -1.0, false, G, UP, DT).state;
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
