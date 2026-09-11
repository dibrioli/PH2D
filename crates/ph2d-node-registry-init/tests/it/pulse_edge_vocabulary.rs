//! **A FAMÍLIA `pulse.*` FALA UMA LÍNGUA SÓ PARA A BORDA** (doc 89, folha 12, linha 46).
//!
//! A conferência anotou a assimetria como um fato sobre a UI: *"`pulse.compare` e
//! `pulse.threshold` **têm** o param `edge` (Rise/Fall/Both) e este não"*. Fechada a
//! omissão no `pulse.on_change`, o que sobra é o risco de ela voltar por outro lado —
//! um quarto produtor de pulso que invente *"Up/Down/Any"*, ou que numere `0 = Both`
//! porque ali o neutro é esse. O artista aprenderia **um seletor por nó**.
//!
//! ⚠️ **Este gate mora AQUI e não nas crates dos nós**, pela mesma razão do
//! `pulse_level_chains`: cada `pulse-*` é uma folha drop-in que **não alcança as
//! irmãs** — a `on-change` re-declara o próprio `ChangeDir` de propósito, porque o
//! vocabulário compartilhado é a PORTA, nunca um símbolo. Esta é a única crate que
//! enxerga as três ao mesmo tempo, então é o único lugar onde *"elas concordam"* é
//! uma afirmação que se pode fazer.
//!
//! ⚠️ **E o que se compara são os RÓTULOS, não os números.** O número é o que o
//! documento guarda; a palavra é o que o artista lê, e é a palavra que tem de ser a
//! mesma nos três. Que os números também coincidam é afirmado à parte, porque a
//! consequência de divergirem é diferente: um preset copiado entre nós passaria a
//! selecionar outra coisa, em silêncio.

use ph2d_node_registry::{NodeRegistry, ParamWidget};
use ph2d_nodegraph::node::NodeTypeId;

/// O registry REAL — o vocabulário sob teste é o que o app ship.
fn registry() -> NodeRegistry {
    let mut reg = NodeRegistry::new();
    ph2d_node_registry_init::register_all_nodes(&mut reg).expect("every node registers");
    reg
}

/// Os três produtores de pulso que escolhem uma DIREÇÃO, e o nome que cada um deu ao
/// seletor. O `pulse.on_change` chama-o `direction` porque ali não há limiar a cruzar —
/// o nome difere de propósito, o vocabulário não.
const EDGE_SELECTORS: &[(&str, &str)] = &[
    ("pulse.threshold", "edge"),
    ("pulse.compare", "edge"),
    ("pulse.on_change", "direction"),
];

fn labels(reg: &NodeRegistry, type_name: &str, param: &str) -> &'static [&'static str] {
    let id = NodeTypeId::of(type_name);
    let hints = reg
        .param_ui(id)
        .unwrap_or_else(|| panic!("{type_name} registers param hints"));
    let hint = hints
        .iter()
        .find(|h| h.param == param)
        .unwrap_or_else(|| panic!("{type_name} declares a `{param}` hint"));
    match hint.widget {
        ParamWidget::Enum { labels } => labels,
        _ => panic!("{type_name}.{param} is a named choice, not a slider"),
    }
}

/// **Os três seletores mostram as MESMAS palavras, na mesma ordem.**
///
/// O controle positivo é a própria lista: se um dia um dos três deixar de existir, o
/// `expect` acima falha alto em vez de a varredura passar vazia.
#[test]
fn the_pulse_family_speaks_one_edge_vocabulary() {
    let reg = registry();
    assert_eq!(
        EDGE_SELECTORS.len(),
        3,
        "o gate compara TRÊS nós; um quarto produtor de direção entra aqui"
    );
    let canonical = ["Rise", "Fall", "Both"];
    for (type_name, param) in EDGE_SELECTORS {
        assert_eq!(
            labels(&reg, type_name, param),
            &canonical,
            "{type_name}.{param} tem de falar a língua da família"
        );
    }
}

/// **E os NÚMEROS por trás das palavras também coincidem** — um preset copiado de um nó
/// para o outro seleciona a mesma coisa. Só o DEFAULT pode divergir, e diverge: o neutro
/// do `on_change` é `Both`, porque é o que ele fazia antes de o param existir.
#[test]
fn the_numbering_matches_even_where_the_default_does_not() {
    let reg = registry();
    let mut defaults = Vec::new();
    for (type_name, param) in EDGE_SELECTORS {
        let id = NodeTypeId::of(type_name);
        let hint = reg
            .param_ui(id)
            .expect("hints")
            .iter()
            .find(|h| h.param == *param)
            .expect("the selector");
        assert_eq!(
            (hint.min, hint.max, hint.step),
            (0.0, 2.0, 1.0),
            "{type_name}.{param}: mesma escada de índices"
        );
        let manifest = reg
            .manifests()
            .find(|m| m.id == id)
            .expect("the node is registered");
        let spec = manifest
            .params
            .iter()
            .find(|p| p.name == *param)
            .expect("the selector is declared");
        defaults.push((*type_name, spec.default));
    }
    // O default do `on_change` é o ÚNICO que difere, e a diferença é a razão de existir
    // deste comentário: `Both` é o mundo anterior ao param, e um neutro que não fosse o
    // default mudaria o desenho de todo grafo já salvo.
    let on_change = defaults
        .iter()
        .find(|(n, _)| *n == "pulse.on_change")
        .expect("listado");
    assert_eq!(on_change.1, 2.0, "o neutro do on_change é `Both`");
    for (name, default) in defaults.iter().filter(|(n, _)| *n != "pulse.on_change") {
        assert_eq!(
            *default, 0.0,
            "{name}: o default dos que cruzam limiar é Rise"
        );
    }
}
