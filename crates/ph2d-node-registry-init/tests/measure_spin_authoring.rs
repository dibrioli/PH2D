//! **SONDA — quem AUTORA o `spin`, e ele chega ao passo?**
//!
//! O `sim.step` passou a integrar uma coluna `spin` no `rot` (doc 89, folha 13 — *POP Spin* /
//! *POP Drag Spin*). ⚠️ **Um integrador de uma coluna que ninguém consegue escrever é um knob
//! morto**, que é o pecado que esta casa recusa — então a pergunta que decide se a célula
//! fecha não é *"o passo integra?"* (isso os gates do nó provam) mas *"o artista alcança?"*.
//!
//! A célula supunha que faltava um CANAL novo no `motion.drive`. ⭐ **Falta nada:** o `drive`
//! tem o canal **`Custom…`** (índice 9), cujo nome de coluna vive num text param — ele escreve
//! **qualquer** coluna pelo nome. Logo:
//!
//! ```text
//!   POP Spin    drive(Custom, column = "spin", mode = Set)   cada peca gira a' sua taxa
//!   POP Torque  drive(Custom, column = "spin", mode = Add)   um empurrao angular que ACUMULA
//! ```
//!
//! Ela **imprime e não afirma**. Rode com
//! `cargo test -p ph2d-node-registry-init --test measure_spin_authoring -- --ignored --nocapture`.

use ph2d_node_registry::NodeRegistry;
use ph2d_nodegraph::attr::Column;
use ph2d_nodegraph::cook::Cook;
use ph2d_nodegraph::graph::{Edge, Graph, NodeId};

/// O índice do canal `Custom…` no enum do `motion.drive` (o último da lista de rótulos).
const CH_CUSTOM: f32 = 9.0;
/// Os modos do `drive` que interessam aqui.
const SET: f32 = 1.0;
const ADD: f32 = 0.0;
/// Quantos quadros a 60 fps.
const TICKS: usize = 120;

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

/// A cadeia: uma fila -> um valor por elemento -> `drive(Custom, "spin")` -> zona -> passo.
/// Devolve o sink e diz que ângulos saem depois de `TICKS` quadros.
fn angles(reg: &NodeRegistry, mode: f32, angular_damping: f32, in_loop: bool) -> Vec<f32> {
    let mut g = Graph::new();
    let grid = g.add_node("motion.grid");
    g.set_param(grid, "rows", 1.0);
    g.set_param(grid, "cols", 4.0);
    g.set_param(grid, "gap_x", 1.0);
    // Um valor por elemento: `0, 1/3, 2/3, 1` — quatro taxas de giro distintas.
    let field = g.add_node("value.instance_field");
    g.set_param(field, "mode", 1.0); // Ramp
    wire(&mut g, grid, 0, field, 0);

    let drive = g.add_node("motion.drive");
    g.set_param(drive, "channel", CH_CUSTOM);
    g.set_param(drive, "mode", mode);
    g.set_param(drive, "scale", 180.0); // graus por segundo, no topo da rampa
    g.set_text_param(drive, "column", "spin");
    // ⚠️ A porta 0 do `drive` recebe o STREAM: a grelha quando ele está fora do laço, e a
    // ZONA quando está dentro (senão ela já está ligada e o grafo recusa com
    // `InputAlreadyConnected` — o que a 1.ª versão desta variante levou).
    if !in_loop {
        wire(&mut g, grid, 0, drive, 0);
    }
    wire(&mut g, field, 0, drive, 1);

    let zone = g.add_node("sim.zone");
    let step = g.add_node("sim.step");
    g.set_param(step, "angular_damping", angular_damping);
    // ⚠️ **ONDE o `drive` está decide o que ele É.** Fora do laço (no que SEMEIA a zona) ele
    // escreve o `spin` UMA vez, ao nascer — isso é *POP Spin*. Dentro do laço ele corre a cada
    // tique sobre o estado, e aí `Add` ACUMULA — isso é *POP Torque*. A 1.ª versão desta sonda
    // pôs os dois modos fora do laço e mediu-os IGUAIS, e a minha nota dizia que o `Add` tinha
    // de crescer mais depressa: *o modo não é a lei — o modo mais o sítio é.*
    let seed = if in_loop { grid } else { drive };
    wire(&mut g, seed, 0, zone, 0);
    // ⚠️ **A aresta que sai da zona para o corpo da sim é ATRASADA** — é ela que fecha o laço
    // de estado sem ser um ciclo. A 1.ª versão ligou-a directa e levou um `WouldCycle`: o laço
    // de estado desta biblioteca não é uma convenção, é um `delayed`.
    let body_in = if in_loop { drive } else { step };
    g.connect(Edge {
        from: (zone, 0),
        to: (body_in, 0),
        delayed: true,
    })
    .expect("laco de estado");
    if in_loop {
        wire(&mut g, drive, 0, step, 0);
    }
    wire(&mut g, step, 0, zone, 1);
    g.validate(reg).expect("bem-tipado");

    let mut cook = Cook::new();
    let mut out = Vec::new();
    for k in 0..TICKS {
        let t = k as f64 / 60.0;
        let s = cook.cook(&g, reg, zone, t).expect("coza")[0]
            .as_stream()
            .clone();
        if k == TICKS - 1 {
            out = match s.get("rot") {
                Some(Column::Scalar(v)) => v.clone(),
                _ => Vec::new(),
            };
        }
        cook.advance_tick(&g, reg, t).expect("avanca");
    }
    out
}

#[test]
#[ignore = "sonda: imprime numeros, nao afirma"]
fn can_the_artist_author_a_spin_with_the_nodes_that_already_exist() {
    let reg = registry();
    eprintln!("\n[spin] `motion.drive(Custom, column = \"spin\")` -> `sim.step`\n");
    eprintln!(
        "  {:<34}  o angulo de cada uma das 4 pecas depois de {} quadros",
        "cadeia", TICKS
    );
    for (rotulo, mode, damp, in_loop) in [
        ("FORA  Set, sem arrasto (POP Spin)", SET, 1.0f32, false),
        ("FORA  Set, arrasto 0,3", SET, 0.3, false),
        ("FORA  Add, sem arrasto", ADD, 1.0, false),
        ("DENTRO Add, sem arrasto (POP Torque)", ADD, 1.0, true),
        ("DENTRO Add, arrasto 0,3", ADD, 0.3, true),
    ] {
        let a = angles(&reg, mode, damp, in_loop);
        let txt: Vec<String> = a.iter().map(|x| format!("{x:.1}")).collect();
        eprintln!("  {rotulo:<34}  [{}]", txt.join(" "));
    }
    eprintln!(
        "\n  LEITURA: se a coluna `rot` sair com QUATRO angulos diferentes, a autoria existe
  com os nos que ja' havia e a celula fecha -- o que faltava era alguem INTEGRAR, nao um
  canal novo. Se ela sair vazia, o `Custom…` nao alcanca o passo e o param novo e' um
  knob morto. ⚠️ E o `Add` so' e' TORQUE quando o `drive` esta' DENTRO do laco: fora
  dele ele escreve uma vez ao nascer e da' o mesmo que o `Set`."
    );
}
