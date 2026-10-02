//! **Os programas lineares** do ORCA (o artigo, §5.2–§5.3): a velocidade mais perto da preferida
//! dentro do disco da velocidade máxima e de todos os semi-planos — e, quando eles não têm ponto em
//! comum, a que viola MENOS o pior vizinho sem violar nenhuma parede.
//!
//! Um semi-plano [`Line`] permite as velocidades à ESQUERDA da recta orientada (`det(dir, point − v)
//! ≤ 0`). Os três programas são o incremental de Seidel sem aleatoriedade: a ordem é a dos semi-planos,
//! e a ordem dos semi-planos é total (ver [`crate::crowd`]).

use crate::lines::Line;
use crate::v2::{EPS, V2, abs_sq, add, det, dot, normalize, scale, sub};

/// A velocidade `v` viola o semi-plano.
#[inline]
pub fn violates(line: &Line, v: V2) -> bool {
    det(line.dir, sub(line.point, v)) > 0.0
}

/// **1D**: o melhor ponto sobre a recta `lines[no]`, dentro do disco e dos semi-planos `lines[..no]`.
/// `false` = vazio.
fn program1(lines: &[Line], no: usize, radius: f64, opt: V2, dir_opt: bool, out: &mut V2) -> bool {
    let l = lines[no];
    let d = dot(l.point, l.dir);
    let disc = d * d + radius * radius - abs_sq(l.point);
    if disc < 0.0 {
        // A recta não toca o disco da velocidade máxima.
        return false;
    }
    let s = disc.sqrt();
    let mut t_left = -d - s;
    let mut t_right = -d + s;
    for other in &lines[..no] {
        let den = det(l.dir, other.dir);
        let num = det(other.dir, sub(l.point, other.point));
        if den.abs() <= EPS {
            // Paralelas: ou a recta toda cabe, ou nenhuma parte dela.
            if num < 0.0 {
                return false;
            }
            continue;
        }
        let t = num / den;
        if den >= 0.0 {
            t_right = t_right.min(t);
        } else {
            t_left = t_left.max(t);
        }
        if t_left > t_right {
            return false;
        }
    }
    *out = if dir_opt {
        // Optimizar uma DIRECÇÃO: o extremo da recta que mais avança nela.
        if dot(opt, l.dir) > 0.0 {
            add(l.point, scale(l.dir, t_right))
        } else {
            add(l.point, scale(l.dir, t_left))
        }
    } else {
        let t = dot(l.dir, sub(opt, l.point)).clamp(t_left, t_right);
        add(l.point, scale(l.dir, t))
    };
    true
}

/// **2D**: o ponto mais perto de `opt` (ou o mais avançado na direcção `opt`, com `dir_opt`) dentro
/// do disco e de todos os semi-planos. Devolve o índice do primeiro semi-plano que tornou o conjunto
/// VAZIO, ou `lines.len()` se não houve nenhum — e `out` fica com a melhor resposta até ali.
pub fn program2(lines: &[Line], radius: f64, opt: V2, dir_opt: bool, out: &mut V2) -> usize {
    *out = if dir_opt {
        scale(opt, radius)
    } else if abs_sq(opt) > radius * radius {
        scale(normalize(opt), radius)
    } else {
        opt
    };
    for i in 0..lines.len() {
        if violates(&lines[i], *out) {
            let antes = *out;
            if !program1(lines, i, radius, opt, dir_opt, out) {
                *out = antes;
                return i;
            }
        }
    }
    lines.len()
}

/// **3D** (o conjunto é vazio a partir de `begin`): a velocidade que minimiza a MAIOR violação dos
/// semi-planos dos VIZINHOS (`lines[n_walls..]`), com os das PAREDES (`lines[..n_walls]`) sempre
/// respeitados — num aperto, quem cede é a distância a outro agente, nunca a parede.
pub fn program3(lines: &[Line], n_walls: usize, begin: usize, radius: f64, out: &mut V2) {
    let mut distance = 0.0;
    let mut proj: Vec<Line> = Vec::with_capacity(lines.len());
    for i in begin..lines.len() {
        let li = lines[i];
        if det(li.dir, sub(li.point, *out)) <= distance {
            continue;
        }
        // A velocidade viola o semi-plano `i` mais do que a pior violação até agora.
        proj.clear();
        proj.extend_from_slice(&lines[..n_walls]);
        for lj in &lines[n_walls..i] {
            let den = det(li.dir, lj.dir);
            let point = if den.abs() <= EPS {
                if dot(li.dir, lj.dir) > 0.0 {
                    // Mesma direcção: o `i` já é o mais apertado dos dois.
                    continue;
                }
                // Opostas: a bissectriz.
                scale(add(li.point, lj.point), 0.5)
            } else {
                add(
                    li.point,
                    scale(li.dir, det(lj.dir, sub(li.point, lj.point)) / den),
                )
            };
            proj.push(Line {
                point,
                dir: normalize(sub(lj.dir, li.dir)),
            });
        }
        let antes = *out;
        if program2(&proj, radius, [-li.dir[1], li.dir[0]], true, out) < proj.len() {
            // Em princípio impossível (a solução até ali é viável); só o arredondamento cá chega,
            // e então a resposta anterior é a melhor que há.
            *out = antes;
        }
        distance = det(li.dir, sub(li.point, *out));
    }
}

/// Por onde a velocidade saiu.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Regime {
    /// Nenhum semi-plano cortou a velocidade preferida.
    Free,
    /// O 2D achou a mais perto dentro de todos.
    Constrained,
    /// ⚠️ Não há velocidade que respeite todos: o 3D escolheu a que viola MENOS — a multidão está
    /// apertada (é aqui que o arredondamento de outro motor pode virar um ramo).
    Dense,
}

/// **A velocidade segura**: o 2D e, se ele esvaziar, o 3D a partir de onde esvaziou.
pub fn solve(lines: &[Line], n_walls: usize, max_speed: f64, pref: V2) -> (V2, Regime) {
    let mut v = [0.0, 0.0];
    let fail = program2(lines, max_speed, pref, false, &mut v);
    if fail < lines.len() {
        program3(lines, n_walls, fail, max_speed, &mut v);
        return (v, Regime::Dense);
    }
    let livre = lines.iter().all(|l| !violates(l, pref)) && abs_sq(pref) <= max_speed * max_speed;
    (v, if livre { Regime::Free } else { Regime::Constrained })
}
