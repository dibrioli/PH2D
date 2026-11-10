//! **Cel ▸ Tween… pelo caminho do produto** (P13 T): o botão da linha do tempo emite o pedido
//! `FrameAction(Tween)` (o mesmo que o clique), o diálogo abre em cartões, Frames = 3 e OK ⇒ três
//! quadros REAIS no documento num só passo; um desfazer tira-os todos.

use ph2d_editor_core::ids;
use ph2d_editor_core::interaction::WidgetEvent;
use ph2d_editor_core::pixel_panel::{PixelDialogKind, PixelFrameAction as A, PixelRequest};

use crate::selecao::Palco;

#[test]
fn o_menu_tween_poe_os_quadros_no_documento_num_passo() {
    let mut p = Palco::limpo();
    p.pede(PixelRequest::FrameAction(A::NewFrame));
    p.pede(PixelRequest::FrameAction(A::Previous));
    let (quadros, passos) = (p.sprite().frames().len(), p.passos());
    assert_eq!(quadros, 2, "A e B");
    let botao = ph2d_panel_pixel_timeline::ids::ACTIONS
        .iter()
        .find(|(a, ..)| *a == A::Tween)
        .expect("o botão Tween… está na barra");
    assert_eq!(botao.3, "panel.pixel.timeline.tween");
    p.pede(PixelRequest::FrameAction(A::Tween));
    assert_eq!(
        p.hero.store.pixel_edit_dialog().kind,
        PixelDialogKind::Tween,
        "o diálogo abriu"
    );
    p.hero
        .store
        .set_number_value(ids::PIXEL_EDIT_TWEEN_FRAMES, 3.0);
    p.hero
        .apply_event(WidgetEvent::ValueChanged(ids::PIXEL_EDIT_TWEEN_FRAMES));
    p.hero
        .apply_event(WidgetEvent::Click(ids::PIXEL_EDIT_TWEEN_EASES[3].1));
    p.quadro();
    p.hero.apply_event(WidgetEvent::Click(ids::PIXEL_EDIT_OK));
    p.quadro();
    assert_eq!(
        p.sprite().frames().len(),
        quadros + 3,
        "três quadros do meio"
    );
    assert_eq!(p.passos(), passos + 1, "um passo");
    assert!(p.app.undo(p.key), "desfaz");
    assert_eq!(
        p.sprite().frames().len(),
        quadros,
        "um desfazer tira-os todos"
    );
}
