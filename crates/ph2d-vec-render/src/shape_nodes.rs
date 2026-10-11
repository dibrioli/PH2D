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

/// ⭐ (F11) **A cara de uma alça de parâmetro** — o idioma dos *Live Shapes* do Illustrator (as
/// fotos, `docs/UI_New_and_Simple/vetor_nova_ui/17_gizmos_do_illustrator.md` §1). Quem decide qual
/// é a LEI da alça (`ph2d_tool_vector::shape_handles_face::glyph_of`); aqui só se pinta.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum HandleFace {
    /// ⊙ — o *corner widget*: aro claro com um ponto (arredonda).
    Corner,
    /// ⇕ — o *points/side widget*: aro maior com duas setas (conta).
    Count,
    /// ⊙ na ponta de uma HASTE — o *pie widget* (o ângulo de um arco).
    Angle,
    /// ○ — o *R2*: aro claro maior, sobre o ponto que mede.
    Radius,
}

/// Uma alça a desenhar: onde (MUNDO), a cara, e a raiz da haste (MUNDO; só o [`HandleFace::Angle`]).
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct HandleMark {
    pub at: [f64; 2],
    pub face: HandleFace,
    pub stem: Option<[f64; 2]>,
}

impl HandleFace {
    /// O raio do aro, em múltiplos do da alça do Envelope (o ⊙ = uma alça da caixa, 12 px; o R2 do
    /// Illustrator ~1,5×; o ⇕ ~2,5× — doc 17 §1).
    #[must_use]
    pub fn radius_px(self) -> f64 {
        crate::ENVELOPE_HANDLE_R_PX
            * match self {
                Self::Corner | Self::Angle => 1.0,
                Self::Radius => RADIUS_K,
                Self::Count => COUNT_K,
            }
    }
}

/// **As alças de PARÂMETRO de uma forma** (F9; a cara de cada uma, F11) — no token `shape-handle`
/// (medido contra as alças da caixa, que convivem com estas na mesma forma), com o miolo claro do
/// Illustrator (`bg-elev`). `track` e `ring` em MUNDO; `hot` (agarrada ou sob o cursor) ganha um
/// anel, o `track` dela diz para onde a mão a pode levar, e o `ring` é o círculo TRACEJADO do arrasto
/// de um raio (o R2 do Illustrator).
pub fn draw_shape_handles(
    marks: &[HandleMark],
    hot: Option<usize>,
    track: &[[f64; 2]],
    ring: &[[f64; 2]],
    camera: Affine,
    theme: Theme,
    target: &mut VectorScene,
) {
    let vello = |t: ColorToken| {
        let c = t.resolve(theme);
        Color::from_rgba8(c.r, c.g, c.b, c.a)
    };
    let (ink, paper) = (vello(ColorToken::ShapeHandle), vello(ColorToken::BgElev));
    let line = |pts: &[[f64; 2]]| {
        let mut l = BezPath::new();
        if let [first, rest @ ..] = pts {
            l.move_to(camera * Point::new(first[0], first[1]));
            for p in rest {
                l.line_to(camera * Point::new(p[0], p[1]));
            }
        }
        l
    };
    let s = target.inner_mut();
    if track.len() > 1 {
        s.stroke(
            &Stroke::new(LINE_PX),
            Affine::IDENTITY,
            &Brush::Solid(ink),
            None,
            &line(track),
        );
    }
    if ring.len() > 1 {
        let dashed = Stroke::new(LINE_PX).with_dashes(0.0, DASH_PX);
        s.stroke(
            &dashed,
            Affine::IDENTITY,
            &Brush::Solid(ink),
            None,
            &line(ring),
        );
    }
    for (i, m) in marks.iter().enumerate() {
        let at = camera * Point::new(m.at[0], m.at[1]);
        if let Some(root) = m.stem {
            s.stroke(
                &Stroke::new(LINE_PX),
                Affine::IDENTITY,
                &Brush::Solid(ink),
                None,
                &line(&[root, m.at]),
            );
        }
        let r = m.face.radius_px();
        let disc = Circle::new(at, r);
        s.fill(
            Fill::NonZero,
            Affine::IDENTITY,
            &Brush::Solid(paper),
            None,
            &disc,
        );
        s.stroke(
            &Stroke::new(LINE_PX),
            Affine::IDENTITY,
            &Brush::Solid(ink),
            None,
            &disc,
        );
        match m.face {
            HandleFace::Corner | HandleFace::Angle => {
                let dot = Circle::new(at, r * DOT_K);
                s.fill(
                    Fill::NonZero,
                    Affine::IDENTITY,
                    &Brush::Solid(ink),
                    None,
                    &dot,
                );
            }
            HandleFace::Count => {
                // Duas setas: ⌃ em cima, ⌄ em baixo — o arrasto é vertical.
                let (w, h) = (r * CHEVRON_W, r * CHEVRON_H);
                for dir in [-1.0, 1.0] {
                    let tip = at.y + dir * r * CHEVRON_AT;
                    let mut c = BezPath::new();
                    c.move_to(Point::new(at.x - w, tip - dir * h));
                    c.line_to(Point::new(at.x, tip));
                    c.line_to(Point::new(at.x + w, tip - dir * h));
                    s.stroke(
                        &Stroke::new(LINE_PX),
                        Affine::IDENTITY,
                        &Brush::Solid(ink),
                        None,
                        &c,
                    );
                }
            }
            HandleFace::Radius => {}
        }
        if hot == Some(i) {
            let halo = Circle::new(at, r + 2.0 * LINE_PX);
            s.stroke(
                &Stroke::new(LINE_PX),
                Affine::IDENTITY,
                &Brush::Solid(ink),
                None,
                &halo,
            );
        }
    }
}

/// Espessura do trilho e do contorno da alça — a da gaiola do Envelope (`envelope::LINE_PX`).
const LINE_PX: f64 = 1.5;
/// O tracejado do círculo do raio (traço, vão) — o do Illustrator, lido na foto do R2.
const DASH_PX: [f64; 2] = [4.0, 3.0];
/// O aro do R2 e do ⇕, em múltiplos do ⊙ (doc 17 §1: ~1,5× e ~2,5× a alça da caixa; o ⇕ fica em
/// 1,6 para não cobrir a alça de canto da caixa, a meia alça dela).
const RADIUS_K: f64 = 1.3;
const COUNT_K: f64 = 1.6;
/// O ponto do ⊙, em fração do aro.
const DOT_K: f64 = 0.4;
/// As setas do ⇕: meia largura, altura e onde fica a ponta (frações do aro).
const CHEVRON_W: f64 = 0.35;
const CHEVRON_H: f64 = 0.25;
const CHEVRON_AT: f64 = 0.6;

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

    /// ⭐ (F11) **O miolo claro se lê contra a tinta da alça** — o aro do ⊙/○/⇕ é `shape-handle`
    /// sobre `bg-elev`: os dois têm de se separar em todos os temas (senão o ⊙ vira um borrão).
    #[test]
    fn the_handle_paper_parts_from_its_ink_in_every_theme() {
        for th in [
            Theme::Forge,
            Theme::Workshop,
            Theme::Sunstone,
            Theme::Blueprint,
            Theme::Dark,
            Theme::Gray,
            Theme::Light,
            Theme::Oled,
        ] {
            let d = distancia(C::ShapeHandle, C::BgElev, th);
            assert!(d >= 0.10, "{th:?}: tinta e miolo a {d:.3}");
        }
    }
}
