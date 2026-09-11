//! **A fixture de nível de artista deste app** — a NEVE (o sistema de partículas de
//! `motion_demo_strobe`), e a porta única que a instala num [`MotionState`].
//!
//! Irmão de `motion_state` pelo cap de 600 LOC do shell (HR-18). O corte é por ASSUNTO e não
//! por tamanho: o pai guarda o que o app FAZ com um documento (cozinhar, salvar, carregar,
//! navegar), e aqui mora o único documento que o repo AUTORA — o que, desde 2026-08-07, é
//! coisa que só os gates fazem.
//!
//! ⚠️ Ela era o documento de BOOT até o Enio dizer *"tire a cena da cachoeira"*. O editor abre
//! vazio; a cena ficou porque tem consumidores que não são o boot, e cada um deles ficaria
//! VÁCUO sobre um documento vazio — ver [`MotionState::with_snow`].

use super::MotionState;
use super::strobe;
use ph2d_motion_doc::MotionDoc;
use ph2d_node_registry::NodeRegistry;
use ph2d_nodegraph::graph::NodeId;

impl MotionState {
    /// **A neve, instalada** — a PORTA ÚNICA da fixture de nível de artista deste app.
    ///
    /// Ela era o documento de boot até 2026-08-07 (Enio: *"tire a cena da cachoeira"*), e
    /// dezenas de gates foram escritos contra ela sem dizer: subgrafo, backdrop, rename,
    /// save/load, relógio, o censo de GPU. Sobre a tela vazia com que o editor abre agora,
    /// **todos eles passariam por vácuo** — não há card para agrupar, nem estado a esquecer,
    /// nem fronteira de CPU a medir. Então a premissa virou uma chamada.
    ///
    /// ⚠️ **Uma porta, não uma por arquivo:** os quatro módulos de teste da ponte não
    /// alcançam o `build_default_document` (privado do `motion_state`), e a alternativa era
    /// cada um montar o próprio grafo rico — quatro documentos diferentes fingindo ser *"o
    /// documento do artista"*, divergindo no dia em que a cena mudasse.
    pub(crate) fn with_snow() -> Self {
        let mut state = Self::new();
        state.sinks = build_default_document(&mut state.doc, &state.registry)
            .expect("the snow is well-typed");
        state
    }
}

/// Author the **snow** into `doc` — a whole particle system (born, falling under gravity,
/// splashing into a shallow sea, melting of old age), built in the `strobe` sibling module.
/// Returns its sinks (the Output nodes) if the graph is well-typed.
///
/// ⚠️ **Isto não é mais o documento de BOOT** (Enio, 2026-08-07: *"tire a cena da
/// cachoeira"*): o editor abre vazio. Ela é a **fixture de nível de artista** deste módulo,
/// e os três consumidores que a mantêm viva são gates, não o produto:
///
/// - **`motion_gpu_coverage`** — o censo escolhe o próximo kernel de GPU MEDINDO a fronteira
///   de CPU dos documentos que existem, e este é o único que um artista poderia ter autorado
///   (os demais são andaimes de caminho-de-GPU, vários moldados à mão para serem 100% device).
///   Tirá-lo do corpus deixaria o censo cego justamente onde ele decide.
/// - **`motion_delay_gate_tests`** — os números do mar são a fixture do gate do `motion.delay`.
/// - **`motion_state_tests`** — save/load/relógio provados sobre um grafo RICO; num documento
///   vazio um round-trip é verde por vácuo.
///
/// A cena carrega um **SUBGRAFO** ("Age & Fade", doc 57) porque a neve é byte-idêntica com e
/// sem ele (gate `grouping_never_changes_the_cook`) — a afirmação inteira do desenho de grupo,
/// no lugar onde é mais fácil de ver. ⚠️ Esse card era também a única vitrine da feature na
/// abertura do app; com o boot vazio, quem apresenta grupos é o command-palette, não a cena.
pub(crate) fn build_default_document(
    doc: &mut MotionDoc,
    reg: &NodeRegistry,
) -> Option<Vec<NodeId>> {
    let demo = strobe::build(&mut doc.graph)?;
    // Same "validate on load" the editor runs before cooking — proves the authored
    // graph is well-typed and membrane-clean.
    doc.graph.validate(reg).ok()?;
    // **The boot document ships a SUBGRAPH** (doc 57): the six nodes that age, colour,
    // shrink and fade a flake are folded into ONE card, sitting inline in the chain
    // with one socket on each side. So the feature is on the canvas the moment the tool
    // opens — double-click the card and you are inside it — and nobody has to build a
    // graph to find out that groups exist.
    //
    // The snow is **byte-identical** with the group as without it (gate:
    // `grouping_never_changes_the_cook`). That is the whole claim of the design, and
    // the boot document is where it is easiest to see: the flakes still fall.
    let sid = 0;
    // The centroid of what it folds — the SAME place the Ctrl+G gesture would put it
    // (`subgraph::group`), so the boot document is a document the artist could have
    // authored, not a special case the code knows about.
    let mut sum = (0.0f32, 0.0f32);
    for n in &demo.aging {
        let p = doc.graph.pos(*n)?;
        sum = (sum.0 + p.x, sum.1 + p.y);
    }
    let n = demo.aging.len() as f32;
    doc.subgraphs.push(ph2d_motion_doc::Subgraph {
        id: sid,
        parent: None,
        x: sum.0 / n,
        y: sum.1 / n,
        title: "Age & Fade".to_string(),
    });
    for id in &demo.aging {
        doc.members.insert(*id, sid);
    }
    Some(demo.sinks)
}
