//! **A cor secundária e a cor pela MÃO** (L11, `docs/Painter/Pinceis/03_MOTORES.md` §4 e `06_LEIS_DAS_LACUNAS.md`
//! §4, §10.7): o *Color dynamics* do Procreate — *Color pressure* (matiz, saturação, brilho, cor secundária) — e o
//! *Foreground/Background Jitter* do Photoshop. O ruído de cor POR TRAÇO (o único que o Rebelle usa) já existe
//! ([`crate::ColorJitterScope::PerStroke`]); isto é o que faltava.
//!
//! ⭐ A lei medida no MyPaint `long_grass` (o único oráculo que muda a cor com a pressão; 0/281 no Rebelle, 0/144
//! do Krita usam a 2.ª cor): da pressão 0 a 0,9 a saturação vai de 0,29 a 0,61 e o valor de 0,62 a 0,91, a
//! matiz fica (± 3°) — a mão leve ESCURECE e DESSATURA. Aqui: `S × (1 − s·(1 − p))`, `V × (1 − v·(1 − p))`,
//! `H + h·(1 − p)`; com `s 0,55`, `v 0,33` dá a curva do `long_grass` (portão `a_pressao_leva_a_cor`).
//!
//! ⭐ **Neutro = a cor de hoje ao bit**: com todas as quantidades a zero nada se calcula nem se sorteia.

use crate::jitter::{hsv_to_rgb, next_f32, rgb_to_hsv};

/// A cor pela mão e a cor secundária do pincel. [`CorDaMao::NEUTRA`] = a cor de sempre.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CorDaMao {
    /// **A cor SECUNDÁRIA** do pincel (sRGB `0..1`) — o *Secondary color* do Procreate, o *Background* do
    /// Photoshop. Só age com [`Self::secondary_jitter`] ou [`Self::pressure_secondary`] acima de zero.
    pub secondary: [f32; 3],
    /// **Mistura sorteada com a secundária** `0..1` por dab (Photoshop *Foreground/Background Jitter*): cada dab
    /// vai `u·amount` do caminho até à cor secundária.
    pub secondary_jitter: f32,
    /// **A pressão leva à secundária** `0..1` (Procreate *Color pressure → Secondary color*): na mão leve a cor
    /// vai `amount·(1 − p)` do caminho até à secundária; na pressão cheia é a primária.
    pub pressure_secondary: f32,
    /// **Pressão → matiz** `−0,5..0,5` voltas: a matiz desloca-se `amount·(1 − p)` na mão leve.
    pub pressure_hue: f32,
    /// **Pressão → saturação** `0..1`: `S × (1 − amount·(1 − p))`.
    pub pressure_sat: f32,
    /// **Pressão → valor** `0..1`: `V × (1 − amount·(1 − p))`.
    pub pressure_val: f32,
}

impl CorDaMao {
    /// A cor de sempre.
    pub const NEUTRA: Self = Self {
        secondary: [1.0, 1.0, 1.0],
        secondary_jitter: 0.0,
        pressure_secondary: 0.0,
        pressure_hue: 0.0,
        pressure_sat: 0.0,
        pressure_val: 0.0,
    };

    /// O maior deslocamento de matiz pela pressão, em voltas (meia roda para cada lado).
    pub const HUE_MAX: f32 = 0.5;

    /// Alguma lei está ligada.
    #[must_use]
    pub fn ativa(&self) -> bool {
        self.secondary_jitter > 0.0
            || self.pressure_secondary > 0.0
            || self.pressure_hue != 0.0
            || self.pressure_sat > 0.0
            || self.pressure_val > 0.0
    }

    /// ⭐ **A cor do dab** `cor` à pressão `p`, com o sorteio da secundária do fluxo `rng` (só se ligado). Ordem:
    /// a secundária (sorteada e pela pressão, uma mistura só) e depois o HSV pela pressão.
    #[must_use]
    pub fn aplica(&self, cor: [f32; 3], p: f32, rng: &mut u64) -> [f32; 3] {
        let leve = 1.0 - p.clamp(0.0, 1.0);
        let mut t = self.pressure_secondary.clamp(0.0, 1.0) * leve;
        if self.secondary_jitter > 0.0 {
            t = 1.0 - (1.0 - t) * (1.0 - next_f32(rng) * self.secondary_jitter.clamp(0.0, 1.0));
        }
        let mut c = cor;
        if t > 0.0 {
            for (k, ck) in c.iter_mut().enumerate() {
                *ck += (self.secondary[k] - *ck) * t;
            }
        }
        let (h, s, v) = (
            self.pressure_hue.clamp(-Self::HUE_MAX, Self::HUE_MAX),
            self.pressure_sat.clamp(0.0, 1.0),
            self.pressure_val.clamp(0.0, 1.0),
        );
        if leve > 0.0 && (h != 0.0 || s > 0.0 || v > 0.0) {
            let mut hsv = rgb_to_hsv(c);
            hsv[0] = (hsv[0] + h * leve).rem_euclid(1.0);
            hsv[1] *= 1.0 - s * leve;
            hsv[2] *= 1.0 - v * leve;
            c = hsv_to_rgb(hsv);
        }
        c
    }
}

impl Default for CorDaMao {
    fn default() -> Self {
        Self::NEUTRA
    }
}

#[cfg(test)]
#[path = "cor_mao_tests.rs"]
mod tests;
