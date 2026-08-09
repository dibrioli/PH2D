//! **O `accel` CHEGA AOS TRÊS GERADORES** (doc 89 §2.1 · W1 SIMULAÇÃO).
//!
//! A conferência dos 118 nós mediu que `motion.boids`, `motion.verlet_rope` e
//! `motion.soft_body` **não declaravam `Coupling` nenhum** — não liam `accel`,
//! nem `falloff`, nem `inv_mass`. A consequência não era um knob faltando: era
//! que a **família `force.*` inteira não alcançava simulação nenhuma**. A fiação
//! já existia (`motion_bridge_plumbing` faz de qualquer porta `state` um
//! feedback host e plumba `out --pre--> head`), o vocabulário já existia
//! (`Coupling::Consumes`), e o que faltava era **uma leitura de coluna**.
//!
//! ⚠️ **Roda AQUI e não nas crates dos nós**, e a razão é a mesma do
//! `param_census`: esta é a crate onde TODO nó é registrado, então é o build
//! mais barato que enxerga um gerador e uma força ao mesmo tempo. Um gate dentro
//! da `ph2d-node-motion-verlet-rope` teria de fabricar um `accel` à mão — o que
//! prova a aritmética e **não** prova que a força do artista chega lá.
//!
//! Cada gerador ganha o mesmo par: **a força CHEGA** (contra um controle sem
//! ela) e **zeros são a IDENTIDADE** (uma força de intensidade zero deixa a
//! simulação byte-idêntica à que não tem força nenhuma na cadeia).

use ph2d_node_registry::{Coupling, NodeRegistry};
use ph2d_nodegraph::attr::{Column, Stream};
use ph2d_nodegraph::cook::Cook;
use ph2d_nodegraph::graph::{Edge, Graph, NodeId};
use ph2d_nodegraph::node::NodeTypeId;

/// O registry REAL — as couplings sob teste são as que o app ship.
fn registry() -> NodeRegistry {
    let mut reg = NodeRegistry::new();
    ph2d_node_registry_init::register_all_nodes(&mut reg).expect("every node registers");
    reg
}

/// Os três geradores de simulação e a porta `state` de cada um (índice 2 nos
/// três: `anchor_x`/`anchor_y`/`state`, `target_x`/`target_y`/`state`).
const GENERATORS: &[&str] = &["motion.verlet_rope", "motion.soft_body", "motion.boids"];
const STATE_PORT: u16 = 2;

/// Monta o gerador com a cadeia de estado que o editor plumba.
///
/// Sem força: `gen.out --pre--> gen.state` (o self-loop de sempre).
/// Com força: `gen.out --pre--> force.wind --> gen.state` — ⚠️ a aresta
/// DELAYED é a que sai do gerador (`out --pre--> head`), exatamente como o
/// `motion_bridge_plumbing` a escreve; a de volta é comum, senão o ciclo tem
/// dois atrasos e o estado chega um tique velho.
fn rig(g: &mut Graph, ty: &str, params: &[(&str, f32)], wind: Option<(f32, f32)>) -> NodeId {
    let sim = g.add_node(ty);
    for (k, v) in params {
        g.set_param(sim, *k, *v);
    }
    // `head` é onde a aresta DELAYED aterrissa, e a porta dela muda com ele: o
    // self-loop volta para `state`, a cadeia com força entra pela porta 0 do
    // primeiro nó dela.
    let (head, head_port) = match wind {
        None => (sim, STATE_PORT),
        Some((angle, strength)) => {
            let w = g.add_node("force.wind");
            g.set_param(w, "angle", angle);
            g.set_param(w, "strength", strength);
            g.set_param(w, "gust", 0.0);
            g.connect(Edge {
                from: (w, 0),
                to: (sim, STATE_PORT),
                delayed: false,
            })
            .expect("force -> state");
            (w, 0)
        }
    };
    g.connect(Edge {
        from: (sim, 0),
        to: (head, head_port),
        delayed: true,
    })
    .expect("out --pre--> head");
    sim
}

/// Cozinha `ticks` quadros a 60 fps e devolve o último stream emitido pelo nó.
fn run(g: &Graph, reg: &NodeRegistry, node: NodeId, ticks: usize) -> Stream {
    let mut cook = Cook::new();
    let mut last = Stream::new(0);
    for k in 0..ticks {
        let t = k as f64 / 60.0;
        last = cook.cook(g, reg, node, t).expect("cooks")[0]
            .as_stream()
            .clone();
        cook.advance_tick(g, reg, t).expect("advances");
    }
    last
}

fn positions(s: &Stream) -> Vec<[f32; 2]> {
    match s.get("P") {
        Some(Column::Vec2(v)) => v.clone(),
        _ => panic!("o gerador tem de emitir P"),
    }
}

/// As posições do gerador `ty` depois de `ticks`, com e sem a força.
fn with_and_without(
    ty: &str,
    params: &[(&str, f32)],
    wind: (f32, f32),
    ticks: usize,
) -> (Vec<[f32; 2]>, Vec<[f32; 2]>) {
    let reg = registry();
    let (mut a, mut b) = (Graph::new(), Graph::new());
    let blown = rig(&mut a, ty, params, Some(wind));
    let still = rig(&mut b, ty, params, None);
    (
        positions(&run(&a, &reg, blown, ticks)),
        positions(&run(&b, &reg, still, ticks)),
    )
}

/// **Os três declaram `Consumes("accel")`.** É a metade que o ADR-0155 lê: sem
/// ela o diagnose vê um `Produces("accel")` sem consumidor e oferece **inserir
/// um `motion.integrate`** na cadeia de `state` — uma "cura" que carimba
/// `sim_t = playhead` de passagem e entrega `dt = 0` ao gerador, CONGELANDO-O.
/// FALSIFICADO por qualquer um dos três voltar a não declarar coupling.
#[test]
fn every_simulation_generator_declares_that_it_consumes_accel() {
    let reg = registry();
    for ty in GENERATORS {
        let cs = reg
            .couplings(NodeTypeId::of(ty))
            .unwrap_or_else(|| panic!("{ty} tem de declarar couplings"));
        assert!(
            cs.contains(&Coupling::Consumes("accel")),
            "{ty} tem de CONSUMIR accel — sem isso a familia force.* nao o alcanca, \
             e o diagnose oferece um integrador que o congela: {cs:?}"
        );
    }
}

/// **A ESTRELA: o vento levanta a corda.** Gravidade zero, então a única coisa
/// que pode mover o fio é o `accel` que a `force.wind` acumulou na cadeia de
/// estado — e o controle (a MESMA corda sem a força) fica **exatamente** parada,
/// o que torna o oráculo insensível a qualquer deriva do motor.
/// FALSIFICADO por `step` ignorar o `accel` (as duas medem 0,0).
#[test]
fn a_wind_in_the_state_chain_lifts_the_rope() {
    let params = [
        ("count", 12.0),
        ("length", 6.0),
        ("gravity", 0.0),
        ("damping", 0.0),
    ];
    // Ângulo 90 = +y (o `cos_sin_cycles` da wind é o ciclo padrão).
    let (blown, still) = with_and_without("motion.verlet_rope", &params, (90.0, 20.0), 60);

    let tail = |p: &[[f32; 2]]| p[p.len() - 1][1];
    assert!(
        tail(&still).abs() < 1e-6,
        "o controle tem de ficar PARADO (gravidade 0, sem forca): {}",
        tail(&still)
    );
    assert!(
        tail(&blown) > 1.0,
        "o vento tem de levantar a ponta livre: {}",
        tail(&blown)
    );
}

/// **O vento cisalha a gelatina.** Topo pinado + gravidade zero: o controle fica
/// exatamente parado e a força empurra a fileira de baixo para o lado.
///
/// ⚠️ **A barra é MEDIDA, e a curva satura** — o corpo cisalha até a restauração
/// elástica do shape matching equilibrar o vento, então o deslocamento não cresce
/// com a força indefinidamente (varrido: `5 → 0,073` · `20 → 0,282` ·
/// **`60 → 0,743`** · `180 → 1,348`). A barra de `0,3` é um fosso de 2,5× sobre a
/// medição, e o modo de falha que ela existe para pegar mede EXATAMENTE zero.
/// FALSIFICADO por a predição ignorar o `accel`.
#[test]
fn a_wind_in_the_state_chain_shears_the_soft_body() {
    let params = [
        ("rows", 4.0),
        ("cols", 4.0),
        ("spacing", 0.5),
        ("gravity", 0.0),
        ("pin", 1.0),
    ];
    let (blown, still) = with_and_without("motion.soft_body", &params, (0.0, 60.0), 60);

    // A última fileira (a mais longe do pino) é onde o cisalhamento se vê.
    let bottom_x = |p: &[[f32; 2]]| p[p.len() - 1][0];
    assert!(
        (bottom_x(&blown) - bottom_x(&still)) > 0.3,
        "o vento +x tem de empurrar a base: {} contra {}",
        bottom_x(&blown),
        bottom_x(&still)
    );
}

/// **O vento carrega o bando.** ⚠️ O controle aqui NÃO é estático — as
/// velocidades iniciais do boids são hasheadas —, então o oráculo é a
/// DIFERENÇA entre as duas corridas, que partem da MESMA semente.
/// FALSIFICADO por `step` ignorar o `accel` (as médias coincidem).
#[test]
fn a_wind_in_the_state_chain_carries_the_flock() {
    let params = [
        ("count", 40.0),
        ("seed", 7.0),
        ("seek", 0.0),
        ("max_speed", 12.0),
    ];
    let (blown, still) = with_and_without("motion.boids", &params, (0.0, 20.0), 60);

    let mean_x = |p: &[[f32; 2]]| p.iter().map(|q| q[0]).sum::<f32>() / p.len() as f32;
    assert!(
        (mean_x(&blown) - mean_x(&still)) > 0.5,
        "o vento +x tem de carregar o bando: {} contra {}",
        mean_x(&blown),
        mean_x(&still)
    );
}

/// **Zeros são a IDENTIDADE, nos três.** Uma `force.wind` de intensidade ZERO na
/// cadeia deixa a simulação **byte-idêntica** à que não tem força nenhuma — é
/// isso que torna esta wave inerte sobre toda arte já autorada, e é propriedade
/// da aritmética (`x + 0·dt²` é `x`), não um caminho rápido a manter em dia com
/// um lento. FALSIFICADO por qualquer termo novo que não anule em zero.
#[test]
fn a_force_of_zero_strength_leaves_every_generator_byte_identical() {
    let cases: [(&str, &[(&str, f32)]); 3] = [
        (
            "motion.verlet_rope",
            &[("count", 12.0), ("length", 6.0), ("gravity", 9.0)],
        ),
        (
            "motion.soft_body",
            &[("rows", 4.0), ("cols", 4.0), ("gravity", 9.0), ("pin", 1.0)],
        ),
        (
            "motion.boids",
            &[("count", 40.0), ("seed", 7.0), ("max_speed", 12.0)],
        ),
    ];
    for (ty, params) in cases {
        let (zeroed, none) = with_and_without(ty, params, (0.0, 0.0), 45);
        assert_eq!(
            zeroed, none,
            "{ty}: uma forca de intensidade zero tem de ser byte-identica a nenhuma forca"
        );
    }
}

/// **O `accel` é CONSUMIDO, nunca reemitido.** O gerador emite o próprio estado
/// e mais nada — se ele carregasse o `accel` adiante, a força do tique seguinte
/// somaria sobre um valor velho e a simulação aceleraria sozinha.
/// FALSIFICADO por o gerador reemitir a coluna.
#[test]
fn no_generator_carries_the_accel_forward() {
    let reg = registry();
    for ty in GENERATORS {
        let mut g = Graph::new();
        let sim = rig(&mut g, ty, &[], Some((90.0, 20.0)));
        let out = run(&g, &reg, sim, 5);
        assert!(
            out.get("accel").is_none(),
            "{ty} nao pode reemitir accel — o proximo tique somaria sobre o valor velho"
        );
    }
}
