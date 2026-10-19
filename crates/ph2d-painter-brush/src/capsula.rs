//! ⭐ **O dab-CÁPSULA** (Continuous Spacing, doc 31 §5): o dab leva o passo do caminho
//! ([`crate::Dab::step`], o centro menos o do dab anterior) e a sua ponta passa a ser a do traço DENSO —
//! os `n` sub-dabs que um Spacing de [`sub_passo_px`] poria de volta até ao centro anterior,
//! `off_k = −D·k/n` (`k < n`; `k = 0` é o próprio dab), compostos no píxel UMA vez.
//!
//! Desloca a ponta INTEIRA (a queda, a pegada e o Shape juntos), então vale para qualquer ponta e em
//! todo meio que a pergunte aqui. A composição é a do próprio carimbo: `n` passagens da mesma cor por
//! cima uma da outra, com opacidades `c·x_k`, são UMA passagem com `1 − Π(1 − c·x_k)` — a [`Lei::Uniao`].
//! Sob uma rampa do Shape (Texture/Falloff Color) a cor de cada sub-dab é OUTRA: compõe-se de frente para
//! trás sobre os mesmos sub-dabs (`dab/cor_capsula.rs`, lista 26 A9); o roteador só manda a cápsula com
//! rampa ao núcleo por píxel (`stamp_dabs_ramped`), o único que a honra.

use std::cell::Cell;

/// O tecto de sub-dabs por dab: o passo de um texel do ladrilho das cerdas dá a volta ao período
/// ([`ph2d_wet_paint::brush::BRISTLE_SIZE`] = 128) — passos maiores só repetem posições já lidas.
pub const MAX_SUB: u32 = 128;

/// O dab-cápsula: o passo, o número de sub-dabs e o ESTADO do dab de trás relativo ao da frente.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Capsula {
    passo: [f32; 2],
    n: u32,
    estado: Estado,
}

/// O dab de trás visto do da frente (prompt 40): o sub-dab `k` leva `lerp(frente, trás, k/n)` de cada um.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Estado {
    /// O raio de trás sobre o da frente.
    pub raio: f32,
    /// A cobertura (a pressão) de trás sobre a da frente.
    pub cobertura: f32,
    /// O rotor da pegada de trás relativo ao da frente (`[1, 0]` = a mesma rotação).
    pub rotor: [f32; 2],
    /// O `arc_len` de trás menos o da frente, px (o Flow).
    pub arco: f32,
}

/// O sub-dab `k`: o desvio do centro e o seu estado relativo ao da frente.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Sub {
    pub off: [f32; 2],
    /// O raio do sub-dab sobre o do dab.
    pub escala: f32,
    /// A cobertura do sub-dab sobre a do dab.
    pub cobertura: f32,
    /// A rotação a compor na pegada do dab; `None` = a mesma.
    pub rotor: Option<[f32; 2]>,
    /// O `arc_len` do sub-dab menos o do dab.
    pub arco: f32,
}

impl Capsula {
    /// A cápsula de um dab com passo `passo` (px) e sub-passo `sub_px`, ou `None` quando ela é o próprio
    /// dab (`n ≤ 1`: sem passo, ou mais curto que o sub-passo).
    #[must_use]
    pub fn new(passo: [f32; 2], sub_px: f32) -> Option<Self> {
        let m = passo[0].abs().max(passo[1].abs());
        if !m.is_finite() || !sub_px.is_finite() || sub_px <= 0.0 {
            return None;
        }
        let n = (m / sub_px).ceil().min(MAX_SUB as f32) as u32;
        // A lei velha (só o centro) não andava o arco; o produto anda-o pela corda.
        let arco = if so_o_centro() {
            0.0
        } else {
            -passo[0].hypot(passo[1])
        };
        let estado = Estado {
            raio: 1.0,
            cobertura: 1.0,
            rotor: [1.0, 0.0],
            arco,
        };
        (n >= 2).then_some(Self { passo, n, estado })
    }

    /// A mesma cápsula a interpolar até o estado `e` do dab de trás. Um rotor a menos de `1e-6` da
    /// identidade é a identidade (a mesma rotação dos dois lados, ao bit).
    #[must_use]
    pub fn com_estado(self, mut e: Estado) -> Self {
        if so_o_centro() {
            return self;
        }
        let l = e.rotor[0].hypot(e.rotor[1]);
        e.rotor = if !l.is_finite() || l < 1e-6 || (e.rotor[1].abs() <= 1e-6 && e.rotor[0] > 0.0) {
            [1.0, 0.0]
        } else {
            [e.rotor[0] / l, e.rotor[1] / l]
        };
        Self { estado: e, ..self }
    }

    #[must_use]
    pub fn estado(self) -> Estado {
        self.estado
    }

    /// Se algum sub-dab gira a pegada.
    #[must_use]
    pub fn gira(self) -> bool {
        self.estado.rotor != [1.0, 0.0]
    }

    /// O maior raio de um sub-dab sobre o do dab (`≥ 1`) — quanto a caixa cresce.
    #[must_use]
    pub fn escala_max(self) -> f32 {
        self.estado.raio.max(1.0)
    }

    /// O sub-dab `k`: `t = k/n` do caminho de volta, cada campo interpolado em `t` (o rotor por nlerp,
    /// que só se afasta do ângulo linear em giros grandes por passo; meia volta fica no da frente).
    #[inline]
    #[must_use]
    pub fn sub_dab(self, k: u32) -> Sub {
        let t = k as f32 / self.n as f32;
        let e = self.estado;
        let lerp = |b: f32| 1.0 + (b - 1.0) * t;
        let rotor = (e.rotor != [1.0, 0.0] && k > 0)
            .then(|| {
                let v = [1.0 + (e.rotor[0] - 1.0) * t, e.rotor[1] * t];
                let l = v[0].hypot(v[1]);
                (l > 1e-6).then(|| [v[0] / l, v[1] / l])
            })
            .flatten();
        Sub {
            off: self.off(k),
            escala: lerp(e.raio),
            cobertura: lerp(e.cobertura),
            rotor,
            arco: e.arco * t,
        }
    }

    #[must_use]
    pub fn n(self) -> u32 {
        self.n
    }

    /// O desvio do sub-dab `k` ao centro do dab, em px: `−D·k/n`.
    #[inline]
    #[must_use]
    pub fn off(self, k: u32) -> [f32; 2] {
        let f = -(k as f32) / self.n as f32;
        [self.passo[0] * f, self.passo[1] * f]
    }

    /// Quanto a caixa do dab cresce para TRÁS, por eixo, em px: `[min, max]` do desvio dos sub-dabs
    /// (o lado do centro anterior; o outro lado fica `0`).
    #[must_use]
    pub fn caixa(self) -> ([f32; 2], [f32; 2]) {
        let u = self.off(self.n - 1);
        (
            [u[0].min(0.0), u[1].min(0.0)],
            [u[0].max(0.0), u[1].max(0.0)],
        )
    }
}

/// O sub-passo da cápsula, em fracção do DIÂMETRO do dab — o Spacing do traço denso que ela iguala.
pub const SUB_SPACING: f32 = 0.025;

/// O sub-passo da cápsula de um dab de raio `raio`, em px: [`SUB_SPACING`] do diâmetro, nunca abaixo de
/// um texel (o mesmo piso do caminhador, [`crate::BrushSpec::dab_spacing_px`]).
#[must_use]
pub fn sub_passo_px(raio: f32) -> f32 {
    sub_passo_px_a(raio, SUB_SPACING)
}

/// [`sub_passo_px`] a outra fracção `fracao` do diâmetro (o sub-passo do pincel do Wet Paint); a de uma sonda
/// ([`com_lei`]) manda sobre as duas.
#[must_use]
pub fn sub_passo_px_a(raio: f32, fracao: f32) -> f32 {
    let o = SUB_PX.with(Cell::get);
    let f = if o > 0.0 { o } else { fracao };
    (f * 2.0 * raio).max(1.0)
}

/// ⭐ **O sub-passo que não BATE com a grelha** (doc 51 §3): o sub-passo dos sub-dabs SOLTOS ([`sub_dabs`]) é um
/// número INTEIRO de células `celula_px`. Um passo fraccionário bate com elas: abaixo de 2 células num período
/// `1 ÷ (1 − 1/passo)` (o Oil Bristle a 41 px, 1,3 px pelo lado fino: 4,2 px; o Express Oil, ~1,1 px: 8,1 px — os
/// medidos), e a escada do Oil Bristle a 103 px lê 3,06 a 2,06 px contra 2,24 a 2,0. ⭐ Arredondado para BAIXO: o
/// traço contínuo nunca fica mais ralo que o natural — a 1,5 → 2 células o Marker a 30 px clareava o miolo 4,01 % e o
/// Dip Pen não escurecia a 3.ª camada (doc 51 §4). O custo dobra onde o natural cai em `[1,25, 2)`: quem não o paga
/// (os secos de grão a 40 px, 20 ms por quadro) pede [`sub_passo_mais_perto`].
#[must_use]
pub fn sub_passo_sem_batimento(sub_px: f32, celula_px: f32) -> f32 {
    (sub_px / celula_px).floor().max(1.0) * celula_px
}

/// [`sub_passo_sem_batimento`] pelo inteiro mais PERTO (até 1,25 células é uma: nunca mais de 25 % de dabs) — a lei
/// de antes do doc 51 §4, a dos pincéis secos de grão (`capsuleRoundNearest`): arredondado para baixo, o Oil Pastel
/// a 40 px ia de 10,3 a 20,2 ms por quadro, o Charcoal a 18,0, o Soft Pastel a 17,4.
#[must_use]
pub fn sub_passo_mais_perto(sub_px: f32, celula_px: f32) -> f32 {
    let n = sub_px / celula_px;
    if n < 1.25 {
        celula_px
    } else {
        n.round().max(2.0) * celula_px
    }
}

/// Como os sub-dabs se compõem no píxel — chave de MEDIÇÃO; o produto corre sempre a [`Lei::Uniao`]
/// (há portão a afirmá-lo).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Lei {
    /// `(1 − Π(1 − c·x_k)) / c`: as `n` passagens do traço denso, compostas numa (exacto para a mesma cor
    /// por cima, e `Σ x_k` quando `c → 0`).
    Uniao,
    /// `Σ x_k / n` — a soma do regime linear sobre `n` (a proposta do prompt 34; dilui).
    Media,
    /// A lei do prompt 32: a queda do CENTRO × a média do Shape sobre os sub-dabs (os retângulos).
    Centro,
    /// A do Wet Paint no prompt 32: a queda do centro × o máximo do Shape (os tracinhos).
    CentroMax,
}

thread_local! {
    static LEI: Cell<Lei> = const { Cell::new(Lei::Uniao) };
    static SUB_PX: Cell<f32> = const { Cell::new(0.0) };
    static SO_O_CENTRO: Cell<bool> = const { Cell::new(false) };
}

/// A lei de antes do prompt 40 está armada nesta thread: a cápsula só desloca o centro (o raio, a
/// rotação, a pressão e o arco são os do dab da frente). Chave de MEDIÇÃO — o CONTROLO dos portões
/// (`capsula_estado.rs`); o produto nunca a arma (há portão a afirmá-lo).
#[must_use]
pub fn so_o_centro() -> bool {
    SO_O_CENTRO.with(Cell::get)
}

/// Corre `f` com a lei de antes do prompt 40 ([`so_o_centro`]); repõe ao sair.
pub fn com_so_o_centro<R>(f: impl FnOnce() -> R) -> R {
    let antes = SO_O_CENTRO.with(|c| c.replace(true));
    let r = f();
    SO_O_CENTRO.with(|c| c.set(antes));
    r
}

/// A lei em vigor nesta thread (lida UMA vez por dab).
#[must_use]
pub fn lei() -> Lei {
    LEI.with(Cell::get)
}

/// Corre `f` com outra lei e outra fracção do sub-passo (`None` = [`SUB_SPACING`]) — as sondas e os
/// CONTROLOS dos portões; repõe as de antes ao sair.
pub fn com_lei<R>(l: Lei, sub_px: Option<f32>, f: impl FnOnce() -> R) -> R {
    let (l0, s0) = (
        LEI.with(|c| c.replace(l)),
        SUB_PX.with(|c| c.replace(sub_px.unwrap_or(0.0))),
    );
    let r = f();
    LEI.with(|c| c.set(l0));
    SUB_PX.with(|c| c.set(s0));
    r
}

/// A composição dos sub-dabs de UM píxel: `mais(x, queda, forma)` por sub-dab, `fim()` no fim. `c` é a
/// opacidade de cada passagem (o carimbo passa a do dab; um meio sem opacidade por passagem, `1`).
#[derive(Clone, Copy, Debug)]
pub struct Junta {
    lei: Lei,
    c: f32,
    n: u32,
    soma: f32,
    prod: f32,
    forma: f32,
    queda0: f32,
}

impl Junta {
    #[must_use]
    pub fn new(lei: Lei, c: f32) -> Self {
        Self {
            lei,
            c: c.clamp(0.0, 1.0),
            n: 0,
            soma: 0.0,
            prod: 1.0,
            forma: 0.0,
            queda0: 0.0,
        }
    }

    /// Um sub-dab, pela ordem `k = 0, 1, …`: `x` a silhueta que ele deixa (a queda composta com o Shape),
    /// e as duas partes — a queda e o Shape — que só as leis do prompt 32 separam.
    #[inline]
    pub fn mais(&mut self, x: f32, queda: f32, forma: f32) {
        if self.n == 0 {
            self.queda0 = queda;
        }
        self.n += 1;
        match self.lei {
            Lei::Uniao => {
                self.soma += x;
                self.prod *= 1.0 - (self.c * x).clamp(0.0, 1.0);
            }
            Lei::Media => self.soma += x,
            Lei::Centro => self.forma += forma,
            Lei::CentroMax => self.forma = self.forma.max(forma),
        }
    }

    /// O peso do dab neste píxel, na unidade da silhueta de um dab (o carimbo multiplica-o por `c`).
    #[must_use]
    pub fn fim(self) -> f32 {
        if self.n == 0 {
            return 0.0;
        }
        match self.lei {
            Lei::Uniao if self.c > 1e-6 => (1.0 - self.prod) / self.c,
            Lei::Uniao => self.soma,
            Lei::Media => self.soma / self.n as f32,
            Lei::Centro => self.queda0 * self.forma / self.n as f32,
            Lei::CentroMax => self.queda0 * self.forma,
        }
    }
}

/// ⭐ **The capsule of `d` as plain dabs ON THE DENSE STROKE'S POINTS** (doc 51 §3), oldest first: the sub-dabs
/// sit at the multiples of `sub_px` of the stroke's arc length that fall in `(arc of the previous dab, arc of d]` —
/// the very points a Spacing of `sub_px` would stamp, whatever the outer Spacing — each the dab INTERPOLATED back
/// to its [`crate::Dab::ante`] (centre, radius, coverage, heading/rotation; `step` zero). A segment with no
/// multiple in it lays nothing; `d` itself with no capsule. Splitting each step into `⌈|D|/sub⌉` instead laid them
/// a non-integer number of pixels apart, which BEATS with the grid (the oils at 41 px: a 4,2 and 8,1 px period).
/// For a medium whose deposit is not a composition the stamp can fold — the Wet Paint gates each dab against the
/// paper.
#[must_use]
pub fn sub_dabs(d: &crate::Dab, sub_px: f32) -> Vec<crate::Dab> {
    let l = d.step[0].hypot(d.step[1]);
    let a1 = d.arc_len;
    let a0 = d.ante.map_or(a1 - l, |a| a.arc_len);
    let comp = a1 - a0;
    // NaN também fica de fora (um comprimento sem número não tem sub-dabs).
    if l.is_nan()
        || l <= 1e-6
        || comp.is_nan()
        || comp <= 1e-6
        || !sub_px.is_finite()
        || sub_px <= 0.0
    {
        return vec![*d];
    }
    // Um dab exactamente sobre um múltiplo conta (o `1e-4` do arredondamento do arco em f32).
    let m0 = (a0 / sub_px + 1e-4).floor() as i64 + 1;
    let m1 = (a1 / sub_px + 1e-4).floor() as i64;
    (m0..=m1.min(m0 + i64::from(MAX_SUB) - 1))
        .map(|m| {
            let a = m as f32 * sub_px;
            let t = ((a1 - a) / comp).clamp(0.0, 1.0);
            let mut s = crate::Dab {
                center: [d.center[0] - d.step[0] * t, d.center[1] - d.step[1] * t],
                arc_len: a,
                step: [0.0, 0.0],
                ante: None,
                ..*d
            };
            if let Some(b) = d.ante {
                let lerp = |f: f32, b: f32| f + (b - f) * t;
                let nlerp = |f: [f32; 2], b: [f32; 2]| {
                    let v = [lerp(f[0], b[0]), lerp(f[1], b[1])];
                    let l = v[0].hypot(v[1]);
                    if l > 1e-6 { [v[0] / l, v[1] / l] } else { f }
                };
                s.radius_px = lerp(d.radius_px, b.radius_px);
                if let Some(j) = b.salto {
                    s.center =
                        salto_do_sub_dab(s.center, &j, m, s.radius_px / d.radius_px.max(1e-6));
                }
                s.coverage = lerp(d.coverage, b.coverage);
                s.rotation = nlerp(d.rotation, b.rotation);
                // An unset heading stays unset (the rotor reads it as the identity).
                if d.dir != [0.0, 0.0] && b.dir != [0.0, 0.0] {
                    s.dir = nlerp(d.dir, b.dir);
                }
            }
            s
        })
        .collect()
}

/// O centro de um sub-dab com o SEU Jitter ([`Salto`]): tira o desvio do dab da frente e sorteia outro no disco,
/// pelo fluxo splitmix do Jitter semeado pelo desvio da frente e pelo múltiplo `m` do sub-passo — determinístico.
fn salto_do_sub_dab(c: [f32; 2], j: &Salto, m: i64, razao_do_raio: f32) -> [f32; 2] {
    let alcance = if j.por_raio {
        j.alcance * razao_do_raio
    } else {
        j.alcance
    };
    let mut rng = (u64::from(j.desvio[0].to_bits()) << 32 | u64::from(j.desvio[1].to_bits()))
        ^ (m as u64).wrapping_mul(0xD1B5_4A32_D192_ED03);
    let (dx, dy) = crate::jitter::disc_sample(&mut rng);
    [
        c[0] - j.desvio[0] + dx * alcance,
        c[1] - j.desvio[1] + dy * alcance,
    ]
}

/// The per-dab state a capsule interpolates back to ([`Dab::ante`]): what the previous dab of the path
/// was, beside its centre. Orientation vectors transform with the dab under Symmetry.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Ante {
    pub radius_px: f32,
    pub coverage: f32,
    pub dir: [f32; 2],
    pub rotation: [f32; 2],
    pub arc_len: f32,
    /// O Jitter de posição do dab da FRENTE ([`Salto`]); `None` = sem Jitter.
    pub salto: Option<Salto>,
}

/// ⭐ **O Jitter de posição de um dab** — o que [`sub_dabs`] precisa para que cada sub-dab tire o SEU, como o
/// traçador faria se pousasse um dab ali (doc 51 §4): sem isto os sub-dabs eram o dab da frente transladado pelo
/// caminho, todos com o MESMO desvio (o traço liso entre saltos a cada Spacing — a textura do Graphite 8,77 → 7,74).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Salto {
    /// O desvio que o dab da frente levou (`center − ponto do caminho`).
    pub desvio: [f32; 2],
    /// O raio do disco do desvio no dab da frente, px.
    pub alcance: f32,
    /// O alcance escala com o raio do dab (`JitterUnit::Brush`); `false` = px de ecrã (`View`).
    pub por_raio: bool,
}

impl Ante {
    /// The state `d` leaves behind for the next dab of its path.
    #[must_use]
    pub fn de(d: &crate::Dab) -> Self {
        Self {
            radius_px: d.radius_px,
            coverage: d.coverage,
            dir: d.dir,
            rotation: d.rotation,
            arc_len: d.arc_len,
            salto: None,
        }
    }

    /// The same state with its orientation vectors mapped by `f` (a Symmetry copy).
    #[must_use]
    pub fn orientado(self, f: impl Fn([f32; 2]) -> [f32; 2]) -> Self {
        Self {
            dir: f(self.dir),
            rotation: f(self.rotation),
            salto: self.salto.map(|j| Salto {
                desvio: f(j.desvio),
                ..j
            }),
            ..self
        }
    }
}

impl crate::Dab {
    /// The dab-capsule ([`crate::capsula`]) its [`Self::step`] lays, at the product's sub-step, carrying
    /// the [`Self::ante`] state relative to this dab's under `spec` (the rotor needs the follow law);
    /// `None` with no step (Continuous Spacing off, the opening dab, the single marks).
    #[must_use]
    pub fn capsula(&self, spec: &crate::BrushSpec) -> Option<Capsula> {
        let cap = Capsula::new(self.step, sub_passo_px(self.radius_px))?;
        let Some(a) = self.ante else {
            return Some(cap);
        };
        let razao = |atras: f32, frente: f32| {
            if frente > 1e-6 && atras.is_finite() && atras >= 0.0 {
                atras / frente
            } else {
                1.0
            }
        };
        let frente = spec.dab_rotor(self);
        let atras = spec.dab_rotor(&Self {
            dir: a.dir,
            rotation: a.rotation,
            ..*self
        });
        // The back rotor relative to the front one: `atras · conj(frente)`.
        let rotor = crate::heading::rotate(atras, [frente[0], -frente[1]]);
        Some(cap.com_estado(Estado {
            raio: razao(a.radius_px, self.radius_px),
            cobertura: razao(a.coverage, self.coverage),
            rotor,
            arco: a.arc_len - self.arc_len,
        }))
    }
}

#[cfg(test)]
#[path = "capsula_tests.rs"]
mod tests;
