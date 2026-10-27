//! ⭐ **A folha inteira com a grelha por cima** ([plano GEF 15](../../../../docs/GameEngineFuncional/15_uso_da_imagem_a_vista_plano.md)
//! D2): o corte vê-se, as células numeram-se e, numa imagem parada, clicar uma escolhe o quadro mostrado. A imagem
//! é a da ranhura Image ([`crate::slot_image`]), com a mesma textura estável.

use super::*;
use ph2d_vector::ImageQuality;

/// Uma célula mais baixa que isto (em px de ecrã) não leva número: ele não caberia.
fn number_fits(cell_h: f32) -> bool {
    cell_h >= TypeToken::Xs.px() + Spacing::Xxs.px()
}

/// Pinta a miniatura da imagem de `info` com a grelha `hframes × vframes`, a célula `info.frame` marcada e, com
/// `clickable`, cada célula como alvo. Devolve o `y` seguinte (o mesmo se a sprite não tem imagem da
/// biblioteca).
#[allow(clippy::too_many_arguments)]
pub(crate) fn paint_sheet_thumb(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    x: f32,
    w: f32,
    y: f32,
    info: &InspectorSpriteInfo,
    clickable: bool,
) -> f32 {
    let Some(img) = crate::slot_image::current_slot_image(info.entity_bits) else {
        return y;
    };
    let Some(tex) = crate::slot_image::stable(&img) else {
        return y;
    };
    let (iw, ih) = (img.w.max(1) as f32, img.h.max(1) as f32);
    // Larga como a coluna, nunca mais alta que larga.
    let k = (w / iw).min(w / ih);
    let (dw, dh) = (iw * k, ih * k);
    let area = Rect::new(x + (w - dw) * 0.5, y, dw, dh);
    let radius = ph2d_editor_core::paint::frame_radius(theme, Radius::Sm.px());
    fill_rounded_rect(scene, area, radius, resolve(ColorToken::Bg2, theme));
    let (x0, y0) = (f64::from(area.x), f64::from(area.y));
    scene.draw_stable_image(
        &tex,
        (x0, y0, x0 + f64::from(dw), y0 + f64::from(dh)),
        ImageQuality::Medium,
    );
    let (cols, rows) = (info.hframes.max(1), info.vframes.max(1));
    let (cw, ch) = (dw / cols as f32, dh / rows as f32);
    let line = resolve(ColorToken::Border, theme);
    for c in 1..cols {
        let lx = f64::from(area.x + cw * c as f32);
        scene.fill_rect(
            ph2d_vector::Rect::new(lx, y0, lx + f64::from(SLOT_BORDER_PX), y0 + f64::from(dh)),
            line,
        );
    }
    for r in 1..rows {
        let ly = f64::from(area.y + ch * r as f32);
        scene.fill_rect(
            ph2d_vector::Rect::new(x0, ly, x0 + f64::from(dw), ly + f64::from(SLOT_BORDER_PX)),
            line,
        );
    }
    let numbers = number_fits(ch) && cols * rows > 1;
    let font = TypeToken::Xs.px();
    let pad = Spacing::Xxs.px();
    for i in 0..cols * rows {
        let cell = Rect::new(
            area.x + cw * (i % cols) as f32,
            area.y + ch * (i / cols) as f32,
            cw,
            ch,
        );
        if numbers {
            paint_text(
                text_system,
                scene,
                // A célula como os campos «From/To (cell)» do mesmo cartão a escrevem (a partir de 0).
                &i.to_string(),
                cell.x + pad,
                cell.y + pad,
                font,
                (cell.w - pad).max(0.0),
                resolve(ColorToken::Text1, theme),
            );
        }
        if clickable {
            hit_index.register(ids::insp_image_cell(i), cell);
        }
    }
    if cols * rows > 1 && info.frame < cols * rows {
        let i = info.frame;
        let cell = Rect::new(
            area.x + cw * (i % cols) as f32,
            area.y + ch * (i / cols) as f32,
            cw,
            ch,
        );
        outline(scene, cell, resolve(ColorToken::Accent, theme));
    }
    area.y + dh + ph2d_tokens::control_gap_px()
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
