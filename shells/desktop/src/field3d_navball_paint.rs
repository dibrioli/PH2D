//! **A pintura do gizmo de navegação** — a metade que faz pixels, separada da
//! [lei](crate::field3d_navball).
//!
//! A mesma separação do gizmo 3D, e pelo mesmo motivo: a lei responde *"onde ficam as bolas e qual
//! está sob o cursor?"* sem janela nenhuma, e este arquivo só a traduz para caminhos. Um erro de
//! pintura nunca pode mudar para onde um clique leva a câmera.
//!
//! # As cores são as dos EIXOS, e já existiam
//!
//! `axis-x` / `axis-y` / `axis-z` são tokens do design system desde o gizmo 3D, com a razão escrita
//! ao lado: *"a identidade de uma **direção do espaço**"*, e a convenção X=vermelho / Y=verde /
//! Z=azul é a de todo modelador. ⭐ O gizmo de navegação **fala a mesma língua que as setas que o
//! artista já usa** — se as duas discordassem, a cor deixaria de ser legenda.
//!
//! # ⚠️ Cheia é o eixo positivo, vazada o negativo
//!
//! É a convenção do Blender (e do Unity), e ela carrega a informação que um círculo sozinho não
//! carrega: *de que lado do modelo estou*. A vazada é a mesma cor num anel, porque **é o mesmo
//! eixo** — dar-lhe uma cor própria diria que são seis direções, e são três.
//!
//! # ⚠️ Só há `fill_path`
//!
//! A [`VectorScene`] não expõe traço: um anel é o disco de fora **mais o de dentro ao contrário**
//! (regra `NonZero`), e um talo é um quadrilátero. É o idioma que o `field3d_gizmo_paint` já usa, e
//! segui-lo é o que evita uma segunda forma de desenhar a mesma coisa.

use ph2d_tokens::{Color, ColorToken, Theme};
use ph2d_vector::{
    Affine, BezPath, Brush, Circle, Color as VelloColor, Point, Shape as _, VectorScene,
};

use crate::field3d_navball::{BALL_R_PX, Ball};
use crate::field3d_views::Standard;

/// Quanto o realce levanta a luminosidade do token — **o mesmo número do gizmo 3D**, porque é o
/// mesmo fato («esta é a que o rato vai pegar») no mesmo app.
const HOVER_LIFT: f64 = 0.35;

/// Meia-espessura do anel e do talo — **a do gizmo 3D**, pela mesma razão da cor.
const STROKE_HALF_PX: f32 = crate::field3d_gizmo::SHAFT_HALF_W_PX;

/// ⚠️ Uma bola **atrás** desbota, e é o que dá volume ao widget: sem isto ele lê como seis discos
/// chapados e o artista não distingue o eixo que aponta para ele do que aponta para trás.
const BEHIND_ALPHA: f64 = 0.45;

/// Pinta o gizmo. `hot` é a bola sob o cursor, se houver. `origin` é o canto da área desenhada — as
/// bolas vêm no referencial dela, e este deslocamento põe-nas na janela.
pub(crate) fn paint(
    scene: &mut VectorScene,
    balls: &[Ball],
    hot: Option<Standard>,
    theme: Theme,
    origin: [f32; 2],
    centre: [f32; 2],
) {
    let at = Affine::translate((f64::from(origin[0]), f64::from(origin[1])));
    // ⚠️ **De trás para a frente**, que é a ordem em que a lei devolve. Ver `field3d_navball::balls`.
    for b in balls {
        let base = colour_of(b.view, theme);
        let c = if hot == Some(b.view) {
            lift(base)
        } else {
            base
        };
        let c = if b.depth >= 0.0 {
            c
        } else {
            with_alpha(c, BEHIND_ALPHA)
        };
        if positive(b.view) {
            // O talo, só nas positivas: são elas que nomeiam o eixo.
            bar(scene, centre, b.at, c, at);
            disc(scene, b.at, BALL_R_PX, c, at);
        } else {
            ring(scene, b.at, BALL_R_PX, c, at);
        }
    }
}

/// ⚠️ **A bola positiva de cada eixo** — `Front`/`Top`/`Right`, porque o olho delas está em
/// `+Z`/`+Y`/`+X`. Derivado de `Standard::eye_axis`, não escrito a dobrar.
fn positive(v: Standard) -> bool {
    let a = v.eye_axis();
    a[0] + a[1] + a[2] > 0.0
}

/// A cor do EIXO da vista — a mesma que a seta do gizmo 3D usa para aquela direção.
fn colour_of(v: Standard, theme: Theme) -> Color {
    let a = v.eye_axis();
    let token = if a[0] != 0.0 {
        ColorToken::AxisX
    } else if a[1] != 0.0 {
        ColorToken::AxisY
    } else {
        ColorToken::AxisZ
    };
    token.resolve(theme)
}

fn disc(scene: &mut VectorScene, at_px: [f32; 2], r: f32, c: Color, tf: Affine) {
    let ctr = Point::new(f64::from(at_px[0]), f64::from(at_px[1]));
    scene.fill_path(&Circle::new(ctr, f64::from(r)).to_path(0.1), &brush(c), tf);
}

/// Um anel. ⚠️ O buraco sai da regra `NonZero` com o furo em sentido **contrário** — a mesma nota do
/// `field3d_gizmo_paint::ring`, e o mesmo motivo: no mesmo sentido ele é pintado por cima.
fn ring(scene: &mut VectorScene, at_px: [f32; 2], r: f32, c: Color, tf: Affine) {
    let ctr = Point::new(f64::from(at_px[0]), f64::from(at_px[1]));
    let outer = Circle::new(ctr, f64::from(r + STROKE_HALF_PX));
    let inner = Circle::new(ctr, f64::from(r - STROKE_HALF_PX));
    let mut p = outer.to_path(0.1);
    p.extend(inner.to_path(0.1).reverse_subpaths());
    scene.fill_path(&p, &brush(c), tf);
}

/// Um segmento com espessura, como um quadrilátero.
fn bar(scene: &mut VectorScene, from: [f32; 2], to: [f32; 2], c: Color, tf: Affine) {
    let (dx, dy) = (to[0] - from[0], to[1] - from[1]);
    let len = dx.hypot(dy);
    if len <= f32::EPSILON {
        return;
    }
    // A normal do segmento, com a meia-espessura.
    let (nx, ny) = (-dy / len * STROKE_HALF_PX, dx / len * STROKE_HALF_PX);
    let mut p = BezPath::new();
    p.move_to(pt(from[0] + nx, from[1] + ny));
    p.line_to(pt(to[0] + nx, to[1] + ny));
    p.line_to(pt(to[0] - nx, to[1] - ny));
    p.line_to(pt(from[0] - nx, from[1] - ny));
    p.close_path();
    scene.fill_path(&p, &brush(c), tf);
}

fn pt(x: f32, y: f32) -> Point {
    Point::new(f64::from(x), f64::from(y))
}

/// A mesma cor, mais clara — derivada do token, para o realce seguir uma re-vestida.
fn lift(c: Color) -> Color {
    let (l, chroma, h) = ph2d_tokens::color::srgb_to_oklch(c.r, c.g, c.b);
    let lifted = l + (1.0 - l) * HOVER_LIFT;
    let out = Color::from_oklch(lifted, chroma, h);
    Color { a: c.a, ..out }
}

fn with_alpha(c: Color, a: f64) -> Color {
    Color {
        a: (f64::from(c.a) * a).round() as u8,
        ..c
    }
}

fn brush(c: Color) -> Brush {
    Brush::Solid(VelloColor::from_rgba8(c.r, c.g, c.b, c.a))
}
