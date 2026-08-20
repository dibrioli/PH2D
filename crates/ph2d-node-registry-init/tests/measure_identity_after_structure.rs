//! **SONDA** (doc 89, folha 08) — *o que acontece às colunas de identidade quando um nó de
//! ESTRUTURA mexe na lista?*
//!
//! ⚠️ **Ela imprime e não afirma.** A wave da renumeração do `motion.sort` (2026-08-19) fechou
//! um caso desta família: o `Index` é um facto sobre a LISTA, e o nó que reordenava a lista
//! levava-o consigo, então o único consumidor da coluna (`motion.tint` em gradiente) pintava a
//! ordem de antes. Os irmãos que **encolhem** (`motion.cull`) e que **crescem**
//! (`motion.mirror`, `motion.kaleidoscope`) mexem na mesma lista — esta sonda mede o que eles
//! deixam lá, para que a célula da folha leve o número e não o meu palpite.
//!
//! Correr: `cargo test -p ph2d-node-registry-init --test measure_identity_after_structure
//! -- --ignored --nocapture`

use ph2d_node_registry::NodeRegistry;
use ph2d_nodegraph::attr::Column;
use ph2d_nodegraph::cook::Cook;
use ph2d_nodegraph::graph::{Edge, Graph, NodeId};

fn registry() -> NodeRegistry {
    let mut reg = NodeRegistry::new();
    ph2d_node_registry_init::register_all_nodes(&mut reg).expect("todo nó registra");
    reg
}

fn wire(g: &mut Graph, from: NodeId, to: NodeId) {
    g.connect(Edge {
        from: (from, 0),
        to: (to, 0),
        delayed: false,
    })
    .expect("liga");
}

/// Imprime `n`, `Index` e `Count` do sink.
fn report(label: &str, g: &Graph, reg: &NodeRegistry, sink: NodeId) {
    let mut cook = Cook::new();
    let out = match cook.cook(g, reg, sink, 0.0) {
        Ok(v) => v,
        Err(e) => {
            println!("  {label:<34} ERRO {e:?}");
            return;
        }
    };
    let st = out[0].as_stream();
    let col = |name: &str| match st.get(name) {
        Some(Column::Scalar(v)) => Some(v.clone()),
        _ => None,
    };
    let show = |v: &Option<Vec<f32>>| match v {
        None => "(ausente)".to_string(),
        Some(v) => {
            let head: Vec<String> = v.iter().take(10).map(|x| format!("{x:.0}")).collect();
            let tail = if v.len() > 10 { ", …" } else { "" };
            format!("[{}{}]", head.join(", "), tail)
        }
    };
    let (idx, cnt) = (col("Index"), col("Count"));
    let honest = match &idx {
        Some(v) => v
            .iter()
            .enumerate()
            .all(|(i, x)| (x - i as f32).abs() < 1e-6),
        None => true,
    };
    let count_ok = match &cnt {
        Some(v) => v.iter().all(|x| (x - st.count() as f32).abs() < 1e-6),
        None => true,
    };
    println!(
        "  {label:<34} n={:<4} Index={:<44} Count={:<12} → Index 0..n-1? {} · Count==n? {}",
        st.count(),
        show(&idx),
        show(&cnt),
        if honest { "SIM" } else { "NÃO" },
        if count_ok { "SIM" } else { "NÃO" },
    );
}

/// Uma grelha 3×3 — nove peças, `Index = 0..8`.
fn grid(g: &mut Graph) -> NodeId {
    let n = g.add_node("motion.grid");
    g.set_param(n, "rows", 3.0);
    g.set_param(n, "cols", 3.0);
    n
}

#[test]
#[ignore = "sonda: imprime, não afirma"]
fn what_the_structure_nodes_leave_in_the_identity_columns() {
    let reg = registry();
    println!("\n== a identidade depois de um nó de ESTRUTURA (grelha 3×3 à entrada) ==");

    {
        let mut g = Graph::new();
        let src = grid(&mut g);
        report("grid(3x3) sozinha", &g, &reg, src);
    }
    for (label, key) in [("X", 1.0), ("Random", 3.0)] {
        let mut g = Graph::new();
        let src = grid(&mut g);
        let s = g.add_node("motion.sort");
        g.set_param(s, "key", key);
        wire(&mut g, src, s);
        report(&format!("sort({label}) reindex=default"), &g, &reg, s);
        g.set_param(s, "reindex", 0.0);
        report(&format!("sort({label}) reindex=0"), &g, &reg, s);
    }
    {
        let mut g = Graph::new();
        let src = grid(&mut g);
        let c = g.add_node("motion.cull");
        g.set_param(c, "amount", 0.5);
        wire(&mut g, src, c);
        report("cull(Fraction 0.5) — ENCOLHE", &g, &reg, c);
    }
    {
        let mut g = Graph::new();
        let src = grid(&mut g);
        let m = g.add_node("motion.mirror");
        wire(&mut g, src, m);
        report("mirror — CRESCE", &g, &reg, m);
    }
    {
        let mut g = Graph::new();
        let src = grid(&mut g);
        let k = g.add_node("motion.kaleidoscope");
        wire(&mut g, src, k);
        report("kaleidoscope — CRESCE", &g, &reg, k);
    }
    {
        let mut g = Graph::new();
        let src = grid(&mut g);
        let c = g.add_node("motion.clone");
        g.set_param(c, "copies", 3.0);
        wire(&mut g, src, c);
        report("clone(3) — CRESCE (renumera)", &g, &reg, c);
    }
    println!();
}
