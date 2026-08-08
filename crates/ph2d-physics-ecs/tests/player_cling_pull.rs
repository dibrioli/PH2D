//! **UMA PERNA NÃO PUXA O CHÃO PARA SI** (W-ClingPull).
//!
//! Report do Enio (2026-08-07): *"ao pular na jangada sobre a água, ao primeiro
//! toque, em vez de empurrar a jangada para baixo, a jangada é ATRAÍDA para o
//! player, e só depois de se aproximar um pouco é que recebe o impulso"*.
//!
//! Medido antes da cura (`measure_landing::measure_first_touch_on_a_raft`): ela
//! **SOBE 96,90 mm** acima da linha d'água antes de descer.
//!
//! # ⚠️ A causa está escrita uma linha acima do sítio
//!
//! ```text
//! // Positivo = está BAIXO demais e a mola empurra para cima; negativo = está
//! // alto demais dentro do `cling_distance` e ela PUXA para baixo.
//! let offset = cfg.float_height - s.distance;
//! ```
//!
//! Na faixa de *cling* o `offset` é negativo, a perna puxa o personagem para
//! baixo, e a 3ª lei transmite **fielmente** o oposto: puxa a jangada para cima.
//! O *cling* é uma **conveniência de modelagem** (é o que mantém o personagem
//! colado ao descer uma lomba), não um músculo — e nenhuma perna real puxa o
//! chão para si.
//!
//! # ⚠️ E a cura NÃO é um interruptor no sinal do `offset`
//!
//! Medido: em repouso o personagem assenta **2,3 mm ACIMA** da altura de
//! repouso (`0,9023` contra `0,900`) e converge para ela **por cima** — ou seja
//! o `offset` de repouso é levemente **negativo para sempre**. Um
//! `if offset >= 0` zeraria a reação em repouso e mataria a 3ª lei inteira: a
//! jangada deixaria de afundar sob um personagem parado.
//!
//! O que é fictício é só o **termo da mola** (`k·x`, que na faixa de cling chega
//! a `0,25 × 400 = 100 m/s²`), e não o **peso** (`− gravity`, que é o *"eu estou
//! apoiado nisto"*). A cura clampa o primeiro e preserva o segundo.

#[path = "platform_water_scene.rs"]
mod water;

use ph2d_ecs::SimWorld;
use ph2d_physics_ecs::PhysicsBridge;
use water::{FLOAT, pool, raft, subject, y_of};

/// A meia-altura da jangada do `platform_water_scene`.
const RAFT_HALF_Y: f32 = 0.2;

/// Assenta a jangada sozinha e devolve `(sim, bridge, linha d'água, tick)`.
fn settled_raft() -> (SimWorld, PhysicsBridge, f32, u64) {
    let mut sim = SimWorld::new();
    pool(&mut sim, 0.0);
    raft(&mut sim, 0.5);
    let mut bridge = PhysicsBridge::new();
    for t in 1..=300u64 {
        bridge.dispatch(&mut sim, true, t);
    }
    let rest = y_of(&sim, "Raft");
    (sim, bridge, rest, 300)
}

/// **A jangada nunca sobe ao encontro de quem cai nela.**
///
/// ⚠️ **A janela termina no instante em que ela é EMPURRADA, e isso é o oráculo,
/// não uma conveniência.** O report é sobre o *primeiro toque* — *"em vez de
/// empurrar para baixo, ela é atraída"* —, e uma jangada é uma **mola de
/// empuxo**: depois de afundada ela **volta acima da linha d'água** por
/// oscilação, que é física honesta e nada tem a ver com a perna. A primeira
/// versão deste gate media os três segundos inteiros e reportava `5,73 mm` de
/// *"puxada"* que era o **rebote**.
#[test]
fn a_raft_is_never_pulled_up_toward_a_landing_player() {
    let (mut sim, mut bridge, rest, t0) = settled_raft();
    let _ = subject(&mut sim, true, rest + RAFT_HALF_Y + 3.0);

    let mut highest = rest;
    for t in (t0 + 1)..=(t0 + 180) {
        bridge.dispatch(&mut sim, true, t);
        let ry = y_of(&sim, "Raft");
        if ry < rest {
            break; // ela já está a ser empurrada: o que vier depois é rebote.
        }
        highest = highest.max(ry);
    }
    let rise_mm = (highest - rest) * 1000.0;

    // ⚠️ **O CONTROLE: quanto ela se move SOZINHA na mesma janela.** Sem ele
    // este gate não distingue *"o player a puxou"* de *"ela ainda estava a
    // assentar"* — e a fixture nasceu sem ele.
    let (mut solo, mut b2, solo_rest, s0) = settled_raft();
    let mut solo_high = solo_rest;
    for t in (s0 + 1)..=(s0 + 180) {
        b2.dispatch(&mut solo, true, t);
        solo_high = solo_high.max(y_of(&solo, "Raft"));
    }
    let wander_mm = (solo_high - solo_rest) * 1000.0;

    assert!(
        rise_mm - wander_mm < 5.0,
        "a jangada subiu {rise_mm:.2} mm ao encontro do player ANTES de ser empurrada, \
         e sozinha ela sobe {wander_mm:.2} mm => atribuivel ao player: {:.2} mm \
         (antes da cura: 96,90). Uma perna nao puxa o chao para si.",
        rise_mm - wander_mm
    );
}

/// **E ela AFUNDA sob quem pousou** — a metade que tem de sobreviver.
///
/// ⚠️ **Este gate é o que impede a cura de ser "zere a reação"**: o peso é real
/// e continua a chegar. Sem ele, apagar o `support` inteiro passaria no gate
/// acima e mataria a 3ª lei em silêncio.
#[test]
fn a_raft_still_sinks_under_a_player_that_stands_on_it() {
    let (mut sim, mut bridge, rest, t0) = settled_raft();
    let _ = subject(&mut sim, true, rest + RAFT_HALF_Y + FLOAT);

    for t in (t0 + 1)..=(t0 + 600) {
        bridge.dispatch(&mut sim, true, t);
    }
    let loaded = y_of(&sim, "Raft");
    assert!(
        loaded < rest - 0.02,
        "a jangada tem de AFUNDAR sob o personagem: repouso {rest:.4}, carregada \
         {loaded:.4} (delta {:.4})",
        loaded - rest
    );
}
