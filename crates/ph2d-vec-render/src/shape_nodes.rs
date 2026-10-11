//! **Os nós do EDIT SHAPE** (F7e) — os nós PRÓPRIOS da forma que o QuickShape acabou de fazer (os
//! cantos, os extremos dos eixos, o ápice do arco). A gramática é a dos pinos do Envelope
//! ([`crate::draw_envelope_pins`]): um ponto cheio, aceso enquanto agarrado — uma alça que se
//! arrasta, distinta do quadrado de âncora que está por baixo.

use ph2d_tokens::{ColorToken, Theme};
use ph2d_vector::{Affine, BezPath, Brush, Circle, Color, Fill, Point, Stroke, VectorScene};

/// `nodes` em MUNDO, `camera` mundo→ecrã; `grabbed` acende o agarrado.
pub fn draw_shape_nodes(
    nodes: &[[f64; 2]],
    grabbed: Option<usize>,
    camera: Affine,
    theme: Theme,
    target: &mut VectorScene,
) {
    let pins: Vec<[[f64; 2]; 2]> = nodes.iter().map(|&p| [p, p]).collect();
    crate::draw_envelope_pins(&pins, grabbed, camera, theme, target);
}

/// **As alças de PARÂMETRO de uma forma** (F9) — o raio do canto, o vale da estrela, os ângulos do
/// arco: as vermelhas do Affinity, aqui no token `shape-handle` (medido contra as alças da caixa,
/// que convivem com estas na mesma forma). `points` e `track` em MUNDO; `hot` (agarrada ou sob o
/// cursor) ganha um anel, e o `track` dela — a guia do Affinity — diz para onde a mão a pode levar.
pub fn draw_shape_handles(
    points: &[[f64; 2]],
    hot: Option<usize>,
    track: &[[f64; 2]],
    camera: Affine,
    theme: Theme,
    target: &mut VectorScene,
) {
    let vello = |t: ColorToken| {
        let c = t.resolve(theme);
        Color::from_rgba8(c.r, c.g, c.b, c.a)
    };
    let (fill, edge) = (
        vello(ColorToken::ShapeHandle),
        vello(ColorToken::BorderEmph),
    );
    if let [first, rest @ ..] = track {
        let mut line = BezPath::new();
        line.move_to(camera * Point::new(first[0], first[1]));
        for p in rest {
            line.line_to(camera * Point::new(p[0], p[1]));
        }
        target.inner_mut().stroke(
            &Stroke::new(LINE_PX),
            Affine::IDENTITY,
            &Brush::Solid(fill),
            None,
            &line,
        );
    }
    for (i, p) in points.iter().enumerate() {
        let at = camera * Point::new(p[0], p[1]);
        let dot = Circle::new(at, crate::ENVELOPE_HANDLE_R_PX);
        target.inner_mut().fill(
            Fill::NonZero,
            Affine::IDENTITY,
            &Brush::Solid(fill),
            None,
            &dot,
        );
        target.inner_mut().stroke(
            &Stroke::new(LINE_PX),
            Affine::IDENTITY,
            &Brush::Solid(edge),
            None,
            &dot,
        );
        if hot == Some(i) {
            let ring = Circle::new(at, crate::ENVELOPE_HANDLE_R_PX * HOT_RING);
            target.inner_mut().stroke(
                &Stroke::new(LINE_PX),
                Affine::IDENTITY,
                &Brush::Solid(fill),
                None,
                &ring,
            );
        }
    }
}

/// Espessura do trilho e do contorno da alça — a da gaiola do Envelope (`envelope::LINE_PX`).
const LINE_PX: f64 = 1.5;
/// O anel da alça quente, em múltiplos do raio dela: fora do contorno (`1 + LINE_PX/raio`) com uma
/// folga do mesmo tamanho.
const HOT_RING: f64 = 1.0 + 2.0 * LINE_PX / crate::ENVELOPE_HANDLE_R_PX;

#[cfg(test)]
mod tests {
    use ph2d_tokens::{ColorToken as C, Theme, srgb_to_oklch};

    fn distancia(x: C, y: C, th: Theme) -> f64 {
        let p = |t: C| {
            let c = t.resolve(th);
            let (l, k, h) = srgb_to_oklch(c.r, c.g, c.b);
            let r = h.to_radians();
            (l, k * r.cos(), k * r.sin())
        };
        let (a, b) = (p(x), p(y));
        ((a.0 - b.0).powi(2) + (a.1 - b.1).powi(2) + (a.2 - b.2).powi(2)).sqrt()
    }

    /// ⭐ **A alça de PARÂMETRO nunca veste a cor das alças da CAIXA** — as duas estão na mesma
    /// forma ao mesmo tempo (Affinity: as vermelhas junto das azuis). A barra é `0,10`, a mesma da
    /// alça de curvatura do osso: o pai escolhido mede `0,122` no pior tema (Workshop) e o vermelho
    /// do Affinity mediria `0,062` (Sunstone, cujo acento é laranja) — `color_alias.rs`.
    #[test]
    fn a_shape_handle_never_wears_the_colour_of_the_box() {
        const BARRA: f64 = 0.10;
        let temas = [
            Theme::Forge,
            Theme::Workshop,
            Theme::Sunstone,
            Theme::Blueprint,
            Theme::Dark,
            Theme::Gray,
            Theme::Light,
            Theme::Oled,
        ];
        assert_eq!(temas.len(), 8, "a varredura tem de cobrir os oito temas");
        for th in temas {
            for caixa in [C::Accent, C::AccentSoft, C::Selection] {
                let d = distancia(C::ShapeHandle, caixa, th);
                assert!(
                    d >= BARRA,
                    "{th:?}: a alça da forma está a {d:.3} de `{caixa:?}`"
                );
            }
        }
    }
}
