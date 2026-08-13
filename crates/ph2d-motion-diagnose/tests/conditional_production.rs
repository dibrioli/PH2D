//! **UMA PRODUÇÃO QUE DEPENDE DO MODO** — o canal `Coupling::ProducesWhen`.
//!
//! O `motion.make_point` ganhou um alvo (`P` · `vel` · `accel`), e só um dos três
//! escreve a coluna **transiente** que o ADR-0155 vigia. Isso põe a declaração
//! entre duas paredes:
//!
//! - **`Produces("accel")` seco** marcaria TODA instância do nó, inclusive as que
//!   constroem posições — o falso positivo que o ADR-0155 combateu no Boids, agora
//!   vindo da declaração em vez da inferência.
//! - **Não declarar nada** devolve a classe de erro que o ADR existe para pegar:
//!   uma aceleração escrita, nada a consome, e a cena fica parada **sem erro**.
//!
//! O canal condicional é a terceira saída, e este arquivo é a MEDIÇÃO de que ele
//! funciona pelas duas pontas — a presença **e** a ausência. Um gate que só
//! afirmasse a presença ficaria verde sobre um `Produces` seco, que é o defeito
//! mais provável de alguém introduzir "simplificando".

use ph2d_motion_diagnose::{Deficit, diagnose};
use ph2d_node_registry::NodeRegistry;
use ph2d_nodegraph::graph::{Edge, Graph, NodeId};

/// Índices do param `target` do `motion.make_point`.
const TARGET_POSITION: f32 = 0.0;
const TARGET_VELOCITY: f32 = 1.0;
const TARGET_ACCELERATION: f32 = 2.0;

fn registry() -> NodeRegistry {
    let mut reg = NodeRegistry::new();
    ph2d_node_registry_init::register_all_nodes(&mut reg).expect("every node registers");
    reg
}

/// `grid → make_point(target)`, com os dois campos de valor ligados e **nada** a
/// jusante — o grafo em que uma coluna transiente escrita é inerte.
fn lone_make_point(target: f32) -> (Graph, NodeId) {
    let mut g = Graph::new();
    let grid = g.add_node("motion.grid");
    g.set_param(grid, "rows", 1.0);
    g.set_param(grid, "cols", 3.0);

    let mp = g.add_node("motion.make_point");
    g.set_param(mp, "target", target);
    g.connect(Edge {
        from: (grid, 0),
        to: (mp, 0),
        delayed: false,
    })
    .expect("in");
    for port in [1u16, 2] {
        let k = g.add_node("debug.const");
        g.connect(Edge {
            from: (k, 0),
            to: (mp, port),
            delayed: false,
        })
        .expect("field");
    }
    (g, mp)
}

fn inert_accel(g: &Graph, reg: &NodeRegistry, node: NodeId) -> bool {
    diagnose(g, reg)
        .iter()
        .any(|d| d.node == node && d.deficit == Deficit::InertProducer("accel"))
}

/// **O modo que escreve a aceleração é DIAGNOSTICADO quando ninguém a integra.**
#[test]
fn a_make_point_that_writes_acceleration_with_no_integrator_is_reported() {
    let reg = registry();
    let (g, mp) = lone_make_point(TARGET_ACCELERATION);
    assert!(
        inert_accel(&g, &reg, mp),
        "uma aceleracao escrita que ninguem consome tem de ser dita: {:?}",
        diagnose(&g, &reg)
    );
}

/// **E os outros dois modos ficam QUIETOS** — a metade que um `Produces` seco
/// quebraria, e a razão de o canal ser condicional.
#[test]
fn the_modes_that_do_not_write_acceleration_are_not_reported() {
    let reg = registry();
    for target in [TARGET_POSITION, TARGET_VELOCITY] {
        let (g, mp) = lone_make_point(target);
        assert!(
            !inert_accel(&g, &reg, mp),
            "target = {target} nao escreve `accel`, entao nao ha o que dizer: {:?}",
            diagnose(&g, &reg)
        );
    }
}

/// **E com um integrador a jusante o modo Acceleration também fica quieto** — o
/// diagnóstico é sobre estar INERTE, não sobre escrever a coluna.
///
/// ⚠️ É este gate que separa *"o canal condicional funciona"* de *"o canal
/// condicional só sabe reclamar"*: sem ele, uma declaração que marcasse o modo
/// Acceleration **sempre** passaria nos dois de cima.
#[test]
fn an_integrator_downstream_makes_the_acceleration_healthy() {
    let reg = registry();
    let (mut g, mp) = lone_make_point(TARGET_ACCELERATION);
    let ig = g.add_node("motion.integrate");
    // O `integrate` lê o `accel` pela porta de forças (o laço), que é a rota real.
    let wired = (0..4u16).any(|port| {
        g.connect(Edge {
            from: (mp, 0),
            to: (ig, port),
            delayed: false,
        })
        .is_ok()
    });
    assert!(wired, "o integrador tem de aceitar o stream em alguma porta");
    assert!(
        !inert_accel(&g, &reg, mp),
        "com um consumidor a jusante a producao e saudavel: {:?}",
        diagnose(&g, &reg)
    );
}

/// **O default do param entra na conta** — um nó recém-criado, cujo `target`
/// ninguém tocou, lê o default do manifesto e não o `0.0` de um mapa vazio.
///
/// ⚠️ Aqui os dois coincidem (o default É Position), então o gate mede a ESCADA,
/// não o número: com o alvo produtor movido para o índice 0 um leitor sem
/// fallback ficaria verde por acaso. Ele guarda a regra para o dia em que o
/// default de algum nó com este canal não seja o modo silencioso.
#[test]
fn an_untouched_param_reads_the_manifest_default() {
    let reg = registry();
    let mut g = Graph::new();
    let grid = g.add_node("motion.grid");
    let mp = g.add_node("motion.make_point");
    // NENHUM `set_param`: o mapa de params deste nó está vazio.
    g.connect(Edge {
        from: (grid, 0),
        to: (mp, 0),
        delayed: false,
    })
    .expect("in");
    assert!(
        !g.node_params().get(&mp).is_some_and(|m| m.contains_key("target")),
        "a premissa do gate: o param nao foi tocado"
    );
    assert!(
        !inert_accel(&g, &reg, mp),
        "e o default (Position) nao produz aceleracao nenhuma"
    );
}
