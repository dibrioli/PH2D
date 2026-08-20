//! **O SOLVER** — o *mixed-integer* do MIQ, com o gauge da árvore geradora.
//!
//! ⚠️ **É aqui que mora a diferença de CLASSE** contra a família local. O
//! `ph2d-quadflow` suaviza: cada vértice absorve os vizinhos, e onde a
//! vizinhança se confunde nasce uma singularidade. Aqui a pergunta *"quantos
//! quartos de volta a cruz dá ao cruzar esta aresta?"* é uma **variável inteira**
//! do problema, e a resposta sai de minimizar a energia da malha **inteira**.
//!
//! # ⛔ A ALTERNÂNCIA INGÊNUA foi construída, MEDIDA e REJEITADA
//!
//! A primeira versão alternava *(resolve `θ` com `p` fixo)* e *(arredonda `p` com
//! `θ` fixo)*, partindo de `p = 0`. Ela **converge na primeira rodada e não faz
//! nada**: com `p = 0` o `θ` sai suave, todos os resíduos ficam abaixo de `π/4`, e
//! nenhum `p` chega a mudar. O campo resultante é a **curvatura quantizada** e
//! mais nada. Medido:
//!
//! | malha | singularidades | índices |
//! |---|---|---|
//! | esfera 24×36 | **2** | `+4`, `+4` |
//! | esfera 48×64 | **2** | `+4`, `+4` |
//! | cubo subdividido | **2** | `+4`, `+4` |
//!
//! ⚠️ **A soma passava no gate topológico** (`Σ = 8`, correto) **e o campo era
//! péssimo**: uma singularidade de índice `+4` é um ponto onde a cruz dá uma
//! volta inteira, e não há grade de quads que a contorne. *A invariante prova que
//! o campo FECHA, não que ele presta* — e foi preciso a segunda régua (a
//! CONTAGEM) para ver isso. A alternância fica em [`solve_alternating`], como
//! controle.
//!
//! # O que o MIQ faz, e por que precisa do gauge
//!
//! Os `p` têm **liberdade de calibre**: somar `k_f · π/2` a cada `θ_f` muda os
//! `p` sem mudar o campo. Fixar `p = 0` numa **árvore geradora** do grafo dual
//! consome exatamente essa liberdade — sobram `E − F + 1` inteiros, um por ciclo
//! independente, e **é neles que a topologia mora**. Sem o gauge, a relaxação
//! contínua tem energia zero (basta `p_e = −r_e/(π/2)`) e não diz nada.
//!
//! Depois: **rounding guloso**. Resolve a relaxação, congela o inteiro mais
//! próximo de um inteiro, re-resolve. É o passo caro e é o que compra a
//! qualidade.

use crate::{CrossField, Dual, QUARTER, wrap};

/// **Quantas iterações de CG por resolução.**
///
/// ⚠️ **Teto de RECURSO com piso de qualidade atrás:** o CG converge em O(√κ), e
/// `κ` cresce com o diâmetro do grafo. A saída também é por **resíduo**, então em
/// malha pequena ele sai muito antes. Medido no corpus: 90 a 340 iterações.
const CG_ITERATIONS: usize = 600;

/// O resíduo relativo em que o CG desiste.
const CG_TOLERANCE: f32 = 1.0e-7;

/// **Que fração dos inteiros ainda livres é congelada por rodada.**
///
/// ⚠️ **O MIQ congela UM de cada vez**, e isso são `E − F + 1` resoluções — 851
/// numa esfera 24×36, 2 500 numa malha do corpus. Congelar **um oitavo** dos mais
/// confiantes por rodada leva o número de resoluções a `log_{8/7}(C)` ≈ **50**, e
/// só o faz sobre os que já estão mais perto de um inteiro.
///
/// ⚠️ **É uma DIVERGÊNCIA DECLARADA da referência, e ela tem preço por medir:**
/// congelar em lote pode fixar um inteiro que a re-resolução teria mudado.
/// ⛔ Não a mexa sem a tabela de qualidade × relógio ao lado.
const BATCH_FRACTION: usize = 8;

/// O que o solver fez.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SolveReport {
    /// Quantas resoluções do sistema linear correram.
    pub solves: usize,
    /// Quantos inteiros o gauge da árvore deixou livres.
    pub free_integers: usize,
    /// Quantos saltos de período saíram diferentes de zero.
    pub nonzero_periods: usize,
}

/// **RESOLVE o campo cruzado** — MIQ com gauge de árvore e rounding guloso.
#[must_use]
pub fn solve_miq(dual: &Dual) -> (CrossField, SolveReport) {
    let n = dual.frames().len();
    let m = dual.edges().len();
    let mut report = SolveReport {
        solves: 0,
        free_integers: 0,
        nonzero_periods: 0,
    };
    if n == 0 {
        return (
            CrossField {
                theta: Vec::new(),
                period: Vec::new(),
            },
            report,
        );
    }

    // ── O GAUGE: `p = 0` em toda aresta de uma árvore geradora ───────────────
    let tree = spanning_tree(dual);
    let mut fixed: Vec<Option<i32>> = vec![None; m];
    for &e in &tree {
        fixed[e as usize] = Some(0);
    }
    let mut free: Vec<usize> = (0..m).filter(|e| fixed[*e].is_none()).collect();
    report.free_integers = free.len();

    let mut theta = vec![0.0f32; n];
    while !free.is_empty() {
        let q = solve_relaxation(dual, &fixed, &free, &mut theta);
        report.solves += 1;

        // ⚠️ **A CONFIANÇA é a distância a um inteiro**, e o desempate é o índice
        // da aresta — sem ele a ordem de congelamento dependeria da ordem de
        // ponto flutuante e a saída deixaria de ser reprodutível (HR-5).
        let mut order: Vec<(usize, f32)> = free
            .iter()
            .enumerate()
            .map(|(i, &_e)| (i, (q[i] - q[i].round()).abs()))
            .collect();
        order.sort_by(|a, b| a.1.total_cmp(&b.1).then(a.0.cmp(&b.0)));

        let take = (free.len() / BATCH_FRACTION).max(1);
        let mut freeze: Vec<usize> = order.iter().take(take).map(|(i, _)| *i).collect();
        freeze.sort_unstable();
        for &i in freeze.iter().rev() {
            let e = free[i];
            fixed[e] = Some(q[i].round() as i32);
            free.remove(i);
        }
    }

    // A resolução final, com todos os inteiros congelados.
    solve_relaxation(dual, &fixed, &[], &mut theta);
    report.solves += 1;

    let period: Vec<i32> = fixed.iter().map(|p| p.unwrap_or(0)).collect();
    report.nonzero_periods = period.iter().filter(|p| **p != 0).count();
    (CrossField { theta, period }, report)
}

/// **⛔ A ALTERNÂNCIA INGÊNUA — construída, MEDIDA e REJEITADA.** Ver o doc do
/// módulo.
///
/// Fica como **controle**: é o que a família local faria se lhe dessem inteiros,
/// e ela mostra que o gauge da árvore não é enfeite — é o que torna o problema
/// não-trivial.
#[must_use]
pub fn solve_alternating(dual: &Dual, max_rounds: usize) -> (CrossField, usize) {
    let n = dual.frames().len();
    let mut theta = vec![0.0f32; n];
    let mut period = vec![0i32; dual.edges().len()];
    let mut rounds = 0usize;
    for round in 1..=max_rounds {
        rounds = round;
        let fixed: Vec<Option<i32>> = period.iter().map(|p| Some(*p)).collect();
        solve_relaxation(dual, &fixed, &[], &mut theta);
        let mut changed = 0usize;
        for (e, de) in dual.edges().iter().enumerate() {
            let r = theta[de.f as usize] - theta[de.g as usize] + de.kappa;
            let want = -(r / QUARTER).round() as i32;
            if want != period[e] {
                period[e] = want;
                changed += 1;
            }
        }
        if changed == 0 {
            break;
        }
    }
    (CrossField { theta, period }, rounds)
}

/// **A ÁRVORE GERADORA do grafo dual** — o gauge.
///
/// ⚠️ **BFS a partir da face 0, com vizinhos em ordem de índice.** Qualquer
/// árvore serve para consumir a liberdade de calibre; o que **não** serve é uma
/// árvore que mude entre corridas, porque ela decide quais inteiros existem.
fn spanning_tree(dual: &Dual) -> Vec<u32> {
    let n = dual.frames().len();
    let mut seen = vec![false; n];
    let mut tree: Vec<u32> = Vec::new();
    let mut queue = std::collections::VecDeque::new();
    // ⚠️ Todas as componentes, não só a do 0: uma malha com duas peças tem duas
    // árvores, e deixar a segunda sem gauge deixaria o sistema singular ali.
    for start in 0..n {
        if seen[start] {
            continue;
        }
        seen[start] = true;
        queue.push_back(start);
        while let Some(f) = queue.pop_front() {
            for &e in dual.incident(f) {
                let de = &dual.edges()[e as usize];
                let other = if de.f as usize == f {
                    de.g as usize
                } else {
                    de.f as usize
                };
                if !seen[other] {
                    seen[other] = true;
                    tree.push(e);
                    queue.push_back(other);
                }
            }
        }
    }
    tree
}

/// **A RELAXAÇÃO CONTÍNUA** — resolve `θ` (e os `q` livres) por CG.
///
/// Devolve os valores contínuos dos inteiros ainda livres, na ordem de `free`.
fn solve_relaxation(
    dual: &Dual,
    fixed: &[Option<i32>],
    free: &[usize],
    theta: &mut [f32],
) -> Vec<f32> {
    let n = theta.len();
    let c = free.len();
    let dim = n + c;
    // aresta -> slot livre.
    let mut slot: Vec<Option<usize>> = vec![None; dual.edges().len()];
    for (i, &e) in free.iter().enumerate() {
        slot[e] = Some(i);
    }

    // b = −scatter( w · constante )
    let mut b = vec![0.0f32; dim];
    for (e, de) in dual.edges().iter().enumerate() {
        let konst = de.kappa + QUARTER * fixed[e].unwrap_or(0) as f32;
        let wc = de.weight * konst;
        b[de.f as usize] -= wc;
        b[de.g as usize] += wc;
        if let Some(i) = slot[e] {
            b[n + i] -= wc * QUARTER;
        }
    }

    let pinned = 0usize;
    let mut x = vec![0.0f32; dim];
    x[..n].copy_from_slice(theta);
    x[pinned] = 0.0;

    let mut ax = vec![0.0f32; dim];
    apply(dual, &slot, n, &x, &mut ax, pinned);
    let mut r: Vec<f32> = (0..dim).map(|i| b[i] - ax[i]).collect();
    r[pinned] = 0.0;
    let mut p = r.clone();
    let mut rr = ddot(&r, &r);
    let r0 = rr.sqrt().max(1.0);

    let mut ap = vec![0.0f32; dim];
    for _ in 0..CG_ITERATIONS {
        if rr.sqrt() <= CG_TOLERANCE * r0 {
            break;
        }
        apply(dual, &slot, n, &p, &mut ap, pinned);
        let denom = ddot(&p, &ap);
        if denom.abs() <= 1.0e-30 {
            break;
        }
        let alpha = rr / denom;
        for i in 0..dim {
            x[i] += alpha * p[i];
            r[i] -= alpha * ap[i];
        }
        let rr_next = ddot(&r, &r);
        let beta = rr_next / rr;
        for i in 0..dim {
            p[i] = r[i] + beta * p[i];
        }
        rr = rr_next;
    }

    theta.copy_from_slice(&x[..n]);
    x[n..].to_vec()
}

/// `A x`, com a linha e a coluna do gauge de `θ` zeradas.
fn apply(dual: &Dual, slot: &[Option<usize>], n: usize, x: &[f32], out: &mut [f32], pinned: usize) {
    out.fill(0.0);
    for (e, de) in dual.edges().iter().enumerate() {
        let (f, g) = (de.f as usize, de.g as usize);
        let tf = if f == pinned { 0.0 } else { x[f] };
        let tg = if g == pinned { 0.0 } else { x[g] };
        let q = slot[e].map_or(0.0, |i| x[n + i]);
        let r = de.weight * (tf - tg + QUARTER * q);
        if f != pinned {
            out[f] += r;
        }
        if g != pinned {
            out[g] -= r;
        }
        if let Some(i) = slot[e] {
            out[n + i] += r * QUARTER;
        }
    }
    out[pinned] = x[pinned];
}

/// ⚠️ Acumulador em `f64`: a soma de dezenas de milhares de termos em `f32`
/// perde bits suficientes para o CG estagnar longe da solução, e o sintoma é um
/// campo que "quase" converge.
fn ddot(a: &[f32], b: &[f32]) -> f32 {
    let mut s = 0.0f64;
    for i in 0..a.len() {
        s += f64::from(a[i]) * f64::from(b[i]);
    }
    s as f32
}

/// **A ENERGIA do campo** — a régua da convergência.
#[must_use]
pub fn energy(dual: &Dual, field: &CrossField) -> f64 {
    let mut sum = 0.0f64;
    for (e, de) in dual.edges().iter().enumerate() {
        let r = wrap(
            field.theta(de.f as usize) - field.theta(de.g as usize)
                + de.kappa
                + QUARTER * field.period(e) as f32,
        );
        sum += f64::from(de.weight) * f64::from(r) * f64::from(r);
    }
    sum
}

/// Os ciclos independentes do grafo dual — `E − F + componentes`.
#[must_use]
pub fn cycle_count(dual: &Dual) -> usize {
    let tree = spanning_tree(dual);
    dual.edges().len() - tree.len()
}
