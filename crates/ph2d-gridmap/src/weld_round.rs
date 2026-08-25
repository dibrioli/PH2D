//! ⭐⭐⭐ **G5 SOLDADO** — o arredondamento inteiro sobre o sistema **reduzido**.
//!
//! # O que muda contra o [`crate::round`]
//!
//! A escada gulosa é a mesma (a de menor erro primeiro, actualizar a seguir, degrau
//! local antes do global). Mudam **as variáveis**, e é a soldadura que as muda:
//!
//! | | penalizado | ⭐ soldado |
//! |---|---|---|
//! | quem se relaxa | uma **cópia** de cada vez | uma **classe** de cada vez |
//! | quais translações são inteiras | as `E − V + componentes` que fecham ciclo, com as de árvore levadas a `0` pelo calibre | **todas** — ver abaixo |
//! | as cópias de um vértice singular | pregava-se uma e **transportava-se** o resto, com uma guarda que recusava saltos > ½ célula | ⭐ **nenhum transporte**: a classe é uma variável só, e uma rotação de um quarto de volta mais uma translação inteira leva inteiros a inteiros |
//!
//! # ⛔ Por que o CALIBRE não se aplica aqui, e não é um esquecimento
//!
//! O [`crate::gauge`] prova que somar uma constante ao `(u, v)` de **um patch** não
//! muda nada — logo as translações de árvore podem ir a `0` de graça. ⚠️ **Essa
//! simetria é do sistema por-patch**: ali cada patch tem variáveis próprias. Com as
//! costuras soldadas, uma cópia do outro lado **é** a mesma variável, e deslocar um
//! patch sozinho deixa de ser uma operação exprimível — a simetria que sobra é a
//! translação **global**.
//!
//! ⇒ ⛔ *aplicar `gauge::fix` a um mapa soldado escreveria por cima da derivação*, e as
//! duas metades de cada costura deixariam de concordar. As translações passam todas
//! pelo guloso; as que ainda forem direcções de calibre custam **zero** ao arredondar,
//! e é o próprio guloso — que escolhe sempre a de menor erro — quem as apanha primeiro.

use ph2d_mesh::Mesh;

use crate::comb::Combed;
use crate::cut::CutMesh;
use crate::round::{RoundOptions, RoundReport};
use crate::solve::{GridMap, SolveReport, assemble};
use crate::weld::{seam_residual, weld};
use crate::weld_solve::{WeldRelaxer, solve_welded};

/// Uma componente ainda por pregar.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Var {
    /// A componente `ax` da classe de um vértice singular.
    Class(u32, usize),
    /// A componente `ax` da translação de uma costura.
    Shift(u32, usize),
}

/// ⭐⭐⭐ **ARREDONDA O MAPA SOLDADO PARA A GRADE INTEIRA.**
///
/// ⚠️ **A ordem é a do método, não uma preferência:** as singularidades primeiro (a
/// imagem delas tem de ser um nó da grade, senão a malha rasga-se ali), as translações
/// depois. É a mesma ordem do [`crate::round`], pela mesma razão.
#[must_use]
#[allow(clippy::too_many_lines)]
pub fn round_welded(
    mesh: &Mesh,
    cut: &CutMesh,
    combed: &Combed,
    h: f32,
    opts: RoundOptions,
    singular: &[u32],
) -> (GridMap, RoundReport) {
    let (mut map, before) = solve_welded(mesh, cut, combed, h, opts.rounds);
    let (w, _) = weld(cut, combed);
    let mut rep = RoundReport {
        seam_before: (before.solve.seam_p50, before.solve.seam_max),
        weld: before.weld,
        ..RoundReport::default()
    };
    let mut solve_rep = SolveReport::default();
    let a = assemble(mesh, cut, combed, h, &mut solve_rep);
    let mut r = WeldRelaxer::new(&a, &w, cut, combed);

    // ── ⭐ AS VARIÁVEIS INTEIRAS. As classes singulares e todas as translações.
    //
    // ⚠️ **Uma classe por vértice singular, não uma cópia** — é a soldadura que o
    // garante: as cópias de um vértice singular são a MESMA variável.
    let wanted: std::collections::BTreeSet<u32> = singular.iter().copied().collect();
    let mut classes: Vec<u32> = Vec::new();
    for (p, origin) in cut.origin.iter().enumerate() {
        for (l, &g) in origin.iter().enumerate() {
            if wanted.contains(&g) {
                if let Some((c, _)) = w.of(p, l) {
                    #[allow(clippy::cast_possible_truncation)]
                    classes.push(c as u32);
                }
            }
        }
    }
    classes.sort_unstable();
    classes.dedup();
    rep.singular_pinned = classes.len();

    let mut free: Vec<Var> = Vec::new();
    if opts.pin_singularities {
        free.extend(classes.iter().flat_map(|&c| [Var::Class(c, 0), Var::Class(c, 1)]));
    }
    for s in 0..cut.seams.len() {
        // ⛔ Sem salto não há costura acoplada; DERIVADA não é variável — quem a
        // escreve é o fecho que a possui.
        if combed.jump.get(s).copied().flatten().is_none() || w.is_derived(s) {
            continue;
        }
        #[allow(clippy::cast_possible_truncation)]
        free.extend([Var::Shift(s as u32, 0), Var::Shift(s as u32, 1)]);
    }
    rep.cycle_seams = free
        .iter()
        .filter(|v| matches!(v, Var::Shift(_, 0)))
        .count();

    // ── A ESCADA GULOSA: a de menor erro primeiro, e actualizar a seguir.
    while !free.is_empty() {
        let read = |v: Var| -> f32 {
            match v {
                Var::Class(c, ax) => w.value(&map, c as usize)[ax],
                Var::Shift(s, ax) => map.shift[s as usize][ax],
            }
        };
        let (best, _) = free
            .iter()
            .enumerate()
            .map(|(i, &v)| {
                let x = read(v);
                (i, (x - x.round()).abs())
            })
            .fold((0usize, f32::INFINITY), |acc, (i, d)| {
                if d < acc.1 { (i, d) } else { acc }
            });
        let v = free.swap_remove(best);
        let x = read(v);
        let step = (x - x.round()).abs();
        rep.worst_step = rep.worst_step.max(step);
        rep.sum_step += step;
        rep.pinned += 1;

        let seeds: Vec<u32> = match v {
            Var::Class(c, ax) => {
                let mut y = w.value(&map, c as usize);
                y[ax] = x.round();
                // ⭐ Numa classe singular, escrever a imagem ESCREVE a translação da
                // costura do fecho — é a mesma variável.
                r.write_singular_at(&mut map, c as usize, y);
                r.freeze_class(c as usize, ax);
                w.settle(&mut map, crate::weld_solve::SETTLE_PASSES);
                r.neighbours(c as usize).to_vec()
            }
            Var::Shift(s, ax) => {
                map.shift[s as usize][ax] = x.round();
                r.freeze_shift(s as usize, ax);
                r.rederive(&mut map, s as usize);
                w.settle(&mut map, crate::weld_solve::SETTLE_PASSES);
                r.touched_by(s as usize).to_vec()
            }
        };

        // ── §5.1 degrau 1: Gauss–Seidel LOCAL sobre as CLASSES, semeado onde se mexeu.
        let (visits, converged) = drain(&r, &mut map, seeds, opts.local_tol, opts.local_cap);
        rep.visits += visits;
        if converged {
            rep.level1 += 1;
        } else {
            // ── degrau 2: varreduras globais, orçamentadas.
            rep.level2 += 1;
            for _ in 0..opts.sweeps {
                if r.sweep(&mut map, cut.seams.len()) < opts.local_tol {
                    break;
                }
            }
        }
        // ── ⭐⭐⭐ **E A PARTE CONTÍNUA ABSORVE**: as translações ainda livres
        // relaxam-se sobre o mapa que acabou de se mexer. É isto que faz «uma de cada
        // vez» ser diferente de «todas de uma vez».
        for s in 0..cut.seams.len() {
            r.relax_shift(&mut map, s);
        }
        w.settle(&mut map, crate::weld_solve::SETTLE_PASSES);
    }

    crate::solve::measure(&a, cut, combed, &map, h, &mut solve_rep);
    rep.seam_after = (solve_rep.seam_p50, solve_rep.seam_max);
    rep.seam = seam_residual(&w, &map);
    crate::weld::holonomy(&w, &map, &mut rep.weld);
    rep.shift_frac_max = map
        .shift
        .iter()
        .map(|t| (t[0] - t[0].round()).abs().max((t[1] - t[1].round()).abs()))
        .fold(0.0f32, f32::max);
    rep.solve = solve_rep;
    (map, rep)
}

/// O degrau 1 — a fila de classes que cresce pelos vizinhos de quem se mexeu.
fn drain(
    r: &WeldRelaxer,
    map: &mut GridMap,
    seeds: Vec<u32>,
    tol: f32,
    cap: usize,
) -> (usize, bool) {
    let mut queue: std::collections::VecDeque<u32> = seeds.iter().copied().collect();
    let mut queued: std::collections::BTreeSet<u32> = seeds.into_iter().collect();
    let mut visits = 0usize;
    while let Some(c) = queue.pop_front() {
        queued.remove(&c);
        visits += 1;
        if visits > cap {
            return (visits, false);
        }
        if r.relax_class(map, c as usize) <= tol {
            continue;
        }
        for &n in r.neighbours(c as usize) {
            if queued.insert(n) {
                queue.push_back(n);
            }
        }
    }
    (visits, true)
}

#[cfg(test)]
#[path = "weld_round_tests.rs"]
mod tests;
