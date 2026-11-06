//! **Edit in Painter**: a composição entre a imagem da sprite e o documento do Painter. A lei mora em `ph2d_sprite_edit`.

use ph2d_i18n::tr;
use ph2d_sprite_edit::achatados::Achatados;
use ph2d_sprite_edit::imagem;
use ph2d_tool_painter::PainterTool;
use ph2d_tool_painter::tool::persist::PaintedDocument;
use std::cell::RefCell;

thread_local! {
    /// As camadas que uma imagem vinda de fora tirou, à espera do Undo.
    static ACHATADOS: RefCell<Achatados<PaintedDocument>> = RefCell::default();
}

/// Nos objectos que receberam píxeis por fora, a imagem vence as camadas, com aviso.
pub(crate) fn achata(
    sim: &mut ph2d_ecs::SimWorld,
    painter: &mut PainterTool,
    t: &imagem::Trocados,
    toasts: &mut ph2d_editor_core::ToastQueue,
) {
    let world = sim.world_mut();
    let ids = (t.antigo, t.novo);
    let algum = ACHATADOS
        .with(|a| (a.borrow_mut()).achata(world, &t.quem, ids, |b| painter.tira_documento(b)));
    if algum {
        toasts.push(ph2d_editor_core::Toast::warning(tr(
            "shell.image_edit.layers_flattened",
        )));
    }
}

/// Depois de um restauro do undo: as camadas acompanham a imagem que voltou.
pub(crate) fn reconcilia_achatados(gfx: &mut crate::AppGfx) {
    let Some(p) = ph2d_app_painter::painter_bridge_queries::o_painter(&mut gfx.tools) else {
        return;
    };
    let p = RefCell::new(p);
    ACHATADOS.with(|a| {
        a.borrow_mut().reconcilia(
            gfx.sim.world_mut(),
            &gfx.atlas_asset_map,
            |b| p.borrow_mut().tira_documento(b),
            |b, d| p.borrow_mut().install_document(b, d),
        );
    });
}

/// O que esperava o Undo não atravessa um load.
pub(crate) fn esquece_o_projecto_que_saiu() {
    ACHATADOS.with(|a| a.borrow_mut().esquece());
}
