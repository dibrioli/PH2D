//! **OS TRÊS TETOS DE INSTÂNCIA SÃO UM SÓ NÚMERO** (CLAUDE.md §0, doc 89 folha 07).
//!
//! `motion.trail`, `fx.drop_shadow` e `fx.rgb_split` limitam a MESMA grandeza — quantas linhas
//! um nó pode emitir no caminho de CPU — e por isso carregam o mesmo teto. Eles são
//! **drop-crates** e não podem depender uns dos outros (ADR-0075), então a const é copiada em
//! cada um, exactamente como o `falloff_at` das behaviours.
//!
//! ⚠️ **Uma const copiada em três sítios é três respostas esperando divergir**, e a única coisa
//! que a mantém honesta é este gate: esta crate é a que vê os três. Quem medir de novo e mover
//! um deles move os três, ou fica vermelho aqui.
//!
//! ⚠️ **A medição que decidiu o número vive na sonda irmã** (`measure_instance_ceiling.rs`) e a
//! tabela está no doc-comment de cada const. Este gate não tem opinião sobre QUAL é o número —
//! ele afirma que há **um** número.

/// O teto MEDIDO: a linha emitida custa ~10–28 ns no caminho de CPU destes três nós, e este é
/// o ponto em que **um** nó passa a ocupar cerca de um terço de um quadro de 60 fps.
///
/// ⚠️ Este literal é o quarto sítio, e é de propósito: sem ele o gate compararia as três consts
/// **umas com as outras** e ficaria verde no dia em que alguém as movesse todas juntas por
/// engano — um oráculo que usa a coisa sob teste para computar o que espera é sempre verde.
const MEASURED_CEILING: usize = 262_144;

#[test]
fn the_three_instance_ceilings_agree() {
    let ceilings = [
        ("motion.trail", ph2d_node_motion_trail::MAX_INSTANCES),
        ("fx.drop_shadow", ph2d_node_fx_drop_shadow::MAX_INSTANCES),
        ("fx.rgb_split", ph2d_node_fx_rgb_split::MAX_INSTANCES),
    ];
    for (name, c) in ceilings {
        assert_eq!(
            c, MEASURED_CEILING,
            "{name} carrega um teto de instancias que ninguem mediu junto com os outros dois: \
             {c} contra {MEASURED_CEILING}. Os tres limitam a MESMA grandeza (linhas emitidas \
             no caminho de CPU) — mover um exige medir e mover os tres, com a tabela ao lado."
        );
    }
}

/// **E o teto tem de estar ACIMA do caso que a nota antiga citava — medido no PRODUTO.**
///
/// ⚠️ A justificativa que shipava era *"4096 vivas × 32 ecos já é 131k quads"* — e sob o teto
/// de `65_536` esse pedido era **CLAMPADO em silêncio** para 16 gerações: o artista pedia 32 e
/// recebia metade, sem nada na tela a dizer porquê. Este gate pina que o caso deixou de ser
/// recusado, que é a metade da wave que o artista VÊ.
///
/// ⚠️ **Ele COZINHA o grafo em vez de comparar duas constantes** — uma comparação de consts é
/// dobrada pelo compilador (não pode falhar em tempo de execução) e, pior, seria cega ao
/// `MAX_LENGTH` do nó, que é privado e capa as gerações por outro caminho. O oráculo é o número
/// de linhas que o cook de facto emite.
#[test]
fn the_case_the_old_note_cited_now_fits() {
    use ph2d_nodegraph::cook::Cook;
    use ph2d_nodegraph::graph::{Edge, Graph};

    let mut reg = ph2d_node_registry::NodeRegistry::new();
    ph2d_node_registry_init::register_all_nodes(&mut reg).expect("every node registers");

    // 64 x 64 = 4096 vivas, o número exacto da nota antiga.
    let mut g = Graph::new();
    let seed = g.add_node("motion.grid");
    g.set_param(seed, "rows", 64.0);
    g.set_param(seed, "cols", 64.0);
    let tr = g.add_node("motion.trail");
    g.set_param(tr, "length", 32.0);
    g.connect(Edge {
        from: (seed, 0),
        to: (tr, 0),
        delayed: false,
    })
    .expect("feed");
    g.connect(Edge {
        from: (tr, 0),
        to: (tr, 1),
        delayed: true,
    })
    .expect("ring");

    let mut cook = Cook::new();
    let mut rows = 0usize;
    // A cauda leva `length` ticks a encher; 40 dá folga.
    for t in 0..40 {
        let t = f64::from(t);
        rows = cook.cook(&g, &reg, tr, t).expect("cooks")[0]
            .as_stream()
            .count();
        cook.advance_tick(&g, &reg, t).expect("advance");
    }
    assert_eq!(
        rows, 131_072,
        "4096 vivas x 32 ecos tinham de emitir 131072 linhas; {rows} significa que o teto de \
         instancias voltou a CLAMPAR em silencio o caso que a nota antiga citava"
    );
}
