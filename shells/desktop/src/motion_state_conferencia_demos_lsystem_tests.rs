//! **A cena `=108` MOSTRA o que diz que mostra** — a metade executável do «se a linha não
//! aparecer, PARE».
//!
//! ⚠️ O anúncio dela promete quatro leituras ao Enio (*"a 2 e a 3 têm de sair diferentes"*,
//! *"a 4 verga"*, …). Uma promessa impressa no terminal é uma frase; estes gates são o que a
//! torna uma afirmação — e o que a mantém verdadeira no dia em que alguém mexer no nó.

use super::*;
use ph2d_nodegraph::attr::Column;
use ph2d_nodegraph::cook::Cook;
use ph2d_nodegraph::value::CookValue;

/// Coze o documento da cena e devolve a nuvem de posições de cada planta.
fn plants() -> Vec<Vec<[f32; 2]>> {
    let mut reg = NodeRegistry::new();
    ph2d_node_registry_init::register_all_nodes(&mut reg).expect("os nos registam");
    let mut doc = MotionDoc::default();
    let sinks = build_lsystem_demo_document(&mut doc, &reg).expect("a cena monta");
    assert_eq!(sinks.len(), PLANTS.len(), "uma sink por planta");
    let mut cook = Cook::new();
    sinks
        .iter()
        .map(
            |s| match &cook.cook(&doc.graph, &reg, *s, 0.0).expect("coze")[0] {
                CookValue::Instances(st) => match st.get("P") {
                    Some(Column::Vec2(v)) => v.clone(),
                    _ => panic!("a planta tem de trazer P"),
                },
                other => panic!("esperava instancias, veio {other:?}"),
            },
        )
        .collect()
}

/// **Toda planta desenha alguma coisa** — o controlo mais barato e o que apanha uma cena que
/// monta um grafo e cozinha o vazio.
#[test]
fn every_plant_in_the_scene_actually_grows() {
    for (k, p) in plants().iter().enumerate() {
        assert!(
            p.len() > 20,
            "a planta {} saiu com {} elementos",
            PLANTS[k].0,
            p.len()
        );
    }
}

/// ⭐ **A 2 e a 3 são a MESMA gramática com sementes diferentes, e TÊM de divergir.**
///
/// É a leitura nº 2 do anúncio. Sem ela, uma estocástica partida (um sorteio que ignora a
/// semente) desenharia duas plantas gémeas e a cena continuaria bonita.
#[test]
fn the_two_stochastic_plants_are_not_twins() {
    let p = plants();
    assert_eq!(
        PLANTS[1].2, PLANTS[2].2,
        "as duas tem de partilhar a gramatica, senao a diferenca nao e' da semente"
    );
    assert_ne!(PLANTS[1].4, PLANTS[2].4, "e tem de diferir na semente");
    let same = p[1].len() == p[2].len() && p[1].iter().zip(&p[2]).all(|(a, b)| a == b);
    assert!(!same, "as duas plantas estocasticas sairam gemeas");
}

/// ⭐ **A 4 é a 1 com gravidade, e ela VERGA.**
///
/// A régua é o `x` mais à direita: com o tropismo a puxar para baixo, as pontas caem e a
/// envergadura muda. As duas partilham tudo menos o tropismo — que é o que faz disto uma
/// afirmação sobre o tropismo.
#[test]
fn the_gravity_plant_hangs_lower_than_the_one_it_copies() {
    let p = plants();
    assert_eq!(PLANTS[0].2, PLANTS[3].2, "a 4 e' a 1, com um numero mudado");
    assert_eq!(PLANTS[0].5, 0.0);
    // ⚠️ **POSITIVO puxa PARA a direcção declarada** (que aqui aponta para baixo). Esta linha
    // dizia `< 0.0` e a cena tinha o sinal trocado — os dois erros concordavam, e a planta
    // «com gravidade» saía mais direita do que a sem. *Duas coisas erradas da mesma maneira
    // não se acusam uma à outra: quem as separou foi a régua GEOMÉTRICA abaixo.*
    assert!(
        PLANTS[3].5 > 0.0,
        "a 4 tem de puxar PARA a direccao do tropismo"
    );
    assert_eq!(PLANTS[3].6, PLANTS[0].6, "e o angulo tem de ser o mesmo");
    // ⚠️ **A régua é o CENTROIDE, e a 1.ª foi o ponto mais alto — que não mede vergar.**
    // Numa árvore simétrica o topo é alcançado pelo caminho mais VERTICAL, e é exactamente
    // aí que o tropismo é mais fraco: a lei do ABOP é `α = e·(H × T)`, e o produto vectorial
    // ANULA-SE quando o ramo já aponta ao longo da gravidade. O topo mexeu-se `0,005` e o
    // gate acusou o produto de estar partido. *Vergar é a massa descer, não a ponta.*
    let mid = |v: &Vec<[f32; 2]>| v.iter().map(|q| q[1]).sum::<f32>() / v.len() as f32;
    let span = |v: &Vec<[f32; 2]>| {
        v.iter().map(|q| q[1]).fold(f32::MIN, f32::max)
            - v.iter().map(|q| q[1]).fold(f32::MAX, f32::min)
    };
    let (a, b) = (mid(&p[0]), mid(&p[3]));
    // ⚠️ **A barra é uma FRACÇÃO da altura da planta, não um número solto.** A cena é um
    // demo: o que ela tem de provar é que a diferença se VÊ, e «vê-se» mede-se contra o
    // tamanho da coisa. Um limiar absoluto seria re-precificado por qualquer mexida no `Step`.
    let bar = span(&p[0]) * 0.1;
    assert!(
        b < a - bar,
        "a massa da planta com gravidade tem de descer >{bar:.3}: {a} contra {b}"
    );
}

/// **A espessura chega ao canvas** — o `!` da gramática, visto na coluna `size`.
///
/// Sem isto, um `size` constante desenharia a árvore como uma nuvem de pontos iguais: a
/// leitura *"o tronco e' grosso e as pontas sao finas"* do anúncio ficaria falsa e nada
/// vermelho o diria.
#[test]
fn the_trunk_is_thicker_than_the_twigs_all_the_way_to_the_sink() {
    let mut reg = NodeRegistry::new();
    ph2d_node_registry_init::register_all_nodes(&mut reg).expect("os nos registam");
    let mut doc = MotionDoc::default();
    let sinks = build_lsystem_demo_document(&mut doc, &reg).expect("a cena monta");
    let mut cook = Cook::new();
    let CookValue::Instances(st) = &cook.cook(&doc.graph, &reg, sinks[0], 0.0).expect("coze")[0]
    else {
        panic!("instancias")
    };
    let Some(Column::Vec2(size)) = st.get("size") else {
        panic!("a planta tem de trazer size")
    };
    let (min, max) = size.iter().fold((f32::MAX, f32::MIN), |(lo, hi), s| {
        (lo.min(s[0]), hi.max(s[0]))
    });
    assert!(
        max > min * 4.0,
        "a espessura tem de variar da raiz a' ponta: {min} a {max}"
    );
}
/// ⭐⭐ **A QUINTA PLANTA ANDA COM O RELÓGIO** — a leitura mais importante da cena, e a que
/// ninguém estava a afirmar.
///
/// ⚠️ **Este gate nasceu de um report de «não há movimento»** (Enio, 2026-08-28). O report era
/// sobre OUTRA cena, mas a pergunta ficou de pé: *o que é que aqui prova que a planta 5 cresce?*
/// Nada. A cena imprimia a promessa no terminal e o resto era confiança.
///
/// A régua é a CONTAGEM de elementos: com o `Generations` ligado a um relógio, a planta ganha e
/// perde gerações inteiras ao longo do ciclo, e a contagem tem de variar. E o CONTROLE são as
/// outras quatro, que têm de ficar **exactamente** paradas — senão o que se estaria a medir era
/// o cook a ser não-determinista.
#[test]
fn only_the_fifth_plant_moves_with_the_clock() {
    use ph2d_nodegraph::cook::Cook;
    use ph2d_nodegraph::value::CookValue;
    let mut reg = NodeRegistry::new();
    ph2d_node_registry_init::register_all_nodes(&mut reg).expect("os nos registam");
    let mut doc = MotionDoc::default();
    let sinks = build_lsystem_demo_document(&mut doc, &reg).expect("a cena monta");
    let mut cook = Cook::new();
    let counts_at = |cook: &mut Cook, t: f64| -> Vec<usize> {
        sinks
            .iter()
            .map(
                |s| match &cook.cook(&doc.graph, &reg, *s, t).expect("coze")[0] {
                    CookValue::Instances(st) => st.count(),
                    other => panic!("esperava instancias, veio {other:?}"),
                },
            )
            .collect()
    };
    let base = counts_at(&mut cook, 0.0);
    let mut moved = vec![false; sinks.len()];
    for k in 1..=8 {
        let now = counts_at(&mut cook, f64::from(k) * 0.35);
        for (i, (a, b)) in base.iter().zip(&now).enumerate() {
            if a != b {
                moved[i] = true;
            }
        }
    }
    let last = sinks.len() - 1;
    assert!(
        moved[last],
        "a planta 5 tem de CRESCER com o relogio — o `Generations` dela esta' ligado a um LFO"
    );
    for (i, m) in moved.iter().enumerate().take(last) {
        assert!(!m, "a planta {} devia estar parada e mexeu-se", i + 1);
    }
}
