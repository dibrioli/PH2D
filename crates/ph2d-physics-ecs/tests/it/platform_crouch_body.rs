//! **O corpo baixo** (`docs/GameEngineFuncional/09_abaixar_plano.md`) — com a cápsula do herói de
//! fábrica o agachar desce pela FORMA (encolhe por cima, os pés no sítio) e a perna fica: o corpo
//! nunca encosta no chão, e uma folha de sprites com os pés na mesma linha continua a pisá-lo.

use crate::platform_crouch_rig as rig_fixture;

use rig_fixture::{Rig, crouch_right, pose, rig_body, walk_right};

/// A cápsula do Add ▸ Character (`ph2d-app-game::object_add::CAPSULA`) e a perna dela.
const HERO: [f32; 2] = [0.15, 0.2];
const FLOAT: f32 = 0.45;
/// O agachar do Add ▸ Character: o troço recto inteiro (0,30) — topo de 0,80 para 0,50.
const LOW: f32 = 0.15;
const TOP_STANDING: f32 = 0.80;
const TOP_LOW: f32 = 0.50;
/// Abaixo do topo de pé, acima do agachado.
const CEILING: f32 = 0.60;

fn top(r: &Rig) -> f32 {
    r.bridge.body_extent(r.player).expect("corpo").1[1]
}

fn base(r: &Rig) -> f32 {
    r.bridge.body_extent(r.player).expect("corpo").0[1]
}

fn travel(ceiling: f32, down: bool) -> f32 {
    let mut r = rig_body(HERO, FLOAT, LOW, Some(ceiling));
    r.run(0, 300, if down { crouch_right() } else { walk_right() });
    pose(&r.sim).0
}

/// Agachado, o topo desce o autorado e a base e o centro ficam; o herói passa sob um tecto que de pé o
/// pára. Controlo: um tecto abaixo do topo agachado pára-o também (o que passa é a forma encolhida).
#[test]
fn the_low_body_passes_under_a_ceiling_that_stops_him_standing() {
    let mut up = rig_body(HERO, FLOAT, LOW, None);
    up.run(0, 90, walk_right());
    let mut low = rig_body(HERO, FLOAT, LOW, None);
    low.run(0, 90, crouch_right());
    assert!(
        (top(&up) - TOP_STANDING).abs() < 0.01,
        "de pé: {}",
        top(&up)
    );
    assert!(
        (top(&low) - TOP_LOW).abs() < 0.01,
        "agachado: {}",
        top(&low)
    );
    assert!((base(&low) - base(&up)).abs() < 0.005, "os pés no sítio");
    assert!((pose(&low.sim).1 - FLOAT).abs() < 0.01, "a perna não desce");

    let standing = travel(CEILING, false);
    let crouched = travel(CEILING, true);
    let too_low = travel(TOP_LOW - 0.1, true);
    assert!(standing < 5.5, "de pé o tecto pára-o: x={standing:.2}");
    assert!(crouched > 9.0, "agachado passa: x={crouched:.2}");
    assert!(
        too_low < 5.5,
        "controlo, tecto abaixo do agachado: x={too_low:.2}"
    );
}

/// Soltar ↓ debaixo do tecto NÃO levanta (continua baixo e a andar); soltar a céu aberto levanta e a
/// forma de pé volta.
#[test]
fn under_the_ceiling_he_stays_low_and_under_open_sky_he_stands() {
    let mut r = rig_body(HERO, FLOAT, LOW, Some(CEILING));
    let t = r.run(0, 300, crouch_right());
    let (x, _) = pose(&r.sim);
    assert!(x > 7.0, "a fixture põe-no debaixo do tecto: x={x:.2}");
    r.run(t, 60, walk_right());
    assert!(r.bridge.player_view(r.player).is_some_and(|v| v.crouching));
    assert!(top(&r) < CEILING, "continua baixo: topo {}", top(&r));
    let (x2, _) = pose(&r.sim);
    assert!(x2 > x + 0.5, "e anda por baixo: {x:.2} → {x2:.2}");

    let mut o = rig_body(HERO, FLOAT, LOW, None);
    let t = o.run(0, 60, crouch_right());
    assert!((top(&o) - TOP_LOW).abs() < 0.01);
    o.run(t, 30, walk_right());
    assert!(!o.bridge.player_view(o.player).is_some_and(|v| v.crouching));
    assert!(
        (top(&o) - TOP_STANDING).abs() < 0.01,
        "a céu aberto levanta: {}",
        top(&o)
    );
}
