//! [`Rgba`] — a cor de storage do documento Flip: RGBA **linear**, **straight
//! alpha**, `f32`.
//!
//! Tipo local (e não `ph2d_color::LinearRgba`) por dois motivos concretos:
//! 1. `LinearRgba` **não deriva serde** e tem campos privados — o documento
//!    precisa serializar (postcard) e construir cores por array;
//! 2. segue o precedente do modelo puro irmão (`ph2d_vec_scene::Rgba8` define a
//!    própria cor; OKLCH via `ph2d-color` é refinamento de fase futura).
//!
//! **Straight alpha, não premultiplicado:** é a forma intuitiva de EDITAR (o
//! artista escolhe cor e opacidade separados; ver a decisão em
//! `02_referencia §1` — "preferir override explícito"). A premultiplicação é do
//! upload GPU (W1), não da storage. Linear porque lerp de Tween (W3) e blend
//! (W1) acontecem em luz linear.

use serde::{Deserialize, Serialize};

/// Cor RGBA linear, straight-alpha, canais em `f32` (tipicamente `[0,1]`, mas
/// não clampado — HDR de traço é possível).
#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Rgba(pub [f32; 4]);

impl Rgba {
    /// Branco opaco — o default de ponto (um traço "sem cor escolhida" pinta com
    /// a cor do pincel; aqui a storage é sempre explícita).
    pub const WHITE: Rgba = Rgba([1.0, 1.0, 1.0, 1.0]);
    /// Preto opaco.
    pub const BLACK: Rgba = Rgba([0.0, 0.0, 0.0, 1.0]);
    /// Totalmente transparente.
    pub const TRANSPARENT: Rgba = Rgba([0.0, 0.0, 0.0, 0.0]);

    /// Constrói a partir dos quatro canais lineares straight-alpha.
    #[must_use]
    pub const fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self([r, g, b, a])
    }

    #[must_use]
    pub const fn r(self) -> f32 {
        self.0[0]
    }
    #[must_use]
    pub const fn g(self) -> f32 {
        self.0[1]
    }
    #[must_use]
    pub const fn b(self) -> f32 {
        self.0[2]
    }
    #[must_use]
    pub const fn a(self) -> f32 {
        self.0[3]
    }
}

impl Default for Rgba {
    /// Branco opaco (ver [`Rgba::WHITE`]).
    fn default() -> Self {
        Self::WHITE
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accessors_and_default() {
        let c = Rgba::new(0.1, 0.2, 0.3, 0.4);
        assert_eq!((c.r(), c.g(), c.b(), c.a()), (0.1, 0.2, 0.3, 0.4));
        assert_eq!(Rgba::default(), Rgba::WHITE);
        assert_eq!(Rgba::TRANSPARENT.a(), 0.0);
    }

    #[test]
    fn round_trips_through_postcard() {
        let c = Rgba::new(0.25, 0.5, 0.75, 1.0);
        let bytes = postcard::to_allocvec(&c).unwrap();
        let back: Rgba = postcard::from_bytes(&bytes).unwrap();
        assert_eq!(back, c);
    }
}
