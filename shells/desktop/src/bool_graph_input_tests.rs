//! Gates da DECISÃO do gesto do diagrama.
//!
//! ⚠️ Eles constroem os pontos de teste a partir das MESMAS funções que o card desenha
//! (`bool_graph_node_center` / `_link_points`) — nunca de coordenadas escritas à mão. Um número a
//! mão passaria a mentir no dia em que um espaçamento mudasse, e o gate diria que está tudo bem
//! enquanto o artista clica ao lado do que vê.

use super::{DownAction, down_action, up_intent};
use ph2d_editor::widget::{
    BoolGraphIntent, BoolGraphLink, BoolGraphNode, BoolGraphView, bool_graph_card_size,
    bool_graph_link_points, bool_graph_node_center, bool_graph_node_radius,
};
use ph2d_editor::zones::Rect;

fn node(id: u64) -> BoolGraphNode {
    BoolGraphNode {
        id,
        label: format!("F{id}"),
        consumed: false,
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

/// **UM CÍRCULO ARMA UMA LIGAÇÃO, E ELA É A FORMA QUE SE VÊ.**
#[test]
fn um_down_num_circulo_arma_a_ligacao_daquela_forma() {
    let v = view();
    let r = card(&v);
    for i in 0..v.nodes.len() {
        let p = bool_graph_node_center(r, &v, i);
        assert_eq!(
            down_action(false, true, Some(r), &v, p, false),
            Some(DownAction::ArmLink(v.nodes[i].id)),
            "círculo {i}"
        );
    }
}

/// **UM CLIQUE NUMA LIGAÇÃO GIRA A OPERAÇÃO** — e a ligação continua a mesma.
#[test]
fn um_clique_numa_ligacao_gira_a_operacao() {
    let v = view();
    let r = card(&v);
    let pts = bool_graph_link_points(r, &v, v.links[0]);
    let meio = pts[pts.len() / 2];
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
    let meio = pts[pts.len() / 2];
    assert_eq!(
        down_action(false, true, Some(r), &v, meio, true),
        Some(DownAction::Intent(BoolGraphIntent::Unlink {
            from: 2,
            to: 1
        }))
    );
    // E a rotação, das quatro voltas, nunca produz um corte.
    let mut op = 0u8;
    for _ in 0..8 {
        let mut w = v.clone();
        w.links[0].op = op;
        let pts = bool_graph_link_points(r, &w, w.links[0]);
        let p = pts[pts.len() / 2];
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
    // Mesmo se o índice dissesse os dois, a banda ganha (ela é registada por cima).
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
    // Um ponto no card, longe de círculos e de arcos: a coluna de rótulos da forma 3 — a única
    // linha que nenhuma ligação toca. ⚠️ A do MEIO não serve: o arco 2→1 PARTE do centro dela, e um
    // ponto ao lado do círculo ainda está ao alcance do traço (foi o que este gate apanhou primeiro
    // — e ele estava certo).
    let c = bool_graph_node_center(r, &v, 2);
    let vazio = (c.0 + bool_graph_node_radius() * 4.0, c.1);
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

/// **SOLTAR NOUTRO CÍRCULO LIGA OS DOIS, NA DIREÇÃO DO GESTO.**
#[test]
fn soltar_noutro_circulo_liga_na_direcao_do_gesto() {
    let v = view();
    let r = card(&v);
    let alvo = bool_graph_node_center(r, &v, 2); // a forma 3
    assert_eq!(
        up_intent(Some(r), &v, 1, alvo),
        Some(BoolGraphIntent::Link {
            from: 1,
            to: 3,
            op: 0
        })
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
    let alvo = bool_graph_node_center(r, &v, 2);
    assert_eq!(
        up_intent(Some(r), &v, 1, alvo),
        Some(BoolGraphIntent::Link {
            from: 1,
            to: 3,
            op: 2
        })
    );
}

/// **SOLTAR EM NADA NÃO LIGA**, e soltar no MESMO círculo também não.
#[test]
fn soltar_em_nada_ou_em_si_mesmo_nao_liga() {
    let v = view();
    let r = card(&v);
    assert_eq!(
        up_intent(Some(r), &v, 1, (r.x + r.w - 2.0, r.y + 2.0)),
        None
    );
    let mesmo = bool_graph_node_center(r, &v, 0); // a forma 1
    assert_eq!(up_intent(Some(r), &v, 1, mesmo), None, "ligou a si mesma");
}
