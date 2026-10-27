//! Régua e compasso antes da ezpz (05 §4.2): um corpo preso por DOIS pinos a peças já conhecidas coloca-se; dois
//! corpos ligados por um pino, cada um preso a uma peça conhecida (a díade do quadrilátero), são um cruzamento de
//! círculos. A RECEITA (que passo, com que pinos) é estrutural e vai na cache; a conta corre por quadro e responde
//! só quando coloca TODOS os corpos e fecha TODOS os pinos — senão a ezpz decide. O ramo da díade é o mais perto
//! do palpite, a pose AUTORADA, como o da ezpz (tocar × scrub = o mesmo bit).

use crate::{Pose2, TOL};

type P = [f64; 2];

/// Um pino entre duas raízes do grupo (índices), com o ponto em coordenadas locais de cada uma.
#[derive(Clone, Copy, Debug)]
pub(crate) struct PinI {
    pub a: usize,
    pub qa: P,
    pub b: usize,
    pub qb: P,
}

/// Um pino visto de um dos lados: `(índice do pino, o corpo é o lado a)`.
type Lado = (usize, bool);

#[derive(Clone, Debug)]
pub(crate) enum Passo {
    Dois { c: usize, um: Lado, outro: Lado },
    Diade { pin: usize, de_a: Lado, de_b: Lado },
}

fn sub(a: P, b: P) -> P {
    [a[0] - b[0], a[1] - b[1]]
}

fn norma(a: P) -> f64 {
    a[0].hypot(a[1])
}

/// O ponto local com a escala do corpo (o que a rotação leva).
fn escalado(p: Pose2, q: P) -> P {
    [q[0] * p.sx, q[1] * p.sy]
}

fn tol(a: P) -> f64 {
    TOL * (1.0 + a[0].abs().max(a[1].abs()))
}

/// O ponto do pino no corpo deste lado, e o corpo do outro lado com o ponto nele.
fn ver(pins: &[PinI], (i, la): Lado) -> (P, usize, P) {
    let p = pins[i];
    if la {
        (p.qa, p.b, p.qb)
    } else {
        (p.qb, p.a, p.qa)
    }
}

/// Os pinos de `c` a corpos já conhecidos, pela ordem dos pinos.
fn presos(c: usize, pins: &[PinI], sabe: &[bool]) -> Vec<Lado> {
    pins.iter()
        .enumerate()
        .filter_map(|(i, p)| {
            if p.a == c && sabe[p.b] {
                Some((i, true))
            } else if p.b == c && sabe[p.a] {
                Some((i, false))
            } else {
                None
            }
        })
        .collect()
}

/// A receita, se a régua e o compasso chegam a todos; `None` = o grupo é da ezpz.
pub(crate) fn receita(fixo: &[bool], pins: &[PinI]) -> Option<Vec<Passo>> {
    let mut sabe = fixo.to_vec();
    let mut passos = Vec::new();
    while sabe.iter().any(|s| !s) {
        let dois = (0..sabe.len()).filter(|&c| !sabe[c]).find_map(|c| {
            let v = presos(c, pins, &sabe);
            let um = *v.first()?;
            let q = ver(pins, um).0;
            let outro = *v
                .iter()
                .skip(1)
                .find(|&&l| norma(sub(ver(pins, l).0, q)) > tol(q))?;
            Some(Passo::Dois { c, um, outro })
        });
        let passo = dois.or_else(|| {
            pins.iter().enumerate().find_map(|(pin, p)| {
                if sabe[p.a] || sabe[p.b] {
                    return None;
                }
                let de_a = *presos(p.a, pins, &sabe).first()?;
                let de_b = *presos(p.b, pins, &sabe).first()?;
                Some(Passo::Diade { pin, de_a, de_b })
            })
        })?;
        match passo {
            Passo::Dois { c, .. } => sabe[c] = true,
            Passo::Diade { pin, .. } => {
                sabe[pins[pin].a] = true;
                sabe[pins[pin].b] = true;
            }
        }
        passos.push(passo);
    }
    Some(passos)
}

/// A pose de `p` (rotação e translação novas, a escala dele) que leva `q1 → m1` e `q2 → m2`.
fn coloca(p: Pose2, (q1, m1): (P, P), (q2, m2): (P, P)) -> Option<Pose2> {
    let (l1, l2) = (escalado(p, q1), escalado(p, q2));
    let (d, dm) = (sub(l2, l1), sub(m2, m1));
    if norma(d) <= tol(l1) || (norma(d) - norma(dm)).abs() > tol(m1).max(tol(m2)) {
        return None;
    }
    let rot = (d[0] * dm[1] - d[1] * dm[0]).atan2(d[0] * dm[0] + d[1] * dm[1]);
    let (s, c) = rot.sin_cos();
    Some(Pose2 {
        x: m1[0] - (c * l1[0] - s * l1[1]),
        y: m1[1] - (s * l1[0] + c * l1[1]),
        rot,
        ..p
    })
}

/// O cruzamento dos círculos `(c1, r1)` e `(c2, r2)` mais perto de `perto`.
fn cruza(c1: P, r1: f64, c2: P, r2: f64, perto: P) -> Option<P> {
    let dc = sub(c2, c1);
    let d = norma(dc);
    if d <= tol(c1) || d > r1 + r2 + tol(c1) || d < (r1 - r2).abs() - tol(c1) {
        return None;
    }
    let a = (r1 * r1 - r2 * r2 + d * d) / (2.0 * d);
    let h = (r1 * r1 - a * a).max(0.0).sqrt();
    let (e, n) = ([dc[0] / d, dc[1] / d], [-dc[1] / d, dc[0] / d]);
    let base = [c1[0] + a * e[0], c1[1] + a * e[1]];
    let mais = [base[0] + h * n[0], base[1] + h * n[1]];
    let menos = [base[0] - h * n[0], base[1] - h * n[1]];
    Some(if norma(sub(menos, perto)) < norma(sub(mais, perto)) {
        menos
    } else {
        mais
    })
}

/// `pose` entra com o palpite (as fixas já na sua pose) e sai construída; `false` = a ezpz decide.
pub(crate) fn executa(passos: &[Passo], pose: &mut [Pose2], pins: &[PinI]) -> bool {
    let no_mundo = |pose: &[Pose2], l: Lado| {
        let (q, o, qo) = ver(pins, l);
        (q, pose[o].ponto(qo))
    };
    for passo in passos {
        match *passo {
            Passo::Dois { c, um, outro } => {
                let (a, b) = (no_mundo(pose, um), no_mundo(pose, outro));
                let Some(p) = coloca(pose[c], a, b) else {
                    return false;
                };
                pose[c] = p;
            }
            Passo::Diade { pin, de_a, de_b } => {
                let p = pins[pin];
                let (pa, pb) = (pose[p.a], pose[p.b]);
                let ((q1, m1), (q2, m2)) = (no_mundo(pose, de_a), no_mundo(pose, de_b));
                let r1 = norma(sub(escalado(pa, p.qa), escalado(pa, q1)));
                let r2 = norma(sub(escalado(pb, p.qb), escalado(pb, q2)));
                let (ja, jb) = (pa.ponto(p.qa), pb.ponto(p.qb));
                let perto = [(ja[0] + jb[0]) / 2.0, (ja[1] + jb[1]) / 2.0];
                let Some(j) = cruza(m1, r1, m2, r2, perto) else {
                    return false;
                };
                match (
                    coloca(pa, (q1, m1), (p.qa, j)),
                    coloca(pb, (q2, m2), (p.qb, j)),
                ) {
                    (Some(a), Some(b)) => {
                        pose[p.a] = a;
                        pose[p.b] = b;
                    }
                    _ => return false,
                }
            }
        }
    }
    pins.iter().all(|p| {
        let (a, b) = (pose[p.a].ponto(p.qa), pose[p.b].ponto(p.qb));
        norma(sub(a, b)) <= tol(a)
    })
}

#[cfg(test)]
#[path = "construcao_tests.rs"]
mod tests;
