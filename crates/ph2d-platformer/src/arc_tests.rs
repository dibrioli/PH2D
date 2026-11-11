//! Os gates da curva do pulo e da sensação. O da lei contra o player REAL mora em
//! `ph2d-physics-ecs/tests/it/jump_arc.rs`.

use super::*;
use crate::feel::{Feel, apply_feel, feel_of};

const W: ArcWorld = ArcWorld {
    gravity: [0.0, -9.81],
    dt: 1.0 / 60.0,
    facing: 1.0,
    substeps: 4,
};

fn cfg() -> PlayerConfig {
    PlayerConfig::STARTING_POINT
}

fn rise(a: &JumpArc) -> f32 {
    a.apex[1]
}

#[test]
fn o_arco_inteiro_sobe_a_altura_e_volta_ao_chao() {
    let a = full_arc(&cfg(), W);
    // Com Peak G 0,5 o topo alonga: um pouco acima dos 2 m autorados (2,04 m simulados).
    assert!((2.0..2.1).contains(&rise(&a)), "ápice {}", rise(&a));
    assert!(a.points.last().unwrap()[1] <= 0.0, "termina na linha dos pés");
    assert!((1.40..1.50).contains(&a.air_time), "no ar {}", a.air_time);
    // Correndo a 6 m/s, o alcance é a velocidade vezes o tempo no ar.
    let reach = a.points.last().unwrap()[0];
    assert!((reach - 6.0 * a.air_time).abs() < 0.2, "alcance {reach}");
}

#[test]
fn o_pulo_curto_corta_e_o_parado_vai_menos_longe() {
    let arcs = jump_arcs(&cfg(), W);
    assert!(rise(&arcs.short) < 0.5 * rise(&arcs.full), "o corte baixa o pulo");
    let far = |a: &JumpArc| a.points.last().unwrap()[0];
    assert!(far(&arcs.standing) < far(&arcs.full));
    assert!(far(&arcs.standing) > 0.0, "a direcção presa acelera no ar");
    assert!(arcs.air.is_empty(), "sem Air Jumps não há curva do ar");
}

#[test]
fn cada_pulo_do_ar_sai_do_apice_do_anterior() {
    let mut c = cfg();
    c.jump.air_jumps = 2;
    let arcs = jump_arcs(&c, W);
    assert_eq!(arcs.air.len(), 2);
    let first = arcs.air[0].points[0];
    assert!((first[1] - rise(&arcs.full)).abs() < 0.05, "do ápice: {first:?}");
    assert!(rise(&arcs.air[1]) > rise(&arcs.air[0]), "a corrente sobe");
}

#[test]
fn a_esquerda_e_o_espelho() {
    let r = full_arc(&cfg(), W);
    let l = full_arc(&cfg(), ArcWorld { facing: -1.0, ..W });
    assert!((r.apex[0] + l.apex[0]).abs() < 1e-4 && (r.apex[1] - l.apex[1]).abs() < 1e-4);
}

#[test]
fn a_alca_cai_no_apice() {
    let mut c = cfg();
    let h = height_for_apex(&c, W, 3.0).expect("há solução");
    c.jump.jump_height = h;
    let a = full_arc(&c, W);
    assert!((rise(&a) - 3.0).abs() < 0.01, "ápice {}", rise(&a));
    let s = speed_for_apex(&c, W, 5.0).expect("há tempo até ao topo");
    c.walk.speed = s;
    assert!((full_arc(&c, W).apex[0] - 5.0).abs() < 0.01);
}

#[test]
fn sem_gravidade_ninguem_resolve() {
    let w = ArcWorld {
        gravity: [0.0, 0.0],
        ..W
    };
    assert!(height_for_apex(&cfg(), w, 2.0).is_none());
    assert!(height_for_apex(&cfg(), W, 0.0).is_none());
    let a = full_arc(&cfg(), w);
    assert!(a.points.len() <= MAX_ARC_TICKS + 1, "o teto segura");
}

#[test]
fn a_sensacao_conserva_a_altura_e_se_reconhece() {
    for f in Feel::ALL {
        let mut c = cfg();
        let before = rise(&full_arc(&c, W));
        apply_feel(&mut c, f, W);
        assert_eq!(feel_of(&c.jump), Some(f));
        let after = rise(&full_arc(&c, W));
        assert!((after - before).abs() < 0.01, "{f:?}: {before} → {after}");
    }
    assert_eq!(feel_of(&cfg().jump), None, "o padrão de hoje não é nenhuma");
}

#[test]
fn mexer_num_numero_apaga_a_sensacao() {
    let mut c = cfg();
    apply_feel(&mut c, Feel::Heavy, W);
    c.jump.fall_gravity += 0.1;
    assert_eq!(feel_of(&c.jump), None);
}

#[test]
fn as_tres_sensacoes_se_distinguem_pelo_tempo_no_ar() {
    let air = |f| {
        let mut c = cfg();
        apply_feel(&mut c, f, W);
        full_arc(&c, W).air_time
    };
    let (fl, sn, he) = (air(Feel::Floaty), air(Feel::Snappy), air(Feel::Heavy));
    assert!(fl > sn && sn > he, "Floaty {fl} > Snappy {sn} > Heavy {he}");
}
