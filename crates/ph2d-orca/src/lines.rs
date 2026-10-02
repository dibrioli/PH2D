//! **Os semi-planos** — um por vizinho e um por parede (o artigo, §4 e §5.1).

use crate::v2::{EPS, V2, abs_sq, add, det, dot, neg, normalize, perp, scale, sub};
use crate::walls::Walls;

/// Um semi-plano de velocidades: as permitidas estão à ESQUERDA da recta que passa por `point` com a
/// direcção (unitária) `dir`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Line {
    pub point: V2,
    pub dir: V2,
}

/// O que um agente sabe de si próprio para construir os seus semi-planos.
#[derive(Clone, Copy, Debug)]
pub struct Me {
    pub pos: V2,
    pub vel: V2,
    pub radius: f64,
}

/// **O semi-plano contra UM vizinho.**
///
/// `share` é a parte do desvio que é DESTE agente: `0.5` contra outro agente que também desvia (o
/// *recíproco* do artigo) e `1.0` contra um corpo que não desvia — ele não vai fazer a metade dele.
pub fn agent_line(
    me: &Me,
    other_pos: V2,
    other_vel: V2,
    other_radius: f64,
    share: f64,
    tau: f64,
    dt: f64,
) -> Line {
    let rel_pos = sub(other_pos, me.pos);
    let rel_vel = sub(me.vel, other_vel);
    let dist_sq = abs_sq(rel_pos);
    let r = me.radius + other_radius;
    let r_sq = r * r;
    let (dir, u);
    if dist_sq > r_sq {
        // Ainda não se tocam: o cone truncado pelo horizonte `τ`.
        let inv_tau = 1.0 / tau;
        let w = sub(rel_vel, scale(rel_pos, inv_tau));
        let w_len_sq = abs_sq(w);
        let d1 = dot(w, rel_pos);
        if d1 < 0.0 && d1 * d1 > r_sq * w_len_sq {
            // A velocidade relativa projecta-se no CÍRCULO do corte.
            let w_len = w_len_sq.sqrt();
            let unit_w = scale(w, 1.0 / w_len);
            dir = [unit_w[1], -unit_w[0]];
            u = scale(unit_w, r * inv_tau - w_len);
        } else {
            // Projecta-se numa das duas PERNAS do cone.
            let leg = (dist_sq - r_sq).sqrt();
            dir = if det(rel_pos, w) > 0.0 {
                scale(
                    [
                        rel_pos[0] * leg - rel_pos[1] * r,
                        rel_pos[0] * r + rel_pos[1] * leg,
                    ],
                    1.0 / dist_sq,
                )
            } else {
                neg(scale(
                    [
                        rel_pos[0] * leg + rel_pos[1] * r,
                        -rel_pos[0] * r + rel_pos[1] * leg,
                    ],
                    1.0 / dist_sq,
                ))
            };
            u = sub(scale(dir, dot(rel_vel, dir)), rel_vel);
        }
    } else {
        // ⚠️ JÁ se tocam: o horizonte passa a ser o TIQUE — separar-se já, neste passo.
        let inv_dt = 1.0 / dt;
        let w = sub(rel_vel, scale(rel_pos, inv_dt));
        let w_len = abs_sq(w).sqrt();
        let unit_w = if w_len > 0.0 {
            scale(w, 1.0 / w_len)
        } else {
            // Mesma posição e mesma velocidade: nenhuma direcção é melhor que outra, e a regra tem de
            // dar UMA (a do eixo `+x`), igual nos três sistemas.
            [1.0, 0.0]
        };
        dir = [unit_w[1], -unit_w[0]];
        u = scale(unit_w, r * inv_dt - w_len);
    }
    Line {
        point: add(me.vel, scale(u, share)),
        dir,
    }
}

/// **Os semi-planos das PAREDES** — acrescentados a `out` pela ordem de `near` (as paredes que contam,
/// da mais perto para a mais longe).
///
/// Cada parede é a aresta `i → next(i)` de [`Walls`], com o espaço livre à DIREITA. `radius` é o do
/// agente CONTRA estas paredes: `0` quando elas já são a fronteira recuada pelo raio dele (a malha).
pub fn wall_lines(me: &Me, radius: f64, walls: &Walls, near: &[u32], tau_obst: f64, out: &mut Vec<Line>) {
    let inv_tau = 1.0 / tau_obst;
    let r_sq = radius * radius;
    for &i in near {
        let mut o1 = i as usize;
        let mut o2 = walls.next(o1);
        let rel1 = sub(walls.point(o1), me.pos);
        let rel2 = sub(walls.point(o2), me.pos);

        // Já coberta pelos semi-planos de paredes anteriores? (o cone desta está todo fora deles)
        let covered = out.iter().any(|l| {
            det(sub(scale(rel1, inv_tau), l.point), l.dir) - inv_tau * radius >= -EPS
                && det(sub(scale(rel2, inv_tau), l.point), l.dir) - inv_tau * radius >= -EPS
        });
        if covered {
            continue;
        }

        let d1 = abs_sq(rel1);
        let d2 = abs_sq(rel2);
        let ov = sub(walls.point(o2), walls.point(o1));
        let s = dot(neg(rel1), ov) / abs_sq(ov);
        let d_line = abs_sq(sub(neg(rel1), scale(ov, s)));

        // ── Já em contacto ────────────────────────────────────────────────────────────────────
        if s < 0.0 && d1 <= r_sq {
            if walls.convex(o1) {
                out.push(Line {
                    point: [0.0, 0.0],
                    dir: normalize([-rel1[1], rel1[0]]),
                });
            }
            continue;
        } else if s > 1.0 && d2 <= r_sq {
            // O vértice da direita só conta se a parede seguinte não o tratar.
            if walls.convex(o2) && det(rel2, walls.dir(o2)) >= 0.0 {
                out.push(Line {
                    point: [0.0, 0.0],
                    dir: normalize([-rel2[1], rel2[0]]),
                });
            }
            continue;
        } else if (0.0..1.0).contains(&s) && d_line <= r_sq {
            out.push(Line {
                point: [0.0, 0.0],
                dir: neg(walls.dir(o1)),
            });
            continue;
        }

        // ── Sem contacto: as duas pernas ───────────────────────────────────────────────────────
        let (mut left_leg, mut right_leg);
        if s < 0.0 && d_line <= r_sq {
            // Vista de esguelha: as duas pernas saem do vértice da ESQUERDA.
            if !walls.convex(o1) {
                continue;
            }
            o2 = o1;
            let leg = (d1 - r_sq).sqrt();
            left_leg = scale(
                [rel1[0] * leg - rel1[1] * radius, rel1[0] * radius + rel1[1] * leg],
                1.0 / d1,
            );
            right_leg = scale(
                [rel1[0] * leg + rel1[1] * radius, -rel1[0] * radius + rel1[1] * leg],
                1.0 / d1,
            );
        } else if s > 1.0 && d_line <= r_sq {
            // … ou as duas do da DIREITA.
            if !walls.convex(o2) {
                continue;
            }
            o1 = o2;
            let leg = (d2 - r_sq).sqrt();
            left_leg = scale(
                [rel2[0] * leg - rel2[1] * radius, rel2[0] * radius + rel2[1] * leg],
                1.0 / d2,
            );
            right_leg = scale(
                [rel2[0] * leg + rel2[1] * radius, -rel2[0] * radius + rel2[1] * leg],
                1.0 / d2,
            );
        } else {
            left_leg = if walls.convex(o1) {
                let leg = (d1 - r_sq).sqrt();
                scale(
                    [rel1[0] * leg - rel1[1] * radius, rel1[0] * radius + rel1[1] * leg],
                    1.0 / d1,
                )
            } else {
                // Vértice CÔNCAVO: a perna prolonga a própria parede.
                neg(walls.dir(o1))
            };
            right_leg = if walls.convex(o2) {
                let leg = (d2 - r_sq).sqrt();
                scale(
                    [rel2[0] * leg + rel2[1] * radius, -rel2[0] * radius + rel2[1] * leg],
                    1.0 / d2,
                )
            } else {
                walls.dir(o1)
            };
        }

        // Uma perna nunca aponta PARA DENTRO da parede vizinha: aí vale o corte dela, e a perna é
        // «estrangeira» (a projecção nela não dá semi-plano — a parede vizinha dá-o).
        let left_nb = walls.prev(o1);
        let mut left_foreign = false;
        let mut right_foreign = false;
        if walls.convex(o1) && det(left_leg, neg(walls.dir(left_nb))) >= 0.0 {
            left_leg = neg(walls.dir(left_nb));
            left_foreign = true;
        }
        if walls.convex(o2) && det(right_leg, walls.dir(o2)) <= 0.0 {
            right_leg = walls.dir(o2);
            right_foreign = true;
        }

        let left_cut = scale(sub(walls.point(o1), me.pos), inv_tau);
        let right_cut = scale(sub(walls.point(o2), me.pos), inv_tau);
        let cut_vec = sub(right_cut, left_cut);
        let same = o1 == o2;
        let t = if same {
            0.5
        } else {
            dot(sub(me.vel, left_cut), cut_vec) / abs_sq(cut_vec)
        };
        let t_left = dot(sub(me.vel, left_cut), left_leg);
        let t_right = dot(sub(me.vel, right_cut), right_leg);

        if (t < 0.0 && t_left < 0.0) || (same && t_left < 0.0 && t_right < 0.0) {
            let unit_w = normalize(sub(me.vel, left_cut));
            out.push(Line {
                dir: [unit_w[1], -unit_w[0]],
                point: add(left_cut, scale(unit_w, radius * inv_tau)),
            });
            continue;
        } else if t > 1.0 && t_right < 0.0 {
            let unit_w = normalize(sub(me.vel, right_cut));
            out.push(Line {
                dir: [unit_w[1], -unit_w[0]],
                point: add(right_cut, scale(unit_w, radius * inv_tau)),
            });
            continue;
        }

        let inf = f64::INFINITY;
        let dc = if t < 0.0 || t > 1.0 || same {
            inf
        } else {
            abs_sq(sub(me.vel, add(left_cut, scale(cut_vec, t))))
        };
        let dl = if t_left < 0.0 {
            inf
        } else {
            abs_sq(sub(me.vel, add(left_cut, scale(left_leg, t_left))))
        };
        let dr = if t_right < 0.0 {
            inf
        } else {
            abs_sq(sub(me.vel, add(right_cut, scale(right_leg, t_right))))
        };

        if dc <= dl && dc <= dr {
            let dir = neg(walls.dir(o1));
            out.push(Line {
                dir,
                point: add(left_cut, scale(perp(dir), radius * inv_tau)),
            });
        } else if dl <= dr {
            if left_foreign {
                continue;
            }
            out.push(Line {
                dir: left_leg,
                point: add(left_cut, scale(perp(left_leg), radius * inv_tau)),
            });
        } else {
            if right_foreign {
                continue;
            }
            let dir = neg(right_leg);
            out.push(Line {
                dir,
                point: add(right_cut, scale(perp(dir), radius * inv_tau)),
            });
        }
    }
}
