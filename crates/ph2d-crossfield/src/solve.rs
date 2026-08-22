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
    solve_miq_aligned(dual, policy, ALIGN_WEIGHT)
}

/// **O MIQ com o PESO DO ALINHAMENTO explícito** — ver [`ALIGN_WEIGHT`].
///
/// ⚠️ **Ele é um parâmetro porque o número tem de sair de uma VARREDURA**, e a
/// primeira tentativa provou-o: com `1,0` a cadeia inteira parou de fechar —
/// patches de **39, 98 e 127 lados**, quantização a recusar em toda fixtura.
/// *Um peso de energia escolhido sem tabela é um palpite com aparência de teoria.*
#[must_use]
pub fn solve_miq_aligned(dual: &Dual, policy: Rounding, align: f32) -> (CrossField, SolveReport) {
    let n = dual.frames().len();
    let m = dual.edges().len();
    let mut report = SolveReport {
        solves: 0,
        free_integers: 0,
        nonzero_periods: 0,
        cg_capped: 0,
        cg_worst_residual: 0.0,
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
        let (q, residual) = solve_relaxation(dual, &fixed, &free, &mut theta, align);
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
    let (_, residual) = solve_relaxation(dual, &fixed, &[], &mut theta, align);
    report.solves += 1;
    report.note(residual);

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
/// ⭐⭐ **O PESO DO ALINHAMENTO na energia** — quanto a cruz é puxada para a
/// direção principal de curvatura, contra a suavidade.
///
/// ⛔ **Sem ele a energia é SÓ suavidade, e o campo não tem como ver o relevo.**
/// Medido em 2026-08-22 com a régua [`ph2d_quadfill::follows_relief`], na fixtura
/// com cristas: a nossa cadeia dava **25,7°** de desvio — **pior que os 22,5° de
/// uma grade aleatória** — contra **13,7°** do porte do Instant Meshes.
///
/// ⚠️ **Ele é MULTIPLICADO PELA ANISOTROPIA de cada face**, então numa esfera
/// (onde as duas curvaturas são iguais e não há direção preferida) o termo
/// desaparece sozinho. *Um alinhamento que puxa onde a forma não tem direção põe
/// uma costura onde ela não pede nenhuma.*
///
/// ⛔⛔ **HOJE ELE É ZERO, e isso é uma MEDIÇÃO e não um esquecimento.** O termo
/// está construído e funciona — o desvio ao relevo cai —, mas **o arredondamento
/// do MIQ não o absorve**: com qualquer peso não-nulo o traçado parte. Medido em
/// 2026-08-22 (`how_much_alignment_can_the_field_take`), fixtura com cristas:
///
/// | peso | desvio ao relevo | patches | maior valência | a cadeia fecha? |
/// |---|---|---|---|---|
/// | **`0` (hoje)** | 25,7° | **21** | **5** | ✅ |
/// | `0,01` | ⭐ **20,4°** | ⛔ 104 | ⛔ 15 | ⚠️ fecha, com 139 irregulares |
/// | `0,03` | — | 134 | ⛔ **60** | ⛔ a montagem recusa |
/// | `1,0` | — | 98 | ⛔ **98** | ⛔ recusa |
///
/// ⭐ **O termo move a agulha certa** (`25,7° → 20,4°`, contra os `22,5°` de uma
/// grade aleatória e os `13,7°` do porte do Instant Meshes). ⛔ **E o layout
/// explode de 21 para 104 patches com um peso de `0,01`** — uma perturbação
/// minúscula de `θ` troca quais inteiros o arredondamento guloso congela, e cada
/// troca é uma singularidade a mais. *Um patch de 60 lados não é «mais detalhe»:
/// é traçado partido.*
///
/// ⛔ **Suavizar o guia foi construído, medido e REJEITADO no mesmo dia:** média
/// 4-RoSy sobre o grafo dual, transportada pelo `κ`, 32 rondas. O desvio ao relevo
/// **piorou** (`20,4° → 24,5°`) e o layout continuou a explodir. *A causa não é a
/// qualidade do guia; é a fragilidade do arredondamento.*
///
/// ⇒ **O próximo passo tem nome:** o arredondamento do MIQ tem de aguentar o
/// termo — a referência não congela guloso sobre um `θ` que acabou de mudar. Até
/// lá, ligar isto seria trocar uma grade que ignora o relevo por uma que não fecha.
const ALIGN_WEIGHT: f32 = 0.0;

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
            if conf <= 0.0 {
                return 0.0;
            }
            let k = ((theta[f] - a) / QUARTER).round();
            align * conf * QUARTER.mul_add(k, a)
        })
        .collect();
    for (f, item) in pull.iter().enumerate().take(n) {
        b[f] += item;
    }

    let pinned = 0usize;
    let mut x = vec![0.0f32; dim];
    x[..n].copy_from_slice(theta);
    x[pinned] = 0.0;

    let mut ax = vec![0.0f32; dim];
    apply(dual, &slot, n, &x, &mut ax, pinned, align);
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
        apply(dual, &slot, n, &p, &mut ap, pinned, align);
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
    (x[n..].to_vec(), rr.sqrt() / r0)
}

/// `A x`, com a linha e a coluna do gauge de `θ` zeradas.
fn apply(
    dual: &Dual,
    slot: &[Option<usize>],
    n: usize,
    x: &[f32],
    out: &mut [f32],
    pinned: usize,
    align: f32,
) {
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
    // ⭐⭐ **A diagonal do ALINHAMENTO.** O termo `λ·c_f·(θ_f − α_f)²` contribui
    // `λ·c_f` na diagonal de `A`; o `α_f` mora no `b` (ver [`solve_relaxation`]).
    //
    // ⚠️ **Ele também é o que torna o sistema DEFINIDO POSITIVO fora do gauge.**
    // Sem alinhamento, somar uma constante a todo `θ` não muda a energia, e é por
    // isso que o `pinned` existe; com ele, cada face confiante já tem um alvo — o
    // `pinned` continua porque uma malha inteiramente isotrópica não tem nenhum.
    for (f, o) in out.iter_mut().enumerate().take(n) {
        if f == pinned {
            continue;
        }
        let (_, conf) = dual.align(f);
        if conf > 0.0 {
            *o += align * conf * x[f];
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
