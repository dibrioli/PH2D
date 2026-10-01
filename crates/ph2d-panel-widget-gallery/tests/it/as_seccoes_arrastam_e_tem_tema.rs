//! ⭐⭐⭐ **As secções da galeria abrem o menu de TEMA e ARRASTAM pela pega** — o gesto REAL, pelo
//! despachante, com os rects que a pintura registou (ordem do dono, 2026-09-30: *«siga com os
//! outros painéis»*). A galeria é a fonte de verdade do cromo (DIRETRIZ §5.2): uma secção dela que
//! não se arrasta seria a galeria a ensinar o que o app já não faz.

use ph2d_a11y::NodeId;
use ph2d_editor_core::ids;
use ph2d_editor_core::interaction::ContextMenuKind;
use ph2d_editor_core::panel::{Panel, PanelHostInternal};
use ph2d_editor_core::zones::Rect;
use ph2d_host::{PointerButton, PointerEvent, PointerKind, PointerSource};
use ph2d_panel_widget_gallery::WidgetGalleryPanel;
use ph2d_panel_widget_gallery::state::WidgetGalleryState;
use ph2d_ui_testkit::MockPanelHost;

const SEC: u128 = 1_000_000_000;

fn viewport() -> Rect {
    Rect::new(0.0, 0.0, 1600.0, 4000.0)
}

fn pointer(kind: PointerKind, button: PointerButton, x: f32, y: f32, t: u128) -> PointerEvent {
    PointerEvent {
        kind,
        x,
        y,
        button,
        source: PointerSource::Mouse,
        pressure: 1.0,
        timestamp_ns: t,
    }
}

fn rect_of(rects: &[(NodeId, Rect)], id: NodeId) -> Option<Rect> {
    rects
        .iter()
        .rev()
        .find(|(n, r)| *n == id && r.w > 0.0 && r.h > 0.0)
        .map(|(_, r)| *r)
}

fn a_vista(rects: &[(NodeId, Rect)]) -> Vec<(NodeId, Rect)> {
    let mut v: Vec<(NodeId, Rect)> = ids::SECTION_IDS
        .iter()
        .filter_map(|&id| rect_of(rects, id).map(|r| (id, r)))
        .collect();
    v.sort_by(|a, b| a.1.y.total_cmp(&b.1.y));
    v
}

fn aberta() -> (MockPanelHost, WidgetGalleryState, Vec<(NodeId, Rect)>) {
    let mut host = MockPanelHost::with_panel::<WidgetGalleryPanel>();
    host.set_panel_visible(WidgetGalleryPanel::ID, true);
    let mut st = WidgetGalleryState::default();
    let rects = host.paint::<WidgetGalleryPanel>(&mut st, viewport());
    (host, st, rects)
}

/// ⭐⭐ **O botão direito no título de uma secção da galeria abre o menu de tema DELA, e ela tem
/// pega.** *Mutação: o `paint_collapsible_header` a registar pelo `hit_index.register` de antes ⇒
/// a pega não existe no índice.*
#[test]
fn o_botao_direito_no_titulo_abre_o_menu_de_tema_e_ha_pega() {
    let (mut host, _, rects) = aberta();
    let vista = a_vista(&rects);
    assert!(
        vista.len() >= 3,
        "fixtura: a galeria tem de pintar três secções ou mais (pintou {})",
        vista.len()
    );
    let (alvo, cab) = vista[1];
    assert!(
        rect_of(&rects, ids::grip_de(alvo)).is_some(),
        "a secção {alvo:?} não registou a pega"
    );
    let _ = host.dispatch_pointer_event(pointer(
        PointerKind::Down,
        PointerButton::Secondary,
        cab.x + cab.w * 0.25,
        cab.y + cab.h * 0.5,
        SEC,
    ));
    let menu = host.store().context_menu().map(|m| m.kind);
    assert!(
        matches!(menu, Some(ContextMenuKind::SectionOutline { section }) if section == alvo),
        "o botão direito no título abriu {menu:?} em vez do menu de tema da secção"
    );
}

/// ⭐⭐ **Arrastar a pega da segunda secção para cima da primeira põe-na à frente**, e a pintura
/// seguinte segue a ordem gravada. *Mutação: o corpo da galeria pela ordem natural (o `ordem` a
/// devolver o `SECTION_IDS`) ⇒ a segunda continua atrás.*
#[test]
fn arrastar_a_pega_poe_a_seccao_onde_se_larga() {
    let (mut host, mut st, rects) = aberta();
    let vista = a_vista(&rects);
    assert!(vista.len() >= 2, "fixtura: duas secções à vista");
    let (primeira, cab1) = vista[0];
    let (segunda, _) = vista[1];
    let pega = rect_of(&rects, ids::grip_de(segunda)).expect("a segunda secção tem pega");
    let (px, py) = (pega.x + pega.w * 0.5, pega.y + pega.h * 0.5);
    let alvo_y = cab1.y + 1.0;
    for (kind, y, t) in [
        (PointerKind::Down, py, SEC),
        (PointerKind::Move, alvo_y, SEC + 1),
        (PointerKind::Up, alvo_y, SEC + 2),
    ] {
        let _ = host.dispatch_pointer_event(pointer(kind, PointerButton::Primary, px, y, t));
    }
    assert_eq!(
        host.store().section_order().first(),
        Some(&segunda),
        "a ordem gravada não pôs a secção arrastada à frente"
    );
    let depois = a_vista(&host.paint::<WidgetGalleryPanel>(&mut st, viewport()));
    let pos = |id: NodeId| depois.iter().position(|(n, _)| *n == id);
    assert!(
        pos(segunda) < pos(primeira),
        "a pintura seguinte não seguiu a ordem gravada ({depois:?})"
    );
}
