//! **A folha do editor de animações**: a imagem, a grelha, a ordem da animação aberta e as faixas de cor de quem
//! usa cada célula. Irmã do [`super::paint`]: lê a disposição e não decide nada.

use super::bands::{anim_color, shown_bands};
use super::doc::{AnimDoc, Layout};
use super::paint::tom;
use ph2d_editor_core::paint::{
    fill_rounded_rect, frame_radius, label_column_width, paint_text, paint_text_centered,
    rect_for_label, stroke_rect,
};
use ph2d_editor_core::zones::Rect;
use ph2d_i18n::{tr, tr_with};
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken, ROW_H_PX, Radius, Spacing, StrokeToken, Theme, TypeToken};
use ph2d_vector::{ImageQuality, VectorScene};

/// A opacidade das faixas das animações que não estão abertas.
const OTHER_BAND_ALPHA: f32 = 0.7; // LITERAL-PX-OK: opacidade

/// O que a última pintura pôs numa célula (os gates leem-no).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct PaintedCell {
    pub cell: u32,
    /// As animações das faixas à vista, de baixo para cima.
    pub bands: Vec<usize>,
    /// O número do «+N».
    pub extra: usize,
}

/// Pinta a folha e devolve o que pôs em cada célula usada.
pub(super) fn paint_sheet(
    doc: &AnimDoc,
    l: &Layout,
    users: &[Vec<usize>],
    scene: &mut VectorScene,
    text: &mut TextSystem,
    theme: Theme,
) -> Vec<PaintedCell> {
    let font = TypeToken::Sm.px();
    let Some(sheet) = doc.snap.sheet.clone() else {
        paint_text(
            text,
            scene,
            tr("anim_editor.no_pixels"),
            l.sheet.x,
            l.sheet.y,
            font,
            l.sheet.w,
            tom(ColorToken::Text3, theme),
        );
        return Vec::new();
    };
    fill_rounded_rect(scene, l.sheet, 0.0, tom(ColorToken::Bg0, theme));
    scene.draw_stable_image(
        &sheet.image,
        (
            f64::from(l.sheet.x),
            f64::from(l.sheet.y),
            f64::from(l.sheet.x + l.sheet.w),
            f64::from(l.sheet.y + l.sheet.h),
        ),
        ImageQuality::Medium,
    );
    for c in 0..users.len() as u32 {
        stroke_rect(
            scene,
            doc.cell_rect(c),
            StrokeToken::Hairline.px(),
            tom(ColorToken::Border, theme),
        );
    }
    let rows = doc.snap.info.as_ref().map_or(&[][..], |i| &i.rows);
    let color_of = |i: usize| anim_color(rows.get(i).map_or(0, |r| r.color));
    let frames = doc.shown_frames();
    let mut painted = Vec::new();
    for (c, u) in users.iter().enumerate() {
        if u.is_empty() {
            continue;
        }
        let r = doc.cell_rect(c as u32);
        let (bands, extra) = shown_bands(u);
        let h = Spacing::Sm.px();
        for (k, i) in bands.iter().enumerate() {
            let y = r.y + r.h - (k + 1) as f32 * h;
            let color = tom(color_of(*i), theme);
            let color = if *i == doc.selected {
                color
            } else {
                color.with_alpha(OTHER_BAND_ALPHA)
            };
            fill_rounded_rect(scene, Rect::new(r.x, y, r.w, h), 0.0, color);
        }
        if extra > 0 {
            let top = r.y + r.h - bands.len() as f32 * h;
            let plus = tr_with("anim_editor.more_bands", &[("n", &extra)]);
            let w = rect_for_label(label_column_width(text, font, [plus.as_str()]));
            let n = Rect::new(r.x + r.w - w, top - ROW_H_PX, w, ROW_H_PX);
            paint_text_centered(text, scene, &plus, n, font, tom(ColorToken::Text1, theme));
        }
        if u.first() == Some(&doc.selected) && frames.contains(&(c as u32)) {
            let color = tom(color_of(doc.selected), theme);
            stroke_rect(scene, r, StrokeToken::Thick.px(), color);
            let order: Vec<String> = frames
                .iter()
                .enumerate()
                .filter(|(_, f)| **f == c as u32)
                .map(|(i, _)| (i + 1).to_string())
                .collect();
            paint_text(
                text,
                scene,
                &order.join(tr("anim_editor.order_sep")),
                r.x + Spacing::Sm.px(),
                r.y,
                font,
                r.w,
                tom(ColorToken::Text1, theme),
            );
        }
        painted.push(PaintedCell {
            cell: c as u32,
            bands: bands.to_vec(),
            extra,
        });
    }
    painted
}

/// A lista completa de quem usa a célula sob o rato, junto dele e sem sair da aba.
pub(super) fn paint_hover_list(
    doc: &AnimDoc,
    l: &Layout,
    names: &[String],
    scene: &mut VectorScene,
    text: &mut TextSystem,
    theme: Theme,
) {
    let Some((x, y)) = doc.hover.filter(|_| !names.is_empty()) else {
        return;
    };
    let font = TypeToken::Sm.px();
    let pad = Spacing::Sm.px();
    let w = rect_for_label(label_column_width(
        text,
        font,
        names.iter().map(String::as_str),
    ));
    let h = names.len() as f32 * ROW_H_PX;
    let a = l.area;
    let bx = (x + pad).min(a.x + a.w - w).max(a.x);
    let by = (y + pad).min(a.y + a.h - h).max(a.y);
    let caixa = Rect::new(bx, by, w, h);
    fill_rounded_rect(
        scene,
        caixa,
        frame_radius(theme, Radius::Sm.px()),
        tom(ColorToken::Bg3, theme),
    );
    for (i, n) in names.iter().enumerate() {
        paint_text(
            text,
            scene,
            n,
            bx + pad,
            by + i as f32 * ROW_H_PX,
            font,
            w,
            tom(ColorToken::Text1, theme),
        );
    }
}
