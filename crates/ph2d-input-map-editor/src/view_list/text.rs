//! O texto da lista, medido antes de pintar: cada um entra no censo com «coube?».

use ph2d_editor_core::paint::{folga_do_texto, paint_text_elided};
use ph2d_editor_core::zones::Rect;
use ph2d_text::TextSystem;
use ph2d_vector::VectorScene;
use ph2d_vector::Color;

use crate::census::Census;

/// Pinta `s` numa linha dentro de `r` (centrado na vertical), regista-o no censo, e devolve a largura usada.
#[allow(clippy::too_many_arguments)]
pub(crate) fn line(
    census: &mut Census,
    ts: &mut TextSystem,
    scene: &mut VectorScene,
    s: &str,
    r: Rect,
    px: f32,
    color: Color,
) -> f32 {
    let w = ts.prefix_width(s, px);
    census.texts.push((r, s.to_owned(), w > r.w + f32::EPSILON));
    paint_text_elided(ts, scene, s, r.x, r.y + folga_do_texto(r.h, px), px, r.w, color);
    w.min(r.w)
}
