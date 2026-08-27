//! Gates da cena `=107` — a preguiça do roteador (doc 89, folha 15).

use super::*;
use ph2d_node_registry::NodeRegistry;

fn scene() -> (MotionDoc, NodeRegistry, Vec<NodeId>) {
    let mut reg = NodeRegistry::new();
    ph2d_node_registry_init::register_all_nodes(&mut reg).expect("os nos registram");
    let mut doc = MotionDoc::default();
    let sinks = build_lazy_switch_demo_document(&mut doc, &reg).expect("a cena monta");
    doc.graph.validate(&reg).expect("bem-tipada");
    (doc, reg, sinks)
}

/// **A CENA ENTRA NO PLANO DE PREGUIÇA COM OS QUATRO RAMOS SALTÁVEIS.**
///
/// ⚠️ Sem isto a cena montaria, correria, e mostraria a mesma coisa nos dois modos — que é
/// exactamente o *«deu errado»* que o texto do smoke descreve, e que a olho se lê como *«o modo
/// não faz nada»*.
#[test]
fn the_scene_is_lazy_and_all_four_branches_are_skippable() {
    let (doc, reg, _) = scene();
    let plan = ph2d_node_value_switch::lazy::plan(&doc.graph, &reg);
    assert_eq!(plan.len(), 1, "um roteador, um registo no plano");
    let e = plan.values().next().expect("o registo");
    assert!(
        e.skippable[..BRANCHES].iter().all(|b| *b),
        "os quatro ramos tinham de ser saltaveis: {:?}",
        e.skippable
    );
}

/// **O `select` FICA DESLIGADO — e essa é a condição, não um esquecimento.**
///
/// Uma porta sem aresta lê o campo vazio (`0` em todo índice), que é uniforme por construção.
/// Ligar-lhe um campo por elemento faria o modo recuar para o caminho de sempre, e a cena
/// mostraria a mesma lentidão nos dois modos sobre produto correcto.
#[test]
fn the_select_port_is_deliberately_unwired() {
    let (doc, _, _) = scene();
    let sw = doc
        .graph
        .nodes()
        .iter()
        .find(|n| n.type_name == "value.switch")
        .expect("ha' um switch");
    assert!(
        doc.graph
            .input_edge(sw.id, ph2d_node_value_switch::SELECT_PORT as usize)
            .is_none(),
        "o select ficou ligado — o modo vai recuar e a cena nao mostra nada"
    );
}

/// **A SAÍDA É A MESMA NOS DOIS MODOS** — a promessa que o texto do smoke pede ao Enio para
/// verificar a olho, aqui afirmada em números.
///
/// ⚠️ Ela vale **nesta cena** porque os quatro ramos têm o mesmo comprimento; a folha regista
/// que, no caso geral, um ramo mais comprido decide a contagem e a preguiça a mudaria (gate
/// `the_output_count_is_decided_by_branches_nobody_chose`). *Uma igualdade medida numa fixtura
/// não é uma lei — e é por isso que o modo se declara em vez de ser silencioso.*
#[test]
fn the_two_modes_agree_bit_for_bit_on_this_scene() {
    use ph2d_nodegraph::attr::Column;
    use ph2d_nodegraph::cook::Cook;
    let (doc, reg, sinks) = scene();
    let read = |lazy: bool| -> Vec<[f32; 2]> {
        let mut cook = Cook::new();
        if lazy {
            cook.set_lazy_branches(ph2d_node_value_switch::lazy::plan(&doc.graph, &reg));
        }
        let v = cook.cook(&doc.graph, &reg, sinks[0], 0.25).expect("coze");
        match v[0].as_stream().get("P") {
            Some(Column::Vec2(p)) => p.clone(),
            _ => Vec::new(),
        }
    };
    let eager = read(false);
    let lazy = read(true);
    assert!(!eager.is_empty(), "a cena cozinhou VAZIO — nada a comparar");
    assert_eq!(
        eager.len(),
        lazy.len(),
        "a preguica mudou a CONTAGEM nesta cena"
    );
    assert!(
        eager
            .iter()
            .zip(&lazy)
            .all(|(a, b)| a[0].to_bits() == b[0].to_bits() && a[1].to_bits() == b[1].to_bits()),
        "a preguica mudou um valor — a imagem nao seria a mesma"
    );
}

/// **A CENA TEM DUAS SAÍDAS, E ISSO É A CONDIÇÃO — não arrumação.**
///
/// ⚠️ **Sem isto a cena volta a não demonstrar nada, e em silêncio.** O cook é GPU-residente por
/// omissão e o plano de GPU recusa um documento com mais de um sink
/// (`motion_bridge_gpu`: `motion.sinks.len() != 1`); com um sink só, este grafo é inteiramente
/// coberto, corre no device — onde o grafo é UM dispatch e não há ramo para saltar — e o botão
/// fica inerte. Medido no quadro real: `motion_active=true` mas `pump.instances = 0`, e o modo
/// não muda um milissegundo.
#[test]
fn the_scene_keeps_two_sinks_so_it_cooks_on_the_cpu() {
    let (_, _, sinks) = scene();
    assert_eq!(
        sinks.len(),
        2,
        "com um sink so' o plano de GPU cobre a cena e o modo fica INERTE"
    );
}
