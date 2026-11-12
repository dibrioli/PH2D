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
    // A lei resolve o arranque pelas fases: o ápice É a Jump Height (era 2,04 com v0 neutro).
    assert!((rise(&a) - 2.0).abs() < 0.02, "ápice {}", rise(&a));
    // Sem terreno a lei desce abaixo do arranque até ao limite de queda (o corte é da cena).
    assert!(
        a.points.last().unwrap()[1] <= -super::MAX_FALL_M,
        "desce abaixo da linha dos pés: {:?}",
        a.points.last()
    );
    assert!((1.40..1.50).contains(&a.air_time), "no ar {}", a.air_time);
    // Correndo a 6 m/s, o alcance na linha dos pés é a velocidade vezes o tempo no ar.
    let back = (a.air_time / W.dt).round() as usize;
    assert!(
        a.points[back][1] <= 0.0 && a.points[back - 1][1] > 0.0,
        "volta à linha no air_time"
    );
    let reach = a.points[back][0];
    assert!((reach - 6.0 * a.air_time).abs() < 0.2, "alcance {reach}");
}

#[test]
fn o_pulo_curto_corta_e_o_parado_vai_menos_longe() {
    let arcs = jump_arcs(&cfg(), W);
    assert!(
        rise(&arcs.short) < 0.5 * rise(&arcs.full),
        "o corte baixa o pulo"
    );
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
    assert!(
        (first[1] - rise(&arcs.full)).abs() < 0.05,
        "do ápice: {first:?}"
    );
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
    c.jump.jump_height = 3.0;
    let a = full_arc(&c, W);
    assert!((rise(&a) - 3.0).abs() < 0.02, "ápice {}", rise(&a));
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
    assert!(speed_for_apex(&cfg(), w, 2.0).is_none());
    let a = full_arc(&cfg(), w);
    assert!(a.points.len() <= MAX_ARC_TICKS + 1, "o teto segura");
}

#[test]
fn a_sensacao_conserva_a_altura_e_se_reconhece() {
    for f in Feel::ALL {
        let mut c = cfg();
        apply_feel(&mut c, f);
        assert_eq!(feel_of(&c.jump), Some(f));
        assert_eq!(c.jump.jump_height, 2.0, "{f:?}: a altura fica");
        let after = rise(&full_arc(&c, W));
        assert!((after - 2.0).abs() < 0.02, "{f:?}: ápice {after}");
    }
    assert_eq!(feel_of(&cfg().jump), None, "o padrão de hoje não é nenhuma");
}

#[test]
fn mexer_num_numero_apaga_a_sensacao() {
    let mut c = cfg();
    apply_feel(&mut c, Feel::Heavy);
    c.jump.fall_gravity += 0.1;
    assert_eq!(feel_of(&c.jump), None);
}

#[test]
fn as_tres_sensacoes_se_distinguem_pelo_tempo_no_ar() {
    let air = |f| {
        let mut c = cfg();
        apply_feel(&mut c, f);
        full_arc(&c, W).air_time
    };
    let (fl, sn, he) = (air(Feel::Floaty), air(Feel::Snappy), air(Feel::Heavy));
    assert!(fl > sn && sn > he, "Floaty {fl} > Snappy {sn} > Heavy {he}");
}

/// A lei nova (GEF 04): o ápice do pulo inteiro e o de cada pulo do ar = a altura escrita, para
/// as três sensações, o padrão e alturas de 0,5 a 2,5 m. O resto é o erro do sub-passo do mundo
/// (`−v0·dt/(2·substeps)`, a lei não vê os sub-passos): 2,3 cm a 6 m com 4 sub-passos.
#[test]
fn o_apice_e_a_altura_escrita_em_todas_as_sensacoes() {
    let mut falhas = Vec::new();
    for feel in [
        None,
        Some(Feel::Floaty),
        Some(Feel::Snappy),
        Some(Feel::Heavy),
    ] {
        for h in [0.5_f32, 1.0, 2.0, 2.5] {
            let mut c = cfg();
            if let Some(f) = feel {
                apply_feel(&mut c, f);
            }
            c.jump.jump_height = h;
            c.jump.air_jump_height = 0.5 * h;
            c.jump.air_jumps = 1;
            let arcs = jump_arcs(&c, W);
            let ground = rise(&arcs.full);
            let air = rise(&arcs.air[0]) - arcs.air[0].points[0][1];
            if (ground - h).abs() >= 0.02 || (air - 0.5 * h).abs() >= 0.02 {
                falhas.push(format!("{feel:?} h={h}: chão {ground:.3} ar {air:.3}"));
            }
        }
    }
    assert!(falhas.is_empty(), "o ápice mente: {falhas:?}");
}

/// Sonda de custo: as curvas são simuladas DUAS vezes por quadro (alça e desenho).
/// `PH2D_RELEASE="mede velocidade" cargo test --release -p ph2d-platformer custo_das_curvas -- --ignored --nocapture`
#[test]
#[ignore = "sonda: imprime o custo"]
fn custo_das_curvas() {
    let mut c = cfg();
    c.jump.air_jumps = 1;
    let mut best = f64::MAX;
    for _ in 0..5 {
        let t = std::time::Instant::now();
        for _ in 0..200 {
            std::hint::black_box(jump_arcs(std::hint::black_box(&c), W));
        }
        best = best.min(t.elapsed().as_secs_f64() / 200.0);
    }
    eprintln!(
        "jump_arcs (Air Jumps 1): {:.1} µs por chamada (mínimo de 5 blocos)",
        best * 1e6
    );
}
