//! A lei de MISTURA (ADR-0197 eq. 1, decidida 31/10): Oklab pesado pelo VOLUME, guardado como SOMAS.
//!
//! Estado de uma fatia (filme ou aberto) = `Soma { v, lab: Σ vᵢ·Labᵢ, t: Σ vᵢ·tᵢ, m: Σ vᵢ·mᵢ }`. A cor da mistura
//! é `lab / v` e a transparência `t / v`. Juntar duas fatias é somar componente a componente, logo a
//! mistura é associativa e comutativa (até ao arredondamento da soma `f32`) e N passadas = uma.
//! Mover a fracção α de uma fatia move α de cada componente (as transferências são proporcionais e
//! independentes da cor, F4_2 §2.2) — a soma é estado suficiente e exacto.
//!
//! Porquê Oklab e não Kubelka–Munk por canal: medido na F2b (`F2b_mistura_de_cor_nos_apps.md` §3).

use crate::oklab::{Lab, linear_para_oklab, oklab_para_linear, srgb_para_oklab};

/// Qual lei de mistura de cor produz a cor da tinta misturada.
///
/// Só `Oklab` existe. **TODO (só se o dono o pedir):** `Pigmento` — média geométrica pesada a 10
/// bandas (WGM, `libmypaint` ISC, já portada em `crates/ph2d-painter-brush/src/espectral.rs`), como o
/// controlo opcional «Pigment mix» (azul + amarelo = verde) da F2b §4.2. Exigiria outra `Soma`
/// (`Σ v·ln R` por banda) e uma régua própria de robustez; K-M a 3 bandas e Mixbox estão recusados.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[non_exhaustive]
pub enum LeiDeMistura {
    /// Média de Oklab pesada pelo volume (por omissão).
    #[default]
    Oklab,
}

/// Uma tinta: a COR escolhida (Oklab) e a transparência do pigmento `t ∈ [0,1]`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Tinta {
    pub lab: Lab,
    pub transparencia: f32,
}

impl Tinta {
    /// Da cor sRGB não linear `[0,1]` (o seletor do artista).
    #[must_use]
    pub fn de_srgb(srgb: [f32; 3], transparencia: f32) -> Self {
        Self {
            lab: srgb_para_oklab(srgb),
            transparencia: transparencia.clamp(0.0, 1.0),
        }
    }

    /// Da cor em RGB linear.
    #[must_use]
    pub fn de_linear(rgb: [f32; 3], transparencia: f32) -> Self {
        Self {
            lab: linear_para_oklab(rgb),
            transparencia: transparencia.clamp(0.0, 1.0),
        }
    }

    /// A cor em RGB linear (pode sair de `[0,1]` se a média caiu fora da gama).
    #[must_use]
    pub fn linear(&self) -> [f32; 3] {
        oklab_para_linear(self.lab)
    }
}

/// O pigmento de uma fatia, em somas pesadas pelo volume `v` (µm de pigmento puro, `≥ 0`).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Soma {
    pub v: f32,
    pub lab: [f32; 3],
    pub t: f32,
    /// `Σ v·m`: o MEIO (`0` = água, `1` = óleo) viaja com o pigmento, como a cor (E9 fecho).
    pub m: f32,
}

impl Soma {
    /// `v` µm da tinta `c`.
    #[must_use]
    pub fn de(c: Tinta, v: f32) -> Self {
        debug_assert!(v >= 0.0 && v.is_finite(), "volume inválido: {v}");
        Self {
            v,
            lab: c.lab.map(|x| v * x),
            t: v * c.transparencia,
            m: 0.0,
        }
    }

    /// A mesma fatia com todo o volume no meio `meio ∈ [0,1]`.
    #[must_use]
    pub fn com_meio(self, meio: f32) -> Self {
        Self {
            m: self.v * meio.clamp(0.0, 1.0),
            ..self
        }
    }

    /// O meio médio pelo volume; `0` sem pigmento.
    #[must_use]
    pub fn meio(&self) -> f32 {
        if self.v > 0.0 {
            (self.m / self.v).clamp(0.0, 1.0)
        } else {
            0.0
        }
    }

    /// Cada componente × `k` (`k ≥ 0`): a mesma tinta noutro volume.
    #[must_use]
    pub fn escala(self, k: f32) -> Self {
        let k = k.max(0.0);
        Self {
            v: self.v * k,
            lab: self.lab.map(|x| x * k),
            t: self.t * k,
            m: self.m * k,
        }
    }

    /// Junta duas fatias (soma componente a componente, `self` primeiro).
    #[must_use]
    pub fn junta(self, o: Self) -> Self {
        Self {
            v: self.v + o.v,
            lab: [
                self.lab[0] + o.lab[0],
                self.lab[1] + o.lab[1],
                self.lab[2] + o.lab[2],
            ],
            t: self.t + o.t,
            m: self.m + o.m,
        }
    }

    /// A tinta misturada: `Σv·Lab / Σv`, `Σv·t / Σv`. `None` sem pigmento (`v = 0`).
    #[must_use]
    pub fn cor(&self) -> Option<Tinta> {
        if self.v <= 0.0 {
            return None;
        }
        let v = self.v;
        Some(Tinta {
            lab: self.lab.map(|x| x / v),
            transparencia: (self.t / v).clamp(0.0, 1.0),
        })
    }
}

/// Mistura `itens` (tinta, volume) pela `lei`, somando na ORDEM do slice. `None` se o volume é 0.
#[must_use]
pub fn mistura(itens: &[(Tinta, f32)], lei: LeiDeMistura) -> Option<Tinta> {
    match lei {
        LeiDeMistura::Oklab => itens
            .iter()
            .fold(Soma::default(), |s, &(c, v)| s.junta(Soma::de(c, v)))
            .cor(),
    }
}
