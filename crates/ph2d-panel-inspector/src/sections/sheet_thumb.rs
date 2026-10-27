//! ⭐ **A imagem sob o seletor «Use»** ([plano GEF 15](../../../../docs/GameEngineFuncional/15_uso_da_imagem_a_vista_plano.md)
//! D2): animada — a folha inteira com a grelha por cima e as células numeradas (o corte vê-se); parada — só
//! «Frame ‹ n / N ›» (smoke do dono 27/10: a grelha numa parada era ruído). A folha é a da ranhura Image
//! ([`crate::slot_image`]), com a mesma textura estável.

use super::*;
use ph2d_i18n::{tr, tr_with};
use ph2d_vector::ImageQuality;

/// Uma célula mais baixa que isto (em px de ecrã) não leva número: ele não caberia.
fn number_fits(cell_h: f32) -> bool {
    cell_h >= TypeToken::Xs.px() + Spacing::Xxs.px()
}

/// A área onde cabe uma imagem `iw × ih` na coluna `w`, centrada, nunca mais alta que larga.
fn fit(x: f32, y: f32, w: f32, iw: f32, ih: f32) -> Rect {
    let k = (w / iw).min(w / ih);
    let (dw, dh) = (iw * k, ih * k);
    Rect::new(x + (w - dw) * 0.5, y, dw, dh)
}

/// Pinta a folha de `info` com a grelha `hframes × vframes` e a célula `info.frame` marcada. Devolve o `y`
/// seguinte (o mesmo se a sprite não tem imagem da biblioteca).
#[allow(clippy::too_many_arguments)]
pub(crate) fn paint_sheet_thumb(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    x: f32,
    w: f32,
    y: f32,
    info: &InspectorSpriteInfo,
) -> f32 {
    let Some(img) = crate::slot_image::current_slot_image(info.entity_bits) else {
        return y;
    };
    let Some(tex) = crate::slot_image::stable(&img) else {
        return y;
    };
    let area = fit(x, y, w, img.w.max(1) as f32, img.h.max(1) as f32);
    let radius = ph2d_editor_core::paint::frame_radius(theme, Radius::Sm.px());
    fill_rounded_rect(scene, area, radius, resolve(ColorToken::Bg2, theme));
    let (x0, y0) = (f64::from(area.x), f64::from(area.y));
    scene.draw_stable_image(
        &tex,
        (x0, y0, x0 + f64::from(area.w), y0 + f64::from(area.h)),
        ImageQuality::Medium,
    );
    let (cols, rows) = (info.hframes.max(1), info.vframes.max(1));
    let (cw, ch) = (area.w / cols as f32, area.h / rows as f32);
    let line = resolve(ColorToken::Border, theme);
    for c in 1..cols {
        let lx = f64::from(area.x + cw * c as f32);
        scene.fill_rect(
            ph2d_vector::Rect::new(
                lx,
                y0,
                lx + f64::from(SLOT_BORDER_PX),
                y0 + f64::from(area.h),
            ),
            line,
        );
    }
    for r in 1..rows {
        let ly = f64::from(area.y + ch * r as f32);
        scene.fill_rect(
            ph2d_vector::Rect::new(
                x0,
                ly,
                x0 + f64::from(area.w),
                ly + f64::from(SLOT_BORDER_PX),
            ),
            line,
        );
    }
    let cell = |i: u32| {
        Rect::new(
            area.x + cw * (i % cols) as f32,
            area.y + ch * (i / cols) as f32,
            cw,
            ch,
        )
    };
    if number_fits(ch) && cols * rows > 1 {
        let pad = Spacing::Xxs.px();
        for i in 0..cols * rows {
            let c = cell(i);
            paint_text(
                text_system,
                scene,
                // A célula como os campos «From/To (cell)» do mesmo cartão a escrevem (a partir de 0).
                &i.to_string(),
                c.x + pad,
                c.y + pad,
                TypeToken::Xs.px(),
                (c.w - pad).max(0.0),
                resolve(ColorToken::Text1, theme),
            );
        }
    }
    if cols * rows > 1 && info.frame < cols * rows {
        outline(scene, cell(info.frame), resolve(ColorToken::Accent, theme));
    }
    area.y + area.h + ph2d_tokens::control_gap_px()
}

/// ⭐ **Uma parada numa folha: só «Frame ‹ n / N ›»** (smoke do dono 27/10, «simplificar ao máximo»): o quadro
/// vê-se no ecrã e na ranhura Image — nem grelha, nem uma segunda imagem. Devolve o `y` seguinte.
#[allow(clippy::too_many_arguments)]
pub(crate) fn paint_still_frame(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    x: f32,
    w: f32,
    y: f32,
    info: &InspectorSpriteInfo,
) -> f32 {
    let n = info.hframes.max(1) * info.vframes.max(1);
    if n < 2 {
        return y;
    }
    let label = tr("panel.inspector.image.frame");
    let sec = ph2d_editor_core::property_row::Seccao::medida(text_system, 1, &[label]);
    let linha = ph2d_editor_core::property_row::paint_label_row(
        scene,
        text_system,
        theme,
        x,
        w,
        y,
        ROW_H_PX,
        label,
        sec,
    );
    let c = linha.control;
    let side = c.h;
    let [prev, next] = ids::INSP_IMAGE_FRAME_STEP;
    for (id, key, rx) in [
        (prev, "panel.inspector.image.frame_prev", c.x),
        (next, "panel.inspector.image.frame_next", c.x + c.w - side),
    ] {
        let r = Rect::new(rx, c.y, side, side);
        hit_index.register(id, r);
        let btn = Button::new(id, tr(key))
            .kind(ButtonKind::Default)
            .visual(store.button_visual(id));
        paint_button(&btn, r, scene, text_system, theme);
    }
    let conta = tr_with(
        "panel.inspector.image.frame_of",
        &[("n", &(info.frame.min(n - 1) + 1)), ("total", &n)],
    );
    let font = TypeToken::Sm.px();
    paint_text(
        text_system,
        scene,
        &conta,
        c.x + side + Spacing::Xs.px(),
        c.y + (c.h - font) * 0.5,
        font,
        (c.w - side * 2.0 - Spacing::Xs.px() * 2.0).max(0.0),
        resolve(ColorToken::Text1, theme),
    );
    y + ROW_H_PX + ph2d_tokens::control_gap_px()
}

/// A moldura do quadro mostrado — quatro tiras, em todos os temas (a moldura de cartão é plana no moderno).
fn outline(scene: &mut VectorScene, r: Rect, color: ph2d_vector::Color) {
    let t = f64::from(SLOT_BORDER_PX + SLOT_BORDER_PX);
    let (x0, y0) = (f64::from(r.x), f64::from(r.y));
    let (x1, y1) = (x0 + f64::from(r.w), y0 + f64::from(r.h));
    for (a, b, c, d) in [
        (x0, y0, x1, y0 + t),
        (x0, y1 - t, x1, y1),
        (x0, y0, x0 + t, y1),
        (x1 - t, y0, x1, y1),
    ] {
        scene.fill_rect(ph2d_vector::Rect::new(a, b, c, d), color);
    }
}
