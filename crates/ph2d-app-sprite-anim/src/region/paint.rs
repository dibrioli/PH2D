//! **A pintura de uma aba do editor de Region** — a barra (Columns · Rows · a ajuda · Cancel · OK) e a tela: o
//! xadrez, a imagem, a grelha, o que fica fora da área escurecido, a área e as 8 alças. Nada aqui decide.

use super::doc::{Field, RegionDoc};
use crate::editor::paint::{button, field};
use ph2d_editor_core::paint::{fill_rounded_rect, paint_text, resolve, stroke_rect};
use ph2d_editor_core::zones::Rect;
use ph2d_i18n::tr;
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken, Spacing, StrokeToken, Theme, TypeToken};
use ph2d_vector::{ImageQuality, VectorScene};

/// Pinta a aba inteira em `area`.
pub fn paint_doc(
    doc: &mut RegionDoc,
    area: Rect,
    scene: &mut VectorScene,
    text: &mut TextSystem,
    theme: Theme,
) {
    let l = doc.lay_out(area, text);
    fill_rounded_rect(scene, area, 0.0, resolve(ColorToken::PanelBg, theme));
    let font = TypeToken::Sm.px();
    let rotulo = |scene: &mut VectorScene, text: &mut TextSystem, r: Rect, key: &str| {
        paint_text(
            text,
            scene,
            tr(key),
            r.x,
            r.y + (r.h - font) * 0.5,
            font,
            r.w,
            resolve(ColorToken::Text2, theme),
        );
    };
    rotulo(scene, text, l.cols_label, "region_editor.columns");
    rotulo(scene, text, l.rows_label, "region_editor.rows");
    for (r, f) in [(l.cols, Field::Cols), (l.rows, Field::Rows)] {
        let (t, aberto) = doc.field_text(f);
        field(scene, text, theme, r, &t, aberto.then_some(t.as_str()));
    }
    let ajuda = if doc.alt {
        "region_editor.hint_alt"
    } else {
        "region_editor.hint"
    };
    rotulo(scene, text, l.hint, ajuda);
    button(
        scene,
        text,
        theme,
        l.cancel,
        tr("region_editor.cancel"),
        false,
    );
    button(scene, text, theme, l.ok, tr("region_editor.ok"), true);

    let c = l.canvas;
    scene.push_clip(&ph2d_vector::Rect::new(
        f64::from(c.x),
        f64::from(c.y),
        f64::from(c.x + c.w),
        f64::from(c.y + c.h),
    ));
    fill_rounded_rect(scene, c, 0.0, resolve(ColorToken::Bg0, theme));
    let img = doc.image_on_screen();
    checker(scene, intersect(img, c), theme);
    match doc.image.clone() {
        Some(sheet) => scene.draw_stable_image(
            &sheet.image,
            (
                f64::from(img.x),
                f64::from(img.y),
                f64::from(img.x + img.w),
                f64::from(img.y + img.h),
            ),
            if doc.zoom >= 1.0 {
                ImageQuality::Low
            } else {
                ImageQuality::Medium
            },
        ),
        None => rotulo(scene, text, c, "region_editor.no_pixels"),
    }
    grid(scene, doc, img, theme);
    let r = doc.rect_on_screen();
    shade_outside(scene, img, r, theme);
    stroke_rect(
        scene,
        r,
        StrokeToken::Thick.px(),
        resolve(ColorToken::Accent, theme),
    );
    for (_, _, h) in doc.handles() {
        fill_rounded_rect(scene, h, 0.0, resolve(ColorToken::Text1, theme));
        stroke_rect(
            scene,
            h,
            StrokeToken::Hairline.px(),
            resolve(ColorToken::Accent, theme),
        );
    }
    scene.pop_layer();
}

fn intersect(a: Rect, b: Rect) -> Rect {
    let (x0, y0) = (a.x.max(b.x), a.y.max(b.y));
    let (x1, y1) = ((a.x + a.w).min(b.x + b.w), (a.y + a.h).min(b.y + b.h));
    Rect::new(x0, y0, (x1 - x0).max(0.0), (y1 - y0).max(0.0))
}

/// O xadrez da transparência sob a imagem (`r` já preso à tela).
fn checker(scene: &mut VectorScene, r: Rect, theme: Theme) {
    let s = Spacing::Md.px();
    if r.w <= 0.0 || r.h <= 0.0 {
        return;
    }
    fill_rounded_rect(scene, r, 0.0, resolve(ColorToken::Bg2, theme));
    let escuro = resolve(ColorToken::Bg3, theme);
    let (nx, ny) = ((r.w / s).ceil() as u32, (r.h / s).ceil() as u32);
    for j in 0..ny {
        for i in (j % 2..nx).step_by(2) {
            let x = r.x + i as f32 * s;
            let y = r.y + j as f32 * s;
            let q = Rect::new(x, y, s.min(r.x + r.w - x), s.min(r.y + r.h - y));
            fill_rounded_rect(scene, q, 0.0, escuro);
        }
    }
}

/// As linhas da grelha de snap (nenhuma com 1 × 1).
fn grid(scene: &mut VectorScene, doc: &RegionDoc, img: Rect, theme: Theme) {
    let w = StrokeToken::Hairline.px();
    // Por cima do xadrez e da arte: o tom do texto secundário lê-se nos dois.
    let cor = resolve(ColorToken::Text2, theme);
    for i in 1..doc.grid[0] {
        let x = img.x + doc.grid_line(0, i) * doc.zoom;
        fill_rounded_rect(scene, Rect::new(x, img.y, w, img.h), 0.0, cor);
    }
    for j in 1..doc.grid[1] {
        let y = img.y + doc.grid_line(1, j) * doc.zoom;
        fill_rounded_rect(scene, Rect::new(img.x, y, img.w, w), 0.0, cor);
    }
}

/// O que fica fora da área escurece: o objecto mostra só o que está dentro.
fn shade_outside(scene: &mut VectorScene, img: Rect, r: Rect, theme: Theme) {
    let sombra = resolve(ColorToken::BgScrim, theme);
    let (x1, y1) = (img.x + img.w, img.y + img.h);
    let (rx1, ry1) = (r.x + r.w, r.y + r.h);
    for q in [
        Rect::new(img.x, img.y, img.w, r.y - img.y),
        Rect::new(img.x, ry1, img.w, y1 - ry1),
        Rect::new(img.x, r.y, r.x - img.x, r.h),
        Rect::new(rx1, r.y, x1 - rx1, r.h),
    ] {
        if q.w > 0.0 && q.h > 0.0 {
            fill_rounded_rect(scene, q, 0.0, sombra);
        }
    }
}
