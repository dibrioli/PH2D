//! **O LAYOUT VIRA REDE BI-DIRIGIDA** — a tradução do §4.4.1 do Bi-MDF.
//!
//! # Um nó por LADO, e duas famílias de aresta
//!
//! | o quê | liga | sinais | por quê |
//! |---|---|---|---|
//! | **arco** `x_a` | o lado de `p` ao lado de `q` que o partilham | `+1, +1` | ele **soma** no comprimento dos dois lados |
//! | **leque** `e_j` | os lados `j−1` e `j+1` do MESMO patch | `−1, −1` | ele é **consumido** pelos dois |
//!
//! A conservação no nó do lado `i` é, letra por letra, a lei do patch:
//! `Σ_{a ∈ lado i} x_a − e_{i−1} − e_{i+1} = 0`.
//!
//! ⚠️ **Os dois sinais iguais são o problema inteiro.** Uma aresta cujas pontas
//! apontam ambas para dentro (*head-head*) ou ambas para fora (*tail-tail*) não é
//! um arco de rede de fluxo comum — e é por isso que o ótimo exato pede matching
//! e não Dijkstra. Ver [`crate::solve`].
//!
//! # ⛔ O nó de emergência do paper NÃO está aqui, e é uma decisão medida
//!
//! O §4.4.1 prevê um *emergency node* por patch, que drena uma quantidade **par**
//! de fluxo e assim admite quantizações **irregulares** (mais de um vértice
//! irregular dentro do patch). Ele é a válvula para layouts sem solução regular.
//!
//! Ele fica de fora do protótipo **de propósito**: com ele, uma resposta
//! "válida" pode conter patches que o [`crate::verify`] não consegue fechar, e o
//! certificado deixa de ser um certificado. Sem ele, *infeasible* é uma resposta
//! honesta e **mensurável** — e a medição sobre o corpus está no PLAN. Ligá-lo é
//! trabalho de meia hora **no dia em que um layout real precisar dele**.

use crate::Layout;

/// Uma aresta bi-dirigida: dois pares `(nó, sinal)`, uma faixa e um custo.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BiEdge {
    /// Primeiro nó.
    pub a: u32,
    /// O sinal com que a aresta entra na conservação de `a`.
    pub sa: i8,
    /// Segundo nó.
    pub b: u32,
    /// O sinal com que a aresta entra na conservação de `b`.
    pub sb: i8,
    /// Piso do fluxo.
    pub lo: i64,
    /// Teto do fluxo.
    pub hi: i64,
    /// O comprimento desejado (`0` para uma aresta de leque, que é livre).
    pub target: f64,
    /// O peso no custo (`0` para uma aresta de leque).
    pub weight: f64,
    /// ⭐⭐ **A FORMA do custo** — ver [`crate::Deviation`]. É ela que decide se o
    /// ótimo esmaga um arco longo ou espalha o erro, e a máquina que a consome
    /// (`step_cost` + `segments`) já existia: só o custo era linear.
    pub kind: crate::Deviation,
    /// O arco do layout, quando esta aresta é um; `None` se é aresta de leque.
    pub arc: Option<u32>,
    /// **A PARTIDA A QUENTE** — o valor que esta aresta provavelmente terá, para
    /// o fluxo não ter de o descobrir uma unidade de cada vez.
    ///
    /// ⚠️ **Só é lida para arestas de LEQUE** (custo zero). Um arco tem custo, e
    /// pôr-lhe fluxo à mão quebraria a otimalidade — ele já parte quente pela
    /// pré-saturação dos degraus de custo negativo. Ver [`crate::mcf::Mcf::preload`].
    pub warm: i64,
}

impl BiEdge {
    /// O custo de fazer esta aresta valer `x` — ver [`crate::Deviation`].
    #[allow(clippy::cast_precision_loss)]
    #[must_use]
    pub fn cost(&self, x: i64) -> f64 {
        crate::deviation(self.kind, self.weight, self.target, x as f64)
    }

    /// O custo do `k`-ésimo passo (de `x = k−1` para `x = k`).
    ///
    /// ⚠️ **É esta função que faz o custo convexo caber num fluxo.** Ela é
    /// não-decrescente em `k` — é a definição de convexidade sobre os inteiros —
    /// e é isso que permite representar `w·|x − t|` por arcos paralelos que o
    /// fluxo preenche na ordem certa **sem ninguém lhe dizer a ordem**.
    #[must_use]
    pub fn step_cost(&self, k: i64) -> f64 {
        self.cost(k) - self.cost(k - 1)
    }
}

/// A rede bi-dirigida de um layout.
#[derive(Debug, Clone, PartialEq)]
pub struct BiNetwork {
    nodes: usize,
    edges: Vec<BiEdge>,
    /// Por patch, o índice do primeiro nó de lado dele.
    first_side: Vec<u32>,
    /// Quantos arcos do layout — os `edges[..n_arcs]` são os arcos, nessa ordem.
    n_arcs: usize,
}

/// ⚠️ **A folga do teto, e o que ela é.** O teto de um arco não é um limite
/// físico: é o ponto a partir do qual o solver não tem razão nenhuma para ir.
/// Ele existe só porque um fluxo com capacidade infinita não termina.
///
/// ⛔ **E ele MENTE se for fixo.** Medido em 2026-08-20 sobre a `sphere_noisy`
/// (3 613 arcos, alvos de 0,1 a 100): com o teto apertado, o solver devolvia
/// *"não existe quantização"* — uma afirmação sobre o LAYOUT — quando o que não
/// cabia era o teto. É por isso que ele **escala** ([`BiNetwork::build_scaled`])
/// em vez de ser uma constante, e que o [`crate::Report::cap_binding`] conta os
/// arcos encostados nele.
const CAP_SLACK: f64 = 8.0;
/// O múltiplo do alvo até onde o teto vai.
const CAP_FACTOR: f64 = 4.0;
/// ⚠️ **Os degraus do teto.** O primeiro é o rápido; os seguintes só correm se o
/// anterior disser *"inviável"*, e cada um é uma pergunta a menos sobre se foi o
/// teto que decidiu. Medido: todo layout **fechado** do oráculo resolve no
/// primeiro degrau.
pub const CAP_STEPS: [i64; 4] = [1, 4, 16, 64];

impl BiNetwork {
    /// **CONSTRÓI** a rede com o teto apertado — o degrau rápido.
    #[must_use]
    pub fn build(layout: &Layout) -> Self {
        Self::build_scaled(layout, 1)
    }

    /// **CONSTRÓI** a rede com o teto multiplicado por `cap`.
    ///
    /// ⚠️ **Só faz sentido subir quando o degrau anterior disse *inviável***: um
    /// teto maior admite mais fluxo e custa mais relógio, sem mudar o ótimo de um
    /// problema que já cabia.
    #[must_use]
    pub fn build_scaled(layout: &Layout, cap: i64) -> Self {
        // Um nó por lado de patch, na ordem dos patches.
        let mut first_side = Vec::with_capacity(layout.patches().len());
        let mut nodes = 0usize;
        for p in layout.patches() {
            first_side.push(u32::try_from(nodes).unwrap_or(u32::MAX));
            nodes += p.sides.len();
        }

        // Onde cada arco aparece: (nó do lado) das duas vezes. O `Layout` já
        // garantiu que são exatamente duas.
        let mut seen: Vec<[u32; 2]> = vec![[u32::MAX; 2]; layout.arcs().len()];
        let mut count = vec![0u8; layout.arcs().len()];
        for (p, patch) in layout.patches().iter().enumerate() {
            for (i, side) in patch.sides.iter().enumerate() {
                let node = first_side[p] + u32::try_from(i).unwrap_or(0);
                for &a in side {
                    let k = count[a as usize] as usize;
                    seen[a as usize][k] = node;
                    count[a as usize] += 1;
                }
            }
        }

        let mut edges = Vec::with_capacity(layout.arcs().len() + nodes);
        let mut arc_hi = Vec::with_capacity(layout.arcs().len());
        for (a, spec) in layout.arcs().iter().enumerate() {
            let lo = i64::from(spec.min);
            let hi = lo + (CAP_FACTOR * spec.target).ceil().max(CAP_SLACK) as i64 * cap;
            arc_hi.push(hi);
            edges.push(BiEdge {
                a: seen[a][0],
                sa: 1,
                b: seen[a][1],
                sb: 1,
                lo,
                hi,
                target: spec.target,
                weight: spec.weight,
                kind: spec.kind,
                arc: Some(u32::try_from(a).unwrap_or(u32::MAX)),
                warm: lo,
            });
        }
        let n_arcs = edges.len();

        // As arestas do leque: `e_j` liga os lados `j−1` e `j+1` do patch.
        for (p, patch) in layout.patches().iter().enumerate() {
            let n = patch.sides.len();
            // ⚠️ O teto de uma aresta de leque é o do lado mais folgado do patch:
            // ela nunca precisa passar do comprimento que um lado pode ter.
            let hi: i64 = patch
                .sides
                .iter()
                .map(|s| s.iter().map(|&a| arc_hi[a as usize]).sum::<i64>())
                .max()
                .unwrap_or(CAP_SLACK as i64);
            // ⭐ **A PARTIDA A QUENTE sai da própria LEI DO PATCH.** A
            // pré-saturação põe cada arco no maior inteiro cujo passo ainda baixa
            // o custo — que é `floor(alvo)`, nunca menos que o mínimo. Com esses
            // comprimentos de lado, [`solve_corners`] devolve os `e_j` que
            // **zeram exatamente** a conservação de cada nó deste patch.
            //
            // ⚠️ **Isto não é afinação, é a diferença entre 0 e 250 ms.** Medido
            // em 2026-08-20: com a estimativa grosseira abaixo, uma grelha de 512
            // arcos com alvos **dispersos** custava **231 ms numa única
            // resolução** de fluxo, contra 0 ms com alvos uniformes — o
            // caminho-mais-curto sucessivo paga uma travessia por unidade de
            // desequilíbrio, e a estimativa só acertava quando o layout era
            // uniforme. *Uma partida a quente que só serve o caso fácil não é
            // partida a quente.*
            let side_start: Vec<u32> = patch
                .sides
                .iter()
                .map(|s| {
                    s.iter()
                        .map(|&a| {
                            let spec = &layout.arcs()[a as usize];
                            let floor = spec.target.floor().max(0.0) as u32;
                            floor.max(spec.min)
                        })
                        .sum()
                })
                .collect();
            // Quando a lei não fecha nesses inteiros (paridade, lado curto), cai
            // na estimativa grosseira — que é exata na grelha uniforme.
            let fan = crate::solve_corners(&side_start).unwrap_or_else(|_| {
                (0..n)
                    .map(|j| {
                        let (b, a) = ((j + n - 1) % n, (j + 1) % n);
                        (side_start[b] + side_start[a]).div_ceil(4).max(1)
                    })
                    .collect()
            });
            for (j, &e) in fan.iter().enumerate().take(n) {
                let (before, after) = ((j + n - 1) % n, (j + 1) % n);
                let prev = first_side[p] + u32::try_from(before).unwrap_or(0);
                let next = first_side[p] + u32::try_from(after).unwrap_or(0);
                let warm = i64::from(e);
                edges.push(BiEdge {
                    a: prev,
                    sa: -1,
                    b: next,
                    sb: -1,
                    lo: 1,
                    hi,
                    target: 0.0,
                    weight: 0.0,
                    // ⚠️ A aresta de leque é **livre** (peso zero), então a forma
                    // do custo é irrelevante — mas ela tem de existir para o campo
                    // não ter um default escondido.
                    kind: crate::Deviation::Abs,
                    arc: None,
                    warm: warm.clamp(1, hi),
                });
            }
        }

        Self {
            nodes,
            edges,
            first_side,
            n_arcs,
        }
    }

    /// Quantos nós (lados de patch).
    #[must_use]
    pub fn nodes(&self) -> usize {
        self.nodes
    }

    /// As arestas. Os primeiros [`BiNetwork::arc_count`] são os arcos do layout.
    #[must_use]
    pub fn edges(&self) -> &[BiEdge] {
        &self.edges
    }

    /// Quantas arestas correspondem a arcos do layout.
    #[must_use]
    pub fn arc_count(&self) -> usize {
        self.n_arcs
    }

    /// O nó do lado `side` do patch `patch`.
    #[must_use]
    pub fn side_node(&self, patch: usize, side: usize) -> u32 {
        self.first_side[patch] + u32::try_from(side).unwrap_or(0)
    }

    /// **A CONSERVAÇÃO, medida** — o resíduo em cada nó para um fluxo dado.
    ///
    /// ⚠️ Ela existe para o gate: um fluxo que o solver diz ser viável e cujo
    /// resíduo não é zero em todo nó é um solver quebrado, e nenhuma inspeção do
    /// resultado final mostraria isso.
    #[must_use]
    pub fn residual(&self, flow: &[i64]) -> Vec<i64> {
        let mut out = vec![0i64; self.nodes];
        for (e, edge) in self.edges.iter().enumerate() {
            out[edge.a as usize] += i64::from(edge.sa) * flow[e];
            out[edge.b as usize] += i64::from(edge.sb) * flow[e];
        }
        out
    }
}
