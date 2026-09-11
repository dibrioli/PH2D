//! **ONDE O `motion.boids` DEIXA DE HONRAR `radius` E `max_speed`** (doc 89, folha 03 — linha 44).
//!
//! O `count` já tem teto medido (2²⁰, a wave do `f9b17c80e`); estes dois não têm nenhum.
//!
//! * **`radius` é o raio de PERCEPÇÃO e a célula da GRADE** (`GridSpec { cell_param: "radius" }`).
//!   Grande demais para a nuvem, todo agente é vizinho de todo agente — o mecanismo que o
//!   doc-comment do `spread` já descreve. Mas *ser vizinho de todos* é uma resposta **correta**
//!   e cara, não uma resposta errada: a pergunta é se existe ponto em que ele deixa de valer.
//! * **`max_speed` é um clamp de VELOCIDADE**, e o `eval` integra contra um `dt` que ele clampa
//!   em `MAX_DT = 0,1`. Rápido demais e o bando percorre mais que o próprio raio de percepção
//!   por passo ⇒ **cada agente chega onde já não vê ninguém**, e o bando deixa de ser bando.
//!   Essa é a fronteira medível, e ela é uma RAZÃO entre os dois params, não um número solto.
//!
//! Rodar: `cargo test -p ph2d-node-registry-init --release --test measure_boids_ceiling -- --ignored --nocapture`

use ph2d_node_registry::NodeRegistry;
use ph2d_nodegraph::attr::{Column, Stream};
use ph2d_nodegraph::cook::Cook;
use ph2d_nodegraph::graph::{Edge, Graph, NodeId};
use ph2d_nodegraph::value::CookValue;

const WORST_DT: f64 = 0.1;

fn registry() -> NodeRegistry {
    let mut reg = NodeRegistry::new();
    ph2d_node_registry_init::register_all_nodes(&mut reg).expect("every node registers");
    reg
}

/// ⚠️ A porta de estado do boids é a **2** (o `GridSpec` a nomeia: `port: 2`). O `pre` self-loop
/// é escrito à mão — sem ele o nó **semeia todo tique** e nunca dá um passo, que é como uma
/// medição anterior deste mesmo nó chegou a 3,2 ns por agente.
/// ⚠️ **Com os quatro pesos no MÁXIMO do slider** — o pior caso honesto: o ponto em que um
/// clamp fica inerte é onde ele passa da maior magnitude que o steering pode ter, e quem a
/// decide são os pesos, que o artista autora.
fn flock_f(radius: f32, max_speed: f32, count: f32, max_force: f32) -> (Graph, NodeId) {
    let (mut g, n) = flock(radius, max_speed, count);
    g.set_param(n, "max_force", max_force);
    for w in ["separation", "alignment", "cohesion", "seek"] {
        g.set_param(n, w, 6.0);
    }
    (g, n)
}

fn flock(radius: f32, max_speed: f32, count: f32) -> (Graph, NodeId) {
    let mut g = Graph::new();
    let n = g.add_node("motion.boids");
    g.set_param(n, "count", count);
    g.set_param(n, "radius", radius);
    g.set_param(n, "max_speed", max_speed);
    g.connect(Edge {
        from: (n, 0),
        to: (n, 2),
        delayed: true,
    })
    .expect("o self-loop de estado");
    (g, n)
}

fn march(g: &Graph, reg: &NodeRegistry, node: NodeId, frames: u64) -> Vec<[f32; 2]> {
    let mut cook = Cook::new();
    let mut last = Vec::new();
    for t in 0..frames {
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
    last
}

/// **O oráculo é a COESÃO, porque é o que um bando É.**
///
/// ⚠️ Nem a posição nem a velocidade servem: um bando que voa longe está certo, e um que voa
/// depressa também. O que distingue *bando* de *nuvem de partículas independentes* é os agentes
/// continuarem **perto uns dos outros** — então a régua é o raio médio ao centroide, e ela só
/// significa alguma coisa contra o raio de PERCEPÇÃO (um bando espalhado por dez vezes o que
/// enxerga não é um bando; é gente que perdeu o bando).
fn spread_over_radius(p: &[[f32; 2]], radius: f32) -> f32 {
    if p.is_empty() {
        return f32::NAN;
    }
    let n = p.len() as f32;
    let cx = p.iter().map(|q| q[0]).sum::<f32>() / n;
    let cy = p.iter().map(|q| q[1]).sum::<f32>() / n;
    let mean = p
        .iter()
        .map(|q| ((q[0] - cx).powi(2) + (q[1] - cy).powi(2)).sqrt())
        .sum::<f32>()
        / n;
    mean / radius
}

#[test]
#[ignore = "probe: prints the measured ceilings, asserts nothing"]
fn measure_where_the_flock_stops_being_a_flock() {
    let reg = registry();

    println!("\n== RADIUS: o raio de percepcao (e a celula da grade) ==");
    println!(
        "{:>14}  {:>16}  {:>18}  {:>10}",
        "radius", "espalh./raio", "espalhamento", "vivo?"
    );
    for &r in &[
        0.5, 2.0, 10.0, 100.0, 1e4, 1e8, 1e12, 1e16, 1e18, 1e20, 1e21,
    ] {
        let p = march(&flock(r, 4.0, 64.0).0, &reg, flock(r, 4.0, 64.0).1, 60);
        let s = spread_over_radius(&p, r);
        println!(
            "{r:>14.3e}  {s:>16.4}  {:>18.4e}  {:>10}",
            s * r,
            if s.is_finite() && s * r > 0.0 {
                "sim"
            } else {
                "MORREU"
            }
        );
    }

    println!("\n== MAX_SPEED: quando o passo passa do raio, o bando perde-se de vista ==");
    println!(
        "{:>14}  {:>16}  {:>18}  {:>14}",
        "max_speed", "passo/raio", "espalh./raio", "ainda bando?"
    );
    let radius = 2.0f32;
    for &v in &[
        1.0, 20.0, 1e3, 1e6, 1e12, 1e16, 1e18, 1e19, 1e20, 1e21, 1e24,
    ] {
        let p = march(
            &flock(radius, v, 64.0).0,
            &reg,
            flock(radius, v, 64.0).1,
            60,
        );
        let s = spread_over_radius(&p, radius);
        // O passo por tique no pior dt, contra o raio de percepcao: acima de 1 o agente
        // atravessa a propria vizinhanca num passo.
        let step_over_radius = v * WORST_DT as f32 / radius;
        println!(
            "{v:>14.3e}  {step_over_radius:>16.3e}  {s:>18.4e}  {:>14}",
            // ⚠️ Dois vereditos distintos: *deixou de ser bando* (espalhou) e *MORREU*
            // (deixou de existir). O 2º é o unico que um teto de representacao pina.
            if !s.is_finite() || s == 0.0 {
                "MORREU"
            } else if s < 10.0 {
                "sim"
            } else {
                "espalhou"
            }
        );
    }
    println!("\n== MAX_FORCE: e um CLAMP -- acima do steering possivel ele fica INERTE ==");
    println!(
        "{:>14}  {:>20}  {:>26}",
        "max_force", "espalh./raio", "identico ao de 1e6?"
    );
    let inert = {
        let (g, n) = flock_f(2.0, 4.0, 64.0, 1e6);
        spread_over_radius(&march(&g, &reg, n, 60), 2.0)
    };
    for &f in &[
        0.0, 50.0, 60.0, 70.0, 80.0, 90.0, 95.0, 99.0, 100.0, 1e4, 1e21,
    ] {
        let (g, n) = flock_f(2.0, 4.0, 64.0, f);
        let s = spread_over_radius(&march(&g, &reg, n, 60), 2.0);
        println!(
            "{f:>14.3e}  {s:>20.6e}  {:>26}",
            if s.to_bits() == inert.to_bits() {
                "SIM (byte a byte)"
            } else if !s.is_finite() {
                "MORREU"
            } else {
                "-"
            }
        );
    }
    println!();
}
