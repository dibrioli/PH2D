//! Gates do undo global (captura · restore · canonicalização · a fila). Módulo irmão do
//! [`super`] pelo teto de LOC do shell (HR-18) — segue o idioma `#[path]` do resto da linha.

use super::*;
use ph2d_ecs::scene::{ComponentRegistry, register_ecs_components};
use ph2d_ecs::{Name, VecPathRef};
use ph2d_render::register_render_components;
use ph2d_vec_scene::rectangle;

fn registry() -> ComponentRegistry {
    let mut reg = ComponentRegistry::new();
    register_ecs_components(&mut reg);
    register_render_components(&mut reg);
    reg
}

/// Uma cena de teste: 1 sprite nomeado + 1 forma vetorial com a entidade dela.
fn scene() -> (SimWorld, VecScene) {
    let mut sim = SimWorld::new();
    let mut vec = VecScene::new();
    sim.world_mut().spawn((
        Transform::from_translation(ph2d_core::Vec2::new(3.0, 4.0)),
        Name::new("Spr"),
    ));
    let id = vec.push_path(rectangle([0.0, 0.0], [2.0, 2.0]));
    sim.world_mut()
        .spawn((Transform::default(), Name::new("Path 0"), VecPathRef(id)));
    (sim, vec)
}

fn capture(sim: &mut SimWorld, vec: &VecScene, reg: &ComponentRegistry) -> ProjectState {
    let mut prop = TransformPropagationState::new(sim.world_mut());
    let mut wl = WorklistBuf::new();
    ProjectState::capture(sim, vec, &FlipDoc::new(), reg, &mut prop, &mut wl)
}

/// Capturar → mexer no mundo E na geometria → restaurar devolve o estado exato,
/// e a ponte path↔entidade fica reconstruída (uma entrada por forma).
#[test]
fn capture_then_restore_round_trips_world_and_geometry() {
    let reg = registry();
    let (mut sim, mut vec) = scene();
    let before = capture(&mut sim, &vec, &reg);

    // Muda tudo: move o sprite, apaga a forma, desenha outra.
    {
        let mut q = sim.world_mut().query::<&mut Transform>();
        for mut t in q.iter_mut(sim.world_mut()) {
            t.translation.x += 100.0;
        }
    }
    vec = VecScene::new();
    vec.push_path(rectangle([9.0, 9.0], [10.0, 10.0]));

    let (rvec, map, _rflip, _fmap) = before.restore(&mut sim, &reg);
    vec = rvec;

    // A geometria voltou ao original.
    assert_eq!(vec.paths().len(), 1);
    assert_eq!(vec.paths()[0].verts[0].anchor, [0.0, 0.0]);
    // A ponte tem exatamente a forma restaurada, e aponta uma entidade viva.
    assert_eq!(map.len(), 1);
    let (&pid, &bits) = map.iter().next().unwrap();
    assert_eq!(pid, vec.paths()[0].id);
    assert!(sim.world().get_entity(Entity::from_bits(bits)).is_ok());
    // O sprite voltou à posição original (a captura preservou o Transform).
    let names: Vec<f32> = {
        let mut q = sim.world_mut().query::<(&Name, &Transform)>();
        q.iter(sim.world())
            .filter(|(n, _)| n.as_str() == "Spr")
            .map(|(_, t)| t.translation.x)
            .collect()
    };
    assert_eq!(names, vec![3.0]);
}

/// A ponte reconstruída faz o `sync` seguinte virar NO-OP — não duplica.
#[test]
fn the_rebuilt_bridge_makes_the_next_sync_a_noop() {
    let reg = registry();
    let (mut sim, vec) = scene();
    let snap = capture(&mut sim, &vec, &reg);
    let (mut rvec, mut map, _rflip, _fmap) = snap.restore(&mut sim, &reg);

    let entities_before = map.len();
    crate::vec_entities::sync(&mut sim, &mut rvec, &mut map);
    assert_eq!(map.len(), entities_before, "sync não spawnou nada");
    // Uma entidade por path, e nenhuma órfã (a contagem de VecPathRef == paths).
    let vecref_count = {
        let mut q = sim.world_mut().query::<&VecPathRef>();
        q.iter(sim.world()).count()
    };
    assert_eq!(vecref_count, rvec.paths().len());
}

/// push_undo/undo/redo alternam corretamente (o registro por-diff é testado no
/// nível do App; aqui é a pilha pura).
#[test]
fn push_undo_then_undo_redo_alternate() {
    let reg = registry();
    let (mut sim, vec) = scene();
    let s0 = capture(&mut sim, &vec, &reg);
    let mut stack = ProjectUndo::default();
    assert!(!stack.can_undo() && !stack.can_redo());

    // Um passo: s0 é o estado-pré.
    let mut vec1 = vec.clone();
    vec1.push_path(rectangle([5.0, 5.0], [6.0, 6.0]));
    let s1 = ProjectState {
        world: s0.world.clone(),
        vec: vec1,
        flip: FlipDoc::new(),
    };
    stack.push_undo(s0.clone());
    assert!(stack.can_undo() && !stack.can_redo());

    // Undo devolve o pré (s0) e arma o redo com o atual (s1).
    let back = stack.undo(s1.clone()).unwrap();
    assert_eq!(back, s0);
    assert!(stack.can_redo());
    // Redo devolve s1 e re-arma o undo.
    let fwd = stack.redo(s0.clone()).unwrap();
    assert_eq!(fwd, s1);
    assert!(stack.can_undo());
    // push_undo limpa o redo.
    let _ = stack.undo(s1.clone());
    stack.push_undo(s0.clone());
    assert!(!stack.can_redo());
}
/// O diff de undo depende disto: capturar o MESMO estado duas vezes tem de dar
/// snapshots IGUAIS. Se não, o `post_frame_undo` registraria um passo espúrio a
/// cada frame com input (Enio 2026-07-09: "precisa de mais de 1 Ctrl+Z").
#[test]
fn capturing_the_same_state_twice_is_identical() {
    let reg = registry();
    let (mut sim, vec) = scene();
    let a = capture(&mut sim, &vec, &reg);
    let b = capture(&mut sim, &vec, &reg);
    assert_eq!(a, b, "captura não-determinística -> diff espúrio");
}

/// E depois de um restore (ids de entidade NOVOS), capturar de novo tem de dar o
/// mesmo snapshot — senão o primeiro frame pós-undo registra um passo fantasma.
#[test]
fn capture_after_restore_matches_the_restored_state() {
    let reg = registry();
    let (mut sim, vec) = scene();
    let snap = capture(&mut sim, &vec, &reg);
    let (rvec, _, _, _) = snap.restore(&mut sim, &reg);
    let again = capture(&mut sim, &rvec, &reg);
    assert_eq!(snap, again, "restore->capture != capture original");
}
/// ADR-0114: o `FlipDoc` entra na captura como 3º campo. Round-trip por
/// capture→restore, a ponte objeto↔entidade é reconstruída, e capturar o
/// mesmo estado 2× é idêntico (o flip é determinístico — sem diff espúrio).
#[test]
fn flip_survives_capture_restore_and_rebuilds_bridge() {
    use ph2d_flip::{Hold, KeyKind};
    let reg = registry();
    let mut sim = SimWorld::new();
    let vec = VecScene::new();
    // Documento Flip: um objeto (camada + frame + traço) + a entidade-ponte.
    let mut flip = FlipDoc::new();
    let oid = flip.push_object("Char");
    {
        let obj = flip.object_mut(oid).unwrap();
        let l = obj.add_layer("L");
        let d = obj
            .insert_frame(l, 0, Hold::Implicit, KeyKind::Keyframe)
            .unwrap();
        obj.drawing_mut(d).unwrap().strokes.push(Default::default());
    }
    sim.world_mut().spawn((
        Transform::default(),
        Name::new("Char"),
        ph2d_ecs::FlipObjectRef(oid.0),
    ));

    let snap = {
        let mut prop = TransformPropagationState::new(sim.world_mut());
        let mut wl = WorklistBuf::new();
        ProjectState::capture(&sim, &vec, &flip, &reg, &mut prop, &mut wl)
    };

    // Muda o flip (adiciona objeto) e restaura ao capturado.
    flip.push_object("Extra");
    let (_rvec, _vmap, rflip, fmap) = snap.restore(&mut sim, &reg);
    assert_eq!(rflip.objects().len(), 1, "o flip voltou ao capturado");
    assert_eq!(fmap.len(), 1, "a ponte flip reconstruída tem o objeto");
    let (&fid, &bits) = fmap.iter().next().unwrap();
    assert_eq!(fid, oid);
    assert!(sim.world().get_entity(Entity::from_bits(bits)).is_ok());

    // Capturar o mesmo estado 2× = idêntico (sem passo espúrio de undo).
    let mut prop = TransformPropagationState::new(sim.world_mut());
    let mut wl = WorklistBuf::new();
    let a = ProjectState::capture(&sim, &vec, &rflip, &reg, &mut prop, &mut wl);
    let b = ProjectState::capture(&sim, &vec, &rflip, &reg, &mut prop, &mut wl);
    assert_eq!(a, b, "flip determinístico -> sem diff espúrio");
}

/// REGRESSÃO (Enio 2026-07-09: "ao deletar não volta no mesmo lugar"). O
/// `RootOrder` (a posição de z / linha na hierarquia) tem de sobreviver ao
/// ciclo capturar → deletar → restaurar.
#[test]
fn delete_then_restore_preserves_z_order() {
    use ph2d_ecs::RootOrder;
    let reg = registry();
    let mut sim = SimWorld::new();
    let vec = VecScene::new();
    // Três sprites com ordens de z distintas.
    let a = sim
        .world_mut()
        .spawn((Transform::default(), Name::new("A"), RootOrder(0)))
        .id();
    let b = sim
        .world_mut()
        .spawn((Transform::default(), Name::new("B"), RootOrder(1)))
        .id();
    let _c = sim
        .world_mut()
        .spawn((Transform::default(), Name::new("C"), RootOrder(2)))
        .id();
    let _ = (a, b);

    let snap = capture(&mut sim, &vec, &reg);
    // Deleta o do meio (B).
    sim.world_mut().despawn(b);
    // Restaura o estado pré-delete.
    let _ = snap.restore(&mut sim, &reg);

    // Cada nome volta com o RootOrder original.
    let mut got: Vec<(String, u32)> = {
        let mut q = sim.world_mut().query::<(&Name, &RootOrder)>();
        q.iter(sim.world())
            .map(|(n, r)| (n.as_str().to_owned(), r.0))
            .collect()
    };
    got.sort();
    assert_eq!(
        got,
        vec![("A".into(), 0), ("B".into(), 1), ("C".into(), 2)],
        "o objeto deletado volta na MESMA posição de z"
    );
}
