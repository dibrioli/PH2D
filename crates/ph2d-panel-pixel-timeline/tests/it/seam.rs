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
    so(
        &a,
        PixelRequest::RangeMove {
            before: 3,
            copy: false,
        },
    );
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

/// **D2 (smoke 31/10): a meio do arrasto o bloco VÊ-SE a mover** — a grelha pinta a ordem prevista
/// (a lei do `move_block`: o vão onde o bloco cai) e nada se pede antes de largar.
#[test]
fn mid_drag_the_grid_paints_the_block_where_it_will_land() {
    use ph2d_editor_core::interaction::{
        GestureMods, GesturePhase, PixelTimelineGesture, PixelTimelineHitKind as H,
    };
    let mut i = info();
    i.range = faixa(K::Frames, vec![], vec![10]);
    let mut p = Palco::novo(i);
    let antes = p.rect(ids::header_id(12)).expect("pintado");
    let (x0, y) = p.meio(ids::RANGE_EDGES[1]);
    let mut passo = |phase, x| {
        p.host
            .store_mut()
            .push_pixel_timeline_gesture(PixelTimelineGesture {
                surface: ph2d_editor_core::ids::PIXEL_TIMELINE_PANEL,
                kind: H::RangeOutline,
                phase,
                x,
                y,
                button: ph2d_host::PointerButton::Primary,
                mods: GestureMods::default(),
            });
        p.rects = p.host.paint::<PixelTimelinePanel>(&mut p.st, VIEWPORT);
        p.host.drained_actions()
    };
    assert!(passo(GesturePhase::Begin, x0).is_empty());
    let a = passo(GesturePhase::Update, antes.x + antes.w);
    assert!(
        !a.iter()
            .any(|e| matches!(e, EditorAction::Pixel(PixelRequest::RangeMove { .. }))),
        "a meio do arrasto nada se move: {a:?}"
    );
    let vao = p.rect(ids::header_id(10)).expect("pintado");
    assert!(
        (vao.x - antes.x).abs() < 0.5,
        "o quadro 10 pinta-se onde vai cair (x {} ≠ {})",
        vao.x,
        antes.x
    );
}

/// Larga a moldura de `range` arrastada pelo canal de gestos de `de` até `a`, com `mods`.
fn larga_moldura(
    range: PixelRange,
    a: impl Fn(&Palco, (f32, f32)) -> (f32, f32),
    mods: ph2d_editor_core::interaction::GestureMods,
) -> Vec<EditorAction> {
    use ph2d_editor_core::interaction::{
        GesturePhase as G, PixelTimelineGesture, PixelTimelineHitKind as H,
    };
    let mut i = info();
    i.range = range;
    let mut p = Palco::novo(i);
    let de = p.meio(ids::RANGE_EDGES[1]);
    let fim = a(&p, de);
    for (phase, (x, y)) in [(G::Begin, de), (G::Update, fim), (G::End, fim)] {
        p.host
            .store_mut()
            .push_pixel_timeline_gesture(PixelTimelineGesture {
                surface: ph2d_editor_core::ids::PIXEL_TIMELINE_PANEL,
                kind: H::RangeOutline,
                phase,
                x,
                y,
                button: ph2d_host::PointerButton::Primary,
                mods,
            });
        p.rects = p.host.paint::<PixelTimelinePanel>(&mut p.st, VIEWPORT);
    }
    p.host.drained_actions()
}

/// **Trial 01/11** (`mover_ctrl`, `mover_cels*`): Ctrl ao largar COPIA o bloco de quadros; a
/// moldura de um intervalo de CELS leva só os cels (colunas e filas andadas, presas à grelha).
#[test]
fn ctrl_at_the_drop_copies_and_a_cel_range_moves_only_its_cels() {
    use ph2d_editor_core::interaction::GestureMods;
    let ctrl = GestureMods {
        cmd: true,
        ..GestureMods::default()
    };
    let col = |p: &Palco| p.rect(ids::header_id(10)).expect("pintado").w;
    let fim_do_12 = |p: &Palco, (_, y): (f32, f32)| {
        let h = p.rect(ids::header_id(12)).expect("pintado");
        (h.x + h.w, y)
    };
    let quadros = faixa(K::Frames, vec![], vec![10]);
    for (mods, copy) in [(GestureMods::default(), false), (ctrl, true)] {
        let a = larga_moldura(quadros.clone(), fim_do_12, mods);
        so(&a, PixelRequest::RangeMove { before: 3, copy });
    }
    let cels = faixa(K::Cels, vec![2], vec![10]);
    let dois = |p: &Palco, (x, y): (f32, f32)| (x + 2.0 * col(p), y);
    for (mods, copy) in [(GestureMods::default(), false), (ctrl, true)] {
        let a = larga_moldura(cels.clone(), dois, mods);
        let pairs = vec![((2, 10), (2, 12))];
        so(
            &a,
            PixelRequest::CelsMove {
                pairs,
                beyond: vec![],
                copy,
            },
        );
    }
    let pitch = ph2d_tokens::row_pitch_px();
    let longe = |p: &Palco, (x, y): (f32, f32)| (x + 9.0 * col(p), y + pitch);
    let a = larga_moldura(cels, longe, GestureMods::default());
    so(
        &a,
        PixelRequest::CelsMove {
            pairs: vec![],
            beyond: vec![((2, 10), (1, 6))],
            copy: false,
        },
    );
}

/// **Trial 01/11** (`round3_gaps.cels_dropped_right_of_last_frame`, `…_below_last_layer`,
/// `…_right_and_below`): um intervalo de cels largado à direita do último quadro pede os destinos
/// além do fim (`beyond`: o `k`-ésimo quadro novo) em vez de se prender; abaixo da última camada a
/// fila prende-se a ela; as duas coisas juntas = à direita na última fila.
#[test]
fn a_cel_range_dropped_past_the_last_frame_asks_for_new_frames_and_the_row_clamps() {
    use ph2d_editor_core::interaction::GestureMods;
    let col = |p: &Palco| p.rect(ids::header_id(10)).expect("pintado").w;
    let pitch = ph2d_tokens::row_pitch_px();
    let cels = faixa(K::Cels, vec![2], vec![11, 12]);
    let direita = |p: &Palco, (x, y): (f32, f32)| (x + 3.0 * col(p), y);
    let a = larga_moldura(cels.clone(), direita, GestureMods::default());
    so(
        &a,
        PixelRequest::CelsMove {
            pairs: vec![],
            beyond: vec![((2, 11), (2, 1)), ((2, 12), (2, 2))],
            copy: false,
        },
    );
    let abaixo = |_: &Palco, (x, y): (f32, f32)| (x, y + 3.0 * pitch);
    let a = larga_moldura(cels.clone(), abaixo, GestureMods::default());
    so(
        &a,
        PixelRequest::CelsMove {
            pairs: vec![((2, 11), (1, 11)), ((2, 12), (1, 12))],
            beyond: vec![],
            copy: false,
        },
    );
    let ambos = |p: &Palco, (x, y): (f32, f32)| (x + 3.0 * col(p), y + 3.0 * pitch);
    let a = larga_moldura(cels, ambos, GestureMods::default());
    so(
        &a,
        PixelRequest::CelsMove {
            pairs: vec![],
            beyond: vec![((2, 11), (1, 1)), ((2, 12), (1, 2))],
            copy: false,
        },
    );
}

/// Foto P6j (cena 16, `copia`): a meio do arrasto com Ctrl as cópias abrem o vão onde caem e a
/// moldura fica SÓ nos originais (as cópias não herdam o id deles).
#[test]
fn mid_ctrl_drag_the_outline_stays_on_the_originals() {
    use ph2d_editor_core::interaction::{
        GestureMods, GesturePhase as G, PixelTimelineGesture, PixelTimelineHitKind as H,
    };
    let mut i = info();
    i.range = faixa(K::Frames, vec![], vec![10]);
    let mut p = Palco::novo(i);
    let antes = p.rect(ids::RANGE_EDGES[1]).expect("pintada");
    let h = p.rect(ids::header_id(12)).expect("pintado");
    let (x0, y) = p.meio(ids::RANGE_EDGES[1]);
    let mods = GestureMods {
        cmd: true,
        ..GestureMods::default()
    };
    for (phase, x) in [(G::Begin, x0), (G::Update, h.x + h.w)] {
        p.host
            .store_mut()
            .push_pixel_timeline_gesture(PixelTimelineGesture {
                surface: ph2d_editor_core::ids::PIXEL_TIMELINE_PANEL,
                kind: H::RangeOutline,
                phase,
                x,
                y,
                button: ph2d_host::PointerButton::Primary,
                mods,
            });
        p.rects = p.host.paint::<PixelTimelinePanel>(&mut p.st, VIEWPORT);
    }
    let depois = p.rect(ids::RANGE_EDGES[1]).expect("pintada");
    assert!(
        (depois.x - antes.x).abs() < 0.5,
        "a moldura não se estica até às cópias (x {} ≠ {})",
        depois.x,
        antes.x
    );
}

/// Sobreviventes da P6j/P6h (A9, B2, A11, B1): o arrasto conta-se ARREDONDADO (colunas, filas e a
/// fronteira), uma fila de grupo recusa o bloco de cels, e o Shift+clique repetido não duplica.
#[test]
fn drags_round_to_the_nearest_cell_a_group_row_refuses_and_shift_never_duplicates() {
    use ph2d_editor_core::interaction::{
        GestureMods, GesturePhase as G, PixelTimelineGesture, PixelTimelineHitKind as H,
    };
    let col = |p: &Palco| p.rect(ids::header_id(10)).expect("pintado").w;
    let pitch = ph2d_tokens::row_pitch_px();
    let cels = faixa(K::Cels, vec![2], vec![10]);
    let quase = |p: &Palco, (x, y): (f32, f32)| (x + 1.6 * col(p), y + 0.6 * pitch);
    let a = larga_moldura(cels, quase, GestureMods::default());
    so(
        &a,
        PixelRequest::CelsMove {
            pairs: vec![((2, 10), (1, 12))],
            beyond: vec![],
            copy: false,
        },
    );
    let quadros = faixa(K::Frames, vec![], vec![10]);
    let antes_do_fim = |p: &Palco, (_, y): (f32, f32)| {
        let h = p.rect(ids::header_id(12)).expect("pintado");
        (h.x + h.w - 0.4 * h.w, y)
    };
    let a = larga_moldura(quadros, antes_do_fim, GestureMods::default());
    so(
        &a,
        PixelRequest::RangeMove {
            before: 3,
            copy: false,
        },
    );

    let mut i = info();
    i.layers.insert(
        0,
        PixelLayerRow {
            group: true,
            ..camada(3, "Group")
        },
    );
    i.range = faixa(K::Cels, vec![2], vec![10]);
    let mut p = Palco::novo(i);
    let de = p.meio(ids::RANGE_EDGES[1]);
    for (phase, (x, y)) in [(G::Begin, de), (G::End, (de.0, de.1 - pitch))] {
        p.host
            .store_mut()
            .push_pixel_timeline_gesture(PixelTimelineGesture {
                surface: ph2d_editor_core::ids::PIXEL_TIMELINE_PANEL,
                kind: H::RangeOutline,
                phase,
                x,
                y,
                button: ph2d_host::PointerButton::Primary,
                mods: GestureMods::default(),
            });
        p.rects = p.host.paint::<PixelTimelinePanel>(&mut p.st, VIEWPORT);
    }
    let a = p.host.drained_actions();
    assert!(
        !a.iter()
            .any(|e| matches!(e, EditorAction::Pixel(PixelRequest::CelsMove { .. }))),
        "para uma fila de grupo nada se pede: {a:?}"
    );

    let mut i = info();
    i.range = faixa(K::Frames, vec![], vec![10, 12]);
    let mut p = Palco::novo(i);
    let (x, y) = p.meio(ids::header_id(12));
    let shift = GestureMods {
        shift: true,
        ..GestureMods::default()
    };
    p.host
        .store_mut()
        .push_pixel_timeline_gesture(PixelTimelineGesture {
            surface: ph2d_editor_core::ids::PIXEL_TIMELINE_PANEL,
            kind: H::Header { frame: 12 },
            phase: G::Begin,
            x,
            y,
            button: ph2d_host::PointerButton::Primary,
            mods: shift,
        });
    p.rects = p.host.paint::<PixelTimelinePanel>(&mut p.st, VIEWPORT);
    let r = ultimo(&p.host.drained_actions()).expect("um intervalo");
    assert_eq!(r.frames, [10, 12], "o Shift+clique repetido não duplica");
}

/// *Playback*: cada interruptor carregado no meio pede as opções com ELE trocado (as outras como o
/// instantâneo as tem), e a velocidade escolhida no menu também.
#[test]
fn the_playback_toggles_and_the_speed_menu_ask_for_the_whole_options() {
    use ph2d_editor_core::pixel_panel::{PixelPlayback, PixelSpeed};
    let base = PixelPlayback::default();
    let casos = [
        (ids::PLAY_ONCE, PixelPlayback { once: true, ..base }),
        (ids::PLAY_ALL, PixelPlayback { all: true, ..base }),
        (
            ids::PLAY_SUBTAGS,
            PixelPlayback {
                subtags: false,
                ..base
            },
        ),
        (
            ids::PLAY_REWIND,
            PixelPlayback {
                rewind: true,
                ..base
            },
        ),
    ];
    for (id, quer) in casos {
        so(
            &Palco::novo(info()).carrega(id),
            PixelRequest::Playback(quer),
        );
    }
    let mut p = Palco::novo(info());
    let r = p.rect(ids::PLAY_SPEED).expect("o chip da velocidade");
    for e in p.host.click_at(r.x + r.w * 0.5, r.y + r.h * 0.5) {
        let _ = p.host.apply_panel_event::<PixelTimelinePanel>(&mut p.st, e);
    }
    p.rects = p.host.paint::<PixelTimelinePanel>(&mut p.st, VIEWPORT);
    assert!(
        p.rect(ids::speed_option_id(4)).is_some(),
        "o clique no chip abre o menu"
    );
    so(
        &p.carrega(ids::speed_option_id(4)),
        PixelRequest::Playback(PixelPlayback {
            speed: PixelSpeed::Two,
            ..base
        }),
    );
}
