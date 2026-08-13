//! Os gates da lei do ARRANQUE (W14).
//!
//! # ⚠️ O `dt` é `1/64`, e a escolha é sobre o ORÁCULO
//!
//! Um sexagésimo não é representável em binário, então `time − k·dt` acumula
//! resíduo e a contagem de tiques de um arranque passa a depender do último bit
//! de um `f32`. Com `dt = 1/64` e `time = 10/64` a aritmética é **exata**, e o
//! gate mede a LEI em vez de medir o resíduo — que é a diferença entre uma
//! asserção e uma flake.

use super::*;

/// O tique desta suíte — ver o aviso do módulo.
const DT: f32 = 1.0 / 64.0;
/// Dez tiques exatos.
const TIME: f32 = 10.0 / 64.0;

const UP: Vec2 = [0.0, 1.0];
const G: Vec2 = [0.0, -9.81];

fn armed() -> DashConfig {
    DashConfig {
        speed: 18.0,
        time: TIME,
        cooldown: 0.2,
    }
}

/// Um tique com o botão do arranque SEGURADO.
///
/// ⚠️ **O `facing` é do CHAMADOR** desde o `W-PlayerOut` — a direção deixou de
/// ser campo do arranque; ele apenas a lê. Os gates que a exercitam passam a
/// compô-la com a [`facing_step`], que é a lei que a produz.
fn hold(cfg: &DashConfig, s: DashState, grounded: bool, facing: f32) -> DashStep {
    dash_step(cfg, s, grounded, facing, true, false, DT)
}

/// Um tique com o botão SOLTO.
fn idle(cfg: &DashConfig, s: DashState, grounded: bool, facing: f32) -> DashStep {
    dash_step(cfg, s, grounded, facing, false, false, DT)
}

/// **O arranque dura o que foi autorado, e nem um tique a mais.**
///
/// ⚠️ **Mutação medida:** tirar o `− dt` do tique que COMEÇA (`next.left =
/// cfg.time`) dá **11** tiques em vez de 10 — o arranque passa a durar
/// `time + dt`, ou seja a duração dele depende da taxa da sim. É o defeito que
/// os relógios em segundos existem para não ter, e ele é invisível a olho.
#[test]
fn a_dash_lasts_the_authored_time_and_not_one_tick_more() {
    let cfg = armed();
    let mut s = DashState::default();
    let mut ticks = 0;
    let first = hold(&cfg, s, true, 1.0);
    assert!(first.active, "o aperto tem de arrancar");
    s = first.state;
    ticks += 1;
    loop {
        let step = hold(&cfg, s, false, 1.0);
        if !step.active {
            break;
        }
        s = step.state;
        ticks += 1;
        assert!(ticks < 100, "o arranque nunca acabou");
    }
    assert_eq!(ticks, 10, "10 tiques de 1/64 s sao os {TIME} s autorados");
}

/// **A velocidade é DEFINIDA, não somada** — a lição que a W13 pagou medindo.
///
/// Um corpo a cair a 8 m/s e a andar a 3 m/s tem de sair do tique exatamente a
/// `(speed, 0)`, e não a `(3 + speed, −8)`.
#[test]
fn the_burst_defines_the_velocity_instead_of_adding_to_it() {
    let cfg = armed();
    let m = dash_burst(&cfg, 1.0, [0.0, 0.0], [3.0, -8.0], UP, G);
    assert!(
        (m.boost[0] - (cfg.speed - 3.0)).abs() < 1e-4,
        "a horizontal vai AO alvo: {}",
        m.boost[0]
    );
    assert!(
        (m.boost[1] - 8.0).abs() < 1e-4,
        "a vertical relativa vai a ZERO: {}",
        m.boost[1]
    );
}

/// **A gravidade é cancelada** — e é isso que faz o traço ser reto.
///
/// ⚠️ **Mutação medida:** `accel: [0, 0]` deixa o arranque SAGAR — o boost põe a
/// velocidade vertical em zero no topo do tique e a gravidade a puxa de volta
/// dentro dele, então o desenho vira uma escada em vez de uma linha.
#[test]
fn a_dash_cancels_gravity() {
    let cfg = armed();
    let m = dash_burst(&cfg, -1.0, [0.0, 0.0], [0.0, 0.0], UP, G);
    assert!((m.accel[0] - -G[0]).abs() < 1e-4);
    assert!((m.accel[1] - -G[1]).abs() < 1e-4);
    assert!(m.boost[0] < 0.0, "o sinal da direcao chega ao boost");
}

/// **O arranque anda no referencial do CHÃO** — arrancar de cima de um vagão
/// leva a velocidade dele junto, em vez de a apagar.
#[test]
fn the_burst_is_measured_in_the_ground_frame() {
    let cfg = armed();
    let m = dash_burst(&cfg, 1.0, [5.0, 0.0], [5.0, 0.0], UP, G);
    assert!(
        (m.boost[0] - cfg.speed).abs() < 1e-4,
        "quem ja' viaja com o chao arranca a partir dele: {}",
        m.boost[0]
    );
}

/// **UM arranque por tempo-de-voo, e o CHÃO é que recarrega.**
///
/// ⚠️ A recuperação está em zero nesta fixture **de propósito**: com ela ligada,
/// a segunda recusa teria duas causas possíveis e o gate não distinguiria a que
/// ele existe para medir.
///
/// ⚠️ **Mutação medida:** repor a carga por relógio em vez de pelo pé no chão
/// (`next.charged = true` incondicional) deixa **voar** — a segunda asserção
/// passa a achar um arranque no ar.
#[test]
fn one_dash_per_airtime_and_the_ground_is_what_refills_it() {
    let cfg = DashConfig {
        cooldown: 0.0,
        ..armed()
    };
    let mut s = hold(&cfg, DashState::default(), true, 1.0).state;
    assert!(!s.charged, "a carga foi gasta");
    // Deixa o arranque acabar, no AR.
    for _ in 0..20 {
        s = idle(&cfg, s, false, 1.0).state;
    }
    // Um segundo aperto, ainda no ar: recusado.
    let second = hold(&cfg, s, false, 1.0);
    assert!(
        !second.active,
        "o segundo arranque no ar tem de ser recusado"
    );
    s = second.state;
    // Toca o chão: a carga volta.
    s = idle(&cfg, s, true, 1.0).state;
    assert!(s.charged, "o pe' no chao repoe a carga");
    assert!(
        hold(&cfg, s, true, 1.0).active,
        "e depois de tocar o chao ele arranca outra vez"
    );
}

/// **A recuperação espaça dois arranques no chão** — e ela conta do FIM.
#[test]
fn the_cooldown_spaces_two_ground_dashes() {
    let cfg = armed();
    let mut s = hold(&cfg, DashState::default(), true, 1.0).state;
    // O arranque inteiro, com o pé no chão (a carga volta, então só a
    // recuperação pode recusar).
    for _ in 0..10 {
        s = idle(&cfg, s, true, 1.0).state;
    }
    assert!(s.charged, "no chao a carga ja' voltou");
    assert!(s.cool > 0.0, "e a recuperacao comecou ao ACABAR");
    assert!(
        !hold(&cfg, s, true, 1.0).active,
        "cedo demais: a recuperacao recusa"
    );
    // Espera-a escorrer.
    let mut waited = 0;
    while s.cool > 0.0 {
        s = idle(&cfg, s, true, 1.0).state;
        waited += 1;
        assert!(waited < 200, "a recuperacao nunca acabou");
    }
    assert!(
        hold(&cfg, s, true, 1.0).active,
        "passada a recuperacao, ele arranca"
    );
}

/// **Com o eixo neutro ele arranca para onde OLHA** — a alternativa é um botão
/// que parece quebrado.
#[test]
fn a_neutral_press_dashes_where_he_faces() {
    let cfg = armed();
    // Anda para a esquerda, para, arranca — com o `facing` a vir da lei que o
    // produz, exactamente como a porta única faz.
    let mut facing = crate::facing_step(1.0, -1.0);
    assert!((facing - -1.0).abs() < 1e-6);
    let s = idle(&cfg, DashState::default(), true, facing).state;
    facing = crate::facing_step(facing, 0.0);
    assert!(
        (facing - -1.0).abs() < 1e-6,
        "parar de andar nao e' virar-se para lugar nenhum"
    );
    let step = hold(&cfg, s, true, facing);
    assert!(step.active);
    assert!(
        (step.state.dir - -1.0).abs() < 1e-6,
        "o arranque saiu para o lado errado"
    );
}

/// **A direção é congelada no instante do arranque** — virar no meio não o curva.
#[test]
fn the_direction_is_frozen_for_the_whole_dash() {
    let cfg = armed();
    let mut s = hold(&cfg, DashState::default(), true, 1.0).state;
    assert!((s.dir - 1.0).abs() < 1e-6);
    let mut facing = 1.0;
    for _ in 0..5 {
        // O jogador vira o eixo para o outro lado no meio do gesto.
        facing = crate::facing_step(facing, -1.0);
        let step = hold(&cfg, s, false, facing);
        assert!(step.active);
        s = step.state;
        assert!(
            (s.dir - 1.0).abs() < 1e-6,
            "a direcao do arranque em curso mudou"
        );
    }
    assert!(
        (facing - -1.0).abs() < 1e-6,
        "mas o lado para onde ele OLHA acompanha o eixo"
    );
}

/// **Um cancelamento acaba com ele AGORA**, e não no tique seguinte.
#[test]
fn a_cancel_ends_the_dash_at_once_and_starts_the_recovery() {
    let cfg = armed();
    let s = hold(&cfg, DashState::default(), true, 1.0).state;
    let cut = dash_step(&cfg, s, false, 1.0, true, true, DT);
    assert!(!cut.active, "o tique do cancelamento nao arranca");
    assert!(cut.state.left <= 0.0);
    assert!(cut.state.cool > 0.0, "e a recuperacao comeca ali");
}

/// **Velocidade zero é o mundo de antes desta wave** — e a carga nem é gasta.
///
/// ⚠️ É a metade que torna a capacidade opt-in verificável: um player já
/// autorado abre com `speed = 0` e nada nesta lei o alcança.
#[test]
fn speed_zero_never_dashes_and_never_spends_the_charge() {
    let cfg = DashConfig {
        speed: 0.0,
        ..armed()
    };
    let step = hold(&cfg, DashState::default(), true, 1.0);
    assert!(!step.active);
    assert!(step.state.charged, "um botao desligado nao gasta nada");
    assert!(step.state.left <= 0.0);
}

/// **Segurar o botão não encadeia arranques** — a borda é derivada aqui.
#[test]
fn holding_the_button_does_not_chain_dashes() {
    let cfg = DashConfig {
        cooldown: 0.0,
        ..armed()
    };
    let mut s = hold(&cfg, DashState::default(), true, 1.0).state;
    // Segura o botão sem soltar, com o pé no chão (carga cheia, sem recuperação).
    for _ in 0..40 {
        s = hold(&cfg, s, true, 1.0).state;
    }
    assert!(
        s.left <= 0.0,
        "segurar nao pode manter um arranque para sempre"
    );
    // Soltar e apertar de novo dispara.
    s = idle(&cfg, s, true, 1.0).state;
    assert!(hold(&cfg, s, true, 1.0).active);
}
