//! Os gates da seleção por clique.
//!
//! ⚠️ **Eles apontam para um sítio ONDE SE SABE quem está** — não para um pixel qualquer da peça. Um
//! gate que clicasse no meio de uma união e afirmasse *"deu um objeto"* passaria com a resposta
//! errada, porque ali qualquer um dos dois é plausível. Aqui aponta-se a **ponta** de cada cilindro,
//! onde só ele existe, e a resposta certa é uma só.

use super::node_under;
use bevy_ecs::entity::Entity;
use ph2d_ecs::SimWorld;
use ph2d_field::{Blend, FieldDoc, Node, NodeId, NodeKind, Op, Primitive, Xform};
use ph2d_field_render::{Orbit, Screen};

const W: u32 = 400;
const H: u32 = 320;

/// Três esferas bem separadas: cada uma tem uma região da tela que é só dela.
fn three_spheres() -> FieldDoc {
    let leaf = |x: f32, y: f32| Node {
        xform: Xform::at(x, y, 0.0),
        kind: NodeKind::Leaf(Primitive::Sphere { radius: 0.15 }),
    };
    FieldDoc::new(
        vec![
            leaf(-0.45, 0.0),
            leaf(0.0, 0.0),
            leaf(0.45, 0.0),
            Node {
                xform: Xform::IDENTITY,
                kind: NodeKind::Combine {
                    op: Op::Union(Blend::Sharp),
                    children: vec![NodeId(0), NodeId(1), NodeId(2)],
                },
            },
        ],
        NodeId(3),
    )
    .expect("três esferas")
}

/// Vista de frente, para o `x` do mundo cair direto no `x` da tela.
fn front() -> Orbit {
    Orbit::from_yaw_pitch(0.0, 0.0)
}

/// ⭐ **Clicar numa esfera devolve AQUELA esfera** — as três, uma a uma.
#[test]
fn clicking_a_shape_returns_that_shape() {
    let mut sim = SimWorld::new();
    let doc = three_spheres();
    let world = sim.world_mut();
    let root = ph2d_field_ecs::spawn_doc(world, &doc, "Model");
    let kids: Vec<Entity> = world
        .get::<bevy_ecs::hierarchy::Children>(root)
        .expect("tem filhos")
        .iter()
        .copied()
        .collect();
    assert_eq!(kids.len(), 3);

    let c = front();
    let s = Screen::new(W, H, c.half_extent);
    for (k, centre) in [(-0.45f32), 0.0, 0.45].into_iter().enumerate() {
        let (px, _) = c.project([centre, 0.0, 0.0], s);
        assert_eq!(
            node_under(sim.world(), root, &doc, &c, s, px),
            Some(kids[k]),
            "o centro da esfera {k} tem de ser dela"
        );
    }
}

/// **Clicar no fundo não devolve nada** — e é o que faz um clique no vazio limpar a seleção em vez
/// de escolher o objeto mais próximo.
#[test]
fn clicking_the_background_returns_nothing() {
    let mut sim = SimWorld::new();
    let doc = three_spheres();
    let world = sim.world_mut();
    let root = ph2d_field_ecs::spawn_doc(world, &doc, "Model");
    let c = front();
    let s = Screen::new(W, H, c.half_extent);
    assert_eq!(node_under(sim.world(), root, &doc, &c, s, [2.0, 2.0]), None);
}

/// ⭐ **A resposta usa a pose de MUNDO**, e o gate prova-o com um grupo deslocado.
///
/// ⚠️ Avaliar cada folha com a pose **local** dela responderia sobre um sítio onde ela não está. Numa
/// peça plana (grupo na identidade) as duas contas dão o mesmo, e o defeito ficaria escondido até
/// alguém agrupar e mover — aí o clique passaria a escolher o vizinho.
#[test]
fn the_answer_uses_the_world_pose_not_the_local_one() {
    let mut sim = SimWorld::new();
    let doc = {
        let leaf = |x: f32| Node {
            xform: Xform::at(x, 0.0, 0.0),
            kind: NodeKind::Leaf(Primitive::Sphere { radius: 0.12 }),
        };
        FieldDoc::new(
            vec![
                leaf(-0.3),
                leaf(0.3),
                Node {
                    // O grupo inteiro anda para a direita e para cima.
                    xform: Xform::at(0.35, 0.25, 0.0),
                    kind: NodeKind::Combine {
                        op: Op::Union(Blend::Sharp),
                        children: vec![NodeId(0), NodeId(1)],
                    },
                },
            ],
            NodeId(2),
        )
        .expect("duas esferas num grupo deslocado")
    };
    let world = sim.world_mut();
    let root = ph2d_field_ecs::spawn_doc(world, &doc, "Model");
    let kids: Vec<Entity> = world
        .get::<bevy_ecs::hierarchy::Children>(root)
        .expect("tem filhos")
        .iter()
        .copied()
        .collect();

    let c = front();
    let s = Screen::new(W, H, c.half_extent);
    // A esfera da ESQUERDA está, no mundo, em (0.05, 0.25) — e é ali que se clica.
    let (px, _) = c.project([0.05, 0.25, 0.0], s);
    assert_eq!(
        node_under(sim.world(), root, &doc, &c, s, px),
        Some(kids[0]),
        "o clique caiu na esfera da esquerda vista no MUNDO"
    );
}

/// ⚠️ **Quanto custa um clique**, medido — não afirmado.
///
/// A rota escolhida compila **uma árvore por folha**, uma vez por clique. O número abaixo é o que
/// diz se essa rota se aguenta ou se ela precisa de cache; ele está no doc 07 ao lado da alternativa
/// que foi recusada (o *id-buffer*, que espalharia o custo por cada pixel de cada quadro).
#[test]
#[ignore = "medição, não gate — corre com --ignored"]
fn measure_pick_cost() {
    let mut sim = SimWorld::new();
    let doc = three_spheres();
    let world = sim.world_mut();
    let root = ph2d_field_ecs::spawn_doc(world, &doc, "Model");
    let c = front();
    let s = Screen::new(1600, 1000, c.half_extent);
    let (px, _) = c.project([0.0, 0.0, 0.0], s);

    let t0 = std::time::Instant::now();
    const N: u32 = 20;
    for _ in 0..N {
        let _ = node_under(sim.world(), root, &doc, &c, s, px);
    }
    let each = t0.elapsed().as_secs_f64() * 1000.0 / f64::from(N);
    println!("[pick] {each:.2} ms por clique — 3 folhas, quadro de 1600x1000");
}
