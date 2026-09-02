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

/// ⭐⭐⭐ **AS DUAS ALÇAS entram na chave** — a lei que o Enio nomeou em 2026-09-01:
///
/// > *"O nó de uma solda é um só para todas as linhas. As alças daquele nó devem servir
/// > simultaneamente para o stroke e para os preenchimentos, senão é impossível que sejam
/// > transformados juntos."*
///
/// ⛔⛔ **A 1.ª redacção deste gate media UMA das duas** (`out_handle`), e o produto tinha
/// exactamente esse buraco: arrastar a alça de ENTRADA de um nó mudava o traço e o preenchimento
/// **não sabia** — a chave do cache não via a mudança, a rede não era refeita, e a área ficava com
/// a curva de antes. *Um gate que mede metade da população aprova a metade que não mediu.*
#[test]
fn the_key_sees_both_handles_move() {
    let base = vec![(vec![v(0.0, 0.0), v(10.0, 0.0)], false)];
    let antes = chave(&base);
    let mut saida = base.clone();
    saida[0].0[0].out_handle = [3.0, 4.0];
    assert_ne!(antes, chave(&saida), "a alca de SAIDA nao entra na chave");
    let mut entrada = base.clone();
    entrada[0].0[1].in_handle = [7.0, -4.0];
    assert_ne!(
        antes,
        chave(&entrada),
        "a alca de ENTRADA nao entra na chave — o traco muda e o preenchimento fica com a curva \
         de antes"
    );
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

/// ⭐⭐⭐ **A ÁREA RE-COZIDA DESCE AO ESPAÇO DO CAMINHO** (report do Enio, 2026-09-01, com foto:
/// *"o preenchimento está nascendo deslocado para fora do stroke"*).
///
/// ⚠️ A rede fala MUNDO; o documento guarda LOCAL. Depois de o `settle_origins` mudar a origem da
/// entidade para o centro da caixa dela, escrever mundo naquele `VecPath` desloca-o **pelo centro
/// dele** — e era por isso que cada área saía com um desvio DIFERENTE.
#[test]
fn the_recooked_area_comes_down_to_the_paths_own_space() {
    let mundo = vec![v(100.0, 50.0), v(140.0, 50.0), v(140.0, 90.0)];
    // A pose que o assentamento deixa: a origem no centro da caixa.
    let xf = ph2d_vec_scene::Xform([1.0, 0.0, 0.0, 1.0, 120.0, 70.0]);
    let local = para_local(mundo.clone(), &xf).expect("a pose e' invertivel");
    assert_eq!(
        local[0].anchor,
        [-20.0, -20.0],
        "a area nao desceu ao local"
    );
    // …e de volta ao mundo pela pose, ela está EXACTAMENTE onde a rede a pôs.
    for (l, m) in local.iter().zip(&mundo) {
        assert_eq!(xf.apply(l.anchor), m.anchor, "o ida-e-volta nao fecha");
    }
    // ⚠️ Identidade é o caminho comum (a forma acabada de nascer) e não paga nada.
    assert_eq!(
        para_local(mundo.clone(), &ph2d_vec_scene::Xform::IDENTITY),
        Some(mundo)
    );
}

/// ⛔⛔ **O preenchimento é DERIVADO, e a vista di-lo** — a outra metade da lei do Enio
/// (2026-09-01). Sem esta linha, os nós dele desenham-se e agarram-se: um segundo conjunto de
/// alças empilhado sobre o das linhas que o produzem.
#[test]
fn a_bucket_fill_is_published_as_derived_and_is_not_pickable() {
    use ph2d_ecs::{Name, SimWorld, Transform, VecBucketFill, VecPathRef};
    let mut sim = SimWorld::default();
    let mut map = crate::vec_entities::VecEntityMap::new();
    let (parede, area) = (7u64, 9u64);
    for (id, fill) in [(parede, false), (area, true)] {
        let mut e = sim
            .world_mut()
            .spawn((Transform::IDENTITY, Name::new("P"), VecPathRef(id)));
        if fill {
            e.insert(VecBucketFill::new([0.0, 0.0]));
        }
        map.insert(id, e.id().to_bits());
    }
    let view = crate::vec_entities::view_state(&sim, &map);
    assert!(
        view.is_derived(area),
        "o preenchimento nao foi publicado como derivado"
    );
    assert!(!view.is_derived(parede), "a parede nao pode ser derivada");
    assert!(
        !view.is_pickable(area),
        "os nos do preenchimento continuam agarraveis"
    );
    assert!(view.is_pickable(parede), "a parede deixou de ser agarravel");
}
