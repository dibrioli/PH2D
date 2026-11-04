//! ⭐ **O laboratório em cartões: só o cromo mudou.** Os chips e a caixa viva respondem no cartão, a lista
//! flutua pelas alças de sempre e o estudo continua a medir na largura real.

use ph2d_a11y::NodeId;
use ph2d_editor_core::ids as core_ids;
use ph2d_editor_core::panel::PanelHostInternal;
use ph2d_editor_core::panel::cards::list_menu_of;
use ph2d_editor_core::zones::Rect;
use ph2d_host::{PointerButton, PointerEvent, PointerKind, PointerSource};
use ph2d_panel_widget_lab::state::WidgetLabState;
use ph2d_panel_widget_lab::{WidgetLabPanel, cards, ids};
use ph2d_ui_testkit::MockPanelHost;

const JANELA: Rect = Rect {
    x: 0.0,
    y: 0.0,
    w: 1600.0,
    h: 1800.0,
};

struct Mesa {
    host: MockPanelHost,
    st: WidgetLabState,
    rects: Vec<(NodeId, Rect)>,
}

impl Mesa {
    fn nova() -> Self {
        cards::force_cards_on_this_thread(Some(true));
        let mut host = MockPanelHost::with_panel::<WidgetLabPanel>();
        host.set_panel_visible(<WidgetLabPanel as ph2d_editor_core::panel::Panel>::ID, true);
        let mut mesa = Self {
            host,
            st: WidgetLabState {
                rect: Some(Rect::new(100.0, 20.0, 420.0, 1700.0)),
                ..WidgetLabState::default()
            },
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
                || h.paint::<WidgetLabPanel>(st, JANELA),
            );
        }
    }

    fn rect(&self, id: NodeId) -> Rect {
        self.rects
            .iter()
            .rev()
            .find(|(n, r)| *n == id && r.w > 0.0 && r.h > 0.0)
            .map_or_else(|| panic!("{id:?} não pintado"), |(_, r)| *r)
    }

    fn clica(&mut self, id: NodeId) {
        let r = self.rect(id);
        let (cx, cy) = (r.x + r.w * 0.5, r.y + r.h * 0.5);
        assert_eq!(self.host.hit_at(cx, cy), Some(id), "outro id no centro");
        for e in self.host.click_at(cx, cy) {
            let _ = self
                .host
                .apply_panel_event::<WidgetLabPanel>(&mut self.st, e);
        }
    }

    fn ponteiro(&mut self, kind: PointerKind, x: f32, y: f32) {
        let _ = self.host.dispatch_pointer_event(PointerEvent {
            kind,
            x,
            y,
            button: PointerButton::Primary,
            source: PointerSource::Mouse,
            pressure: 1.0,
            timestamp_ns: 0,
        });
    }
}

/// Cada chip do estudo responde ao clique real, dentro do cartão.
#[test]
fn every_chip_of_the_study_moves_the_study_from_inside_the_card() {
    let mut mesa = Mesa::nova();
    let design = mesa.st.style.design;
    mesa.clica(ids::LAB_VARIANT_NEXT);
    assert_ne!(mesa.st.style.design, design);
    mesa.clica(ids::LAB_VARIANT_PREV);
    assert_eq!(mesa.st.style.design, design);
    let raio = mesa.st.style.radius;
    mesa.clica(ids::LAB_RADIUS_CYCLE);
    assert_ne!(mesa.st.style.radius, raio);
    let dens = mesa.st.style.density;
    mesa.clica(ids::LAB_DENSITY_CYCLE);
    assert_ne!(mesa.st.style.density, dens);
    mesa.assenta();
    let cor = mesa.st.accent;
    mesa.clica(ids::LAB_ACCENT_CYCLE);
    assert_ne!(mesa.st.accent, cor);
    let deco = mesa.st.decorator;
    mesa.clica(ids::LAB_DECORATOR_TOGGLE);
    assert_ne!(mesa.st.decorator, deco);
    let comp = mesa.st.compare;
    mesa.clica(ids::LAB_COMPARE_TOGGLE);
    assert_ne!(mesa.st.compare, comp);
}

/// A caixa viva é um slider a sério, dentro do cartão: o arrasto move o store.
#[test]
fn the_live_box_is_still_a_real_slider() {
    let mut mesa = Mesa::nova();
    let r = mesa.rect(ids::LAB_LIVE_BOX);
    let antes = mesa.host.store().slider(ids::LAB_LIVE_BOX).map(|(_, v)| v);
    let y = r.y + r.h * 0.5;
    mesa.ponteiro(PointerKind::Down, r.x + r.w * 0.2, y);
    mesa.ponteiro(PointerKind::Move, r.x + r.w * 0.8, y);
    mesa.ponteiro(PointerKind::Up, r.x + r.w * 0.8, y);
    assert_ne!(
        mesa.host.store().slider(ids::LAB_LIVE_BOX).map(|(_, v)| v),
        antes
    );
}

/// A lista flutua: a faixa move-a, o «×» fecha e o tema de quem desenha o estudo volta ao fim.
#[test]
fn the_floating_list_moves_closes_and_gives_the_look_back() {
    let antes = ph2d_editor_core::paint::ui_look();
    let mut mesa = Mesa::nova();
    assert_eq!(ph2d_editor_core::paint::ui_look(), antes);
    let faixa = mesa.rect(ids::LAB_DRAG_HANDLE);
    let fecho = mesa.rect(ids::LAB_CLOSE);
    assert!(faixa.x + faixa.w <= fecho.x, "{faixa:?} tapa {fecho:?}");
    let (x, y) = (faixa.x + 20.0, faixa.y + faixa.h * 0.5);
    mesa.ponteiro(PointerKind::Down, x, y);
    mesa.ponteiro(PointerKind::Move, x + 50.0, y + 30.0);
    mesa.ponteiro(PointerKind::Up, x + 50.0, y + 30.0);
    mesa.assenta();
    let nova = mesa.rect(ids::LAB_DRAG_HANDLE);
    assert!(
        (nova.x - faixa.x - 50.0).abs() < 1.0,
        "{nova:?} × {faixa:?}"
    );
    assert!(
        mesa.host
            .store()
            .tooltip_for(list_menu_of(core_ids::LAB_PANEL))
            .is_some()
    );
    let _ = mesa.rect(ids::LAB_RESIZE_HANDLE);
    let _ = mesa.rect(ids::LAB_RESIZE_HANDLE_BL);
    mesa.clica(ids::LAB_CLOSE);
    assert!(
        !mesa
            .host
            .panel_visible(<WidgetLabPanel as ph2d_editor_core::panel::Panel>::ID)
    );
}

/// `PH2D_CARDS=0`: a moldura de antes, sem cartão.
#[test]
fn cards_off_paints_the_old_body() {
    cards::force_cards_on_this_thread(Some(false));
    let mut host = MockPanelHost::with_panel::<WidgetLabPanel>();
    host.set_panel_visible(<WidgetLabPanel as ph2d_editor_core::panel::Panel>::ID, true);
    let mut st = WidgetLabState {
        rect: Some(Rect::new(100.0, 20.0, 420.0, 1700.0)),
        ..WidgetLabState::default()
    };
    let rects = host.paint::<WidgetLabPanel>(&mut st, JANELA);
    assert!(rects.iter().any(|(n, _)| *n == ids::LAB_LIVE_BOX));
    assert!(!rects.iter().any(|(n, _)| *n == ids::LAB_SEC_BENCH));
}
