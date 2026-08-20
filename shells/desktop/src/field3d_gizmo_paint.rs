//! **A pintura do gizmo 3D** — a metade que faz pixels, separada da [lei](crate::field3d_gizmo).
//!
//! ⚠️ A separação é a que o resto do módulo já usa: a lei responde *"onde ficam as alças e o que o
//! arrasto vale?"* sem janela nenhuma, e este arquivo só a traduz para caminhos. Um gate de gesto
//! nunca precisa de abrir uma janela; um erro de pintura nunca pode mudar o que o arrasto faz.
//!
//! # As cores são TOKENS, e os eixos ganharam os seus
//!
//! `axis-x` / `axis-y` / `axis-z` entraram no design system (HR-15: zero hex). ⚠️ Não se reciclou o
//! `curve-r/g/b`: aquilo é o tinto de um **canal de cor**, isto é a identidade de uma **direção do
//! espaço** — e a convenção X=vermelho / Y=verde / Z=azul é a de todo modelador 3D. O dia em que
//! alguém re-vestir o editor de Curvas não pode mover os eixos junto.
//!
//! O realce de quem está sob o cursor é **derivado do próprio token** (a mesma cor, mais clara em
//! OKLCH), e não uma segunda cor escrita à mão: re-vestir um eixo re-veste o realce dele.

use ph2d_tokens::{Color, ColorToken, Theme};
// ⚠️ Os tipos de caminho vêm da `ph2d-vector`, e não do `vello` direto: ela re-exporta-os de
// propósito para não haver duas versões do mesmo `BezPath` a atravessar a fronteira.
use ph2d_vector::{
    Affine, BezPath, Brush, Circle, Color as VelloColor, Point, Shape as _, VectorScene,
};

use crate::field3d_gizmo::{
    HEAD_HALF_W_PX, HEAD_PX, Handle, INNER_PX, Projected, SHAFT_HALF_W_PX, Shape,
};

/// Quanto o realce levanta a luminosidade do token, em OKLCH.
///
/// ⚠️ **Uma fração do que falta para o branco**, e não uma soma: somar 0,2 a um token já claro
/// estoura para branco e o realce desaparece justamente nos temas claros, que é onde ele é mais
/// difícil de ver.
const HOVER_LIFT: f64 = 0.35;

/// A opacidade de uma alça de plano. Ela é uma **superfície**, e uma superfície opaca no meio do
/// gizmo taparia a peça que se está a mover.
const PLANE_ALPHA: f64 = 0.5;

/// Pinta o gizmo. `hot` é a alça sob o cursor (ou a agarrada), se houver.
/// `origin` é o canto da área desenhada: as alças vêm projetadas no referencial dela, e é este
/// deslocamento que as põe na janela. ⚠️ Ele viaja como transformação do caminho, e não somado às
/// coordenadas — assim a lei continua a falar só de área e nunca de janela.
pub(crate) fn paint(
    scene: &mut VectorScene,
    handles: &[Projected],
    hot: Option<Handle>,
    theme: Theme,
    origin: [f32; 2],
) {
    let at = Affine::translate((f64::from(origin[0]), f64::from(origin[1])));
    // ⚠️ **Do fundo para a frente, ao contrário da ordem de apontar.** A lista vem ordenada de
    // dentro para fora porque é assim que o `pick` desempata; desenhar nessa ordem poria o disco
    // central por baixo das hastes que ele tem de tapar.
    for h in handles.iter().rev() {
        if !h.live {
            continue;
        }
        let base = colour_of(h.handle, theme);
        let c = if hot == Some(h.handle) {
            lift(base)
        } else {
            base
        };
        match h.shape {
            Shape::Arrow { from, to } => arrow(scene, from, to, c, at),
            Shape::Quad(q) => quad(scene, q, with_alpha(c, PLANE_ALPHA), at),
            Shape::Disc { center, radius } => {
                // Um anel, não um disco: cheio ele tapava o ponto da peça que o gizmo marca.
                ring(scene, center, radius, c, at);
            }
        }
    }
}

fn colour_of(handle: Handle, theme: Theme) -> Color {
    let token = match handle {
        Handle::Axis(0) | Handle::Plane(0) => ColorToken::AxisX,
        Handle::Axis(1) | Handle::Plane(1) => ColorToken::AxisY,
        Handle::Axis(2) | Handle::Plane(2) => ColorToken::AxisZ,
        // ⚠️ O disco de vista não é um eixo — ele move no plano da tela, que não tem direção no
        // mundo. Pintá-lo com uma das três cores diria uma coisa falsa sobre o que ele faz.
        _ => ColorToken::Text1,
    };
    token.resolve(theme)
}

/// A mesma cor, mais clara — **derivada do token**, para o realce seguir uma re-vestida.
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

/// Haste (um quadrilátero fino) + ponta (um triângulo). ⚠️ **Preenchimento, e não traço**: a
/// `VectorScene` da casa preenche caminhos, e uma haste desenhada como retângulo tem a mesma
/// espessura em qualquer direção sem depender de um `stroke` que ela não expõe.
fn arrow(scene: &mut VectorScene, from: [f32; 2], to: [f32; 2], c: Color, at: Affine) {
    let d = [to[0] - from[0], to[1] - from[1]];
    let len = d[0].hypot(d[1]);
    if len <= INNER_PX + HEAD_PX {
        return;
    }
    let u = [d[0] / len, d[1] / len];
    let n = [-u[1], u[0]];
    let pt = |t: f32, off: f32| -> Point {
        Point::new(
            f64::from(from[0] + u[0] * t + n[0] * off),
            f64::from(from[1] + u[1] * t + n[1] * off),
        )
    };

    let shaft_end = len - HEAD_PX;
    let mut p = BezPath::new();
    p.move_to(pt(INNER_PX, -SHAFT_HALF_W_PX));
    p.line_to(pt(shaft_end, -SHAFT_HALF_W_PX));
    p.line_to(pt(shaft_end, SHAFT_HALF_W_PX));
    p.line_to(pt(INNER_PX, SHAFT_HALF_W_PX));
    p.close_path();
    scene.fill_path(&p, &brush(c), at);

    let mut head = BezPath::new();
    head.move_to(pt(len, 0.0));
    head.line_to(pt(shaft_end, -HEAD_HALF_W_PX));
    head.line_to(pt(shaft_end, HEAD_HALF_W_PX));
    head.close_path();
    scene.fill_path(&head, &brush(c), at);
}

fn quad(scene: &mut VectorScene, q: [[f32; 2]; 4], c: Color, at: Affine) {
    let mut p = BezPath::new();
    p.move_to(Point::new(f64::from(q[0][0]), f64::from(q[0][1])));
    for v in &q[1..] {
        p.line_to(Point::new(f64::from(v[0]), f64::from(v[1])));
    }
    p.close_path();
    scene.fill_path(&p, &brush(c), at);
}

/// Um anel de espessura [`SHAFT_HALF_W_PX`]`·2`, como dois círculos com regra par-ímpar.
fn ring(scene: &mut VectorScene, center: [f32; 2], radius: f32, c: Color, at: Affine) {
    let ctr = Point::new(f64::from(center[0]), f64::from(center[1]));
    let outer = Circle::new(ctr, f64::from(radius + SHAFT_HALF_W_PX));
    let inner = Circle::new(ctr, f64::from(radius - SHAFT_HALF_W_PX));
    let mut p = outer.to_path(0.1);
    // ⚠️ O buraco sai da regra NonZero com o furo em sentido CONTRÁRIO — é assim que a
    // `VectorScene` preenche (`Fill::NonZero`), e um furo no mesmo sentido seria simplesmente
    // pintado por cima.
    p.extend(inner.to_path(0.1).reverse_subpaths());
    scene.fill_path(&p, &brush(c), at);
}
