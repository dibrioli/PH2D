//! **SONDA — os dois DEFEITOS de junção da folha 08, medidos na cadeia real.**
//!
//! As duas células descrevem comportamento errado, não knob ausente, e é por isso que vêm
//! antes do resto da folha:
//!
//! 1. **`motion.combine` não renumera.** O `concat` copia toda coluna verbatim, e cada fonte
//!    escreve o seu próprio `Index = 0..n−1` e `Count = n` ⇒ juntar duas grelhas devolve uma
//!    lista cujas duas colunas de identidade **mentem**. Os irmãos `motion.clone` e
//!    `motion.duplicator` renumeram de propósito (*"so a downstream ramp reads one 0..total
//!    run"*); este não.
//! 2. **`motion.duplicator` descarta a ESCALA do ponto.** Só `P` e `rot` somam; toda outra
//!    coluna do ponto é deitada fora, então um `motion.scatter` que já produziu `size` por
//!    ponto perde-o ao carimbar. As três referências (Houdini `pscale`, Blender o socket
//!    `Scale`, Cavalry `Shape Scale`) são unânimes.
//!
//! Ela **imprime e não afirma**. Rode com
//! `cargo test -p ph2d-node-registry-init --test measure_stream_join_defects -- --ignored --nocapture`.

use ph2d_node_registry::NodeRegistry;
use ph2d_nodegraph::attr::{Column, Stream};
use ph2d_nodegraph::cook::Cook;
use ph2d_nodegraph::graph::{Edge, Graph, NodeId};
use ph2d_nodegraph::value::CookValue;

fn registry() -> NodeRegistry {
    let mut reg = NodeRegistry::new();
    ph2d_node_registry_init::register_all_nodes(&mut reg).expect("every node registers");
    reg
}

fn wire(g: &mut Graph, from: NodeId, fp: u16, to: NodeId, tp: u16) {
    g.connect(Edge {
        from: (from, fp),
        to: (to, tp),
        delayed: false,
    })
    .expect("wire");
}

fn grid(g: &mut Graph, rows: f32, cols: f32) -> NodeId {
    let n = g.add_node("motion.grid");
    g.set_param(n, "rows", rows);
    g.set_param(n, "cols", cols);
    n
}

fn cooked(g: &Graph, reg: &NodeRegistry, sink: NodeId) -> Stream {
    let mut cook = Cook::new();
    let out = cook.cook(g, reg, sink, 0.0).expect("coza");
    let CookValue::Instances(s) = &out[0] else {
        panic!("stream")
    };
    s.clone()
}

fn scalars(s: &Stream, name: &str) -> Vec<f32> {
    match s.get(name) {
        Some(Column::Scalar(v)) => v.clone(),
        _ => Vec::new(),
    }
}

#[test]
#[ignore = "sonda: imprime numeros, nao afirma"]
fn what_the_combine_does_to_the_identity_columns() {
    let reg = registry();
    let mut g = Graph::new();
    let a = grid(&mut g, 3.0, 3.0); // 9
    let b = grid(&mut g, 2.0, 2.0); // 4
    let c = g.add_node("motion.combine");
    wire(&mut g, a, 0, c, 0);
    wire(&mut g, b, 0, c, 1);
    g.validate(&reg).expect("bem-tipado");
    let s = cooked(&g, &reg, c);

    eprintln!("\n[join] `grid(9) + grid(4) -> motion.combine`\n");
    eprintln!("  linhas: {}", s.count());
    for col in ["Index", "Count"] {
        let v = scalars(&s, col);
        eprintln!(
            "  {col:>6}: {}",
            v.iter()
                .map(|x| format!("{x:.0}"))
                .collect::<Vec<_>>()
                .join(" ")
        );
    }
    eprintln!(
        "\n  LEITURA: com 13 linhas, o `Index` honesto e' 0..12 e o `Count` e' 13 em todas.
  Se aparecerem dois `0` no Index, ou dois valores distintos no Count, as duas
  colunas de identidade MENTEM — e todo efeito dirigido por indice a jusante
  (uma rampa de cor, um stagger) le' a lista como se fossem duas."
    );
}

#[test]
#[ignore = "sonda: imprime numeros, nao afirma"]
fn what_the_duplicator_keeps_from_the_point() {
    let reg = registry();
    let mut g = Graph::new();
    // A "forma": uma linha com `size` conhecido.
    let shape = grid(&mut g, 1.0, 1.0);
    // Os PONTOS, com um `size` por ponto que varia — o que um `motion.scatter` produz.
    let pts = grid(&mut g, 1.0, 4.0);
    let vary = g.add_node("value.instance_field");
    g.set_param(vary, "mode", 0.0); // índice normalizado
    wire(&mut g, pts, 0, vary, 0);
    let drive = g.add_node("motion.drive");
    g.set_param(drive, "channel", 3.0); // Size
    g.set_param(drive, "mode", 1.0); // Set
    g.set_param(drive, "scale", 4.0);
    wire(&mut g, pts, 0, drive, 0);
    wire(&mut g, vary, 0, drive, 1);

    let dup = g.add_node("motion.duplicator");
    wire(&mut g, shape, 0, dup, 0);
    wire(&mut g, drive, 0, dup, 1);
    g.validate(&reg).expect("bem-tipado");

    let points = cooked(&g, &reg, drive);
    let out = cooked(&g, &reg, dup);
    eprintln!("\n[join] o `size` que os PONTOS carregam, e o que sai do carimbo\n");
    let show = |tag: &str, s: &Stream| {
        eprintln!(
            "  {tag:>18}: {} linhas · size {:?}",
            s.count(),
            match s.get("size") {
                Some(Column::Vec2(v)) => v.iter().map(|q| format!("{:.2}", q[0])).collect(),
                Some(Column::Scalar(v)) => v.iter().map(|x| format!("{x:.2}")).collect(),
                _ => vec!["<sem coluna>".into()],
            }
        );
    };
    show("os pontos", &points);
    show("depois do carimbo", &out);
    eprintln!(
        "\n  LEITURA: se o `size` da saida for constante enquanto o dos pontos VARIA, a escala
  autorada do ponto foi deitada fora — que e' o defeito da celula."
    );
}
