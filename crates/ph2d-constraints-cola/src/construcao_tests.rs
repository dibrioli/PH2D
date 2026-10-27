//! A construção dá a MESMA resposta que a ezpz onde as duas respondem (05 §4.2): o quadrilátero da bancada do
//! kill, a volta inteira, com escala não uniforme, espelho e o balancim como condutor.

use std::collections::BTreeMap;

use super::{PinI, executa, receita};
use crate::ez::{self, Pin};
use crate::{Corpo, Pose2, local};

/// A receita e a conta, sobre as poses e os pinos por `Corpo`.
fn resolve(
    pose: &BTreeMap<Corpo, Pose2>,
    fixo: impl Fn(Corpo) -> bool,
    pins: &[Pin],
) -> Option<BTreeMap<Corpo, Pose2>> {
    let corpos: Vec<Corpo> = pose.keys().copied().collect();
    let i = |c: Corpo| corpos.binary_search(&c).unwrap();
    let pins: Vec<PinI> = pins
        .iter()
        .map(|p| PinI {
            a: i(p.a),
            qa: p.qa,
            b: i(p.b),
            qb: p.qb,
        })
        .collect();
    let fixa: Vec<bool> = corpos.iter().map(|&c| fixo(c)).collect();
    let mut v: Vec<Pose2> = pose.values().copied().collect();
    executa(&receita(&fixa, &pins)?, &mut v, &pins).then(|| corpos.into_iter().zip(v).collect())
}

fn pose(x: f64, y: f64, rot: f64, sx: f64, sy: f64) -> Pose2 {
    Pose2 { x, y, rot, sx, sy }
}

/// Chão 1, manivela 2, acoplador 3, balancim 4 (a geometria da bancada `kill.rs`).
fn quadrilatero(s3: (f64, f64), s4: (f64, f64)) -> (BTreeMap<Corpo, Pose2>, Vec<Pin>) {
    let a = 3.5f64.powi(2) / 6.0;
    let h = (3.5f64.powi(2) - a * a).sqrt();
    let p = BTreeMap::from([
        (1, pose(0.0, 0.0, 0.0, 1.0, 1.0)),
        (2, pose(0.0, 0.0, 0.0, 1.0, 1.0)),
        (3, pose(1.0, 0.0, h.atan2(a), s3.0, s3.1)),
        (4, pose(4.0, 0.0, h.atan2(a - 3.0), s4.0, s4.1)),
    ]);
    let pins = [
        (1, 2, [0.0, 0.0]),
        (2, 3, [1.0, 0.0]),
        (3, 4, [1.0 + a, h]),
        (1, 4, [4.0, 0.0]),
    ]
    .into_iter()
    .map(|(i, j, m)| Pin {
        a: i,
        qa: local(p[&i], m).unwrap(),
        b: j,
        qb: local(p[&j], m).unwrap(),
    })
    .collect();
    (p, pins)
}

fn desvio(p: Pose2, q: Pose2) -> f64 {
    let d = (p.rot - q.rot).rem_euclid(std::f64::consts::TAU);
    let da = d.min(std::f64::consts::TAU - d);
    (p.x - q.x).abs().max((p.y - q.y).abs()).max(da)
}

#[test]
fn the_construction_gives_what_ezpz_gives_around_the_whole_turn() {
    let mut pior: f64 = 0.0;
    let mut vistos = 0;
    for (s3, s4) in [
        ((1.0, 1.0), (1.0, 1.0)),
        ((1.7, 0.6), (1.0, 1.0)),
        ((1.0, 1.0), (-1.0, 1.0)),
        ((0.5, -2.0), (1.3, 0.8)),
    ] {
        for condutor in [2, 4] {
            for k in 0..48 {
                let (mut p, pins) = quadrilatero(s3, s4);
                let delta = f64::from(k) * 7.5f64.to_radians();
                p.get_mut(&condutor).unwrap().rot += if condutor == 2 {
                    delta
                } else {
                    delta / 8.0 - 0.5
                };
                let fixo = |c: Corpo| c == 1 || c == condutor;
                let nossa = resolve(&p, fixo, &pins);
                let dela = ez::resolve(&p, fixo, |_| false, &pins);
                match (nossa, dela) {
                    (Some(n), Some(e)) => {
                        for c in 1..=4 {
                            pior = pior.max(desvio(n[&c], e[&c]));
                        }
                        vistos += 1;
                    }
                    (None, None) => {}
                    (n, e) => panic!(
                        "{s3:?} {s4:?} condutor {condutor} passo {k}: nossa {n:?} × ezpz {e:?}"
                    ),
                }
            }
        }
    }
    println!("construção × ezpz: {vistos} casos, pior desvio {pior:e}");
    assert!(vistos >= 280, "{vistos}");
    // A ezpz pára no critério de convergência dela (medido: 8,9e-9); a construção fecha os pinos a `TOL`.
    assert!(pior <= 1e-7, "{pior:e}");
}

/// Um laço que não fecha não se constrói: quem diz «conflito» continua a ser a ezpz.
#[test]
fn a_loop_that_cannot_close_is_not_constructed() {
    let (mut p, pins) = quadrilatero((1.0, 1.0), (1.0, 1.0));
    p.get_mut(&4).unwrap().x += 10.0;
    assert!(resolve(&p, |c| c == 1 || c == 2 || c == 4, &pins).is_none());
    p.get_mut(&4).unwrap().x -= 10.0;
    p.get_mut(&1).unwrap().x = 50.0;
    assert!(resolve(&p, |c| c == 1 || c == 2, &pins).is_none());
}
