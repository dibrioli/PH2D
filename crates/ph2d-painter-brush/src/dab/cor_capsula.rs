//! ⭐ **A COR do dab-cápsula** (Continuous, [`crate::capsula`]; lista 26 A9): a cor de uma rampa do Shape
//! (Texture Color e Falloff Color) compõe-se sobre os MESMOS sub-dabs e pela MESMA ordem da cobertura
//! ([`super::silhouette_capsula_com`]) — de frente para trás, `cor += T·o_k·tinta_k; T *= 1 − o_k` com
//! `o_k = c·x_k·α_k`, cada `tinta_k` pela porta única ([`crate::ramp_alpha::ramp_texel`]) com o valor do
//! Shape no centro do sub-dab e o índice `x_k` dele. Uma passagem com a cor normalizada por `1 − T`
//! deixa o que as `n` passagens do traço denso deixam (source-over é associativo). Lida uma vez no
//! centro do dab da frente, o padrão não andava com os sub-dabs: Δcor 0,39 contra o Spacing 0,025.

use crate::ramp_alpha::RampTexel;

/// O acumulador de UM píxel; `c` é a mesma opacidade por passagem que a [`crate::capsula::Junta`] usa.
pub(super) struct CorCapsula {
    c: f32,
    /// `Π(1 − o_k)` com o alfa da rampa (`Strength`).
    t: f32,
    /// `Π(1 − c·x_k)` — o da cobertura.
    t0: f32,
    cor: [f32; 3],
    alfa_proprio: f32,
}

impl CorCapsula {
    pub(super) fn new(c: f32) -> Self {
        Self {
            c: c.clamp(0.0, 1.0),
            t: 1.0,
            t0: 1.0,
            cor: [0.0; 3],
            alfa_proprio: 0.0,
        }
    }

    /// Um sub-dab de silhueta `x` (já com o corte do filme); `tinta` só corre onde ele pinta. A tinta é a
    /// da rampa SEM o Grain: o `g` dela é só o alfa da rampa.
    #[inline]
    pub(super) fn mais(&mut self, x: f32, tinta: impl FnOnce() -> RampTexel) {
        if x <= 0.0 {
            return;
        }
        let o = (self.c * x).clamp(0.0, 1.0);
        let t = tinta();
        let oa = (o * t.g).clamp(0.0, 1.0);
        let peso = self.t * oa;
        let cor = t.color.unwrap_or([0.0; 3]);
        for (a, b) in self.cor.iter_mut().zip(cor) {
            *a += peso * b;
        }
        self.alfa_proprio += self.t * o * t.stamp_alpha;
        self.t *= 1.0 - oa;
        self.t0 *= 1.0 - o;
    }

    /// A tinta da cápsula neste píxel, com `mordida` = a do Grain (fora da composição, como na
    /// cobertura): a cor normalizada e `g = mordida · (1 − T)/(1 − T0)`. `None` ⇒ nada pinta aqui.
    pub(super) fn fim(self, mordida: f32) -> Option<RampTexel> {
        let (a, a0) = (1.0 - self.t, 1.0 - self.t0);
        if a0 <= 1e-9 {
            return None;
        }
        let inv = if a > 1e-9 { 1.0 / a } else { 0.0 };
        Some(RampTexel {
            color: Some(self.cor.map(|v| v * inv)),
            g: mordida * a / a0,
            stamp_alpha: self.alfa_proprio / a0,
        })
    }
}
