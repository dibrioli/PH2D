//! O easing do `motion.stagger` — a porta é a da casa, [`ph2d_ease::ease`] (ciclo 17, doc 126
//! §3.1); aqui só mora a NONA família, a que o artista DESENHA, porque a curva vive no text param
//! deste nó e não na tabela.
//!
//! Família (`ease_curve`): 0 Linear · 1 Quad · 2 Cubic · 3 Quart · 4 Quint · 5 Circ · 6 Back ·
//! 7 Bounce · 8 Custom · 9 Sine · 10 Expo · 11 Elastic · 12 Steps · 13 Spring · 14 Bezier.
//! Direção (`ease_dir`): 0 In · 1 Out · 2 In-Out — lida só pelas [`ph2d_ease::USES_DIRECTION`].

/// **A NONA FAMÍLIA: a que o artista DESENHA** (doc 89, folha 06 · Cavalry *Stagger ▸ Graph*).
/// A forma vive num text param ([`super::CURVE_KEY`]); não-setada = `t`, o `Linear`. Ela ignora o
/// `dir` pela mesma razão que o `Linear`: a curva desenhada É a intenção do artista.
pub(crate) const EASE_CUSTOM: i32 = ph2d_ease::CUSTOM;

/// A ease completa: a `Custom` lê a curva desenhada; as outras catorze são a tabela da casa.
pub(crate) fn ease(
    curve: i32,
    dir: i32,
    t: f32,
    custom: Option<&ph2d_curve::Curve>,
    shape: &ph2d_ease::Shape,
) -> f32 {
    if curve == EASE_CUSTOM {
        return custom.map_or(t, |c| c.eval(t));
    }
    ph2d_ease::ease(curve, dir, t, shape)
}
