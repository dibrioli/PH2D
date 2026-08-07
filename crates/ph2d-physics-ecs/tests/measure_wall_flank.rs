//! **O FLANCO** (W13, item aberto) — quanto custa o sensor lateral ler só a
//! meia-altura do corpo.
//!
//! `cargo test -p ph2d-physics-ecs --test measure_wall_flank -- --ignored --nocapture`
//!
//! O aviso de `bridge::player::probe_wall` diz, desde a wave: *"a altura é o MEIO
//! do corpo … uma beirada que só alcance os pés (ou só os ombros) não é vista"*.
//! Isto põe um NÚMERO nessa frase antes de qualquer linha ser escrita.
//!
//! # ⚠️ A cena é uma parede com FRESTA, e as duas primeiras tentativas falharam
//!
//! Uma parede que *começa* num topo não serve: o personagem anda por cima dela e
//! vai embora (medido — atravessa `x = 1,2 … 9,7` em queda livre sem nunca
//! encostar). Uma beirada solta também não: ele a atravessa em três tiques e sai
//! dela de qualquer jeito. A **fresta** segura — ele desce a parede inteira, e o
//! defeito aparece no instante exato em que o buraco passa pela cintura dele.

#[path = "platform_wall_rig.rs"]
mod rig_fixture;

use rig_fixture::{GAP_CENTER, Rig, into_wall, pose, rig_gapped};

/// A trajetória inteira: `(altura, descida naquele tique)` por tique.
fn fall(r: &mut Rig, ticks: u64) -> Vec<(f32, f32)> {
    r.bridge.set_player_input(r.player, into_wall());
    let mut prev = pose(&r.sim).1;
    let mut out = Vec::with_capacity(ticks as usize);
    for t in 1..=ticks {
        r.bridge.dispatch(&mut r.sim, true, t);
        let y = pose(&r.sim).1;
        out.push((y, prev - y));
        prev = y;
    }
    out
}

/// **Quão rápido ele desce ENQUANTO a fresta cruza o corpo** — a maior descida
/// por tique dentro da janela em que o meio do corpo está dentro do buraco.
///
/// ⚠️ Preso à parede a descida é a autorada; solto, ela ACELERA. É a diferença
/// entre os dois regimes que nomeia o defeito, e o `slide/60` ao lado é a régua.
fn worst_drop_across_the_gap(r: &mut Rig, gap: f32) -> (f32, f32) {
    let lo = GAP_CENTER - gap * 0.5;
    let hi = GAP_CENTER + gap * 0.5;
    let mut worst = 0.0f32;
    let mut fell = 0.0f32;
    let mut entered = None;
    for (y, drop) in fall(r, 600) {
        if y <= hi && y >= lo - 1.5 {
            worst = worst.max(drop);
            if entered.is_none() {
                entered = Some(y);
            }
            fell = entered.unwrap_or(y) - y;
        }
    }
    (worst, fell)
}

/// **A TABELA** — a mesma parede e o mesmo personagem, frestas de alturas
/// diferentes.
///
/// O corpo mede **1,0 m** de caixa envolvente (cápsula `half_height 0,3` +
/// `radius 0,2`), então uma fresta abaixo disso deixa sempre pés OU ombros
/// contra a parede: toda linha desta tabela é geometria que o flanco vê e o meio
/// não.
#[test]
#[ignore = "sonda: imprime a tabela, nao afirma"]
fn measure_what_a_gap_costs() {
    const SLIDE: f32 = 3.0;
    println!(
        "fresta   pior descida/tique   regime   (escorregando = {:.4})",
        SLIDE / 60.0
    );
    for gap in [0.0f32, 0.2, 0.4, 0.6, 0.8] {
        let mut r = rig_gapped(SLIDE, 0.0, gap);
        let (worst, fell) = worst_drop_across_the_gap(&mut r, gap.max(0.2));
        let regime = if worst > SLIDE / 60.0 * 1.5 {
            "SOLTO"
        } else {
            "preso"
        };
        println!("{gap:>6.1}   {worst:>17.4}   {regime:<6}   caiu {fell:.3} m na janela");
    }
}

/// **A CONSEQUÊNCIA AFIADA: o pulo de parede é RECUSADO na fresta.**
///
/// ⚠️ O escorregamento quase não denuncia o defeito, e a tabela acima é a prova
/// — a **COLA** que o cabeçalho do `platform_wall` documenta (atrito + gravidade
/// do ápice) segura o personagem mesmo sem a assistência, então perder o
/// agarrar-se por oito tiques custa centímetros. O **pulo** não tem cola: ou a
/// lei vê parede naquele tique, ou o aperto do botão não faz nada.
///
/// A cena desce até a fresta, solta a entrada por um tique e aperta pulo.
#[test]
#[ignore = "sonda: imprime a tabela, nao afirma"]
fn measure_whether_the_wall_jump_survives_a_gap() {
    const SLIDE: f32 = 3.0;
    const JUMP: f32 = 2.0;
    println!("fresta   pico apos o pulo   subiu");
    for gap in [0.0f32, 0.6, 0.7, 0.75, 0.8, 0.9] {
        let mut r = rig_gapped(SLIDE, JUMP, gap);
        r.bridge.set_player_input(r.player, into_wall());
        // Desce ate o meio do corpo estar DENTRO da fresta.
        let mut t = 0u64;
        while t < 400 && pose(&r.sim).1 > GAP_CENTER {
            t += 1;
            r.bridge.dispatch(&mut r.sim, true, t);
        }
        let at_gap = pose(&r.sim).1;
        // Aperta pulo, segurando a direcao da parede (o gesto real).
        let mut input = into_wall();
        input.jump = true;
        r.bridge.set_player_input(r.player, input);
        let mut peak = at_gap;
        for _ in 0..90 {
            t += 1;
            r.bridge.dispatch(&mut r.sim, true, t);
            peak = peak.max(pose(&r.sim).1);
        }
        println!(
            "{gap:>6.2}   {peak:>16.4}   {:>6.3} m   (y={at_gap:.3}, sobreposicao pe/ombro {:.3} m)",
            peak - at_gap,
            (1.0 - gap) * 0.5
        );
    }
}

/// **A trajetória crua**, para ler o que a tabela resume.
#[test]
#[ignore = "sonda: imprime a tabela, nao afirma"]
fn measure_the_trajectory_through_a_gap() {
    const SLIDE: f32 = 3.0;
    const GAP: f32 = 0.4;
    let mut r = rig_gapped(SLIDE, 0.0, GAP);
    println!(
        "fresta em [{:.2}, {:.2}]; corpo mede 1,0 m; escorregando = {:.4}/tique",
        GAP_CENTER - GAP * 0.5,
        GAP_CENTER + GAP * 0.5,
        SLIDE / 60.0
    );
    for (i, (y, drop)) in fall(&mut r, 400).into_iter().enumerate() {
        if y < GAP_CENTER + 2.0 && y > GAP_CENTER - 3.0 && i % 3 == 0 {
            println!("t={:>3} y={y:>8.4} descida={drop:>7.4}", i + 1);
        }
    }
}
