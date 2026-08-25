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
//! ⚠️ *(A tabela acima descreve o desenho; o que se prega são as **incógnitas livres do
//! sistema dos fechos** — as translações que sobraram e as imagens dos vértices
//! singulares. As dependentes escrevem-se por substituição, e são inteiras porque os
//! pivôs têm `|det| = 1`.)*
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
use crate::weld_flat::Var;
use crate::weld_solve::{WeldRelaxer, solve_welded};

/// ⭐⭐⭐ **ARREDONDA O MAPA SOLDADO PARA A GRADE INTEIRA.**
///
/// ⚠️ **Quais são as variáveis inteiras não é uma escolha:** são as **livres** do
/// sistema dos fechos. As outras translações são escritas por substituição, e caem em
/// inteiros porque os pivôs da eliminação têm `|det| = 1` — *a integralidade é uma
/// propriedade da eliminação, não uma verificação no fim.*
#[must_use]
#[allow(clippy::too_many_lines)]
pub fn round_welded(
    mesh: &Mesh,
    cut: &CutMesh,
    combed: &Combed,
    h: f32,
    opts: RoundOptions,
    _singular: &[u32],
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

    // ── ⭐ AS VARIÁVEIS INTEIRAS: as livres do sistema reduzido.
    //
    // ⚠️ **Os vértices singulares saem dos FECHOS, não de uma segunda contagem.** Um
    // fecho que roda *é* a assinatura de um vértice singular, e a medição mostrou que os
    // dois números batem exactamente (`8` para `8`, `12` para `12`). *Perguntar duas
    // vezes «quem é singular» é ter duas respostas que podem discordar.*
    let mut free: Vec<(usize, usize)> = Vec::new();
    for i in 0..r.sys.free().len() {
        if !opts.pin_singularities && matches!(r.sys.free()[i], Var::Class(_)) {
            continue;
        }
        free.push((i, 0));
        free.push((i, 1));
    }
    rep.singular_pinned = r
        .sys
        .free()
        .iter()
        .filter(|v| matches!(v, Var::Class(_)))
        .count();
    rep.cycle_seams = r
        .sys
        .free()
        .iter()
        .filter(|v| matches!(v, Var::Shift(_)))
        .count();

    // ── A ESCADA GULOSA: a de menor erro primeiro, e actualizar a seguir.
    while !free.is_empty() {
        let (best, _) = free
            .iter()
            .enumerate()
            .map(|(k, &(i, ax))| {
                let x = r.read_free(&map, i)[ax];
                (k, (x - x.round()).abs())
            })
            .fold((0usize, f32::INFINITY), |acc, (k, d)| {
                if d < acc.1 { (k, d) } else { acc }
            });
        let (i, ax) = free.swap_remove(best);
        let mut v = r.read_free(&map, i);
        let step = (v[ax] - v[ax].round()).abs();
        rep.worst_step = rep.worst_step.max(step);
        rep.sum_step += step;
        rep.pinned += 1;
        v[ax] = v[ax].round();
        r.write_free(&mut map, i, v);
        r.freeze_free(i, ax);

        // ── §5.1 degrau 1: Gauss–Seidel LOCAL sobre as CLASSES, semeado onde se mexeu.
        let seeds = r.classes_of_free(i);
        let (visits, converged) = drain(&r, &mut map, seeds, opts.local_tol, opts.local_cap);
        rep.visits += visits;
        if converged {
            rep.level1 += 1;
        } else {
            rep.level2 += 1;
            for _ in 0..opts.sweeps {
                if r.sweep(&mut map) < opts.local_tol {
                    break;
                }
            }
        }
        // ── ⭐⭐⭐ **E A PARTE CONTÍNUA ABSORVE**: as livres ainda por pregar relaxam-se
        // sobre o mapa que acabou de se mexer.
        for &(j, _) in &free {
            r.relax_free(&mut map, j);
        }
    }

    crate::solve::measure(&a, cut, combed, &map, h, &mut solve_rep);
    rep.seam_after = (solve_rep.seam_p50, solve_rep.seam_max);
    rep.seam = seam_residual(&w, &map);
    crate::weld::holonomy(&w, &map, &mut rep.weld);
    rep.shift_frac_max = map
        .shift
        .iter()
        .enumerate()
        .filter(|(s, _)| combed.jump.get(*s).copied().flatten().is_some())
        .map(|(_, t)| (t[0] - t[0].round()).abs().max((t[1] - t[1].round()).abs()))
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
