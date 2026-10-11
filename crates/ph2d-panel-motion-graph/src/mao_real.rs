//! **A MÃO REAL** — o arnês de todo gate de GESTO deste painel (BUGS do Motion #18).
//!
//! O pintor regista os alvos, o despacho do núcleo recebe `Down → Move → Up` como a mão os dá
//! (com a folga `PressSlop`, os tempos, a captura), e o painel drena os gestos na pintura
//! seguinte. Um gate que empurra um `GesturePhase` sintético já assumiu a resposta do despacho —
//! e o tremor que virava arrasto morava exactamente nessa metade.

use crate::MotionGraphPanel;
use crate::snapshot::{GraphViewSnapshot, drain_intents, set_current_motion_graph};
use crate::state::{MotionGraphPanelState, ViewState};
use crate::{GraphIntent, geom};
use ph2d_editor_core::zones::Rect;
use ph2d_host::{PointerButton, PointerEvent, PointerKind, PointerSource};
use ph2d_ui_testkit::MockPanelHost;

pub(crate) const VIEWPORT: Rect = Rect {
    x: 0.0,
    y: 0.0,
    w: 900.0, // LITERAL-PX-OK: janela fictícia do arnês de teste, não chrome
    h: 600.0, // LITERAL-PX-OK: janela fictícia do arnês de teste, não chrome
};

pub(crate) struct Mao {
    host: MockPanelHost,
    pub(crate) state: MotionGraphPanelState,
    cena: GraphViewSnapshot,
    t: u128,
}

impl Mao {
    /// O painel a `zoom`, sem pan, com `cena` publicada e já pintada uma vez.
    pub(crate) fn nova(cena: GraphViewSnapshot, zoom: f32) -> Self {
        set_current_motion_graph(Some(cena.clone()));
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
            cena,
            t: 0,
        };
        m.pinta();
        m
    }

    pub(crate) fn pinta(&mut self) {
        let mut layout = ph2d_editor_core::screens::layout::HeroLayout::for_viewport(VIEWPORT);
        layout.motion_graph = VIEWPORT;
        let _ = self
            .host
            .paint_with_layout::<MotionGraphPanel>(&mut self.state, layout, VIEWPORT);
    }

    pub(crate) fn evento(&mut self, kind: PointerKind, x: f32, y: f32) {
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
    pub(crate) fn gesto(&mut self, (x, y): (f32, f32), (dx, dy): (f32, f32)) -> Vec<GraphIntent> {
        let _ = drain_intents();
        self.evento(PointerKind::Down, x, y);
        self.evento(PointerKind::Move, x + dx, y + dy);
        self.evento(PointerKind::Up, x + dx, y + dy);
        self.pinta();
        drain_intents()
    }

    /// O centro da TRILHA da fileira `i` do cartão `no` (o sítio onde a mão aponta um campo).
    pub(crate) fn fileira(&self, no: usize, i: usize) -> (f32, f32) {
        let view = geom::View::new(VIEWPORT, self.state.view);
        let r = geom::param_track_rect(
            geom::param_row_rect(&self.cena.nodes[no], &view, i),
            view.zoom,
        );
        (r.x + r.w * 0.5, r.y + r.h * 0.5)
    }
}

/// ⛔ **OS GESTOS SINTÉTICOS NOS TESTES DO GRAFO SÓ DESCEM** — catraca EXACTA por ficheiro.
///
/// Um ficheiro `*_tests.rs` novo, ou um que cresce, reprova: o gate de gesto novo usa [`Mao`].
/// Um que desce também reprova até a tabela descer com ele — o número é o real, nunca licença.
/// Fronteira: só os `*_tests.rs` (a convenção da crate); os `match` de produção não contam.
#[test]
fn synthetic_gestures_in_the_graph_tests_only_shrink() {
    const TABELA: &[(&str, usize)] = &[
        ("interact_context_menu_tests.rs", 4),
        ("interact_drop_tests.rs", 9),
        ("interact_f2_tests.rs", 25),
        ("interact_f2b_tests.rs", 27),
        ("interact_node_drop_tests.rs", 9),
        ("interact_param_row_selector_tests.rs", 1),
        ("interact_param_row_tests.rs", 6),
        ("interact_select_tests.rs", 4),
        ("interact_subgraph_tests.rs", 13),
        ("interact_tests.rs", 11),
        ("interact_wire_tests.rs", 4),
    ];
    let src = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut medido: Vec<(String, usize)> = std::fs::read_dir(&src)
        .expect("src/ legível")
        .filter_map(|e| {
            let nome = e.ok()?.file_name().into_string().ok()?;
            nome.ends_with("_tests.rs").then_some(nome)
        })
        .filter_map(|nome| {
            let n = std::fs::read_to_string(src.join(&nome))
                .ok()?
                .matches("GesturePhase::")
                .count();
            (n > 0).then_some((nome, n))
        })
        .collect();
    medido.sort();
    assert!(
        medido.len() >= 5,
        "controlo: a varredura viu {} ficheiros",
        medido.len()
    );
    let tabela: Vec<(String, usize)> = TABELA.iter().map(|(f, n)| ((*f).to_string(), *n)).collect();
    assert_eq!(
        medido, tabela,
        "gesto SINTÉTICO num teste do grafo mudou: um gate de gesto novo passa pela `mao_real::Mao` \
         (Down/Move/Up no despacho); se o número DESCEU, desça a tabela"
    );
}
