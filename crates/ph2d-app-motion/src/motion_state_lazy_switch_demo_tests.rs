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

/// **O ANÚNCIO CITA OS NÚMEROS DA CENA** — o gate que as outras cinco cenas anunciadas já tinham
/// e esta não.
///
/// ⚠️ **Ela era a única fora dele, e por uma razão estrutural:** os dois milissegundos viajavam
/// como **literais inline** no `motion_state_demo_announce.rs`, então não havia `const` de onde
/// saíssem e o gate padrão nem era escrevível. Quando esta jornada mudou a cena (o 2.º sink
/// deixou de desenhar o campo), os números do anúncio ficaram errados e **nada** o disse.
/// *Um número que a prosa repete é um número que só envelhece em silêncio.*
#[test]
fn the_announcement_cites_the_numbers_the_scene_uses() {
    let src = include_str!("motion_state_demo_announce.rs");
    for k in [
        "lazy_switch_demo::SIDE",
        "lazy_switch_demo::COOK_ON_MS",
        "lazy_switch_demo::COOK_OFF_MS",
    ] {
        assert!(src.contains(k), "o anuncio tem de citar `{k}`");
    }
}

/// **A SONDA QUE ESCOLHE O `SIDE`** — imprime, não afirma.
///
/// ```text
/// cargo test -p ph2d-app-motion --lib measure_lazy_switch_cost -- --ignored --nocapture
/// ```
///
/// ⚠️ **Ela existe porque a tabela do doc-comment do [`SIDE`] não tinha instrumento.** A 1.ª
/// versão foi lida de quadros reais à mão, uma corrida por célula, e a auditoria de 2026-08-27
/// mostrou o preço: a coluna OFF era **super-linear de um jeito que a ON não é** (`224 → 256`
/// sobe `1,31×` em peças e `4,36×` em milissegundos) sem que nada nomeasse o recurso, e as duas
/// colunas carregavam o custo fixo do 2.º sink — que a mesma auditoria mandou embora. *Um número
/// sem instrumento não se pode reconferir, e este já mudou duas vezes.*
///
/// ⚠️ **O que se mede é o COZIMENTO, que é o que o modo muda** — não o quadro inteiro. A mediana
/// de `REPS` corridas, com uma de aquecimento fora da conta, e o **piso** (um ramo só) ao lado
/// como controle: sem ele, «ligado é rápido» não tem com que se comparar.
#[test]
#[ignore = "sonda: imprime numeros, nao afirma"]
fn measure_lazy_switch_cost() {
    use ph2d_nodegraph::cook::Cook;
    use std::time::Instant;
    const REPS: usize = 7;
    let (doc, reg, sinks) = scene();
    let plan = ph2d_node_value_switch::lazy::plan(&doc.graph, &reg);
    let once = |lazy: bool, t: f64| -> f64 {
        let mut cook = Cook::new();
        if lazy {
            cook.set_lazy_branches(plan.clone());
        }
        let start = Instant::now();
        cook.cook(&doc.graph, &reg, sinks[0], t).expect("coze");
        start.elapsed().as_secs_f64() * 1e3
    };
    let median = |mut v: Vec<f64>| -> f64 {
        v.sort_by(f64::total_cmp);
        v[v.len() / 2]
    };
    println!("\n# cena =107 · SIDE = {SIDE} · {} pecas", SIDE * SIDE);
    println!("# mediana de {REPS} cozimentos (1 de aquecimento fora), maquina calma\n");
    println!("{:<12} {:>10}", "modo", "ms/cook");
    for lazy in [true, false] {
        let _ = once(lazy, 0.0); // aquecimento
        #[expect(clippy::cast_precision_loss, reason = "0..REPS")]
        let ms = median(
            (0..REPS)
                .map(|i| once(lazy, 0.25 + i as f64 * 0.01))
                .collect(),
        );
        println!(
            "{:<12} {ms:>10.2}",
            if lazy { "LIGADO" } else { "DESLIGADO" }
        );
    }
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

/// ⭐⭐ **A CENA PEDE A CPU, e a leitura da rota diz porquê** (doc 119 W3).
///
/// ⛔ **A premissa que este gate substitui MORREU à vista:** até ao ciclo 11 a cena tinha DUAS
/// saídas, e o gate `the_scene_keeps_two_sinks_so_it_cooks_on_the_cpu` afirmava que a contagem de
/// sinks sozinha a mandava para a CPU — era a cerca do multi-sink a fazer o trabalho. O ciclo 11
/// levantou a cerca (o plano da união + o `cook_many`), e com duas saídas a cena iria à placa, onde
/// o grafo é UM dispatch e o botão da preguiça fica inerte.
///
/// ⇒ o pedido é EXPLÍCITO, e as duas metades afirmam-se: esta cena pede a CPU, e uma cena VIZINHA
/// não (senão um pedido que respondesse sempre passaria aqui e mandaria o app inteiro para a CPU).
#[test]
fn a_cena_pede_a_cpu_e_diz_porque() {
    use crate::motion_state::demo_router::cena_pede_a_cpu_em;
    assert_eq!(
        cena_pede_a_cpu_em(Some("107")),
        Some(super::PEDE_A_CPU),
        "a cena que ensina o modo da CPU tem de a pedir"
    );
    assert!(
        super::PEDE_A_CPU.starts_with("CPU:"),
        "a leitura da rota lê-se como as outras recusas"
    );
    // O CONTROLO: a vizinha e o documento de artista não pedem nada.
    assert_eq!(cena_pede_a_cpu_em(Some("106")), None);
    assert_eq!(cena_pede_a_cpu_em(None), None);
}

/// **UMA SAÍDA SÓ, e é o campo todo** — a 2.ª saída, que existia para contornar a cerca, saiu com
/// ela. ⚠️ Ela custou uma auditoria (2026-08-27): desenhada por cima, escondia `83 %` da onda.
#[test]
fn a_cena_tem_uma_saida_e_ela_e_o_campo() {
    use ph2d_nodegraph::cook::Cook;
    let (doc, reg, sinks) = scene();
    assert_eq!(
        sinks.len(),
        1,
        "a segunda saida era a cerca a fazer o trabalho"
    );
    let mut cook = Cook::new();
    let field = cook.cook(&doc.graph, &reg, sinks[0], 0.25).expect("coze")[0]
        .as_stream()
        .count();
    #[expect(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "um lado de grelha"
    )]
    let expected = (SIDE as usize) * (SIDE as usize);
    assert_eq!(field, expected, "a saida tem de ser o campo todo");
}
