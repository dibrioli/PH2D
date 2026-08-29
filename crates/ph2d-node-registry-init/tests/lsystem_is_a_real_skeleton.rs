//! **O L-SYSTEM EMITE UM ESQUELETO A SÉRIO** — e a prova é o `rig.fk` não lhe tocar.
//!
//! # Por que este gate vive aqui e não no crate do nó
//!
//! O `source.lsystem` afirma, no doc dele, que as colunas que emite são *"exactamente o
//! contrato do `rig.*`"* e que a posição é calculada *"como o `rig.fk` a calcula"*. As duas
//! frases são afirmações sobre **outro módulo**, e afirmações medem-se. O crate do nó não
//! pode medi-las: os dois são drop-crates e não se conhecem (ADR-0075). Esta crate é a que vê
//! os dois.
//!
//! # A régua é a IDENTIDADE, e é a mais forte que existe aqui
//!
//! `rig.fk` recalcula `P` e `wrot` a partir de `(parent, len, rot)`. Se ele mexer numa única
//! posição, uma de três coisas é falsa: ou a árvore que o L-System emite não é uma árvore
//! bem-formada, ou o `len` que ele declara não é a distância que ele desenhou, ou as duas
//! contas de passo divergiram. ⭐ **Ao BIT** e não por tolerância — as duas fazem a mesma
//! sequência de operações sobre os mesmos `f32`, e uma tolerância esconderia exactamente a
//! divergência que este gate existe para apanhar.
//!
//! ⚠️ E o CONTROLO é uma nuvem de pontos: sobre ela o `rig.fk` também é a identidade (é a
//! regra de identidade dele), então um gate só com o lado positivo ficaria verde com um
//! L-System que emitisse pontos soltos — que é precisamente o desenho que esta wave recusou.

use ph2d_nodegraph::attr::{Column, Stream};
use ph2d_nodegraph::cook::Cook;
use ph2d_nodegraph::graph::{Edge, Graph};
use ph2d_nodegraph::value::CookValue;

fn registry() -> ph2d_node_registry::NodeRegistry {
    let mut reg = ph2d_node_registry::NodeRegistry::new();
    ph2d_node_registry_init::register_all_nodes(&mut reg).expect("every node registers");
    reg
}

fn positions(s: &Stream) -> Vec<[f32; 2]> {
    match s.get("P") {
        Some(Column::Vec2(v)) => v.clone(),
        _ => panic!("o stream tem de trazer P"),
    }
}

fn stream_of(out: &[CookValue]) -> Stream {
    match &out[0] {
        CookValue::Instances(s) => s.clone(),
        other => panic!("esperava um stream de instancias, veio {other:?}"),
    }
}

/// Cozinha `source.lsystem` sozinho e depois com um `rig.fk` a jusante, com a gramática dada.
fn both_sides(axiom: &str, rules: &str, generations: f32) -> (Stream, Stream) {
    let reg = registry();
    let mut g = Graph::new();
    let l = g.add_node("source.lsystem");
    // ⚠️⚠️ **A fixtura DECLARA que autora uma gramática, e sem isto ela mede outra coisa.**
    // Desde 2026-08-29 o default do nó é `Guided` — os sliders de forma —, e no guiado o texto
    // **não é lido**. Estas quatro gramáticas exercitam ramos, cortes e marcas de propósito;
    // sem o modo, as quatro cozinhariam a MESMA árvore derivada e três dos gates deste
    // ficheiro passariam a medir a mesma planta com quatro nomes. *Um default que muda
    // re-pergunta a toda fixtura que dependia do antigo* — e este ficheiro deu o aviso: o
    // `the_emitted_topology_is_a_tree_and_not_a_chain_or_a_cloud` caiu de `> 4` bifurcações
    // para **3**, que é o que a árvore binária guiada tem.
    g.set_param(
        l,
        ph2d_node_source_lsystem::param::MODE,
        ph2d_node_source_lsystem::MODE_GRAMMAR as f32,
    );
    g.set_text_param(l, ph2d_node_source_lsystem::AXIOM_PARAM, axiom);
    g.set_text_param(l, ph2d_node_source_lsystem::RULES_PARAM, rules);
    g.set_param(l, ph2d_node_source_lsystem::param::GENERATIONS, generations);
    // ⚠️ **`Local`, e a escolha é o assunto deste ficheiro.** Desde 2026-08-28 o nó tem um
    // param `Shape Faces` que decide o que a coluna `rot` quer dizer, e o DEFAULT é `Growth`
    // (o ângulo de MUNDO), porque é isso que o desenho quer: sem ele a forma carimbada sai
    // sempre em pé, qualquer que seja a direcção do ramo (report do Enio). O contrato do
    // `rig.*` pede o ângulo LOCAL — logo é o modo `Local` que estes gates medem, e é o gate
    // `growth_orientation_is_not_a_skeleton` que afirma a outra metade.
    g.set_param(l, ph2d_node_source_lsystem::param::ORIENT, 1.0);
    let fk = g.add_node("rig.fk");
    g.connect(Edge {
        from: (l, 0),
        to: (fk, 0),
        delayed: false,
    })
    .expect("o L-System sai num stream que o rig.fk aceita");

    let mut cook = Cook::new();
    let raw = stream_of(cook.cook(&g, &reg, l, 0.0).expect("coze"));
    let resolved = stream_of(cook.cook(&g, &reg, fk, 0.0).expect("coze"));
    (raw, resolved)
}

/// ⭐⭐ **`source.lsystem → rig.fk` não move um bit.**
///
/// Sobre quatro gramáticas que exercitam tudo o que pode partir a invariante: ramos
/// aninhados, um SALTO (`f`, que faz nascer raiz nova), espessura, passo variável, e uma
/// planta paramétrica de verdade.
#[test]
fn the_fk_pass_does_not_move_a_single_lsystem_element() {
    for (axiom, rules, g) in [
        ("F", "F -> F[+F]F[-F]F", 3.0),
        ("F", "F -> F[+F[-F]F]F", 3.0),
        ("F", "F -> F f F[+F]", 3.0),
        (
            ph2d_node_source_lsystem::DEFAULT_AXIOM,
            ph2d_node_source_lsystem::DEFAULT_RULES,
            5.0,
        ),
    ] {
        let (raw, resolved) = both_sides(axiom, rules, g);
        assert!(raw.count() > 4, "a fixtura {rules} tem de ter elementos");
        assert_eq!(raw.count(), resolved.count());
        let (a, b) = (positions(&raw), positions(&resolved));
        for (i, (p, q)) in a.iter().zip(b.iter()).enumerate() {
            assert_eq!(
                (p[0].to_bits(), p[1].to_bits()),
                (q[0].to_bits(), q[1].to_bits()),
                "{rules}: o rig.fk mexeu no elemento {i}: {p:?} -> {q:?}"
            );
        }
    }
}

/// **E o `wrot` também sobrevive** — a metade que o `P` sozinho não cobre.
///
/// O `rig.fk` reescreve as DUAS colunas derivadas. Um L-System que acertasse nas posições e
/// errasse nos ângulos de mundo passaria o gate acima e entregaria um esqueleto cujas juntas
/// apontam para outro lado — invisível até alguém lhe pendurar uma sprite.
#[test]
fn the_world_angles_survive_the_fk_pass_too() {
    let (raw, resolved) = both_sides("F", "F -> F[+F]F[-F]F", 3.0);
    let w = |s: &Stream| match s.get("wrot") {
        Some(Column::Scalar(v)) => v.clone(),
        _ => panic!("wrot"),
    };
    let (a, b) = (w(&raw), w(&resolved));
    for (i, (p, q)) in a.iter().zip(b.iter()).enumerate() {
        assert!(
            (p - q).abs() < 1e-3,
            "o rig.fk mudou o angulo de mundo da junta {i}: {p} -> {q}"
        );
    }
}

/// ⚠️ **O CONTROLO — a árvore é uma ÁRVORE, não uma corrente nem uma nuvem.**
///
/// Sem isto, um L-System que emitisse `parent[i] = i − 1` (uma corrente) ou `parent = -1`
/// para todos (uma nuvem) passaria os dois gates acima: a identidade do `rig.fk` vale para os
/// dois casos degenerados. O que só uma ÁRVORE produz é **dois elementos pendurados no mesmo
/// pai** — que é o que um ramo é.
#[test]
fn the_emitted_topology_is_a_tree_and_not_a_chain_or_a_cloud() {
    let (raw, _) = both_sides("F", "F -> F[+F]F[-F]F", 3.0);
    let parent = match raw.get("parent") {
        Some(Column::Scalar(v)) => v.clone(),
        _ => panic!("o L-System tem de emitir a coluna parent"),
    };
    let roots = parent.iter().filter(|p| **p < 0.0).count();
    assert_eq!(roots, 1, "uma planta sem saltos tem UMA raiz, deu {roots}");

    let mut counts = std::collections::BTreeMap::<i64, usize>::new();
    for p in parent.iter().filter(|p| **p >= 0.0) {
        *counts.entry(*p as i64).or_default() += 1;
    }
    let forks = counts.values().filter(|c| **c > 1).count();
    assert!(
        forks > 4,
        "uma corrente nao tem bifurcacoes e uma nuvem nao tem pais: achei {forks}"
    );
    // E nenhum pai aponta para a frente — a ordem topológica que o `rig.fk` assume.
    for (i, p) in parent.iter().enumerate() {
        assert!(
            *p < i as f32,
            "o elemento {i} pendura no {p}, que ainda nao existe"
        );
    }
}

/// ⚠️⚠️ **E o MODO DE CRESCIMENTO não é um esqueleto — a fronteira, dita em voz alta.**
///
/// No default (`Shape Faces = Growth`) a coluna `rot` carrega o ângulo de **MUNDO**, que é o
/// que o lowering desenha. O `rig.fk` lê a mesma coluna como o ângulo **LOCAL** da junta, então
/// ele re-resolve a planta noutro sítio — não por estar partido, mas porque os dois consumidores
/// perguntam coisas diferentes ao mesmo nome.
///
/// ⚠️ Este gate existe para essa troca ser **medida** em vez de descoberta: quem ligar um
/// `rig.*` a jusante e vir a planta a saltar vem aqui e encontra o param, em vez de abrir um bug.
/// *Um nome com dois donos precisa de um interruptor e de uma nota, não de uma escolha
/// escondida.*
#[test]
fn growth_orientation_is_not_a_skeleton_and_that_is_the_trade() {
    let reg = registry();
    let mut g = Graph::new();
    let l = g.add_node("source.lsystem");
    // ⚠️⚠️ **ESTE TESTE ESTAVA A MEDIR OUTRA PLANTA** — auditoria de 2026-08-29. Ele é o único
    // do ficheiro que não passa pelo `both_sides`, e por isso não declarava o modo; desde que o
    // `Mode` nasce `Guided`, os dois `set_text_param` abaixo **não eram lidos**. Medido: a
    // gramática que ele NOMEIA dá `126` elementos e `62` bifurcações, e o que ele de facto
    // media eram `8` elementos e `3` bifurcações — byte-idênticos ao guiado com texto vazio.
    // O ⚠️⚠️ do `both_sides`, dez linhas acima, descreve exactamente esta armadilha, e este
    // teste ficou de fora dele. *Um aviso escrito ao lado do sítio que ele descreve não é uma
    // defesa — só um censo é.*
    g.set_param(
        l,
        ph2d_node_source_lsystem::param::MODE,
        ph2d_node_source_lsystem::MODE_GRAMMAR as f32,
    );
    g.set_text_param(l, ph2d_node_source_lsystem::AXIOM_PARAM, "F");
    g.set_text_param(l, ph2d_node_source_lsystem::RULES_PARAM, "F -> F[+F]F[-F]F");
    g.set_param(l, ph2d_node_source_lsystem::param::GENERATIONS, 3.0);
    // O DEFAULT — nenhum override de `orient`.
    let fk = g.add_node("rig.fk");
    g.connect(Edge {
        from: (l, 0),
        to: (fk, 0),
        delayed: false,
    })
    .expect("liga");
    let mut cook = Cook::new();
    let raw = stream_of(cook.cook(&g, &reg, l, 0.0).expect("coze"));
    let resolved = stream_of(cook.cook(&g, &reg, fk, 0.0).expect("coze"));
    let (a, b) = (positions(&raw), positions(&resolved));
    let moved = a
        .iter()
        .zip(&b)
        .filter(|(p, q)| (p[0] - q[0]).abs() > 1e-4 || (p[1] - q[1]).abs() > 1e-4)
        .count();
    assert!(
        moved > a.len() / 4,
        "no modo de crescimento o rig.fk TEM de re-resolver a planta noutro sitio — se ele nao \
         mexesse, ou o default deixou de ser `Growth` ou a coluna `rot` deixou de ser o angulo \
         de mundo, e a forma carimbada voltou a sair em pe'"
    );
}

/// ⭐⭐⭐ **O CENSO que impede a terceira vez** — todo `source.lsystem` deste ficheiro que
/// escreve uma gramática TEM de declarar o modo.
///
/// ⚠️ Em 2026-08-29 o default do nó passou a ser `Guided`, e o `both_sides` ganhou um ⚠️⚠️ a
/// dizê-lo. Mesmo assim o `growth_orientation_is_not_a_skeleton_and_that_is_the_trade` ficou
/// de fora e passou a medir 8 elementos em vez de 126 — **verde, sobre outra planta**.
/// *Um aviso escrito ao lado do sítio que ele descreve protege quem já o leu; um censo
/// protege quem não leu.*
///
/// Ele lê o FONTE deste ficheiro, que é a pergunta que o texto responde exactamente: *escreve
/// um `RULES_PARAM` e não escreve o `MODE`?*
#[test]
fn every_fixture_in_this_file_that_authors_a_grammar_declares_the_mode() {
    // ⚠️ O `file!()` é relativo à RAIZ da workspace e o cwd de um teste é a pasta do crate —
    // ler `file!()` directamente dá `NotFound`. O nome sai dele (nunca escrito duas vezes), a
    // pasta sai do `CARGO_MANIFEST_DIR`.
    let name = std::path::Path::new(file!())
        .file_name()
        .expect("o ficheiro tem nome");
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join(name);
    let src = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("o ficheiro le-se a si mesmo ({}): {e}", path.display()));
    let writes_rules = src
        .matches("set_text_param(l, ph2d_node_source_lsystem::RULES_PARAM")
        .count();
    let declares_mode = src
        .matches("ph2d_node_source_lsystem::param::MODE,")
        .count();
    // ⚠️ O CONTROLE: se a varredura casar ZERO, ela responde «está tudo bem» para sempre.
    assert!(
        writes_rules >= 2,
        "a varredura so' achou {writes_rules} fixturas a escrever gramatica — ela esta' partida"
    );
    assert!(
        declares_mode >= writes_rules,
        "{writes_rules} fixturas escrevem uma gramatica e so' {declares_mode} declaram o modo \
         — a que nao declara mede a derivada dos sliders, nao o texto que escreveu"
    );
}
