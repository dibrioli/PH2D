//! **A multidão de UM tique**: a fotografia de todos os corpos que contam, a vizinhança por grelha, e
//! a velocidade segura de cada agente — EM SEQUÊNCIA, pela ordem da fotografia ([`Crowd::solve_all`]).

use std::collections::BTreeMap;

use crate::lines::{Line, Me, agent_line, wall_lines};
use crate::lp::{self, Regime};
use crate::v2::{V2, abs_sq, len, sub};
use crate::walls::Walls;

/// Um corpo na fotografia do tique.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Agent {
    pub pos: V2,
    /// A velocidade AGORA.
    pub vel: V2,
    /// A velocidade que ele QUER (a condução deu-a).
    pub pref: V2,
    pub radius: f64,
    pub max_speed: f64,
    /// ⚠️ **Ele também desvia?** Um agente que desvia faz METADE do desvio de cada encontro (o
    /// recíproco); um corpo que não desvia (o herói, um agente com o desvio desligado) é um obstáculo
    /// que ANDA — quem o encontra faz o desvio inteiro, e ele não é resolvido.
    pub avoids: bool,
    /// O corpo (índice na fotografia) de que ele NÃO se desvia: o ALVO de um perseguidor — desviar
    /// dele seria nunca lhe tocar.
    pub ignores: Option<u32>,
}

/// Os números do desvio — em segundos e metros.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Params {
    /// O horizonte contra outros corpos: as velocidades que batem em alguém antes disto são proibidas.
    pub time_horizon: f64,
    /// O horizonte contra as paredes.
    pub time_horizon_walls: f64,
    /// ⚠️ `None` = **a distância a partir da qual um vizinho PROVADAMENTE não restringe** (ver
    /// [`lossless_range`]) — cortar aí não muda resposta nenhuma. `Some(d)` = a do Godot (paridade).
    pub neighbor_dist: Option<f64>,
    /// `None` = todos os vizinhos ao alcance; `Some(n)` = os `n` mais perto (paridade com o Godot).
    pub max_neighbors: Option<usize>,
    /// ⭐ **Passar pela DIREITA** (a queixa Q7): `0` = o ORCA puro (paridade). Ver [`Crowd::solve`].
    pub side_bias: f64,
}

impl Params {
    /// **Os números do produto.**
    ///
    /// - `τ = 1 s` contra corpos: o de fábrica do Godot (`agent_get_time_horizon_agents`, medido).
    /// - `τ = 1 s` contra paredes: o de fábrica do Godot é `0`, que DESLIGA as paredes; com `1 s` as seis
    ///   cenas do banco passam sem que um centro chegue à parede (`tests/it/banco_de_cenarios.rs`).
    /// - os vizinhos ao alcance SEM PERDA ([`lossless_range`]), os [`MAX_NEIGHBORS`] mais perto.
    /// - [`SIDE_BIAS`].
    pub const PRODUCT: Self = Self {
        time_horizon: 1.0,
        time_horizon_walls: 1.0,
        neighbor_dist: None,
        max_neighbors: Some(MAX_NEIGHBORS),
        side_bias: SIDE_BIAS,
    };
}

/// ⚠️ **O tecto de vizinhos é de TEMPO DE QUADRO** — medido em `--release` (`tests/it/custo.rs`,
/// `load 2,6`), numa multidão DENSA a querer atravessar-se (o pior caso: todos se vêem):
///
/// | agentes | sem tecto | `6` | `10` | `16` | `40` |
/// |---|---|---|---|---|---|
/// | 100 | 0,21 ms | 0,059 | 0,067 | 0,075 | 0,124 |
/// | 1 000 | **9,5 ms** (400 vizinhos cada, 57 % do quadro) | 2,41 | 2,44 | 2,56 | 3,01 |
///
/// O banco de cenários dá o MESMO desfecho a `6`, `10` e `16` (nenhum aperto lá tem mais de oito). A
/// resposta numa multidão densa muda com o tecto e não tem joelho (`0,26`/`0,29`/`0,22`/`0,14 m/s` de
/// diferença ao «sem tecto» a `10`/`16`/`24`/`40`): ali quase todos estão no regime APERTADO, onde o
/// 3D depende de todos os semi-planos. ⇒ `10`, o do Godot — a paridade foi medida com ele. ⏳ O custo
/// que sobra a `1 000` é a VARRIDA dos candidatos (a célula da grelha é o alcance sem perda): uma
/// procura dos `k` mais perto por anéis de uma grelha fina tirá-lo-ia.
pub const MAX_NEIGHBORS: usize = 10;

/// ⭐ **O peso de passar pela direita**, MEDIDO no banco de cenários (em sequência; o quadro em que
/// todos chegaram, somado sobre as seis cenas):
///
/// | peso | frente | círculo de 8 | porta | soma |
/// |---|---|---|---|---|
/// | `0` | **nunca** (o empate do Godot) | — | — | — |
/// | `0,05` | 182 | 160 | 215 | 1 150 |
/// | `0,1` | 182 | 160 | 215 | 1 153 |
/// | `0,25` | 181 | 159 | 227 | 1 158 |
/// | `0,5` | 181 | 158 | 231 | 1 149 |
/// | `1` | 181 | 153 | **385** | 1 290 |
///
/// Qualquer peso desfaz o empate; de `0,05` a `0,5` o desfecho é plano, e a `1` a porta paga `+70 %`.
/// `0,25` é o meio (geométrico) do planalto — longe das duas pontas.
pub const SIDE_BIAS: f64 = 0.25;

/// ⭐ **O alcance SEM PERDA entre dois corpos.** Para lá dele o semi-plano do vizinho não corta
/// nenhuma velocidade que o agente possa escolher, logo ignorá-lo dá a MESMA resposta.
///
/// A prova (o artigo, §4): toda velocidade relativa no cone truncado tem módulo `≥ (d − R)/τ`, logo
/// o vector `u` até ele mede `≥ (d − R)/τ − (s_a + s_b)`; o semi-plano só proíbe ao agente mudar de
/// velocidade mais de `|u|/2` na direcção de `u`, e a mudança possível é `≤ 2·s_a`. Inactivo quando
/// `(d − R)/τ ≥ 5·s_a + s_b` — com `s` = o maior entre a velocidade máxima e a de agora.
///
/// ⚠️ `s` é o da FOTOGRAFIA: em [`Crowd::solve_all`] a velocidade dos anteriores muda, mas a nova cabe
/// sempre no disco da máxima, logo o alcance da fotografia continua a ser um majorante.
#[must_use]
pub fn lossless_range(a: &Agent, b: &Agent, tau: f64) -> f64 {
    a.radius + b.radius + tau * (5.0 * speed_bound(a) + speed_bound(b))
}

fn speed_bound(a: &Agent) -> f64 {
    a.max_speed.max(len(a.vel))
}

/// A fotografia do tique, com a grelha da vizinhança.
pub struct Crowd {
    agents: Vec<Agent>,
    /// `(raio, 5·s, s)` de cada um, da fotografia — o alcance sem perda sem uma raiz por par.
    bounds: Vec<(f64, f64, f64)>,
    params: Params,
    cell: f64,
    grid: BTreeMap<(i64, i64), Vec<u32>>,
    /// O buffer da vizinhança (reaproveitado; nenhum estado entre consultas).
    scratch: std::cell::RefCell<Vec<(f64, u32)>>,
}

impl Crowd {
    #[must_use]
    pub fn new(agents: Vec<Agent>, params: Params) -> Self {
        // A célula é o MAIOR alcance possível: os vizinhos de alguém estão sempre nas 3×3 à volta.
        let cell = match params.neighbor_dist {
            Some(d) => d,
            None => {
                let r = agents.iter().map(|a| a.radius).fold(0.0, f64::max);
                let s = agents.iter().map(speed_bound).fold(0.0, f64::max);
                2.0 * r + 6.0 * s * params.time_horizon
            }
        };
        let mut grid: BTreeMap<(i64, i64), Vec<u32>> = BTreeMap::new();
        if cell.is_finite() && cell > 0.0 {
            for (i, a) in agents.iter().enumerate() {
                grid.entry(cell_of(a.pos, cell)).or_default().push(i as u32);
            }
        }
        let tau = params.time_horizon;
        let bounds = agents
            .iter()
            .map(|a| {
                let s = speed_bound(a);
                (a.radius, 5.0 * s * tau, s * tau)
            })
            .collect();
        Self {
            agents,
            bounds,
            params,
            cell,
            grid,
            scratch: std::cell::RefCell::new(Vec::new()),
        }
    }

    #[must_use]
    pub fn agents(&self) -> &[Agent] {
        &self.agents
    }

    /// **Os vizinhos que contam para `i`**, do mais perto para o mais longe (o índice desempata).
    pub fn neighbors(&self, i: usize, out: &mut Vec<u32>) {
        out.clear();
        if !(self.cell.is_finite() && self.cell > 0.0) {
            return;
        }
        let me = &self.agents[i];
        let (ri, ki, _) = self.bounds[i];
        let (cx, cy) = cell_of(me.pos, self.cell);
        let mut found = self.scratch.borrow_mut();
        found.clear();
        for gx in cx - 1..=cx + 1 {
            for gy in cy - 1..=cy + 1 {
                let Some(list) = self.grid.get(&(gx, gy)) else {
                    continue;
                };
                for &j in list {
                    if j as usize == i || me.ignores == Some(j) {
                        continue;
                    }
                    let other = &self.agents[j as usize];
                    let range = match self.params.neighbor_dist {
                        Some(d) => d,
                        // = [`lossless_range`], com os termos da fotografia.
                        None => {
                            let (rj, _, sj) = self.bounds[j as usize];
                            ri + rj + ki + sj
                        }
                    };
                    let d = abs_sq(sub(other.pos, me.pos));
                    if d < range * range {
                        found.push((d, j));
                    }
                }
            }
        }
        // A ordem é TOTAL (o índice desempata), logo a ordenação instável dá a mesma lista — e com um
        // tecto, a selecção dos `n` mais perto (linear) antes de ordenar só esses.
        let ordem = |x: &(f64, u32), y: &(f64, u32)| x.0.total_cmp(&y.0).then(x.1.cmp(&y.1));
        if let Some(n) = self.params.max_neighbors
            && found.len() > n
        {
            if n == 0 {
                found.clear();
            } else {
                found.select_nth_unstable_by(n - 1, ordem);
                found.truncate(n);
            }
        }
        found.sort_unstable_by(ordem);
        out.extend(found.iter().map(|&(_, j)| j));
    }

    /// ⭐⭐ **Todos os agentes, EM SEQUÊNCIA pela ordem da fotografia** — cada um resolve com a
    /// velocidade NOVA dos anteriores (as posições ficam as da fotografia: quem anda é o mover, depois).
    /// Devolve a velocidade de cada corpo (a de agora, para quem não desvia).
    ///
    /// ⛔ **A fotografia comum (Jacobi, o artigo) foi MEDIDA e recusada:** com ela o círculo de oito
    /// fica preso num anel à volta do centro com QUALQUER peso de lado — cada um encostado aos dois
    /// vizinhos, e o empurrão de um anula o do outro. Em sequência a simetria parte-se pela ordem (é o
    /// que o Godot faz — medido na paridade) e as seis cenas do banco passam. A ordem é a das ENTIDADES:
    /// a mesma nos três sistemas e num replay.
    pub fn solve_all<'w>(
        &mut self,
        walls: impl Fn(usize) -> Option<(&'w Walls, f64)>,
        dt: f64,
    ) -> Vec<V2> {
        self.solve_all_why(walls, dt)
            .into_iter()
            .map(|(v, _)| v)
            .collect()
    }

    /// [`Self::solve_all`], e para cada agente se foi OUTRO corpo a cortar-lhe a velocidade pedida
    /// (o semi-plano de um vizinho a excluí-la) — as paredes sozinhas não contam. É a pergunta da
    /// leitura *«a dar passagem»*: só a velocidade não a responde (sozinho, a quina também trava).
    pub fn solve_all_why<'w>(
        &mut self,
        walls: impl Fn(usize) -> Option<(&'w Walls, f64)>,
        dt: f64,
    ) -> Vec<(V2, bool)> {
        let mut out = Vec::with_capacity(self.agents.len());
        for i in 0..self.agents.len() {
            let (v, _, outros) = self.resolve(i, walls(i), dt);
            self.agents[i].vel = v;
            out.push((v, outros));
        }
        out
    }

    /// **A velocidade segura do agente `i`**, contra os vizinhos e — se houver — as paredes `walls`,
    /// contra as quais ele tem o raio `wall_radius` (`0` quando elas já são a malha recuada pelo raio
    /// dele). Um corpo que não desvia devolve a velocidade de agora.
    #[must_use]
    pub fn velocity(&self, i: usize, walls: Option<(&Walls, f64)>, dt: f64) -> V2 {
        self.solve(i, walls, dt).0
    }

    /// [`Self::velocity`], e por onde ela saiu.
    #[must_use]
    pub fn solve(&self, i: usize, walls: Option<(&Walls, f64)>, dt: f64) -> (V2, Regime) {
        let (v, r, _) = self.resolve(i, walls, dt);
        (v, r)
    }

    /// [`Self::solve`], e se um semi-plano de VIZINHO exclui a velocidade pedida.
    fn resolve(&self, i: usize, walls: Option<(&Walls, f64)>, dt: f64) -> (V2, Regime, bool) {
        let a = self.agents[i];
        if !a.avoids {
            return (a.vel, Regime::Free, false);
        }
        let me = Me {
            pos: a.pos,
            vel: a.vel,
            radius: a.radius,
        };
        let mut lines: Vec<Line> = Vec::new();
        if let Some((w, r)) = walls
            && self.params.time_horizon_walls > 0.0
        {
            let mut near = Vec::new();
            w.near(
                a.pos,
                self.params.time_horizon_walls * a.max_speed + r,
                &mut near,
            );
            wall_lines(&me, r, w, &near, self.params.time_horizon_walls, &mut lines);
        }
        let n_walls = lines.len();
        let mut nb = Vec::new();
        self.neighbors(i, &mut nb);
        for j in nb {
            let o = &self.agents[j as usize];
            let share = if o.avoids { 0.5 } else { 1.0 };
            lines.push(agent_line(
                &me,
                o.pos,
                o.vel,
                o.radius,
                share,
                self.params.time_horizon,
                dt,
            ));
        }
        let outros = lines[n_walls..].iter().any(|l| lp::violates(l, a.pref));
        let (v, regime) = lp::solve(&lines, n_walls, a.max_speed, a.pref);
        if regime == Regime::Free || self.params.side_bias <= 0.0 {
            return (v, regime, outros);
        }
        // ⭐ **O empate SIMÉTRICO** (Q7, medido no Godot: dois frente a frente no MESMO eixo param a
        // `25 px` um do outro e ficam): o semi-plano de um vizinho alinhado é perpendicular ao
        // caminho, e a melhor resposta é só TRAVAR — sem uma componente de lado, ninguém sai do eixo.
        // A cura da indústria é a preferência de lado (o `weightSide` do DetourCrowd): a velocidade
        // pedida ganha uma componente à DIREITA, do tamanho do que o aperto TIROU — sozinho, nada
        // muda; travado a fundo, o pedido vira para a direita. Os dois do empate viram cada um para
        // a SUA direita, e cruzam-se.
        let tirado = len(sub(a.pref, v));
        let p = len(a.pref);
        if p <= 0.0 {
            return (v, regime, outros);
        }
        let direita = [a.pref[1] / p, -a.pref[0] / p];
        let pedido = [
            a.pref[0] + direita[0] * tirado * self.params.side_bias,
            a.pref[1] + direita[1] * tirado * self.params.side_bias,
        ];
        let (v, regime) = lp::solve(&lines, n_walls, a.max_speed, pedido);
        (v, regime, outros)
    }
}

fn cell_of(p: V2, cell: f64) -> (i64, i64) {
    ((p[0] / cell).floor() as i64, (p[1] / cell).floor() as i64)
}
