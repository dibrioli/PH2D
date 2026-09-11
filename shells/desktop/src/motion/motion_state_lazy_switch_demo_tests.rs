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
/// cargo test -p ph2d-host-desktop --bins measure_lazy_switch_cost -- --ignored --nocapture
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
    use crate::render_loop::motion_bridge::gpu::{GpuRoute, gpu_route};
    let (_, _, sinks) = scene();
    assert_eq!(
        sinks.len(),
        2,
        "com um sink so' o plano de GPU cobre a cena e o modo fica INERTE"
    );
    // ⚠️ **A metade que faltava: a ROTA, não a declaração.** `sinks.len() == 2` é o *proxy* —
    // a lei mora em `gpu_route`, que é pura e chamável. Com a GPU LIGADA e sem escopos nem
    // fronteiras (o melhor caso para o device), a contagem de sinks tem de ser sozinha o
    // bastante para mandar a cena para a CPU. *Um gate sobre a premissa fica verde no dia em
    // que a conclusão mudar de dono.*
    assert_eq!(
        gpu_route(true, sinks.len(), true, &[], 0),
        GpuRoute::Cpu,
        "a cena iria para o device — o botao da preguica ficaria inerte"
    );
    // E o controle: com UM sink a mesma chamada escolhe o device. Sem ele, um `gpu_route` que
    // devolvesse `Cpu` sempre passaria neste gate.
    assert_eq!(
        gpu_route(true, 1, true, &[], 0),
        GpuRoute::FullyGpu,
        "controle: com um sink so' a rota TEM de ser a do device"
    );
}

/// ⛔⛔ **A SEGUNDA SAÍDA ESCOLHE A ROTA; ELA NÃO PODE DESENHAR O CAMPO.**
///
/// O pump **acumula** os sinks num `Vec` só e desenha por ordem, então uma 2.ª saída ligada ao
/// mesmo fluxo lowerava `SIDE²` instâncias **em repouso, por cima da onda** — medido pela
/// auditoria de 2026-08-27: a laje parada cobria `3,78` dos `4,59` da onda, restando `17%` da
/// altura a ondular, sobre um campo opaco. A cena escondia o que pedia para se julgar.
///
/// ⚠️ **A régua é a CONTAGEM de instâncias, não a caixa.** Uma caixa menor ainda pode tapar o
/// meio; o que torna a 2.ª saída inofensiva é ela não ter praticamente nada para desenhar.
#[test]
fn the_second_sink_never_draws_over_the_field_the_artist_is_asked_to_judge() {
    use ph2d_nodegraph::cook::Cook;
    let (doc, reg, sinks) = scene();
    let count = |sink: NodeId| -> usize {
        let mut cook = Cook::new();
        cook.cook(&doc.graph, &reg, sink, 0.25).expect("coze")[0]
            .as_stream()
            .count()
    };
    let field = count(sinks[0]);
    let anchor = count(sinks[1]);
    #[expect(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "224"
    )]
    let expected = (SIDE as usize) * (SIDE as usize);
    assert_eq!(field, expected, "a saida principal tem de ser o campo todo");
    assert!(
        anchor <= 1,
        "a segunda saida lowera {anchor} instancias — ela desenha POR CIMA do campo que o \
         smoke pede ao Enio para julgar (o pump acumula os sinks e pinta por ordem)"
    );
}
