//! **SONDA — a zona tem RELÓGIO?** (doc 89, folha 13, célula 60 — *ciclo de vida*).
//!
//! A célula pede `start`/`delay`/`duration`/`loop` (o *Emitter State* do Niagara) e responde
//! **NÃO**, com a razão certa: `ctx.started()` é *"eu emiti algo no tique passado?"*, não um
//! relógio — nada a montante adia o 1.º cook da zona. Ela regista uma rota PARCIAL para UMA das
//! quatro (*"pare de nascer depois de N s"*, dirigindo `sim.spawn.rate` a zero).
//!
//! ⚠️ **Esta sonda mede as OUTRAS TRÊS**, que é o que decide o preço:
//!
//! ```text
//!   ADIAR      a sim ja' esta' a correr no tique 0?
//!   PARAR      ha' como congelar o que ja' nasceu (nao so' parar de nascer)?
//!   REINICIAR  ha' como voltar ao `init` a meio?
//! ```
//!
//! ⚠️ **Duas fixturas erradas antes desta**: `sim.spawn` não é passagem — a porta `0` dele é o
//! **template** dos recém-nascidos, então pô-lo no meio do interior faz a população inteira
//! desaparecer no 1.º tique (`5 → 0`); e `value.math` recebe o 2.º operando por PORTA, não por
//! param. *Uma fixtura que colapsa mede a fixtura.*
//!
//! Ela **imprime e não afirma**. Rode com
//! `cargo test -p ph2d-node-registry-init --test measure_zone_life_cycle -- --ignored --nocapture`.

use ph2d_node_registry::NodeRegistry;
use ph2d_nodegraph::attr::Column;
use ph2d_nodegraph::cook::Cook;
use ph2d_nodegraph::graph::{Edge, Graph, NodeId};

const TICKS: usize = 300;

fn registry() -> NodeRegistry {
    let mut reg = NodeRegistry::new();
    ph2d_node_registry_init::register_all_nodes(&mut reg).expect("every node registers");
    reg
}

fn wire(g: &mut Graph, from: NodeId, fp: u16, to: NodeId, tp: u16, delayed: bool) {
    g.connect(Edge {
        from: (from, fp),
        to: (to, tp),
        delayed,
    })
    .expect("wire");
}

/// Uma zona com uma fileira semeada no `init` e gravidade dentro — o interior mínimo que
/// existe (o molde é o da cena `=101`, sem os colisores).
fn falling_zone(g: &mut Graph) -> NodeId {
    let grid = g.add_node("motion.grid");
    g.set_param(grid, "rows", 1.0);
    g.set_param(grid, "cols", 5.0);
    g.set_param(grid, "gap_x", 0.4);

    let zone = g.add_node("sim.zone");
    wire(g, grid, 0, zone, 0, false);

    // ⚠️ **Não há `force.gravity` neste catálogo** — a gravidade é o `force.wind` apontado
    // para baixo, que é o que as cenas de chuva já fazem.
    let grav = g.add_node("force.wind");
    g.set_param(grav, "angle", 270.0);
    g.set_param(grav, "strength", 4.0);
    g.set_param(grav, "gust", 0.0);
    wire(g, zone, 0, grav, 0, true);
    let step = g.add_node("sim.step");
    g.set_param(step, "damping", 1.0);
    wire(g, grav, 0, step, 0, false);
    wire(g, step, 0, zone, 1, false);
    zone
}

#[test]
#[ignore = "sonda: imprime numeros, nao afirma"]
fn does_anything_today_delay_freeze_or_restart_a_zone() {
    let reg = registry();
    let mut g = Graph::new();
    let z = falling_zone(&mut g);
    g.validate(&reg).expect("bem-tipado");

    let mut cook = Cook::new();
    eprintln!("\n[ciclo de vida] 5 pecas semeadas numa zona, com gravidade dentro\n");
    eprintln!("  {:>6}  {:>5}  {:>9}", "t (s)", "pecas", "y medio");
    for k in 0..TICKS {
        let t = k as f64 / 60.0;
        let v = cook.cook(&g, &reg, z, t).expect("coza");
        if k % 60 == 0 || k == TICKS - 1 {
            let s = v[0].as_stream();
            let (n, y) = match s.get("P") {
                Some(Column::Vec2(p)) if !p.is_empty() => (
                    p.len(),
                    p.iter().map(|q| q[1]).sum::<f32>() / p.len() as f32,
                ),
                _ => (0, 0.0),
            };
            eprintln!("  {:>6.2}  {n:>5}  {y:>+9.3}", t);
        }
        cook.advance_tick(&g, &reg, t).expect("avanca");
    }

    eprintln!(
        "\n  LEITURA:
  ADIAR     -- se o `y` ja' desce entre t=0 e t=1, a sim corre desde o tique 0 e
               nada a montante a adia. Nao ha' param nenhum que mude isso.
  REINICIAR -- se o `y` DESCE monotonamente ate' ao fim, nada a devolve ao `init`.
               E' estrutural, nao acidente: a zona so' le^ o `init` quando
               `!ctx.started()`, e `started()` fica verdadeiro para sempre depois
               do 1.o emit (e' a cerca 3 da folha: um `start frame` NAO pode ser
               construido sobre «o meu estado esta' vazio»).
  CONGELAR  -- nao ha' rota nenhuma: a rota parcial que a celula nomeia para de
               NASCER, e nao para o que ja' nasceu."
    );
}
