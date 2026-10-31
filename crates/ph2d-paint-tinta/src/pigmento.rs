//! Pigmento `(K, S)` por banda, em µm⁻¹ (por µm de pigmento puro).

use crate::bandas::Bandas;

/// Piso do masstone: `R∞ = 0` daria `K/S = ∞`. A `1e-3` o `K/S` é ~`499`, finito em `f32` e abaixo
/// de qualquer preto de tubo real (negro de fumo ≈ `0,03` sRGB-linear).
pub const R_INF_MIN: f32 = 1.0e-3;
/// `S` do pigmento com transparência `0` (opaco). PROVISÓRIO — ver F4_1 §calibração.
pub const S_OPACO: f32 = 2.0;
/// `S` do pigmento com transparência `1`. Nunca `0`: `K = S·(1−R∞)²/2R∞` anular-se-ia com ele.
pub const S_TRANSPARENTE: f32 = 0.02;

/// Absorção e espalhamento por banda de UM pigmento.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Pigmento<const N: usize> {
    pub k: Bandas<N>,
    pub s: Bandas<N>,
}

impl<const N: usize> Pigmento<N> {
    /// A partir do masstone `R∞` (linear, por banda) e de `S`: `K = S·(1−R∞)²/(2R∞)` (Kubelka–Munk).
    /// É a mesma construção de `pigmento()` em `lei_unica.py`.
    #[must_use]
    pub fn de_rinf(r_inf: Bandas<N>, s: Bandas<N>) -> Self {
        let mut k = [0.0; N];
        for (i, ki) in k.iter_mut().enumerate() {
            let r = r_inf.0[i].clamp(R_INF_MIN, 1.0);
            let um_menos = 1.0 - r;
            *ki = s.0[i] * (um_menos * um_menos) / (2.0 * r);
        }
        Self { k: Bandas(k), s }
    }

    /// O masstone (camada infinita) deste pigmento sob o factor de espalhamento `s` do veículo.
    #[must_use]
    pub fn masstone(&self, s_fator: f32) -> Bandas<N> {
        Bandas(core::array::from_fn(|i| {
            crate::lei::r_infinito(self.k.0[i] / (s_fator * self.s.0[i]))
        }))
    }
}

impl Pigmento<3> {
    /// A partir da COR escolhida (sRGB não linear, `[0,1]`) e da transparência do PIGMENTO `t ∈ [0,1]`.
    ///
    /// Bandas = R, G, B lineares (EOTF sRGB exacta, IEC 61966-2-1). `R∞` = a cor linear (o masstone
    /// É a cor que o artista escolhe). `S = S_OPACO·(S_TRANSPARENTE/S_OPACO)^t`, igual nas 3 bandas:
    /// a transparência escala o espalhamento em escala log (de opaco a vidrado em duas décadas) sem
    /// mudar o masstone, porque `K/S` fica fixo por `R∞`.
    #[must_use]
    pub fn de_cor(srgb: [f32; 3], transparencia: f32) -> Self {
        let t = transparencia.clamp(0.0, 1.0);
        let s = S_OPACO * libm::powf(S_TRANSPARENTE / S_OPACO, t);
        let lin = srgb.map(srgb_para_linear);
        Self::de_rinf(Bandas(lin), Bandas::uniforme(s))
    }
}

/// EOTF sRGB (IEC 61966-2-1), por `libm::powf` para o mesmo bit em todo SO.
#[must_use]
fn srgb_para_linear(c: f32) -> f32 {
    let c = c.clamp(0.0, 1.0);
    if c <= 0.040_45 {
        c / 12.92
    } else {
        libm::powf((c + 0.055) / 1.055, 2.4)
    }
}
