//! A costura da linha do tempo: cada controlo carregado no MEIO, pelo rato de verdade, chega ao
//! barramento como exactamente o pedido dele.

use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::interaction::WidgetEvent;
use ph2d_editor_core::panel::PanelHostInternal;
use ph2d_editor_core::pixel_layer_row::PixelLayerWidget;
use ph2d_editor_core::pixel_panel::{
    PixelCel, PixelDirection, PixelFrameAction, PixelFrameCol, PixelLayerRow, PixelPanelInfo,
    PixelRange, PixelRangeKind as K, PixelRequest, PixelTagInfo,
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
        continuous: false,
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

impl Palco {
    /// Um clique numa superfície da grelha (cabeçalho, cel): os gestos drenam-se no quadro seguinte.
    fn toca(&mut self, id: ph2d_a11y::NodeId) -> Vec<EditorAction> {
        let r = self
            .rect(id)
            .unwrap_or_else(|| panic!("o painel nunca pintou {id:?}"));
        let ev = self.host.click_at(r.x + r.w * 0.5, r.y + r.h * 0.5);
        assert!(ev.is_empty(), "a superfície captura o clique: {ev:?}");
        self.rects = self
            .host
            .paint::<PixelTimelinePanel>(&mut self.st, VIEWPORT);
        self.host.drained_actions()
    }

    /// Arrasta do meio de `id` até `(x, y)`.
    fn arrasta(&mut self, id: ph2d_a11y::NodeId, (x, y): (f32, f32)) -> Vec<EditorAction> {
        let r = self
            .rect(id)
            .unwrap_or_else(|| panic!("o painel nunca pintou {id:?}"));
        let _ = self.host.drag_at(r.x + r.w * 0.5, r.y + r.h * 0.5, x, y);
        self.rects = self
            .host
            .paint::<PixelTimelinePanel>(&mut self.st, VIEWPORT);
        self.host.drained_actions()
    }

    fn meio(&self, id: ph2d_a11y::NodeId) -> (f32, f32) {
        let r = self.rect(id).expect("pintado");
        (r.x + r.w * 0.5, r.y + r.h * 0.5)
    }
}

fn faixa(kind: K, layers: Vec<u32>, frames: Vec<u32>) -> PixelRange {
    PixelRange {
        kind,
        layers,
        frames,
    }
}

/// O último intervalo empurrado (um arrasto empurra um por passo).
fn ultimo(a: &[EditorAction]) -> Option<PixelRange> {
    a.iter().rev().find_map(|e| match e {
        EditorAction::Pixel(PixelRequest::Range(r)) => Some(r.clone()),
        _ => None,
    })
}

fn so(a: &[EditorAction], r: PixelRequest) {
    assert_eq!(a, [EditorAction::Pixel(r)]);
}

/// O clique numa célula escolhe a camada E o frame pelos ids estáveis (não pelas posições); o
/// clique num cabeçalho escolhe o frame.
#[test]
fn a_celula_e_o_cabecalho_chegam_ao_barramento_pelos_ids_do_frame() {
    let mut p = Palco::novo(info());
    let pede = |r| EditorAction::Pixel(r);
    assert_eq!(
        p.toca(ids::cell_id(1, 12)),
        [
            pede(PixelRequest::CelSelect {
                layer: 1,
                frame: 12,
            }),
            pede(PixelRequest::Range(faixa(K::Cels, vec![1], vec![12]))),
        ]
    );
    assert_eq!(
        p.toca(ids::header_id(11)),
        [
            pede(PixelRequest::FrameSelect(11)),
            pede(PixelRequest::Range(faixa(K::Frames, vec![], vec![11]))),
        ]
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
    assert_eq!(
        p.carrega(ids::layer_widget_id(1, PixelLayerWidget::Row)),
        [
            EditorAction::Pixel(PixelRequest::LayerSelect(1)),
            EditorAction::Pixel(PixelRequest::Range(faixa(K::Layers, vec![1], vec![]))),
        ],
        "o nome escolhe a fila inteira"
    );
    so(
        &p.carrega(ids::layer_widget_id(1, PixelLayerWidget::Continuous)),
        PixelRequest::LayerContinuous(1),
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

/// Shift (ou Ctrl) + clique ACRESCENTA o quadro solto, sem preencher o vão (lei do Trial).
#[test]
fn shift_click_adds_one_frame_and_never_fills_the_gap() {
    let mut i = info();
    i.range = faixa(K::Frames, vec![], vec![10]);
    let mut p = Palco::novo(i);
    p.host.store_mut().set_shift_held(true);
    let a = p.toca(ids::header_id(12));
    assert_eq!(ultimo(&a), Some(faixa(K::Frames, vec![], vec![10, 12])));
}

/// Arrastar pelos cabeçalhos escolhe o intervalo contíguo; pelos cels, o rectângulo.
#[test]
fn dragging_headers_picks_a_span_and_dragging_cels_a_rectangle() {
    let mut p = Palco::novo(info());
    let fim = p.meio(ids::header_id(12));
    let a = p.arrasta(ids::header_id(10), fim);
    assert_eq!(ultimo(&a), Some(faixa(K::Frames, vec![], vec![10, 11, 12])));
    let fim = p.meio(ids::cell_id(1, 11));
    let a = p.arrasta(ids::cell_id(2, 10), fim);
    assert_eq!(ultimo(&a), Some(faixa(K::Cels, vec![2, 1], vec![10, 11])));
}

/// A ponta direita da tag larga-se no quadro debaixo do cursor.
#[test]
fn dragging_the_right_end_of_a_tag_drops_it_on_the_frame_under_the_cursor() {
    let mut p = Palco::novo(info());
    let (x, _) = p.meio(ids::header_id(12));
    let (_, y) = p.meio(ids::tag_id(0));
    let a = p.arrasta(ids::tag_edge_id(0, true), (x, y));
    so(
        &a,
        PixelRequest::TagRange {
            index: 0,
            from: 0,
            to: 2,
        },
    );
}

/// A moldura do intervalo escolhido arrasta-se e larga o bloco na fronteira mais próxima.
#[test]
fn dragging_the_range_outline_drops_the_block_at_the_nearest_boundary() {
    let mut i = info();
    i.range = faixa(K::Frames, vec![], vec![10]);
    let mut p = Palco::novo(i);
    let h = p.rect(ids::header_id(12)).expect("pintado");
    let (_, y) = p.meio(ids::RANGE_EDGES[1]);
    let a = p.arrasta(ids::RANGE_EDGES[1], (h.x + h.w, y));
    so(&a, PixelRequest::RangeMove { before: 3 });
}

/// *Constant Frame Rate* abre: o campo da duração passa a valer para todos (o 1.º quadro), e as
/// pontas da tag editam-se em base 1.
#[test]
fn constant_frame_rate_turns_the_duration_into_all_frames_and_tag_ends_are_base_one() {
    let mut p = Palco::novo(info());
    let _ = p.carrega(ids::CONSTANT_RATE);
    p.rects = p.host.paint::<PixelTimelinePanel>(&mut p.st, VIEWPORT);
    p.host
        .store_mut()
        .set_number_value(ids::FRAME_DURATION, 70.0);
    let _ = p.host.apply_panel_event::<PixelTimelinePanel>(
        &mut p.st,
        WidgetEvent::ValueChanged(ids::FRAME_DURATION),
    );
    so(
        &p.host.drained_actions(),
        PixelRequest::ConstantFrameRate(70),
    );
    p.host.store_mut().set_number_value(ids::TAG_TO, 3.0);
    let _ = p
        .host
        .apply_panel_event::<PixelTimelinePanel>(&mut p.st, WidgetEvent::ValueChanged(ids::TAG_TO));
    so(
        &p.host.drained_actions(),
        PixelRequest::TagRange {
            index: 0,
            from: 0,
            to: 2,
        },
    );
}
