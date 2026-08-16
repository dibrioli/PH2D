//! **O PESO POR PARTÍCULA ALCANÇA A GELATINA — pelo mesmo fio do `accel` e do
//! `inv_mass`.**
//!
//! A folha 03 pedia *goal/peso por partícula* (o **Goal** por vertex group do
//! Blender Softbody · a espinha MOPs *"TODO modificador é modulado por
//! `mops_falloff`"*) e registava a causa: *"o nó não lê `falloff` nem
//! `inv_mass`"*. A segunda metade envelheceu com a wave do pino; esta fecha a
//! primeira.
//!
//! ⚠️ **Medido ANTES de uma linha ser escrita**, com um `field.index_range` no
//! laço de estado e o corpo a ignorá-lo: **pior deslocamento `0,000000`**. O vão
//! era real, e a cura é **uma leitura de coluna** — nenhum kernel novo, nenhuma
//! porta nova, nenhum schema.
//!
//! ⚠️ **E o `pre` mora na aresta que ENTRA no campo**, não na que sai dele: é ela
//! que quebra o ciclo, e o `field.index_range` é `Effect::Pure` ⇒ não carimba
//! `sim_t`, então o solver ainda vê o `dt` do próprio relógio no tique seguinte.
//! (A mesma topologia que o `the_pin_reaches_the_sims` usa para o pino.)

use ph2d_node_registry::NodeRegistry;
use ph2d_nodegraph::attr::Column;
use ph2d_nodegraph::cook::Cook;
use ph2d_nodegraph::graph::{Edge, Graph, NodeId};

const TICKS: usize = 120;
const DT: f64 = 1.0 / 60.0;

fn registry() -> NodeRegistry {
    let mut reg = NodeRegistry::new();
    ph2d_node_registry_init::register_all_nodes(&mut reg).expect("todo nó registra");
    reg
}

fn wire(g: &mut Graph, from: NodeId, fp: u16, to: NodeId, tp: u16, delayed: bool) {
    g.connect(Edge {
        from: (from, fp),
        to: (to, tp),
        delayed,
    })
    .expect("liga");
}

/// O campo no laço de estado, ou a ausência dele. `band` é a fracção da lista
/// que fica com peso CHEIO (o resto vai a zero).
#[derive(Clone, Copy)]
enum Field {
    None,
    Band { end: f32 },
}

fn pose(field: Field) -> Vec<[f32; 2]> {
    let reg = registry();
    let mut g = Graph::new();
    let b = g.add_node("motion.soft_body");
    g.set_param(b, "rows", 6.0);
    g.set_param(b, "cols", 6.0);
    g.set_param(b, "pin", 1.0);
    g.set_param(b, "gravity", 9.8);
    match field {
        Field::None => wire(&mut g, b, 0, b, 2, true),
        Field::Band { end } => {
            let f = g.add_node("field.index_range");
            g.set_param(f, "start", 0.0);
            g.set_param(f, "end", end);
            g.set_param(f, "soft", 0.0);
            wire(&mut g, b, 0, f, 0, true);
            wire(&mut g, f, 0, b, 2, false);
        }
    }

    let mut cook = Cook::new();
    let mut out = Vec::new();
    for k in 0..TICKS {
        let t = k as f64 * DT;
        cook.advance_tick(&g, &reg, t)
            .expect("o tique avança o `pre`");
        let v = cook.cook(&g, &reg, b, t).expect("o corpo coze");
        if let Some(Column::Vec2(p)) = v[0].as_stream().get("P") {
            out = p.clone();
        }
    }
    out
}

fn worst(a: &[[f32; 2]], b: &[[f32; 2]]) -> f32 {
    a.iter()
        .zip(b)
        .map(|(p, q)| ((p[0] - q[0]).powi(2) + (p[1] - q[1]).powi(2)).sqrt())
        .fold(0.0f32, f32::max)
}

/// **UM CAMPO NO LAÇO MOVE A GELATINA.** Este é o gate que nasceu VERMELHO com
/// `0,000000` — o vão que a folha 03 nomeava.
#[test]
fn a_field_in_the_state_chain_reaches_the_soft_body() {
    let plain = pose(Field::None);
    let banded = pose(Field::Band { end: 0.5 });
    let d = worst(&plain, &banded);
    assert!(
        d > 0.5,
        "um `field.*` no laço tem de mudar a pose do corpo; pior deslocamento {d:.6}"
    );
}

/// **O CONTROLE, e ele é mais forte que a ausência:** um campo que dá peso CHEIO
/// a toda partícula tem a coluna PRESENTE e o leitor a CORRER — então ele prova
/// o RAMO, não só o `else`. A barra é `1e-3` e não zero pelo motivo escrito no
/// `falloff_col`: com a coluna presente o centroide de repouso passa a ser
/// calculado em vez de assumido zero, e ele vale `~1e-7` numa malha real.
#[test]
fn a_field_that_selects_everyone_leaves_the_body_where_it_was() {
    let plain = pose(Field::None);
    let all = pose(Field::Band { end: 1.0 });
    let d = worst(&plain, &all);
    assert!(
        d < 1e-3,
        "peso cheio em toda parte tem de ser o corpo de sempre; {d:.6}"
    );
}
