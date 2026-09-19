//! Os portões da [`super`] — a LEI (*uma corrente que não veio de uma forma não vira pixel*) e o
//! AVISO que a torna legível ao artista (ordem do dono, 2026-09-19).

use crate::motion_state::MotionState;
use ph2d_nodegraph::port::Domain;

/// Os nós que o registo marcou como fontes de posições, por nome e ordenados.
fn marcados(m: &MotionState) -> Vec<&'static str> {
    let mut v: Vec<_> = m
        .registry
        .manifests()
        .filter(|man| m.registry.so_posicoes(man.id))
        .map(|man| man.name)
        .collect();
    v.sort_unstable();
    v
}

/// ⭐⭐⭐ **O AVISO NOMEIA EXACTAMENTE QUEM SÓ ENTREGA POSIÇÕES** — a população é DERIVADA do
/// manifesto, e este gate volta a derivá-la aqui, à mão, contra a que o registo guarda.
///
/// ⚠️ **A régua não pode ser a mesma função**, senão ela concorda consigo mesma: o gate escreve
/// o predicado outra vez (*emite instâncias · não recebe instâncias*) e compara os dois
/// conjuntos. É a lei que a casa cobra de todo censo derivado — *uma lista escrita à mão ficaria
/// muda no primeiro nó novo*, e é por isso que ninguém escreve a lista.
///
/// **Mutação que deve sangrar:** inverter o `!` do `inputs` na
/// [`ph2d_node_registry::NodeRegistry::marca_as_fontes_de_posicoes`] — o conjunto passa a ser o
/// dos nós de PASSAGEM, e o aviso aparece em todo `motion.move` do grafo.
#[test]
fn o_aviso_nomeia_exactamente_as_fontes_de_posicoes() {
    use ph2d_nodegraph::port::Dim;
    let m = MotionState::new();
    let recebe = |p: &ph2d_nodegraph::node::PortSpec| p.ty.domain == Domain::Instances;
    let emite_posicoes = |p: &ph2d_nodegraph::node::PortSpec| {
        p.ty.domain == Domain::Instances && p.ty.dim == Dim::Vec2
    };
    let mut esperado: Vec<&'static str> = m
        .registry
        .manifests()
        .filter(|man| {
            man.outputs.iter().any(emite_posicoes)
                && !man.inputs.iter().any(recebe)
                && !m.registry.is_object_source(man.id)
                && !m.registry.is_live_vector_source(man.id)
        })
        .map(|man| man.name)
        .collect();
    esperado.sort_unstable();
    assert_eq!(
        marcados(&m),
        esperado,
        "o registo tem de marcar EXACTAMENTE quem emite instancias e nao recebe nenhuma"
    );
}

/// ⭐⭐ **O PISO DE POPULAÇÃO, e ele é metade do gate acima.**
///
/// ⚠️ Sem ele, o irmão fica **trivialmente verde** no dia em que a marcação varrer zero nós —
/// dois conjuntos vazios são iguais. É o modo de falha MUDO que o CLAUDE.md §5.0 nomeia para os
/// censos que passam a varrer nada, e a cura é sempre a mesma: um piso dentro do próprio gate.
///
/// O número sai da medição: a regra das TRÊS cláusulas marca **9** nós (das `14` que a sonda
/// `quem_e_como_o_grid` conta com a regra larga, saem `value.number` e `debug.const` por serem
/// escalares e as três origens de aparência por serem a cura).
/// ⛔ **Não é um teto:** uma fonte nova entra sozinha, e o gate não tem porque a barrar.
#[test]
fn a_populacao_do_aviso_nao_pode_esvaziar_se() {
    const PISO: usize = 6;
    let m = MotionState::new();
    let n = marcados(&m).len();
    assert!(
        n >= PISO,
        "so {n} fontes de posicoes marcadas (piso {PISO}) -- a derivacao varreu quase nada, e \
         dois conjuntos vazios sao iguais: o gate irmao ficaria verde a afirmar nada"
    );
}

/// ⭐⭐⭐ **OS DOIS LADOS NOMEADOS** — o gate que fala a língua do report do dono.
///
/// ⚠️ **A metade NEGATIVA vale metade:** o `motion.duplicator` é a CURA que a frase nomeia, e o
/// `motion.move` é passagem — se algum deles ganhasse o aviso, a frase passaria a acender no nó
/// que a resolve e em todo transformador do grafo. *Um aviso que soa sempre é ruído.*
#[test]
fn a_grelha_ganha_o_aviso_e_a_passagem_nao() {
    let m = MotionState::new();
    let marcou = |t: &str| {
        m.registry
            .manifests()
            .find(|man| man.name == t)
            .map(|man| m.registry.so_posicoes(man.id))
    };
    assert_eq!(
        marcou("motion.grid"),
        Some(true),
        "a `motion.grid` e' a fonte do proprio report do dono"
    );
    for passagem in ["motion.move", "motion.scale", "motion.duplicator"] {
        assert_eq!(
            marcou(passagem),
            Some(false),
            "`{passagem}` recebe uma corrente de instancias -- ela nao e' uma fonte de posicoes"
        );
    }
    // ⛔⛔ **A cláusula que a MEDIÇÃO comprou.** Estes três emitem posições e não recebem
    // instâncias — passam nas duas primeiras cláusulas — e são **exactamente o que o aviso manda
    // ir buscar**. Marcá-los poria a frase *«precisa de um Duplicator + uma forma»* no cartão do
    // nó que É a forma, e ela viraria mentira no clique seguinte (no valor de fábrica eles ainda
    // não trazem aparência, o que é o que torna este erro tão fácil de não ver).
    for origem in ["source.object", "source.shape", "source.text"] {
        assert_eq!(
            marcou(origem),
            Some(false),
            "`{origem}` E' uma origem de aparencia -- o aviso no cartao dele seria uma mentira"
        );
    }
    // ⛔ E os NÚMEROS por elemento: eles emitem `Instances` mas em `Scalar`, e um aviso sobre
    // duplicadores num nó de valor não quer dizer nada.
    for valor in ["value.number", "debug.const"] {
        assert_eq!(
            marcou(valor),
            Some(false),
            "`{valor}` emite numeros por elemento, nao posicoes"
        );
    }
}

/// ⭐⭐⭐ **A BANDEIRA CHEGA AO CARTÃO** — a pergunta que o CLAUDE.md §5.0 diz que nenhum
/// instrumento desta casa faz: *o valor chega ao consumidor?*
///
/// Os gates acima medem o REGISTO e os do painel medem o CARTÃO — e entre os dois está o
/// `snapshot_from`, que é onde a bandeira é copiada. ⚠️ **Sem esta metade, cravar
/// `so_posicoes: false` no construtor da vista deixa as duas pontas verdes e o aviso nunca
/// aparece** — é a rotura que a `line/components` pagou duas vezes (os 24 gates que entravam
/// todos pelo canal interno, abaixo do corte).
///
/// **Mutação que deve sangrar:** `so_posicoes: false` no `snapshot_build.rs`.
#[cfg(feature = "panel-motion-graph")]
#[test]
fn a_bandeira_chega_ao_cartao() {
    use ph2d_nodegraph::graph::Graph;
    let m = MotionState::new();
    let mut g = Graph::new();
    let grelha = g.add_node("motion.grid");
    let mover = g.add_node("motion.move");
    let vista = ph2d_panel_motion_graph::snapshot_from(&g, &m.registry);
    let card = |n: ph2d_nodegraph::graph::NodeId| {
        vista
            .nodes
            .iter()
            .find(|c| c.id == n.0)
            .map(|c| c.so_posicoes)
    };
    assert_eq!(
        card(grelha),
        Some(true),
        "o cartao da `motion.grid` tem de trazer a bandeira do registo -- senao o aviso nunca \
         e' pintado, com os gates das duas pontas verdes"
    );
    assert_eq!(
        card(mover),
        Some(false),
        "e o de um no' de passagem NAO a traz -- sem esta metade, cravar `true` passava"
    );
}

/// ⭐⭐⭐ **SONDA: a bandeira estática concorda com a LEI, cozinhando?** — para cada candidato,
/// coze-se o nó sozinho e pergunta-se `tem_aparencia` à corrente que ele entrega.
///
/// `cargo test -p ph2d-app-motion --lib -- --ignored --nocapture a_bandeira_contra_a_lei`
#[test]
#[ignore = "sonda, nao um gate"]
fn a_bandeira_contra_a_lei() {
    use ph2d_nodegraph::graph::Graph;
    let mut m = MotionState::new();
    let inst = |p: &ph2d_nodegraph::node::PortSpec| p.ty.domain == Domain::Instances;
    let candidatos: Vec<(&'static str, bool)> = m
        .registry
        .manifests()
        .filter(|man| man.outputs.iter().any(inst) && !man.inputs.iter().any(inst))
        .map(|man| (man.name, m.registry.so_posicoes(man.id)))
        .collect();
    eprintln!("\n=== A BANDEIRA CONTRA A LEI ===\n");
    eprintln!("  no                          │ dim  │ marcado │ tem_aparencia");
    for (nome, marcado) in candidatos {
        let dim = m
            .registry
            .manifests()
            .find(|man| man.name == nome)
            .and_then(|man| man.outputs.first().map(|p| format!("{:?}", p.ty.dim)))
            .unwrap_or_default();
        let mut g = Graph::new();
        let n = g.add_node(nome);
        let saida = g.add_node("motion.output");
        let _ = g.connect(ph2d_nodegraph::graph::Edge {
            from: (n, 0),
            to: (saida, 0),
            delayed: false,
        });
        m.doc.graph = g;
        let aparencia = m
            .pump
            .cook
            .cook(&m.doc.graph, &m.registry, saida, 0.0)
            .ok()
            .map(|out| ph2d_eval_motion::tem_aparencia(out[0].as_stream()));
        eprintln!(
            "  {nome:<27} │ {dim:<4} │ {:<7} │ {}",
            if marcado { "SIM" } else { " - " },
            match aparencia {
                Some(true) => "SIM  ⛔ o aviso MENTIRIA",
                Some(false) => " -   ✅",
                None => "(nao coze sozinho)",
            }
        );
    }
    eprintln!();
}
