//! A suíte do `vec_transform` — **o pivô, o afim e o assentamento**.
//!
//! Arquivo IRMÃO por LOC (HR-18) e módulo FILHO por `#[path]`, então `use super::*` alcança os
//! itens privados exatamente como quando ele morava dentro do pai.

use super::*;
use ph2d_ecs::{ChildOf, Name, VecPathRef};

fn t(tx: f32, ty: f32, scale: f32) -> Transform {
    Transform {
        translation: ph2d_core::Vec2::new(tx, ty),
        scale: ph2d_core::Vec2::new(scale, scale),
        ..Transform::IDENTITY
    }
}

/// **Um caminho com efeitos é PONTO FIXO do assentamento de origens.**
///
/// O undo global regista por DIFF: qualquer sistema que reescreva a cena depois do estado
/// ter assentado produz um passo espúrio, e o 1º Ctrl+Z do artista gasta-se a desfazer o
/// lixo — foi assim que o bug do z-order se manifestou (*"o undo só faz uma etapa"*).
///
/// A pilha de efeitos (ADR-0132) toca este sistema por um caminho indireto: `path_curve_bbox`
/// mede a geometria **cozida**, então a caixa de um caminho aparado é a do pedaço que
/// sobrou, e é essa que decide o centro. Convergir continua a ser obrigatório.
#[test]
fn a_path_with_effects_is_a_fixed_point_of_settling() {
    use ph2d_vec_scene::effect::{FxEntry, PathEffect};
    let mut sim = SimWorld::default();
    let mut map = crate::vec_entities::VecEntityMap::new();
    let mut scene = ph2d_vec_scene::VecScene::new();
    // Longe da origem: um caminho já centrado não exercita o assentamento.
    let mut path = ph2d_vec_scene::VecPath {
        verts: [[100.0, 60.0], [140.0, 60.0], [140.0, 100.0], [100.0, 100.0]]
            .map(ph2d_vec_scene::VecVertex::corner)
            .to_vec(),
        closed: true,
        ..ph2d_vec_scene::VecPath::default()
    };
    // ⚠️ Um REPEATER, e não um Trim: ele é o efeito que MULTIPLICA contornos, então a caixa
    // cozida cresce muito e desloca-se — que é precisamente o que o assentamento lê.
    path.effects = vec![FxEntry::new(PathEffect::Repeat(
        ph2d_vec_scene::fx_repeat::RepeatSpec {
            copies_x: 3.0,
            move_x: 120.0,
            copies_y: 2.0,
            move_y: 40.0,
            spin: 11.0,
            orbit: 7.0,
        },
    ))];
    let id = scene.push_path(path);
    let e = sim
        .world_mut()
        .spawn((Transform::IDENTITY, Name::new("P"), VecPathRef(id)))
        .id();
    map.insert(id, e.to_bits());

    settle_origins(&mut sim, &mut scene, &map, &[]);
    let after_first = (scene.clone(), *sim.world().get::<Transform>(e).unwrap());
    settle_origins(&mut sim, &mut scene, &map, &[]);
    let after_second = (scene.clone(), *sim.world().get::<Transform>(e).unwrap());
    assert!(
        after_first == after_second,
        "assentar duas vezes deu resultados diferentes — cada frame produziria um passo de \
         undo que o artista não pediu"
    );
}

/// Identidade não entra no mapa: o caminho comum (todo path recém-desenhado)
/// não paga nada, e `xform_of` já devolve identidade para o ausente.
#[test]
fn an_untransformed_path_is_absent_from_the_map() {
    let mut sim = SimWorld::default();
    let mut map = crate::vec_entities::VecEntityMap::new();
    let e = sim
        .world_mut()
        .spawn((Transform::IDENTITY, Name::new("P"), VecPathRef(7)))
        .id();
    map.insert(7, e.to_bits());
    assert!(build(&sim, &map).is_empty());
    assert_eq!(
        ph2d_vec_scene::xform_of(&build(&sim, &map), 7),
        Xform::IDENTITY
    );
}

/// A pose de mundo COMPÕE a cadeia de pais — é o que faz um path parentado a um
/// sprite andar junto com ele.
#[test]
fn a_child_path_inherits_the_pose_of_its_parent() {
    let mut sim = SimWorld::default();
    let mut map = crate::vec_entities::VecEntityMap::new();
    let parent = sim
        .world_mut()
        .spawn((t(10.0, 0.0, 2.0), Name::new("S")))
        .id();
    let child = sim
        .world_mut()
        .spawn((
            t(1.0, 1.0, 1.0),
            Name::new("P"),
            VecPathRef(7),
            ChildOf(parent),
        ))
        .id();
    map.insert(7, child.to_bits());

    let x = ph2d_vec_scene::xform_of(&build(&sim, &map), 7);
    // pai escala 2× e translada +10 em x; o filho translada (1,1) no espaço do pai.
    // A origem local do filho cai em (10 + 2·1, 0 + 2·1) = (12, 2).
    let o = x.apply([0.0, 0.0]);
    assert!(
        (o[0] - 12.0).abs() < 1e-6 && (o[1] - 2.0).abs() < 1e-6,
        "{o:?}"
    );
    // E um ponto local [1,0] anda 2 world-units (a escala do pai).
    let p = x.apply([1.0, 0.0]);
    assert!((p[0] - 14.0).abs() < 1e-6, "{p:?}");
}

/// Assentar a origem põe o pivô no CENTRO da forma sem mover um pixel dela.
/// Era o bug: um path nascia com a geometria em coordenadas de mundo e a origem
/// em (0,0) — o centro do mundo.
#[test]
fn settling_puts_the_origin_at_the_shape_center_without_moving_the_shape() {
    let mut sim = SimWorld::default();
    let mut scene = ph2d_vec_scene::VecScene::new();
    let mut map = crate::vec_entities::VecEntityMap::new();
    // Quadrado de [10,20] a [30,40]: centro em (20, 30).
    let id = scene.push_path(ph2d_vec_scene::rectangle([10.0, 20.0], [30.0, 40.0]));
    let e = sim
        .world_mut()
        .spawn((Transform::IDENTITY, VecPathRef(id)))
        .id();
    map.insert(id, e.to_bits());

    let world_before = scene.path_curve_bbox(id).unwrap();
    settle_origins(&mut sim, &mut scene, &map, &[]);

    let t = sim.world().get::<Transform>(e).copied().unwrap();
    assert!(
        (t.translation.x - 20.0).abs() < 1e-4 && (t.translation.y - 30.0).abs() < 1e-4,
        "a origem foi para o centro da forma: {:?}",
        t.translation
    );
    // A geometria recuou o mesmo tanto ⇒ o bbox de MUNDO não mudou.
    let x = ph2d_vec_scene::xform_of(&build(&sim, &map), id);
    let (lo, hi) = scene.path_curve_bbox(id).unwrap();
    let (wlo, whi) = (x.apply(lo), x.apply(hi));
    assert!((wlo[0] - world_before.0[0]).abs() < 1e-4, "{wlo:?}");
    assert!((whi[1] - world_before.1[1]).abs() < 1e-4, "{whi:?}");
    // E a bbox local ficou centrada na origem.
    assert!((lo[0] + hi[0]).abs() < 1e-4 && (lo[1] + hi[1]).abs() < 1e-4);

    // Idempotente: rodar de novo não mexe em nada.
    settle_origins(&mut sim, &mut scene, &map, &[]);
    let t2 = sim.world().get::<Transform>(e).copied().unwrap();
    assert_eq!(t2.translation, t.translation);
}

/// REGRESSÃO (Enio 2026-07-09: "forma sendo desenhada com pivot no centro do mundo
/// e com um offset bizarro em relação ao mouse").
///
/// A ferramenta de forma empurra o path no Down e **reescreve a geometria em
/// coordenadas de MUNDO** a cada Move. Se a origem for assentada no meio do gesto,
/// geometria e `Transform` passam a somar, e a forma sai deslocada exatamente pelo
/// ponto onde o arrasto começou.
#[test]
fn a_shape_still_being_dragged_is_never_settled() {
    let mut sim = SimWorld::default();
    let mut scene = ph2d_vec_scene::VecScene::new();
    let mut map = crate::vec_entities::VecEntityMap::new();
    // O degenerado do Down: um quadrado minúsculo longe da origem.
    let id = scene.push_path(ph2d_vec_scene::rectangle([40.0, 40.0], [40.0, 40.0]));
    let e = sim
        .world_mut()
        .spawn((Transform::IDENTITY, VecPathRef(id)))
        .id();
    map.insert(id, e.to_bits());

    settle_origins(&mut sim, &mut scene, &map, &[id]);
    assert_eq!(
        sim.world().get::<Transform>(e).copied().unwrap(),
        Transform::IDENTITY,
        "o gesto continua em world-space: identidade"
    );
    assert!(build(&sim, &map).is_empty(), "afim identidade: sem offset");

    // Terminado o gesto, aí sim assenta.
    settle_origins(&mut sim, &mut scene, &map, &[]);
    let t = sim.world().get::<Transform>(e).copied().unwrap();
    assert!((t.translation.x - 40.0).abs() < 1e-4);
}

/// **Um CONECTOR nunca é assentado.** A geometria dele é reescrita em MUNDO a
/// cada frame (`connector_live`) — assentar o pivô somaria geometria e
/// `Transform`, e a rota sairia deslocada das duas formas que ela liga. Mesmo
/// motivo do gesto em curso, e da forma viva: geometria DERIVADA não tem pivô a
/// assentar. Ele fica na identidade — e é por isso que o gizmo não o arrasta.
#[test]
fn a_connector_is_never_settled() {
    let mut sim = SimWorld::default();
    let mut scene = ph2d_vec_scene::VecScene::new();
    let mut map = crate::vec_entities::VecEntityMap::new();
    // Uma "rota" longe da origem: assentada, ela ganharia translação (20, 30).
    let id = scene.push_path(ph2d_vec_scene::line([10.0, 20.0], [30.0, 40.0]));
    let e = sim
        .world_mut()
        .spawn((
            Transform::IDENTITY,
            VecPathRef(id),
            ph2d_ecs::VecConnector::between(1, 2),
        ))
        .id();
    map.insert(id, e.to_bits());

    settle_origins(&mut sim, &mut scene, &map, &[]);

    assert_eq!(
        sim.world().get::<Transform>(e).copied().unwrap(),
        Transform::IDENTITY,
        "o conector tem de ficar na IDENTIDADE — a geometria dele ja e mundo"
    );
    assert!(build(&sim, &map).is_empty(), "afim identidade: sem offset");
}

/// O path que a caneta ainda constrói NÃO é assentado — a origem ficaria pulando
/// a cada vértice novo.
#[test]
fn the_path_being_drawn_is_left_alone() {
    let mut sim = SimWorld::default();
    let mut scene = ph2d_vec_scene::VecScene::new();
    let mut map = crate::vec_entities::VecEntityMap::new();
    let id = scene.push_path(ph2d_vec_scene::rectangle([10.0, 20.0], [30.0, 40.0]));
    let e = sim
        .world_mut()
        .spawn((Transform::IDENTITY, VecPathRef(id)))
        .id();
    map.insert(id, e.to_bits());
    settle_origins(&mut sim, &mut scene, &map, &[id]);
    assert_eq!(
        sim.world().get::<Transform>(e).copied().unwrap(),
        Transform::IDENTITY
    );
}

/// `move_origin_to` respeita a escala própria da forma: a geometria local recua
/// o delta DIVIDIDO pela escala, senão a forma saltaria.
#[test]
fn moving_the_origin_of_a_scaled_shape_does_not_move_the_shape() {
    let mut sim = SimWorld::default();
    let mut scene = ph2d_vec_scene::VecScene::new();
    let id = scene.push_path(ph2d_vec_scene::rectangle([-1.0, -1.0], [1.0, 1.0]));
    let e = sim
        .world_mut()
        .spawn((t(0.0, 0.0, 3.0), VecPathRef(id)))
        .id();
    let mut map = crate::vec_entities::VecEntityMap::new();
    map.insert(id, e.to_bits());

    let x0 = ph2d_vec_scene::xform_of(&build(&sim, &map), id);
    let corner_before = x0.apply([1.0, 1.0]); // mundo (3,3)

    assert!(move_origin_to(&mut sim, &mut scene, e, id, [3.0, 3.0]));

    let x1 = ph2d_vec_scene::xform_of(&build(&sim, &map), id);
    let corner_after = x1.apply(scene.paths()[0].verts_all().nth(2).unwrap().anchor);
    assert!(
        (corner_after[0] - corner_before[0]).abs() < 1e-4
            && (corner_after[1] - corner_before[1]).abs() < 1e-4,
        "o canto não se moveu: {corner_before:?} -> {corner_after:?}"
    );
    let tr = sim
        .world()
        .get::<Transform>(e)
        .copied()
        .unwrap()
        .translation;
    assert!(
        (tr.x - 3.0).abs() < 1e-4 && (tr.y - 3.0).abs() < 1e-4,
        "{tr:?}"
    );
}

/// Uma entidade morta não produz afim (a `sync` a removeria, mas o mapa pode
/// estar um frame atrasado).
#[test]
fn a_dead_entity_contributes_nothing() {
    let mut sim = SimWorld::default();
    let mut map = crate::vec_entities::VecEntityMap::new();
    let e = sim
        .world_mut()
        .spawn((t(5.0, 5.0, 1.0), VecPathRef(1)))
        .id();
    map.insert(1, e.to_bits());
    assert_eq!(build(&sim, &map).len(), 1);
    sim.world_mut().despawn(e);
    assert!(build(&sim, &map).is_empty());
}
