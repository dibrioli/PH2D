//! ⭐ **A capa pintada** — a cor do matiz ou a IMAGEM recortada como o `object-fit: cover` (centrada,
//! a escala que enche a caixa sem deformar); metade = faixa no topo, inteira = o cartão todo com o
//! título por cima sobre um véu. O véu escurece o que a imagem REAL pede para o título se ler a
//! 4,5:1 (a linha mais clara da faixa do texto, [`front::cover::veil_alpha`]).

use super::probe::Painted;
use crate::front::cover::{ThumbRef, color_luminance, contrast, veil_alpha};
use crate::front::{self, Fill, Front};
use crate::metrics as m;
use crate::paint::{c, label, text_top};
use ph2d_editor_core::paint::{fill_rounded_rect_radii, paint_text_title};
use ph2d_editor_core::zones::Rect;
use ph2d_kanban_model::{LabelColor, Shade};
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken as T, Theme};
use ph2d_vector::{
    Affine, Brush, Fill as Rule, Gradient, ImageQuality, Point, RoundedRect, VectorScene,
};

/// O véu mínimo sob o título (o da maquete à altura do texto: `0,62 × 36/66`).
const VEIL_FLOOR: f32 = 0.34;

/// A imagem não tem tema: o véu e o título dela são os do tema ESCURO em qualquer tema.
const IMAGE_THEME: Theme = Theme::Dark;

/// O retângulo de cantos `radii` (sup. esq., sup. dir., inf. dir., inf. esq.).
fn rounded(r: Rect, (a, b, cc, d): (f32, f32, f32, f32)) -> RoundedRect {
    RoundedRect::new(
        f64::from(r.x),
        f64::from(r.y),
        f64::from(r.x + r.w),
        f64::from(r.y + r.h),
        (f64::from(a), f64::from(b), f64::from(cc), f64::from(d)),
    )
}

/// O destino da imagem de `size` que enche `r` sem deformar, centrada.
pub(crate) fn cover_dest(size: [u32; 2], r: Rect) -> Rect {
    #[allow(clippy::cast_precision_loss)]
    let (w, h) = (size[0].max(1) as f32, size[1].max(1) as f32);
    let s = (r.w / w).max(r.h / h);
    let (dw, dh) = (w * s, h * s);
    Rect::new(r.x + (r.w - dw) / 2.0, r.y + (r.h - dh) / 2.0, dw, dh)
}

/// O enchimento de `r` com cantos `radii`: a cor, a imagem (recortada) ou a espera.
fn paint_fill(
    scene: &mut VectorScene,
    theme: Theme,
    f: &Front,
    r: Rect,
    radii: (f32, f32, f32, f32),
    seen: &mut Painted,
) {
    let Some(cv) = f.cover else { return };
    match (cv.fill, &f.cover_px) {
        (Fill::Color(hue), _) => {
            let (fill, _) = label(
                theme,
                LabelColor {
                    hue,
                    shade: Shade::Normal,
                },
            );
            fill_rounded_rect_radii(scene, r, radii, fill);
        }
        (Fill::Image(_), Some(ThumbRef(t))) => {
            let d = cover_dest(t.size, r);
            scene.push_clip(&rounded(r, radii));
            let dest = (
                f64::from(d.x),
                f64::from(d.y),
                f64::from(d.x + d.w),
                f64::from(d.y + d.h),
            );
            scene.draw_image_rgba(&t.rgba, t.size[0], t.size[1], dest, ImageQuality::Medium);
            scene.pop_layer();
            seen.image = Some((d, r));
        }
        (Fill::Image(_), None) => {
            fill_rounded_rect_radii(scene, r, radii, c(theme, T::KanbanCardRaised))
        }
    }
}

/// A capa metade em `r` (os cantos de cima do cartão).
pub(super) fn half(scene: &mut VectorScene, theme: Theme, f: &Front, r: Rect, seen: &mut Painted) {
    let k = m::CARD_RADIUS;
    paint_fill(scene, theme, f, r, (k, k, 0.0, 0.0), seen);
}

/// A luminância da linha mais clara da imagem entre `y0` e `y1` (no ecrã) — conservadora: a linha
/// inteira da miniatura, mesmo a parte que o recorte esconde.
fn brightest(t: &ThumbRef, dest: Rect, y0: f32, y1: f32) -> f32 {
    let rows = t.0.row_max.len();
    #[allow(clippy::cast_precision_loss)]
    let to_row = |y: f32| ((y - dest.y) / dest.h * rows as f32).floor();
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let (a, b) = (
        to_row(y0).clamp(0.0, rows as f32) as usize,
        (to_row(y1) + 1.0).clamp(0.0, rows as f32) as usize,
    );
    t.0.row_max[a..b.max(a)].iter().copied().fold(0.0, f32::max)
}

/// ⭐ A capa inteira: o enchimento no cartão todo e o título em baixo, sobre o véu se é imagem.
/// Devolve o fundo pintado.
pub(super) fn full(
    scene: &mut VectorScene,
    ts: &mut TextSystem,
    theme: Theme,
    f: &Front,
    r: Rect,
    seen: &mut Painted,
) -> f32 {
    let k = m::CARD_RADIUS;
    let h = front::full_h(ts, &f.title);
    let area = Rect::new(r.x, r.y, r.w, h);
    paint_fill(scene, theme, f, area, (k, k, k, k), seen);
    let inner = r.w - 2.0 * m::CARD_PAD_X;
    #[allow(clippy::cast_precision_loss)]
    let lines = front::lines(
        ts,
        &f.title,
        front::FULL_TITLE_PX,
        front::FULL_TITLE_WEIGHT,
        inner,
    ) as f32;
    let text_y = area.y + area.h - front::FULL_PAD_BOTTOM - lines * m::LINE_H;
    let image = matches!(f.cover.map(|c| c.fill), Some(Fill::Image(_)));
    let fg = if image {
        let ink = c(IMAGE_THEME, T::KanbanCardFg);
        let veil = c(IMAGE_THEME, T::KanbanShadowFlight);
        let (l_ink, l_veil) = (color_luminance(ink), color_luminance(veil.with_alpha(1.0)));
        let worst = match (&f.cover_px, seen.image) {
            (Some(t), Some((dest, _))) => brightest(t, dest, text_y, area.y + area.h),
            // Ainda a chegar: a espera é o fundo; o véu do pior caso (branco).
            _ => 1.0,
        };
        let a = veil_alpha(worst, l_ink, l_veil, VEIL_FLOOR);
        let under = worst * (1.0 - a) + l_veil * a;
        seen.veil = Some((a, contrast(l_ink, under)));
        scrim(scene, area, text_y, veil.with_alpha(a));
        ink
    } else {
        c(theme, T::KanbanLabelInk)
    };
    let top = text_top(ts, text_y, m::LINE_H, front::FULL_TITLE_PX);
    let x = r.x + m::CARD_PAD_X;
    paint_text_title(ts, scene, &f.title, x, top, front::FULL_TITLE_PX, inner, fg);
    area.y + area.h
}

/// O véu: transparente [`front::FULL_PAD_TOP`] acima do texto e cheio (`solid`) do texto para
/// baixo, dentro dos cantos de baixo do cartão.
fn scrim(scene: &mut VectorScene, area: Rect, text_y: f32, solid: ph2d_vector::Color) {
    let k = m::CARD_RADIUS;
    let fade_y = (text_y - front::FULL_PAD_TOP).max(area.y);
    let band = Rect::new(area.x, fade_y, area.w, area.y + area.h - fade_y);
    let fade = Gradient::new_linear(
        Point::new(0.0, f64::from(fade_y)),
        Point::new(0.0, f64::from(text_y)),
    )
    .with_stops([solid.with_alpha(0.0), solid]);
    scene.inner_mut().fill(
        Rule::NonZero,
        Affine::IDENTITY,
        &Brush::Gradient(fade),
        None,
        &rounded(band, (0.0, 0.0, k, k)),
    );
}
