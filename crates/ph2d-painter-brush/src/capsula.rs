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

/// O dab-cápsula: o passo e o número de sub-dabs.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Capsula {
    passo: [f32; 2],
    n: u32,
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
        (n >= 2).then_some(Self { passo, n })
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
    let o = SUB_PX.with(Cell::get);
    let f = if o > 0.0 { o } else { SUB_SPACING };
    (f * 2.0 * raio).max(1.0)
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

/// The capsule of `d` as plain dabs, oldest first: the sub-dabs at `d.center + off_k` (`step` zero,
/// the arc length drawn back with them), or `d` itself with no capsule. For a medium whose deposit is
/// not a composition the stamp can fold — the Wet Paint gates each dab against the paper.
pub fn sub_dabs(d: &crate::Dab, sub_px: f32) -> impl Iterator<Item = crate::Dab> + '_ {
    let cap = Capsula::new(d.step, sub_px);
    (0..cap.map_or(1, Capsula::n)).rev().map(move |k| {
        let o = cap.map_or([0.0, 0.0], |c| c.off(k));
        crate::Dab {
            center: [d.center[0] + o[0], d.center[1] + o[1]],
            arc_len: d.arc_len - o[0].hypot(o[1]),
            step: [0.0, 0.0],
            ..*d
        }
    })
}

#[cfg(test)]
#[path = "capsula_tests.rs"]
mod tests;
