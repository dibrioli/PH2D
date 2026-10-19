//! W23 B5(2) — **os textos de uma seta**: cada um no seu ponto da linha, deitado ao longo dela (ou na
//! horizontal), sobre um recorte do fundo — a linha não lhe passa por baixo (doc 14).

use ph2d_board_model::{Connector, Element};
use ph2d_board_route::{CAPTION_PAD, CaptionFrame, LABEL_WRAP, Routed};
use ph2d_text::TextSystem;
use ph2d_vector::{Affine, Brush, Rect, Shape as _, VectorScene};

use crate::{GREEK_ALPHA, GREEK_MIN_PX, READ_PX, RenderCache, TextSel};

/// O salto da chave do texto moldado de um texto de seta (a do texto de uma forma é o id dela).
const OWNER_SALT: u64 = 0x6361_0000_0000_0000;

/// Até onde (mundo) um texto pode sair da caixa da rota: meia quebra na diagonal, rodado.
#[must_use]
pub(crate) fn reach(c: &Connector) -> f64 {
    if c.captions.is_empty() {
        0.0
    } else {
        LABEL_WRAP / 2.0 * std::f64::consts::SQRT_2
    }
}

/// Mundo → ecrã do bloco de texto de altura `block_h` em `f` (`v` = mundo → ecrã).
#[must_use]
pub(crate) fn block_to_screen(v: Affine, f: CaptionFrame, block_h: f64) -> Affine {
    let [ox, oy] = CaptionFrame::block_origin(block_h);
    v * Affine::translate((f.center[0], f.center[1]))
        * Affine::rotate(f.angle)
        * Affine::translate((ox, oy))
}

/// Pinta os textos de `c` sobre a rota `r`.
#[allow(clippy::too_many_arguments)]
pub(crate) fn paint(
    scene: &mut VectorScene,
    el: &Element,
    c: &Connector,
    r: &Routed,
    v: Affine,
    zoom: f64,
    ts: &mut TextSystem,
    cache: &mut RenderCache,
    owner: (u64, u64),
    paper: &Brush,
    sel: Option<&TextSel<'_>>,
) {
    let st = &c.style;
    let font_px = st.font_size * zoom;
    if c.captions.is_empty() || font_px < cache.lod(GREEK_MIN_PX) {
        return;
    }
    let read_px = cache.lod(READ_PX);
    // O texto assenta num recorte do FUNDO do quadro: segue a tinta do tema.
    let ink = cache.ink.color(st.text_color);
    let face = ph2d_board_edit::face(el);
    for (i, cap) in c.captions.iter().enumerate() {
        let key = (owner.0 ^ OWNER_SALT.wrapping_add(i as u64), owner.1);
        let layout = cache.text.get(
            ts,
            key,
            &cap.text,
            &[],
            st.font_size as f32,
            LABEL_WRAP as f32,
            face,
        );
        let f = r.caption_frame(cap.at, c.turn);
        let at = block_to_screen(v, f, f64::from(layout.height()));
        let bars = ph2d_board_layout::line_bars(layout);
        let [x0, y0, x1, y1] = ph2d_board_layout::knock_box(layout, st.font_size * CAPTION_PAD);
        scene.fill_path(&Rect::new(x0, y0, x1, y1).to_path(0.1), paper, at);
        if let Some(sel) = sel.filter(|s| s.caption == Some(i)) {
            sel.paint(scene, at);
        }
        if font_px >= read_px {
            ph2d_board_layout::paint(scene, layout, &cap.text, at, ink);
        } else {
            scene.fill_path(&bars, &Brush::Solid(ink.with_alpha(GREEK_ALPHA)), at);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ph2d_vector::Point;

    /// ⭐ O bloco de um texto ao longo da linha centra-se no ponto dela e as letras correm na direcção
    /// dela: numa vertical, o meio do bloco cai no ponto e o fim da 1.ª linha fica ACIMA dele (lê-se de
    /// baixo para cima), nunca ao lado.
    #[test]
    fn an_along_caption_block_is_centred_on_the_line_and_turned_with_it() {
        let f = CaptionFrame {
            center: [50.0, 200.0],
            angle: -std::f64::consts::FRAC_PI_2,
        };
        let h = 24.0;
        let t = block_to_screen(Affine::IDENTITY, f, h);
        let mid = t * Point::new(LABEL_WRAP / 2.0, h / 2.0);
        assert!(
            (mid.x - 50.0).abs() < 1e-9 && (mid.y - 200.0).abs() < 1e-9,
            "{mid:?}"
        );
        let end = t * Point::new(LABEL_WRAP, h / 2.0);
        assert!((end.x - 50.0).abs() < 1e-9, "{end:?}");
        assert!((end.y - (200.0 - LABEL_WRAP / 2.0)).abs() < 1e-9, "{end:?}");
    }
}
