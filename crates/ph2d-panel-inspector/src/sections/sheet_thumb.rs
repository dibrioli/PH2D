//! ⭐ **O quadro de uma parada sob o seletor «Use»** ([plano GEF 15](../../../../docs/GameEngineFuncional/15_uso_da_imagem_a_vista_plano.md)
//! D2): só «Frame ‹ n / N ›». Nem a parada nem a animada mostram a folha com as células (dono 27/10: «em
//! Animated exibir as imagens de quadros não é necessário») — o corte lê-se em H/V Frames.

use super::*;
use ph2d_i18n::{tr, tr_with};

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
