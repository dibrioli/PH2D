//! **OS TETOS DO `motion.boids`, E A UNIDADE** (doc 88 A/B2 · doc 89 folha 03 linha 44).
//!
//! Números e tabelas no doc-comment do `PARAM_HARD_MAX` da crate; a sonda é a
//! `measure_boids_ceiling`. Os gates afirmam a PROPRIEDADE, e um deles afirma uma **ausência**.

use ph2d_node_registry::{NodeRegistry, ParamUnit};
use ph2d_nodegraph::attr::{Column, Stream};
use ph2d_nodegraph::cook::Cook;
use ph2d_nodegraph::graph::{Edge, Graph, NodeId};
use ph2d_nodegraph::node::NodeTypeId;
use ph2d_nodegraph::value::CookValue;

const BOIDS: NodeTypeId = NodeTypeId::of("motion.boids");
const WORST_DT: f64 = 0.1;

fn registry() -> NodeRegistry {
    let mut reg = NodeRegistry::new();
    ph2d_node_registry_init::register_all_nodes(&mut reg).expect("every node registers");
    reg
}

/// ⚠️ A porta de estado é a **2** (o `GridSpec` a nomeia). Sem o `pre` self-loop — que o editor
/// plumba ao SOLTAR o nó, e que `add_node` não dá — o boids **semeia todo tique e nunca dá um
/// passo**; é assim que uma medição anterior deste nó chegou a 3,2 ns por agente.
fn flock(radius: f32, max_speed: f32, max_force: f32, big_weights: bool) -> (Graph, NodeId) {
    let mut g = Graph::new();
    let n = g.add_node("motion.boids");
    g.set_param(n, "count", 64.0);
    g.set_param(n, "radius", radius);
    g.set_param(n, "max_speed", max_speed);
    g.set_param(n, "max_force", max_force);
    if big_weights {
        for w in ["separation", "alignment", "cohesion", "seek"] {
            g.set_param(n, w, 6.0);
        }
    }
    g.connect(Edge {
        from: (n, 0),
        to: (n, 2),
        delayed: true,
    })
    .expect("o self-loop de estado");
    (g, n)
}

/// O raio médio ao centroide sobre o raio de percepção — **a coesão, que é o que um bando É**.
/// Posição e velocidade não servem de oráculo: voar longe e voar depressa são ambos corretos.
fn cohesion(g: &Graph, reg: &NodeRegistry, node: NodeId, radius: f32) -> f32 {
    let mut cook = Cook::new();
    let mut last: Vec<[f32; 2]> = Vec::new();
    for t in 0..60u64 {
        let playhead = t as f64 * WORST_DT;
        cook.advance_tick(g, reg, playhead).expect("tick");
        let out = cook.cook(g, reg, node, playhead).expect("cook");
        let CookValue::Instances(s) = &out[0] else {
            panic!("a saida do boids e um stream")
        };
        last = match Stream::get(s, "P") {
            Some(Column::Vec2(v)) => v.clone(),
            _ => Vec::new(),
        };
    }
    let n = last.len() as f32;
    let cx = last.iter().map(|q| q[0]).sum::<f32>() / n;
    let cy = last.iter().map(|q| q[1]).sum::<f32>() / n;
    last.iter()
        .map(|q| ((q[0] - cx).powi(2) + (q[1] - cy).powi(2)).sqrt())
        .sum::<f32>()
        / n
        / radius
}

fn ceiling(param: &str) -> f32 {
    registry()
        .param_hard_max(BOIDS, param)
        .unwrap_or_else(|| panic!("o boids declara um teto digitavel para `{param}`"))
}

/// **`max_speed`: no teto o bando ainda EXISTE; dez vezes acima, ele deixa de existir.**
#[test]
fn at_the_max_speed_ceiling_the_flock_is_still_there_and_past_it_it_is_not() {
    let reg = registry();
    let cap = ceiling("max_speed");
    let (g, n) = flock(2.0, cap, 0.0, false);
    let alive = cohesion(&g, &reg, n, 2.0);
    assert!(
        alive.is_finite(),
        "no teto de max_speed ({cap}) o bando ainda tem de ser finito (coesao {alive})"
    );
    let (g2, n2) = flock(2.0, cap * 10.0, 0.0, false);
    let past = cohesion(&g2, &reg, n2, 2.0);
    assert!(
        !past.is_finite(),
        "dez vezes o teto tem de o levar ao inf (coesao {past}) -- o teto existe por causa desse \
         fim, nao por causa do bando espalhar (isso e visivel e reversivel)"
    );
}

/// **`max_force`: no teto o clamp ainda MORDE; acima, ele é byte a byte o DESLIGADO.**
///
/// ⚠️ Medido com os quatro pesos no MÁXIMO — são eles que decidem a maior magnitude possível do
/// steering, logo o ponto em que um clamp deixa de a alcançar. Um teto medido com os pesos no
/// default seria um teto sobre um documento que o artista não é obrigado a autorar.
#[test]
fn at_the_max_force_ceiling_the_clamp_still_bites_and_above_it_it_is_the_off_switch() {
    let reg = registry();
    let cap = ceiling("max_force");
    let at = |f: f32| {
        let (g, n) = flock(2.0, 4.0, f, true);
        cohesion(&g, &reg, n, 2.0)
    };
    let off = at(0.0); // `0` é o DESLIGADO deste param, não "força zero"
    assert_ne!(
        at(cap).to_bits(),
        off.to_bits(),
        "no teto ({cap}) o clamp ainda tem de MORDER -- se ja e igual a desligado, o teto esta \
         acima do que o kernel honra"
    );
    for over in [cap * 2.0, 1e4, 1e12] {
        assert_eq!(
            at(over).to_bits(),
            off.to_bits(),
            "com max_force = {over} o mundo tem de ser byte a byte o de max_force = 0 (o \
             desligado) -- e por isso a caixa de texto nao pode aceita-lo"
        );
    }
}

/// **`radius` NÃO tem teto, e a razão é medida: acima da extensão do bando ele SATURA.**
///
/// Todo agente passa a ser vizinho de todo agente, o que é uma resposta cara e **correta** — o
/// espalhamento absoluto para de mudar e nada morre, nem em `1e21`. Sem este gate a próxima
/// varredura "completa" a tabela e inventa o palpite que o §0 proíbe.
#[test]
fn the_perception_radius_has_no_ceiling_because_it_saturates_instead_of_breaking() {
    let reg = registry();
    assert!(
        registry().param_hard_max(BOIDS, "radius").is_none(),
        "o radius nao tem teto de proposito -- ele satura, nao quebra"
    );
    let spread = |r: f32| {
        let (g, n) = flock(r, 4.0, 0.0, false);
        cohesion(&g, &reg, n, r) * r // o espalhamento ABSOLUTO
    };
    let base = spread(1e4);
    for r in [1e8f32, 1e12, 1e16, 1e20] {
        let s = spread(r);
        assert!(
            s.is_finite() && (s - base).abs() < 0.01 * base.max(1.0),
            "acima da extensao do bando o espalhamento SATURA e nao muda mais \
             (r=1e4 -> {base}, r={r} -> {s})"
        );
    }
}

/// **A unidade do `radius`, e o VÃO deliberado nos outros dois.**
///
/// ⚠️ A metade de baixo é a que importa: `max_speed` e `max_force` são velocidade e aceleração,
/// e o `ParamUnit` não tem variante para nenhuma das duas. Declará-los `Length` seria a mentira
/// que a convenção da família proíbe — *uma unidade errada é pior que uma ausente*.
#[test]
fn the_world_distance_is_declared_and_the_speed_is_deliberately_bare() {
    let reg = registry();
    let unit = |p: &str| {
        reg.param_units(BOIDS)
            .unwrap_or(&[])
            .iter()
            .find(|d| d.param == p)
            .map_or(ParamUnit::None, |d| d.unit)
    };
    assert_eq!(
        unit("radius"),
        ParamUnit::Length,
        "o radius e uma distancia de MUNDO"
    );
    for bare in ["max_speed", "max_force"] {
        assert_eq!(
            unit(bare),
            ParamUnit::None,
            "`{bare}` fica NU de proposito: nao existe variante de velocidade nem de aceleracao, \
             e rotula-lo Length ensinaria ao artista algo falso"
        );
    }
}
