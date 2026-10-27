//! Laço ou ≥ 2 condutores → `kcl-ezpz` (05 §2.3). Cada sólido = 3 pontos (origem, `+x̂`, `+ŷ` levados pela pose)
//! com as 3 distâncias fixas, de modo que a escala autorada viaja; cada pino = um ponto partilhado com as 3
//! distâncias a cada sólido (âncoras não colineares: nunca degenera, nem com o pino sobre o eixo). Condutores =
//! `Fixed` nos 6 escalares. O palpite é a pose AUTORADA do quadro, nunca a do anterior (tocar × scrub = o mesmo bit).

use std::collections::BTreeMap;

use kcl_ezpz::datatypes::inputs::DatumPoint;
use kcl_ezpz::{Config, Constraint as K, ConstraintRequest as R, IdGenerator};

use crate::{Corpo, Pose2};

/// Um pino entre duas raízes, com o ponto em coordenadas locais de cada uma.
pub(crate) struct Pin {
    pub a: Corpo,
    pub qa: [f64; 2],
    pub b: Corpo,
    pub qb: [f64; 2],
}

struct Quadro {
    o: DatumPoint,
    u: DatumPoint,
    v: DatumPoint,
}

pub(crate) fn resolve(
    pose: &BTreeMap<Corpo, Pose2>,
    fixo: impl Fn(Corpo) -> bool,
    macio: impl Fn(Corpo) -> bool,
    pins: &[Pin],
) -> Option<BTreeMap<Corpo, Pose2>> {
    let mut ids = IdGenerator::default();
    let mut palpite: Vec<(u32, f64)> = Vec::new();
    let mut reqs = Vec::new();
    let mut macias = std::collections::BTreeSet::new();
    let por = |d: DatumPoint, p: [f64; 2], palpite: &mut Vec<(u32, f64)>| {
        palpite.push((d.id_x(), p[0]));
        palpite.push((d.id_y(), p[1]));
    };

    let mut quadros = BTreeMap::new();
    for (&c, &p) in pose {
        let q = Quadro {
            o: DatumPoint::new(&mut ids),
            u: DatumPoint::new(&mut ids),
            v: DatumPoint::new(&mut ids),
        };
        let (o, u, v) = (
            p.ponto([0.0, 0.0]),
            p.ponto([1.0, 0.0]),
            p.ponto([0.0, 1.0]),
        );
        for (d, x) in [(q.o, o), (q.u, u), (q.v, v)] {
            por(d, x, &mut palpite);
            if fixo(c) {
                // A mão que cede perde a prioridade: fica onde as colas deixam (o arrasto da W4).
                let r = |k| {
                    if macio(c) {
                        R::new(k, 1)
                    } else {
                        R::highest_priority(k)
                    }
                };
                if macio(c) {
                    macias.extend([reqs.len(), reqs.len() + 1]);
                }
                reqs.push(r(K::Fixed(d.id_x(), x[0])));
                reqs.push(r(K::Fixed(d.id_y(), x[1])));
            }
        }
        reqs.push(R::highest_priority(K::Distance(q.o, q.u, p.sx.abs())));
        reqs.push(R::highest_priority(K::Distance(q.o, q.v, p.sy.abs())));
        reqs.push(R::highest_priority(K::Distance(q.u, q.v, p.sx.hypot(p.sy))));
        quadros.insert(c, q);
    }

    for pin in pins {
        let d = DatumPoint::new(&mut ids);
        let (pa, pb) = (pose[&pin.a].ponto(pin.qa), pose[&pin.b].ponto(pin.qb));
        por(
            d,
            [(pa[0] + pb[0]) / 2.0, (pa[1] + pb[1]) / 2.0],
            &mut palpite,
        );
        for (c, q) in [(pin.a, pin.qa), (pin.b, pin.qb)] {
            let (s, f) = (pose[&c], &quadros[&c]);
            let l = [q[0] * s.sx, q[1] * s.sy];
            reqs.push(R::highest_priority(K::Distance(d, f.o, l[0].hypot(l[1]))));
            reqs.push(R::highest_priority(K::Distance(
                d,
                f.u,
                (l[0] - s.sx).hypot(l[1]),
            )));
            reqs.push(R::highest_priority(K::Distance(
                d,
                f.v,
                l[0].hypot(l[1] - s.sy),
            )));
        }
    }

    let saida = kcl_ezpz::solve(&reqs, palpite, Config::default()).ok()?;
    // Só a mão que cede pode ficar por cumprir.
    if saida.unsatisfied().iter().any(|k| !macias.contains(k)) {
        return None;
    }
    let f = saida.final_values();
    let em = |d: DatumPoint| [f[d.id_x() as usize], f[d.id_y() as usize]];
    Some(
        pose.iter()
            .map(|(&c, &p)| {
                if fixo(c) && !macio(c) {
                    return (c, p);
                }
                let (o, u) = (em(quadros[&c].o), em(quadros[&c].u));
                // `u − o` é a rotação aplicada a `(sx, 0)`: com `sx < 0` (espelho) aponta para trás.
                let s = p.sx.signum();
                let rot = (s * (u[1] - o[1])).atan2(s * (u[0] - o[0]));
                (
                    c,
                    Pose2 {
                        x: o[0],
                        y: o[1],
                        rot,
                        ..p
                    },
                )
            })
            .collect(),
    )
}
