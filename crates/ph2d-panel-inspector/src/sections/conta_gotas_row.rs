//! **A linha de TEXTO com o conta-gotas dentro da ponta direita da caixa** — o molde do Target da câmera
//! (GEF 20 C2), partilhado pelo Target da bala (GEF 06) e pelo «Ignore up to» do raio (GEF 07 C8).
//! Armado, o botão fica premido; o clique no palco é da app (`conta_gotas`).

use super::*;
use ph2d_editor_core::property_row::Seccao;

/// Pinta a linha `rotulo` com a caixa `input` (id `campo`) e o conta-gotas `gotas`. Devolve o `y` seguinte.
#[allow(clippy::too_many_arguments)]
pub(crate) fn linha(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    x: f32,
    w: f32,
    y: f32,
    (rotulo, campo, input): (TextKey, ph2d_a11y::NodeId, TextInput),
    (gotas, armado): (ph2d_a11y::NodeId, bool),
    seccao: Seccao,
) -> f32 {
    let fim = super::anim_rows::text_row(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        x,
        w,
        y,
        rotulo,
        campo,
        input,
        seccao,
    );
    let caixa = ph2d_editor_core::property_row::caixa_da_linha_pintada(x, w, y, fim, seccao);
    let lado = if ph2d_editor_core::widget::card_ink::card_body().is_some() {
        ph2d_card_stack::metrics::GRID_CELL
    } else {
        (ROW_H_PX * 0.82).min(caixa.h) // LITERAL-PX-OK: o quadrado compacto da linha (o mesmo da câmera)
    };
    let inset = (caixa.h - lado) * 0.5;
    let botao = Rect::new(
        caixa.x + caixa.w - lado - inset,
        caixa.y + inset,
        lado,
        lado,
    );
    paint_icon_button(
        botao,
        IconGlyph::Builtin(IconId::Eyedropper),
        IconButtonStyle::Compact,
        if armado {
            (ButtonState::Pressed, ph2d_editor_core::motion::SETTLED)
        } else {
            store.button_visual(gotas)
        },
        scene,
        theme,
    );
    hit_index.register(gotas, botao);
    fim
}
