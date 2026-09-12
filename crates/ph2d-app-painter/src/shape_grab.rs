//! **A tolerância de agarrar uma alça do editor de figuras** — a lei, em pixels de IMAGEM.
//!
//! ⚠️ Ela vivia no `shells/desktop/src/input_dispatch/painter_canvas_input.rs`, que é um ficheiro
//! de **gesto** (`impl App`) e por desenho fica na shell (W2 Fase D). A lei em si não tem `App`
//! nenhum: entra um afim, sai um `f32`. Enquanto morava lá, ela prendia à shell o
//! `painter_bridge_overlays.rs` — que só a queria para desenhar as alças no tamanho certo.
//!
//! ⇒ *o que sai são os CORPOS; o que decide a ordem do quadro fica* — e uma constante de geometria
//! não decide ordem nenhuma. Os três consumidores (o gesto de canvas, o gesto da curva e o
//! desenho das alças) leem daqui, que é o que garante que a alça DESENHADA e a alça AGARRÁVEL
//! têm o mesmo tamanho.

/// Shape-editor (Curve/Circle) control-handle grab radius in SCREEN px — scaled to image px by the
/// sprite footprint before it's forwarded to the tool, so the hit target stays a constant on-screen
/// size at any zoom.
pub const SHAPE_GRAB_TOL_SCREEN_PX: f32 = 10.0;

/// The shape-editor grab tolerance in IMAGE px for a given image→screen `affine`: the constant screen
/// tolerance ÷ the affine's per-image-pixel screen scale (`|linear column 0|`), so the hit target — and
/// the on-canvas handles drawn at a multiple of it — stay a constant on-screen size at any zoom/rotation.
pub fn shape_grab_tol_from_affine(affine: &ph2d_vector::Affine) -> f32 {
    let c = affine.as_coeffs();
    let pixel_scale = ((c[0] * c[0] + c[1] * c[1]) as f32).sqrt();
    if pixel_scale > 0.0 {
        SHAPE_GRAB_TOL_SCREEN_PX / pixel_scale
    } else {
        SHAPE_GRAB_TOL_SCREEN_PX
    }
}

#[cfg(test)]
mod tests {
    //! ⚠️ Este teste veio do `input_dispatch/painter_canvas_input.rs` **com a lei** (W2 Fase D,
    //! `HOWTO` §2.6: *o gate que mede a lei da família vive com a família*). O irmão dele
    //! ficou lá, e com razão: aquele mede o `canvas_down_accepts`, que é o GESTO.
    use super::{SHAPE_GRAB_TOL_SCREEN_PX, shape_grab_tol_from_affine};
    use ph2d_vector::Affine;

    #[test]
    fn grab_tol_scales_inversely_with_zoom() {
        // The image-space grab tolerance = a constant SCREEN radius ÷ the image→screen scale, so it holds
        // a constant on-screen size. This is what keeps the on-canvas handles (drawn at HANDLE_DIST·tol)
        // at a fixed screen distance — and refreshing it every frame is what removes the first-grab snap.
        let tol = |s: f64| shape_grab_tol_from_affine(&Affine::scale(s));
        assert!(
            (tol(1.0) - SHAPE_GRAB_TOL_SCREEN_PX).abs() < 1e-4,
            "1x -> screen radius in image px"
        );
        assert!(
            (tol(2.0) - SHAPE_GRAB_TOL_SCREEN_PX / 2.0).abs() < 1e-4,
            "zoom 2x -> half the image px"
        );
        assert!(
            (tol(0.5) - SHAPE_GRAB_TOL_SCREEN_PX * 2.0).abs() < 1e-4,
            "zoom 0.5x -> twice the image px"
        );
        // A degenerate (zero) scale falls back to the screen constant — never NaN / infinity.
        assert_eq!(tol(0.0), SHAPE_GRAB_TOL_SCREEN_PX);
    }
}
