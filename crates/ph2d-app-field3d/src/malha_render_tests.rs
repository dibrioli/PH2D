//! A peça → os objetos do Render (ordem do dono, 02/10): o que se toca funde, o que está solto
//! separa, e só é MÓVEL o objeto cujas unidades não aparecem em mais nenhum.

use bevy_ecs::entity::Entity;
use ph2d_ecs::SimWorld;
use ph2d_field::{Blend, FieldDoc, Node, NodeId, NodeKind, Op, Primitive, Xform};

fn ball(x: f32, y: f32, r: f32) -> Node {
    Node::new(Xform::at(x, y, 0.0), NodeKind::Leaf(Primitive::Sphere { radius: r }))
}

fn op(o: Op, kids: &[u32]) -> Node {
    Node::new(
        Xform::IDENTITY,
        NodeKind::Combine {
            op: o,
            children: kids.iter().copied().map(NodeId).collect(),
        },
    )
}

fn scene(doc: &FieldDoc) -> (SimWorld, Entity) {
    let mut sim = SimWorld::new();
    crate::scene::sync_scene(&mut sim, Some(doc), 0.0);
    let world = sim.world_mut();
    let mut q = world.query::<(Entity, &ph2d_field_ecs::FieldObject)>();
    let root = q.iter(world).next().map(|(e, _)| e).expect("a peça");
    (sim, root)
}

fn objetos(doc: &FieldDoc) -> (SimWorld, Vec<super::ObjetoRender>) {
    let (sim, root) = scene(doc);
    let o = super::extrair(sim.world(), root, &crate::smoke::sampled_registry());
    (sim, o)
}

/// ⭐ A cena 28 do smoke: quatro nós de toro — e eles TOCAM-SE. Medido (02/10) com o campo de cada
/// um sobre os vértices do outro: o 2 entra `0,0099` no 1 e `0,0087` no 3; o 0 fica a `0,0020` do 1
/// sem tocar. ⇒ pela regra do dono são DOIS objetos: o 0 sozinho, e o 1+2+3 fundidos.
#[test]
fn the_knots_of_scene_28_that_touch_are_one_object() {
    let doc = crate::smoke::scenes::scene(28);
    let (_, o) = objetos(&doc);
    let mut tamanhos: Vec<usize> = o.iter().map(|x| x.unidades.len()).collect();
    tamanhos.sort_unstable();
    assert_eq!(tamanhos, vec![1, 3]);
    assert!(o.iter().all(|x| x.movel));
    assert!(o.iter().all(|x| x.malha.triangulos() > 1000));
}

/// Duas bolas sobrepostas numa união são UM objeto, que move as duas.
#[test]
fn two_overlapping_balls_are_one_object_that_moves_both() {
    let doc = FieldDoc::new(
        vec![
            ball(-0.15, 0.0, 0.3),
            ball(0.15, 0.0, 0.3),
            ball(1.5, 0.0, 0.3),
            op(Op::Union(Blend::Sharp), &[0, 1, 2]),
        ],
        NodeId(3),
    )
    .expect("doc");
    let (_, o) = objetos(&doc);
    assert_eq!(o.len(), 2);
    let mut tamanhos: Vec<usize> = o.iter().map(|x| x.unidades.len()).collect();
    tamanhos.sort_unstable();
    assert_eq!(tamanhos, vec![1, 2]);
    assert!(o.iter().all(|x| x.movel));
}

/// Uma bola cortada ao meio por uma laje: dois objetos que partilham a MESMA unidade (a diferença)
/// — nenhum dos dois se move sozinho.
#[test]
fn the_halves_of_a_cut_are_two_objects_that_cannot_move_alone() {
    let doc = FieldDoc::new(
        vec![
            ball(0.0, 0.0, 0.5),
            Node::new(
                Xform::IDENTITY,
                NodeKind::Leaf(Primitive::Box {
                    half: [0.08, 1.0, 1.0],
                    round: 0.0,
                    chamfer: 0.0,
                }),
            ),
            op(Op::Difference(Blend::Sharp), &[0, 1]),
            ball(2.0, 0.0, 0.3),
            op(Op::Union(Blend::Sharp), &[2, 3]),
        ],
        NodeId(4),
    )
    .expect("doc");
    let (_, o) = objetos(&doc);
    assert_eq!(o.len(), 3, "as duas metades e a bola solta");
    assert_eq!(o.iter().filter(|x| !x.movel).count(), 2);
    assert_eq!(o.iter().filter(|x| x.movel).count(), 1);
}

/// ⭐ Cada triângulo tem a cor da folha dele: duas bolas sobrepostas com materiais diferentes dão um
/// objeto com DOIS índices de material.
#[test]
fn a_fused_object_keeps_the_material_of_each_leaf() {
    let doc = FieldDoc::new(
        vec![
            ball(-0.15, 0.0, 0.3),
            ball(0.15, 0.0, 0.3),
            op(Op::Union(Blend::Sharp), &[0, 1]),
        ],
        NodeId(2),
    )
    .expect("doc");
    let (_, o) = objetos(&doc);
    assert_eq!(o.len(), 1);
    let mut m = o[0].malha.material.clone();
    m.sort_unstable();
    m.dedup();
    assert_eq!(m, vec![0, 1]);
}

/// A malha cola ao campo: todo vértice a menos de uma célula da superfície, e a normal aponta para
/// fora (a mesma direção do gradiente).
#[test]
fn the_mesh_sits_on_the_field_and_faces_outwards() {
    let doc = FieldDoc::new(vec![ball(0.0, 0.0, 0.5)], NodeId(0)).expect("doc");
    let (_, o) = objetos(&doc);
    assert_eq!(o.len(), 1);
    let m = &o[0].malha;
    let cell = ph2d_field_eval::extract::cell_size(
        &doc,
        &crate::smoke::sampled_registry(),
        super::DEPTH,
    ) as f32;
    for (p, n) in m.posicoes.iter().zip(&m.normais) {
        let r = (p[0] * p[0] + p[1] * p[1] + p[2] * p[2]).sqrt();
        assert!((r - 0.5).abs() < 0.05 * cell, "vértice fora da superfície: {r}");
        let d = (p[0] * n[0] + p[1] * n[1] + p[2] * n[2]) / r;
        assert!(d > 0.99, "normal para dentro ou torta: {d}");
    }
    assert!(m.ao.iter().all(|a| *a > 0.95), "uma bola sozinha não se tapa");
}

/// A quina de uma caixa é VIVA: o vértice dela parte-se em três normais.
#[test]
fn a_box_corner_keeps_three_sharp_normals() {
    let doc = FieldDoc::new(
        vec![Node::new(
            Xform::IDENTITY,
            NodeKind::Leaf(Primitive::Box {
                half: [0.4, 0.3, 0.2],
                round: 0.0,
                chamfer: 0.0,
            }),
        )],
        NodeId(0),
    )
    .expect("doc");
    let (_, o) = objetos(&doc);
    let m = &o[0].malha;
    // Toda normal da caixa é um eixo: a aresta viva não mistura faces.
    for n in &m.normais {
        let max = n.iter().fold(0.0f32, |a, c| a.max(c.abs()));
        assert!(max > 0.999, "normal misturada numa caixa: {n:?}");
    }
}

/// ⭐ O delta de desenho: mover a unidade depois da extração desloca a malha pelo MESMO gesto.
#[test]
fn the_draw_pose_is_the_move_since_extraction() {
    let antes = Xform {
        translation: [0.3, -0.2, 0.1],
        rotation: [0.0, 0.38268343, 0.0, 0.9238795],
        scale: 1.5,
    };
    let depois = Xform {
        translation: [1.0, 0.5, -0.4],
        ..antes
    };
    let d = super::delta(depois, antes);
    let p_local = [0.2, 0.1, -0.3];
    let em_antes = antes.apply(p_local);
    let movido = d.apply(em_antes);
    let esperado = depois.apply(p_local);
    for k in 0..3 {
        assert!((movido[k] - esperado[k]).abs() < 1e-5, "{movido:?} vs {esperado:?}");
    }
}


/// ⏱ SONDA (ignorada): o preço de ENTRAR no Render — extrair a peça em objetos — e quantos triângulos
/// saem. `cargo test --release -p ph2d-app-field3d --lib sonda_entrar_no_render -- --ignored --nocapture`
#[test]
#[ignore = "sonda de relógio — corre em release, à mão"]
fn sonda_entrar_no_render() {
    let reg = crate::smoke::sampled_registry();
    for n in [28u32, 29, 30, 36] {
        let doc = crate::smoke::scenes::scene(n);
        let (sim, root) = scene(&doc);
        let mut tempos = Vec::new();
        let mut o = Vec::new();
        for _ in 0..3 {
            let t = std::time::Instant::now();
            o = super::extrair(sim.world(), root, &reg);
            tempos.push(t.elapsed().as_secs_f64() * 1e3);
        }
        tempos.sort_by(f64::total_cmp);
        let tris: usize = o.iter().map(|x| x.malha.triangulos()).sum();
        let verts: usize = o.iter().map(|x| x.malha.posicoes.len()).sum();
        eprintln!(
            "SONDA cena {n}: {} objetos · {tris} triângulos · {verts} vértices · mediana {:.1} ms (min {:.1})",
            o.len(),
            tempos[1],
            tempos[0]
        );
    }
}

