//! **A miniatura de um preset** — o último quadro do [`super::preview::pintar_preview`] com a obra
//! NEUTRA (tela branca, sem papel), na cor da bancada, no relógio FIXO: o mesmo preset dá os mesmos
//! bytes, e por isso as miniaturas da fábrica são o corpus de regressão do motor (uma mudança de lei
//! acusa em todos de uma vez). O preview vivo do painel é o mesmo pintor sobre a obra de AGORA.

use super::preview::{DIAMETRO_MAX, DIAMETRO_MIN, pintar_preview};
use crate::{PaintMedia, PainterTool};
use ph2d_brush_presets::{Miniatura, Preset};

/// O diâmetro do traço da miniatura quando o preset não fixa o tamanho — o `size_px` da bancada
/// (doc 03: o Rebelle e a sonda pintam a 40).
pub const DIAMETRO_DA_MINIATURA: f32 = 40.0;
/// A tinta da miniatura (o azul da bancada, `sonda_gestos::AZUL`).
const AZUL: [u8; 3] = [23, 101, 179];

/// Pinta a miniatura de `p`.
#[must_use]
pub fn miniatura_do_preset(p: &Preset) -> Miniatura {
    let mut t = PainterTool::default();
    t.set_paint_media(PaintMedia::WetPaint);
    // Uma textura que não se leu já foi dita a quem aplicou o preset; a miniatura pinta o que há.
    let _ = t.wet_preset_aplicar(p);
    let d = p
        .parametros
        .volumes
        .size_px
        .map_or(DIAMETRO_DA_MINIATURA, |d| d as f32)
        .clamp(DIAMETRO_MIN, DIAMETRO_MAX);
    t.set_brush_size_px(d * 0.5);
    t.set_brush_color_srgb8(AZUL);
    let mut ultimo = Miniatura {
        largura: 0,
        altura: 0,
        rgba: Vec::new(),
    };
    pintar_preview(&t.wet_preview_cena(), &|| false, &mut |px, w, h| {
        ultimo = Miniatura {
            largura: w,
            altura: h,
            rgba: px.to_vec(),
        };
    });
    ultimo
}
