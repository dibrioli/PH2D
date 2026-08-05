//! **A varredura do AGACHAR** (W15) — os números que a wave escreveu saíram
//! daqui (CLAUDE.md §0).
//!
//! `cargo test -p ph2d-physics-ecs --test measure_crouch -- --ignored --nocapture`
//!
//! ⚠️ **Sondas, não gates:** elas imprimem e não afirmam. O que fica gateado é a
//! PROPRIEDADE (o `platform_crouch`); o que fica aqui é a tabela que escolheu
//! cada constante.

#[path = "platform_crouch_rig.rs"]
mod rig_fixture;

use rig_fixture::{BODY_HALF, CROUCH_HEIGHT, FLOAT_HEIGHT, crouch_right, pose, rig, walk_right};

/// **Quanto a silhueta BAIXA** — o número que decide sob que teto ele passa.
///
/// ⚠️ A previsão é exacta e vale a pena vê-la confirmada: baixar a perna baixa o
/// corpo INTEIRO pelo mesmo delta, porque o collider não muda de forma.
#[test]
#[ignore]
fn measure_how_far_the_silhouette_drops() {
    eprintln!("== a silhueta, de pe' e agachado ==");
    for h in [0.6_f32, 0.7, 0.8, 0.9] {
        let mut r = rig(h, None);
        r.run(0, 90, walk_right());
        let (_, up) = pose(&r.sim);
        let mut c = rig(h, None);
        c.run(0, 90, crouch_right());
        let (_, low) = pose(&c.sim);
        eprintln!(
            "  crouch_height {h:4.2} -> centro {up:5.3} -> {low:5.3}   topo {:5.3} -> {:5.3}   \
             (delta autorado {:5.3})",
            up + BODY_HALF,
            low + BODY_HALF,
            FLOAT_HEIGHT - h
        );
    }
}

/// **O PISO GEOMÉTRICO** — abaixo dele a cápsula enterra no chão, e a lei pura
/// não pode saber disso.
///
/// ⚠️ Esta sonda existe para o número ficar MEDIDO em vez de deduzido: a cápsula
/// mede `half_height + radius = 0,5` do centro para baixo, e o que se observa
/// abaixo disso é o solver a empurrar o corpo para fora do chão contra a mola.
#[test]
#[ignore]
fn measure_the_geometric_floor_of_a_crouch() {
    eprintln!("== abaixo de {BODY_HALF:.2} a capsula enterra ==");
    for h in [0.30_f32, 0.40, 0.50, 0.55, 0.60] {
        let mut r = rig(h, None);
        r.run(0, 120, crouch_right());
        let (_, y) = pose(&r.sim);
        eprintln!(
            "  crouch_height {h:4.2} -> centro {y:5.3}   erro contra o pedido {:+6.3}",
            y - h
        );
    }
}

/// **Sob que teto ele passa** — a varredura que escolheu a altura da marquise da
/// cena de smoke.
#[test]
#[ignore]
fn measure_which_ceiling_he_fits_under() {
    eprintln!("== quanto ele anda sob uma marquise, de pe' e agachado ==");
    for bottom in [1.10_f32, 1.25, 1.35, 1.50, 1.70] {
        let mut up = rig(CROUCH_HEIGHT, Some(bottom));
        up.run(0, 300, walk_right());
        let (ux, _) = pose(&up.sim);
        let mut low = rig(CROUCH_HEIGHT, Some(bottom));
        low.run(0, 300, crouch_right());
        let (lx, _) = pose(&low.sim);
        eprintln!("  fundo {bottom:4.2} -> de pe' x={ux:6.2}   agachado x={lx:6.2}");
    }
}

/// **A velocidade agachado** — o segundo número do par que o artista autora.
#[test]
#[ignore]
fn measure_how_slow_a_crouched_walk_is() {
    eprintln!("== distancia em 120 tiques ==");
    let mut up = rig(CROUCH_HEIGHT, None);
    up.run(0, 120, walk_right());
    let (ux, _) = pose(&up.sim);
    let mut low = rig(CROUCH_HEIGHT, None);
    low.run(0, 120, crouch_right());
    let (lx, _) = pose(&low.sim);
    eprintln!(
        "  de pe' {ux:6.3} m   agachado {lx:6.3} m   razao {:5.3}",
        lx / ux
    );
}

/// **Os números que a CENA DE SMOKE afirma** — medidos antes de serem escritos
/// nela (CLAUDE.md §0).
///
/// A cena 94 usa `float_height = 0,90` e `crouch_height = 0,55`, com o túnel
/// baixo em `1,20` e o alto em `1,60`. Esta sonda dirige exactamente esses
/// números e imprime onde o personagem para.
#[test]
#[ignore]
fn measure_the_smoke_scene_numbers() {
    const SMOKE_FLOAT: f32 = 0.9;
    const SMOKE_CROUCH: f32 = 0.55;
    let run = |bottom: f32, input: ph2d_physics_ecs::PlayerInput| {
        let mut r = rig(SMOKE_CROUCH, Some(bottom));
        r.player_cfg(|p| p.float_height = SMOKE_FLOAT);
        r.run(0, 300, input);
        pose(&r.sim)
    };
    eprintln!("== a cena 94, com os numeros que ela escreve ==");
    for bottom in [1.20_f32, 1.60] {
        let (ux, uy) = run(bottom, walk_right());
        let (lx, ly) = run(bottom, crouch_right());
        eprintln!(
            "  tunel {bottom:4.2} -> de pe' x={ux:6.2} (topo {:5.3})   \
             agachado x={lx:6.2} (topo {:5.3})",
            uy + BODY_HALF,
            ly + BODY_HALF
        );
    }
}
