//! A costura da Preview: o `×` fecha, o botão do título pede tocar/parar, arrastar o título move a
//! janela e a roda sobre a imagem muda o zoom (o editor não a vê) — tudo pelo rato de verdade.

use std::sync::Arc;

use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::interaction::WidgetEvent;
use ph2d_editor_core::panel::{Panel, PanelHostInternal};
use ph2d_editor_core::pixel_panel::{PixelFrameAction, PixelRequest};
use ph2d_editor_core::zones::Rect;
use ph2d_panel_pixel_preview::{
    PixelPreviewPanel, PixelPreviewPanelState, PreviewFrame, ids, publish,
};
use ph2d_ui_testkit::MockPanelHost;

const VIEWPORT: Rect = Rect {
    x: 0.0,
    y: 0.0,
    w: 1920.0,
    h: 1080.0,
};

struct Palco {
    host: MockPanelHost,
    st: PixelPreviewPanelState,
    rects: Vec<(ph2d_a11y::NodeId, Rect)>,
}

impl Palco {
    fn novo() -> Self {
        publish(Some(PreviewFrame {
            rgba: Arc::new(vec![255; 16 * 8 * 4]),
            w: 16,
            h: 8,
            seq: 1,
            frame: 7,
            playing: false,
        }));
        let mut p = Self {
            host: MockPanelHost::with_panel::<PixelPreviewPanel>(),
            st: PixelPreviewPanelState::default(),
            rects: Vec::new(),
        };
        p.pinta();
        p
    }

    fn pinta(&mut self) {
        self.rects = self.host.paint::<PixelPreviewPanel>(&mut self.st, VIEWPORT);
    }

    fn meio(&self, id: ph2d_a11y::NodeId) -> (f32, f32) {
        let r = self
            .rects
            .iter()
            .find(|(n, _)| *n == id)
            .map(|(_, r)| *r)
            .unwrap_or_else(|| panic!("o painel nunca pintou {id:?}"));
        (r.x + r.w * 0.5, r.y + r.h * 0.5)
    }

    fn janela(&self) -> Rect {
        self.host
            .store()
            .panel_rect(PixelPreviewPanel::NODE_ID)
            .expect("a janela publica o rect")
    }

    fn carrega(&mut self, id: ph2d_a11y::NodeId) -> Vec<EditorAction> {
        let (x, y) = self.meio(id);
        let ev = self.host.click_at(x, y);
        assert!(
            ev.iter()
                .any(|e| matches!(e, WidgetEvent::Click(c) if *c == id)),
            "carregar no meio de {id:?} deu {ev:?}: MORTO sob o dedo"
        );
        for e in ev {
            let _ = self
                .host
                .apply_panel_event::<PixelPreviewPanel>(&mut self.st, e);
        }
        self.host.drained_actions()
    }
}

#[test]
fn o_x_fecha_a_preview() {
    let mut p = Palco::novo();
    let _ = p.carrega(ids::CLOSE);
    assert!(!p.host.panel_visible(PixelPreviewPanel::ID), "o × fecha-a");
}

#[test]
fn o_botao_do_titulo_pede_tocar_a_preview() {
    let mut p = Palco::novo();
    assert_eq!(
        p.carrega(ids::PLAY),
        [EditorAction::Pixel(PixelRequest::FrameAction(
            PixelFrameAction::PlayPreview
        ))]
    );
}

#[test]
fn arrastar_o_titulo_move_a_janela() {
    let mut p = Palco::novo();
    let antes = p.janela();
    let (x, y) = p.meio(ids::DRAG);
    let _ = p.host.drag_at(x, y, x - 200.0, y + 100.0);
    p.pinta();
    let depois = p.janela();
    assert!(
        (depois.x - (antes.x - 200.0)).abs() < 0.5 && (depois.y - (antes.y + 100.0)).abs() < 0.5,
        "arrastar o título por (-200, +100) levou a janela de {antes:?} a {depois:?}"
    );
    assert_eq!(
        (depois.w, depois.h),
        (antes.w, antes.h),
        "o arrasto não muda o tamanho"
    );
}

#[test]
fn a_roda_sobre_a_imagem_muda_o_zoom_da_preview() {
    let mut p = Palco::novo();
    let caber = p.st.escala();
    assert!(caber > 0.0, "pintou a imagem");
    let r = p.janela();
    let (x, y) = (r.x + r.w * 0.5, r.y + r.h * 0.6);
    let ev = p.host.wheel_at(x, y, 16.0);
    assert!(ev.is_empty(), "a roda é da Preview: {ev:?}");
    p.pinta();
    let mais = p.st.escala();
    assert!(mais > caber, "a roda para cima aproxima: {caber} → {mais}");
    let _ = p.host.wheel_at(x, y, -16.0);
    p.pinta();
    let _ = p.host.wheel_at(x, y, -16.0);
    p.pinta();
    assert!(
        p.st.escala() < mais,
        "a roda para baixo afasta: {mais} → {}",
        p.st.escala()
    );
}
