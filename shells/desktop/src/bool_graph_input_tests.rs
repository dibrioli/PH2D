//! Gates da DECISÃO do gesto do diagrama.
//!
//! ⚠️ Eles constroem os pontos de teste a partir das MESMAS funções que o card desenha
//! (`bool_graph_node_center` / `_link_points`) — nunca de coordenadas escritas à mão. Um número a
//! mão passaria a mentir no dia em que um espaçamento mudasse, e o gate diria que está tudo bem
//! enquanto o artista clica ao lado do que vê.

use super::{DownAction, down_action, drag_move, up_intents};
use ph2d_editor::widget::{
    BoolGraphDrag, BoolGraphIntent, BoolGraphLink, BoolGraphNode, BoolGraphView,
    bool_graph_canvas_rect, bool_graph_card_size, bool_graph_link_points, bool_graph_node_center,
    bool_graph_node_radius, bool_graph_ring_inner_radius,
};
use ph2d_editor::zones::Rect;

fn node(id: u64) -> BoolGraphNode {
    BoolGraphNode {
        id,
        label: format!("F{id}"),
        consumed: false,
        at: None,
    }
}

/// Três formas, com `2 → 1` em `Union`.
fn view() -> BoolGraphView {
    BoolGraphView {
        nodes: vec![node(1), node(2), node(3)],
        links: vec![BoolGraphLink {
            from: 2,
            to: 1,
            op: 0,
        }],
        cycle: false,
    }
}

fn card(v: &BoolGraphView) -> Rect {
    let (w, h) = bool_graph_card_size(v);
    Rect::new(200.0, 100.0, w, h)
}

/// Um ponto no ARO do círculo `i` — a alça de ligar.
fn on_ring(r: Rect, v: &BoolGraphView, i: usize) -> (f32, f32) {
    let c = bool_graph_node_center(r, v, i);
    (
        c.0 + (bool_graph_ring_inner_radius() + bool_graph_node_radius()) * 0.5,
        c.1,
    )
}

/// **O MIOLO ARMA UM MOVIMENTO; O ARO ARMA UMA LIGAÇÃO.**
///
/// ⚠️ Sem a separação, arrastar para mover e arrastar para ligar seriam o mesmo gesto — e um deles
/// teria de ganhar, deixando o outro inexprimível.
#[test]
fn o_miolo_arma_movimento_e_o_aro_arma_ligacao() {
    let v = view();
    let r = card(&v);
    for i in 0..v.nodes.len() {
        let miolo = bool_graph_node_center(r, &v, i);
        match down_action(false, true, Some(r), &v, miolo, false) {
            Some(DownAction::Arm(d)) => {
                assert_eq!(d.from, v.nodes[i].id);
                assert!(!d.link, "o miolo armou uma LIGAÇÃO");
                assert!(!d.moved, "o gesto nasce como um possível CLIQUE");
            }
            outra => panic!("o miolo do círculo {i} deu {outra:?}"),
        }
        match down_action(false, true, Some(r), &v, on_ring(r, &v, i), false) {
            Some(DownAction::Arm(d)) => assert!(d.link, "o aro armou um MOVIMENTO"),
            outra => panic!("o aro do círculo {i} deu {outra:?}"),
        }
    }
}

/// **UM CLIQUE NO MIOLO SELECIONA A FORMA NO CANVAS.**
///
/// ⚠️ Não é conforto: é a ÚNICA porta. Um operando consumido desenha vazio, e a lei do canvas é
/// *"nada desenhado, nada pego"* — sem isto ele fica inalcançável pelo ponteiro (Enio, 2026-08-22:
/// *"só é possível selecionar e mover no canvas uma shape"*).
#[test]
fn um_clique_no_miolo_seleciona_a_forma() {
    let v = view();
    let r = card(&v);
    let parado = BoolGraphDrag {
        from: 2,
        link: false,
        at: [10.0, 10.0],
        moved: false,
    };
    assert_eq!(
        up_intents(Some(r), &v, parado, bool_graph_node_center(r, &v, 1)),
        vec![BoolGraphIntent::Select { id: 2 }]
    );
}

/// **UM ARRASTO DO MIOLO MOVE, E ESCREVE UMA VEZ SÓ.**
///
/// ⚠️ A escrita única é o que protege o undo: escrever a cada frame do arrasto criaria um passo por
/// frame, e o Ctrl+Z andaria pixel a pixel para trás.
#[test]
fn um_arrasto_do_miolo_move_e_escreve_uma_vez_so() {
    let v = view();
    let r = card(&v);
    let arrastado = BoolGraphDrag {
        from: 2,
        link: false,
        at: [120.0, 90.0],
        moved: true,
    };
    assert_eq!(
        up_intents(Some(r), &v, arrastado, (0.0, 0.0)),
        vec![BoolGraphIntent::Move {
            id: 2,
            at: [120.0, 90.0]
        }],
        "o arrasto não produziu UMA e só uma escrita"
    );
}

/// **O MOVIMENTO LIGA `moved` E ELE NUNCA VOLTA A DESLIGAR.**
///
/// ⚠️ Um gesto que oscilasse entre clique e arrasto faria o *Up* significar coisas diferentes
/// conforme o último pixel: soltar no ponto de partida depois de dar uma volta selecionaria a forma
/// em vez de a deixar onde ficou.
#[test]
fn o_movimento_liga_moved_e_ele_nunca_desliga() {
    let v = view();
    let r = card(&v);
    let plane = bool_graph_canvas_rect(r);
    let d0 = BoolGraphDrag {
        from: 2,
        link: false,
        at: [100.0, 100.0],
        moved: false,
    };
    let d1 = drag_move(Some(r), d0, (plane.x + 200.0, plane.y + 150.0)).unwrap();
    assert!(d1.moved, "o ponteiro andou e o gesto não virou arrasto");

    // ⚠️ **O ponteiro PARA no meio do arrasto** — um frame com o mesmo ponto. É aqui que a lei se
    // prova: comparar com a posição ANTERIOR (em vez de acumular) faria uma pausa devolver o gesto
    // ao estado de clique, e soltar depois dela SELECIONARIA a forma em vez de a deixar onde ficou.
    // Um mutante sobreviveu ao gate anterior, que só voltava ao ponto de partida — e ali as duas
    // leituras respondem igual.
    let parado = drag_move(Some(r), d1, (plane.x + 200.0, plane.y + 150.0)).unwrap();
    assert!(parado.moved, "uma PAUSA desfez o arrasto");

    // E voltar ao ponto de partida também não o desfaz.
    let d2 = drag_move(Some(r), parado, (plane.x + 100.0, plane.y + 100.0)).unwrap();
    assert!(d2.moved, "voltar ao início desfez o arrasto");
}

/// **UM ARRASTO DO ARO SOLTO NOUTRO CÍRCULO LIGA, NA DIREÇÃO DO GESTO.**
#[test]
fn um_arrasto_do_aro_solto_noutro_circulo_liga() {
    let v = view();
    let r = card(&v);
    let d = BoolGraphDrag {
        from: 1,
        link: true,
        at: [0.0, 0.0],
        moved: true,
    };
    let alvo = bool_graph_node_center(r, &v, 2); // a forma 3
    assert_eq!(
        up_intents(Some(r), &v, d, alvo),
        vec![BoolGraphIntent::Link {
            from: 1,
            to: 3,
            op: 0
        }]
    );
}

/// **UMA LIGAÇÃO NOVA HERDA A OPERAÇÃO DAS QUE JÁ EXISTEM.**
///
/// ⚠️ É o que faz montar uma rede uniforme custar **um arrasto por ligação**, em vez de um arrasto
/// mais quatro cliques a girar de volta ao mesmo verbo que todas as outras já usam.
#[test]
fn uma_ligacao_nova_herda_a_operacao_das_existentes() {
    let mut v = view();
    v.links[0].op = 2; // Intersect
    let r = card(&v);
    let d = BoolGraphDrag {
        from: 1,
        link: true,
        at: [0.0, 0.0],
        moved: true,
    };
    assert_eq!(
        up_intents(Some(r), &v, d, bool_graph_node_center(r, &v, 2)),
        vec![BoolGraphIntent::Link {
            from: 1,
            to: 3,
            op: 2
        }]
    );
}

/// **SOLTAR UMA LIGAÇÃO EM NADA — OU EM SI MESMA — NÃO LIGA.**
#[test]
fn soltar_uma_ligacao_em_nada_ou_em_si_mesma_nao_liga() {
    let v = view();
    let r = card(&v);
    let d = BoolGraphDrag {
        from: 1,
        link: true,
        at: [0.0, 0.0],
        moved: true,
    };
    assert!(up_intents(Some(r), &v, d, (r.x + 2.0, r.y + r.h - 2.0)).is_empty());
    let mesmo = bool_graph_node_center(r, &v, 0); // a forma 1
    assert!(
        up_intents(Some(r), &v, d, mesmo).is_empty(),
        "ligou a si mesma"
    );
}

/// **UM CLIQUE NUM TRAÇO GIRA A OPERAÇÃO** — e a ligação continua a mesma.
#[test]
fn um_clique_num_traco_gira_a_operacao() {
    let v = view();
    let r = card(&v);
    let pts = bool_graph_link_points(r, &v, v.links[0]);
    let meio = (pts[0].0.midpoint(pts[1].0), pts[0].1.midpoint(pts[1].1));
    assert_eq!(
        down_action(false, true, Some(r), &v, meio, false),
        Some(DownAction::Intent(BoolGraphIntent::Link {
            from: 2,
            to: 1,
            op: 1
        })),
        "a rotação não saiu de Union para Subtract"
    );
}

/// **SHIFT+CLIQUE CORTA** — o gesto próprio, que a rotação deliberadamente não faz.
///
/// ⚠️ É a lei: cortar por sobre-rodar seria o engano mais fácil do diagrama. Quem quer ir de
/// *Union* a *Subtract* e passa do ponto apagaria a ligação em vez de continuar a rodar.
#[test]
fn shift_clique_corta_e_a_rotacao_nunca_corta() {
    let v = view();
    let r = card(&v);
    let pts = bool_graph_link_points(r, &v, v.links[0]);
    let meio = (pts[0].0.midpoint(pts[1].0), pts[0].1.midpoint(pts[1].1));
    assert_eq!(
        down_action(false, true, Some(r), &v, meio, true),
        Some(DownAction::Intent(BoolGraphIntent::Unlink {
            from: 2,
            to: 1
        }))
    );
    // E a rotação, dando duas voltas completas, nunca produz um corte.
    let mut op = 0u8;
    for _ in 0..8 {
        let mut w = v.clone();
        w.links[0].op = op;
        let pts = bool_graph_link_points(r, &w, w.links[0]);
        let p = (pts[0].0.midpoint(pts[1].0), pts[0].1.midpoint(pts[1].1));
        match down_action(false, true, Some(r), &w, p, false) {
            Some(DownAction::Intent(BoolGraphIntent::Link { op: next, .. })) => op = next,
            outra => panic!("a rotação produziu {outra:?} em vez de uma ligação"),
        }
    }
}

/// **A BANDA DE TÍTULO ARRASTA, E ELA GANHA DO CORPO.**
#[test]
fn a_banda_de_titulo_arrasta() {
    let v = view();
    let r = card(&v);
    assert_eq!(
        down_action(true, false, Some(r), &v, (r.x, r.y), false),
        Some(DownAction::DragTitle)
    );
    assert_eq!(
        down_action(true, true, Some(r), &v, (r.x, r.y), false),
        Some(DownAction::DragTitle)
    );
}

/// **O CORPO ENGOLE O QUE CAI EM NADA.**
///
/// ⚠️ É a metade que impede o ponteiro de atravessar para a arte por baixo: sem ela, arrastar
/// dentro do card MOVERIA as formas — o oposto exato do gesto.
#[test]
fn o_corpo_engole_o_que_cai_em_nada() {
    let v = view();
    let r = card(&v);
    // O canto do plano: longe dos círculos do anel e de qualquer traço.
    let vazio = (r.x + 4.0, r.y + r.h - 4.0);
    assert_eq!(
        down_action(false, true, Some(r), &v, vazio, false),
        Some(DownAction::Swallow)
    );
}

/// **FORA DO CARD, O PONTEIRO SEGUE.** Um `None` é o que deixa a arte continuar a receber cliques.
#[test]
fn fora_do_card_o_ponteiro_segue() {
    let v = view();
    let r = card(&v);
    assert_eq!(
        down_action(false, false, Some(r), &v, (0.0, 0.0), false),
        None
    );
    // E sem rect desenhado (card fechado), um corpo "acertado" também não decide nada.
    assert_eq!(down_action(false, true, None, &v, (0.0, 0.0), false), None);
}
