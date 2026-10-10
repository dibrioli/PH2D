//! ⭐⭐⭐ **UM TOQUE QUE TREME AINDA É UM TOQUE** (BUGS do Motion #18, report do dono de 10/10:
//! *«apertar botões em todo módulo … é impreciso e inconsistente»*).
//!
//! Estes gates passam pelo caminho REAL: o pintor regista os alvos, o despacho do núcleo recebe
//! `Down → Move → Up` como a mão os dá, e o painel drena os gestos na pintura seguinte. Os gates
//! antigos empurravam um `Click` sintético e saltavam exactamente a metade onde o defeito morava
//! (sem folga, um pixel de tremor fazia do clique um arrasto).

use crate::MotionGraphPanel;
use crate::snapshot::{
    CardParam, GraphNodeView, GraphViewSnapshot, NodeViewKind, drain_intents,
    set_current_motion_graph,
};
use crate::state::{MotionGraphPanelState, ViewState};
use crate::{GraphIntent, geom};
use ph2d_editor_core::zones::Rect;
use ph2d_host::{PointerButton, PointerEvent, PointerKind, PointerSource};
use ph2d_node_registry::{NodeSilhouette, NodeUiCategory, ParamUiHint, ParamWidget};
use ph2d_ui_testkit::MockPanelHost;

const VIEWPORT: Rect = Rect {
    x: 0.0,
    y: 0.0,
    w: 900.0,
    h: 600.0,
};

fn hint(param: &'static str, widget: ParamWidget) -> ParamUiHint {
    ParamUiHint {
        param,
        label: "Rows",
        min: 0.0,
        max: 10.0,
        step: 0.1,
        widget,
    }
}

/// Um cartão com um NÚMERO (fileira 0) e um TOGGLE (fileira 1).
fn cena() -> GraphViewSnapshot {
    GraphViewSnapshot {
        level: None,
        breadcrumb: Vec::new(),
        nodes: vec![GraphNodeView {
            primary_input: 0,
            kind: NodeViewKind::Node,
            id: 7,
            display_name: "Grid".into(),
            category: NodeUiCategory::Source,
            silhouette: NodeSilhouette::Rect,
            x: 40.0,
            y: 40.0,
            inputs: vec![],
            outputs: vec![],
            readout: None,
            count: None,
            hot: false,
            is_sink: false,
            preview: None,
            bypassed: false,
            inert: false,
            thumbnail: None,
            params: vec![
                CardParam::from_hint(hint("rows", ParamWidget::Slider), 5.0),
                CardParam::from_hint(hint("on", ParamWidget::Toggle), 0.0),
            ],
            sections: Vec::new(),
        }],
        edges: Vec::new(),
        backdrops: Vec::new(),
        probe: None,
        now: 0.0,
    }
}

struct Mao {
    host: MockPanelHost,
    state: MotionGraphPanelState,
    t: u128,
}

impl Mao {
    fn nova(zoom: f32) -> Self {
        set_current_motion_graph(Some(cena()));
        let _ = drain_intents();
        let mut m = Self {
            host: MockPanelHost::with_panel::<MotionGraphPanel>(),
            state: MotionGraphPanelState {
                fitted: true,
                view: ViewState {
                    pan_x: 0.0,
                    pan_y: 0.0,
                    zoom,
                },
                ..MotionGraphPanelState::default()
            },
            t: 0,
        };
        m.pinta();
        m
    }

    fn pinta(&mut self) {
        let mut layout = ph2d_editor_core::screens::layout::HeroLayout::for_viewport(VIEWPORT);
        layout.motion_graph = VIEWPORT;
        let _ = self
            .host
            .paint_with_layout::<MotionGraphPanel>(&mut self.state, layout, VIEWPORT);
    }

    fn evento(&mut self, kind: PointerKind, x: f32, y: f32) {
        self.t += 1_000_000_000; // um segundo: nunca um duplo clique
        let _ = self.host.dispatch_pointer_event(PointerEvent {
            x,
            y,
            pressure: 1.0,
            kind,
            source: PointerSource::Mouse,
            button: PointerButton::Primary,
            timestamp_ns: self.t,
        });
    }

    /// Aperta em `(x, y)`, a mão anda `(dx, dy)`, solta lá — e o painel drena na pintura seguinte.
    fn gesto(&mut self, (x, y): (f32, f32), (dx, dy): (f32, f32)) -> Vec<GraphIntent> {
        let _ = drain_intents();
        self.evento(PointerKind::Down, x, y);
        self.evento(PointerKind::Move, x + dx, y + dy);
        self.evento(PointerKind::Up, x + dx, y + dy);
        self.pinta();
        drain_intents()
    }

    /// O centro da TRILHA da fileira `i` (o sítio onde a mão aponta um campo).
    fn fileira(&self, i: usize) -> (f32, f32) {
        let view = geom::View::new(VIEWPORT, self.state.view);
        let snap = cena();
        let r = geom::param_track_rect(geom::param_row_rect(&snap.nodes[0], &view, i), view.zoom);
        (r.x + r.w * 0.5, r.y + r.h * 0.5)
    }
}

fn set_params(saiu: &[GraphIntent]) -> Vec<(&'static str, f32)> {
    saiu.iter()
        .filter_map(|i| match i {
            GraphIntent::SetParam { param, value, .. } => Some((*param, *value)),
            _ => None,
        })
        .collect()
}

/// ⭐ Um número: um toque que treme 2 px ABRE a caixa e não escreve nada; só um arrasto de verdade
/// (12 px) escreve o valor. Antes da cura o tremor escrevia um valor e a caixa nunca abria.
#[test]
fn a_trembling_tap_on_a_number_opens_its_box_and_writes_nothing() {
    let mut m = Mao::nova(1.0);
    let p = m.fileira(0);
    let saiu = m.gesto(p, (2.0, 1.0));
    assert!(set_params(&saiu).is_empty(), "o tremor escreveu {saiu:?}");
    assert!(m.state.param_edit.is_some(), "a caixa tem de abrir");

    let mut m = Mao::nova(1.0);
    let saiu = m.gesto(p, (12.0, 0.0));
    assert!(
        set_params(&saiu).iter().any(|(n, _)| *n == "rows"),
        "CONTROLO: o arrasto escreve: {saiu:?}"
    );
    assert!(
        m.state.param_edit.is_none(),
        "e um arrasto não abre a caixa"
    );
}

/// ⭐ Um toggle troca com um toque que treme — sempre, e não «às vezes».
#[test]
fn a_trembling_tap_on_a_toggle_toggles_it() {
    let mut m = Mao::nova(1.0);
    let p = m.fileira(1);
    for (dx, dy) in [(0.0, 0.0), (2.0, -2.0), (-3.0, 3.0), (4.0, 0.0)] {
        let saiu = m.gesto(p, (dx, dy));
        assert_eq!(
            set_params(&saiu),
            vec![("on", 1.0)],
            "tremor ({dx}, {dy}): {saiu:?}"
        );
    }
}

/// ⭐ **Ler é clicar; agarrar é arrastar** — no zoom em que a fileira se LÊ mas não regista alvo
/// (`0,655 ≤ zoom < 0,818`), um toque nela age nela; um arrasto continua a mover o cartão.
#[test]
fn where_a_row_is_read_but_not_grabbed_a_tap_acts_and_a_drag_moves_the_card() {
    let mut m = Mao::nova(0.72);
    let view = geom::View::new(VIEWPORT, m.state.view);
    assert!(geom::param_text_is_drawn(&view) && !geom::param_row_is_grabbable(&view));
    let p = m.fileira(1);
    let saiu = m.gesto(p, (1.0, 1.0));
    assert_eq!(
        set_params(&saiu),
        vec![("on", 1.0)],
        "o toque troca: {saiu:?}"
    );

    let saiu = m.gesto(p, (30.0, 0.0));
    assert!(
        set_params(&saiu).is_empty(),
        "o arrasto não mexe no param: {saiu:?}"
    );
    assert!(
        saiu.iter()
            .any(|i| matches!(i, GraphIntent::MoveNodes { .. })),
        "o arrasto move o cartão: {saiu:?}"
    );
}
