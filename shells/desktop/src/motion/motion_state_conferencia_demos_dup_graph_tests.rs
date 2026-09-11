//! Gates da cena `=110` sobre **como ela está LIGADA** — irmão de [`super::tests`] por
//! RESPONSABILIDADE (HR-18), que mede o que ela DESENHA.
//!
//! ⚠️ **São duas perguntas mesmo, e nasceram de dois reports do Enio no mesmo dia:** *«tantos nós
//! interligados que não pude entender»* (a topologia) e *«você colocou grid entrando em Shape de
//! Duplicator!»* (o idioma de cada porta). Nenhuma das duas se vê nas colunas que a cena cospe —
//! elas vivem no GRAFO, e é por isso que têm ficheiro próprio.

use super::tests::scene;
use super::*;

/// ⭐⭐⭐ **UMA CADEIA POR SAÍDA, E ELA É FECHADA** — a lei desta cena, medida.
///
/// Report do Enio (2026-09-06): *«tem tantos nós interligados que não pude entender. Crie uma
/// cadeia de nós por output.»* A 1.ª versão partilhava as formas entre as bandas de cada
/// fileira, e os fios atravessavam a tela.
///
/// ⚠️ **A régua é a TOPOLOGIA, não a contagem de nós:** o que torna um grafo ilegível não é ele
/// ser grande, é um fio sair de uma banda e chegar a outra. ⇒ contam-se as **componentes
/// ligadas**, e têm de ser exactamente tantas quantas as saídas. FALSIFICADO por partilhar
/// qualquer nó entre duas bandas — as componentes fundem-se e a contagem cai.
#[test]
fn each_output_is_its_own_closed_chain() {
    let (state, sinks) = scene();
    let nos: Vec<NodeId> = state.doc.graph.nodes().iter().map(|n| n.id).collect();
    let indice = |id: NodeId| nos.iter().position(|n| *n == id).expect("o no' existe");
    // Union-find sobre as arestas: duas bandas que partilhem um nó caem no mesmo balde.
    let mut pai: Vec<usize> = (0..nos.len()).collect();
    fn achar(pai: &mut [usize], mut i: usize) -> usize {
        while pai[i] != i {
            pai[i] = pai[pai[i]];
            i = pai[i];
        }
        i
    }
    for e in state.doc.graph.edges() {
        let (a, b) = (indice(e.from.0), indice(e.to.0));
        let (ra, rb) = (achar(&mut pai, a), achar(&mut pai, b));
        pai[ra] = rb;
    }
    let mut raizes: Vec<usize> = (0..nos.len()).map(|i| achar(&mut pai, i)).collect();
    raizes.sort_unstable();
    raizes.dedup();
    assert_eq!(
        raizes.len(),
        BANDS,
        "o grafo tem {} ilhas para {BANDS} saidas: alguma banda partilha um no' com a vizinha",
        raizes.len()
    );
    // E cada SINK cai numa ilha diferente — sem isto, `BANDS` ilhas com dois sinks numa e um nó
    // solto noutra passaria pela contagem.
    let mut das_saidas: Vec<usize> = sinks.iter().map(|s| achar(&mut pai, indice(*s))).collect();
    das_saidas.sort_unstable();
    das_saidas.dedup();
    assert_eq!(das_saidas.len(), BANDS, "duas saidas na mesma cadeia");
}

/// ⭐⭐⭐ **O QUE ENTRA NA PORTA `shape` É UMA FORMA** — report do Enio, 2026-09-06: *«você colocou
/// grid entrando em Shape de Duplicator! Essa aplicação é correta?»*
///
/// Não era. Um `motion.grid` de uma célula funciona e desenha o ladrilho de omissão, mas o que
/// uma forma É neste app é um `source.shape` — e o doc dele nomeia esta composição à letra. ⛔ Uma
/// cena de demonstração que põe uma GRELHA onde vai uma FORMA ensina o artista a fazer o mesmo.
///
/// FALSIFICADO por voltar a alimentar o `shape` com qualquer coisa que não seja uma fonte de
/// forma: a porta 0 de cada carimbo é seguida até à origem dela.
#[test]
fn the_shape_port_is_fed_by_a_shape_source_and_the_points_port_by_an_arrangement() {
    let (state, _sinks) = scene();
    let g = &state.doc.graph;
    let tipo = |id: NodeId| {
        g.nodes()
            .iter()
            .find(|n| n.id == id)
            .map(|n| n.type_name.clone())
            .unwrap_or_default()
    };
    /// Sobe pela porta 0 até um nó SEM entradas — a fonte daquele braço.
    fn origem(g: &ph2d_nodegraph::graph::Graph, mut id: NodeId) -> NodeId {
        for _ in 0..32 {
            match g.edges().iter().find(|e| e.to.0 == id && e.to.1 == 0) {
                Some(e) => id = e.from.0,
                None => break,
            }
        }
        id
    }
    let mut carimbos = 0usize;
    for n in g
        .nodes()
        .iter()
        .filter(|n| n.type_name == "motion.duplicator")
    {
        carimbos += 1;
        for (porta, esperado) in [(0u16, "source.shape"), (1, "motion.grid")] {
            let e = g
                .edges()
                .iter()
                .find(|e| e.to.0 == n.id && e.to.1 == porta)
                .unwrap_or_else(|| panic!("o carimbo {:?} tem a porta {porta} ligada", n.id));
            let raiz = tipo(origem(g, e.from.0));
            assert_eq!(
                raiz, esperado,
                "a porta {porta} do carimbo {:?} nasce num `{raiz}`",
                n.id
            );
        }
    }
    assert_eq!(carimbos, BANDS, "um carimbo por banda");
}
