//! A amostra de cor de uma camada do **Composite Brush** e o picker partilhado — abrir, semear e
//! ler de volta. Espelho do que o `event/shape_layer_picker.rs` faz para o Per-Layer Color, com uma
//! diferença de PAYLOAD que é deliberada.
//!
//! ⚠️ **O payload é `"r,g,b"` e não `"i,r,g,b"`**, ao contrário do irmão da Shape: ali há UM id de
//! fábrica por camada gerado de um índice e o tool volta a parsear o índice; aqui cada posição tem
//! o **id dela** no array `PAINTER_BRUSH_COMPOSITE_COLOR`, logo o índice já está na chave e
//! reescrevê-lo no valor seria a segunda resposta à mesma pergunta — a que diverge no dia em que
//! alguém reordenar a lista. É exactamente o formato do `PAINTER_COLOR_THUMB` do pincel.

use ph2d_a11y::NodeId;
use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::ids as core_ids;
use ph2d_editor_core::panel::{PaintCtx, PanelHostInternal};
use ph2d_editor_core::tool::PanelEvent;

fn enc(c: f32) -> u8 {
    (c.clamp(0.0, 1.0) * 255.0 + 0.5) as u8 // LITERAL-PX-OK: sRGB 8-bit normalize
}

/// `id` é a amostra de cor de uma camada? Devolve a POSIÇÃO dela.
pub(crate) fn posicao(id: NodeId) -> Option<usize> {
    ph2d_tool_painter::ids::PAINTER_BRUSH_COMPOSITE_COLOR
        .iter()
        .position(|x| *x == id)
}

/// A amostra da camada `pos` foi clicada → liga/desliga o picker partilhado apontado a ela,
/// semeado com a cor que ela mostra.
pub(crate) fn on_swatch_click(host: &mut dyn PanelHostInternal, id: NodeId, pos: usize) {
    if host.store().picker_target() == Some(id) {
        host.store_mut().set_picker_target(None);
        return;
    }
    let rgba = crate::state::current_brush()
        .map(|b| {
            let c = b.composite_color[pos];
            [enc(c[0]), enc(c[1]), enc(c[2]), 255u8]
        })
        .unwrap_or([0, 0, 0, 255]);
    let store = host.store_mut();
    store.set_blender_value(
        core_ids::INSP_BLENDER_PICKER,
        ph2d_tokens::ColorValue::from_rgba8(rgba[0], rgba[1], rgba[2], rgba[3]),
    );
    store.set_widget_color(id, rgba);
    store.set_picker_target(Some(id));
}

/// Com o picker apontado a esta amostra, encaminha a cor VIVA dele para a ferramenta — uma vez por
/// quadro e **só quando ela mudou**, senão cada quadro empurraria uma edição idêntica ao barramento.
pub(crate) fn readback(ctx: &mut PaintCtx, swatch_id: NodeId, cur: [f32; 3]) {
    if ctx.host.store().picker_target() != Some(swatch_id) {
        return;
    }
    let Some(picked) = ctx.host.store().widget_color(swatch_id) else {
        return;
    };
    if [enc(cur[0]), enc(cur[1]), enc(cur[2])] == [picked[0], picked[1], picked[2]] {
        return;
    }
    ctx.host
        .bus_mut()
        .push(EditorAction::ToolPanelEvent(PanelEvent::SelectOption(
            swatch_id,
            format!("{},{},{}", picked[0], picked[1], picked[2]),
        )));
}
