//! ⭐⭐ **Um painel SEM secções também tem notas** (2026-10-01, ordem do dono: *«a possibilidade de
//! criar notas deve existir em quaisquer painéis de qualquer tipo»*). O Padding é uma ferramenta de
//! imagem sem secção nenhuma: as notas dele pintam-se no FIM do corpo, pela porta de rolagem
//! partilhada (`panel::scroll_area::close` → `notes_chrome::pinta_as_que_sobram`), e é ela que
//! declara o painel anfitrião — o botão direito oferece *Create Note* aqui.

use ph2d_a11y::NodeId;
use ph2d_editor_core::ids::{self as core_ids, CTX_MENU_CREATE_NOTE};
use ph2d_editor_core::interaction::ContextMenuKind;
use ph2d_editor_core::zones::Rect;
use ph2d_host::{PointerButton, PointerEvent, PointerKind, PointerSource};
use ph2d_panel_padding::PaddingPanel;
use ph2d_panel_padding::state::PaddingPanelState;
use ph2d_ui_testkit::MockPanelHost;

const P: NodeId = core_ids::PAD_PANEL;

fn viewport() -> Rect {
    Rect::new(0.0, 0.0, 700.0, 2000.0)
}

fn rect_of(rects: &[(NodeId, Rect)], id: NodeId) -> Option<Rect> {
    rects
        .iter()
        .rev()
        .find(|(n, r)| *n == id && r.w > 0.0 && r.h > 0.0)
        .map(|(_, r)| *r)
}

/// ⭐⭐ **O botão direito abre *Create Note*, a nota nasce SEM secção e pinta-se no fim do corpo.**
/// *Mutação: a porta de rolagem sem as notas do fim ⇒ o menu não abre e a nota nunca aparece.*
#[test]
fn o_padding_cria_e_pinta_uma_nota() {
    let mut host = MockPanelHost::with_panel::<PaddingPanel>();
    let mut st = PaddingPanelState;
    let _ = host.paint::<PaddingPanel>(&mut st, viewport());
    let corpo = host
        .store()
        .panel_rect(P)
        .expect("o painel publicou o rect");
    let (x, y) = (corpo.x + corpo.w - 6.0, corpo.y + corpo.h - 6.0);
    let _ = host.dispatch_pointer_event(PointerEvent {
        kind: PointerKind::Down,
        x,
        y,
        button: PointerButton::Secondary,
        source: PointerSource::Mouse,
        pressure: 1.0,
        timestamp_ns: 1_000_000_000,
    });
    let menu = host.store().context_menu().map(|m| m.kind);
    assert!(
        matches!(menu, Some(ContextMenuKind::CreateNote { panel, section: None }) if panel == P),
        "o botão direito no Padding abriu {menu:?} em vez de criar uma nota sem secção"
    );
    assert!(host.choose_context_menu_row(CTX_MENU_CREATE_NOTE));
    assert_eq!(
        host.store().notes_for_panel(P).len(),
        1,
        "a nota não nasceu"
    );
    let depois = host.paint::<PaddingPanel>(&mut st, viewport());
    let nota = rect_of(&depois, core_ids::note_ids(P).slot[0]).expect("a nota foi pintada");
    assert!(
        corpo.contains(nota.x + nota.w * 0.5, nota.y + 1.0),
        "a nota ({nota:?}) não ficou dentro do painel ({corpo:?})"
    );
}
