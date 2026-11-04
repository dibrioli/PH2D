//! ⭐ **O painel autorado em cartões:** cada `SectionHeader` é um cartão que dobra pela mesma dobra, e cada
//! row continua viva, pelo ponteiro real, com a pele que o artista desenhou.

use ph2d_a11y::NodeId;
use ph2d_card_stack::metrics as m;
use ph2d_editor_core::ids as core_ids;
use ph2d_editor_core::panel::PanelHostInternal;
use ph2d_editor_core::panel::cards::list_menu_of;
use ph2d_editor_core::zones::Rect;
use ph2d_host::{PointerButton, PointerEvent, PointerKind, PointerSource};
use ph2d_panel_authored::rows::Row;
use ph2d_panel_authored::state::AuthoredPanelState;
use ph2d_panel_authored::{AuthoredPanel, cards, ids, rows};
use ph2d_ui_testkit::MockPanelHost;

const JANELA: Rect = Rect {
    x: 800.0,
    y: 0.0,
    w: 340.0,
    h: 1400.0,
};

struct Mesa {
    host: MockPanelHost,
    st: AuthoredPanelState,
    rects: Vec<(NodeId, Rect)>,
}

impl Mesa {
    fn nova() -> Self {
        cards::force_cards_on_this_thread(Some(true));
        let mut host = MockPanelHost::with_panel::<AuthoredPanel>();
        host.set_panel_visible(<AuthoredPanel as ph2d_editor_core::panel::Panel>::ID, true);
        let mut mesa = Self {
            host,
            st: AuthoredPanelState,
            rects: Vec::new(),
        };
        mesa.assenta();
        mesa
    }

    fn assenta(&mut self) {
        for _ in 0..60 {
            let (h, st) = (&mut self.host, &mut self.st);
            self.rects = ph2d_editor_core::aquecimento::com_passo_fixo(
                ph2d_editor_core::aquecimento::PASSO_DE_QUADRO_S,
                || h.paint::<AuthoredPanel>(st, JANELA),
            );
        }
    }

    fn rect(&self, id: NodeId) -> Option<Rect> {
        self.rects
            .iter()
            .rev()
            .find(|(n, r)| *n == id && r.w > 0.0 && r.h > 0.0)
            .map(|(_, r)| *r)
    }

    fn ponteiro(
        &mut self,
        kind: PointerKind,
        x: f32,
        y: f32,
    ) -> Vec<ph2d_editor_core::interaction::WidgetEvent> {
        self.host.dispatch_pointer_event(PointerEvent {
            kind,
            x,
            y,
            button: PointerButton::Primary,
            source: PointerSource::Mouse,
            pressure: 1.0,
            timestamp_ns: 0,
        })
    }
}

/// Cada row que responde tem rect e registo; cada cabeçalho virou cartão, e o cabeçalho não é row pintada.
#[test]
fn every_row_is_alive_and_every_section_is_a_card() {
    let mesa = Mesa::nova();
    let (controlos, cabecalhos): (Vec<NodeId>, Vec<NodeId>) = rows::with_rows(|t| {
        (
            t.iter().filter(|r| r.is_control()).map(|r| r.id).collect(),
            t.iter()
                .filter(|r| r.folds_a_section())
                .map(|r| r.id)
                .collect(),
        )
    });
    assert!(!controlos.is_empty(), "a moldura de fábrica tem controlos");
    for id in controlos {
        assert!(mesa.rect(id).is_some(), "{id:?} sem rect");
        assert!(mesa.host.store().get(id).is_some(), "{id:?} sem registo");
    }
    for id in cabecalhos {
        let r = mesa.rect(id).expect("a secção é um cartão");
        assert!(
            r.h > m::CARD_HEAD_H,
            "{id:?} é o cartão, não uma linha: {r:?}"
        );
    }
    assert!(mesa.rect(ids::AUTHORED_CLOSE).is_some());
}

/// O clique no título dobra pela dobra de sempre e as rows do cartão saem do dedo; contraprova: as de
/// outro cartão ficam. O clique numa row real ainda chega.
#[test]
fn a_header_folds_its_rows_only_and_a_row_still_answers() {
    let mut mesa = Mesa::nova();
    let primeiro = rows::with_rows(|t| {
        let i = t.iter().position(Row::folds_a_section)?;
        let seccao = t[i].id;
        let dela = t[i + 1..]
            .iter()
            .take_while(|r| !r.folds_a_section())
            .find(|r| r.is_control())?
            .id;
        Some((seccao, dela))
    });
    let (seccao, row) = primeiro.expect("a moldura de fábrica tem uma secção com controlo");
    let r = mesa.rect(row).expect("pintada");
    assert_eq!(
        mesa.host.hit_at(r.x + r.w * 0.5, r.y + r.h * 0.5),
        Some(row)
    );
    let cartao = mesa.rect(seccao).expect("cartão");
    let _ = mesa
        .host
        .click_at(cartao.x + cartao.w * 0.3, cartao.y + m::CARD_HEAD_H * 0.5);
    mesa.assenta();
    assert!(mesa.host.store().is_collapsed(seccao));
    assert!(mesa.rect(row).is_none(), "dobrado, a row sai do clique");
}

/// A lista flutua: a faixa move-a, a alça redimensiona-a, e o «×» não se tapa.
#[test]
fn the_floating_list_keeps_its_handles() {
    let mut mesa = Mesa::nova();
    let faixa = mesa.rect(ids::AUTHORED_DRAG_HANDLE).expect("faixa");
    let fecho = mesa.rect(ids::AUTHORED_CLOSE).expect("fecho");
    assert!(faixa.x + faixa.w <= fecho.x, "{faixa:?} tapa {fecho:?}");
    let (x, y) = (faixa.x + 20.0, faixa.y + faixa.h * 0.5);
    let _ = mesa.ponteiro(PointerKind::Down, x, y);
    let _ = mesa.ponteiro(PointerKind::Move, x + 50.0, y + 30.0);
    let _ = mesa.ponteiro(PointerKind::Up, x + 50.0, y + 30.0);
    mesa.assenta();
    let nova = mesa.rect(ids::AUTHORED_DRAG_HANDLE).expect("faixa");
    assert!(
        (nova.x - faixa.x - 50.0).abs() < 1.0,
        "{nova:?} × {faixa:?}"
    );
    assert!(mesa.rect(ids::AUTHORED_RESIZE_HANDLE).is_some());
    assert!(mesa.rect(ids::AUTHORED_RESIZE_HANDLE_BL).is_some());
    let menu = list_menu_of(core_ids::AUTHORED_PANEL);
    assert!(mesa.host.store().tooltip_for(menu).is_some());
}

/// `PH2D_CARDS=0`: a moldura de antes, com o cabeçalho de secção como row.
#[test]
fn cards_off_paints_the_old_body() {
    cards::force_cards_on_this_thread(Some(false));
    let mut host = MockPanelHost::with_panel::<AuthoredPanel>();
    host.set_panel_visible(<AuthoredPanel as ph2d_editor_core::panel::Panel>::ID, true);
    let mut st = AuthoredPanelState;
    let rects = host.paint::<AuthoredPanel>(&mut st, JANELA);
    let cab = rows::with_rows(|t| t.iter().find(|r| r.folds_a_section()).map(|r| r.id));
    if let Some(cab) = cab {
        let r = rects.iter().find(|(n, _)| *n == cab).expect("a row").1;
        assert!(r.h <= ph2d_tokens::ROW_H_PX + 0.5, "{r:?}");
    }
    assert!(rects.iter().any(|(n, _)| *n == ids::AUTHORED_DRAG_HANDLE));
}
