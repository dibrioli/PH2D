//! Os gates dos verbos do menu de um cartão ([`super`]).

use super::users_of;
use ph2d_ecs::{ChildOf, MasterRoot, Name, SimWorld, SpritePixels, StableId, Transform};
use ph2d_editor::interaction::drag_payload::DragPayload;
use std::collections::BTreeMap;

/// ⭐⭐⭐ **Uma PEÇA DE RECEITA não é um utilizador** (auditoria de 2026-08-30, achado nº 1).
///
/// ⛔ A varredura crua contava-a — as peças de uma receita carregam `SpritePixels` como qualquer
/// outra — e elas **não estão na cena**: o extract salta-as e a Hierarquia não lhes dá linha.
/// Consequência medida: com uma imagem usada só por uma receita, o *Select users* dizia
/// *«Selected 1 object(s)»* e **nada acendia**. É a espécie *«o consumidor que PROJECTA o valor
/// fora»* do §5.0 — o fio está completo e quem o lê descarta.
///
/// ⚠️ **A fixtura tem o fenómeno de propósito:** a MESMA textura usada por uma peça de receita e
/// por um objecto da cena. Sem o objecto da cena, um filtro que devolvesse sempre vazio passaria.
///
/// **Mutação que deve sangrar:** apagar o `.filter(|e| !is_unedited_recipe(..))`.
#[test]
fn a_recipe_piece_is_not_a_user_because_the_artist_cannot_reach_it() {
    let mut sim = SimWorld::new();
    let db = ph2d_asset::AssetDb::new();
    let tex = db.insert_image_rgba8(2, 2, vec![5u8; 16]);

    // A receita, com uma peça que usa a textura.
    let master = sim
        .world_mut()
        .spawn((Transform::IDENTITY, Name::new("Recipe"), MasterRoot))
        .id();
    let hidden_piece = sim
        .world_mut()
        .spawn((
            Transform::IDENTITY,
            Name::new("RecipePiece"),
            SpritePixels(tex),
            ChildOf(master),
        ))
        .id();
    // E um objecto da CENA que usa a mesma textura.
    let on_canvas = sim
        .world_mut()
        .spawn((
            Transform::IDENTITY,
            Name::new("OnCanvas"),
            SpritePixels(tex),
        ))
        .id();
    ph2d_ecs::assign_missing_stable_ids(sim.world_mut());
    // ⚠️ A marca é DERIVADA — sem este passe a peça da receita não é peça de nada, e o gate mediria
    // um mundo que o produto nunca tem.
    ph2d_ecs::assign_master_pieces(sim.world_mut());

    let users = users_of(
        &mut sim,
        DragPayload::Image {
            asset: *tex.as_bytes(),
        },
        // ⚠️ Mapa VAZIO: estes dois gates medem o caminho do `SpritePixels`, e um mapa cheio faria
        // a resposta certa sair pelo outro braço.
        &BTreeMap::new(),
    );
    assert!(
        users.contains(&on_canvas.to_bits()),
        "o objecto da cena TEM de contar"
    );
    assert!(
        !users.contains(&hidden_piece.to_bits()),
        "a peça da receita não pode contar — o artista não a vê nem a alcança"
    );
    assert_eq!(users.len(), 1, "e mais ninguém: {users:?}");
}

/// ⚠️ **Uma entidade nascida no mesmo quadro ainda não tem `StableId`** — o
/// `assign_missing_stable_ids` corre uma vez por quadro. Exigi-lo na consulta fá-la-ia desaparecer
/// da resposta **em silêncio**, que é o defeito que o `map_or(u64::MAX, …)` evita.
///
/// **Mutação que deve sangrar:** voltar a `query::<(Entity, &SpritePixels, &StableId)>()`.
#[test]
fn an_object_born_this_frame_still_counts_as_a_user() {
    let mut sim = SimWorld::new();
    let db = ph2d_asset::AssetDb::new();
    let tex = db.insert_image_rgba8(2, 2, vec![5u8; 16]);
    let with_id = sim
        .world_mut()
        .spawn((Transform::IDENTITY, SpritePixels(tex)))
        .id();
    ph2d_ecs::assign_missing_stable_ids(sim.world_mut());
    assert!(sim.world().get::<StableId>(with_id).is_some());
    // Este nasce DEPOIS do passe do quadro — é o caso que morde.
    let newborn = sim
        .world_mut()
        .spawn((Transform::IDENTITY, SpritePixels(tex)))
        .id();
    assert!(sim.world().get::<StableId>(newborn).is_none());

    let users = users_of(
        &mut sim,
        DragPayload::Image {
            asset: *tex.as_bytes(),
        },
        // ⚠️ Mapa VAZIO: estes dois gates medem o caminho do `SpritePixels`, e um mapa cheio faria
        // a resposta certa sair pelo outro braço.
        &BTreeMap::new(),
    );
    assert!(
        users.contains(&newborn.to_bits()),
        "o recém-nascido tem de contar: {users:?}"
    );
    assert_eq!(users.len(), 2);
}

/// ⭐⭐⭐ **UMA SPRITE DE ÁTLAS conta como utilizador** — e a 1.ª versão não a via.
///
/// ⛔ O `Select users` perguntava só pelo `SpritePixels`, que é o carimbo da MINORIA: toda imagem
/// importada e todo canvas novo vivem no **átlas**. Consequência: *«Nothing is using this image»*
/// sobre uma imagem que o artista acabou de pôr na cena.
///
/// **Mutação que deve sangrar:** voltar a `query::<(Entity, &SpritePixels)>()`.
#[test]
fn an_atlas_sprite_counts_as_a_user_of_the_image_it_shows() {
    let mut sim = SimWorld::new();
    let db = ph2d_asset::AssetDb::new();
    let tex = db.insert_image_rgba8(2, 2, vec![5u8; 16]);
    let e = sim
        .world_mut()
        .spawn((
            Transform::IDENTITY,
            Name::new("Imported"),
            ph2d_render::Sprite::atlas(7, [1.0, 1.0], [1.0, 1.0, 1.0, 1.0]),
        ))
        .id();
    let mut atlas = BTreeMap::new();
    atlas.insert(7u32, tex);
    let users = users_of(
        &mut sim,
        DragPayload::Image {
            asset: *tex.as_bytes(),
        },
        &atlas,
    );
    assert_eq!(users, vec![e.to_bits()], "a sprite de átlas TEM de contar");
}

/// ⭐⭐⭐ **EDITAR um componente pela biblioteca põe a SELECÇÃO na receita** (report do Enio,
/// 2026-09-05: *«não tem como editar o componente»*).
///
/// ⚠️ **É a selecção que faz a receita existir na tela** — ela não está na cena e volta enquanto
/// está seleccionada (a marca derivada `MasterEditing`). ⇒ este gate mede o **único** efeito que o
/// verbo tem, e é ele que o liga a toda a maquinaria que já existia.
///
/// **Mutação que deve sangrar:** o braço `EditPrefab` não escrever o `select_out` (o item passa a
/// comer o clique e a dizer que editou, sem nada acontecer — a 1.ª espécie de controlo morto).
#[test]
fn editing_a_prefab_from_the_library_selects_the_recipe() {
    let mut sim = SimWorld::new();
    let r = crate::init::build_component_registry();
    let mut echo = crate::instance_sync::MasterEcho::default();
    let mut gizmo = ph2d_editor::screens::hero::GizmoStateGroup::default();
    let mut toasts = ph2d_editor::ToastQueue::default();
    let (mut sc, mut mp) = crate::instance_docs::empty_docs();
    let mut docs = crate::instance_docs::OwnedDocs {
        vec_scene: &mut sc,
        vec_entities: &mut mp,
    };

    let master = sim
        .world_mut()
        .spawn((Transform::IDENTITY, Name::new("Badge"), MasterRoot))
        .id();
    ph2d_ecs::assign_missing_stable_ids(sim.world_mut());
    let stable_id = sim.world().get::<StableId>(master).expect("id").0;

    let mut select_out = None;
    let acted = super::drain(
        DragPayload::Prefab { stable_id },
        ph2d_editor::action_bus::AssetCardAction::EditPrefab,
        &mut sim,
        &r,
        &mut echo,
        &mut gizmo,
        &mut toasts,
        &mut docs,
        [0.0, 0.0],
        &BTreeMap::new(),
        &mut select_out,
    );
    assert!(acted, "o verbo nao agiu");
    assert_eq!(
        select_out,
        Some(master.to_bits()),
        "a seleccao nao foi para a receita — o item come o clique e nada aparece"
    );
}

/// ⛔ **Uma IMAGEM recusa com o FACTO, e a recusa FALA** — a tabela do menu é plana, e um item que
/// come o clique em silêncio é pior que um ausente (a lei das outras três recusas deste ficheiro).
#[test]
fn editing_an_image_is_refused_out_loud() {
    let mut sim = SimWorld::new();
    let r = crate::init::build_component_registry();
    let mut echo = crate::instance_sync::MasterEcho::default();
    let mut gizmo = ph2d_editor::screens::hero::GizmoStateGroup::default();
    let mut toasts = ph2d_editor::ToastQueue::default();
    let (mut sc, mut mp) = crate::instance_docs::empty_docs();
    let mut docs = crate::instance_docs::OwnedDocs {
        vec_scene: &mut sc,
        vec_entities: &mut mp,
    };

    let mut select_out = None;
    let acted = super::drain(
        DragPayload::Image { asset: [7; 32] },
        ph2d_editor::action_bus::AssetCardAction::EditPrefab,
        &mut sim,
        &r,
        &mut echo,
        &mut gizmo,
        &mut toasts,
        &mut docs,
        [0.0, 0.0],
        &BTreeMap::new(),
        &mut select_out,
    );
    assert!(!acted);
    assert!(
        select_out.is_none(),
        "uma imagem nao tem receita a seleccionar"
    );
    assert!(
        !toasts.is_empty(),
        "a recusa foi MUDA — o artista carrega e conclui que o app esta' partido"
    );
}
