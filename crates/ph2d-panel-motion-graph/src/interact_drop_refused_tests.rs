//! ⛔ **Um fio solto no CORPO de um nó que não o aceita diz PORQUÊ — não abre a paleta** (smoke
//! do dono da `=141`, 13/10: *«o modal de nós se abre»*). Pela MÃO REAL ([`crate::mao_real::Mao`]):
//! o despacho recebe `Down → Move → Up` sobre os alvos que o pintor registou.
//!
//! A cena é a da `=141` em miniatura: um `Oscillator` (corrente `Vec2`), um `Transform` cuja
//! entrada do mesmo tipo já está ocupada e cujo único param oferecido (`Scale`) é um VALOR, e um
//! `LFO` (um valor) — o controlo que abre o menu.

use crate::GraphIntent;
use crate::geom;
use crate::mao_real::{Mao, VIEWPORT};
use crate::snapshot::{
    ChoiceTarget, GraphEdgeView, GraphNodeView, GraphViewSnapshot, HiddenPorts, NodeViewKind,
    PortChoice, PortView, set_card_hidden_ports,
};
use crate::state::MenuBody;
use ph2d_node_registry::{NodeSilhouette, NodeUiCategory};
use ph2d_nodegraph::port::{Clock, Dim, Domain};

const OSC: u32 = 1;
const TRANSFORM: u32 = 2;
const LFO: u32 = 4;

fn porta(dim: Dim) -> PortView {
    PortView {
        name: "out",
        domain: Domain::Instances,
        dim,
        clock: Clock::Frame,
    }
}

fn no(
    id: u32,
    nome: &str,
    x: f32,
    y: f32,
    ins: Vec<PortView>,
    outs: Vec<PortView>,
) -> GraphNodeView {
    GraphNodeView {
        primary_input: 0,
        kind: NodeViewKind::Node,
        id,
        display_name: nome.into(),
        category: NodeUiCategory::Utility,
        silhouette: NodeSilhouette::Rect,
        x,
        y,
        inputs: ins,
        outputs: outs,
        readout: None,
        count: None,
        hot: false,
        is_sink: false,
        preview: None,
        bypassed: false,
        inert: false,
        thumbnail: None,
        params: Vec::new(),
        sections: Vec::new(),
    }
}

fn cena() -> GraphViewSnapshot {
    set_card_hidden_ports(
        [(
            TRANSFORM,
            HiddenPorts {
                inputs: vec![PortChoice {
                    node: TRANSFORM,
                    target: ChoiceTarget::Param("scale"),
                    label: "Transform: Scale".into(),
                    category: NodeUiCategory::Utility,
                    port_type: porta(Dim::Scalar),
                }],
                outputs: Vec::new(),
            },
        )]
        .into_iter()
        .collect(),
    );
    GraphViewSnapshot {
        level: None,
        breadcrumb: Vec::new(),
        nodes: vec![
            no(
                OSC,
                "Oscillator",
                40.0,
                40.0,
                vec![],
                vec![porta(Dim::Vec2)],
            ),
            no(
                TRANSFORM,
                "Transform",
                420.0,
                40.0,
                vec![porta(Dim::Vec2)],
                vec![],
            ),
            no(LFO, "LFO", 40.0, 300.0, vec![], vec![porta(Dim::Scalar)]),
        ],
        // A entrada do `Transform` já é alimentada (as peças do `Duplicator`, na `=141`).
        edges: vec![GraphEdgeView {
            from_node: 3,
            from_port: 0,
            to_node: TRANSFORM,
            to_port: 0,
            delayed: false,
            out_domain: Domain::Instances,
        }],
        backdrops: Vec::new(),
        probe: None,
        now: 0.0,
    }
}

/// Puxa um fio da saída 0 do nó `de` e solta-o em `onde` (ecrã). Devolve o que o painel emitiu.
fn larga(m: &mut Mao, s: &GraphViewSnapshot, de: u32, onde: (f32, f32)) -> Vec<GraphIntent> {
    let view = geom::View::new(VIEWPORT, m.state.view);
    let n = s.nodes.iter().find(|n| n.id == de).expect("o nó existe");
    let (x, y) = geom::socket_center(n, &view, true, 0);
    m.gesto((x, y), (onde.0 - x, onde.1 - y))
}

fn corpo(m: &Mao, s: &GraphViewSnapshot, id: u32) -> (f32, f32) {
    let view = geom::View::new(VIEWPORT, m.state.view);
    let r = geom::card_rect(s.nodes.iter().find(|n| n.id == id).expect("o nó"), &view);
    (r.x + r.w * 0.5, r.y + r.h * 0.5)
}

/// ⭐⭐ **(i)** O fio do `Oscillator` solto no corpo do `Transform` emite a RECUSA — nem a paleta
/// (`OpenLibrary`), nem um menu, nem um fio.
#[test]
fn o_fio_de_uma_corrente_solto_num_corpo_que_nao_o_aceita_e_recusado_e_nao_abre_a_paleta() {
    let s = cena();
    let mut m = Mao::nova(s.clone(), 1.0);
    let alvo = corpo(&m, &s, TRANSFORM);
    let saiu = larga(&mut m, &s, OSC, alvo);
    assert_eq!(
        saiu,
        vec![GraphIntent::DropRefused {
            from_node: OSC,
            from_port: 0,
            to_node: TRANSFORM,
        }],
        "a largada no corpo tem de dizer porquê, não abrir nada"
    );
    assert!(m.state.menu.is_none(), "abriu um menu: {:?}", m.state.menu);
}

/// **(iii)** Controlo: o MESMO fio solto no canvas VAZIO continua a abrir a paleta filtrada.
#[test]
fn o_mesmo_fio_no_canvas_vazio_continua_a_abrir_a_paleta() {
    let s = cena();
    let mut m = Mao::nova(s.clone(), 1.0);
    let saiu = larga(&mut m, &s, OSC, (700.0, 520.0));
    assert!(
        matches!(
            saiu.as_slice(),
            [GraphIntent::OpenLibrary {
                connect_from: Some((OSC, 0)),
                ..
            }]
        ),
        "o canvas vazio é a paleta: {saiu:?}"
    );
}

/// Controlo: o fio do `LFO` (um valor) solto no MESMO corpo abre o menu com `Transform: Scale` — o
/// passo (3) do roteiro da `=141`.
#[test]
fn o_fio_de_um_valor_no_mesmo_corpo_abre_o_menu_dos_params() {
    let s = cena();
    let mut m = Mao::nova(s.clone(), 1.0);
    let alvo = corpo(&m, &s, TRANSFORM);
    let saiu = larga(&mut m, &s, LFO, alvo);
    assert!(saiu.is_empty(), "o menu ainda não escolheu nada: {saiu:?}");
    let rotulos: Vec<&str> = match m.state.menu.as_ref().map(|menu| &menu.body) {
        Some(MenuBody::CardPorts { rows, .. }) => rows.iter().map(|r| r.label.as_str()).collect(),
        outro => panic!("o fio do LFO tem de abrir o menu dos params: {outro:?}"),
    };
    assert_eq!(rotulos, vec!["Transform: Scale"]);
}
