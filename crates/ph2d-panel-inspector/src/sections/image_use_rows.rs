//! ⭐ **Os campos de quadros POR USO** ([plano GEF 13](../../../../docs/GameEngineFuncional/13_isolamento_por_uso_plano.md)
//! D4): parada = o corte da imagem e a célula; repetida = nenhum, e o aviso + «Make it Animated» se a imagem tem
//! várias células; animada = o cartão Animation. Irmão do `render_source.rs` (tecto de LOC).

use super::*;
use ph2d_editor_core::InspectorImageUse;
use ph2d_i18n::{tr, tr_with};

/// Pinta o que o uso de `info` pede. Devolve o `y` seguinte.
#[allow(clippy::too_many_arguments)]
pub(super) fn paint_image_use_rows(
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
    if crate::state::current_inspector_anim().is_some() {
        return y;
    }
    match info.image_use {
        InspectorImageUse::Animated => y,
        InspectorImageUse::Repeated if info.image_cells > 1 => {
            let n = info.image_cells;
            let y = super::rows::aviso(
                scene,
                text_system,
                theme,
                x,
                w,
                y,
                &tr_with("panel.inspector.image.animated_in_block", &[("n", &n)]),
                ColorToken::Warn,
            );
            let label = tr("panel.inspector.image.make_animated");
            let rect = ph2d_editor_core::property_row::caixa_do_botao(text_system, x, w, y, label);
            let id = ids::INSP_IMAGE_MAKE_ANIMATED;
            hit_index.register(id, rect);
            let btn = Button::new(id, label)
                .kind(ButtonKind::Default)
                .visual(store.button_visual(id));
            paint_button(&btn, rect, scene, text_system, theme);
            ph2d_editor_core::property_row::abaixo_do_botao(rect)
        }
        InspectorImageUse::Repeated => y,
        InspectorImageUse::Still => {
            let y = super::rows::aviso(
                scene,
                text_system,
                theme,
                x,
                w,
                y,
                tr("panel.inspector.image.cut_of_the_image"),
                ColorToken::Text3,
            );
            let sec = super::sprite_pose::seccao(text_system);
            super::sprite_pose::paint_sheet_grid_rows(
                scene,
                text_system,
                theme,
                hit_index,
                store,
                x,
                w,
                y,
                info.hframes,
                info.vframes,
                info.hframes.max(1) * info.vframes.max(1) > 1,
                sec,
            )
        }
    }
}
