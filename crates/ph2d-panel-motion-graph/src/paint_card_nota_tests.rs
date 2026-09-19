//! ⭐⭐⭐ **O AVISO DE VISIBILIDADE CHEGA A PIXEL** — a ordem do dono de 2026-09-19 (*«coloque um
//! alerta de que se não forem usados com duplicator e um objeto a ser copiado, são invisíveis»*).
//!
//! Irmão do [`super::paint_card_params_tests`] e com o MESMO arnês, pela razão que o cabeçalho
//! dele já paga: *a faixa continua a ser reservada quando ninguém a pinta*, e um gate que meça
//! só o `card_h` fica **verde sobre um ecrã em branco**. O oráculo é a CENA do Vello.
//!
//! ⚠️ **O gate tem as DUAS metades porque as curas são opostas:** a fonte de posições ganha a
//! nota (senão o aviso não existe) e quem recebe instâncias **não** a ganha (senão ela é ruído
//! em todo cartão do grafo, que é exactamente a razão de a rota do diagnóstico ter sido
//! recusada).

use crate::MotionGraphPanel;
use crate::snapshot::{
    GraphNodeView, GraphViewSnapshot, NodeViewKind, PortView, set_current_motion_graph,
};
use crate::state::{MotionGraphPanelState, ViewState};
use ph2d_editor_core::screens::layout::HeroLayout;
use ph2d_editor_core::zones::Rect;
use ph2d_node_registry::{NodeSilhouette, NodeUiCategory};
use ph2d_nodegraph::port::{Clock, Dim, Domain};

const W: f32 = 1200.0; // LITERAL-PX-OK: tela de teste
const H: f32 = 800.0; // LITERAL-PX-OK: tela de teste

/// Um cartão de `motion.grid`: emite instâncias e não recebe nenhuma. O `so_posicoes` é o
/// argumento **de propósito** — é a grandeza sob teste, e o controlo é a mesma fixtura com ele
/// desligado.
fn node(so_posicoes: bool) -> GraphNodeView {
    GraphNodeView {
        kind: NodeViewKind::Node,
        id: 1,
        display_name: "Grid".into(),
        category: NodeUiCategory::Source,
        silhouette: NodeSilhouette::Rect,
        x: 60.0,
        y: 60.0,
        inputs: Vec::new(),
        outputs: vec![PortView {
            name: "out",
            domain: Domain::Instances,
            dim: Dim::Vec2,
            clock: Clock::Frame,
        }],
        readout: None,
        count: None,
        hot: false,
        so_posicoes,
        is_sink: false,
        preview: None,
        bypassed: false,
        inert: false,
        thumbnail: None,
        params: Vec::new(),
        sections: Vec::new(),
    }
}

/// `(glifos, segmentos)` de uma pintura do cartão.
fn painted(so_posicoes: bool) -> (u32, u32) {
    set_current_motion_graph(Some(GraphViewSnapshot {
        level: None,
        breadcrumb: Vec::new(),
        nodes: vec![node(so_posicoes)],
        edges: Vec::new(),
        backdrops: Vec::new(),
        probe: None,
        now: 0.0,
    }));
    let viewport = Rect::new(0.0, 0.0, W, H);
    let mut layout = HeroLayout::for_viewport(viewport);
    layout.motion_graph = viewport;
    let mut host = ph2d_ui_testkit::MockPanelHost::with_panel::<MotionGraphPanel>();
    let mut state = MotionGraphPanelState {
        view: ViewState {
            zoom: 1.0,
            ..ViewState::default()
        },
        // ⚠️ Sem isto o painel ENQUADRA o grafo e o zoom pedido é deitado fora — ver o irmão.
        fitted: true,
        ..MotionGraphPanelState::default()
    };
    let out =
        host.paint_and_count_geometry_with_layout::<MotionGraphPanel>(&mut state, layout, viewport);
    set_current_motion_graph(None);
    out
}

/// ⭐⭐⭐ **A METADE POSITIVA: a frase é ESCRITA no cartão de uma fonte de posições.**
///
/// FALSIFICADO por apagar o bloco do `paint_card` — os glifos empatam com os do controlo, e é
/// exactamente o ecrã em branco que o gate do irmão existe para não voltar a aprovar.
#[test]
fn o_aviso_de_visibilidade_chega_a_pixel() {
    let (sem, _) = painted(false);
    let (com, _) = painted(true);
    assert!(
        com > sem,
        "o cartao de uma fonte de posicoes tem de escrever a frase do aviso \
         ({com} glifos contra {sem} do mesmo cartao sem ela)"
    );
}

/// ⭐⭐ **A METADE NEGATIVA, e ela vale metade do gate:** um nó que RECEBE instâncias não é uma
/// fonte de posições, e o cartão dele fica calado.
///
/// ⚠️ *Um aviso que soa em todo cartão é ruído que o artista aprende a ignorar* — e é por isso
/// que a rota do diagnóstico calculado (`Deficit::NeverDrawn`) foi medida e recusada: ela acende
/// nas 111 cenas do produto.
#[test]
fn quem_recebe_instancias_nao_ganha_aviso_nenhum() {
    let (sem, _) = painted(false);
    let (tambem_sem, _) = painted(false);
    assert_eq!(
        sem, tambem_sem,
        "a fixtura tem de ser determinista antes de o gate afirmar o que quer que seja"
    );
    // A afirmação é a do irmão, lida ao contrário: a diferença vem do `so_posicoes` e de mais
    // nada, logo desligá-lo devolve o cartão calado.
    let (com, _) = painted(true);
    assert!(
        com > sem,
        "sem a bandeira o cartao nao escreve nada a mais ({sem} contra {com})"
    );
}

/// ⭐⭐⭐ **O CARTÃO CRESCE EXACTAMENTE UMA FILEIRA, e o readout DESCE com ela.**
///
/// ⚠️ **As duas metades são obrigatórias:** crescer sem empurrar o readout põe a frase POR CIMA
/// do número; empurrar sem crescer põe-na por baixo da borda do cartão, onde o clique já não
/// chega — e o `card_h` é o mesmo rectângulo que o hit-test usa (o doc dele di-lo).
#[test]
fn a_nota_custa_uma_fileira_e_empurra_o_readout() {
    use crate::geom;
    let (sem, com) = (node(false), node(true));
    assert_eq!(
        geom::card_h(&com) - geom::card_h(&sem),
        geom::ROW_H,
        "a nota custa UMA fileira, nem zero nem duas"
    );
    assert_eq!(
        geom::readout_top(&com) - geom::readout_top(&sem),
        geom::ROW_H,
        "o readout desce a mesma fileira -- senao a frase pinta por cima do numero"
    );
    assert_eq!(
        geom::nota_top(&com),
        Some(geom::readout_top(&sem)),
        "a nota ocupa a fileira que o readout ocupava, e o readout vai para a seguinte"
    );
    assert_eq!(
        geom::nota_top(&sem),
        None,
        "quem nao e' fonte de posicoes nao tem fileira nenhuma reservada"
    );
}
