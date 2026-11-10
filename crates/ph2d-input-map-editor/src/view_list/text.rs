//! O texto da lista, medido antes de pintar: cada um entra no censo com «coube?».

use ph2d_editor_core::paint::{folga_do_texto, paint_text_elided, paint_text_title_elided};
use ph2d_editor_core::zones::Rect;
use ph2d_text::{FontWeight, TextSystem};
use ph2d_vector::Color;
use ph2d_vector::VectorScene;

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
    paint_text_elided(
        ts,
        scene,
        s,
        r.x,
        r.y + folga_do_texto(r.h, px),
        px,
        r.w,
        color,
    );
    w.min(r.w)
}

/// A largura de `s` em SemiBold (o peso de [`title`]).
pub(crate) fn title_width(ts: &mut TextSystem, s: &str, px: f32) -> f32 {
    ts.prefix_width_weighted(s, px, FontWeight::SEMI_BOLD)
}

/// [`line`] em SemiBold (o nome da acção e o destaque do pé, maquete A): mede e pinta com o MESMO peso.
#[allow(clippy::too_many_arguments)]
pub(crate) fn title(
    census: &mut Census,
    ts: &mut TextSystem,
    scene: &mut VectorScene,
    s: &str,
    r: Rect,
    px: f32,
    color: Color,
) -> f32 {
    let w = title_width(ts, s, px);
    census.texts.push((r, s.to_owned(), w > r.w + f32::EPSILON));
    paint_text_title_elided(
        ts,
        scene,
        s,
        r.x,
        r.y + folga_do_texto(r.h, px),
        px,
        r.w,
        color,
    );
    w.min(r.w)
}
