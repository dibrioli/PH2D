//! ⭐⭐⭐ **G5 — O ARREDONDAMENTO INTEIRO, uma variável de cada vez.**
//!
//! ⛔⛔ **A extracção ASSUME que as translações de transição são inteiras.** Se não
//! forem, a grade de uma carta não casa com a da vizinha e o saneamento da extracção
//! apenas *arredonda o erro para dentro*. É o bloqueador nomeado desta cadeia: o
//! [`crate::solve`] mede hoje `0,29` de célula de resíduo nas translações de ciclo.
//!
//! # A lei, e o nome dela é a receita
//!
//! *Misto-inteiro* significa **arredondar UMA variável de cada vez e ACTUALIZAR**,
//! nunca em lote:
//!
//! 1. resolver o sistema contínuo (é o [`crate::solve`]);
//! 2. entre as variáveis ainda livres que têm de ser inteiras, escolher a que **causa
//!    o menor erro de arredondamento** — a de menor `|x − round(x)|`;
//! 3. **pregá-la** nesse inteiro e **actualizar** a parte contínua (§5.1, abaixo);
//! 4. repetir até não sobrar variável livre.
//!
//! ⭐ **Por que uma-a-uma:** arredondar todas de uma vez desloca todas as outras ao
//! mesmo tempo, e o erro **soma**. Actualizar depois de cada uma deixa o sistema
//! **absorver** o deslocamento nas que ainda estão livres. *A premissa é que um erro
//! de arredondamento pequeno tem impacto pequeno no resultado — e é por isso que se
//! escolhe sempre o menor.*
//!
//! ⛔ [`crate::solve::rounded_shifts`] faz o contrário — arredonda **todas** de uma
//! vez — e é por isso que ele fica como controlo e não como produto.
//!
//! # ⭐⭐⭐ QUAIS variáveis, e a resposta já estava nesta crate
//!
//! ⚠️ **Não são todas as translações.** O [`crate::gauge`] provou que a translação de
//! uma costura é grandeza de **calibre**: somar uma constante ao `(u, v)` de um patch
//! muda todas as translações que lhe tocam sem mudar nada na peça. Numa **árvore** de
//! expansão do grafo de patches elas podem ser todas levadas a **zero** — de graça,
//! porque a energia só vê gradientes e diferenças de costura.
//!
//! ⇒ **os inteiros a escolher são as costuras que FECHAM CICLO**, e são
//! `E − V + componentes` delas. As da árvore ficam em `0`, que já é inteiro.
//!
//! # §5.1 — a ACTUALIZAÇÃO é adaptativa, e NÃO é um re-solve completo
//!
//! ⛔⛔ Re-resolver o sistema `k` vezes é exactamente o custo que o método existe para
//! não pagar. A receita é uma **escada**, subida só quando o degrau anterior não
//! converge:
//!
//! | degrau | o que é | quando |
//! |---|---|---|
//! | **1** | Gauss–Seidel **local**: fila semeada nos vértices da costura pregada, que cresce pelos vizinhos de quem se mexeu | o caminho normal |
//! | **2** | varreduras **globais**, orçamentadas | quando a fila estoura o tecto |
//! | **3** | factorização esparsa directa | ⏳ **não construído** — ver [`RoundReport::level2`] |
//!
//! ⭐ *A propagação segue o grafo de dependências, e na prática pára cedo: uma
//! variável arredondada mexe num vizinho local, não na malha inteira.*
//!
//! ⛔ **O que medir, e é o que separa isto de uma tradução:** a **fracção de
//! arredondamentos que fica no degrau 1** ([`RoundReport::level1`]). Se ela for baixa,
//! o tecto ou a tolerância estão mal escolhidos — e o custo vai para o degrau caro,
//! que é o que se queria evitar.

use ph2d_mesh::Mesh;

use crate::comb::Combed;
use crate::cut::CutMesh;
use crate::gauge;
use crate::solve::{
    Assembly, GridMap, SEAM_WEIGHT, SolveReport, assemble, measure, solve_with, turn2,
};

/// Os pares casados de uma costura, e o salto de período dela.
type SeamPairs = (Vec<(u32, u32, u32, u32)>, i32);

/// Os botões do arredondamento.
#[derive(Debug, Clone, Copy)]
pub struct RoundOptions {
    /// O peso da costura — o mesmo do [`crate::solve`].
    pub weight: f32,
    /// Rondas do solve contínuo inicial.
    pub rounds: usize,
    /// ⭐ **A TOLERÂNCIA do degrau 1**, em células: abaixo dela um vértice que se
    /// mexeu não acorda os vizinhos.
    pub local_tol: f32,
    /// ⭐ **O TECTO de visitas** do degrau 1, por arredondamento.
    pub local_cap: usize,
    /// Varreduras globais do degrau 2.
    pub sweeps: usize,
    /// ⭐⭐⭐ **A MODALIDADE DAS SINGULARIDADES**: pregar as **imagens dos vértices
    /// singulares** em pontos inteiros, antes de tocar nas translações.
    ///
    /// ⚠️ **Ela não é um refinamento — é o que faz a grade fechar à volta de uma
    /// singularidade**, e a medição di-lo: sem ela, o ponto fixo da holonomia de um
    /// vértice singular cai num **meio-inteiro** (a forma fechada dele é uma metade da
    /// translação), o vértice deixa de ser um nó da grade, e a malha rasga-se ali. Na
    /// esfera fina saíam **4** nós de vértice onde uma esfera precisa de oito.
    ///
    /// ⭐ Com a imagem pregada num inteiro `x`, a translação da holonomia passa a ser
    /// `(I − R^r)·x`, que é inteira **e com a paridade certa** por construção.
    pub pin_singularities: bool,
}

impl Default for RoundOptions {
    fn default() -> Self {
        Self {
            weight: SEAM_WEIGHT,
            rounds: crate::solve::ROUNDS,
            local_tol: LOCAL_TOL,
            local_cap: LOCAL_CAP,
            sweeps: SWEEPS,
            pin_singularities: true,
        }
    }
}

/// ⭐ **A TOLERÂNCIA do degrau 1**, em células da grade.
///
/// ⛔⛔ **O número sai da varredura, e a varredura desmentiu o palpite** (esfera
/// remalhada, 5 084 faces, 54 inteiros a pregar —
/// `the_rounding_ladder_sweeps_its_two_constants`):
///
/// | tolerância | tecto | ⭐ fica no degrau 1 | visitas | costura p50 · max | ângulo |
/// |---|---|---|---|---|---|
/// | ⭐ **`1e-2`** | **`20 000`** | ⭐ **`100 %`** | ⭐ **`17 775`** | `0,0285` · `1,038` | `8,6°` |
/// | `1e-2` | `2 000` | `98,1 %` | `17 439` | `0,0279` · `1,037` | `8,7°` |
/// | `1e-3` | `20 000` | `96,3 %` | `304 145` | `0,0284` · `1,091` | `8,3°` |
/// | ⛔ `1e-4` | `20 000` | ⛔ **`14,8 %`** | ⛔ **`1 025 615`** | `0,0279` · `1,090` | `8,4°` |
/// | ⛔ `1e-4` | `200 000` | `83,3 %` | ⛔ **`5 322 491`** | `0,0277` · `1,091` | `8,5°` |
///
/// ⇒ **`1e-4` custa 250× mais visitas para o MESMO resultado.** A primeira redacção
/// tinha-a a `1e-4` e a escada caía para o degrau caro em dois terços dos
/// arredondamentos — que é exactamente o sintoma que o método manda medir.
pub const LOCAL_TOL: f32 = 1.0e-2;

/// ⭐ **O TECTO de visitas do degrau 1**, por arredondamento.
///
/// ⚠️ **`2 000` já dá `98,1 %` e não poupa nada** (`17 439` visitas contra `17 775`):
/// o tecto não é o que limita, é a tolerância. Fica em `20 000` porque é onde a
/// fracção chega a `100 %` — *e um tecto que nenhuma fixtura toca é folga, não
/// política.*
pub const LOCAL_CAP: usize = 20_000;

/// Varreduras globais do degrau 2.
pub const SWEEPS: usize = 200;

/// O que o arredondamento mediu de si próprio.
#[derive(Debug, Clone, Default)]
pub struct RoundReport {
    /// Quantas componentes inteiras foram pregadas.
    pub pinned: usize,
    /// ⭐⭐⭐ **Quantas fecharam no degrau 1** (Gauss–Seidel local). *É a régua que
    /// separa a escada adaptativa de um re-solve disfarçado.*
    pub level1: usize,
    /// Quantas precisaram do degrau 2 (varreduras globais).
    ///
    /// ⏳ **O degrau 3 (factorização directa) não está construído**, e a razão é esta
    /// coluna: enquanto ela for zero, ele não teria consumidor nenhum e nada o
    /// mediria. *Construir o degrau caro antes de o barato falhar é construir o que
    /// nenhuma medição pede.*
    pub level2: usize,
    /// Visitas de vértice gastas no degrau 1, somadas.
    pub visits: usize,
    /// ⭐ O maior `|x − round(x)|` que foi preciso pagar, em células.
    pub worst_step: f32,
    /// A soma dos passos pagos — o custo total do arredondamento.
    pub sum_step: f32,
    /// Costuras na árvore de calibre (translação levada a zero, de graça).
    pub tree_seams: usize,
    /// ⭐ Costuras que **fecham ciclo** — as que carregam inteiros de verdade.
    pub cycle_seams: usize,
    /// ⭐ Vértices singulares cuja imagem foi pregada num ponto inteiro.
    pub singular_pinned: usize,
    /// ⛔⛔ **Cópias que o transporte RECUSOU mover** por estarem a mais de meia
    /// célula do valor transportado.
    ///
    /// ⚠️ **Cada uma nomeia uma costura cuja translação era ambígua** — ela ficou a
    /// meio caminho entre dois inteiros, e o arredondamento teve de escolher um lado.
    /// É o sintoma de que o solver contínuo ainda não fecha aquela costura, e **não**
    /// um defeito do arredondamento: ver o resíduo de costura em
    /// [`Self::seam_before`].
    pub ambiguous_seams: usize,
    /// ⭐ **Cópias** de vértices singulares levadas ao valor transportado.
    ///
    /// ⚠️ **Pregar UMA cópia não chega, e a medição di-lo:** na esfera fina saíam
    /// `6` nós de vértice para `8` singularidades pregadas, porque a extracção lê a
    /// imagem na carta do canto que encontrar primeiro — e as outras cópias ficavam
    /// onde a costura mole as tinha deixado, a um resto de inteiro.
    pub singular_copies: usize,
    /// ⚠️ **O CASO DE CANTO, e ele tem nome:** as singularidades esgotaram-se e ainda
    /// sobravam costuras por arredondar — acontece em peças com **alça**, e o nosso
    /// corpus tem um toro. ⛔ Terminar ali deixaria o mapa *quase* inteiro, que é pior
    /// que contínuo.
    pub switched_to_seams: bool,
    /// ⛔ **A pior distância a inteiro DEPOIS**. Tem de ser exactamente `0`.
    pub shift_frac_max: f32,
    /// O resíduo de costura antes e depois — o **preço** do arredondamento.
    pub seam_before: (f32, f32),
    /// O resíduo de costura depois, `(p50, max)`.
    pub seam_after: (f32, f32),
    /// As réguas do mapa final.
    pub solve: SolveReport,
    /// ⭐ A estrutura da soldadura — **só o caminho soldado a preenche**.
    pub weld: crate::weld::WeldReport,
    /// ⭐ O resíduo da costura separado por espécie — **só o caminho soldado**.
    ///
    /// ⚠️ Ele responde ao que [`Self::seam_after`] não distingue: aquele mistura as
    /// ligações eliminadas (onde o resíduo é o chão da representação) com as que fecham
    /// ciclo (onde é um facto do mapa). *Uma coluna que soma as duas lê-se como se a
    /// eliminação não tivesse acontecido.*
    pub seam: crate::weld::SeamResidual,
}

/// ⭐⭐⭐ **ARREDONDA O MAPA PARA A GRADE INTEIRA.**
///
/// Devolve um mapa cujas translações de transição são **todas inteiras** — que é a
/// pré-condição que a extracção assume e mede.
#[must_use]
pub fn round_to_integers(
    mesh: &Mesh,
    cut: &CutMesh,
    combed: &Combed,
    h: f32,
    opts: RoundOptions,
    singular: &[u32],
) -> (GridMap, RoundReport) {
    let (mut map, before) = solve_with(mesh, cut, combed, h, opts.weight, opts.rounds);
    let mut rep = RoundReport {
        seam_before: (before.seam_p50, before.seam_max),
        ..RoundReport::default()
    };
    let mut solve_rep = SolveReport::default();
    let a = assemble(mesh, cut, combed, h, &mut solve_rep);
    let mut r = Relaxer::new(&a, cut, combed, opts.weight);

    // ── 1. FIXAR O CALIBRE: as costuras de árvore vão a zero **de graça**.
    let (g, grep) = gauge::fix(cut, combed, &map);
    rep.tree_seams = grep.tree;
    rep.cycle_seams = grep.cycles;
    for (p, uv) in map.uv.iter_mut().enumerate() {
        let o = g.offset[p];
        for z in uv.iter_mut() {
            z[0] += o[0];
            z[1] += o[1];
        }
    }
    for (s, t) in map.shift.iter_mut().enumerate() {
        if g.in_tree[s] {
            *t = [0.0, 0.0];
        }
    }
    for &(s, inv) in &g.cycle {
        map.shift[s] = inv;
    }

    let mut pinned_seeds: Vec<(usize, usize)> = Vec::new();
    // ── 2. ⭐⭐⭐ MODALIDADE DAS SINGULARIDADES: a imagem de cada vértice singular
    // vai para um ponto INTEIRO, uma componente de cada vez.
    if opts.pin_singularities && !singular.is_empty() {
        let mut copies: Vec<(usize, usize)> = Vec::new();
        let wanted: std::collections::BTreeSet<u32> = singular.iter().copied().collect();
        for (p, origin) in cut.origin.iter().enumerate() {
            for (l, &g) in origin.iter().enumerate() {
                if wanted.contains(&g) && !copies.iter().any(|&(_, _)| false) {
                    copies.push((p, l));
                }
            }
        }
        // ⚠️ **UMA cópia por vértice global.** Pregar duas cópias do mesmo vértice
        // seria pedir ao sistema duas respostas para a mesma pergunta, e a costura
        // entre elas passaria a ter de as reconciliar.
        let mut seen: std::collections::BTreeSet<u32> = std::collections::BTreeSet::new();
        copies.retain(|&(p, l)| seen.insert(cut.origin[p][l]));
        let mut free_v: Vec<(usize, usize, usize)> = copies
            .iter()
            .flat_map(|&(p, l)| [(p, l, 0usize), (p, l, 1usize)])
            .collect();
        while !free_v.is_empty() {
            let (best, _) = free_v
                .iter()
                .enumerate()
                .map(|(i, &(p, l, ax))| {
                    let x = map.uv[p][l][ax];
                    (i, (x - x.round()).abs())
                })
                .fold((0usize, f32::INFINITY), |acc, (i, d)| {
                    if d < acc.1 { (i, d) } else { acc }
                });
            let (p, l, ax) = free_v.swap_remove(best);
            let x = map.uv[p][l][ax];
            let step = (x - x.round()).abs();
            rep.worst_step = rep.worst_step.max(step);
            rep.sum_step += step;
            map.uv[p][l][ax] = x.round();
            r.freeze(p, l, ax);
            rep.pinned += 1;
            let (visits, converged) = r.local_at(&mut map, p, l, opts.local_tol, opts.local_cap);
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
        }
        rep.singular_pinned = copies.len();
        pinned_seeds = copies;
    }

    // ── 3. ⚠️ **E AGORA AS COSTURAS.** As singularidades esgotaram-se e as
    // translações continuam por arredondar — terminar aqui deixaria o mapa *quase*
    // inteiro, que é pior que contínuo.
    // A ESCADA GULOSA: a de menor erro primeiro, e actualizar a seguir.
    let mut free: Vec<(usize, usize)> = g
        .cycle
        .iter()
        .flat_map(|&(s, _)| [(s, 0usize), (s, 1usize)])
        .collect();
    rep.switched_to_seams = rep.singular_pinned > 0 && !free.is_empty();
    // ⚠️ **As translações relêem-se do mapa depois de as singularidades se pregarem**
    // — o calibre foi fixado antes disso, e usar os valores velhos seria escolher a
    // ordem gulosa com números que já não descrevem o mapa.
    for &(sid, _) in &g.cycle {
        if let Some(fresh) = r.reread_shift(&map, sid) {
            map.shift[sid] = fresh;
        }
    }
    while !free.is_empty() {
        let (best, _) = free
            .iter()
            .enumerate()
            .map(|(i, &(s, ax))| {
                let x = map.shift[s][ax];
                (i, (x - x.round()).abs())
            })
            .fold((0usize, f32::INFINITY), |acc, (i, d)| {
                if d < acc.1 { (i, d) } else { acc }
            });
        let (s, ax) = free.swap_remove(best);
        let x = map.shift[s][ax];
        let step = (x - x.round()).abs();
        rep.worst_step = rep.worst_step.max(step);
        rep.sum_step += step;
        map.shift[s][ax] = x.round();
        rep.pinned += 1;

        // ── §5.1 degrau 1: Gauss–Seidel LOCAL, semeado na costura que se mexeu.
        let (visits, converged) = r.local(&mut map, s, opts.local_tol, opts.local_cap);
        rep.visits += visits;
        if converged {
            rep.level1 += 1;
        } else {
            // ── degrau 2: varreduras globais, orçamentadas.
            rep.level2 += 1;
            for _ in 0..opts.sweeps {
                if r.sweep(&mut map) < opts.local_tol {
                    break;
                }
            }
        }

        // ── ⭐⭐⭐ **E A PARTE CONTÍNUA ABSORVE**: as translações ainda LIVRES
        // relêem-se do mapa que acabou de se mexer. É isto que faz «uma de cada vez»
        // ser diferente de «todas de uma vez» — ver [`Relaxer::reread_shift`].
        let mut touched: Vec<usize> = free.iter().map(|&(s, _)| s).collect();
        touched.sort_unstable();
        touched.dedup();
        for t in touched {
            let Some(fresh) = r.reread_shift(&map, t) else {
                continue;
            };
            for (ax, &value) in fresh.iter().enumerate() {
                if free.contains(&(t, ax)) {
                    map.shift[t][ax] = value;
                }
            }
        }
    }

    // ── 4. ⭐⭐ **AS CÓPIAS DE UMA SINGULARIDADE SÃO A MESMA IMAGEM.**
    //
    // Com as translações já inteiras, transportar a cópia pregada pelas transições dá
    // inteiros em todas as cartas — uma rotação de um quarto de volta mais uma
    // translação inteira leva inteiros a inteiros. ⚠️ *É a propagação do saneamento da
    // extracção, um nível acima, e sem ela metade das singularidades não é um nó da
    // grade.*
    let (moved, refused) = r.propagate(&mut map, &pinned_seeds);
    rep.singular_copies = moved;
    rep.ambiguous_seams = refused;

    measure(&a, cut, combed, &map, h, &mut solve_rep);
    rep.seam_after = (solve_rep.seam_p50, solve_rep.seam_max);
    rep.shift_frac_max = map
        .shift
        .iter()
        .map(|t| (t[0] - t[0].round()).abs().max((t[1] - t[1].round()).abs()))
        .fold(0.0f32, f32::max);
    rep.solve = solve_rep;
    (map, rep)
}

/// ⭐ **O RELAXADOR** — o mesmo sistema do [`crate::solve`], um vértice de cada vez.
///
/// ⚠️ **Ele NÃO é um segundo solver.** A varredura do [`crate::solve`] calcula os
/// numeradores de um patch inteiro antes de aplicar (Jacobi por patch); este aplica
/// vértice a vértice (Gauss–Seidel). São dois **calendários** sobre a **mesma**
/// equação, e é por isso que a paridade se cobra como *«o mapa convergido de um é
/// ponto fixo do outro»* e não ao bit — ver `round_tests`.
pub(crate) struct Relaxer<'a> {
    a: &'a Assembly,
    /// Por costura, os pares `(patch, local)` dos dois lados.
    on_seam: Vec<Vec<(u32, u32)>>,
    /// ⭐ Por patch, por vértice local, que componentes estão **pregadas**.
    ///
    /// ⚠️ **Por COMPONENTE e não por vértice:** o guloso prega `u` e `v` em momentos
    /// diferentes (é a de menor erro primeiro), e congelar as duas de uma vez faria a
    /// segunda deixar de poder absorver o passo da primeira.
    frozen: Vec<Vec<[bool; 2]>>,
    /// ⭐ Por costura, os pares casados `(patch_a, local_a, patch_b, local_b)` e o
    /// salto de período — o que é preciso para **reler** a translação do mapa vivo.
    pairs: Vec<SeamPairs>,
    weight: f32,
}

impl<'a> Relaxer<'a> {
    pub(crate) fn new(a: &'a Assembly, cut: &CutMesh, combed: &Combed, weight: f32) -> Self {
        let mut on_seam: Vec<Vec<(u32, u32)>> = vec![Vec::new(); cut.seams.len()];
        for (p, list) in a.partners.iter().enumerate() {
            for (l, qs) in list.iter().enumerate() {
                for q in qs {
                    #[allow(clippy::cast_possible_truncation)]
                    on_seam[q.seam as usize].push((p as u32, l as u32));
                    on_seam[q.seam as usize].push((q.patch, q.local));
                }
            }
        }
        for v in &mut on_seam {
            v.sort_unstable();
            v.dedup();
        }
        let mut pairs: Vec<SeamPairs> = vec![(Vec::new(), 0); cut.seams.len()];
        for (s, seam) in cut.seams.iter().enumerate() {
            let Some(k) = combed.jump.get(s).copied().flatten() else {
                continue;
            };
            pairs[s].1 = k;
            for (la, lb) in seam.side[0].local.iter().zip(&seam.side[1].local) {
                if let (Some(la), Some(lb)) = (la, lb) {
                    pairs[s]
                        .0
                        .push((seam.side[0].patch, *la, seam.side[1].patch, *lb));
                }
            }
        }
        let frozen = a.denom.iter().map(|d| vec![[false; 2]; d.len()]).collect();
        Self {
            a,
            on_seam,
            pairs,
            frozen,
            weight,
        }
    }

    /// ⭐⭐⭐ **RELÊ a translação de uma costura do MAPA VIVO** — a média do resíduo,
    /// que é a mesma lei com que o solver contínuo a move.
    ///
    /// ⛔⛔ **Sem isto o guloso é o LOTE disfarçado**, e a medição di-lo em voz alta:
    /// varrendo a tolerância e o tecto por nove configurações, a **soma dos passos
    /// pagos saiu idêntica nas nove** (`14,24`). Uma escolha gulosa que não muda
    /// quando a actualização muda não está a ver a actualização — ela lê os valores
    /// congelados do calibre inicial, e arredondar uma variável deixa de deslocar as
    /// outras. *O nome do método é «uma de cada vez» precisamente por causa disso.*
    fn reread_shift(&self, map: &GridMap, s: usize) -> Option<[f32; 2]> {
        let (pairs, k) = self.pairs.get(s)?;
        if pairs.is_empty() {
            return None;
        }
        let (mut acc, mut n) = ([0.0f32; 2], 0.0f32);
        for &(pa, la, pb, lb) in pairs {
            let za = turn2(map.uv[pa as usize][la as usize], *k);
            let zb = map.uv[pb as usize][lb as usize];
            acc[0] += zb[0] - za[0];
            acc[1] += zb[1] - za[1];
            n += 1.0;
        }
        Some([acc[0] / n, acc[1] / n])
    }

    /// Prega uma componente de um vértice.
    fn freeze(&mut self, p: usize, l: usize, ax: usize) {
        self.frozen[p][l][ax] = true;
    }

    /// **RELAXA UM VÉRTICE** e devolve o quanto ele se mexeu.
    ///
    /// ⚠️ A equação é a do [`crate::solve`] — Poisson sobre os triângulos incidentes,
    /// mais um termo por par de costura, com o peso **relativo ao denominador do
    /// próprio vértice** (é isso que o torna independente da escala da peça).
    pub(crate) fn relax(&self, map: &mut GridMap, p: usize, l: usize) -> f32 {
        let base = self.a.denom[p][l];
        if base <= 0.0 {
            return 0.0;
        }
        let [mut nu, mut nv] = crate::solve::poisson_numerator(self.a, map, p, l);
        let w = self.weight * base;
        let mut den = base;
        for q in &self.a.partners[p][l] {
            let other = map.uv[q.patch as usize][q.local as usize];
            let t = map.shift[q.seam as usize];
            let want = if q.first {
                turn2([other[0] - t[0], other[1] - t[1]], -q.jump)
            } else {
                let r = turn2(other, q.jump);
                [r[0] + t[0], r[1] + t[1]]
            };
            nu += w * want[0];
            nv += w * want[1];
            den += w;
        }
        let old = map.uv[p][l];
        let f = self.frozen[p][l];
        let next = [
            if f[0] { old[0] } else { nu / den },
            if f[1] { old[1] } else { nv / den },
        ];
        map.uv[p][l] = next;
        (next[0] - old[0]).abs().max((next[1] - old[1]).abs())
    }

    /// Uma varredura global — devolve o maior movimento.
    pub(crate) fn sweep(&self, map: &mut GridMap) -> f32 {
        let mut worst = 0.0f32;
        for p in 0..self.a.denom.len() {
            for l in 0..self.a.denom[p].len() {
                worst = worst.max(self.relax(map, p, l));
            }
        }
        worst
    }

    /// ⭐ **O DEGRAU 1** — Gauss–Seidel local, semeado nos vértices de uma costura.
    ///
    /// Devolve `(visitas, convergiu)`. ⚠️ *«Convergiu» significa que a fila esvaziou
    /// antes do tecto* — e é essa a distinção que a [`RoundReport::level1`] conta.
    fn local(&self, map: &mut GridMap, seam: usize, tol: f32, cap: usize) -> (usize, bool) {
        self.drain(map, self.on_seam[seam].clone(), tol, cap)
    }

    /// O mesmo degrau 1, semeado num **vértice** — a modalidade das singularidades.
    fn local_at(
        &self,
        map: &mut GridMap,
        p: usize,
        l: usize,
        tol: f32,
        cap: usize,
    ) -> (usize, bool) {
        #[allow(clippy::cast_possible_truncation)]
        let seeds = self.neighbours(p, l);
        self.drain(map, seeds, tol, cap)
    }

    fn drain(
        &self,
        map: &mut GridMap,
        seeds: Vec<(u32, u32)>,
        tol: f32,
        cap: usize,
    ) -> (usize, bool) {
        let mut queue: std::collections::VecDeque<(u32, u32)> = seeds.iter().copied().collect();
        let mut queued: std::collections::BTreeSet<(u32, u32)> = seeds.into_iter().collect();
        let mut visits = 0usize;
        while let Some((p, l)) = queue.pop_front() {
            queued.remove(&(p, l));
            visits += 1;
            if visits > cap {
                return (visits, false);
            }
            if self.relax(map, p as usize, l as usize) <= tol {
                continue;
            }
            for n in self.neighbours(p as usize, l as usize) {
                if queued.insert(n) {
                    queue.push_back(n);
                }
            }
        }
        (visits, true)
    }

    /// ⭐ **TRANSPORTA cada cópia de um vértice pregado** a partir da cópia semente,
    /// pelas transições. Devolve quantas cópias mexeu.
    fn propagate(&self, map: &mut GridMap, seeds: &[(usize, usize)]) -> (usize, usize) {
        let mut moved = 0usize;
        let mut refused = 0usize;
        for &(p0, l0) in seeds {
            #[allow(clippy::cast_possible_truncation)]
            let start = (p0 as u32, l0 as u32);
            let mut seen: std::collections::BTreeSet<(u32, u32)> =
                std::collections::BTreeSet::from([start]);
            let mut queue = std::collections::VecDeque::from([start]);
            while let Some((p, l)) = queue.pop_front() {
                let here = map.uv[p as usize][l as usize];
                for q in &self.a.partners[p as usize][l as usize] {
                    if !seen.insert((q.patch, q.local)) {
                        continue;
                    }
                    let t = map.shift[q.seam as usize];
                    // ⚠️ **O sentido é o do solver**, e trocá-lo põe a cópia do outro
                    // lado num sítio plausível e errado: `z_b = R^k z_a + t`.
                    let want = if q.first {
                        let rr = turn2(here, q.jump);
                        [rr[0] + t[0], rr[1] + t[1]]
                    } else {
                        turn2([here[0] - t[0], here[1] - t[1]], -q.jump)
                    };
                    let slot = &mut map.uv[q.patch as usize][q.local as usize];
                    // ⛔⛔ **A GUARDA, e ela nomeia um fenómeno real.** Se o valor
                    // transportado está a mais de meia célula de onde a cópia já
                    // estava, a translação daquela costura é **ambígua** — ela ficou a
                    // meio caminho entre dois inteiros e o arredondamento escolheu um
                    // lado. Forçar a cópia ali rasga a malha por uma célula inteira,
                    // e foi o que a medição mostrou (`costura max 0,23 → 1,00`).
                    // ⇒ *deixa-se a cópia onde está e CONTA-SE*, em vez de escrever um
                    // número plausível por cima de um defeito de outra fase.
                    let far = (want[0] - slot[0]).abs().max((want[1] - slot[1]).abs());
                    if far <= 0.5 {
                        if *slot != want {
                            *slot = want;
                            moved += 1;
                        }
                    } else {
                        refused += 1;
                    }
                    queue.push_back((q.patch, q.local));
                }
            }
        }
        (moved, refused)
    }

    /// Os vizinhos de um vértice: os do triângulo, mais os pares de costura.
    fn neighbours(&self, p: usize, l: usize) -> Vec<(u32, u32)> {
        let mut out: Vec<(u32, u32)> = Vec::with_capacity(12);
        #[allow(clippy::cast_possible_truncation)]
        let p32 = p as u32;
        for &ti in &self.a.by_vert[p][l] {
            for v in self.a.tris[p][ti as usize].v {
                if v as usize != l {
                    out.push((p32, v));
                }
            }
        }
        for q in &self.a.partners[p][l] {
            out.push((q.patch, q.local));
        }
        out.sort_unstable();
        out.dedup();
        out
    }
}

#[cfg(test)]
#[path = "round_tests.rs"]
pub(crate) mod tests;
