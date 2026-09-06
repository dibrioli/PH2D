//! **O RECTÂNGULO — preencher e traçar, com quinas.**
//!
//! Irmão do [`super`] pelo tecto de 700 LOC do ficheiro, e o corte é por RESPONSABILIDADE: ali
//! mora o *despacho* de pintura (a resolução de token, o texto centrado, a paleta de ícones, a
//! porta da moldura); aqui, *a forma que quase todo o cromo desta casa desenha*.
//!
//! ⚠️ **Ele cresceu no dia em que o raio deixou de ser um número e passou a ser QUATRO**
//! (2026-09-06, a lei do grupo do Blender: numa fileira de botões vizinhos só as bordas de fora
//! arredondam) — e foi esse crescimento que empurrou o `paint.rs` para lá do tecto. *Um ficheiro
//! que cruza o tecto por causa de uma família nova está a dizer que a família tem casa própria.*

use super::rect_to_vello;
use crate::published::radius_scale;
use crate::zones::Rect;
use ph2d_vector::{Affine, Color, Fill, RoundedRect, Stroke, VectorScene};
fn scale_radius(r: f32) -> f32 {
    let s = radius_scale();
    if s == 1.0 {
        r
    } else {
        // Preserve perfect-circle / pill semantics — `Radius::Full`
        // (999) was chosen specifically so it always wraps to the
        // shortest axis. Scaling it would un-pill pills at scale < 1.
        if r >= 999.0 { r } else { r * s }
    }
}

/// Fill a rect with rounded corners. Pass `radius == 0` for sharp.
pub fn fill_rounded_rect(scene: &mut VectorScene, rect: Rect, radius: f32, color: Color) {
    let radius = scale_radius(radius);
    if radius <= 0.0 {
        scene.fill_rect(rect_to_vello(rect), color);
        return;
    }
    let rr = RoundedRect::new(
        rect.x as f64,
        rect.y as f64,
        (rect.x + rect.w) as f64,
        (rect.y + rect.h) as f64,
        radius as f64,
    );
    scene
        .inner_mut()
        .fill(Fill::NonZero, Affine::IDENTITY, color, None, &rr);
}

/// ⭐⭐⭐ **Preenche um rect com um raio POR CANTO** — a lei do grupo de botões do Blender.
///
/// Enio, 2026-09-06: *«se 2 ou mais botões estão lado a lado, só as bordas externas dos botões das
/// extremidades recebem arredondamento»*. É isso que faz uma fileira ler-se como **um controlo**
/// em vez de N botões soltos, e é por isso que a fileira dele não precisa de vão nenhum entre as
/// peças — *o que separa duas peças de um grupo é a QUINA, não o espaço*.
///
/// A ordem é a do `RoundedRectRadii` do kurbo: `(cima-esq, cima-dir, baixo-dir, baixo-esq)`.
pub fn fill_rounded_rect_radii(
    scene: &mut VectorScene,
    rect: Rect,
    radii: (f32, f32, f32, f32),
    color: Color,
) {
    let (tl, tr, br, bl) = radii;
    let (tl, tr, br, bl) = (
        scale_radius(tl) as f64,
        scale_radius(tr) as f64,
        scale_radius(br) as f64,
        scale_radius(bl) as f64,
    );
    if tl <= 0.0 && tr <= 0.0 && br <= 0.0 && bl <= 0.0 {
        scene.fill_rect(rect_to_vello(rect), color);
        return;
    }
    let rr = RoundedRect::new(
        rect.x as f64,
        rect.y as f64,
        (rect.x + rect.w) as f64,
        (rect.y + rect.h) as f64,
        (tl, tr, br, bl),
    );
    scene
        .inner_mut()
        .fill(Fill::NonZero, Affine::IDENTITY, color, None, &rr);
}

/// Stroke an axis-aligned rect (sharp corners) with the given line
/// width and color. Default `Stroke` uses round joins/caps.
pub fn stroke_rect(scene: &mut VectorScene, rect: Rect, width: f32, color: Color) {
    let stroke = Stroke::new(width as f64);
    scene
        .inner_mut()
        .stroke(&stroke, Affine::IDENTITY, color, None, &rect_to_vello(rect));
}

/// Stroke a rect with rounded corners. Same defaults as
/// [`stroke_rect`]. Pass `radius == 0` to fall through to sharp.
pub fn stroke_rounded_rect(
    scene: &mut VectorScene,
    rect: Rect,
    radius: f32,
    width: f32,
    color: Color,
) {
    let radius = scale_radius(radius);
    if radius <= 0.0 {
        stroke_rect(scene, rect, width, color);
        return;
    }
    let rr = RoundedRect::new(
        rect.x as f64,
        rect.y as f64,
        (rect.x + rect.w) as f64,
        (rect.y + rect.h) as f64,
        radius as f64,
    );
    let stroke = Stroke::new(width as f64);
    scene
        .inner_mut()
        .stroke(&stroke, Affine::IDENTITY, color, None, &rr);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::published::set_radius_scale;

    /// ⚠️ **A escala do raio é um thread-local, e o `Radius::Full` (999) escapa-lhe de
    /// propósito** — senão uma pílula deixava de ser pílula a escala < 1. *O teste mudou-se com
    /// a função no corte de 2026-09-06: uma prova que fica na casa antiga mede um nome, não uma
    /// lei.*
    #[test]
    fn radius_scale_preserves_full_pill() {
        set_radius_scale(0.2);
        assert!((scale_radius(999.0) - 999.0).abs() < f32::EPSILON);
        assert!((scale_radius(12.0) - 2.4).abs() < 1e-4);
        set_radius_scale(1.6);
        assert!((scale_radius(999.0) - 999.0).abs() < f32::EPSILON);
        assert!((scale_radius(8.0) - 12.8).abs() < 1e-4);
        set_radius_scale(1.0);
    }
}
