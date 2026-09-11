//! **A prova de que o ESPAÇO DO ELEMENTO era inexprimível** (doc 89, folha 06, célula 41).
//!
//! O `motion.drive` ganhou `Space = World | Element` porque nenhuma cadeia de nós conseguia
//! transformar a coluna `rot` numa direcção — e isso não é um argumento, é uma **enumeração
//! do catálogo**. Ela mora aqui e não na crate do nó porque o ADR-0075 proíbe uma crate-nó de
//! depender de outra: esta é a única casa de onde o catálogo inteiro é visível.
//!
//! ⚠️ **Se um dia alguém acrescentar `Sin` ao `value.math`, este gate cai** — e a nota de que
//! a célula era capacidade (e não ergonomia) tem de ser reconferida nesse mesmo commit.

use ph2d_node_registry::{NodeRegistry, ParamWidget};
use ph2d_nodegraph::node::NodeTypeId;

/// As palavras que denunciam uma operação capaz de virar um ângulo em direcção.
const TRIG: [&str; 6] = ["sin", "cos", "tan", "atan", "angle", "polar"];

#[test]
fn no_value_op_can_turn_an_angle_into_a_direction() {
    let mut reg = NodeRegistry::new();
    ph2d_node_registry_init::register_all_nodes(&mut reg).expect("every node registers");
    let hints = reg
        .param_ui(NodeTypeId::of("value.math"))
        .expect("hints do value.math");
    let row = hints
        .iter()
        .find(|h| h.param == "op")
        .expect("a linha do `op`");
    let ParamWidget::Enum { labels } = row.widget else {
        panic!("o `op` e' um seletor nomeado")
    };
    // O CONTROLE: a varredura achou de facto o catálogo. Uma lista vazia passaria por vácuo,
    // que é como um gate deste feitio mente.
    assert!(
        labels.len() >= 15,
        "o catalogo do `value.math` tem {} ops -- a varredura quebrou, nao o catalogo",
        labels.len()
    );
    for l in labels {
        let low = l.to_ascii_lowercase();
        for t in TRIG {
            assert!(
                !low.contains(t),
                "`{l}` e' trigonometrica: o espaco do elemento do `motion.drive` deixou de ser \
                 inexprimivel por composicao, e a celula 41 da folha 06 precisa de ser reconferida"
            );
        }
    }
}
