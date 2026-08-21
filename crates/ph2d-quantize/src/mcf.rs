//! **FLUXO DE CUSTO MÍNIMO** com demandas e custos negativos.
//!
//! ⚠️ **Este módulo é DIRIGIDO e nada mais.** Ele não sabe o que é um patch, um
//! arco ou uma bi-aresta: é o motor sobre o qual a *dupla cobertura* do
//! [`crate::solve`] roda. Mantê-lo ignorante é o que o torna testável sozinho.
//!
//! # A única esperteza aqui, e por que ela basta
//!
//! Custos **negativos** aparecem naturalmente (o primeiro passo de um arco que
//! está abaixo do seu alvo *reduz* o custo), e caminho-mais-curto com peso
//! negativo pede Bellman–Ford, que é lento e desagradável.
//!
//! A cura clássica: **satura todo arco de custo negativo antes de começar**.
//! Depois disso, um arco ou está no piso com custo `>= 0` (só existe o resíduo
//! para a frente) ou está no teto com custo `< 0` (só existe o resíduo para
//! trás, de custo `> 0`). ⭐ **Todo custo residual é `>= 0`** — e Dijkstra com
//! potenciais volta a ser exato, do primeiro passo ao último.

use std::cmp::Ordering;
use std::collections::{BinaryHeap, VecDeque};

/// Por que um fluxo não existe.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum McfError {
    /// As demandas não podem ser satisfeitas: falta capacidade em algum corte.
    Infeasible {
        /// Quanto ficou por rotear.
        missing: i64,
    },
    /// ⚠️ **O teto de AUMENTOS mordeu antes de o fluxo fechar.** É afirmação
    /// sobre o esforço, não sobre a rede — confundi-la com
    /// [`McfError::Infeasible`] faria um problema resolúvel parecer impossível.
    Exhausted {
        /// Quantos aumentos foram gastos.
        augmentations: usize,
    },
}

/// O motor. Arcos entram por [`Mcf::arc`], demandas por [`Mcf::demand`].
#[derive(Debug, Clone)]
pub struct Mcf {
    graph: Vec<Vec<usize>>,
    to: Vec<usize>,
    res: Vec<i64>,
    cost: Vec<f64>,
    cap0: Vec<i64>,
    demand: Vec<i64>,
    source: usize,
    sink: usize,
    augmentations: usize,
}

impl Mcf {
    /// Uma rede com `nodes` nós. ⚠️ Dois nós extra (super-fonte e super-sorvedouro)
    /// são reservados no fim e **não** devem ser referenciados por [`Mcf::arc`].
    #[must_use]
    pub fn new(nodes: usize) -> Self {
        Self {
            graph: vec![Vec::new(); nodes + 2],
            to: Vec::new(),
            res: Vec::new(),
            cost: Vec::new(),
            cap0: Vec::new(),
            demand: vec![0; nodes + 2],
            source: nodes,
            sink: nodes + 1,
            augmentations: 0,
        }
    }

    /// Acrescenta um arco `from -> to` com capacidade e custo unitário.
    /// Devolve o id pelo qual o fluxo se lê depois ([`Mcf::flow`]).
    pub fn arc(&mut self, from: usize, to: usize, cap: i64, cost: f64) -> usize {
        let id = self.to.len();
        self.graph[from].push(id);
        self.to.push(to);
        self.res.push(cap);
        self.cost.push(cost);
        self.cap0.push(cap);
        self.graph[to].push(id + 1);
        self.to.push(from);
        self.res.push(0);
        self.cost.push(-cost);
        self.cap0.push(0);
        id
    }

    /// Exige que o nó `v` receba `d` de fluxo líquido (entrada menos saída).
    pub fn demand(&mut self, v: usize, d: i64) {
        self.demand[v] += d;
    }

    /// **PARTIDA A QUENTE** — põe fluxo num arco **de custo zero** antes de resolver.
    ///
    /// ⭐ **Isto não é um palpite que o solver depois corrige: é exato.** O que a
    /// pré-saturação e o Dijkstra exigem é que todo arco residual tenha custo
    /// `>= 0` (ver o doc do módulo). Num arco de custo **zero** ambos os sentidos
    /// custam zero, então **qualquer** quantidade inicial preserva a condição — e
    /// o ótimo encontrado é o mesmo.
    ///
    /// ⚠️ Ele muda o RELÓGIO, e muito: sem ele, as arestas de leque partem do piso
    /// `1` enquanto os arcos já pré-saturaram perto do alvo, e o desequilíbrio que
    /// sobra em cada nó é da ordem do **comprimento do lado**. O caminho-mais-curto
    /// sucessivo paga uma travessia por unidade desse desequilíbrio.
    ///
    /// # Panics
    /// Se o arco não tem custo zero — pôr fluxo à mão num arco com custo quebraria
    /// a otimalidade em silêncio, que é exatamente o erro que este `assert` recusa.
    pub fn preload(&mut self, id: usize, amount: i64) {
        assert!(
            self.cost[id] == 0.0,
            "partida a quente so' e' exata em arco de custo zero"
        );
        let f = amount.clamp(0, self.res[id]);
        self.res[id] -= f;
        self.res[id ^ 1] += f;
    }

    /// Quantos aumentos a resolução gastou — a unidade de esforço deste motor.
    #[must_use]
    pub fn augmentations(&self) -> usize {
        self.augmentations
    }

    /// O fluxo que passou pelo arco `id`.
    #[must_use]
    pub fn flow(&self, id: usize) -> i64 {
        self.cap0[id] - self.res[id]
    }

    /// **RESOLVE.** Devolve o custo total, mínimo.
    ///
    /// ⚠️ `max_augment` é o teto de **aumentos**, que é a unidade de esforço real
    /// deste motor: o relógio de uma resolução não é função do tamanho do grafo,
    /// é de quanto desequilíbrio há para rotear.
    ///
    /// # Errors
    /// [`McfError::Infeasible`] se as demandas não fecham;
    /// [`McfError::Exhausted`] se o teto de aumentos mordeu antes.
    pub fn solve(&mut self, max_augment: usize) -> Result<f64, McfError> {
        // 1. Satura os negativos — é o que deixa todo resíduo com custo >= 0.
        let n_real = self.to.len();
        for id in (0..n_real).step_by(2) {
            if self.cost[id] < 0.0 {
                let f = self.res[id];
                self.res[id] = 0;
                self.res[id + 1] += f;
            }
        }
        // 2. O desequilíbrio que sobra em cada nó.
        let mut bal = vec![0i64; self.demand.len()];
        for id in (0..n_real).step_by(2) {
            let f = self.flow(id);
            if f != 0 {
                bal[self.to[id]] += f;
                bal[self.to[id + 1]] -= f;
            }
        }
        for (v, d) in self.demand.iter().enumerate() {
            bal[v] -= d;
        }
        // 3. Fonte e sorvedouro artificiais.
        let (source, sink) = (self.source, self.sink);
        let mut need = 0i64;
        // ⚠️ **O nó com EXCESSO pendura-se na FONTE, não no sorvedouro** — e a
        // troca é silenciosa. A fonte injecta o excedente em `v`, e a conservação
        // no aumento obriga `v` a mandar exatamente essa quantidade embora pelos
        // arcos reais: é isso que **subtrai** o excesso em vez de o dobrar.
        // Invertido, o fluxo ainda fecha e ainda devolve `Ok` — e a resposta
        // satisfaz a conservação de outro problema.
        let terminals: Vec<(usize, i64)> = bal
            .iter()
            .enumerate()
            .filter(|(_, b)| **b != 0)
            .map(|(v, b)| (v, *b))
            .collect();
        for (v, b) in terminals {
            match b.cmp(&0) {
                Ordering::Greater => {
                    self.arc(source, v, b, 0.0);
                    need += b;
                }
                _ => {
                    self.arc(v, sink, -b, 0.0);
                }
            }
        }
        // 4. ⭐ **PRIMAL-DUAL**: Dijkstra fixa os potenciais e um **fluxo
        //    bloqueante** esgota, de uma vez, TODOS os caminhos de custo mínimo
        //    daquela fase — não um.
        //
        // ⚠️ **Um caminho por Dijkstra é o que fazia esta fase custar 250 ms.**
        // Medido em 2026-08-20: numa grelha de 512 arcos com alvos **dispersos**,
        // o desequilíbrio inicial somava milhares de unidades e cada travessia
        // levava umas dezenas — centenas de Dijkstras sobre o grafo inteiro. Com
        // alvos uniformes o mesmo layout custava **0 ms**, porque o desequilíbrio
        // era nulo. *O custo não era o tamanho do grafo, era o número de
        // aumentos* — e é exatamente esse número que o fluxo bloqueante colapsa.
        let mut pot = vec![0f64; self.demand.len()];
        let mut sent = 0i64;
        while sent < need {
            let (dist, _) = self.dijkstra(source, &pot);
            if !dist[sink].is_finite() {
                break;
            }
            for (v, p) in pot.iter_mut().enumerate() {
                if dist[v].is_finite() {
                    *p += dist[v];
                }
            }
            let before = sent;
            // Dinic sobre o subgrafo ADMISSÍVEL (custo reduzido zero). Os níveis
            // tornam-no acíclico — sem eles, um ciclo de custo reduzido zero faz
            // a busca em profundidade rodar para sempre.
            loop {
                let level = self.levels(source, &pot);
                if level[sink] < 0 {
                    break;
                }
                let mut iter = vec![0usize; self.demand.len()];
                loop {
                    let pushed = self.augment(source, sink, need - sent, &level, &pot, &mut iter);
                    if pushed == 0 {
                        break;
                    }
                    self.augmentations += 1;
                    if self.augmentations > max_augment {
                        return Err(McfError::Exhausted {
                            augmentations: self.augmentations,
                        });
                    }
                    sent += pushed;
                    if sent >= need {
                        break;
                    }
                }
                if sent >= need {
                    break;
                }
            }
            // Uma fase que não move nada não moverá na seguinte: o resto é
            // inalcançável, e quem decide é o `sent < need` lá em baixo.
            if sent == before {
                break;
            }
        }
        if sent < need {
            return Err(McfError::Infeasible {
                missing: need - sent,
            });
        }
        // 5. O custo, contado só sobre os arcos REAIS (os artificiais são 0).
        let total = (0..n_real)
            .step_by(2)
            .map(|id| self.cost[id] * self.flow(id) as f64)
            .sum();
        Ok(total)
    }

    /// **É este arco elegível para o fluxo bloqueante?** Só os de custo reduzido
    /// **zero** — são eles que compõem os caminhos de custo mínimo.
    ///
    /// ⚠️ **A tolerância é relativa e conservadora de propósito.** Deixar passar
    /// um arco de custo reduzido ligeiramente **positivo** paga custo a mais no
    /// resultado; deixar de fora um que era mesmo zero só custa mais uma fase de
    /// Dijkstra. Dos dois erros, só um estraga a resposta.
    fn admissible(&self, id: usize, u: usize, v: usize, pot: &[f64]) -> bool {
        let rc = self.cost[id] + pot[u] - pot[v];
        let scale = 1.0 + pot[u].abs().max(pot[v].abs());
        rc.abs() <= 1e-9 * scale
    }

    /// Níveis de BFS sobre o subgrafo admissível — o que o torna acíclico.
    fn levels(&self, from: usize, pot: &[f64]) -> Vec<i32> {
        let n = self.demand.len();
        let mut level = vec![-1i32; n];
        let mut queue = VecDeque::with_capacity(n);
        level[from] = 0;
        queue.push_back(from);
        while let Some(u) = queue.pop_front() {
            for &id in &self.graph[u] {
                let v = self.to[id];
                if self.res[id] <= 0 || level[v] >= 0 || !self.admissible(id, u, v, pot) {
                    continue;
                }
                level[v] = level[u] + 1;
                queue.push_back(v);
            }
        }
        level
    }

    /// Empurra UM caminho pelo grafo de níveis e devolve quanto passou.
    ///
    /// ⚠️ **Iterativo de propósito.** A versão recursiva é mais curta e desce
    /// tantos quadros quantos nós tiver o caminho; num layout de dezenas de
    /// milhares de nós isso é a pilha de uma thread de teste.
    fn augment(
        &mut self,
        from: usize,
        sink: usize,
        limit: i64,
        level: &[i32],
        pot: &[f64],
        iter: &mut [usize],
    ) -> i64 {
        if limit <= 0 {
            return 0;
        }
        let mut path: Vec<usize> = Vec::new();
        let mut u = from;
        loop {
            if u == sink {
                let push = path.iter().map(|&id| self.res[id]).fold(limit, i64::min);
                for &id in &path {
                    self.res[id] -= push;
                    self.res[id ^ 1] += push;
                }
                // ⚠️ Nada de recuar à mão: o arco saturado ficou com resíduo zero
                // e o `while` abaixo salta-o sozinho na próxima descida.
                return push;
            }
            let mut advanced = false;
            while iter[u] < self.graph[u].len() {
                let id = self.graph[u][iter[u]];
                let v = self.to[id];
                // ⚠️ **O nível NÃO substitui a admissibilidade.** Um arco caro
                // pode ligar dois níveis consecutivos por acaso — os níveis foram
                // construídos por OUTROS arcos. Medido em 2026-08-20: sem esta
                // segunda condição o fluxo escolhia a rota de custo 5 quando a de
                // custo 1 estava aberta, e o gate `the_cheaper_of_two_parallel_routes_wins`
                // devolvia 32 onde o ótimo é 24.
                if self.res[id] > 0 && level[v] == level[u] + 1 && self.admissible(id, u, v, pot) {
                    path.push(id);
                    u = v;
                    advanced = true;
                    break;
                }
                iter[u] += 1;
            }
            if !advanced {
                let Some(id) = path.pop() else {
                    return 0;
                };
                u = self.to[id ^ 1];
                iter[u] += 1;
            }
        }
    }

    /// Dijkstra sobre o resíduo, com custo reduzido.
    fn dijkstra(&self, from: usize, pot: &[f64]) -> (Vec<f64>, Vec<usize>) {
        let n = self.demand.len();
        let mut dist = vec![f64::INFINITY; n];
        let mut prev = vec![usize::MAX; n];
        let mut done = vec![false; n];
        dist[from] = 0.0;
        let mut heap = BinaryHeap::new();
        heap.push(Key(0.0, from));
        while let Some(Key(_, u)) = heap.pop() {
            if done[u] {
                continue;
            }
            done[u] = true;
            for &id in &self.graph[u] {
                if self.res[id] <= 0 {
                    continue;
                }
                let v = self.to[id];
                // ⚠️ O custo reduzido é `>= 0` por invariante (ver o doc do módulo);
                // um `max(0.0)` esconderia um erro de construção em vez de o expor.
                let w = self.cost[id] + pot[u] - pot[v];
                let nd = dist[u] + w;
                if nd < dist[v] {
                    dist[v] = nd;
                    prev[v] = id;
                    heap.push(Key(nd, v));
                }
            }
        }
        (dist, prev)
    }
}

/// Chave do heap: menor distância primeiro, empate pelo índice do nó.
///
/// ⚠️ **O desempate por índice não é cosmético.** Sem ele, dois nós à mesma
/// distância saem em ordem de heap, que depende da história de inserções — e a
/// mesma malha daria quantizações diferentes conforme a ordem de construção.
#[derive(Debug, Clone, Copy, PartialEq)]
struct Key(f64, usize);

impl Eq for Key {}

impl Ord for Key {
    fn cmp(&self, other: &Self) -> Ordering {
        // `BinaryHeap` é max-heap: invertemos para tirar o menor primeiro.
        other
            .0
            .total_cmp(&self.0)
            .then_with(|| other.1.cmp(&self.1))
    }
}

impl PartialOrd for Key {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
