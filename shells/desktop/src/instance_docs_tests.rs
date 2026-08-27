//! Os gates da clonagem de documentos possuídos (ADR-0164 / F4.6).
//!
//! ⚠️ **O oráculo é a GEOMETRIA da cópia, e nunca «o componente existe»**: um `VecPathRef` que
//! aponte para o path do original passa em qualquer teste de presença e é o defeito que a F4.2
//! evitou de propósito (duas entidades a escrever no mesmo documento).

use super::{DROPPED, OwnedDocs, clone_owned_documents};
use ph2d_ecs::{ChildOf, Entity, MasterRoot, Name, SimWorld, Transform, VecPathRef};
use ph2d_vec_scene::{VecPathId, VecScene, rectangle};

use crate::vec_entities::VecEntityMap;

fn reg() -> ph2d_ecs::scene::ComponentRegistry {
    crate::init::build_component_registry()
}

/// Uma receita cuja peça é uma FORMA VETORIAL: raiz + um filho com `VecPathRef`.
fn master_with_a_vector_piece(
    sim: &mut SimWorld,
    scene: &mut VecScene,
    map: &mut VecEntityMap,
) -> (Entity, VecPathId) {
    let id = scene.push_path(rectangle([-1.0, -1.0], [1.0, 1.0]));
    let root = sim
        .world_mut()
        .spawn((Transform::IDENTITY, Name::new("Badge"), MasterRoot))
        .id();
    let piece = sim
        .world_mut()
        .spawn((
            Transform::from_translation(ph2d_core::Vec2::new(0.5, 0.0)),
            Name::new("Plate"),
            VecPathRef(id),
            ChildOf(root),
        ))
        .id();
    map.insert(id, piece.to_bits());
    ph2d_ecs::assign_master_pieces(sim.world_mut());
    (root, id)
}

/// A peça da instância chamada `name`.
fn piece(sim: &SimWorld, root: Entity, name: &str) -> Entity {
    let mut stack = vec![root];
    while let Some(e) = stack.pop() {
        if e != root && sim.world().get::<Name>(e).is_some_and(|n| n.0 == name) {
            return e;
        }
        if let Some(kids) = sim.world().get::<ph2d_ecs::Children>(e) {
            stack.extend(kids.iter().copied());
        }
    }
    panic!("nao ha' peca chamada {name:?}");
}

/// ⭐⭐⭐ **Uma peça vetorial da instância tem GEOMETRIA PRÓPRIA** (doc 04 §2.9).
///
/// ⛔ Antes desta fatia ela nascia **sem `VecPathRef` nenhum** — uma linha na Hierarquia que não
/// desenha um pixel, porque a cópia profunda salta os documentos possuídos.
///
/// ⚠️ Os **dois** lados: o id é OUTRO (senão as duas entidades escreveriam no mesmo path) e a
/// forma é a MESMA (senão a cópia não é uma cópia).
///
/// (Mutação: apagar a chamada a `clone_owned_documents` ⇒ RED na ausência.)
#[test]
fn a_vector_piece_of_an_instance_gets_its_own_path() {
    let mut sim = SimWorld::new();
    let mut scene = VecScene::new();
    let mut map = VecEntityMap::new();
    let r = reg();
    let (master, main_id) = master_with_a_vector_piece(&mut sim, &mut scene, &mut map);
    let inst = crate::instantiate::instantiate_master(
        &mut sim,
        &r,
        master,
        None,
        &mut OwnedDocs {
            vec_scene: &mut scene,
            vec_entities: &mut map,
        },
    )
    .expect("instanciou");

    let copy_ref = sim
        .world()
        .get::<VecPathRef>(piece(&sim, inst, "Plate"))
        .copied()
        .expect("a peca da instancia nasceu SEM geometria");
    assert_ne!(
        copy_ref.0, main_id,
        "a peca da copia aponta para o path do MESTRE — duas entidades a escrever no mesmo documento"
    );
    let (a, b) = (
        scene.path(main_id).expect("o path do mestre"),
        scene.path(copy_ref.0).expect("o path da copia"),
    );
    assert_eq!(a.verts, b.verts, "a copia nao levou a forma do mestre");
    assert_eq!(a.fill, b.fill, "a copia nao levou o preenchimento");
}

/// ⚠️⚠️ **O par `path ⟺ entidade` é REGISTADO**, senão o `vec_entities::sync` cunha uma segunda
/// entidade para o clone: a arte aparece duas vezes na Hierarquia e uma delas é inalcançável.
///
/// ⚠️ O oráculo é o **sync a correr**, e não o mapa: um gate sobre o mapa mediria a marca em vez do
/// fim (a lição de 26/08).
///
/// (Mutação: apagar o `docs.vec_entities.insert(...)` ⇒ RED com a entidade a mais.)
#[test]
fn the_clone_is_registered_so_the_sync_mints_no_ghost() {
    let mut sim = SimWorld::new();
    let mut scene = VecScene::new();
    let mut map = VecEntityMap::new();
    let r = reg();
    let (master, _) = master_with_a_vector_piece(&mut sim, &mut scene, &mut map);
    crate::instantiate::instantiate_master(
        &mut sim,
        &r,
        master,
        None,
        &mut OwnedDocs {
            vec_scene: &mut scene,
            vec_entities: &mut map,
        },
    )
    .expect("instanciou");

    let before = sim.world().entities().len();
    crate::vec_entities::sync(&mut sim, &mut scene, &mut map);
    assert_eq!(
        sim.world().entities().len(),
        before,
        "o sync cunhou uma entidade fantasma para o path clonado"
    );
}

/// ⚠️ **O clone entra SEM deslocamento** — a geometria de um `VecPath` é LOCAL, e quem põe a peça
/// no sítio é o `Transform` que a cópia profunda já levou verbatim.
///
/// ⛔ **A 1.ª versão deste gate comparava `subpaths` e a mutação SOBREVIVEU:** `translate_path`
/// mexe nos **vértices**, e `subpaths` é a estrutura dos contornos. *Comparar a estrutura não é
/// comparar a geometria* — e o gate ficava verde sobre uma arte deslocada.
///
/// ⛔ Um `paste_clip` com offset (a porta do *Duplicate* de canvas) moveria a arte DENTRO da cópia.
///
/// (Mutação: `translate_path(new_id, 8.0, 8.0)` depois do `push_path` ⇒ RED.)
#[test]
fn the_clone_carries_no_offset_because_the_pose_is_in_the_transform() {
    let mut sim = SimWorld::new();
    let mut scene = VecScene::new();
    let mut map = VecEntityMap::new();
    let r = reg();
    let (master, main_id) = master_with_a_vector_piece(&mut sim, &mut scene, &mut map);
    let inst = crate::instantiate::instantiate_master(
        &mut sim,
        &r,
        master,
        None,
        &mut OwnedDocs {
            vec_scene: &mut scene,
            vec_entities: &mut map,
        },
    )
    .expect("instanciou");
    let copy_piece = piece(&sim, inst, "Plate");
    let copy_id = sim
        .world()
        .get::<VecPathRef>(copy_piece)
        .expect("geometria")
        .0;
    assert_eq!(
        scene.path(main_id).expect("mestre").verts,
        scene.path(copy_id).expect("copia").verts,
        "o clone entrou deslocado — a arte da copia sai fora do sitio da irma"
    );
    // E o `Transform` é que carrega o lugar, copiado verbatim.
    assert_eq!(
        sim.world()
            .get::<Transform>(copy_piece)
            .expect("pose")
            .translation,
        ph2d_core::Vec2::new(0.5, 0.0),
        "a peca perdeu a pose que a receita lhe deu"
    );
}

/// ⛔⛔ **CENSO de dois lados: todo documento possuído é clonado OU declarado dropado.**
///
/// ⚠️ Um bridge novo que não venha a esta decisão nasce **mudo** — a cópia dele desaparece sem
/// nada na tela dizer porquê, que é a forma de defeito mais cara deste subsistema.
///
/// (Mutação: tirar `FlipObjectRef` do `DROPPED` ⇒ RED a nomeá-lo.)
#[test]
fn every_owned_document_is_cloned_or_declared_dropped() {
    /// Os que esta porta de facto clona.
    const CLONED: &[&str] = &["ph2d::ecs::VecPathRef"];
    let r = reg();
    let owned: Vec<&str> = r
        .iter()
        .filter(|e| e.desc.is_some_and(|d| d.owned_document))
        .map(|e| e.canonical_name)
        .collect();
    assert!(
        !owned.is_empty(),
        "o registo deixou de ter documentos possuidos — o censo nao mede nada"
    );
    for name in &owned {
        assert!(
            CLONED.contains(name) || DROPPED.iter().any(|(n, _)| n == name),
            "o documento possuido `{name}` nao esta' nem em CLONED nem em DROPPED — a copia dele \
             desaparece em silencio"
        );
    }
    // E o outro lado: nada na lista de dropados deixou de ser um documento possuído.
    for (name, _) in DROPPED {
        assert!(
            owned.contains(name),
            "`{name}` esta' declarado dropado e ja' nao e' um documento possuido — a nota envelheceu"
        );
    }
}

/// **Uma cópia sem documento nenhum não faz nada** — o controlo que impede o gate acima de ficar
/// verde por uma travessia que nunca corre.
#[test]
fn a_copy_without_documents_clones_nothing() {
    let mut sim = SimWorld::new();
    let mut scene = VecScene::new();
    let mut map = VecEntityMap::new();
    let r = reg();
    let root = sim
        .world_mut()
        .spawn((Transform::IDENTITY, Name::new("Plain")))
        .id();
    let copy = ph2d_ecs::deep_copy_subtree(sim.world_mut(), &r, root, None).expect("copiou");
    assert_eq!(
        clone_owned_documents(
            &mut sim,
            &r,
            &mut OwnedDocs {
                vec_scene: &mut scene,
                vec_entities: &mut map,
            },
            &copy,
        ),
        super::DocReport::default()
    );
}

/// ⭐⭐ **O que a cópia DEIXA CAIR é nomeado, não silencioso.**
///
/// ⚠️ *Um importador que ignora em silêncio é pior que um que recusa* (a lei do `.ase`): uma
/// sprite pintada que perde as camadas ao ser copiada não tem nada na tela a dizer porquê. O
/// relatório é o que o chamador transforma numa linha de log.
///
/// (Mutação: devolver `dropped` sempre vazio ⇒ RED.)
#[test]
fn a_dropped_document_is_named_in_the_report() {
    let mut sim = SimWorld::new();
    let mut scene = VecScene::new();
    let mut map = VecEntityMap::new();
    let r = reg();
    let src = sim
        .world_mut()
        .spawn((
            Transform::IDENTITY,
            Name::new("Painted"),
            ph2d_ecs::PaintedDoc(7),
        ))
        .id();
    let copy = ph2d_ecs::deep_copy_subtree(sim.world_mut(), &r, src, None).expect("copiou");
    let report = clone_owned_documents(
        &mut sim,
        &r,
        &mut OwnedDocs {
            vec_scene: &mut scene,
            vec_entities: &mut map,
        },
        &copy,
    );
    assert_eq!(
        report.dropped,
        vec!["ph2d::ecs::PaintedDoc"],
        "a copia perdeu o documento do Painter em SILENCIO"
    );
    // ⚠️ O controlo: uma entidade SEM aquele componente não o nomeia (senão o relatório mentiria
    // sobre toda cópia).
    let plain = sim
        .world_mut()
        .spawn((Transform::IDENTITY, Name::new("Plain")))
        .id();
    let copy2 = ph2d_ecs::deep_copy_subtree(sim.world_mut(), &r, plain, None).expect("copiou");
    assert!(
        clone_owned_documents(
            &mut sim,
            &r,
            &mut OwnedDocs {
                vec_scene: &mut scene,
                vec_entities: &mut map,
            },
            &copy2,
        )
        .dropped
        .is_empty()
    );
}

/// ⭐⭐ **DUPLICAR um grupo com forma vetorial dentro** — o defeito irmão, que existia antes de
/// haver instâncias.
///
/// A row *Duplicate* da Hierarquia roteia uma forma vetorial SOZINHA para a porta do documento
/// (`duplicate_vec_paths`); um **grupo** que a contenha cai na cópia profunda, e as peças
/// vetoriais dele nasciam sem geometria.
///
/// (Mutação: apagar a chamada em `duplicate_subtree` ⇒ RED.)
#[test]
fn duplicating_a_group_gives_its_vector_children_their_own_paths() {
    let mut sim = SimWorld::new();
    let mut scene = VecScene::new();
    let mut map = VecEntityMap::new();
    let r = reg();
    let (group, main_id) = master_with_a_vector_piece(&mut sim, &mut scene, &mut map);
    // Um grupo comum, e não uma receita — é o caminho do *Duplicate*.
    sim.world_mut().entity_mut(group).remove::<MasterRoot>();
    ph2d_ecs::assign_master_pieces(sim.world_mut());

    let copy = crate::instantiate::duplicate_subtree(
        &mut sim,
        &r,
        group,
        &mut OwnedDocs {
            vec_scene: &mut scene,
            vec_entities: &mut map,
        },
        // ⚠️ Degrau ZERO de propósito: o assunto deste gate é a GEOMETRIA do clone, e o
        // deslocamento do objecto na cena é medido por `the_duplicate_lands_beside_its_source`.
        [0.0, 0.0],
    )
    .expect("duplicou");
    let copy_id = sim
        .world()
        .get::<VecPathRef>(piece(&sim, copy, "Plate"))
        .expect("a peca da copia nasceu SEM geometria")
        .0;
    assert_ne!(copy_id, main_id, "as duas escrevem no mesmo path");
    assert_eq!(
        scene.path(main_id).expect("original").verts,
        scene.path(copy_id).expect("copia").verts
    );
}

/// ⭐⭐⭐ **A CENA 2 monta o que ela diz que monta** — as duas peças, em cada uma das três cópias,
/// com geometria **própria** e a **mesma forma** da receita.
///
/// ⛔ É a metade headless do instrumento do §14: a cena imprime este diagnóstico no app, e aqui ele
/// é um gate. *Um smoke que descreve uma coisa e monta outra é pior que não haver smoke.*
///
/// (Mutação: instanciar uma vez só ⇒ RED na contagem; não clonar o documento ⇒ RED em «SEM
/// GEOMETRIA».)
#[test]
fn the_vector_smoke_scene_builds_three_copies_with_their_own_art() {
    let mut sim = SimWorld::new();
    let mut scene = VecScene::new();
    let mut map = VecEntityMap::new();
    let r = reg();
    // ⚠️ **A PORTA da cena**, e não os ingredientes dela: um gate que remontasse as cópias por
    // conta própria ficaria verde sobre uma cena que instancia uma vez só (a mutação que o provou).
    let (master, roots) = crate::instance_smoke::spawn_vector_scene(
        &mut sim,
        &r,
        &mut OwnedDocs {
            vec_scene: &mut scene,
            vec_entities: &mut map,
        },
    );
    assert_eq!(roots.len(), 3, "a cena nao montou TRES copias");
    let mut seen: std::collections::BTreeSet<u64> = std::collections::BTreeSet::new();
    for name in ["Box", "Label"] {
        let m = crate::instance_smoke::piece_path(&sim, master, name)
            .unwrap_or_else(|| panic!("a receita nao tem a peca {name:?} com geometria"));
        seen.insert(m);
        for (i, &root) in roots.iter().enumerate() {
            let id = crate::instance_smoke::piece_path(&sim, root, name).unwrap_or_else(|| {
                panic!("a copia {} nasceu SEM GEOMETRIA na peca {name:?}", i + 1)
            });
            assert!(
                seen.insert(id),
                "a copia {} partilha o path da receita (ou de outra copia) na peca {name:?}",
                i + 1
            );
            assert_eq!(
                scene.path(m).expect("receita").verts,
                scene.path(id).expect("copia").verts,
                "a copia {} nasceu com outra forma na peca {name:?}",
                i + 1
            );
        }
    }
    // 2 peças × (1 receita + 3 cópias) = 8 paths distintos.
    assert_eq!(seen.len(), 8, "paths distintos: {}", seen.len());
}
