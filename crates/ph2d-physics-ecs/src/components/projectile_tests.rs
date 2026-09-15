//! Gates do componente — a porta componente ⇄ lei, e o que o ficheiro guarda.

use super::*;

/// ⚠️ **A porta é uma IDA E VOLTA** — um campo esquecido numa das metades é um knob que o artista
/// mexe e que a lei nunca vê, e ele lê-se exactamente como um controlo morto.
#[test]
fn a_porta_componente_lei_e_uma_ida_e_volta() {
    let l = ProjectileLaw {
        initial_speed: 7.5,
        acceleration: -3.0,
        max_speed: 40.0,
        gravity: 9.81,
        bounciness: 0.35,
        max_bounces: 7,
        range: 12.5,
        face_velocity: false,
        homing_accel: 250.0,
    };
    let c = ProjectileMotion::from_law(l, 0xDEAD_BEEF);
    assert_eq!(c.law(), l, "a ida e volta perdeu um campo");
    assert_eq!(
        c.homing_target, 0xDEAD_BEEF,
        "o alvo nao viaja na lei, mas viaja no componente"
    );
}

/// ⚠️ **Os defaults do componente SÃO os da lei** — uma segunda cópia deles aqui divergiria no dia
/// em que um mudasse, e o painel mostraria números que a cena não tem.
#[test]
fn os_defaults_do_componente_sao_os_da_lei() {
    assert_eq!(ProjectileMotion::default().law(), ProjectileLaw::default());
    assert_eq!(
        ProjectileMotion::default().homing_target,
        0,
        "sem alvo e' ZERO, que e' a convencao de «ninguem» desta casa"
    );
}

/// ⚠️ **O componente atravessa o ficheiro** — o `postcard` é posicional, e um campo que não
/// sobrevivesse à ida e volta seria uma cena gravada a abrir com outro voo.
#[test]
fn o_componente_atravessa_o_ficheiro() {
    let c = ProjectileMotion::from_law(
        ProjectileLaw {
            gravity: 9.81,
            max_bounces: 3,
            range: 20.0,
            ..ProjectileLaw::default()
        },
        42,
    );
    let bytes = postcard::to_allocvec(&c).expect("serializa");
    let volta: ProjectileMotion = postcard::from_bytes(&bytes).expect("desserializa");
    assert_eq!(volta, c);
}
