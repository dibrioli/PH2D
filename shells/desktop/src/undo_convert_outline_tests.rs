//! «Convert to Collision Outline» pela fila de undo real: o quadro da conversão (a porta que a
//! `fase_convert_outline` chama + o `sync` dos vectores) é UM passo; o Ctrl+Z devolve o desenho — nome,
//! pai, vaga, `Transform` e o MESMO documento vectorial — e tira a forma; o redo converte de novo.

use super::tests::{capture, registry};
use super::*;
use ph2d_ecs::{ChildOf, Entity, Name, VecPathRef, ordered_children};
use ph2d_physics_ecs::{BodyKind, ColliderOutline, RigidBody};
use ph2d_vec_scene::{Contour, FillRule, Paint, Rgba8, rectangle};

fn por_nome(sim: &mut SimWorld, n: &str) -> Option<Entity> {
    let mut q = sim.world_mut().query::<(Entity, &Name)>();
    q.iter(sim.world())
        .find(|(_, x)| x.as_str() == n)
        .map(|(e, _)| e)
}

fn nomes(sim: &mut SimWorld, pai: Entity) -> Vec<String> {
    ordered_children(sim.world(), pai)
        .into_iter()
        .map(|c| sim.world().get::<Name>(c).map_or_else(String::new, |n| n.as_str().to_owned()))
        .collect()
}

/// *Mutações: o path sai do documento só no `sync` do quadro SEGUINTE ⇒ dois passos; o restauro sem o
/// documento vectorial ⇒ o desenho volta sem o path; sem refazer o mapa ⇒ o `sync` cria um gémeo.*
#[test]
fn converter_o_desenho_e_um_passo_e_o_ctrl_z_devolve_o_vetor() {
    let mut reg = registry();
    ph2d_physics_ecs::register_physics_components(&mut reg);
    let mut sim = SimWorld::new();
    let mut vec = VecScene::new();
    let mut mapa = ph2d_vec_entities::entities::VecEntityMap::default();
    let tinta = Rgba8 {
        r: 20,
        g: 20,
        b: 20,
        a: 255,
    };
    let id = vec.push_path(ph2d_vec_scene::VecPath {
        fill: Some(Paint::solid(tinta)),
        subpaths: vec![Contour {
            verts: rectangle([-1.0, -1.0], [1.0, 1.0]).verts,
            closed: true,
        }],
        fill_rule: FillRule::EvenOdd,
        ..rectangle([-2.0, -2.0], [2.0, 2.0])
    });
    let w = sim.world_mut();
    let corpo = w
        .spawn((
            Name::new("Body"),
            Transform::default(),
            RigidBody {
                kind: BodyKind::Static,
            },
        ))
        .id();
    let a = w.spawn((Name::new("a"), Transform::default(), ChildOf(corpo))).id();
    let pose = Transform::from_translation(ph2d_core::Vec2::new(0.5, 1.0));
    let v = w
        .spawn((Name::new("Frame"), pose, VecPathRef(id), ChildOf(corpo)))
        .id();
    let b = w.spawn((Name::new("b"), Transform::default(), ChildOf(corpo))).id();
    ph2d_ecs::set_sibling_order(w, corpo, &[a, v, b]);
    mapa.insert(id, v.to_bits());
    ph2d_ecs::assign_missing_stable_ids(sim.world_mut());
    ph2d_vec_entities::entities::sync(&mut sim, &mut vec, &mut mapa);
    let vec_antes = vec.clone();
    let antes = capture(&mut sim, &vec, &reg);
    let mut base = antes.clone();
    let mut fila = ProjectUndo::default();
    let quadro = |sim: &mut SimWorld,
                  vec: &mut VecScene,
                  mapa: &mut ph2d_vec_entities::entities::VecEntityMap,
                  fila: &mut ProjectUndo,
                  base: &mut ProjectState| {
        ph2d_vec_entities::entities::sync(sim, vec, mapa);
        let agora = capture(sim, vec, &reg);
        if *base != agora {
            fila.push_undo(std::mem::replace(base, agora), SelectionMark::default());
        }
    };
    // O quadro do clique: a porta da fase, depois o fim de quadro; e um quadro parado a seguir.
    let forma = ph2d_physics_outline_vec::convert(sim.world_mut(), &mut vec, v).expect("converte");
    quadro(&mut sim, &mut vec, &mut mapa, &mut fila, &mut base);
    quadro(&mut sim, &mut vec, &mut mapa, &mut fila, &mut base);
    assert_eq!(fila.depth(), 1, "converter tem de ser UM passo");
    assert!(sim.world().get::<ColliderOutline>(forma).is_some());
    assert_eq!(nomes(&mut sim, corpo), ["a", "Collision Shape", "b"]);
    let depois = base.clone();

    let (volta, _) = fila.undo(depois.clone(), SelectionMark::default()).expect("há um passo");
    assert_eq!(volta, antes, "o Ctrl+Z devolve o estado de antes, documento vectorial incluído");
    let (rvec, rmapa, _, _) = volta.restore(&mut sim, &reg);
    (vec, mapa) = (rvec, rmapa);
    assert_eq!(vec, vec_antes, "o MESMO documento vectorial");
    let corpo = por_nome(&mut sim, "Body").expect("corpo");
    let f = por_nome(&mut sim, "Frame").expect("o desenho voltou");
    assert!(por_nome(&mut sim, "Collision Shape").is_none(), "a forma saiu");
    assert_eq!(nomes(&mut sim, corpo), ["a", "Frame", "b"], "a mesma vaga");
    assert_eq!(sim.world().get::<ChildOf>(f).map(|c| c.0), Some(corpo));
    assert_eq!(sim.world().get::<Transform>(f), Some(&pose));
    let r = sim.world().get::<VecPathRef>(f).copied().expect("VecPathRef");
    assert_eq!(vec.path(r.0), vec_antes.path(id), "o mesmo path");
    ph2d_vec_entities::entities::sync(&mut sim, &mut vec, &mut mapa);
    assert_eq!(vec, vec_antes, "o sync não desfaz o restauro");
    let refs = {
        let mut q = sim.world_mut().query::<&VecPathRef>();
        q.iter(sim.world()).count()
    };
    assert_eq!(refs, 1, "UM objecto para o path (o mapa refeito; sem gémeo do sync)");

    let (refeito, _) = fila.redo(volta, SelectionMark::default()).expect("redo");
    assert_eq!(refeito, depois, "o redo converte de novo");
    let (rvec, _, _, _) = refeito.restore(&mut sim, &reg);
    assert!(rvec.is_empty());
    assert!(por_nome(&mut sim, "Collision Shape").is_some());
    assert!(por_nome(&mut sim, "Frame").is_none());
}
