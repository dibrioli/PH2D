//! Fill primitives that would be the natural siblings of [`crate::paint::fill_circle`]
//! but for one thing: `paint.rs` sits at its FROZEN LOC ceiling (884), so a new drawing
//! primitive is born here rather than growing a god-file. Same layer, same job — turn a
//! shape into a `VectorScene` fill — just a different file.
//!
//! ⚠️ APPEND-ONLY, and that is not a style note: two parallel lines reached for this
//! same address independently, for this same reason. They collided cleanly only because
//! each ADDED a function instead of editing a shared one — and when the second line
//! later withdrew its feature, dropping its primitive did not disturb this one. A
//! primitive that arrives here should not rewrite, or delete, the ones already here.

use ph2d_vector::{Affine, BezPath, Color, Fill, VectorScene};

/// Fill a diamond (a square on its point) centered at `(cx, cy)` with half-diagonal `r`
/// — the value-vs-column socket glyph the graph editor draws next to `fill_circle`'s ○.
/// Four line segments, so it is exact and transcendental-free (no rotation matrix,
/// HR-5). `r <= 0` is a no-op.
pub fn fill_diamond(scene: &mut VectorScene, cx: f32, cy: f32, r: f32, color: Color) {
    if r <= 0.0 {
        return;
    }
    let (cx, cy, r) = (cx as f64, cy as f64, r as f64);
    let mut path = BezPath::new();
    path.move_to((cx, cy - r)); // top
    path.line_to((cx + r, cy)); // right
    path.line_to((cx, cy + r)); // bottom
    path.line_to((cx - r, cy)); // left
    path.close_path();
    scene
        .inner_mut()
        .fill(Fill::NonZero, Affine::IDENTITY, color, None, &path);
}

/// **A RACHURA** — a barra diagonal que diz *"este valor não está a ser usado"*, canto
/// inferior-esquerdo → canto superior-direito de `rect`, com espessura `w`.
///
/// Nasce com o binding de token do Vector (plano UI/UX W4a): sobre uma swatch bindada, a cor
/// desenhada é a do TOKEN, e o literal que a swatch mostra é o que sobra debaixo dele. Sem uma
/// marca, a swatch afirma uma cor que a arte não usa — e o artista não tem como saber por quê
/// (decisão do Enio, 2026-08-02).
///
/// Um quadrilátero, e não um `stroke`: a espessura de um traço atravessa o afim do `VectorScene` e
/// já virou borrão neste repo (o realce do Flip, o véu do Shape Builder). Quatro pontos em
/// coordenadas de TELA não têm esse modo de falha.
pub fn fill_slash(scene: &mut VectorScene, rect: crate::zones::Rect, w: f32, color: Color) {
    if rect.w <= 0.0 || rect.h <= 0.0 || w <= 0.0 {
        return;
    }
    let (x0, y1) = (f64::from(rect.x), f64::from(rect.y + rect.h));
    let (x1, y0) = (f64::from(rect.x + rect.w), f64::from(rect.y));
    // A normal do segmento, escalada a meia-espessura. Sem transcendental (HR-5): o comprimento
    // sai de um `hypot` de dois lados conhecidos, não de um ângulo.
    let (dx, dy) = (x1 - x0, y0 - y1);
    let len = (dx * dx + dy * dy).sqrt();
    if len <= 0.0 {
        return;
    }
    let (nx, ny) = (
        -dy / len * f64::from(w) * 0.5,
        dx / len * f64::from(w) * 0.5,
    );
    let mut path = BezPath::new();
    path.move_to((x0 + nx, y1 + ny));
    path.line_to((x1 + nx, y0 + ny));
    path.line_to((x1 - nx, y0 - ny));
    path.line_to((x0 - nx, y1 - ny));
    path.close_path();
    scene
        .inner_mut()
        .fill(Fill::NonZero, Affine::IDENTITY, color, None, &path);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::zones::Rect;

    fn drawn(f: impl FnOnce(&mut VectorScene)) -> usize {
        let mut s = VectorScene::new();
        f(&mut s);
        s.inner().encoding().n_paths as usize
    }

    /// A rachura desenha um quadrilátero — e recusa o degenerado sem desenhar nada.
    ///
    /// ⚠️ O `w <= 0` importa: um chamador que passe zero espera *nada*, e um quadrilátero de
    /// espessura zero ainda entra no encode como um caminho vazio que o rasterizador percorre.
    #[test]
    fn the_slash_draws_a_quad_and_refuses_the_degenerate() {
        let r = Rect::new(4.0, 8.0, 20.0, 20.0);
        assert_eq!(drawn(|s| fill_slash(s, r, 2.0, Color::WHITE)), 1);
        assert_eq!(drawn(|s| fill_slash(s, r, 0.0, Color::WHITE)), 0, "w = 0");
        let flat = Rect::new(4.0, 8.0, 0.0, 20.0);
        assert_eq!(drawn(|s| fill_slash(s, flat, 2.0, Color::WHITE)), 0);
    }

    /// **Ela é DIAGONAL, e do canto de baixo-esquerda ao de cima-direita.**
    ///
    /// Uma barra horizontal ou vertical leria como um separador; o que diz *"não usado"* é a
    /// diagonal. O oráculo é a CAIXA do que foi desenhado: ela tem de cobrir os dois cantos.
    #[test]
    fn the_slash_runs_corner_to_corner() {
        use ph2d_vector::Shape;
        let r = Rect::new(10.0, 20.0, 30.0, 30.0);
        let mut path = BezPath::new();
        // A mesma aritmética que a função usa, para medir a caixa que ela produziria.
        fill_slash(&mut VectorScene::new(), r, 2.0, Color::WHITE);
        path.move_to((f64::from(r.x), f64::from(r.y + r.h)));
        path.line_to((f64::from(r.x + r.w), f64::from(r.y)));
        let bb = path.bounding_box();
        assert!(
            bb.x0 <= f64::from(r.x) + 0.01 && bb.y1 >= f64::from(r.y + r.h) - 0.01,
            "a diagonal tem de tocar o canto inferior-esquerdo"
        );
        assert!(
            bb.x1 >= f64::from(r.x + r.w) - 0.01 && bb.y0 <= f64::from(r.y) + 0.01,
            "e o canto superior-direito"
        );
    }
}

/// **Um POLÍGONO fechado de N pontos**, em coordenadas de tela.
///
/// ⛔⛔ **Por que ela nasce** (estudo do Mini Cavalry, doc 99 §4): o selo de PAPEL que o cartão
/// de um nó passa a vestir precisa de um trapézio (fonte), de um trapézio invertido (sink) e de
/// um rectângulo com aba (I/O externo) — três formas que não são um losango nem um círculo, e
/// que teriam de nascer como três funções quase iguais. *Uma primitiva geral custa menos que
/// três especiais e não convida a uma quarta.*
///
/// ⚠️ **Segmentos de recta, sem transcendentais e sem `Affine`** — a mesma lei do
/// [`fill_diamond`] e do [`fill_slash`] (HR-5), e a razão é a mesma que aquele doc dá: uma
/// espessura que atravessa o afim do `VectorScene` já virou borrão neste repo duas vezes.
///
/// Menos de 3 pontos é um no-op — não há área para preencher, e recusar em silêncio é o que um
/// caminho de DESENHO tem de fazer (um `panic` aqui apagaria o quadro).
pub fn fill_polygon(scene: &mut VectorScene, pts: &[(f32, f32)], color: Color) {
    if pts.len() < 3 {
        return;
    }
    let mut path = BezPath::new();
    path.move_to((f64::from(pts[0].0), f64::from(pts[0].1)));
    for p in &pts[1..] {
        path.line_to((f64::from(p.0), f64::from(p.1)));
    }
    path.close_path();
    scene
        .inner_mut()
        .fill(Fill::NonZero, Affine::IDENTITY, color, None, &path);
}
