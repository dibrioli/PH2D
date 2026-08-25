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
///
/// ⭐ **A tabela chegou em 2026-08-21, e o preço era grande** — ver
/// [`Rounding`] e o `PLAN.md` §4-octies.
const BATCH_FRACTION: usize = 8;

/// **A POLÍTICA DE ARREDONDAMENTO** — quantos inteiros congelar por rodada, e
/// quão confiante um deles tem de ser.
///
/// ⚠️ **Ela existe porque a lei em lote tinha um preço que ninguém tinha medido.**
/// Medido: a mesma esfera remalhada isotropicamente sai com **8** singularidades a
/// 2 608 vértices e **194** a 10 251 — enquanto a esfera ESTRUTURADA de 13 682 sai
/// com **7**. Não é resolução (a estruturada é maior), não é convergência do CG (a
/// estruturada tem o pior resíduo da tabela) e não é a finura da referência
/// (7× mais fina move 194 → 168). *É o lote: ele congela `livres/8` de uma vez, e
/// `livres` cresce com a malha.*
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Rounding {
    /// Congela no máximo `livres / fraction` por rodada. `1` congela todos os que
    /// passarem no [`Self::max_deviation`]; um número grande aproxima-se do
    /// *um-de-cada-vez* da referência.
    pub fraction: usize,
    /// ⭐ **A distância máxima a um inteiro que ainda conta como confiante.**
    /// `0,5` aceita qualquer um (é a lei antiga, que só ordenava); um valor
    /// pequeno recusa os genuinamente fracionários e deixa-os para a rodada
    /// seguinte, **depois** de a re-resolução os ter movido.
    ///
    /// ⚠️ **Um por rodada é sempre congelado**, mesmo que nenhum passe — senão o
    /// laço não termina.
    pub max_deviation: f32,
}

impl Default for Rounding {
    /// A lei em vigor: um oitavo por rodada, sem exigência de confiança.
    fn default() -> Self {
        Self {
            fraction: BATCH_FRACTION,
            max_deviation: 0.5,
        }
    }
}

/// O que o solver fez.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SolveReport {
    /// Quantas resoluções do sistema linear correram.
    pub solves: usize,
    /// Quantos inteiros o gauge da árvore deixou livres.
    pub free_integers: usize,
    /// Quantos saltos de período saíram diferentes de zero.
    pub nonzero_periods: usize,
    /// ⭐ **Quantas resoluções gastaram as [`CG_ITERATIONS`] sem atingir a
    /// tolerância** — ou seja, quantas devolveram uma resposta que o método ainda
    /// estava a construir.
    ///
    /// ⚠️ **Este campo existe porque a sua ausência custou um diagnóstico
    /// inteiro** (2026-08-21): a mesma esfera remalhada mais fina passava de **8**
    /// para **194** singularidades, e a soma dos índices continuava `8` — porque
    /// ela é forçada por Poincaré–Hopf e não pode denunciar nada. *Um teto de
    /// recurso sem instrumento é indistinguível de um algoritmo errado.*
    pub cg_capped: usize,
    /// O pior resíduo relativo com que uma resolução saiu — `≤ 1e-7` quando todas
    /// convergiram.
    pub cg_worst_residual: f32,
    /// ⭐ **Quantas RE-CENTRAGENS do arredondamento inteiro melhoraram o
    /// objectivo** — ver [`Continuation`]. `0` quando a primeira passagem já era a
    /// melhor (é sempre o caso com `align = 0`, e o gate exige-o).
    pub recentres: usize,
}

impl SolveReport {
    /// Regista o resíduo com que uma resolução saiu.
    fn note(&mut self, residual: f32) {
        self.cg_worst_residual = self.cg_worst_residual.max(residual);
        if residual > CG_TOLERANCE {
            self.cg_capped += 1;
        }
    }
}

/// **RESOLVE o campo cruzado** — MIQ com gauge de árvore e rounding guloso.
#[must_use]
pub fn solve_miq(dual: &Dual) -> (CrossField, SolveReport) {
    solve_miq_with(dual, Rounding::default())
}

/// **A mesma coisa, com a política de arredondamento na mão** — ver [`Rounding`].
///
/// ⚠️ **Ponto de extensão append-only** (`CLAUDE.md` §0.2): o [`solve_miq`] passa
/// a ser esta função com o `default`, e nada que o chame vê diferença.
#[must_use]
pub fn solve_miq_with(dual: &Dual, policy: Rounding) -> (CrossField, SolveReport) {
    crate::continuation::solve_miq_aligned(dual, policy, crate::ALIGN_WEIGHT)
}

/// **UMA PASSAGEM COMPLETA do arredondamento guloso**, a partir do `theta` dado.
///
/// ⚠️ **O `theta` entra como SEMENTE e sai como resultado**, e é isso que torna a
/// [`Continuation`] possível: a passagem seguinte herda o campo da anterior, então
/// o representante 4-RoSy do [`solve_relaxation`] já nasce certo.
pub(super) fn round_once(
    dual: &Dual,
    policy: Rounding,
    align: f32,
    theta: &mut [f32],
    report: &mut SolveReport,
) -> Vec<i32> {
    let m = dual.edges().len();
    // ── O GAUGE: `p = 0` em toda aresta de uma árvore geradora ───────────────
    let tree = spanning_tree(dual);
    let mut fixed: Vec<Option<i32>> = vec![None; m];
    for &e in &tree {
        fixed[e as usize] = Some(0);
    }
    let mut free: Vec<usize> = (0..m).filter(|e| fixed[*e].is_none()).collect();
    report.free_integers = free.len();

    while !free.is_empty() {
        let (q, residual) = solve_relaxation(dual, &fixed, &free, theta, align);
        report.solves += 1;
        report.note(residual);

        // ⚠️ **A CONFIANÇA é a distância a um inteiro**, e o desempate é o índice
        // da aresta — sem ele a ordem de congelamento dependeria da ordem de
        // ponto flutuante e a saída deixaria de ser reprodutível (HR-5).
        let mut order: Vec<(usize, f32)> = free
            .iter()
            .enumerate()
            .map(|(i, &_e)| (i, (q[i] - q[i].round()).abs()))
            .collect();
        order.sort_by(|a, b| a.1.total_cmp(&b.1).then(a.0.cmp(&b.0)));

        // ⚠️ **As duas portas são independentes, e o `.max(1)` é a que garante
        // TERMINAÇÃO**: se nenhum candidato passar na confiança, congela-se
        // mesmo assim o mais confiante de todos — senão o laço não anda.
        let take = (free.len() / policy.fraction.max(1)).max(1);
        let confident = order
            .iter()
            .take(take)
            .take_while(|(_, d)| *d <= policy.max_deviation)
            .count()
            .max(1);
        let mut freeze: Vec<usize> = order.iter().take(confident).map(|(i, _)| *i).collect();
        freeze.sort_unstable();
        for &i in freeze.iter().rev() {
            let e = free[i];
            fixed[e] = Some(q[i].round() as i32);
            free.remove(i);
        }
    }

    // A resolução final, com todos os inteiros congelados.
    let (_, residual) = solve_relaxation(dual, &fixed, &[], theta, align);
    report.solves += 1;
    report.note(residual);
    fixed.iter().map(|p| p.unwrap_or(0)).collect()
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
        // ⚠️ **Sem alinhamento, de propósito.** Esta função é o CONTROLE medido e
        // rejeitado do §doc do módulo (ela converge na primeira ronda e não faz
        // nada); acrescentar-lhe um termo novo mudaria o que ela é o controle de.
        let _ = solve_relaxation(dual, &fixed, &[], &mut theta, 0.0);
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

/// ⭐⭐⭐ **QUEM ANCORA O `θ` DE CADA COMPONENTE — e NUNCA duas vezes a mesma.**
///
/// Com os inteiros congelados, o sistema em `θ` é um laplaciano: a solução fica
/// determinada **a menos de uma constante por componente**, e é por isso que uma face tem
/// de ser presa. ⭐ **Uma face restringida JÁ é essa âncora.**
///
/// # ⛔⛔ Por que a face `0` deixa de ser presa quando há restrição
///
/// A `θ = 0` da face `0` é uma escolha arbitrária de **calibre**, e ela é legítima enquanto
/// for a única: somar uma constante a todo `θ` não muda a energia, então uma referência tem
/// de ser escrita. ⚠️ **Ao lado de uma restrição real ela passa a ser uma segunda equação,
/// falsa** — o ângulo entre a face `0` e a face restringida fica fixo num valor que ninguém
/// pediu, e a suavidade deixa de poder decidi-lo.
///
/// # ⚠️ E o que ela NÃO curou — a atribuição honesta
///
/// ⛔ **Esta cura sozinha vale ~zero, e foi medida:** ela levou o controlo de `111` para
/// `109` singularidades onde a resposta certa era `25`. Quem curou foi a
/// [`spanning_tree`] — e a 1.ª redacção deste bloco atribuía-lhe o mérito. *Duas correcções
/// no mesmo turno leem-se como uma; só a medição separada diz qual pagou.*
///
/// ⇒ Ela fica por ser **certa**, não por ter movido um número: uma equação falsa que hoje
/// custa uma face em 4 654 continua a ser uma equação falsa. ⚠️ **Nenhuma mutação a mata
/// pelo resultado** (`gate_feature_sparse` fica verde sem ela); quem a defende é o gate
/// estrutural [`crate::Dual::constrain`] ⇒ `the_gauge_is_written_once_per_component`.
///
/// ⚠️ **Com nenhuma restrição a resposta é `[face 0]`, exactamente como antes** — ⛔ e não
/// «uma por componente», que seria mais correcto **e** mudaria a saída de toda malha de
/// duas peças. *Corrigir de passagem um defeito que ninguém mediu é mudar o produto sem a
/// tabela ao lado.*
pub(crate) fn gauge_seeds(dual: &Dual) -> Vec<bool> {
    let n = dual.frames().len();
    let mut seeds = vec![false; n];
    if dual.constrained_count() == 0 {
        if n > 0 {
            seeds[0] = true;
        }
        return seeds;
    }
    let mut comp = vec![usize::MAX; n];
    let mut queue = std::collections::VecDeque::new();
    let mut roots: Vec<(usize, bool)> = Vec::new();
    for start in 0..n {
        if comp[start] != usize::MAX {
            continue;
        }
        let c = roots.len();
        roots.push((start, dual.constrained(start).is_some()));
        comp[start] = c;
        queue.push_back(start);
        while let Some(f) = queue.pop_front() {
            for &e in dual.incident(f) {
                let de = &dual.edges()[e as usize];
                let other = if de.f as usize == f {
                    de.g as usize
                } else {
                    de.f as usize
                };
                if comp[other] == usize::MAX {
                    comp[other] = c;
                    if dual.constrained(other).is_some() {
                        roots[c].1 = true;
                    }
                    queue.push_back(other);
                }
            }
        }
    }
    for (start, held) in roots {
        if !held {
            seeds[start] = true;
        }
    }
    seeds
}

/// **A ÁRVORE GERADORA do grafo dual** — o gauge.
///
/// ⚠️ **BFS a partir da face 0, com vizinhos em ordem de índice.** Qualquer
/// árvore serve para consumir a liberdade de calibre; o que **não** serve é uma
/// árvore que mude entre corridas, porque ela decide quais inteiros existem.
///
/// # ⛔⛔ UMA FACE RESTRINGIDA NÃO PODE SER FILHA, e ignorá-lo custou 5× as singularidades
///
/// O gauge é uma **liberdade**: `θ_f → θ_f + (π/2)·m_f` com `p_e` compensado deixa a
/// energia igual, e é ela que permite pôr `p_e = 0` em toda aresta da árvore — cada
/// face **absorve** no seu `θ` o salto da aresta que a alcançou.
///
/// ⭐⭐⭐ **Eliminar `θ_f` remove essa liberdade naquela face** ([`crate::Dual::constrain`]):
/// o `θ` dela está fixo, `m_f` tem de ser `0`, e não há como absorver nada. Forçar
/// `p_e = 0` na aresta que a alcança injecta ali um quarto de volta arbitrário — e cada
/// um deles é uma singularidade a mais.
///
/// ⚠️ **MEDIDO (2026-08-25, peça do artista, 4 654 faces):** com as restringidas a serem
/// filhas como as outras, **26** faces fixas levavam o campo de **25** para **128**
/// singularidades, e 486 faces fixas a **579**. *O sintoma lê-se exactamente como «a
/// espec avisou: marcar feição a mais planta singularidades» — e não era isso.*
///
/// ⇒ **Elas entram como RAÍZES:** exploram-se a partir delas (o vizinho livre pode
/// absorver), e nenhuma aresta da árvore aponta para dentro delas. ⭐ Com nenhuma
/// restrição a lista de raízes é vazia e esta função é a de sempre, aresta a aresta.
fn spanning_tree(dual: &Dual) -> Vec<u32> {
    let n = dual.frames().len();
    let mut seen = vec![false; n];
    let mut tree: Vec<u32> = Vec::new();
    let mut queue = std::collections::VecDeque::new();
    // ⛔⛔ **Marcadas ANTES da primeira travessia, e a 1.ª versão desta cura não o fazia** —
    // ela punha-as só na frente da lista de raízes, e a BFS que arrancava na primeira delas
    // alcançava as outras 25 e dava-lhes um pai à mesma. ⚠️ *O sintoma era mudo: o
    // `free_integers` saía **idêntico** ao da corrida sem restrição nenhuma* — que é
    // exactamente o número que prova que a árvore não viu nada.
    for (f, s) in seen.iter_mut().enumerate().take(n) {
        if dual.constrained(f).is_some() {
            *s = true;
        }
    }
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
///
/// ⚠️ **O `align` entra por parâmetro e o representante 4-RoSy sai do `theta`
/// CORRENTE** — é dessa dependência que a [`crate::Continuation`] vive: mudar a
/// semente muda o alvo de cada face, e é por isso que refazer o arredondamento a
/// partir de um `θ` melhor não é a mesma conta outra vez.
fn solve_relaxation(
    dual: &Dual,
    fixed: &[Option<i32>],
    free: &[usize],
    theta: &mut [f32],
    align: f32,
) -> (Vec<f32>, f32) {
    let n = theta.len();
    let c = free.len();
    let dim = n + c;
    // aresta -> slot livre.
    let mut slot: Vec<Option<usize>> = vec![None; dual.edges().len()];
    for (i, &e) in free.iter().enumerate() {
        slot[e] = Some(i);
    }

    // ⭐⭐⭐ **AS VARIÁVEIS ELIMINADAS** — o gauge (face `0` a zero) e as faces que uma
    // aresta de feição restringe ([`crate::Dual::constrain`]).
    //
    // ⚠️ **As duas são a MESMA coisa para o sistema**, e é isso que torna a obra B
    // barata: o `pinned` já era uma eliminação, com valor `0` e cardinalidade `1`.
    // Generalizá-lo é dar-lhe um valor e um conjunto.
    //
    // ⚠️ **O representante 4-RoSy sai do `θ` CORRENTE**, pela razão do doc do
    // [`crate::ConstrainReport`]: `α` e `α + k·π/2` são a mesma cruz, e escolher o `k`
    // longe do `θ` de partida faria a face dar meia volta que o vizinho teria de pagar.
    let seeds = gauge_seeds(dual);
    let fix: Vec<Option<f32>> = (0..n)
        .map(|f| match dual.constrained(f) {
            Some(a) => {
                let k = ((theta[f] - a) / QUARTER).round();
                Some(QUARTER.mul_add(k, a))
            }
            None if seeds[f] => Some(0.0),
            None => None,
        })
        .collect();

    // b = −scatter( w · constante )
    let mut b = vec![0.0f32; dim];
    for (e, de) in dual.edges().iter().enumerate() {
        let mut konst = de.kappa + QUARTER * fixed[e].unwrap_or(0) as f32;
        // ⚠️ **O `θ` de uma face eliminada é CONSTANTE**, então ele viaja aqui, ao
        // lado do `κ` e do salto de período. ⭐ Com `fix` vazio o termo é `+0,0 − 0,0`
        // sobre a face do gauge e a conta é a de sempre, bit a bit.
        if let Some(a) = fix[de.f as usize] {
            konst += a;
        }
        if let Some(a) = fix[de.g as usize] {
            konst -= a;
        }
        let wc = de.weight * konst;
        b[de.f as usize] -= wc;
        b[de.g as usize] += wc;
        if let Some(i) = slot[e] {
            b[n + i] -= wc * QUARTER;
        }
    }

    // ⭐⭐ **O TERMO DE ALINHAMENTO, no lado direito.** A energia ganha
    // `λ·c_f·(θ_f − α_f)²`, cuja normal-equação é `+λ·c_f` na diagonal e
    // `+λ·c_f·α_f` no `b`.
    //
    // ⚠️ **O representante 4-RoSy é escolhido pelo `θ` CORRENTE**, e não é
    // detalhe: `α` e `α ± k·π/2` descrevem a mesma cruz, e fixar um deles puxaria
    // `θ` para um braço arbitrário — o campo daria meia volta onde a forma não
    // vira. Na primeira resolução o `θ` é o de partida, e as resoluções seguintes
    // já o refinam (o MIQ resolve dezenas de vezes).
    let pull: Vec<f32> = (0..n)
        .map(|f| {
            let (a, conf) = dual.align(f);
            // ⛔ Uma face eliminada não tem para onde ser puxada: ela já não é
            // incógnita, e um termo suave sobre ela seria energia sem variável.
            if conf <= 0.0 || fix[f].is_some() {
                return 0.0;
            }
            let k = ((theta[f] - a) / QUARTER).round();
            align * conf * QUARTER.mul_add(k, a)
        })
        .collect();
    for (f, item) in pull.iter().enumerate().take(n) {
        b[f] += item;
    }

    let mut x = vec![0.0f32; dim];
    x[..n].copy_from_slice(theta);
    for (f, item) in fix.iter().enumerate().take(n) {
        if item.is_some() {
            x[f] = 0.0;
        }
    }

    let mut ax = vec![0.0f32; dim];
    apply(dual, &slot, n, &x, &mut ax, &fix, align);
    let mut r: Vec<f32> = (0..dim).map(|i| b[i] - ax[i]).collect();
    for (f, item) in fix.iter().enumerate().take(n) {
        if item.is_some() {
            r[f] = 0.0;
        }
    }
    let mut p = r.clone();
    let mut rr = ddot(&r, &r);
    let r0 = rr.sqrt().max(1.0);

    let mut ap = vec![0.0f32; dim];
    for _ in 0..CG_ITERATIONS {
        if rr.sqrt() <= CG_TOLERANCE * r0 {
            break;
        }
        apply(dual, &slot, n, &p, &mut ap, &fix, align);
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

    // ⚠️ **O `θ` de uma face eliminada é o valor FIXO, nunca o `x` dela** — o `x` de
    // uma eliminada é `0` por construção, e escrevê-lo apagaria a restrição no exacto
    // sítio onde ela vale.
    for (f, t) in theta.iter_mut().enumerate().take(n) {
        *t = fix[f].unwrap_or(x[f]);
    }
    (x[n..].to_vec(), rr.sqrt() / r0)
}

/// `A x`, com a linha e a coluna de **cada variável eliminada** zeradas.
///
/// ⚠️ **O `fix` entrou no lugar do `pinned`, e não é uma generalização gratuita:** a
/// linha identidade que mantinha o sistema não-singular sobre a face do gauge é
/// exactamente a que uma face restringida precisa. *Um gauge é uma restrição com valor
/// zero, e escrever as duas coisas duas vezes seria pedir que divergissem.*
fn apply(
    dual: &Dual,
    slot: &[Option<usize>],
    n: usize,
    x: &[f32],
    out: &mut [f32],
    fix: &[Option<f32>],
    align: f32,
) {
    out.fill(0.0);
    for (e, de) in dual.edges().iter().enumerate() {
        let (f, g) = (de.f as usize, de.g as usize);
        let tf = if fix[f].is_some() { 0.0 } else { x[f] };
        let tg = if fix[g].is_some() { 0.0 } else { x[g] };
        let q = slot[e].map_or(0.0, |i| x[n + i]);
        let r = de.weight * (tf - tg + QUARTER * q);
        if fix[f].is_none() {
            out[f] += r;
        }
        if fix[g].is_none() {
            out[g] -= r;
        }
        if let Some(i) = slot[e] {
            out[n + i] += r * QUARTER;
        }
    }
    // ⭐⭐ **A diagonal do ALINHAMENTO.** O termo `λ·c_f·(θ_f − α_f)²` contribui
    // `λ·c_f` na diagonal de `A`; o `α_f` mora no `b` (ver [`solve_relaxation`]).
    //
    // ⚠️ **Ele também é o que torna o sistema DEFINIDO POSITIVO fora do gauge.**
    // Sem alinhamento, somar uma constante a todo `θ` não muda a energia, e é por
    // isso que o `pinned` existe; com ele, cada face confiante já tem um alvo — o
    // `pinned` continua porque uma malha inteiramente isotrópica não tem nenhum.
    for (f, o) in out.iter_mut().enumerate().take(n) {
        if fix[f].is_some() {
            continue;
        }
        let (_, conf) = dual.align(f);
        if conf > 0.0 {
            *o += align * conf * x[f];
        }
    }
    for (f, item) in fix.iter().enumerate().take(n) {
        if item.is_some() {
            out[f] = x[f];
        }
    }
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
