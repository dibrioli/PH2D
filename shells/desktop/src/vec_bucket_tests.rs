//! Gates da COSTURA do **BALDE** — o que a lei pura não alcança: a cena, a pose e o cache.

use super::*;

fn v(x: f64, y: f64) -> VecVertex {
    VecVertex {
        anchor: [x, y],
        in_handle: [x, y],
        out_handle: [x, y],
        kind: ph2d_vec_scene::VertexKind::Corner,
        corner_radius: 0.0,
    }
}

/// ⚠️ **A chave é o CONTEÚDO, não a contagem.** Mover uma forma não muda quantas há, e um cache
/// que não visse isso acenderia uma face onde já não há linha nenhuma.
#[test]
fn the_cache_key_follows_the_geometry_not_the_count() {
    let a = vec![(vec![v(0.0, 0.0), v(10.0, 0.0)], false)];
    let b = vec![(vec![v(0.0, 0.0), v(10.0, 0.5)], false)];
    assert_ne!(chave(&a), chave(&b), "mover uma ponta nao mudou a chave");
    assert_eq!(chave(&a), chave(&a.clone()), "a chave tem de ser estavel");
}

/// ⚠️ **A ALÇA entra na chave.** Um param de forma pode mover só as alças (a curva muda, as âncoras
/// não) — e a face muda com ela.
#[test]
fn the_key_sees_a_handle_move() {
    let mut a = vec![(vec![v(0.0, 0.0), v(10.0, 0.0)], false)];
    let antes = chave(&a);
    a[0].0[0].out_handle = [3.0, 4.0];
    assert_ne!(antes, chave(&a));
}

/// ⛔ **Um caminho ESCONDIDO não cerca nada** — ele estaria a preencher contra uma parede
/// invisível.
#[test]
fn a_hidden_path_is_not_a_wall() {
    let mut scene = VecScene::new();
    let id = scene.push_path(VecPath {
        verts: vec![v(0.0, 0.0), v(10.0, 0.0)],
        ..VecPath::default()
    });
    let todos = contornos_mundo(&scene, &VecXforms::new(), &|_| false);
    assert_eq!(todos.len(), 1);
    let nenhum = contornos_mundo(&scene, &VecXforms::new(), &|x| x == id);
    assert!(nenhum.is_empty(), "o escondido entrou na rede");
}

/// ⚠️⚠️ **A POSE entra na conta**: dois traços só se cruzam depois de o `Transform` os pôr no
/// lugar, e medir na geometria local diria que eles não se encontram.
#[test]
fn the_contours_come_out_in_world_space() {
    let mut scene = VecScene::new();
    let id = scene.push_path(VecPath {
        verts: vec![v(0.0, 0.0), v(10.0, 0.0)],
        ..VecPath::default()
    });
    let mut xf = VecXforms::new();
    xf.insert(id, ph2d_vec_scene::Xform([1.0, 0.0, 0.0, 1.0, 100.0, 0.0]));
    let c = contornos_mundo(&scene, &xf, &|_| false);
    assert_eq!(c[0].0[0].anchor, [100.0, 0.0], "a pose nao entrou");
}

/// ⛔⛔ **UM PREENCHIMENTO NÃO É PAREDE** — a metade de CENA da lei.
///
/// Report do Enio (2026-09-01): *"ao usar o balde nas áreas coloridas, ele para de funcionar nas
/// áreas não coloridas."* O mecanismo (arestas coincidentes envenenam o passeio) está medido na
/// lei pura; aqui prova-se que a shell **reconhece** o preenchimento e o mantém fora.
#[test]
fn a_bucket_fill_is_not_a_wall() {
    use ph2d_ecs::{Name, SimWorld, Transform, VecBucketFill, VecPathRef};
    let mut sim = SimWorld::default();
    let mut map = crate::vec_entities::VecEntityMap::new();
    let mut scene = VecScene::new();
    let parede = scene.push_path(VecPath {
        verts: vec![v(0.0, 0.0), v(10.0, 0.0)],
        ..VecPath::default()
    });
    let area = scene.push_path(VecPath {
        verts: vec![v(0.0, 0.0), v(10.0, 0.0), v(10.0, 10.0)],
        closed: true,
        ..VecPath::default()
    });
    for (id, fill) in [(parede, false), (area, true)] {
        let mut e = sim
            .world_mut()
            .spawn((Transform::IDENTITY, Name::new("P"), VecPathRef(id)));
        if fill {
            e.insert(VecBucketFill::new([5.0, 2.0]));
        }
        map.insert(id, e.id().to_bits());
    }
    let fills = preenchimentos(&sim, &map);
    assert_eq!(fills.len(), 1, "a shell nao reconheceu o preenchimento");
    assert_eq!(fills[0].0, area);
    assert_eq!(fills[0].2, [5.0, 2.0], "a semente nao voltou inteira");

    // ⚠️ **A MESMA porta que o produto usa** (`fora_da_rede`) — um fecho escrito aqui testaria o
    // fecho deste teste, e foi assim que a mutação que apagava o termo do preenchimento sobreviveu.
    let so_fill: std::collections::BTreeSet<u64> = fills.iter().map(|(id, _, _)| *id).collect();
    let paredes = contornos_mundo(&scene, &VecXforms::new(), &|id| {
        fora_da_rede(false, so_fill.contains(&id))
    });
    assert!(
        fora_da_rede(false, true),
        "um preenchimento tem de ficar FORA"
    );
    assert!(fora_da_rede(true, false), "um escondido tem de ficar FORA");
    assert!(!fora_da_rede(false, false), "uma linha comum e' PAREDE");
    assert_eq!(
        paredes.len(),
        1,
        "o preenchimento entrou na rede como parede"
    );
    assert!(
        !paredes[0].1,
        "a parede que sobrou tem de ser a linha ABERTA"
    );
}
