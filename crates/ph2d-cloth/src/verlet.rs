//! ⭐⭐⭐ **O SOLVER DA REFERÊNCIA — Verlet por posições + relaxação de restrições de
//! distância** (a família Jakobsen 2001 / PBD 2006), portado clean-room da
//! [espec](../../../docs/3D/cleanroom/SPEC_cloth_brush.md) §5.
//!
//! # ⚠️ Por que esta lei existe ao lado do VBD
//!
//! A [auditoria de 05/09](../../../docs/3D/cloth/03_auditoria_2026-09-05.md) mediu
//! que nenhuma afinação do VBD+StVK produzia prega (§8-bis, §8-ter), e a espec do
//! comportamento do alvo mostrou que **o pincel do alvo não é um solver de pano
//! com um pincel em cima**: é um solver de restrições de distância que corre UM
//! passo por passo do pincel, com **um** passo de atraso entre a força e a
//! resposta das restrições — e é *isso* que dobra. O que este módulo porta é a
//! lei; a expressão é nossa.
//!
//! # As três coisas que decidem o desenho (espec §5)
//!
//! - **A relaxação vem ANTES da integração**, no mesmo passo: a força de um passo
//!   só é vista pelas restrições no passo seguinte.
//! - **Só a POSIÇÃO é corrigida** — a velocidade do passo seguinte sai da
//!   diferença de posições, logo as correcções das restrições entram nela.
//! - **Não há massa por vértice, limite de esticão, dobra própria nem
//!   sub-passos.** Uma massa global é um ganho inverso sobre um `dt` fixo.

use crate::V3;

/// Rigidez por restrição, por varredura (espec §5.2). ⚠️ `0,5` até 2020-10, subiu
/// para reduzir artefactos quando espécies de restrição diferentes disputam um
/// vértice (espec §9 nº 13).
pub const RIGIDEZ: f64 = 0.6;
/// Varreduras de relaxação por passo (espec §5.5): tempo × rigidez aparente.
pub const VARREDURAS: u32 = 5;
/// O passo de tempo FIXO (espec §5.5): é a escala do deslocamento por força —
/// `0,1/massa` a força máxima.
pub const DT: f64 = 0.01;

/// **Quem é o ponto B de uma restrição** — A é sempre um vértice (espec §3.2).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Alvo {
    /// Outro vértice: os dois lados movem-se (estrutural).
    Vertice(u32),
    /// A âncora de deformação do próprio vértice A (Grab · Snake Hook): só A
    /// se move, e a força é multiplicada por `σ_A`.
    Ancora,
    /// A memória de forma de A (corpo mole / plasticidade).
    Memoria,
    /// A posição de repouso do traço de A (o pino da fronteira).
    Repouso,
}

/// «A distância entre o ponto A e o ponto B tem de valer `ℓ`» (espec §3.2).
#[derive(Clone, Copy, Debug)]
pub struct Restricao {
    pub a: u32,
    pub b: Alvo,
    /// O comprimento de repouso.
    pub l: f64,
    /// A força `s` da restrição, fixa na criação.
    pub s: f64,
}

/// Os números do solver, com a omissão do código da referência (espec §8.1).
#[derive(Clone, Copy, Debug)]
pub struct Solver {
    /// Ganho inverso puro sobre `DT` — dobrar divide o deslocamento por força
    /// exactamente por dois (espec §5.4). Faixa `0,01..2`.
    pub massa: f64,
    /// A fracção de velocidade PERDIDA por passo (espec §5.3) — ⚠️ não é
    /// Rayleigh nem viscosidade. Omissão `0,01` ⇒ `99 %` de retenção.
    pub amortecimento: f64,
    /// *Soft Body Plasticity* `ρ` (espec §5.2): `0` = a memória segue o vértice e
    /// nunca o puxa; `1` = o vértice volta à memória.
    pub plasticidade: f64,
    /// Varreduras de relaxação por passo (espec §5.5): o recurso é tempo por
    /// passo × rigidez aparente. A referência corre [`VARREDURAS`].
    pub varreduras: u32,
}

impl Default for Solver {
    fn default() -> Self {
        Self {
            massa: 1.0,
            amortecimento: 0.01,
            plasticidade: 0.0,
            varreduras: VARREDURAS,
        }
    }
}

/// **A simulação de UM traço** (espec §6.3): nasce com o traço e morre com ele.
#[derive(Clone, Debug, Default)]
pub struct Verlet {
    /// Posição de simulação (relida da malha a cada passo — espec §1 fase 2).
    pub x: Vec<V3>,
    /// A posição corrigida do passo ANTERIOR (espec §5.4).
    pub x_prev: Vec<V3>,
    /// Aceleração acumulada neste passo; zerada depois de integrar.
    pub a: Vec<V3>,
    /// As posições de REPOUSO DO TRAÇO — de quando a simulação nasceu.
    pub repouso: Vec<V3>,
    /// O desvio de repouso por vértice do `Expand` (espec §4.5).
    pub tau: Vec<f64>,
    /// A âncora de deformação de cada vértice (o ponto B de [`Alvo::Ancora`]).
    pub ancora: Vec<V3>,
    /// O factor por passo `σ_v` das âncoras (espec §3.2, §4.3).
    pub sigma: Vec<f64>,
    /// A memória de forma (corpo mole), que nasce na posição de repouso.
    pub memoria: Vec<V3>,
    /// `φ_v = (1 − máscara) · auto-máscara · w(p⁰_v)` (espec §5.2).
    pub phi: Vec<f64>,
    /// A retenção de banda `w(p⁰_v)` da velocidade (espec §5.3).
    pub w_repouso: Vec<f64>,
    /// Quem é integrado neste passo (a «célula activa» — espec §2.1).
    pub activo: Vec<bool>,
    /// Este vértice já tem as suas restrições construídas? (uma vez por traço)
    pub construido: Vec<bool>,
    /// As restrições, na ORDEM de criação — e resolvidas nessa ordem.
    pub restricoes: Vec<Restricao>,
    /// Os pares não ordenados já criados (dedup global — espec §3.1).
    pares: std::collections::BTreeSet<(u32, u32)>,
    /// Quantos passos já simularam (o 1.º passo nunca simula — espec §1).
    pub passos_simulados: u32,
}

impl Verlet {
    /// Nasce em repouso: `x = x_prev = repouso`, sem restrições, ninguém activo.
    #[must_use]
    pub fn nascer(repouso: Vec<V3>) -> Self {
        let n = repouso.len();
        Self {
            x: repouso.clone(),
            x_prev: repouso.clone(),
            a: vec![[0.0; 3]; n],
            tau: vec![0.0; n],
            ancora: repouso.clone(),
            sigma: vec![0.0; n],
            memoria: repouso.clone(),
            phi: vec![1.0; n],
            w_repouso: vec![1.0; n],
            activo: vec![false; n],
            construido: vec![false; n],
            restricoes: Vec::new(),
            pares: std::collections::BTreeSet::new(),
            passos_simulados: 0,
            repouso,
        }
    }

    /// Quantos vértices.
    #[must_use]
    pub fn len(&self) -> usize {
        self.repouso.len()
    }

    /// Sem vértices?
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.repouso.is_empty()
    }

    /// **CONSTRÓI as restrições estruturais de `v`** (espec §3.1): uma de
    /// distância a cada vizinho do anel-1, e uma a cada PAR de vizinhos
    /// distintos — cada par não ordenado entra UMA vez por simulação.
    ///
    /// ⭐ É isto que faz a dobra sem modelo de dobra: numa grelha de quads, as
    /// 4 arestas + as 2 «diagonais longas» (`2h`) + as 4 diagonais (`√2·h`); numa
    /// malha de triângulos, as 6 arestas + os 9 que atravessam o anel.
    ///
    /// ⚠️ `ℓ` sai das posições de REPOUSO DO TRAÇO — nunca das actuais.
    /// **REABRE a construção** de `vs`: esquece o registo de duplicados e as
    /// marcas de «já construído», de modo que um [`Self::construir`] a seguir
    /// volte a acrescentar TODAS as restrições daqueles vértices.
    ///
    /// ⚠️ **Isto não é um utilitário — é a lei da área *Local*** (espec, a
    /// emenda Q8 de 06/09): ali a construção corre mais de uma vez e o registo
    /// de duplicados **vive uma construção só**, logo cada restrição acaba
    /// repetida na lista. Como a lista é resolvida **na ordem**, uma lista
    /// `[c₁..c_N, c₁..c_N]` percorrida `k` vezes é, bit a bit, a lista simples
    /// percorrida `2k` vezes: *a repetição não é desperdício, é a rigidez.*
    pub fn reabrir(&mut self, vs: &[u32]) {
        self.pares.clear();
        for &v in vs {
            self.construido[v as usize] = false;
        }
    }

    pub fn construir(&mut self, v: u32, anel: &[u32]) {
        let vi = v as usize;
        if self.construido[vi] {
            return;
        }
        self.construido[vi] = true;
        for &n in anel {
            self.par(v, n);
        }
        for (i, &a) in anel.iter().enumerate() {
            for &b in &anel[i + 1..] {
                self.par(a, b);
            }
        }
    }

    fn par(&mut self, a: u32, b: u32) {
        if a == b {
            return;
        }
        let chave = if a < b { (a, b) } else { (b, a) };
        if !self.pares.insert(chave) {
            return;
        }
        let (pa, pb) = (self.repouso[a as usize], self.repouso[b as usize]);
        self.restricoes.push(Restricao {
            a,
            b: Alvo::Vertice(b),
            l: dist(pa, pb),
            s: 1.0,
        });
    }

    /// Uma restrição de ÂNCORA para `v` (espec §3.2): `ℓ = 0`, força `s`.
    pub fn ancorar(&mut self, v: u32, s: f64) {
        self.restricoes.push(Restricao {
            a: v,
            b: Alvo::Ancora,
            l: 0.0,
            s,
        });
    }

    /// O PINO da fronteira para `v` (espec §2.3): à posição de repouso, força `1 − w`.
    pub fn pregar(&mut self, v: u32, s: f64) {
        self.restricoes.push(Restricao {
            a: v,
            b: Alvo::Repouso,
            l: 0.0,
            s,
        });
    }

    /// A restrição de corpo mole para `v` (espec §3.2): à memória de forma.
    pub fn amolecer(&mut self, v: u32) {
        self.restricoes.push(Restricao {
            a: v,
            b: Alvo::Memoria,
            l: 0.0,
            s: 1.0,
        });
    }

    /// **UM PASSO DE SIMULAÇÃO** (espec §5): as varreduras de relaxação sobre
    /// TODAS as restrições, depois — só para os vértices activos — a
    /// integração. `x` tem de ter sido relido da malha antes (fase 2 do §1) e
    /// `a` preenchido pelo gesto (fase 4).
    pub fn passo(&mut self, solver: &Solver) {
        let rho = solver.plasticidade.clamp(0.0, 1.0);
        for _ in 0..solver.varreduras {
            for k in 0..self.restricoes.len() {
                let r = self.restricoes[k];
                let ai = r.a as usize;
                let pa = self.x[ai];
                let pb = match r.b {
                    Alvo::Vertice(b) => self.x[b as usize],
                    Alvo::Ancora => self.ancora[ai],
                    Alvo::Memoria => self.memoria[ai],
                    Alvo::Repouso => self.repouso[ai],
                };
                let d = [pb[0] - pa[0], pb[1] - pa[1], pb[2] - pa[2]];
                let dist = norm(d);
                // Espec §5.2: com `D = 0` a correcção é zero (a divisão por
                // zero foi curada no alvo em 2020-03).
                if dist <= 0.0 {
                    continue;
                }
                let l = match r.b {
                    Alvo::Vertice(b) => r.l + (self.tau[ai] + self.tau[b as usize]) * 0.5,
                    _ => r.l,
                };
                let f = RIGIDEZ * (1.0 - l / dist);
                let h = [d[0] * f * 0.5, d[1] * f * 0.5, d[2] * f * 0.5];
                let phi_a = self.phi[ai];
                match r.b {
                    Alvo::Vertice(b) => {
                        let bi = b as usize;
                        let phi_b = self.phi[bi];
                        let ka = phi_a * r.s;
                        let kb = phi_b * r.s;
                        for (c, hc) in h.iter().enumerate() {
                            self.x[ai][c] += hc * ka;
                            self.x[bi][c] -= hc * kb;
                        }
                    }
                    Alvo::Ancora => {
                        let k = phi_a * r.s * self.sigma[ai];
                        for (c, hc) in h.iter().enumerate() {
                            self.x[ai][c] += hc * k;
                        }
                    }
                    Alvo::Repouso => {
                        let k = phi_a * r.s;
                        for (c, hc) in h.iter().enumerate() {
                            self.x[ai][c] += hc * k;
                        }
                    }
                    Alvo::Memoria => {
                        // Espec §5.2: o vértice vai `ρ` para a memória e a memória
                        // vem `1 − ρ` para o vértice.
                        let ka = phi_a * r.s * rho;
                        let km = phi_a * r.s * (1.0 - rho);
                        for (c, hc) in h.iter().enumerate() {
                            self.x[ai][c] += hc * ka;
                            self.memoria[ai][c] -= hc * km;
                        }
                    }
                }
            }
        }
        // ── integração (espec §5.4), por vértice ACTIVO ─────────────────────
        // ⚠️ A massa entra UMA vez, na conversão força → aceleração (espec §4.2:
        // `a += F/massa`), e não aqui: a 1.ª redacção dividia duas vezes e a
        // fixture `massa2_1passo` lia METADE do oráculo (`0,0248` contra `0,0496`).
        let ret = 1.0 - solver.amortecimento.clamp(0.0, 1.0);
        for i in 0..self.x.len() {
            if !self.activo[i] {
                continue;
            }
            let phi = self.phi[i];
            let v = [
                self.x[i][0] - self.x_prev[i][0],
                self.x[i][1] - self.x_prev[i][1],
                self.x[i][2] - self.x_prev[i][2],
            ];
            self.x_prev[i] = self.x[i];
            let kv = phi * ret * self.w_repouso[i];
            for (c, vc) in v.iter().enumerate() {
                self.x[i][c] += self.a[i][c] * phi * DT;
                self.x[i][c] += vc * kv;
            }
            self.a[i] = [0.0; 3];
        }
        self.passos_simulados += 1;
    }
}

/// `|b − a|`.
#[must_use]
pub fn dist(a: V3, b: V3) -> f64 {
    norm([b[0] - a[0], b[1] - a[1], b[2] - a[2]])
}

/// `|v|`.
#[must_use]
pub fn norm(v: V3) -> f64 {
    (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt()
}

/// `v/|v|`, ou zero se `v` for nulo.
#[must_use]
pub fn unit(v: V3) -> V3 {
    let n = norm(v);
    if n > 0.0 {
        [v[0] / n, v[1] / n, v[2] / n]
    } else {
        [0.0; 3]
    }
}
