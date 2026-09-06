//! ⭐⭐⭐ **O GESTO da referência** — a área simulada, a banda graduada, o factor por
//! vértice e os oito modos, portados clean-room da
//! [espec](../../../docs/3D/cleanroom/SPEC_cloth_brush.md) §2 e §4.
//!
//! ⚠️ **Este módulo não sabe o que é uma malha nem um pincel.** Recebe posições,
//! normais, o anel-1 de cada vértice e o cursor; devolve, por passo, o que o
//! solver precisa (forças, âncoras, desvios de repouso) e corre-o. É o adaptador
//! do `ph2d-sculpt3d` que traduz `Brush`/`Dab`/`Mesh` para isto.

use crate::V3;
use crate::verlet::{Solver, Verlet, dist, norm, unit};

/// Os oito tipos de deformação (espec §4).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Modo {
    /// Força na direcção UNITÁRIA do movimento do cursor — a mesma para todos.
    Arrastar,
    /// Força para DENTRO, ao longo da normal da área, com magnitude `2R`.
    Empurrar,
    /// Força unitária do vértice PARA o cursor.
    ApertarPonto,
    /// Força para a LINHA do traço (só as componentes perpendiculares).
    ApertarLinha,
    /// Força ao longo da normal do vértice, para fora.
    Inflar,
    /// Âncora `p⁰ + δ_total · f`, pegada congelada na malha de partida.
    Agarrar,
    /// Âncora `x + δ_incremental · f`, re-pegada a cada passo.
    Gancho,
    /// Desvio do comprimento de repouso, `τ += 0,01 · f` por passo.
    Expandir,
}

/// A área simulada (espec §2.1).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Area {
    /// Esfera de raio `R₀(1+L)` centrada na localização inicial do traço.
    Local,
    /// Tudo; `w ≡ 1`.
    Global,
    /// Esfera de raio `R(1+L)` centrada no cursor actual.
    Dinamica,
}

/// A forma espacial do peso da força (espec §4.4).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FalloffForca {
    Radial,
    Plano,
}

/// A curva de falloff do pincel (espec §4.1). Só as que o oráculo usa.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Curva {
    /// `3u² − 2u³`, `u = 1 − d/R`.
    Suave,
    /// `u²`.
    Aguda,
    /// `1`.
    Constante,
}

impl Curva {
    /// O peso em `[0, 1]` de uma distância `d` num raio `r`.
    #[must_use]
    pub fn peso(self, d: f64, r: f64) -> f64 {
        if r <= 0.0 || d >= r {
            return 0.0;
        }
        let u = 1.0 - d / r;
        match self {
            Self::Suave => 3.0 * u * u - 2.0 * u * u * u,
            Self::Aguda => u * u,
            Self::Constante => 1.0,
        }
    }
}

/// Os controlos do pincel de tecido (espec §8.1 — as omissões do CÓDIGO).
#[derive(Clone, Copy, Debug)]
pub struct Pincel {
    pub modo: Modo,
    pub area: Area,
    pub falloff_forca: FalloffForca,
    pub curva: Curva,
    /// O raio, em unidades de objecto.
    pub raio: f64,
    /// *Strength* em `[0,1]` — ⚠️ o alvo eleva ao QUADRADO nos modos de força.
    pub forca: f64,
    /// A dureza `h ∈ [0,1]`: distância `< h·R` ⇒ peso `1`.
    pub dureza: f64,
    /// *Simulation Limit* `L` (omissão `2,5`).
    pub limite: f64,
    /// *Simulation Falloff* `F` (omissão `0,75`).
    pub banda: f64,
    /// *Pin Simulation Boundary* (só *Local*; omissão `false`).
    pub pino: bool,
    /// `flip = ±1` (Add/Subtract).
    pub flip: f64,
    pub solver: Solver,
    /// ⚠️ **Experimento de paridade** — a escala do `limite` que o leitor de
    /// banda das RESTRIÇÕES (`φ`) usa: `1` = a espec §2.2 (o mesmo `R(1+L)` da
    /// força). Medido em 2026-09-06 porque nenhum conjunto de restrições
    /// reproduz Local e Global ao mesmo tempo.
    pub escala_phi: f64,
    /// Idem para o leitor da RETENÇÃO de velocidade.
    pub escala_retencao: f64,
}

impl Default for Pincel {
    fn default() -> Self {
        Self {
            modo: Modo::Arrastar,
            area: Area::Local,
            falloff_forca: FalloffForca::Radial,
            curva: Curva::Suave,
            raio: 0.35,
            forca: 1.0,
            dureza: 0.0,
            limite: 2.5,
            banda: 0.75,
            pino: false,
            flip: 1.0,
            solver: Solver::default(),
            escala_phi: 1.0,
            escala_retencao: 1.0,
        }
    }
}

/// **O peso de BANDA** `w(p)` (espec §2.2): `1` dentro do início, `0` fora do
/// limite, *smoothstep* entre os dois.
#[must_use]
pub fn banda(p: V3, c: V3, r: f64, limite: f64, falloff: f64) -> f64 {
    let fim = r * (1.0 + limite);
    let inicio = r * (1.0 + limite * falloff);
    let d = dist(p, c);
    if d < inicio {
        1.0
    } else if d > fim || fim <= inicio {
        0.0
    } else {
        let t = 1.0 - (d - inicio) / (fim - inicio);
        3.0 * t * t - 2.0 * t * t * t
    }
}

/// O que um passo do pincel recebe de fora (espec §4).
pub struct Passo<'a> {
    /// A localização do cursor neste passo (re-apanhada na superfície nos modos
    /// de força; no pen-down para o Grab; `c + δ` para o Gancho).
    pub cursor: V3,
    /// O delta de agarrar `δ` (espec §4.3): incremental para todos os modos,
    /// TOTAL para o Grab.
    pub delta: V3,
    /// O cursor não se mexeu no ecrã? ⇒ sem forças neste passo.
    pub parado: bool,
    /// A normal da ÁREA sob o pincel (espec §4.4).
    pub normal_area: V3,
    /// As normais ACTUAIS por vértice (o Inflate lê-as).
    pub normais: &'a [V3],
    /// Pressão em `[0,1]`.
    pub pressao: f64,
}

/// **O pincel de tecido de UM traço**: a sessão, o centro da área e a simulação.
#[derive(Clone, Debug)]
pub struct PincelTecido {
    pub pincel: Pincel,
    pub sim: Verlet,
    /// A localização INICIAL do traço (o centro da área *Local*; a origem do
    /// delta total do Grab).
    pub inicio: V3,
    /// `R₀` — o raio no 1.º passo (a área *Local* ignora a pressão).
    pub raio0: f64,
    /// O cursor do passo anterior (para o delta incremental).
    pub anterior: V3,
    /// Quem está na área simulada neste passo.
    pub dentro: Vec<u32>,
    /// A máscara por vértice em `[0,1]` (`1` = imóvel); vazia = sem máscara.
    pub mascara: Vec<f64>,
    /// Ainda não houve passo nenhum? (o 1.º constrói e não simula)
    pub primeiro: bool,
}

impl PincelTecido {
    /// O pen-down: a simulação nasce nas posições ACTUAIS, que passam a ser o
    /// repouso do traço.
    #[must_use]
    pub fn pen_down(pincel: Pincel, posicoes: &[V3], cursor: V3) -> Self {
        Self {
            raio0: pincel.raio,
            pincel,
            sim: Verlet::nascer(posicoes.to_vec()),
            inicio: cursor,
            anterior: cursor,
            dentro: Vec::new(),
            mascara: Vec::new(),
            primeiro: true,
        }
    }

    /// O centro e o raio da ÁREA neste passo (espec §2.1).
    #[must_use]
    pub fn centro_da_area(&self, cursor: V3) -> (V3, f64) {
        match self.pincel.area {
            Area::Local | Area::Global => (self.inicio, self.raio0),
            Area::Dinamica => (cursor, self.pincel.raio),
        }
    }

    /// O peso de banda de um ponto (espec §2.2), `1` em *Global*.
    #[must_use]
    pub fn w(&self, p: V3, cursor: V3) -> f64 {
        self.w_com(p, cursor, 1.0)
    }

    /// O peso de banda com o `limite` ESCALADO — os três leitores da espec §2.2
    /// (força · `φ` · retenção) partilham a forma e, por experimento, o alcance.
    #[must_use]
    pub fn w_com(&self, p: V3, cursor: V3, escala: f64) -> f64 {
        if self.pincel.area == Area::Global {
            return 1.0;
        }
        let (c, r) = self.centro_da_area(cursor);
        banda(p, c, r, self.pincel.limite * escala, self.pincel.banda)
    }

    fn mascara_de(&self, v: usize) -> f64 {
        self.mascara.get(v).copied().unwrap_or(0.0)
    }

    /// **UM PASSO DO PINCEL** (espec §1, as cinco fases). `posicoes` são as
    /// posições ACTUAIS da malha; `anel(v)` o anel-1. Devolve `true` se simulou
    /// (o 1.º passo nunca simula) — e nesse caso `sim.x` traz as posições novas
    /// dos vértices ACTIVOS, que o chamador escreve na malha.
    pub fn passo(
        &mut self,
        posicoes: &[V3],
        anel: &dyn Fn(u32) -> Vec<u32>,
        passo: &Passo<'_>,
    ) -> bool {
        let n = posicoes.len();
        debug_assert_eq!(n, self.sim.len());
        let cursor = passo.cursor;
        // fase 1 — a área deste passo, e as restrições de quem entra nela
        let (c, r) = self.centro_da_area(cursor);
        let alcance = r * (1.0 + self.pincel.limite);
        self.dentro.clear();
        for (v, p) in posicoes.iter().enumerate() {
            let dentro = match self.pincel.area {
                Area::Global => true,
                // ⚠️ Local: o teste da construção é sobre o REPOUSO (espec §3.1).
                Area::Local => dist(self.sim.repouso[v], c) < alcance,
                Area::Dinamica => dist(*p, c) < alcance,
            };
            if dentro {
                self.dentro.push(u32::try_from(v).unwrap_or(u32::MAX));
            }
        }
        let novos: Vec<u32> = self
            .dentro
            .iter()
            .copied()
            .filter(|v| !self.sim.construido[*v as usize])
            .collect();
        for &v in &novos {
            let vizinhos = anel(v);
            self.sim.construir(v, &vizinhos);
            let vi = v as usize;
            // Espec §2.3: o pino da fronteira, só em Local, força `1 − w`.
            if self.pincel.pino && self.pincel.area == Area::Local {
                let wv = banda(
                    self.sim.repouso[vi],
                    cursor,
                    self.raio0,
                    self.pincel.limite,
                    self.pincel.banda,
                );
                if wv < 1.0 {
                    self.sim.pregar(v, 1.0 - wv);
                }
            }
            if self.pincel.solver.plasticidade > 0.0 {
                self.sim.amolecer(v);
            }
            // As âncoras de deformação nascem com o vértice (espec §4.3).
            match self.pincel.modo {
                Modo::Agarrar => {
                    let d0 = dist(self.sim.repouso[vi], self.inicio);
                    match self.pincel.falloff_forca {
                        FalloffForca::Radial => {
                            if d0 < self.raio0 {
                                let s = 0.1 * self.pincel.curva.peso(d0, self.raio0);
                                self.sim.ancorar(v, s);
                                self.sim.sigma[vi] = 1.0;
                            }
                        }
                        FalloffForca::Plano => {
                            self.sim.ancorar(v, 0.1);
                        }
                    }
                }
                Modo::Gancho => self.sim.ancorar(v, 0.35),
                _ => {}
            }
        }
        // O 1.º passo de uma passagem nunca simula (espec §1 fase 0): o alvo
        // precisa de um deslocamento do cursor válido para orientar a ponta, e
        // no 1.º passo ele é zero.
        if self.primeiro {
            self.primeiro = false;
            self.anterior = cursor;
            return false;
        }
        // fase 2 — guardar o estado: x ← malha
        self.sim.x.copy_from_slice(posicoes);
        // fase 3 — activar
        self.sim.activo.fill(false);
        for &v in &self.dentro {
            self.sim.activo[v as usize] = true;
        }
        // φ e a retenção de banda são lidos no REPOUSO do traço (espec §2.2).
        for v in 0..n {
            let p0 = self.sim.repouso[v];
            self.sim.phi[v] =
                (1.0 - self.mascara_de(v)) * self.w_com(p0, cursor, self.pincel.escala_phi);
            self.sim.w_repouso[v] = self.w_com(p0, cursor, self.pincel.escala_retencao);
        }
        // fase 4 — o gesto
        self.gesto(posicoes, passo);
        // fase 5 — o passo de simulação
        self.sim.passo(&self.pincel.solver);
        self.anterior = cursor;
        true
    }

    /// **O factor por vértice `f`** (espec §4.1), sem o `B`: máscara · banda ·
    /// corte no raio · curva com dureza.
    fn factor(&self, p: V3, cursor: V3, r: f64, d: f64) -> f64 {
        if d >= r {
            return 0.0;
        }
        let h = self.pincel.dureza.clamp(0.0, 1.0);
        let d_remap = if h > 0.0 {
            if d < h * r {
                0.0
            } else {
                (d - h * r) / (1.0 - h)
            }
        } else {
            d
        };
        self.w(p, cursor) * self.pincel.curva.peso(d_remap, r)
    }

    /// A distância que a curva lê (espec §4.1): esférica ao cursor, ou ao PLANO
    /// de falloff (normal = direcção do movimento, pelo centro da área).
    fn distancia(&self, p: V3, cursor: V3, delta_u: V3) -> f64 {
        match self.pincel.falloff_forca {
            FalloffForca::Radial => dist(p, cursor),
            FalloffForca::Plano => {
                let q = [p[0] - cursor[0], p[1] - cursor[1], p[2] - cursor[2]];
                (q[0] * delta_u[0] + q[1] * delta_u[1] + q[2] * delta_u[2]).abs()
            }
        }
    }

    /// Fase 4 (espec §4): forças → `a`; âncoras; desvios de repouso.
    fn gesto(&mut self, posicoes: &[V3], passo: &Passo<'_>) {
        let cursor = passo.cursor;
        let r = self.pincel.raio;
        let delta_u = unit(passo.delta);
        let alpha = self.pincel.forca * self.pincel.forca;
        let flip = self.pincel.flip;
        let pressao = passo.pressao.clamp(0.0, 1.0);
        let n_area = unit(passo.normal_area);
        // O referencial local do traço (espec §4.4).
        let x_hat = unit(cruz(n_area, delta_u));
        let dentro = self.dentro.clone();
        match self.pincel.modo {
            Modo::Agarrar => {
                // Espec §4.3: âncora = p⁰ + δ_total · f, com f medido na malha
                // de PARTIDA e o raio inicial; σ = 1 (radial) ou clamp(f) (plano).
                for &v in &dentro {
                    let vi = v as usize;
                    let p0 = self.sim.repouso[vi];
                    let d = self.distancia(p0, self.inicio, delta_u);
                    let f = self.factor(p0, self.inicio, self.raio0, d) * self.pincel.forca;
                    let m = 1.0 - self.mascara_de(vi);
                    let f = f * m;
                    self.sim.ancora[vi] = [
                        p0[0] + passo.delta[0] * f,
                        p0[1] + passo.delta[1] * f,
                        p0[2] + passo.delta[2] * f,
                    ];
                    if self.pincel.falloff_forca == FalloffForca::Plano {
                        self.sim.sigma[vi] = f.clamp(0.0, 1.0);
                    }
                }
            }
            Modo::Gancho => {
                // Espec §4.3: âncora = x + δ · f, e σ = f reescrito a cada passo —
                // ZERO fora do pincel.
                let b = self.pincel.forca * pressao;
                for &v in &dentro {
                    let vi = v as usize;
                    let p = posicoes[vi];
                    let d = self.distancia(p, cursor, delta_u);
                    let f = self.factor(p, cursor, r, d) * b * (1.0 - self.mascara_de(vi));
                    self.sim.ancora[vi] = [
                        p[0] + passo.delta[0] * f,
                        p[1] + passo.delta[1] * f,
                        p[2] + passo.delta[2] * f,
                    ];
                    self.sim.sigma[vi] = f;
                }
            }
            Modo::Expandir => {
                if passo.parado {
                    return;
                }
                let b = 0.1 * alpha * flip * pressao;
                for &v in &dentro {
                    let vi = v as usize;
                    let p = posicoes[vi];
                    let d = self.distancia(p, cursor, delta_u);
                    let f = self.factor(p, cursor, r, d) * b * (1.0 - self.mascara_de(vi));
                    self.sim.tau[vi] += 0.01 * f;
                }
            }
            Modo::Arrastar
            | Modo::Empurrar
            | Modo::ApertarPonto
            | Modo::ApertarLinha
            | Modo::Inflar => {
                // Espec §4.2: nenhuma força num passo em que o cursor não se mexeu.
                if passo.parado {
                    return;
                }
                let b = 10.0 * alpha * flip * pressao;
                for &v in &dentro {
                    let vi = v as usize;
                    let p = posicoes[vi];
                    let d = self.distancia(p, cursor, delta_u);
                    let f = self.factor(p, cursor, r, d) * b * (1.0 - self.mascara_de(vi));
                    if f == 0.0 {
                        continue;
                    }
                    let u: V3 = match self.pincel.modo {
                        Modo::Arrastar => delta_u,
                        Modo::Empurrar => [
                            -n_area[0] * 2.0 * r,
                            -n_area[1] * 2.0 * r,
                            -n_area[2] * 2.0 * r,
                        ],
                        Modo::ApertarPonto => match self.pincel.falloff_forca {
                            FalloffForca::Radial => {
                                unit([cursor[0] - p[0], cursor[1] - p[1], cursor[2] - p[2]])
                            }
                            // Espec §4.2: com falloff de plano o alvo é o PLANO.
                            FalloffForca::Plano => {
                                let q = [p[0] - cursor[0], p[1] - cursor[1], p[2] - cursor[2]];
                                let s = q[0] * delta_u[0] + q[1] * delta_u[1] + q[2] * delta_u[2];
                                let sg = if s > 0.0 { -1.0 } else { 1.0 };
                                [delta_u[0] * sg, delta_u[1] * sg, delta_u[2] * sg]
                            }
                        },
                        Modo::ApertarLinha => {
                            let para = unit([cursor[0] - p[0], cursor[1] - p[1], cursor[2] - p[2]]);
                            let cx = para[0] * x_hat[0] + para[1] * x_hat[1] + para[2] * x_hat[2];
                            let cz =
                                para[0] * n_area[0] + para[1] * n_area[1] + para[2] * n_area[2];
                            [
                                x_hat[0] * cx + n_area[0] * cz,
                                x_hat[1] * cx + n_area[1] * cz,
                                x_hat[2] * cx + n_area[2] * cz,
                            ]
                        }
                        Modo::Inflar => unit(passo.normais[vi]),
                        _ => [0.0; 3],
                    };
                    let inv_m = 1.0 / self.pincel.solver.massa.max(1e-9);
                    for (c, uc) in u.iter().enumerate() {
                        self.sim.a[vi][c] += f * uc * inv_m;
                    }
                }
            }
        }
    }
}

/// `a × b`.
#[must_use]
pub fn cruz(a: V3, b: V3) -> V3 {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

/// Sem uso fora dos testes de paridade, mas é a régua: `|v|`.
#[must_use]
pub fn comprimento(v: V3) -> f64 {
    norm(v)
}
