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

/// ⭐ **O INTERRUPTOR, numa porta só:** `PH2D_GRIDMAP_WELD=0` volta ao G3 penalizado.
///
/// ⚠️ **Ele vive aqui e não em cada chamador** porque houve dois — o instrumento e o
/// produto — e eles nasceram a ler a MESMA variável com sentidos **opostos** (um
/// tratava-a como opt-in, o outro como opt-out). *Uma pergunta com duas respostas é a
/// que envelhece.*
#[must_use]
pub fn welded_enabled() -> bool {
    std::env::var("PH2D_GRIDMAP_WELD").ok().as_deref() != Some("0")
}

/// ⭐⭐⭐ **ARREDONDA O MAPA SOLDADO PARA A GRADE INTEIRA.**
///
/// ⚠️ **Quais são as variáveis inteiras não é uma escolha:** são as **livres** do
/// sistema dos fechos. As outras translações são escritas por substituição, e caem em
/// inteiros porque os pivôs da eliminação têm `|det| = 1` — *a integralidade é uma
/// propriedade da eliminação, não uma verificação no fim.*
#[must_use]
#[allow(clippy::too_many_lines)]
/// O que a escada gulosa prega — uma livre do sistema reduzido, ou uma **classe solta**.
///
/// ⚠️ **Os dois casos partilham a escada de propósito**: a ordem «menor erro primeiro» só
/// significa alguma coisa se todos os candidatos competirem na mesma fila.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Target {
    Free(usize),
    Class(usize),
}

fn read(r: &WeldRelaxer<'_>, map: &GridMap, t: Target) -> [f32; 2] {
    match t {
        Target::Free(i) => r.read_free(map, i),
        Target::Class(c) => r.read_class(map, c),
    }
}

pub fn round_welded(
    mesh: &Mesh,
    cut: &CutMesh,
    combed: &Combed,
    h: f32,
    opts: RoundOptions,
    singular: &[u32],
) -> (GridMap, RoundReport) {
    let (mut map, before) = solve_welded(mesh, cut, combed, h, opts.welded_rounds);
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
    // fecho que roda *é* a assinatura de um vértice singular.
    //
    // ⛔⛔ **CORRECÇÃO (2026-08-25): a frase que estava aqui dizia que «os dois números
    // batem exactamente (`8` para `8`, `12` para `12`)», e ela foi medida nas DUAS únicas
    // peças do corpus em que pode bater.** Na peça do artista o campo planta `25`
    // singularidades e este passo prega `17`: um vértice singular que o corte **não
    // duplicou** não tem cópias, logo não tem fecho, logo **nunca é pregado num inteiro**.
    // Medido no corpus (`chain_info`, coluna «SINGULARES contra o CORTE»):
    //
    // | peça | duplicados | ⛔ uma cópia só | pregados | órfãs | bordo |
    // |---|---|---|---|---|---|
    // | enrugada | 8 | 0 | 8/8 | 0 | 0 |
    // | com orelha | 8 | 0 | 8/8 | 0 | 0 |
    // | com cristas | 12 | 0 | 12/12 | 0 | 0 |
    // | com gancho | 11 | 4 | 12/15 | 0 | 0 |
    // | ⛔ **do artista** | 19 | **6** | **17/25** | **11** | **14** |
    //
    // ⭐⭐⭐ **Toda peça com zero na coluna do meio tem zero furos.** E a cadeia causal
    // está medida ponta a ponta: singular não pregado ⇒ a imagem dele fica fraccionária ⇒
    // as transições à volta dele ficam fraccionárias (resíduo `0,47` de célula na peça do
    // artista contra `1e-7` nas limpas) ⇒ o extractor **arredonda-as para células
    // inteiras** ⇒ o traçado da isolinha cai `3,000` células ao lado num triângulo de
    // `1,440` ⇒ órfã ⇒ saída pendente ⇒ célula abandonada ⇒ **aresta de bordo, na ponta**.
    //
    // ⚠️ *Uma afirmação de que dois números batem, verificada só onde eles batem, é a
    // forma mais cara de nota errada: ela fecha a pergunta.*
    let mut free: Vec<(Target, usize)> = Vec::new();
    for i in 0..r.sys.free().len() {
        if !opts.pin_singularities && matches!(r.sys.free()[i], Var::Class(_)) {
            continue;
        }
        free.push((Target::Free(i), 0));
        free.push((Target::Free(i), 1));
    }

    // ⭐⭐⭐ **OS SINGULARES SOLTOS** — os que o corte NÃO duplicou, e que por isso não
    // têm fecho nenhum a representá-los. Ver a tabela acima: são eles que deixam a imagem
    // fraccionária e, no fim da cadeia, o furo na ponta.
    //
    // ⚠️ **Entram na MESMA escada gulosa**, e não num passe à parte: a escada escolhe
    // sempre o menor erro primeiro, e um segundo passe depois dela pregaria estes sobre um
    // mapa que as costuras já moveram. *Duas escadas sobre a mesma energia é o defeito que
    // a obra A mediu nos dois subsistemas.*
    let mut loose: Vec<usize> = Vec::new();
    if opts.pin_singularities && opts.pin_lone_singularities {
        let wanted: std::collections::BTreeSet<u32> = singular.iter().copied().collect();
        let mut seen: std::collections::BTreeSet<usize> = std::collections::BTreeSet::new();
        for (p, origin) in cut.origin.iter().enumerate() {
            for (l, &g) in origin.iter().enumerate() {
                if !wanted.contains(&g) {
                    continue;
                }
                let Some((c, _)) = w.of(p, l) else { continue };
                // ⚠️ **UMA CÓPIA SÓ, e a restrição não é conservadorismo: é o que foi
                // MEDIDO.** A coluna «SINGULARES contra o CORTE» conta `6` vértices com
                // uma cópia na peça do artista; sem esta linha o critério `class_is_loose`
                // apanhava **19** — classes que o sistema dos fechos não escreve mas que
                // têm cópias e, por isso, já têm quem lhes imponha coerência.
                // ⛔ Medido: pregar as 19 curava o defeito alvo (órfãs «sem saída» de `8`
                // para `2`) e fazia explodir o outro (`sem parceira` de `3` para `18`).
                // *Uma cura mais larga que a medição que a motivou é outra experiência.*
                if w.members_pub(c).count() == 1 && r.class_is_loose(c) && seen.insert(c) {
                    loose.push(c);
                }
            }
        }
        for &c in &loose {
            free.push((Target::Class(c), 0));
            free.push((Target::Class(c), 1));
        }
    }
    rep.singular_loose_pinned = loose.len();
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
            .map(|(k, &(tgt, ax))| {
                let x = read(&r, &map, tgt)[ax];
                (k, (x - x.round()).abs())
            })
            .fold((0usize, f32::INFINITY), |acc, (k, d)| {
                if d < acc.1 { (k, d) } else { acc }
            });
        let (tgt, ax) = free.swap_remove(best);
        let mut v = read(&r, &map, tgt);
        let step = (v[ax] - v[ax].round()).abs();
        rep.worst_step = rep.worst_step.max(step);
        rep.sum_step += step;
        rep.pinned += 1;
        v[ax] = v[ax].round();
        match tgt {
            Target::Free(i) => {
                r.write_free(&mut map, i, v);
                r.freeze_free(i, ax);
            }
            Target::Class(c) => {
                r.write_class(&mut map, c, v);
                r.freeze_class(c, ax);
            }
        }

        // ── §5.1 degrau 1: Gauss–Seidel LOCAL sobre as CLASSES, semeado onde se mexeu.
        let seeds = match tgt {
            Target::Free(i) => r.classes_of_free(i),
            #[allow(clippy::cast_possible_truncation)]
            Target::Class(c) => vec![c as u32],
        };
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
            match j {
                Target::Free(i) => {
                    r.relax_free(&mut map, i);
                }
                Target::Class(c) => {
                    r.relax_class(&mut map, c);
                }
            }
        }
    }

    // ── ⭐⭐⭐ **O FECHO: reconstruir as dependentes das livres, e ENCAIXAR nos
    // inteiros que elas matematicamente já são.**
    //
    // ⚠️ **As duas metades são de representação, não de modelo.** (1) A propagação
    // incremental (`bump`) é exacta em ℝ e acumula erro de `f32` ao longo de dezenas de
    // pregos ⇒ reconstrói-se a expressão inteira uma vez. (2) Uma dependente sai de uma
    // substituição com pivôs de `|det| = 1` sobre livres inteiras ⇒ ela **é** inteira, e
    // o que se lê (`1,3e-5` na esfera lisa) é o resíduo de a ter somado em `f32`.
    //
    // ⛔ **A DISTÂNCIA MEDE-SE ANTES DE ENCAIXAR**, e é ela que fica no relatório: se
    // algum pivô tivesse determinante `2`, a dependente cairia num **meio-inteiro** e
    // encaixá-la calaria o defeito. *Um encaixe que não mede primeiro é um tapete.*
    r.sys.apply(&w, &mut map);
    rep.shift_frac_max = map
        .shift
        .iter()
        .enumerate()
        .filter(|(s, _)| combed.jump.get(*s).copied().flatten().is_some())
        .map(|(_, t)| (t[0] - t[0].round()).abs().max((t[1] - t[1].round()).abs()))
        .fold(0.0f32, f32::max);
    for (s, t) in map.shift.iter_mut().enumerate() {
        if combed.jump.get(s).copied().flatten().is_some() {
            *t = [t[0].round(), t[1].round()];
        }
    }
    for c in 0..w.classes() {
        w.derive(&mut map, c);
    }

    crate::solve::measure(&a, cut, combed, &map, h, &mut solve_rep);
    rep.seam_after = (solve_rep.seam_p50, solve_rep.seam_max);
    rep.seam = seam_residual(&w, &map);
    crate::weld::holonomy(&w, &map, &mut rep.weld);
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
