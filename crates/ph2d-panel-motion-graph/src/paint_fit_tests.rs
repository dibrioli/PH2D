//! ⛔⛔ **«O GRAFO ABRE COM A PARTE DE CIMA ESCONDIDA»** — report do Enio, 2026-10-03. O
//! enquadramento automático, pela costura real (`MockPanelHost::paint_with_layout`): o que ele
//! promete é que o que se DESENHA cabe no painel.
//!
//! Medido no app (sonda `[SONDA-FIT]`, cena `=128` a 1930×1040): o quadro 0 enquadra contra um
//! painel de `1918` px de largura e o quadro 1 já o tem a `1310` (os painéis laterais chegaram) —
//! os cartões ficam `304` px à direita. E com o zoom no piso (`0,35`) os cartões desenham-se como
//! PÍLULAS, mas o enquadramento media-os ABERTOS: a caixa centrada era a dos cartões abertos, e as
//! pílulas ficavam no topo dela, cortadas.

use super::*;
use crate::geom::{View, card_rect};
use crate::snapshot::{
    CardParam, GraphNodeView, GraphViewSnapshot, NodeViewKind, set_current_motion_graph,
};
use crate::state::MotionGraphPanelState;
use ph2d_editor_core::screens::layout::{CenterSplit, HeroLayout};
use ph2d_node_registry::{NodeSilhouette, NodeUiCategory, ParamUiHint, ParamWidget};

/// A janela da foto do dono.
const JANELA: Rect = Rect {
    x: 0.0,
    y: 0.0,
    w: 1930.0, // LITERAL-PX-OK: a janela fotografada
    h: 1040.0, // LITERAL-PX-OK: a janela fotografada
};

/// O layout da janela, com o painel do grafo no retângulo MEDIDO no app (quadro 1 em diante, a
/// timeline aberta por baixo): `1310 × 205,5` px.
fn layout() -> HeroLayout {
    let mut l = HeroLayout::for_viewport_split(
        JANELA,
        false,
        ph2d_editor_core::screens::layout::rail_w(),
        CenterSplit::Horizontal {
            t: CenterSplit::T_DEFAULT,
        },
    );
    l.motion_graph = Rect {
        x: 312.0,  // LITERAL-PX-OK: medido no app
        y: 566.5,  // LITERAL-PX-OK: medido no app
        w: 1310.0, // LITERAL-PX-OK: medido no app
        h: 205.5,  // LITERAL-PX-OK: medido no app
    };
    l
}

fn no(id: u32, nome: &str, x: f32, y: f32, params: usize) -> GraphNodeView {
    let hint = ParamUiHint {
        param: "strength",
        label: "Strength",
        min: 0.0,
        max: 1.0,
        step: 0.1,
        widget: ParamWidget::Slider,
    };
    GraphNodeView {
        primary_input: 0,
        id,
        kind: NodeViewKind::Node,
        display_name: nome.into(),
        category: NodeUiCategory::Utility,
        silhouette: NodeSilhouette::Rect,
        x,
        y,
        inputs: Vec::new(),
        outputs: Vec::new(),
        readout: None,
        count: None,
        hot: false,
        is_sink: false,
        preview: None,
        bypassed: false,
        inert: false,
        thumbnail: None,
        params: vec![CardParam::from_hint(hint, 0.5); params],
        sections: Vec::new(),
    }
}

fn grafo(nos: Vec<GraphNodeView>) -> GraphViewSnapshot {
    GraphViewSnapshot {
        level: None,
        breadcrumb: Vec::new(),
        nodes: nos,
        edges: Vec::new(),
        backdrops: Vec::new(),
        probe: None,
        now: 0.0,
    }
}

/// O desenho da `=127`: a fila das forças (cartões altos, oito params) a `y = −220`, o Number a
/// `−400`, a cadeia das estrelas a `0` e a forma a `220`.
fn a_galaxia() -> GraphViewSnapshot {
    let mut nos = vec![no(1, "Number", 160.0, -400.0, 3)];
    for (i, nome) in ["Falloff", "Vortex", "Attractor", "Curl Noise"]
        .iter()
        .enumerate()
    {
        nos.push(no(10 + i as u32, nome, i as f32 * 160.0, -220.0, 8));
    }
    for (i, nome) in ["Grid", "Integrate", "Duplicator", "Output"]
        .iter()
        .enumerate()
    {
        nos.push(no(20 + i as u32, nome, i as f32 * 220.0, 0.0, 2));
    }
    nos.push(no(30, "Shape", 0.0, 220.0, 6));
    grafo(nos)
}

/// Pinta o painel com `layout` e devolve o retângulo de cada cartão TAL COMO DESENHADO.
fn pinta(
    host: &mut ph2d_ui_testkit::MockPanelHost,
    state: &mut MotionGraphPanelState,
    layout: HeroLayout,
    snap: &GraphViewSnapshot,
) -> Vec<Rect> {
    let _ = host.paint_with_layout::<crate::MotionGraphPanel>(state, layout, JANELA);
    let view = View::new(layout.motion_graph, state.view);
    snap.nodes.iter().map(|n| card_rect(n, &view)).collect()
}

fn dentro(r: Rect, painel: Rect) -> bool {
    const FOLGA: f32 = 0.5; // LITERAL-PX-OK: meio pixel de arredondamento
    r.x >= painel.x - FOLGA
        && r.y >= painel.y - FOLGA
        && r.x + r.w <= painel.x + painel.w + FOLGA
        && r.y + r.h <= painel.y + painel.h + FOLGA
}

/// O desenho da `=128` (a fila das forças por baixo da cadeia): cabe ao piso, em pílulas.
fn a_galaxia_baixa() -> GraphViewSnapshot {
    let mut nos = vec![no(1, "Number", -220.0, 160.0, 3)];
    for (i, nome) in ["Falloff", "Vortex", "Attractor", "Curl Noise"]
        .iter()
        .enumerate()
    {
        nos.push(no(10 + i as u32, nome, i as f32 * 220.0, 160.0, 8));
    }
    for (i, nome) in ["Grid", "Integrate", "Duplicator", "Output"]
        .iter()
        .enumerate()
    {
        nos.push(no(20 + i as u32, nome, i as f32 * 220.0, 0.0, 2));
    }
    nos.push(no(30, "Shape", 440.0, 60.0, 6));
    grafo(nos)
}

/// O enquadramento automático do 1.º quadro, pela costura.
fn abre(snap: &GraphViewSnapshot) -> (MotionGraphPanelState, Vec<Rect>) {
    set_current_motion_graph(Some(snap.clone()));
    let mut host = ph2d_ui_testkit::MockPanelHost::with_panel::<crate::MotionGraphPanel>();
    let mut state = MotionGraphPanelState::default();
    let rects = pinta(&mut host, &mut state, layout(), snap);
    set_current_motion_graph(None);
    (state, rects)
}

fn fora(snap: &GraphViewSnapshot, rects: &[Rect], painel: Rect) -> Vec<String> {
    snap.nodes
        .iter()
        .zip(rects)
        .filter(|(_, r)| !dentro(**r, painel))
        .map(|(n, r)| format!("{} em {r:?}", n.display_name))
        .collect()
}

/// ⛔⛔ **O QUE SE DESENHA CABE NO PAINEL** — a galáxia da `=128`, em que o zoom cai no regime das
/// pílulas. CONTROLO: medida com cartões ABERTOS ela não cabe ao piso (era isso que o
/// enquadramento centrava), e os cartões saem de facto como pílulas.
#[test]
fn every_card_the_auto_fit_draws_lies_inside_the_panel() {
    let snap = a_galaxia_baixa();
    let painel = layout().motion_graph;
    let aberta = 160.0 + crate::geom::card_h(&snap.nodes[2]);
    assert!(
        aberta * ZOOM_FIT_MIN > painel.h - 2.0 * FIT_PAD,
        "a fixtura tem de NÃO caber aberta no painel de {} px",
        painel.h
    );
    let (state, rects) = abre(&snap);
    assert!(
        state.view.zoom < crate::geom::ZOOM_DA_CAPSULA,
        "a este zoom os cartões são pílulas ({})",
        state.view.zoom
    );
    let f = fora(&snap, &rects, painel);
    assert!(
        f.is_empty(),
        "painel {painel:?} — fora dele:\n  {}",
        f.join("\n  ")
    );
    // E o que cabe fica ao CENTRO — medido nas pílulas que se desenham, e não nos cartões abertos.
    let (cima, baixo) = rects.iter().fold((f32::MAX, f32::MIN), |(a, b), r| {
        (a.min(r.y), b.max(r.y + r.h))
    });
    let desvio = (cima + baixo) * 0.5 - (painel.y + painel.h * 0.5);
    assert!(
        desvio.abs() < 1.0,
        "as pílulas estão {desvio} px fora do centro vertical"
    );
}

/// ⛔⛔ **O QUE NÃO CABE PERDE O FUNDO, NUNCA O TOPO** — o desenho da `=127` num painel de `205` px
/// não cabe nem em pílulas ao piso da leitura. A fila de CIMA (o Number) tem de estar inteira;
/// CONTROLO: há de facto alguma coisa fora (senão o caso não é o de não caber).
#[test]
fn a_graph_taller_than_the_panel_keeps_its_top_row_in_view() {
    let snap = a_galaxia();
    let painel = layout().motion_graph;
    let (_, rects) = abre(&snap);
    let f = fora(&snap, &rects, painel);
    assert!(!f.is_empty(), "a fixtura tem de NÃO caber");
    assert!(
        dentro(rects[0], painel),
        "o Number (a fila de cima) está em {:?}, fora de {painel:?}",
        rects[0]
    );
    assert!(
        f.iter().all(|l| l.starts_with("Shape")),
        "só a fila de BAIXO pode ficar de fora: {f:?}"
    );
}

/// ⭐ **O CONTROLO do regime aberto:** um grafo que cabe com os cartões abertos continua a abrir
/// com eles abertos (o texto dos params legível) e dentro do painel.
#[test]
fn a_small_graph_still_opens_with_full_cards_inside_the_panel() {
    let snap = grafo(vec![
        no(1, "Grid", 0.0, 0.0, 0),
        no(2, "Output", 220.0, 0.0, 0),
    ]);
    set_current_motion_graph(Some(snap.clone()));
    let mut host = ph2d_ui_testkit::MockPanelHost::with_panel::<crate::MotionGraphPanel>();
    let mut state = MotionGraphPanelState::default();
    let painel = layout().motion_graph;
    let rects = pinta(&mut host, &mut state, layout(), &snap);
    set_current_motion_graph(None);
    assert!(
        state.view.zoom >= crate::geom::ZOOM_DA_CAPSULA,
        "cartões abertos ({})",
        state.view.zoom
    );
    assert!(
        rects.iter().all(|r| dentro(*r, painel)),
        "{rects:?} em {painel:?}"
    );
}

/// ⛔⛔ **O PAINEL QUE MUDA DE TAMANHO ANTES DE O ARTISTA TOCAR NA VISTA RE-ENQUADRA** — o quadro 0
/// do app tem o painel a `1918` px e o quadro 1 a `1310`. CONTROLO: depois de o artista mexer na
/// vista (um pan), uma mudança de tamanho já não lhe rouba o enquadramento.
#[test]
fn a_panel_that_resizes_before_the_artist_touches_the_view_is_refitted() {
    let snap = a_galaxia();
    let certo = layout();
    let mut largo = certo;
    largo.motion_graph = Rect {
        x: 6.0,    // LITERAL-PX-OK: o quadro 0 medido no app
        w: 1918.0, // LITERAL-PX-OK: o quadro 0 medido no app
        ..certo.motion_graph
    };
    let centro_x = |rects: &[Rect]| {
        let (a, b) = rects.iter().fold((f32::MAX, f32::MIN), |(a, b), r| {
            (a.min(r.x), b.max(r.x + r.w))
        });
        (a + b) * 0.5
    };
    let painel = certo.motion_graph;
    set_current_motion_graph(Some(snap.clone()));
    for artista_mexeu in [false, true] {
        let mut host = ph2d_ui_testkit::MockPanelHost::with_panel::<crate::MotionGraphPanel>();
        let mut state = MotionGraphPanelState::default();
        let _ = pinta(&mut host, &mut state, largo, &snap);
        if artista_mexeu {
            state.view.pan_x += 40.0;
        }
        let antes = state.view;
        let rects = pinta(&mut host, &mut state, certo, &snap);
        if artista_mexeu {
            assert_eq!(state.view, antes, "a vista que o artista mexeu é dele");
        } else {
            let desvio = centro_x(&rects) - (painel.x + painel.w * 0.5);
            assert!(
                desvio.abs() < 1.0,
                "os cartões têm de ficar ao centro do painel que se vê, e estão {desvio} px ao lado"
            );
        }
    }
    set_current_motion_graph(None);
}
