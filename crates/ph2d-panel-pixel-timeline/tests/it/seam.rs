//! A costura da linha do tempo: cada controlo carregado no MEIO, pelo rato de verdade, chega ao
//! barramento como exactamente o pedido dele.

use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::interaction::WidgetEvent;
use ph2d_editor_core::pixel_layer_row::PixelLayerWidget;
use ph2d_editor_core::pixel_panel::{
    PixelCel, PixelDirection, PixelFrameAction, PixelFrameCol, PixelLayerRow, PixelPanelInfo,
    PixelRequest, PixelTagInfo,
};
use ph2d_editor_core::zones::Rect;
use ph2d_panel_pixel_timeline::{
    PixelTimelinePanel, PixelTimelinePanelState, ids, set_current_pixel,
};
use ph2d_ui_testkit::MockPanelHost;

const VIEWPORT: Rect = Rect {
    x: 0.0,
    y: 0.0,
    w: 1920.0,
    h: 1080.0,
};

fn camada(id: u32, name: &str) -> PixelLayerRow {
    PixelLayerRow {
        id,
        name: name.to_owned(),
        depth: 0,
        group: false,
        collapsed: false,
        visible: true,
        locked: false,
        reference: false,
        background: false,
        opacity: 255,
        blend: Default::default(),
    }
}

fn cel(layer: u32, frame: u32, owner: u32) -> PixelCel {
    PixelCel {
        layer,
        frame,
        owner,
        opacity: 255,
        z: 0,
        empty: false,
    }
}

/// Duas camadas × três frames (ids 10, 11, 12 — ≠ das posições), os cels 10 e 11 da «Ink» ligados
/// (o mesmo dono) e uma tag sobre as posições 0..=1.
fn info() -> PixelPanelInfo {
    PixelPanelInfo {
        editing: true,
        layers: vec![camada(2, "Ink"), camada(1, "Background")],
        active_layer: Some(2),
        frames: vec![
            PixelFrameCol {
                id: 10,
                duration_ms: 100,
            },
            PixelFrameCol {
                id: 11,
                duration_ms: 250,
            },
            PixelFrameCol {
                id: 12,
                duration_ms: 100,
            },
        ],
        active_frame: Some(10),
        cels: vec![
            cel(2, 10, 10),
            cel(2, 11, 10),
            cel(1, 10, 10),
            cel(1, 12, 12),
        ],
        tags: vec![PixelTagInfo {
            name: "walk".to_owned(),
            from: 0,
            to: 1,
            direction: PixelDirection::PingPong,
            repeat: 2,
            color: [200, 60, 60, 255],
        }],
        ..Default::default()
    }
}

struct Palco {
    host: MockPanelHost,
    st: PixelTimelinePanelState,
    rects: Vec<(ph2d_a11y::NodeId, Rect)>,
}

impl Palco {
    fn novo(i: PixelPanelInfo) -> Self {
        let mut p = Self {
            host: MockPanelHost::with_panel::<PixelTimelinePanel>(),
            st: PixelTimelinePanelState::default(),
            rects: Vec::new(),
        };
        set_current_pixel(i);
        p.rects = p.host.paint::<PixelTimelinePanel>(&mut p.st, VIEWPORT);
        p
    }

    fn rect(&self, id: ph2d_a11y::NodeId) -> Option<Rect> {
        self.rects.iter().find(|(n, _)| *n == id).map(|(_, r)| *r)
    }

    fn carrega(&mut self, id: ph2d_a11y::NodeId) -> Vec<EditorAction> {
        let r = self
            .rect(id)
            .unwrap_or_else(|| panic!("o painel nunca pintou {id:?}"));
        let ev = self.host.click_at(r.x + r.w * 0.5, r.y + r.h * 0.5);
        assert!(
            ev.iter()
                .any(|e| matches!(e, WidgetEvent::Click(c) if *c == id)),
            "carregar no meio de {id:?} deu {ev:?}: MORTO sob o dedo"
        );
        for e in ev {
            let _ = self
                .host
                .apply_panel_event::<PixelTimelinePanel>(&mut self.st, e);
        }
        self.host.drained_actions()
    }
}

fn so(a: &[EditorAction], r: PixelRequest) {
    assert_eq!(a, [EditorAction::Pixel(r)]);
}

/// O clique numa célula escolhe a camada E o frame pelos ids estáveis (não pelas posições); o
/// clique num cabeçalho escolhe o frame.
#[test]
fn a_celula_e_o_cabecalho_chegam_ao_barramento_pelos_ids_do_frame() {
    let mut p = Palco::novo(info());
    so(
        &p.carrega(ids::cell_id(1, 12)),
        PixelRequest::CelSelect {
            layer: 1,
            frame: 12,
        },
    );
    so(
        &p.carrega(ids::header_id(11)),
        PixelRequest::FrameSelect(11),
    );
    // A tag leva ao primeiro frame dela.
    so(&p.carrega(ids::tag_id(0)), PixelRequest::FrameSelect(10));
}

/// A barra e a coluna das camadas: cada botão o seu pedido.
#[test]
fn a_barra_e_a_coluna_das_camadas_chegam_ao_barramento() {
    for (a, id, ..) in ids::TRANSPORT {
        so(
            &Palco::novo(info()).carrega(id),
            PixelRequest::FrameAction(a),
        );
    }
    let acoes = ids::ACTIONS.iter().map(|(a, id, ..)| (*a, *id));
    for (a, id) in ids::TOGGLES.iter().map(|(a, id, _)| (*a, *id)).chain(acoes) {
        so(
            &Palco::novo(info()).carrega(id),
            PixelRequest::FrameAction(a),
        );
    }
    let mut p = Palco::novo(info());
    so(
        &p.carrega(ids::layer_widget_id(1, PixelLayerWidget::Eye)),
        PixelRequest::LayerVisible(1),
    );
    so(
        &p.carrega(ids::layer_widget_id(1, PixelLayerWidget::Row)),
        PixelRequest::LayerSelect(1),
    );
    // A tag sob o frame activo mostra o apagar, e ele apaga ESSA tag.
    so(&p.carrega(ids::TAG_DELETE), PixelRequest::TagDelete(0));
    assert_eq!(PixelFrameAction::Play, ids::TRANSPORT[2].0);
}

/// As filas da grelha alinham com as linhas das camadas, e cada cel pinta-se na coluna do frame.
#[test]
fn a_grelha_alinha_com_as_camadas_e_com_os_cabecalhos() {
    let p = Palco::novo(info());
    for (layer, frame) in [(2, 10), (2, 12), (1, 11)] {
        let c = p.rect(ids::cell_id(layer, frame)).expect("célula pintada");
        let l = p
            .rect(ids::layer_widget_id(layer, PixelLayerWidget::Row))
            .expect("linha da camada pintada");
        let h = p.rect(ids::header_id(frame)).expect("cabeçalho pintado");
        assert!(
            (c.y - l.y).abs() < 0.5,
            "a fila do cel {c:?} ≠ a da camada {l:?}"
        );
        assert!(
            (c.x - h.x).abs() < 0.5 && (c.w - h.w).abs() < 0.5,
            "{c:?} ≠ coluna {h:?}"
        );
    }
}

/// Sem sprite em edição, nada pede nada.
#[test]
fn sem_sprite_em_edicao_nao_ha_pedido() {
    let mut i = info();
    i.editing = false;
    let p = Palco::novo(i);
    assert!(p.rect(ids::cell_id(2, 10)).is_none());
    assert!(p.rect(ids::TRANSPORT[0].1).is_none());
}

/// Camadas a mais do que a banda ROLAM com a roda (nunca recortadas em silêncio): depois da roda a
/// última camada fica à vista e a primeira subiu.
#[test]
fn as_filas_a_mais_rolam_com_a_roda() {
    let mut i = info();
    i.layers = (1..=12).rev().map(|id| camada(id, "Layer")).collect();
    let mut p = Palco::novo(i);
    let antes = p
        .rect(ids::layer_widget_id(12, PixelLayerWidget::Row))
        .expect("a 1.ª camada");
    assert!(
        p.rect(ids::layer_widget_id(1, PixelLayerWidget::Row))
            .is_none(),
        "a banda de fábrica não cabe doze camadas — o teste não mede a rolagem"
    );
    let _ = p
        .host
        .wheel_at(antes.x + antes.w * 0.5, antes.y + antes.h * 0.5, -2000.0);
    p.rects = p.host.paint::<PixelTimelinePanel>(&mut p.st, VIEWPORT);
    let depois = p.rect(ids::layer_widget_id(1, PixelLayerWidget::Row));
    assert!(
        depois.is_some(),
        "a última camada nunca chegou à vista depois da roda"
    );
}
