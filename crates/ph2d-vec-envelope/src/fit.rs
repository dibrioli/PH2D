//! A ponte para o `ParamCurveFit` do kurbo — o passo que torna a deformação *correta* em vez de
//! *quase certa*.
//!
//! O kurbo não é uma conveniência aqui: a doc do trait dele nomeia o nosso caso de uso, textual —
//! *"It is also intended for conversion between curve types ... and **distortion effects such as
//! perspective transform**"*. Ele já resolve subdivisão adaptativa, continuidade G1 nos pontos de
//! corte e a métrica de erro. Reimplementar isso seria reimplementar a nossa própria dependência.

use core::ops::Range;

use kurbo::{BezPath, CubicBez, CurveFitSample, ParamCurveFit, Point, Vec2, fit_to_bezpath};

use crate::{Warp, map_pt_deriv};

/// Uma cúbica-fonte vista **através** de um [`Warp`]: a curva paramétrica `t -> W(C(t))`.
///
/// É só isto que o kurbo precisa para fitar. A curva deformada nunca é materializada; ela é
/// *amostrada*.
pub struct WarpedCubic<'a, W: Warp> {
    src: CubicBez,
    warp: &'a W,
}

impl<'a, W: Warp> WarpedCubic<'a, W> {
    #[must_use]
    pub fn new(src: CubicBez, warp: &'a W) -> Self {
        Self { src, warp }
    }
}

impl<W: Warp> ParamCurveFit for WarpedCubic<'_, W> {
    fn sample_pt_tangent(&self, t: f64, _sign: f64) -> CurveFitSample {
        // `sign` escolhe de que lado de uma descontinuidade amostrar a tangente. Ignorá-lo é
        // coerente com `break_cusp` não reportar nenhuma (abaixo): não há lado a escolher.
        let (p, tangent) = map_pt_deriv(self.warp, &self.src, t);
        CurveFitSample { p, tangent }
    }

    fn sample_pt_deriv(&self, t: f64) -> (Point, Vec2) {
        map_pt_deriv(self.warp, &self.src, t)
    }

    /// **Nenhuma cúspide — e isto é uma afirmação sobre os mapas em escopo, não uma omissão.**
    ///
    /// Uma cúspide em `W(C(t))` nasce onde `J_W(C(t))·C'(t) = 0`: ou a fonte já tinha cúspide (e aí
    /// ela é a *fronteira* de um segmento, nunca o interior — os segmentos entram aqui um a um), ou
    /// **`J_W` é singular**, isto é, o mapa dobra.
    ///
    /// Nenhum mapa das fatias A–D dobra: a gaiola do Quad é **convexa por construção** (ADR-0129
    /// §5), e uma homografia de retângulo para quadrilátero estritamente convexo não põe a linha de
    /// fuga dentro do retângulo — `J_W` é não-singular em todo o domínio. Coons e presets são
    /// suaves.
    ///
    /// ⚠️ **A Fatia E (pinos / MLS) QUEBRA esta premissa.** O MLS-rigid dobra a ~90° de torção de
    /// pino, medido — `det J` muda de sinal. Quem levar o MLS adiante **tem de** implementar este
    /// método; deixá-lo em `None` não trava o fit (a doc do kurbo: uma cúspide perdida vira mais
    /// subdivisão, *"generally not a disaster"*), mas gasta segmentos para aproximar um bico que
    /// não pode ser aproximado. E, pior, ela produz contorno **auto-interseccionado** — que é a
    /// saga da lasca da booleana de novo.
    fn break_cusp(&self, _range: Range<f64>) -> Option<f64> {
        None
    }
}

/// Fita **uma** cúbica-fonte deformada, a `accuracy` (Fréchet).
pub(crate) fn fit_warped(src: &CubicBez, warp: &impl Warp, accuracy: f64) -> BezPath {
    fit_to_bezpath(&WarpedCubic::new(*src, warp), accuracy)
}
